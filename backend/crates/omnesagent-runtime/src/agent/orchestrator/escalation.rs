// Seamless mid-conversation escalation engine.

use super::profile::SessionExecutionProfile;
use super::heuristic::classify_message;
use super::tool_group::filter_tools_for_intent;
use omnesagent_api::tool::ToolSpec;

/// Evaluates whether the incoming user message warrants an escalation or de-escalation of session mode.
pub fn evaluate_escalation(
    current_profile: &SessionExecutionProfile,
    new_message: &str,
    all_tools: &[ToolSpec],
    recent_tool_call_count: usize,
) -> SessionExecutionProfile {
    let decision = classify_message(new_message);
    let new_intent = decision.intent;

    // Upgrades are always granted immediately (e.g. Chat -> EngineeringTask).
    if new_intent.privilege_level() > current_profile.intent.privilege_level() {
        let tools = filter_tools_for_intent(all_tools, new_intent);
        return SessionExecutionProfile::new(new_intent, decision.engine, tools);
    }

    // Downgrades (e.g. EngineeringTask -> DirectChat) are granted only if recent turns had no active tool calls.
    if new_intent.privilege_level() < current_profile.intent.privilege_level()
        && recent_tool_call_count == 0
    {
        let tools = filter_tools_for_intent(all_tools, new_intent);
        return SessionExecutionProfile::new(new_intent, decision.engine, tools);
    }

    // Otherwise, preserve current profile
    current_profile.clone()
}
