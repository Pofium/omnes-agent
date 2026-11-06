//! Skills configuration: the `[skills]` section - external registries,
//! prompt-injection mode, and the skill creation / improvement / install
//! suggestion sub-configs.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::<SkillsType>` path is unchanged.

use super::default_true;
use omnesagent_macros::Configurable;
use serde::{Deserialize, Serialize};

/// Skills loading configuration (`[skills]` section).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum SkillsPromptInjectionMode {
    /// Default behavior for the v0.8.x line: inline full skill instructions.
    #[default]
    Full,
    /// Inline compact skill metadata
    /// (name/description/location + callable tool specs) and load instructions
    /// on demand via `read_skill`.
    Compact,
}

/// An external, user-configured skill registry OmnesAgent can install from.
///
/// Reuses the same git-clone mechanism as the default `omnesagent-skills`
/// registry. Install a skill from it with `registry:<name>/<skill>`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ExternalRegistryKind {
    #[default]
    Git,
    Unsupported(String),
}

impl ExternalRegistryKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Git => "git",
            Self::Unsupported(kind) => kind.as_str(),
        }
    }
}

impl From<String> for ExternalRegistryKind {
    fn from(kind: String) -> Self {
        if kind == "git" {
            Self::Git
        } else {
            Self::Unsupported(kind)
        }
    }
}

impl From<ExternalRegistryKind> for String {
    fn from(kind: ExternalRegistryKind) -> Self {
        kind.as_str().to_string()
    }
}

impl std::fmt::Display for ExternalRegistryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for ExternalRegistryKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ExternalRegistryKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(String::deserialize(deserializer)?.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub struct ExternalRegistry {
    /// Short alias used in `registry:<name>/<skill>` install specs.
    pub name: String,
    /// Git repository URL of the registry (must expose a top-level `skills/` dir).
    pub url: String,
    /// Registry protocol. Only `"git"` is supported today; other protocols
    /// are reserved for a future additive release.
    #[serde(default)]
    #[cfg_attr(feature = "schema-export", schemars(with = "String"))]
    pub kind: ExternalRegistryKind,
    /// Whether this registry is eligible for installs. Default: `true`.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl ExternalRegistry {
    /// Returns true when `name` can be addressed by `registry:<name>/<skill>`.
    ///
    /// Keep this as the single registry-alias rule used by both config
    /// validation and runtime install-spec parsing. Lowercase aliases also
    /// avoid clone-directory collisions on case-insensitive filesystems.
    pub fn is_valid_name(name: &str) -> bool {
        !name.is_empty()
            && name
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
    }
}

/// Skills loading configuration (`[skills]` section).
#[derive(Debug, Clone, Serialize, Deserialize, Default, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "skills"]
pub struct SkillsConfig {
    /// Enable loading and syncing the community open-skills repository.
    /// Default: `false` (opt-in).
    #[serde(default)]
    pub open_skills_enabled: bool,
    /// Optional path to a local open-skills repository.
    /// If unset, defaults to `$HOME/open-skills` when enabled.
    #[serde(default)]
    pub open_skills_dir: Option<String>,
    /// Allow script-like files in skills (`.sh`, `.bash`, `.ps1`, shebang shell files).
    /// Default: `false` (secure by default).
    #[serde(default)]
    pub allow_scripts: bool,
    /// URL of the skills registry repository for bare-name installs.
    /// Default: `https://github.com/omnesagent-labs/omnesagent-skills`
    #[serde(default)]
    pub registry_url: Option<String>,
    /// Additional user-configured skill registries, installed via
    /// `registry:<name>/<skill>`. Each reuses the git-clone registry path and
    /// is cloned to its own `extra-registry-<name>/` workspace directory.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_registries: Vec<ExternalRegistry>,
    /// Controls how skills are injected into the system prompt. Omission
    /// defaults to `full` throughout the v0.8.x release line; `compact`
    /// remains available as an explicit global or runtime-profile setting.
    #[serde(default)]
    pub prompt_injection_mode: SkillsPromptInjectionMode,
    /// Autonomous skill creation from successful multi-step task executions.
    #[serde(default)]
    #[nested]
    pub skill_creation: SkillCreationConfig,
    /// Prompt-triggered install suggestions for missing skills.
    #[serde(default, alias = "install-suggestions")]
    #[nested]
    pub install_suggestions: SkillInstallSuggestionsConfig,
    /// Automatic skill self-improvement after successful skill usage.
    #[serde(default)]
    #[nested]
    pub skill_improvement: SkillImprovementConfig,
}

/// Autonomous skill creation configuration (`[skills.skill_creation]` section).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "skills.skill_creation"]
#[serde(default)]
pub struct SkillCreationConfig {
    /// Enable automatic skill creation after successful multi-step tasks.
    /// Default: `false`.
    pub enabled: bool,
    /// Maximum number of auto-generated skills to keep.
    /// When exceeded, the oldest auto-generated skill is removed (LRU eviction).
    pub max_skills: usize,
    /// Embedding similarity threshold for deduplication.
    /// Skills with descriptions more similar than this value are skipped.
    pub similarity_threshold: f64,
    /// Synthesize a canonical `SKILL.md` from the execution trace via a
    /// bounded model-provider reflection call instead of the deterministic
    /// `SKILL.toml` generator. Requires `enabled = true`. Falls back to
    /// `SKILL.toml` whenever the reflection call or its output is invalid, so
    /// turning this on never leaves a skill un-created. This is distinct from
    /// the `[skills.skill_improvement]` background review fork, which patches
    /// existing skills after use rather than creating new ones from a trace.
    /// Default: `false`.
    #[serde(default = "default_reflection_enabled")]
    pub reflection_enabled: bool,
    /// Maximum characters of the final assistant answer fed into the
    /// reflection prompt. Bounds prompt size so a long answer cannot blow up
    /// the reflection request. Default: `2000`.
    #[serde(default = "default_max_final_answer_chars")]
    pub max_final_answer_chars: usize,
    /// Maximum characters of the rendered tool-call trace fed into the
    /// reflection prompt. Default: `4000`.
    #[serde(default = "default_max_tool_trace_chars")]
    pub max_tool_trace_chars: usize,
    /// Maximum characters of the task description fed into the reflection
    /// prompt. Default: `1000`.
    #[serde(default = "default_max_task_chars")]
    pub max_task_chars: usize,
}

fn default_reflection_enabled() -> bool {
    false
}

fn default_max_final_answer_chars() -> usize {
    2000
}

fn default_max_tool_trace_chars() -> usize {
    4000
}

fn default_max_task_chars() -> usize {
    1000
}

impl Default for SkillCreationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_skills: 500,
            similarity_threshold: 0.85,
            reflection_enabled: default_reflection_enabled(),
            max_final_answer_chars: default_max_final_answer_chars(),
            max_tool_trace_chars: default_max_tool_trace_chars(),
            max_task_chars: default_max_task_chars(),
        }
    }
}

