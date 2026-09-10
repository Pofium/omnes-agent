//! Configuration for the Ralph autonomous development orchestrator and ob2h-bridge.

use serde::{Deserialize, Serialize};
use omnesagent_macros::{ConfigEnum, Configurable};

/// Executor kind for Ralph loops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, ConfigEnum)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum RalphExecutorKind {
    #[default]
    Native,
    Omnescode,
    Claude,
    Hermes,
    Custom,
}

/// Autonomy level for Ralph gates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, ConfigEnum)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub enum RalphAutonomy {
    #[serde(rename = "L0")]
    L0,
    #[default]
    #[serde(rename = "L1")]
    L1,
    #[serde(rename = "L2")]
    L2,
}

/// Operating mode for Ralph (Ponytail modes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, ConfigEnum)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum RalphMode {
    Lite,
    #[default]
    Full,
    Off,
}

/// Git commit format for Ralph iteration commits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, ConfigEnum)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum RalphCommitFormat {
    /// Conventional Commits: feat(ralph/T-XXX): iN — summary
    #[default]
    Cc,
    /// Legacy format: ralph(T-XXX, iN): summary
    Legacy,
}

/// Test output format parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, ConfigEnum)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum TestOutputFormat {
    #[default]
    ExitCode,
    JunitXml,
}

/// Limits on Ralph execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub struct RalphLimitsConfig {
    /// Total tokens allowed across the run (0 = unlimited).
    #[serde(default)]
    pub total_tokens: u64,
}

/// Main Ralph orchestrator configuration (`[ralph]`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "ralph"]
pub struct RalphConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub executor: RalphExecutorKind,
    #[serde(default)]
    pub executor_command: String,
    #[serde(default)]
    pub default_autonomy: RalphAutonomy,
    #[serde(default = "default_max_iterations_per_task")]
    pub max_iterations_per_task: usize,
    #[serde(default = "default_max_total_iterations")]
    pub max_total_iterations: usize,
    #[serde(default = "default_plateau_window")]
    pub plateau_window: usize,
    #[serde(default = "default_context_max_tokens")]
    pub context_max_tokens: usize,
    #[serde(default = "default_test_command_fallback")]
    pub test_command_fallback: String,
    #[serde(default)]
    pub test_output_format: TestOutputFormat,
    #[serde(default = "default_executor_timeout_secs")]
    pub executor_timeout_secs: u64,
    #[serde(default)]
    pub mode: RalphMode,
    #[serde(default = "default_true")]
    pub minimality_review: bool,
    #[serde(default = "default_true")]
    pub debt_harvest: bool,
    #[serde(default)]
    pub notify_channel: String,
    #[serde(default)]
    pub commit_format: RalphCommitFormat,
    #[serde(default)]
    #[nested]
    pub limits: RalphLimitsConfig,
}

fn default_true() -> bool {
    true
}
fn default_max_iterations_per_task() -> usize {
    5
}
fn default_max_total_iterations() -> usize {
    60
}
fn default_plateau_window() -> usize {
    3
}
fn default_context_max_tokens() -> usize {
    6000
}
fn default_test_command_fallback() -> String {
    "cargo test".to_string()
}
fn default_executor_timeout_secs() -> u64 {
    300
}

impl Default for RalphConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            executor: RalphExecutorKind::default(),
            executor_command: String::new(),
            default_autonomy: RalphAutonomy::default(),
            max_iterations_per_task: default_max_iterations_per_task(),
            max_total_iterations: default_max_total_iterations(),
            plateau_window: default_plateau_window(),
            context_max_tokens: default_context_max_tokens(),
            test_command_fallback: default_test_command_fallback(),
            test_output_format: TestOutputFormat::default(),
            executor_timeout_secs: default_executor_timeout_secs(),
            mode: RalphMode::default(),
            minimality_review: true,
            debt_harvest: true,
            notify_channel: String::new(),
            commit_format: RalphCommitFormat::default(),
            limits: RalphLimitsConfig::default(),
        }
    }
}

/// Push scope for ob2h-bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, ConfigEnum)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum BridgePushScope {
    #[default]
    Verified,
    All,
}

/// Configuration for the ob2h knowledge bridge (`[ob2h_bridge]`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "ob2h_bridge"]
pub struct Ob2hBridgeConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_ob2h_command")]
    pub ob2h_command: String,
    #[serde(default = "default_ob2h_args")]
    pub ob2h_args: Vec<String>,
    #[serde(default)]
    pub projects: Vec<String>,
    #[serde(default)]
    pub push_scope: BridgePushScope,
    #[serde(default)]
    pub auto_push_after_archive: bool,
}

fn default_ob2h_command() -> String {
    "ob2h".to_string()
}
fn default_ob2h_args() -> Vec<String> {
    vec!["serve".to_string()]
}

impl Default for Ob2hBridgeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ob2h_command: default_ob2h_command(),
            ob2h_args: default_ob2h_args(),
            projects: Vec::new(),
            push_scope: BridgePushScope::default(),
            auto_push_after_archive: false,
        }
    }
}
