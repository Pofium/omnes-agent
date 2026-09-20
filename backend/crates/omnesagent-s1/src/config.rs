//! Configuration parser and provider factory for System One.

use std::path::PathBuf;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::provider::{NoOpSystemOne, SharedSystemOne};
use crate::fake::FakeSystemOne;

/// Supported System One provider backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    #[default]
    Off,
    Laya,
    Fake,
}

impl ProviderKind {
    /// Parses provider kind from string, falling back to Off for unknown values.
    #[must_use]
    pub fn from_str_lenient(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "laya" => Self::Laya,
            "fake" | "stub" => Self::Fake,
            _ => Self::Off,
        }
    }
}

/// Compute device target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    #[default]
    Cpu,
    Cuda,
}

impl DeviceKind {
    #[must_use]
    pub fn from_str_lenient(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "cuda" | "gpu" => Self::Cuda,
            _ => Self::Cpu,
        }
    }
}

/// Quantization format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QuantizationKind {
    #[default]
    None,
    Q8,
}

impl QuantizationKind {
    #[must_use]
    pub fn from_str_lenient(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "q8" | "int8" => Self::Q8,
            _ => Self::None,
        }
    }
}

/// Consolidated configuration for System One inference and integrations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOneConfig {
    pub provider: ProviderKind,
    pub model_path: Option<PathBuf>,
    pub device: DeviceKind,
    pub quantization: QuantizationKind,
    pub max_tokens: usize,
    pub log_enabled: bool,
    pub channel_intent_enabled: bool,
    pub compaction_enabled: bool,
    pub memory_rerank_enabled: bool,
    pub tool_gate_enabled: bool,
    pub router_enabled: bool,
}

impl Default for SystemOneConfig {
    fn default() -> Self {
        Self {
            provider: ProviderKind::Off,
            model_path: None,
            device: DeviceKind::Cpu,
            quantization: QuantizationKind::None,
            max_tokens: 512,
            log_enabled: false,
            channel_intent_enabled: false,
            compaction_enabled: false,
            memory_rerank_enabled: false,
            tool_gate_enabled: false,
            router_enabled: false,
        }
    }
}

impl SystemOneConfig {
    /// Loads configuration from environment variables, checking both OMNESAGENT_S1_* and OMNES_S1_*.
    #[must_use]
    pub fn from_env() -> Self {
        let get_var = |key: &str| -> Option<String> {
            std::env::var(format!("OMNESAGENT_S1_{key}"))
                .or_else(|_| std::env::var(format!("OMNES_S1_{key}")))
                .ok()
        };

        let get_bool = |key: &str| -> bool {
            get_var(key).map(|v| {
                let v = v.trim().to_lowercase();
                v == "1" || v == "true" || v == "yes" || v == "on"
            }).unwrap_or(false)
        };

        let provider = get_var("PROVIDER")
            .map(|s| ProviderKind::from_str_lenient(&s))
            .unwrap_or(ProviderKind::Off);

        let model_path = get_var("PATH").map(PathBuf::from);

        let device = get_var("DEVICE")
            .map(|s| DeviceKind::from_str_lenient(&s))
            .unwrap_or(DeviceKind::Cpu);

        let quantization = get_var("QUANT")
            .map(|s| QuantizationKind::from_str_lenient(&s))
            .unwrap_or(QuantizationKind::None);

        let max_tokens = get_var("MAX_TOKENS")
            .and_then(|s| s.trim().parse::<usize>().ok())
            .unwrap_or(512)
            .max(64);

        let log_enabled = get_bool("LOG");
        let channel_intent_enabled = get_bool("CHANNEL_INTENT");
        let compaction_enabled = get_bool("COMPACTION");
        let memory_rerank_enabled = get_bool("MEMORY_RERANK");
        let tool_gate_enabled = get_bool("TOOL_GATE");
        let router_enabled = get_bool("ROUTER");

        Self {
            provider,
            model_path,
            device,
            quantization,
            max_tokens,
            log_enabled,
            channel_intent_enabled,
            compaction_enabled,
            memory_rerank_enabled,
            tool_gate_enabled,
            router_enabled,
        }
    }
}

/// Builds an initialized System One provider based on config.
/// Falls back safely to NoOpSystemOne on any initialization failure.
#[must_use]
pub fn create_system_one(config: &SystemOneConfig) -> SharedSystemOne {
    match config.provider {
        ProviderKind::Off => Arc::new(NoOpSystemOne),
        ProviderKind::Fake => Arc::new(FakeSystemOne::new()),
        ProviderKind::Laya => {
            #[cfg(feature = "candle")]
            {
                match crate::laya::CandleLayaProvider::new(config) {
                    Ok(laya) => Arc::new(laya),
                    Err(e) => {
                        ::omnesagent_log::record!(
                            WARN,
                            ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Fail)
                                .with_category(::omnesagent_log::EventCategory::System)
                                .with_outcome(::omnesagent_log::EventOutcome::Failure)
                                .with_attrs(::serde_json::json!({
                                    "error": format!("{:#}", e),
                                    "fallback": "NoOpSystemOne"
                                })),
                            "Failed to initialize CandleLayaProvider, falling back to NoOpSystemOne"
                        );
                        Arc::new(NoOpSystemOne)
                    }
                }
            }
            #[cfg(not(feature = "candle"))]
            {
                ::omnesagent_log::record!(
                    WARN,
                    ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Fail),
                    "Candle feature is not compiled in, falling back to NoOpSystemOne"
                );
                Arc::new(NoOpSystemOne)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_lenient_parsing() {
        assert_eq!(ProviderKind::from_str_lenient("LAYA"), ProviderKind::Laya);
        assert_eq!(ProviderKind::from_str_lenient("unknown"), ProviderKind::Off);
        assert_eq!(DeviceKind::from_str_lenient("gpu"), DeviceKind::Cuda);
        assert_eq!(QuantizationKind::from_str_lenient("int8"), QuantizationKind::Q8);
    }

    #[test]
    fn test_create_system_one_off_returns_noop() {
        let config = SystemOneConfig::default();
        let s1 = create_system_one(&config);
        assert!(!s1.is_available());
        assert_eq!(s1.model_id(), "disabled");
    }

    #[test]
    fn test_create_system_one_fake_returns_available() {
        let config = SystemOneConfig {
            provider: ProviderKind::Fake,
            ..Default::default()
        };
        let s1 = create_system_one(&config);
        assert!(s1.is_available());
        assert_eq!(s1.model_id(), "fake-laya-stub");
    }
}
