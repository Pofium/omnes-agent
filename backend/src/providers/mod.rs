//! ModelProvider subsystem — re-exported from `omnesagent-providers`.

pub use omnesagent_providers::*;

// Keep traits.rs as a file module so its #[cfg(test)] block compiles.
#[path = "traits.rs"]
pub mod traits;
