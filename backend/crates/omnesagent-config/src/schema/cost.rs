//! Cost tracking and budget enforcement config: the `[cost]` section, its
//! enforcement policy, and the operator rate sheet under `[cost.rates.*]`.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::<CostType>` path is unchanged.

use crate::validation_bail;
use anyhow::Result;
use omnesagent_macros::Configurable;
use serde::{Deserialize, Serialize};

// ── Cost tracking and budget enforcement ───────────────────────────

/// Cost tracking and budget enforcement configuration (`[cost]` section).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "cost"]
pub struct CostConfig {
    /// Enable cost tracking (default: true)
    #[tab(Limits)]
    #[serde(default = "default_cost_enabled")]
    pub enabled: bool,

    /// Daily spending limit in USD (default: 10.00)
    #[tab(Limits)]
    #[serde(default = "default_daily_limit")]
    pub daily_limit_usd: f64,

    /// Monthly spending limit in USD (default: 100.00)
    #[tab(Limits)]
    #[serde(default = "default_monthly_limit")]
    pub monthly_limit_usd: f64,

    /// Warn when spending reaches this percentage of limit (default: 80)
    #[tab(Limits)]
    #[serde(default = "default_warn_percent")]
    pub warn_at_percent: u8,

    /// Allow requests to exceed budget with --override flag (default: false)
    #[tab(Limits)]
    #[serde(default)]
    pub allow_override: bool,

    /// Cost enforcement behavior when budget limits are approached or exceeded.
    #[tab(Limits)]
    #[serde(default)]
    #[nested]
    pub enforcement: CostEnforcementConfig,

    /// Stamp each recorded cost entry with the originating agent alias so
    /// `/api/cost?agent=<alias>` and CLI rollups can attribute spend to a
    /// specific agent. Disable on high-volume deployments if the extra
    /// HashMap aggregation shows up in profiles (default: true).
    #[tab(Limits)]
    #[serde(default = "default_track_per_agent")]
    pub track_per_agent: bool,

    /// Operator-managed rate sheet at `[cost.rates.*]`. Sections mirror
    /// the `[providers.*]` dotted-path exactly with the trailing `alias`
    /// segment replaced by the resource the rate applies to (model id,
    /// tool name, …). Layout:
    ///
    /// ```toml
    /// [cost.rates.providers.models.anthropic."claude-opus-4-7"]
    /// input_per_mtok = 15.0
    /// output_per_mtok = 75.0
    /// cached_input_per_mtok = 1.5
    /// cache_write_per_mtok = 18.75
    ///
    /// [cost.rates.providers.tts.openai."tts-1-hd"]
    /// per_mchar = 30.0
    ///
    /// [cost.rates.providers.transcription.openai.whisper-1]
    /// per_minute = 0.006
    ///
    /// [cost.rates.tools.web_search]
    /// per_call = 0.005
    /// ```
    #[tab(Costs)]
    #[serde(default)]
    #[nested]
    pub rates: CostRatesConfig,
}

/// Configuration for cost enforcement behavior when budget limits are reached.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "cost.enforcement"]
pub struct CostEnforcementConfig {
    /// Enforcement mode: "warn", "block", or "route_down".
    #[serde(default = "default_cost_enforcement_mode")]
    pub mode: String,
    /// Model hint to route to when budget is exceeded (used with "route_down" mode).
    #[serde(default)]
    pub route_down_model: Option<String>,
    /// Reserve this percentage of budget for critical operations.
    #[serde(default = "default_reserve_percent")]
    pub reserve_percent: u8,
}

fn default_cost_enforcement_mode() -> String {
    "warn".to_string()
}

fn default_reserve_percent() -> u8 {
    10
}

impl Default for CostEnforcementConfig {
    fn default() -> Self {
        Self {
            mode: default_cost_enforcement_mode(),
            route_down_model: None,
            reserve_percent: default_reserve_percent(),
        }
    }
}

fn default_daily_limit() -> f64 {
    10.0
}

fn default_monthly_limit() -> f64 {
    100.0
}

fn default_warn_percent() -> u8 {
    80
}

fn default_cost_enabled() -> bool {
    true
}

fn default_track_per_agent() -> bool {
    true
}

