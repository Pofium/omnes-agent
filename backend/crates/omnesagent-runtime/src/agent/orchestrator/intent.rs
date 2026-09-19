// Intent definition and privilege level for OmnesAgent Triage Router.

use serde::{Deserialize, Serialize};

/// High-level intent of the user message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentIntent {
    /// Pure conversational question, greeting, general advice, explanations.
    /// Does not require tools; zero tool definitions injected; no TODO block.
    DirectChat,

    /// Code or workspace investigation: searching, reading files, architectural exploration.
    /// Injects only read-only inspection tools (grep, read_file, ob2h AST, etc.).
    CodeExploration,

    /// Code writing, bug fixing, test running, refactoring.
    /// Injects the full toolset (edit, write, shell, test). Enables TODO tracking and approval bridges.
    EngineeringTask,

    /// System administration, Docker, SSH, VPS, migrations, DevOps commands.
    /// Full toolset with elevated safety policies and strict command approvals.
    SystemAdmin,
}

impl AgentIntent {
    /// Numerical privilege level (similar to Unix read -> read-write -> execute).
    /// Used by the escalation engine to allow seamless mid-session upgrades.
    #[inline]
    pub const fn privilege_level(self) -> u8 {
        match self {
            Self::DirectChat => 0,
            Self::CodeExploration => 1,
            Self::EngineeringTask => 2,
            Self::SystemAdmin => 3,
        }
    }

    /// Default string identifier for UI headers and telemetry.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DirectChat => "chat",
            Self::CodeExploration => "explore",
            Self::EngineeringTask => "task",
            Self::SystemAdmin => "admin",
        }
    }
}

impl std::fmt::Display for AgentIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Execution engine backend responsible for handling the turn or task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutionEngine {
    /// Standard agent loop (`omnesagent-runtime::agent::loop_`).
    /// Best for interactive conversations, queries, quick answers, single-turn edits.
    StandardLoop,

    /// Specialized autonomous engineering agent (`omnesagent-ralph::loop_driver::RalphLoopDriver`).
    /// Best for multi-file refactoring, test-driven development, self-healing test loops.
    RalphAutonomous,
}

impl ExecutionEngine {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StandardLoop => "standard_loop",
            Self::RalphAutonomous => "ralph_autonomous",
        }
    }
}

impl std::fmt::Display for ExecutionEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
