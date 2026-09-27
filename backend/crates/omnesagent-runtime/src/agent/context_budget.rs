//! Context budget profiling and execution mode configuration.
//!
//! Tracks token distribution across prompt components (system prompt, KAG AST,
//! memory, dialog history, tool outputs, compression savings) and provides
//! operational profiles (Fast, DeepCode, Architect, RalphLoop).

use serde::{Deserialize, Serialize};

/// Operational profile determining model selection, reasoning budget, and KAG depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentExecutionMode {
    /// Priority on local S1 (Laya) inference, tightly compressed context, lightweight models.
    #[default]
    Fast,
    /// Maximum KAG depth (up to 3 levels), AST validation, auto compiler/linter checks.
    DeepCode,
    /// Deep reasoning / thinking budget (DeepSeek-R1), two-phase verification, structured plan artifacts.
    Architect,
    /// Autonomous multi-iteration loop (up to 10 iterations) with automated testing and rollback on regression.
    RalphLoop,
}

impl AgentExecutionMode {
    /// Returns default temperature setting for this mode.
    #[must_use]
    pub fn default_temperature(&self) -> f32 {
        match self {
            Self::Fast => 0.2,
            Self::DeepCode => 0.1,
            Self::Architect => 0.6,
            Self::RalphLoop => 0.2,
        }
    }

    /// Returns the maximum KAG dependency traversal depth.
    #[must_use]
    pub fn kag_max_depth(&self) -> usize {
        match self {
            Self::Fast => 1,
            Self::DeepCode => 3,
            Self::Architect => 2,
            Self::RalphLoop => 3,
        }
    }

    /// Whether this mode automatically verifies code via compiler or test suite.
    #[must_use]
    pub fn auto_verify_compiler(&self) -> bool {
        matches!(self, Self::DeepCode | Self::RalphLoop)
    }

    /// Whether this mode requires explicit plan artifact generation.
    #[must_use]
    pub fn requires_plan_artifact(&self) -> bool {
        matches!(self, Self::Architect)
    }

    /// Maximum loop iterations before human confirmation is required.
    #[must_use]
    pub fn max_autonomous_iterations(&self) -> usize {
        match self {
            Self::Fast => 1,
            Self::DeepCode => 5,
            Self::Architect => 3,
            Self::RalphLoop => 10,
        }
    }
}

/// Dynamic snapshot of prompt token allocations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextBudgetSnapshot {
    /// Base system prompt and tool definitions.
    pub system_tokens: usize,
    /// Injected KAG code graph symbols, signatures, and PageRank nodes.
    pub kag_ast_tokens: usize,
    /// Injected long-term memory facts after MMR filtering.
    pub memory_tokens: usize,
    /// Compressed dialogue history turns.
    pub dialog_history_tokens: usize,
    /// Completed tool call observations from current turn.
    pub tool_outputs_tokens: usize,
    /// Total tokens saved via SQZ / zstd / S1 semantic trimming.
    pub compression_saved_tokens: usize,
    /// Total prompt tokens consumed.
    pub total_prompt_tokens: usize,
    /// Hard context limit of target model.
    pub context_limit: usize,
}

impl ContextBudgetSnapshot {
    /// Heuristic estimation of tokens from raw character count (~4 chars per token).
    #[must_use]
    pub fn estimate_tokens(text: &str) -> usize {
        (text.len() + 3) / 4
    }

    /// Constructs a new snapshot, computing total prompt tokens.
    #[must_use]
    pub fn new(
        system_tokens: usize,
        kag_ast_tokens: usize,
        memory_tokens: usize,
        dialog_history_tokens: usize,
        tool_outputs_tokens: usize,
        compression_saved_tokens: usize,
        context_limit: usize,
    ) -> Self {
        let total_prompt_tokens = system_tokens
            .saturating_add(kag_ast_tokens)
            .saturating_add(memory_tokens)
            .saturating_add(dialog_history_tokens)
            .saturating_add(tool_outputs_tokens);

        Self {
            system_tokens,
            kag_ast_tokens,
            memory_tokens,
            dialog_history_tokens,
            tool_outputs_tokens,
            compression_saved_tokens,
            total_prompt_tokens,
            context_limit,
        }
    }

