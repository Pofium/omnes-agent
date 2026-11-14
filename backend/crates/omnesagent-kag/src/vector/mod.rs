//! Векторная математика и сериализация (BLOB float32/int8, cosine, RRF, BQ).

pub mod bq;
pub mod rrf;
pub mod simd;
pub mod similarity;
#[cfg(feature = "vec0")]
pub mod vec0;

pub use bq::{
    compute_mean_centered_bq_mask, hamming_distance, hamming_distance_bytes, two_stage_rescore,
    unpack_bq_mask, BqCandidate,
};
pub use rrf::{rrf_merge, RankedItem};
pub use simd::{dot, norm_sq};
pub use similarity::{cosine, deserialize, normalize, serialize, top_k};
