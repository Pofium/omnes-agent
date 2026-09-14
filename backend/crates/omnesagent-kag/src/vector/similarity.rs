//! Сериализация BLOB (f32 legacy + int8 v2) и косинусный поиск.

/// Магический байт формата v2: [0x01][scale f32 LE][i8 × dim] (Ф24 PLAN_v1.3).
/// Длина v2 = dim + 5 (для 384d = 389 байт вместо 1536 байт, сжатие ~75%).
pub const Q_MAGIC: u8 = 0x01;

/// Сериализация вектора f32 в бинарный BLOB (little-endian). Легаси-формат;
/// новые записи должны использовать serialize_q (int8, ~4× компактнее).
pub fn serialize(vec: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vec.len() * 4);
    for &f in vec {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}

/// Квантование int8 per-vector scale: scale = max|v| / 127, i8 = round(v / scale).
pub fn serialize_q(vec: &[f32]) -> Vec<u8> {
    let max_abs = vec.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
    let scale = if max_abs == 0.0 { 1.0 } else { max_abs / 127.0 };
    let mut out = Vec::with_capacity(vec.len() + 5);
    out.push(Q_MAGIC);
    out.extend_from_slice(&scale.to_le_bytes());
    for &v in vec {
        let q = (v / scale).round().clamp(-127.0, 127.0) as i8;
        out.push(q as u8);
    }
    out
}

/// Десериализация легаси f32 BLOB.
pub fn deserialize_f32(blob: &[u8]) -> Option<Vec<f32>> {
    if blob.is_empty() || !blob.len().is_multiple_of(4) {
        return None;
    }
    let floats: Vec<f32> = blob
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes(c.try_into().expect("valid 4-byte chunk")))
        .collect();
    Some(floats)
}

/// Десериализация v2 (int8 + scale).
pub fn deserialize_q(blob: &[u8]) -> Option<Vec<f32>> {
    if blob.len() < 5 || blob[0] != Q_MAGIC {
        return None;
    }
    let scale = f32::from_le_bytes(blob[1..5].try_into().ok()?);
    Some(blob[5..].iter().map(|&b| (b as i8) as f32 * scale).collect())
}

/// Dual-read: v2 по magic-байту, иначе легаси f32.
pub fn deserialize(blob: &[u8]) -> Option<Vec<f32>> {
    if blob.first() == Some(&Q_MAGIC) {
        deserialize_q(blob)
    } else {
        deserialize_f32(blob)
    }
}

/// Нормализация вектора L2.
pub fn normalize(vec: &[f32]) -> Vec<f32> {
    let norm_sq: f32 = vec.iter().map(|v| v * v).sum();
    let norm = norm_sq.sqrt();
    if norm == 0.0 {
        return vec.to_vec();
    }
    vec.iter().map(|v| v / norm).collect()
}

/// Косинусное сходство между двумя векторами.
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut norm_a_sq = 0.0f32;
    let mut norm_b_sq = 0.0f32;

    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a_sq += x * x;
        norm_b_sq += y * y;
    }

    let denom = norm_a_sq.sqrt() * norm_b_sq.sqrt();
    if denom == 0.0 {
        0.0
    } else {
        dot / denom
    }
}

