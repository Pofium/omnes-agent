// Unified Triage & Execution Orchestrator for OmnesAgent.
// Provides zero-latency deterministic intent classification, execution profiling,
// dynamic tool pruning, and seamless escalation.

pub mod intent;
pub mod profile;
pub mod tool_group;
pub mod heuristic;
pub mod escalation;
pub mod auto_continue;

#[cfg(test)]
mod tests;

pub use intent::{AgentIntent, ExecutionEngine};
pub use profile::{SessionExecutionProfile, UiFlags};
pub use tool_group::{
    filter_tools_for_intent, filter_tools_for_intent_with_query, is_mcp_or_external_tool,
    is_read_only_tool, prune_mcp_tools, ToolGroup,
};
pub use heuristic::{classify_message, HeuristicDecision};
pub use escalation::evaluate_escalation;
pub use auto_continue::{analyze_truncation, continuation_prompt, TruncationAnalysis, MAX_AUTO_CONTINUE_ROUNDS};

use omnesagent_api::tool::ToolSpec;

/// Primary entry point: analyze user message and generate a tailored execution profile.
pub fn triage(message: &str, all_tools: &[ToolSpec]) -> SessionExecutionProfile {
    let decision = classify_message(message);
    let allowed_tools = filter_tools_for_intent_with_query(
        all_tools,
        decision.intent,
        Some(message),
        Some(12),
    );
    let profile = SessionExecutionProfile::new(decision.intent, decision.engine, allowed_tools);

    ::omnesagent_log::record!(
        INFO,
        ::omnesagent_log::Event::new("omnesagent_runtime::orchestrator", ::omnesagent_log::Action::Resolve)
            .with_category(::omnesagent_log::EventCategory::Agent)
            .with_attrs(::serde_json::json!({
                "intent": profile.intent.as_str(),
                "engine": profile.engine.as_str(),
                "tools_count": profile.allowed_tools.len(),
                "show_todo": profile.ui_flags.show_todo_widget,
                "confidence": decision.confidence,
                "signals": decision.matched_signals,
            })),
        "Triage decision resolved for user turn"
    );

    profile
}