/// `[cost.rates]` — top-level rate-sheet namespace. Mirrors the
/// `[providers.*]` shape so each subsection here points at the same
/// kind of resource its `[providers.*]` counterpart configures.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "cost.rates"]
pub struct CostRatesConfig {
    /// `[cost.rates.providers.*]` — rates for everything under
    /// `[providers.*]` (models, TTS, transcription, …).
    #[serde(default)]
    #[nested]
    pub providers: ProviderCostRates,

    /// `[cost.rates.tools.<name>]` — per-call rates for tools that
    /// hit paid APIs. Keyed by the tool's registered name.
    #[serde(default)]
    #[nested]
    #[resource_key]
    pub tools: std::collections::HashMap<String, ToolCostRates>,
}

impl CostRatesConfig {
    /// Lookup model token rates by `(provider_type, model)`. Dispatch
    /// lives on the typed wrapper — see [`crate::providers::ModelCostRatesByProvider`].
    #[must_use]
    pub fn model_rates(&self, provider_type: &str, model: &str) -> Option<&ModelCostRates> {
        self.providers.models.get(provider_type, model)
    }

    /// Lookup TTS rates by `(provider_type, voice)`.
    #[must_use]
    pub fn tts_rates(&self, provider_type: &str, voice: &str) -> Option<&TtsCostRates> {
        self.providers.tts.get(provider_type, voice)
    }

    /// Lookup transcription rates by `(provider_type, model)`.
    #[must_use]
    pub fn transcription_rates(
        &self,
        provider_type: &str,
        model: &str,
    ) -> Option<&TranscriptionCostRates> {
        self.providers.transcription.get(provider_type, model)
    }

    /// Lookup tool per-call rate by registered name.
    #[must_use]
    pub fn tool_rates(&self, tool_name: &str) -> Option<&ToolCostRates> {
        self.tools.get(tool_name)
    }

    /// Validate that every configured cost rate is finite, non-negative and
    /// within the shared sanity bound (upstream #9939). Zero remains valid so
    /// a deliberately free resource is distinguishable from unavailable
    /// pricing.
    pub fn validate(&self) -> Result<()> {
        fn validate_rate(path: String, value: Option<f64>) -> Result<()> {
            if let Some(value) = value
                && !crate::cost::is_sane_usd_rate(value)
            {
                let max = crate::cost::MAX_SANE_USD_RATE;
                validation_bail!(
                    InvalidNumericRange,
                    path.clone(),
                    "{path} = {value} is invalid; cost rates must be finite and between 0 and {max} USD per configured unit"
                );
            }
            Ok(())
        }

        let mut model_rates: Vec<_> = self.providers.models.iter_entries().collect();
        model_rates.sort_unstable_by(|left, right| (left.0, left.1).cmp(&(right.0, right.1)));
        for (provider, model, rates) in model_rates {
            let prefix = format!("cost.rates.providers.models.{provider}.{model}");
            validate_rate(format!("{prefix}.input_per_mtok"), rates.input_per_mtok)?;
            validate_rate(format!("{prefix}.output_per_mtok"), rates.output_per_mtok)?;
            validate_rate(
                format!("{prefix}.cached_input_per_mtok"),
                rates.cached_input_per_mtok,
            )?;
            validate_rate(
                format!("{prefix}.cache_write_per_mtok"),
                rates.cache_write_per_mtok,
            )?;
        }

        let mut tts_rates: Vec<_> = self.providers.tts.iter_entries().collect();
        tts_rates.sort_unstable_by(|left, right| (left.0, left.1).cmp(&(right.0, right.1)));
        for (provider, voice, rates) in tts_rates {
            validate_rate(
                format!("cost.rates.providers.tts.{provider}.{voice}.per_mchar"),
                rates.per_mchar,
            )?;
        }

        let mut transcription_rates: Vec<_> = self.providers.transcription.iter_entries().collect();
        transcription_rates
            .sort_unstable_by(|left, right| (left.0, left.1).cmp(&(right.0, right.1)));
        for (provider, model, rates) in transcription_rates {
            validate_rate(
                format!("cost.rates.providers.transcription.{provider}.{model}.per_minute"),
                rates.per_minute,
            )?;
        }

        let mut tool_rates: Vec<_> = self.tools.iter().collect();
        tool_rates.sort_unstable_by(|left, right| left.0.cmp(right.0));
        for (tool, rates) in tool_rates {
            validate_rate(format!("cost.rates.tools.{tool}.per_call"), rates.per_call)?;
        }

        Ok(())
    }
}

