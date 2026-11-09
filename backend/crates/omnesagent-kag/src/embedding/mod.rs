//! Провайдеры векторных представлений (Embeddings).

pub mod fake;
#[cfg(feature = "candle")]
pub mod local_bert;

use async_trait::async_trait;
use lru::LruCache;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub use fake::FakeEmbedding;
#[cfg(feature = "candle")]
pub use local_bert::LocalBertEmbedding;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Получить эмбеддинги для списка текстов (батч).
    async fn embed(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>>;

    /// Размерность вектора.
    fn dim(&self) -> usize;
}

/// Ф62.2: нормализация текста в ключ кэша (trim + lower-case).
fn cache_key(text: &str) -> String {
    text.trim().to_lowercase()
}

/// Ф62.2: потокобезопасный LRU-кэш эмбеддингов запросов. Повторные запросы
/// агентов получают вектор из памяти (~0 мс) без вызова модели.
pub struct CachedEmbeddingProvider {
    inner: Arc<dyn EmbeddingProvider>,
    cache: Mutex<LruCache<String, Vec<f32>>>,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl CachedEmbeddingProvider {
    /// Ёмкость кэша по умолчанию — 1000 записей (Ф62.2).
    pub const DEFAULT_CAPACITY: usize = 1000;

    pub fn new(inner: Arc<dyn EmbeddingProvider>) -> Self {
        Self::with_capacity(inner, Self::DEFAULT_CAPACITY)
    }

    pub fn with_capacity(inner: Arc<dyn EmbeddingProvider>, capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity.max(1)).expect("capacity > 0");
        Self {
            inner,
            cache: Mutex::new(LruCache::new(cap)),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    /// Текущее число записей в кэше (для тестов/статистики).
    pub fn cache_len(&self) -> usize {
        self.cache.lock().len()
    }

    /// (попадания, промахи) — для диагностики.
    pub fn stats(&self) -> (u64, u64) {
        (
            self.hits.load(Ordering::Relaxed),
            self.misses.load(Ordering::Relaxed),
        )
    }
}

#[async_trait]
impl EmbeddingProvider for CachedEmbeddingProvider {
    async fn embed(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let keys: Vec<String> = texts.iter().map(|t| cache_key(t)).collect();
        // Слоты результата: Some(вектор) — из кэша, None — нужно посчитать.
        let mut slots: Vec<Option<Vec<f32>>> = vec![None; texts.len()];
        let mut miss_idx: Vec<usize> = Vec::new();
        {
            let mut cache = self.cache.lock();
            for (i, key) in keys.iter().enumerate() {
                match cache.get(key) {
                    Some(vec) => {
                        slots[i] = Some(vec.clone());
                        self.hits.fetch_add(1, Ordering::Relaxed);
                    }
                    None => {
                        miss_idx.push(i);
                        self.misses.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }

        if !miss_idx.is_empty() {
            let miss_texts: Vec<String> = miss_idx.iter().map(|&i| texts[i].clone()).collect();
            let fresh = self.inner.embed(&miss_texts).await?;
            if fresh.len() != miss_idx.len() {
                anyhow::bail!(
                    "embedding provider вернул {} векторов на {} текстов",
                    fresh.len(),
                    miss_idx.len()
                );
            }
            let mut cache = self.cache.lock();
            for (slot_i, vec) in miss_idx.iter().zip(fresh) {
                cache.put(keys[*slot_i].clone(), vec.clone());
                slots[*slot_i] = Some(vec);
            }
        }

        slots
            .into_iter()
            .map(|s| s.ok_or_else(|| anyhow::anyhow!("embedding slot not filled")))
            .collect()
    }

    fn dim(&self) -> usize {
        self.inner.dim()
    }
}

/// Фабрика провайдера эмбеддингов по умолчанию (с LRU-кэшем запросов, Ф62.2).
pub fn default_embedding_provider() -> Arc<dyn EmbeddingProvider> {
    let inner: Arc<dyn EmbeddingProvider> = {
        #[cfg(feature = "candle")]
        {
            match LocalBertEmbedding::new("local") {
                Ok(local) => Arc::new(local),
                Err(e) => {
                    tracing::warn!(
                        "Не удалось инициализировать Candle embedder ({e}), используется FakeEmbedding"
                    );
                    Arc::new(FakeEmbedding::new(384))
                }
            }
        }
        #[cfg(not(feature = "candle"))]
        {
            Arc::new(FakeEmbedding::new(384))
        }
    };
    // Ф62.2: кэш эмбеддингов запросов поверх любого бэкенда.
    Arc::new(CachedEmbeddingProvider::new(inner))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Считающий провайдер: учитывает число фактически посчитанных текстов.
    struct CountingEmbedding {
        dim: usize,
        calls: AtomicU64,
    }

    impl CountingEmbedding {
        fn new(dim: usize) -> Self {
            Self {
                dim,
                calls: AtomicU64::new(0),
            }
        }
        fn calls(&self) -> u64 {
            self.calls.load(Ordering::Relaxed)
        }
    }

    #[async_trait]
    impl EmbeddingProvider for CountingEmbedding {
        async fn embed(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
            self.calls.fetch_add(texts.len() as u64, Ordering::Relaxed);
            Ok(texts.iter().map(|_| vec![0.5_f32; self.dim]).collect())
        }
        fn dim(&self) -> usize {
            self.dim
        }
    }

    #[tokio::test]
    async fn empty_input_bypasses_backend() {
        let inner = Arc::new(CountingEmbedding::new(8));
        let cached = CachedEmbeddingProvider::new(inner.clone());
        let out = cached.embed(&[]).await.unwrap();
        assert!(out.is_empty());
        assert_eq!(inner.calls(), 0, "пустой вход не должен вызывать бэкенд");
    }

    #[tokio::test]
    async fn repeated_query_is_served_from_cache() {
        let inner = Arc::new(CountingEmbedding::new(8));
        let cached = CachedEmbeddingProvider::new(inner.clone());

        let first = cached.embed(&["Привет мир".to_string()]).await.unwrap();
        let second = cached.embed(&["Привет мир".to_string()]).await.unwrap();

        assert_eq!(first, second);
        assert_eq!(inner.calls(), 1, "повторный запрос берётся из кэша");
        assert_eq!(cached.cache_len(), 1);
        assert_eq!(cached.stats(), (1, 1));
    }

    #[tokio::test]
    async fn cache_key_normalizes_case_and_whitespace() {
        let inner = Arc::new(CountingEmbedding::new(4));
        let cached = CachedEmbeddingProvider::new(inner.clone());
        cached.embed(&["  Hello World ".to_string()]).await.unwrap();
        cached.embed(&["hello world".to_string()]).await.unwrap();
        assert_eq!(inner.calls(), 1, "trim+lowercase дают один ключ");
    }

    #[tokio::test]
    async fn batch_with_partial_hits_only_embeds_misses() {
        let inner = Arc::new(CountingEmbedding::new(4));
        let cached = CachedEmbeddingProvider::new(inner.clone());
        cached.embed(&["alpha".to_string()]).await.unwrap();
        assert_eq!(inner.calls(), 1);

        let out = cached
            .embed(&["alpha".to_string(), "beta".to_string()])
            .await
            .unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(inner.calls(), 2, "только промах 'beta' уходит в бэкенд");
        assert_eq!(cached.cache_len(), 2);
    }

    #[tokio::test]
    async fn lru_evicts_oldest_beyond_capacity() {
        let inner = Arc::new(CountingEmbedding::new(4));
        let cached = CachedEmbeddingProvider::with_capacity(inner.clone(), 2);
        cached.embed(&["a".to_string(), "b".to_string()]).await.unwrap();
        cached.embed(&["c".to_string()]).await.unwrap();
        assert_eq!(cached.cache_len(), 2, "ёмкость LRU ограничена");

        // "a" вытеснен -> повторный запрос снова идёт в бэкенд.
        cached.embed(&["a".to_string()]).await.unwrap();
        assert_eq!(inner.calls(), 4);
    }

    #[tokio::test]
    async fn dim_delegates_to_inner() {
        let inner = Arc::new(CountingEmbedding::new(16));
        let cached = CachedEmbeddingProvider::new(inner);
        assert_eq!(cached.dim(), 16);
    }
}
