//! Codex CLI tool config (the `[codex_cli]` section) plus the
//! argument-safety warning inventory (`RISKY_CODEX_CLI_FLAGS` and helpers)
//! that classifies operator-controlled `extra_args` which can weaken
//! Codex sandbox or approval boundaries.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::CodexCliConfig` is unchanged. The warning machinery stays
//! reachable from `Config` validation (`config_impl`) via `pub(super)`.

use super::*;

// ── Codex CLI ───────────────────────────────────────────────────

/// Codex CLI tool configuration (`[codex_cli]` section).
///
/// Delegates coding tasks to the `codex exec` CLI. Authentication uses the
/// binary's own session by default — no API key needed unless
/// `env_passthrough` includes `OPENAI_API_KEY`.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "codex_cli"]
pub struct CodexCliConfig {
    /// Enable the `codex_cli` tool
    #[serde(default)]
    pub enabled: bool,
    /// Maximum execution time in seconds (coding tasks can be long)
    #[serde(default = "default_codex_cli_timeout_secs")]
    pub timeout_secs: u64,
    /// Maximum output size in bytes (2MB default)
    #[serde(default = "default_codex_cli_max_output_bytes")]
    pub max_output_bytes: usize,
    /// Extra env vars passed to the codex subprocess (e.g. OPENAI_API_KEY)
    #[serde(default)]
    #[credential_class = "legacy_env_path"]
    pub env_passthrough: Vec<String>,
    /// Extra CLI arguments appended to `codex exec` before the prompt.
    ///
    /// Values come from operator-controlled config (same trust level as
    /// `env_passthrough`) and remain allowed without blocking. Config validation
    /// warns when a recognized argument can disable Codex's sandbox, alter
    /// approval or policy loading, expand workspace access, or register a
    /// locally executable integration. This is a warning inventory, not an
    /// allowlist; ordinary and unknown arguments remain allowed and silent.
    ///
    /// **Warning:** `--sandbox=danger-full-access` disables Codex's bubblewrap
    /// isolation; only use in environments where the container itself provides
    /// isolation (e.g. Kubernetes pods with restricted PSS).
    ///
    /// Example: `["--sandbox=danger-full-access", "--skip-git-repo-check"]`
    #[serde(default)]
    pub extra_args: Vec<String>,
}

impl CodexCliConfig {
    /// Returns the configured arguments exactly as OmnesAgent forwards them to
    /// `codex exec`, paired with their original config indices.
    ///
    /// Trimming and empty-entry removal live here so subprocess construction
    /// and security-boundary diagnostics cannot interpret different argv
    /// sequences. Original indices keep warning paths stable for operators.
    pub fn effective_extra_args(&self) -> impl Iterator<Item = (usize, &str)> {
        effective_codex_cli_extra_args(&self.extra_args)
    }
}

fn effective_codex_cli_extra_args(extra_args: &[String]) -> impl Iterator<Item = (usize, &str)> {
    extra_args
        .iter()
        .enumerate()
        .filter_map(|(index, raw_arg)| {
            let arg = raw_arg.trim();
            (!arg.is_empty()).then_some((index, arg))
        })
}