/// Prompt-triggered skill install suggestions (`[skills.install_suggestions]` section).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "skills.install_suggestions"]
#[serde(default)]
pub struct SkillInstallSuggestionsConfig {
    /// Enable suggestions for installable skills before normal agent turns.
    /// Default: `false`.
    pub enabled: bool,
}

/// Skill self-improvement configuration (`[skills.skill-improvement]` section).
///
/// Controls the post-turn background review fork that may patch, expand, or
/// archive skills based on what the conversation revealed. The fork runs in a
/// restricted toolset (only `skills_list`, `skill_view`, `skill_manage`) and
/// never touches the user-visible conversation.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "skills.skill_improvement"]
pub struct SkillImprovementConfig {
    /// Enable the background skill-review fork. Default: `false`.
    /// Opt-in: users must explicitly enable this to allow post-turn skill
    /// mutations.
    #[serde(default)]
    pub enabled: bool,
    /// Minimum interval (in seconds) between reviews for the same skill.
    /// Acts as a durable rate limit on patches to any one skill. Default: `3600`.
    #[serde(default = "default_skill_improvement_cooldown")]
    pub cooldown_secs: u64,
    /// Spawn a review fork once at least this many tool-call iterations have
    /// accumulated in the current run. `0` disables iteration-based triggering.
    /// Default: `10`.
    #[serde(default = "default_nudge_interval_iterations")]
    pub nudge_interval_iterations: u32,
    /// Maximum tool-call iterations the review fork itself is allowed to make.
    /// Caps the fork's LLM cost. Default: `8`.
    #[serde(default = "default_max_review_iterations")]
    pub max_review_iterations: u32,
}

fn default_skill_improvement_cooldown() -> u64 {
    3600
}

fn default_nudge_interval_iterations() -> u32 {
    10
}

fn default_max_review_iterations() -> u32 {
    8
}

impl Default for SkillImprovementConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cooldown_secs: 3600,
            nudge_interval_iterations: 10,
            max_review_iterations: 8,
        }
    }
}
