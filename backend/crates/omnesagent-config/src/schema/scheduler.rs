//! Scheduler loop configuration: the `[scheduler]` section - enabling the
//! built-in cron loop, polling caps, catch-up on startup, and run-history
//! retention.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::SchedulerConfig` path is unchanged.

use super::*;

// ── Scheduler ────────────────────────────────────────────────────

/// Scheduler configuration for periodic task execution (`[scheduler]` section).
///
/// Owns the cron-runtime knobs: per-job declarations live on
/// `Config.cron: HashMap<String, CronJobDecl>` (alias-keyed), while the
/// scheduler loop's runtime behavior (`enabled`, polling cap, catch-up) lives here.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "scheduler"]
pub struct SchedulerConfig {
    /// Enable the built-in scheduler loop. When false, no cron jobs run.
    #[serde(default = "default_scheduler_enabled")]
    pub enabled: bool,
    /// Maximum number of persisted scheduled tasks per polling cycle.
    #[serde(default = "default_scheduler_max_tasks")]
    pub max_tasks: usize,
    /// Maximum tasks executed in parallel within a single polling cycle.
    #[serde(default = "default_scheduler_max_concurrent")]
    pub max_concurrent: usize,
    /// Run all overdue jobs at scheduler startup. Default: `true`.
    ///
    /// When the daemon restarts late, jobs whose `next_run` is in the past
    /// fire once before normal polling resumes. Disable to wait for the
    /// next scheduled occurrence instead.
    #[serde(default = "default_true")]
    pub catch_up_on_startup: bool,
    /// Maximum number of historical cron run records to retain. Default: `50`.
    #[serde(default = "default_max_run_history")]
    pub max_run_history: u32,
}

fn default_scheduler_enabled() -> bool {
    true
}

fn default_scheduler_max_tasks() -> usize {
    64
}

fn default_scheduler_max_concurrent() -> usize {
    4
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            enabled: default_scheduler_enabled(),
            max_tasks: default_scheduler_max_tasks(),
            max_concurrent: default_scheduler_max_concurrent(),
            catch_up_on_startup: true,
            max_run_history: default_max_run_history(),
        }
    }
}
