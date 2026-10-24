// Execution profile and UI flags constructed by the Triage Router.

use serde::{Deserialize, Serialize};
use omnesagent_api::tool::ToolSpec;
use super::intent::{AgentIntent, ExecutionEngine};

/// UI-level display flags transmitted via SSE to the desktop / web client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiFlags {
    /// Whether to render the persistent TODO / Task list widget on top of the answer.
    /// Strict rule: false for DirectChat & CodeExploration; true only for active Engineering tasks.
    pub show_todo_widget: bool,

    /// Whether to render the full timeline of session tool executions.
    pub show_session_timeline: bool,

    /// Whether to stream and render Ralph autonomous progress (phases, test runs, diff).
    pub show_ralph_progress: bool,

    /// Human-readable mode identifier for the UI header badge ("chat", "explore", "task", "admin", "ralph").
    pub display_mode: String,
}

impl Default for UiFlags {
    fn default() -> Self {
        Self {
            show_todo_widget: false,
            show_session_timeline: false,
            show_ralph_progress: false,
            display_mode: "chat".to_string(),
        }
    }
}

/// Dynamic session execution profile configured by the orchestrator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionExecutionProfile {
    /// Classified intent of the conversation turn.
    pub intent: AgentIntent,

    /// Selected execution engine backend.
    pub engine: ExecutionEngine,

    /// Filtered list of tools permitted for this turn.
    pub allowed_tools: Vec<ToolSpec>,

    /// Dynamic token compression and deduplication policy.
    pub compression_policy: omnesagent_compression::CompressionPolicy,

    /// Optional system prompt overlay/instruction (e.g. "Focus on direct concise response without tool preamble").
    pub system_prompt_overlay: Option<String>,

    /// Seamless auto-continuation flag upon hitting `finish_reason: length`.
    pub auto_continue: bool,

    /// Maximum number of automatic continuation rounds allowed before safety stop.
    pub max_auto_continue_rounds: u8,

    /// UI display flags dispatched to the frontend.
    pub ui_flags: UiFlags,
}

impl SessionExecutionProfile {
    /// Build default execution profile for a given intent and engine.
    pub fn new(intent: AgentIntent, engine: ExecutionEngine, allowed_tools: Vec<ToolSpec>) -> Self {
        let (show_todo, show_timeline, show_ralph, display_mode) = match (intent, engine) {
            (AgentIntent::DirectChat, _) => (false, false, false, "chat".to_string()),
            (AgentIntent::CodeExploration, _) => (false, true, false, "explore".to_string()),
            (AgentIntent::EngineeringTask, ExecutionEngine::RalphAutonomous) => {
                (true, true, true, "ralph".to_string())
            }
            (AgentIntent::EngineeringTask, ExecutionEngine::StandardLoop) => {
                (true, true, false, "task".to_string())
            }
            (AgentIntent::SystemAdmin, _) => (true, true, false, "admin".to_string()),
        };

        let auto_continue = true;
        let max_auto_continue_rounds = 5;

        let system_prompt_overlay = match intent {
            AgentIntent::DirectChat => Some(
                "You are in Direct Conversation mode. Respond clearly, thoroughly, and directly. Do not propose task lists or plans unless explicitly asked.".to_string(),
            ),
            AgentIntent::CodeExploration => Some(
                "You are in Code Exploration mode. You have read-only inspection tools. Analyze the codebase, explain architecture, and report findings without modifying files.".to_string(),
            ),
            AgentIntent::EngineeringTask => None,
            AgentIntent::SystemAdmin => Some(
                "You are in System Administration mode. Exercise extreme caution with destructive commands and confirm before running irreversible operations.".to_string(),
            ),
        };

        let compression_policy = compression_policy_for_intent(intent);

        Self {
            intent,
            engine,
            allowed_tools,
            compression_policy,
            system_prompt_overlay,
            auto_continue,
            max_auto_continue_rounds,
            ui_flags: UiFlags {
                show_todo_widget: show_todo,
                show_session_timeline: show_timeline,
                show_ralph_progress: show_ralph,
                display_mode,
            },
        }
    }

    /// Adapts the session compression policy to a specific model family.
    ///
    /// The family policy keeps authority over family-specific knobs (schema
    /// mode, dedup ref format, cache breakpoints); the intent policy only
    /// narrows the feature toggles, so a capability the intent disables stays
    /// disabled regardless of family defaults.
    #[must_use]
    pub fn with_model_family(mut self, family: omnesagent_compression::ModelFamily) -> Self {
        self.compression_policy = narrow_by_family(self.compression_policy, family);
        self
    }

    /// Overrides the session compression policy directly.
    #[must_use]
    pub fn with_compression_policy(mut self, policy: omnesagent_compression::CompressionPolicy) -> Self {
        self.compression_policy = policy;
        self
    }
}

/// Intent-specific baseline compression policy, family-independent.
#[must_use]
pub fn compression_policy_for_intent(
    intent: AgentIntent,
) -> omnesagent_compression::CompressionPolicy {
    match intent {
        AgentIntent::DirectChat => omnesagent_compression::CompressionPolicy::default(),
        AgentIntent::CodeExploration => omnesagent_compression::CompressionPolicy {
            enable_ast_code_collapse: true,
            ..omnesagent_compression::CompressionPolicy::default()
        },
        AgentIntent::EngineeringTask => omnesagent_compression::CompressionPolicy {
            enable_ast_code_collapse: true,
            enable_dedup: true,
            enable_smart_crusher: true,
            ..omnesagent_compression::CompressionPolicy::default()
        },
        AgentIntent::SystemAdmin => omnesagent_compression::CompressionPolicy {
            enable_smart_crusher: true,
            ..omnesagent_compression::CompressionPolicy::default()
        },
    }
}

/// Intersects intent feature toggles with the model-family tuning.
#[must_use]
pub fn narrow_by_family(
    intent_policy: omnesagent_compression::CompressionPolicy,
    family: omnesagent_compression::ModelFamily,
) -> omnesagent_compression::CompressionPolicy {
    let mut family_policy = omnesagent_compression::CompressionPolicy::for_family(family);
    family_policy.enable_dedup &= intent_policy.enable_dedup;
    family_policy.enable_smart_crusher &= intent_policy.enable_smart_crusher;
    family_policy.enable_ast_code_collapse &= intent_policy.enable_ast_code_collapse;
    family_policy
}

/// Resolves the effective tool-output compression policy for one turn:
/// the intent classified from the triggering user message, narrowed by the
/// serving model's family tuning. This is the seam the turn loop uses —
/// the rest of [`SessionExecutionProfile`] is consumed as the Triage
/// Router's remaining phases come online.
#[must_use]
pub fn resolve_compression_policy(
    message: &str,
    model: &str,
) -> omnesagent_compression::CompressionPolicy {
    let intent = super::heuristic::classify_message(message).intent;
    let intent_policy = compression_policy_for_intent(intent);
    let family = omnesagent_compression::ModelFamily::detect_from_name(model);
    narrow_by_family(intent_policy, family)
}
