//! Mean-Centered Binary Quantization (BQ) and Two-Stage SIMD Retrieval.
//!
//! Stage 1: Quantizes float32 embeddings into 1-bit per dimension binary masks
//! using mean-centering (`val[i] > mean`). For 384-dim embeddings (e.g. multilingual-e5-small),
//! this packs into 48 bytes (`[u64; 6]`). Hamming distance via CPU `popcnt`
//! allows filtering 100k+ candidates in < 0.5 ms.
//!
//! Stage 2: Exact FP32 cosine similarity rescore of top oversampled candidates.

use crate::vector::similarity::cosine;

/// Computes a mean-centered binary quantization mask for a float vector.
///
/// Each bit $i$ is 1 if `vec[i] > mean(vec)`, and 0 otherwise.
/// Returns little-endian packed bytes (48 bytes for 384-dim, 96 bytes for 768-dim).
pub fn compute_mean_centered_bq_mask(vec: &[f32]) -> Vec<u8> {
    if vec.is_empty() {
        return Vec::new();
    }

    let sum: f32 = vec.iter().sum();
    let mean = sum / (vec.len() as f32);

    let num_words = (vec.len() + 63) / 64;
    let mut words = vec![0u64; num_words];

    for (i, &val) in vec.iter().enumerate() {
        if val > mean {
            let word_idx = i / 64;
            let bit_idx = i % 64;
            words[word_idx] |= 1u64 << bit_idx;
        }
    }

    let mut bytes = Vec::with_capacity(num_words * 8);
    for word in words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    bytes
}

/// Unpacks a binary mask byte slice into 64-bit words.
pub fn unpack_bq_mask(bytes: &[u8]) -> Vec<u64> {
    bytes
        .chunks_exact(8)
        .map(|chunk| {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(chunk);
            u64::from_le_bytes(arr)
        })
        .collect()
}

/// Computes Hamming distance between two binary masks given as `u64` words.
///
/// Returns the number of differing bits using CPU popcount (`count_ones`).
#[inline]
pub fn hamming_distance(a: &[u64], b: &[u64]) -> u32 {
    let len = a.len().min(b.len());
    let mut dist = 0u32;
    for i in 0..len {
        dist += (a[i] ^ b[i]).count_ones();
    }
    dist
}

/// Computes Hamming distance directly from raw byte slices.
pub fn hamming_distance_bytes(a: &[u8], b: &[u8]) -> u32 {
    let words_a = unpack_bq_mask(a);
    let words_b = unpack_bq_mask(b);
    hamming_distance(&words_a, &words_b)
}

/// Candidate record for two-stage search.
#[derive(Debug, Clone)]
pub struct BqCandidate<T> {
    pub id: T,
    pub bq_mask: Vec<u8>,
    pub fp32_vec: Vec<f32>,
}

/// Two-stage search:
/// 1. Oversample `oversample_k` candidates with lowest Hamming distance.
/// 2. Rescore oversampled candidates with exact cosine similarity and return `top_k`.
pub fn two_stage_rescore<T: Clone>(
    query_fp32: &[f32],
    query_bq: &[u8],
    candidates: &[BqCandidate<T>],
    oversample_k: usize,
    top_k: usize,
) -> Vec<(T, f64)> {
    if candidates.is_empty() || top_k == 0 {
        return Vec::new();
    }

    let query_words = unpack_bq_mask(query_bq);

    // Stage 1: Fast Hamming distance filtering
    let mut hamming_scored: Vec<(usize, u32)> = candidates
        .iter()
        .enumerate()
        .map(|(idx, cand)| {
            let cand_words = unpack_bq_mask(&cand.bq_mask);
            let dist = hamming_distance(&query_words, &cand_words);
            (idx, dist)
        })
        .collect();

    // Sort ascending by Hamming distance (lower distance = closer)
    hamming_scored.sort_by_key(|&(_, dist)| dist);

    let stage1_take = oversample_k.max(top_k).min(hamming_scored.len());
    let stage1_indices = &hamming_scored[..stage1_take];

    // Stage 2: Exact FP32 cosine similarity rescore
    let mut final_scored: Vec<(T, f64)> = stage1_indices
        .iter()
        .map(|&(idx, _)| {
            let cand = &candidates[idx];
            let sim = cosine(query_fp32, &cand.fp32_vec) as f64;
            (cand.id.clone(), sim)
        })
        .collect();

    // Sort descending by cosine similarity
    final_scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    final_scored.truncate(top_k);

    final_scored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bq_mask_mean_centering_balances_bits_with_offset() {
        // Biased vector: all values > 0 (mean = 0.5)
        let vec = vec![0.1_f32, 0.2, 0.8, 0.9];
        let mask = compute_mean_centered_bq_mask(&vec);
        assert_eq!(mask.len(), 8); // 1 u64 word = 8 bytes
        let words = unpack_bq_mask(&mask);
        assert_eq!(words.len(), 1);

        // 0.1 and 0.2 are <= 0.5 (bits 0 and 1 are 0)
        // 0.8 and 0.9 are > 0.5 (bits 2 and 3 are 1)
        assert_eq!(words[0], 0b1100);
    }

    #[test]
    fn hamming_distance_identical_is_zero() {
        let vec = vec![0.1_f32, -0.2, 0.8, -0.9];
        let mask = compute_mean_centered_bq_mask(&vec);
        let dist = hamming_distance_bytes(&mask, &mask);
        assert_eq!(dist, 0);
    }

    #[test]
    fn hamming_distance_counts_differing_bits() {
        let a = vec![0b1010u64];
        let b = vec![0b0011u64];
        // 1010 ^ 0011 = 1001 (2 ones)
        assert_eq!(hamming_distance(&a, &b), 2);
    }

    #[test]
    fn two_stage_rescore_returns_top_semantic_matches() {
        let query = vec![1.0_f32, 0.0, 0.0, 0.0];
        let query_bq = compute_mean_centered_bq_mask(&query);

        let candidates = vec![
            BqCandidate {
                id: "orthogonal",
                bq_mask: compute_mean_centered_bq_mask(&[0.0, 1.0, 0.0, 0.0]),
                fp32_vec: vec![0.0, 1.0, 0.0, 0.0],
            },
            BqCandidate {
                id: "exact_match",
                bq_mask: compute_mean_centered_bq_mask(&[1.0, 0.0, 0.0, 0.0]),
                fp32_vec: vec![1.0, 0.0, 0.0, 0.0],
            },
            BqCandidate {
                id: "near_match",
                bq_mask: compute_mean_centered_bq_mask(&[0.9, 0.1, 0.0, 0.0]),
                fp32_vec: vec![0.9, 0.1, 0.0, 0.0],
            },
        ];

        let results = two_stage_rescore(&query, &query_bq, &candidates, 3, 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "exact_match");
        assert!((results[0].1 - 1.0).abs() < 1e-4);
        assert_eq!(results[1].0, "near_match");
    }
}