/// `[cost.rates.providers.*]` — provider-shaped rate sheets. Each field
/// here mirrors a corresponding field on `[providers.*]` with the
/// trailing alias segment replaced by the resource the rate prices.
/// The inner typed wrappers carry the per-provider-type slot layout
/// and own dispatch (their slot list is the single source of truth,
/// shared with their providers counterpart via the `for_each_*_provider_slot!`
/// macros in [`crate::providers`]).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "cost.rates.providers"]
pub struct ProviderCostRates {
    /// `[cost.rates.providers.models.<type>.<model>]`.
    #[serde(default)]
    #[nested]
    pub models: crate::providers::ModelCostRatesByProvider,
    /// `[cost.rates.providers.tts.<type>.<voice>]`.
    #[serde(default)]
    #[nested]
    pub tts: crate::providers::TtsCostRatesByProvider,
    /// `[cost.rates.providers.transcription.<type>.<model>]`.
    #[serde(default)]
    #[nested]
    pub transcription: crate::providers::TranscriptionCostRatesByProvider,
}

/// The provider families that carry `[cost.rates.providers.*]` rate sheets, as
/// the trailing segment shared by their `[providers.<category>]` section. One
/// entry per `#[nested]` field on `ProviderCostRates`; this list is the registry
/// the TUI and web surfaces resolve cost tabs from, never a per-surface literal.
pub const PROVIDER_COST_CATEGORIES: [&str; 3] = ["models", "tts", "transcription"];

/// Cost-rate category for a `providers.<category>` section key, or `None` for
/// sections that carry no rate sheets. Resolves against
/// [`PROVIDER_COST_CATEGORIES`] so a new rate-bearing family is added in one
/// place.
#[must_use]
pub fn cost_category_for_provider_section(section_key: &str) -> Option<&'static str> {
    let suffix = section_key.strip_prefix("providers.")?;
    PROVIDER_COST_CATEGORIES
        .into_iter()
        .find(|category| *category == suffix)
}

/// Token-cost rates for a single chat / completion model, in USD per
/// 1M tokens. Every field optional so partial sheets work without
/// ceremony (an operator who only knows the input rate can record it).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "cost.rates.providers.models"]
pub struct ModelCostRates {
    /// Input tokens (USD per 1M).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_per_mtok: Option<f64>,
    /// Output tokens (USD per 1M).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_per_mtok: Option<f64>,
    /// Cached input tokens (USD per 1M). Optional — leave unset on
    /// providers that don't charge separately for prompt cache hits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_input_per_mtok: Option<f64>,
    /// Cache-write tokens (USD per 1M). Optional вЂ” the premium providers
    /// charge to write prompt data into their cache (Anthropic bills 1.25x
    /// the input rate for the 5-minute TTL and 2x for the 1-hour TTL).
    /// Leave unset to keep pricing cache writes at the plain input rate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_write_per_mtok: Option<f64>,
}

/// Rates for a TTS model, in USD per 1M characters.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "cost.rates.providers.tts"]
pub struct TtsCostRates {
    /// Characters synthesised (USD per 1M).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per_mchar: Option<f64>,
}

/// Rates for a transcription model, in USD per minute of audio.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "cost.rates.providers.transcription"]
pub struct TranscriptionCostRates {
    /// Audio transcribed (USD per minute).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per_minute: Option<f64>,
}

/// Rates for a tool that hits a paid external API. Keyed in
/// `[cost.rates.tools.<name>]` by the tool's registered name.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "cost.rates.tools"]
pub struct ToolCostRates {
    /// Per-call cost (USD).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per_call: Option<f64>,
}

impl Default for CostConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            daily_limit_usd: default_daily_limit(),
            monthly_limit_usd: default_monthly_limit(),
            warn_at_percent: default_warn_percent(),
            allow_override: false,
            enforcement: CostEnforcementConfig::default(),
            track_per_agent: default_track_per_agent(),
            rates: CostRatesConfig::default(),
        }
    }
}