#[derive(Debug, Clone, Copy)]
enum RiskyCodexCliArgValue {
    Presence,
    AnyValue,
    ExactValue(&'static str),
    SecurityConfigOverride,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RiskyCodexCliFlag {
    spellings: &'static [&'static str],
    pub(super) display: &'static str,
    value: RiskyCodexCliArgValue,
    pub(super) effect: &'static str,
}

/// Recognized Codex CLI arguments that can weaken the warning contract below.
///
/// The scope is deliberately finite: direct sandbox, approval, policy-source,
/// and workspace selectors, plus config families that register local commands,
/// hooks, plugins, apps, or MCP tools. This is not an allowlist or an exhaustive
/// classification of every behavioral Codex setting. Unknown arguments remain
/// allowed and silent for forward compatibility.
const RISKY_CODEX_CLI_FLAGS: &[RiskyCodexCliFlag] = &[
    // `danger-full-access` removes the command sandbox; read-only and
    // workspace-write remain ordinary supported operator choices.
    RiskyCodexCliFlag {
        spellings: &["--sandbox", "-s"],
        display: "--sandbox danger-full-access / -s danger-full-access",
        value: RiskyCodexCliArgValue::ExactValue("danger-full-access"),
        effect: "disable Codex command sandboxing",
    },
    // The long flag and its `--yolo` alias disable both independent gates.
    RiskyCodexCliFlag {
        spellings: &["--dangerously-bypass-approvals-and-sandbox", "--yolo"],
        display: "--dangerously-bypass-approvals-and-sandbox / --yolo",
        value: RiskyCodexCliArgValue::Presence,
        effect: "disable Codex approval prompts and sandbox enforcement",
    },
    // Automatic review changes who decides approval requests even though the
    // command sandbox remains enabled.
    RiskyCodexCliFlag {
        spellings: &["--approve-for-me", "--not-so-yolo"],
        display: "--approve-for-me / --not-so-yolo",
        value: RiskyCodexCliArgValue::Presence,
        effect: "route Codex approval requests through automatic review",
    },
    // Hooks may execute outside the command sandbox, so bypassing their trust
    // review crosses a separate execution boundary.
    RiskyCodexCliFlag {
        spellings: &["--dangerously-bypass-hook-trust"],
        display: "--dangerously-bypass-hook-trust",
        value: RiskyCodexCliArgValue::Presence,
        effect: "run enabled hooks without persisted hook trust",
    },
    // Execpolicy rules are an operator-authored command constraint layer.
    RiskyCodexCliFlag {
        spellings: &["--ignore-rules"],
        display: "--ignore-rules",
        value: RiskyCodexCliArgValue::Presence,
        effect: "skip execpolicy .rules files that constrain command execution",
    },
    // Codex treats every added directory as writable alongside the workspace.
    RiskyCodexCliFlag {
        spellings: &["--add-dir"],
        display: "--add-dir",
        value: RiskyCodexCliArgValue::AnyValue,
        effect: "add writable directories alongside the selected workspace",
    },
    // OmnesAgent validates the tool's working_directory before spawning Codex;
    // this flag can replace that validated root inside the child process.
    RiskyCodexCliFlag {
        spellings: &["--cd", "-C"],
        display: "--cd / -C",
        value: RiskyCodexCliArgValue::AnyValue,
        effect: "replace the OmnesAgent-validated Codex working root",
    },
    // A named profile is layered over the base user config and can replace its
    // approval, sandbox, permission, and workspace boundary settings.
    RiskyCodexCliFlag {
        spellings: &["--profile", "-p"],
        display: "--profile / -p",
        value: RiskyCodexCliArgValue::AnyValue,
        effect: "load a Codex configuration profile that can override approval, sandbox, or workspace policy",
    },
    // Omitting the base user config can remove stricter policy selected there.
    RiskyCodexCliFlag {
        spellings: &["--ignore-user-config"],
        display: "--ignore-user-config",
        value: RiskyCodexCliArgValue::Presence,
        effect: "skip Codex user configuration that may define stricter approval, sandbox, or workspace policy",
    },
    // Codex folds these global selectors into `features.<name>` config
    // overrides for every subcommand. Feature gates include permission,
    // network, sandbox, hooks, apps, plugins, and other external capabilities,
    // so classify the selector without copying Codex's fast-moving registry.
    RiskyCodexCliFlag {
        spellings: &["--enable", "--disable"],
        display: "--enable / --disable feature toggle",
        value: RiskyCodexCliArgValue::AnyValue,
        effect: "change Codex feature gates that can alter security or external capability boundaries",
    },
    // `-c` is global in Codex and can override approval policy, legacy sandbox
    // mode, feature gates, or the newer named permission-profile configuration.
    RiskyCodexCliFlag {
        spellings: &["--config", "-c"],
        display: "--config / -c security or executable-integration override",
        value: RiskyCodexCliArgValue::SecurityConfigOverride,
        effect: "override a recognized Codex security or executable-integration setting",
    },
];

/// Current Codex config namespaces within this warning's finite contract.
/// Match each namespace as a family so new descendants inherit the warning
/// without turning unrelated keys or near matches into false positives.
const RISKY_CODEX_CLI_CONFIG_KEY_FAMILIES: &[&str] = &[
    // Approval, sandbox, workspace, and environment policy.
    "approval_policy",
    "approvals_reviewer",
    "auto_review",
    "sandbox_mode",
    "sandbox_permissions",
    "sandbox_workspace_write",
    "default_permissions",
    "permissions",
    "shell_environment_policy",
    "allow_login_shell",
    "features",
    "use_legacy_landlock",
    // Project trust and platform sandbox policy.
    "projects",
    "windows",
    // Locally executable or tool-providing integrations.
    "mcp_servers",
    "hooks",
    "plugins",
    "apps",
    "notify",
];

pub(super) const CODEX_CLI_EXTRA_ARGS_SECURITY_BOUNDARY_WARNING: &str =
    "codex_cli_extra_args_security_boundary";

#[derive(Debug, Clone, Copy)]
pub(super) struct RiskyCodexCliArgMatch {
    pub(super) index: usize,
    pub(super) flag: &'static RiskyCodexCliFlag,
}

pub(super) fn risky_codex_cli_arg_matches(extra_args: &[String]) -> Vec<RiskyCodexCliArgMatch> {
    let mut matches = Vec::new();
    let effective_args = effective_codex_cli_extra_args(extra_args).collect::<Vec<_>>();

    for (effective_index, (original_index, arg)) in effective_args.iter().copied().enumerate() {
        if arg == "--" {
            break;
        }

        for flag in RISKY_CODEX_CLI_FLAGS {
            if flag.spellings.iter().any(|spelling| {
                codex_cli_flag_matches(&effective_args, effective_index, arg, spelling, flag.value)
            }) {
                matches.push(RiskyCodexCliArgMatch {
                    index: original_index,
                    flag,
                });
                break;
            }
        }
    }

    matches
}

fn codex_cli_flag_matches(
    effective_args: &[(usize, &str)],
    index: usize,
    arg: &str,
    spelling: &str,
    value_rule: RiskyCodexCliArgValue,
) -> bool {
    if arg == spelling {
        return match value_rule {
            RiskyCodexCliArgValue::Presence => true,
            _ => effective_args
                .get(index + 1)
                .map(|(_, value)| *value)
                .filter(|value| !value.starts_with('-'))
                .is_some_and(|value| codex_cli_value_matches(value, value_rule)),
        };
    }

    !matches!(value_rule, RiskyCodexCliArgValue::Presence)
        && codex_cli_attached_value(arg, spelling)
            .is_some_and(|value| codex_cli_value_matches(value, value_rule))
}

fn codex_cli_attached_value<'a>(arg: &'a str, spelling: &str) -> Option<&'a str> {
    let remainder = arg.strip_prefix(spelling)?;
    if spelling.starts_with("--") {
        remainder.strip_prefix('=')
    } else if spelling.len() == 2 && !remainder.is_empty() {
        Some(remainder.strip_prefix('=').unwrap_or(remainder))
    } else {
        None
    }
}