/// Поиск top_k ближайших кандидатов по косинусному сходству.
pub fn top_k(
    query: &[f32],
    candidates: &[(i64, Option<&[u8]>)],
    k: usize,
    min_score: f32,
) -> Vec<(i64, f32)> {
    let mut scored: Vec<(i64, f32)> = Vec::new();

    for &(id, blob_opt) in candidates {
        if let Some(blob) = blob_opt
            && let Some(vec) = deserialize(blob)
                && vec.len() == query.len() {
                    let score = cosine(query, &vec);
                    if score >= min_score {
                        scored.push((id, score));
                    }
                }
    }

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    if scored.len() > k {
        scored.truncate(k);
    }
    scored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize() {
        let original = vec![0.1f32, -0.5, 1.25, 0.0];
        let bytes = serialize(&original);
        assert_eq!(bytes.len(), 16);
        let restored = deserialize(&bytes).expect("must deserialize");
        assert_eq!(original, restored);
    }

    #[test]
    fn test_cosine() {
        let a = vec![1.0f32, 0.0, 0.0];
        let b = vec![1.0f32, 0.0, 0.0];
        let c = vec![0.0f32, 1.0, 0.0];
        assert!((cosine(&a, &b) - 1.0).abs() < 1e-5);
        assert!((cosine(&a, &c) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn deserialize_rejects_empty_and_ragged_blobs() {
        assert_eq!(deserialize(&[]), None);
        assert_eq!(deserialize(&[1, 2, 3]), None, "3 bytes is not a f32");
        assert_eq!(deserialize(&[0; 7]), None, "7 bytes is not a f32 multiple");
        assert_eq!(deserialize(&serialize(&[7.5f32])), Some(vec![7.5]));
    }

    #[test]
    fn normalize_returns_unit_vector_and_passes_zero_through() {
        let n = normalize(&[3.0f32, 4.0]);
        assert!((n[0] - 0.6).abs() < 1e-6);
        assert!((n[1] - 0.8).abs() < 1e-6);
        assert!((normalize(&n).iter().map(|v| v * v).sum::<f32>() - 1.0).abs() < 1e-5);

        let zero = vec![0.0f32, 0.0, 0.0];
        assert_eq!(normalize(&zero), zero, "zero vector must not divide by 0");
        assert_eq!(normalize(&[]), Vec::<f32>::new());
    }

    #[test]
    fn cosine_guards_length_mismatch_empty_and_zero_vectors() {
        assert_eq!(cosine(&[1.0f32], &[1.0, 2.0]), 0.0, "length mismatch");
        assert_eq!(cosine(&[], &[]), 0.0, "both empty");
        assert_eq!(cosine(&[0.0f32, 0.0], &[1.0, 1.0]), 0.0, "zero norm denominator");
        let a = vec![1.0f32, 0.0];
        let anti = vec![-1.0f32, 0.0];
        assert!((cosine(&a, &anti) + 1.0).abs() < 1e-5, "opposite vectors");
    }

    #[test]
    fn top_k_filters_skips_and_truncates_deterministically() {
        let query = vec![1.0f32, 0.0];
        let good = serialize(&[1.0f32, 0.0]);
        let weak = serialize(&[0.1f32, 0.9]);
        let wrong_dim = serialize(&[1.0f32, 0.0, 0.0]);
        let candidates: Vec<(i64, Option<&[u8]>)> = vec![
            (1, Some(&good)),
            (2, Some(&weak)),      // below min_score
            (3, None),             // NULL embedding blob
            (4, Some(&wrong_dim)), // dimension mismatch
            (5, Some(&good)),      // ties with id 1
        ];

        let hits = top_k(&query, &candidates, 10, 0.5);
        assert_eq!(hits.len(), 2, "only min_score-passing dims match");
        assert_eq!(hits[0].0, 1);
        assert!((hits[0].1 - 1.0).abs() < 1e-5);
        assert_eq!(hits[1].0, 5);

        let capped = top_k(&query, &candidates, 1, 0.0);
        assert_eq!(capped.len(), 1, "k must truncate");
        assert_eq!(capped[0].0, 1, "sort is descending by score");
    }

    #[test]
    fn quantize_int8_roundtrip_and_cosine_preservation() {
        let v1 = vec![0.1f32, -0.4, 0.8, -0.2, 0.05, 0.95];
        let v2 = vec![0.12f32, -0.38, 0.79, -0.18, 0.04, 0.91];
        let orig_cos = cosine(&v1, &v2);

        let q1 = serialize_q(&v1);
        let q2 = serialize_q(&v2);
        assert_eq!(q1[0], Q_MAGIC);
        assert_eq!(q1.len(), v1.len() + 5);

        let d1 = deserialize(&q1).expect("deserialize quantized 1");
        let d2 = deserialize(&q2).expect("deserialize quantized 2");
        let quant_cos = cosine(&d1, &d2);

        assert!((orig_cos - quant_cos).abs() < 0.005, "cosine difference must be < 0.005");
    }
}
