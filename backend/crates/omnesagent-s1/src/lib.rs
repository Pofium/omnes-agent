//! OmnesAgent System One (S1) Layer.
//!
//! Provides fast, local, typed decisions (Choice, Score, Noul)
//! powered by the Laya model (ModernBERT-large + Decision Head).

pub mod config;
pub mod contract;
pub mod fake;
pub mod log;
pub mod provider;

#[cfg(feature = "candle")]
pub mod laya;

pub use config::{
    DeviceKind, ProviderKind, QuantizationKind, SystemOneConfig, create_system_one,
};
pub use contract::{Answer, Question};
pub use fake::FakeSystemOne;
pub use log::log_decision;
pub use provider::{NoOpSystemOne, SharedSystemOne, SystemOne};

#[cfg(feature = "candle")]
pub use laya::CandleLayaProvider;