fn codex_cli_value_matches(value: &str, rule: RiskyCodexCliArgValue) -> bool {
    let value = value.trim();
    match rule {
        // Presence-only clap flags do not accept attached values. Their exact
        // spelling is handled before this value matcher is called.
        RiskyCodexCliArgValue::Presence => false,
        RiskyCodexCliArgValue::AnyValue => !value.is_empty(),
        RiskyCodexCliArgValue::ExactValue(expected) => {
            codex_cli_unquote(value).eq_ignore_ascii_case(expected)
        }
        RiskyCodexCliArgValue::SecurityConfigOverride => codex_cli_security_config_override(value),
    }
}

fn codex_cli_security_config_override(value: &str) -> bool {
    let Some((key, raw_value)) = value.split_once('=') else {
        return false;
    };
    let key = key.trim();
    let value = codex_cli_unquote(raw_value.trim());

    !value.is_empty()
        && RISKY_CODEX_CLI_CONFIG_KEY_FAMILIES
            .iter()
            .any(|family| codex_cli_config_key_is_in_family(key, family))
}

fn codex_cli_config_key_is_in_family(key: &str, family: &str) -> bool {
    key == family
        || key
            .strip_prefix(family)
            .is_some_and(|remainder| remainder.starts_with('.'))
}

fn codex_cli_unquote(value: &str) -> &str {
    value.trim_matches(|character| character == '\'' || character == '"')
}

fn default_codex_cli_timeout_secs() -> u64 {
    600
}

fn default_codex_cli_max_output_bytes() -> usize {
    2_097_152
}

impl Default for CodexCliConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            timeout_secs: default_codex_cli_timeout_secs(),
            max_output_bytes: default_codex_cli_max_output_bytes(),
            env_passthrough: Vec::new(),
            extra_args: Vec::new(),
        }
    }
}
