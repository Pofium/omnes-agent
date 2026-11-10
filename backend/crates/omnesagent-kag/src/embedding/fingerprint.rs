//! Ф6.2 (ob2h v2.0 / Ф72): идентификатор векторного пространства
//! (`embedding_fingerprint`) и защита от смешивания несовместимых пространств
//! в одном хранилище.
//!
//! Два вектора можно сравнивать косинусом только если они из одного
//! пространства: та же модель и та же размерность. Отпечаток фиксирует это в
//! таблице `kv`; при смене модели запись новых векторов в то же хранилище
//! запрещена — требуется переиндексация (`ProjectService::reindex_*`).

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::EmbeddingProvider;

/// Ключ в таблице `kv`, где хранится отпечаток активного векторного пространства.
pub const KV_KEY: &str = "embedding_fingerprint";

/// Отпечаток векторного пространства: идентификатор модели + размерность.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingFingerprint {
    pub model_id: String,
    pub dim: usize,
}

impl EmbeddingFingerprint {
    pub fn new(model_id: impl Into<String>, dim: usize) -> Self {
        Self {
            model_id: model_id.into(),
            dim,
        }
    }

    /// Отпечаток активного провайдера.
    pub fn of(provider: &dyn EmbeddingProvider) -> Self {
        Self::new(provider.model_id(), provider.dim())
    }

    /// Стабильный короткий ключ (sha256 hex[..16]) — для логов/диагностики.
    pub fn key(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}|{}", self.model_id, self.dim).as_bytes());
        hex::encode(hasher.finalize())[..16].to_string()
    }
}

/// Результат сверки активного провайдера с сохранённым отпечатком.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FingerprintStatus {
    /// Отпечаток ещё не сохранён (первый запуск или легаси-база без отпечатка).
    Fresh,
    /// Совпадает — этим провайдером можно и писать, и читать вектора.
    Matched,
    /// Не совпадает — запись смешала бы два несовместимых пространства.
    Mismatch { stored: EmbeddingFingerprint },
}

impl FingerprintStatus {
    /// Разрешена ли запись эмбеддингов активным провайдером.
    pub fn allows_write(&self) -> bool {
        !matches!(self, Self::Mismatch { .. })
    }
}

/// Читает сохранённый отпечаток (`None`, если не сохранён или повреждён).
pub fn load(conn: &Connection) -> Option<EmbeddingFingerprint> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT value FROM kv WHERE key = ?1",
            params![KV_KEY],
            |r| r.get(0),
        )
        .ok();
    raw.and_then(|s| serde_json::from_str(&s).ok())
}

/// Сохраняет отпечаток активного пространства.
pub fn store(conn: &Connection, fp: &EmbeddingFingerprint) -> rusqlite::Result<()> {
    let json = serde_json::to_string(fp)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    conn.execute(
        "INSERT OR REPLACE INTO kv (key, value) VALUES (?1, ?2)",
        params![KV_KEY, json],
    )?;
    Ok(())
}

/// Сверяет активный отпечаток с сохранённым (без записи).
pub fn check(conn: &Connection, current: &EmbeddingFingerprint) -> rusqlite::Result<FingerprintStatus> {
    Ok(match load(conn) {
        None => FingerprintStatus::Fresh,
        Some(stored) if &stored == current => FingerprintStatus::Matched,
        Some(stored) => FingerprintStatus::Mismatch { stored },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::FakeEmbedding;
    use crate::schema::migrate;

    fn conn() -> Connection {
        let c = Connection::open_in_memory().expect("open in-memory sqlite");
        migrate(&c).expect("migrate");
        c
    }

    #[test]
    fn of_reads_provider_identity() {
        let fp = EmbeddingFingerprint::of(&FakeEmbedding::new(384));
        assert_eq!(fp.model_id, "fake");
        assert_eq!(fp.dim, 384);
    }

    #[test]
    fn key_is_stable_and_changes_with_model_or_dim() {
        let a = EmbeddingFingerprint::new("model-a", 384);
        assert_eq!(a.key(), a.clone().key(), "same input, same key");
        assert_ne!(a.key(), EmbeddingFingerprint::new("model-b", 384).key());
        assert_ne!(a.key(), EmbeddingFingerprint::new("model-a", 768).key());
        assert_eq!(a.key().len(), 16);
    }

    #[test]
    fn store_load_check_roundtrip_classifies_all_states() {
        let c = conn();
        let a = EmbeddingFingerprint::new("model-a", 384);
        assert_eq!(check(&c, &a).unwrap(), FingerprintStatus::Fresh);

        store(&c, &a).unwrap();
        assert_eq!(load(&c), Some(a.clone()));
        assert_eq!(check(&c, &a).unwrap(), FingerprintStatus::Matched);
        assert!(check(&c, &a).unwrap().allows_write());

        let b = EmbeddingFingerprint::new("model-b", 384);
        assert_eq!(
            check(&c, &b).unwrap(),
            FingerprintStatus::Mismatch { stored: a }
        );
        assert!(!check(&c, &b).unwrap().allows_write(), "mismatch forbids writes");
    }

    #[test]
    fn load_tolerates_absent_or_garbage_marker() {
        let c = conn();
        assert_eq!(load(&c), None, "no marker yet");
        c.execute(
            "INSERT INTO kv (key, value) VALUES (?1, 'not-json')",
            params![KV_KEY],
        )
        .unwrap();
        assert_eq!(load(&c), None, "corrupt marker degrades to Fresh");
    }
}