    /// Percentage of the total context window consumed (0..100).
    #[must_use]
    pub fn utilization_percent(&self) -> u8 {
        if self.context_limit == 0 {
            return 0;
        }
        let pct = (self.total_prompt_tokens as f64 / self.context_limit as f64) * 100.0;
        pct.clamp(0.0, 100.0) as u8
    }
}

/// Profiler measuring token distribution across agent turn components.
#[derive(Debug, Default)]
pub struct ContextBudgetProfiler {
    pub system_tokens: usize,
    pub kag_ast_tokens: usize,
    pub memory_tokens: usize,
    pub dialog_history_tokens: usize,
    pub tool_outputs_tokens: usize,
    pub compression_saved_tokens: usize,
}

impl ContextBudgetProfiler {
    /// Creates a new profiler initialized with zeros.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records system prompt tokens.
    pub fn record_system_prompt(&mut self, text: &str) {
        self.system_tokens = ContextBudgetSnapshot::estimate_tokens(text);
    }

    /// Records KAG AST graph injected tokens.
    pub fn record_kag_ast(&mut self, text: &str) {
        self.kag_ast_tokens = ContextBudgetSnapshot::estimate_tokens(text);
    }

    /// Records memory injection tokens.
    pub fn record_memory(&mut self, text: &str) {
        self.memory_tokens = ContextBudgetSnapshot::estimate_tokens(text);
    }

    /// Records dialog history tokens.
    pub fn record_dialog_history(&mut self, text: &str) {
        self.dialog_history_tokens = ContextBudgetSnapshot::estimate_tokens(text);
    }

    /// Records tool output tokens.
    pub fn record_tool_outputs(&mut self, text: &str) {
        self.tool_outputs_tokens = ContextBudgetSnapshot::estimate_tokens(text);
    }

    /// Records tokens saved through compression algorithms.
    pub fn record_compression_savings(&mut self, saved_tokens: usize) {
        self.compression_saved_tokens = self.compression_saved_tokens.saturating_add(saved_tokens);
    }

    /// Produces an immutable snapshot against the target model limit.
    #[must_use]
    pub fn snapshot(&self, context_limit: usize) -> ContextBudgetSnapshot {
        ContextBudgetSnapshot::new(
            self.system_tokens,
            self.kag_ast_tokens,
            self.memory_tokens,
            self.dialog_history_tokens,
            self.tool_outputs_tokens,
            self.compression_saved_tokens,
            context_limit,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_budget_snapshot_calculations() {
        let snapshot = ContextBudgetSnapshot::new(500, 300, 200, 1000, 400, 600, 4000);
        assert_eq!(snapshot.total_prompt_tokens, 2400);
        assert_eq!(snapshot.utilization_percent(), 60);
    }

    #[test]
    fn test_profiler_records_and_estimates() {
        let mut profiler = ContextBudgetProfiler::new();
        profiler.record_system_prompt("You are a helpful assistant.");
        profiler.record_compression_savings(150);
        let snap = profiler.snapshot(8192);
        assert!(snap.system_tokens > 0);
        assert_eq!(snap.compression_saved_tokens, 150);
        assert_eq!(snap.context_limit, 8192);
    }

    #[test]
    fn test_execution_modes() {
        assert_eq!(AgentExecutionMode::Fast.kag_max_depth(), 1);
        assert_eq!(AgentExecutionMode::DeepCode.kag_max_depth(), 3);
        assert!(AgentExecutionMode::DeepCode.auto_verify_compiler());
        assert!(AgentExecutionMode::Architect.requires_plan_artifact());
        assert_eq!(AgentExecutionMode::RalphLoop.max_autonomous_iterations(), 10);
    }
}
