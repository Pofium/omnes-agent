//! Векторная математика и сериализация (BLOB float32/int8, cosine, RRF).

pub mod rrf;
pub mod simd;
pub mod similarity;
#[cfg(feature = "vec0")]
pub mod vec0;

pub use rrf::{rrf_merge, RankedItem};
pub use simd::{dot, norm_sq};
pub use similarity::{cosine, deserialize, normalize, serialize, top_k};
