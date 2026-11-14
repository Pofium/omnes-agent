//! Runtime adapter configuration: the `[runtime]` section - adapter kind
//! (`RuntimeKind`: native / docker / cloudflare) and the Docker sub-config
//! (`[runtime.docker]`).
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::<RuntimeType>` path is unchanged.

use super::*;

// ── Runtime ──────────────────────────────────────────────────────

/// Runtime adapter kind.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum RuntimeKind {
    #[default]
    Native,
    Docker,
    Cloudflare,
}

impl RuntimeKind {
    #[must_use]
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Docker => "docker",
            Self::Cloudflare => "cloudflare",
        }
    }
}

/// Runtime adapter configuration (`[runtime]` section).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "runtime"]
pub struct RuntimeConfig {
    /// Runtime kind: native | docker | cloudflare.
    #[serde(default, deserialize_with = "deserialize_enum_lenient")]
    pub kind: RuntimeKind,

    /// Docker runtime settings (used when `kind = "docker"`).
    #[serde(default)]
    #[nested]
    pub docker: DockerRuntimeConfig,

    /// Shell binary the native runtime uses for command execution.
    ///
    /// Applies only to `runtime.kind = "native"`; other runtimes ignore it.
    /// When unset or `null`, the system default `sh` is used.
    ///
    /// **Unix:** POSIX-compatible shells are invoked as
    /// `<shell> -c "<command>"`. Accepted forms:
    /// - a bare command name resolved via `PATH` (e.g. `"bash"`), or
    /// - an absolute path (e.g. `"/bin/bash"`, `"/usr/bin/zsh"`).
    ///
    /// `powershell` and `pwsh` select the PowerShell policy dialect and run as
    /// `<interpreter> -NoProfile -NonInteractive -Command <command>` on every
    /// supported desktop host; other Unix interpreters are treated as
    /// POSIX-compatible shells.
    ///
    /// The value is validated when the native runtime is constructed, so a bad
    /// value is reported up front rather than failing on the first shell
    /// command. Rejected: empty/whitespace; a relative path with separators
    /// (e.g. `"./sh"`, `"bin/sh"` — use a bare `PATH` name or an absolute path
    /// instead); a bare name not found on `PATH`; and a path that does not
    /// exist or is not executable.
    ///
    /// **Windows:** the value selects the interpreter *family* by its file stem
    /// (case-insensitive), which fixes the invocation convention:
    /// - `"powershell"` (Windows PowerShell 5.x) or `"pwsh"` (PowerShell 7+),
    ///   as a bare name resolved via `PATH` or an absolute path (e.g.
    ///   `"C:\\Program Files\\PowerShell\\7\\pwsh.exe"`), run the command as
    ///   `<interpreter> -NoProfile -NonInteractive -Command <command>`;
    /// - any other value (including the default `sh` and an explicit `"cmd"`)
    ///   runs `cmd.exe /C "<command>"`, preserving the historical behaviour.
    ///
    /// Only an empty/whitespace value is rejected on Windows; the interpreter is
    /// located at spawn time.
    ///
    /// Command policy uses the selected interpreter's dialect. PowerShell
    /// accepts a bounded grammar of simple command invocations, arguments,
    /// variable reads, and pipelines; expression and invocation constructs
    /// that cannot be safely classified are rejected before execution.
    ///
    /// **Ignored on Android** (always `/system/bin/sh`, whose shell is not on
    /// `PATH` for spawned processes).
    ///
    /// **Examples:**
    /// ```toml
    /// [runtime]
    /// shell = "bash" # Unix: resolves via PATH
    /// shell = "/bin/zsh" # Unix: absolute path
    /// shell = "pwsh" # PowerShell 7+ (Windows, Linux, or macOS)
    /// shell = "powershell" # Windows: Windows PowerShell 5.x
    /// ```
    #[serde(default)]
    pub shell: Option<String>,

    /// Global reasoning override for model_providers that expose explicit controls.
    /// - `None`: model_provider default behavior
    /// - `Some(true)`: request reasoning/thinking when supported
    /// - `Some(false)`: disable reasoning/thinking when supported
    #[serde(default)]
    pub reasoning_enabled: Option<bool>,
    /// Optional reasoning effort for model_providers that expose a level control.
    #[serde(default, deserialize_with = "deserialize_reasoning_effort_opt")]
    pub reasoning_effort: Option<String>,
}

/// Docker runtime configuration (`[runtime.docker]` section).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "runtime.docker"]
pub struct DockerRuntimeConfig {
    /// Runtime image used to execute shell commands.
    #[serde(default = "default_docker_image")]
    pub image: String,

    /// Docker network mode (`none`, `bridge`, etc.).
    #[serde(default = "default_docker_network")]
    pub network: String,

    /// Optional memory limit in MB (`None` = no explicit limit).
    #[serde(default = "default_docker_memory_limit_mb")]
    pub memory_limit_mb: Option<u64>,

    /// Optional CPU limit (`None` = no explicit limit).
    #[serde(default = "default_docker_cpu_limit")]
    pub cpu_limit: Option<f64>,

    /// Mount root filesystem as read-only.
    #[serde(default = "default_true")]
    pub read_only_rootfs: bool,

    /// Mount configured workspace into `/workspace`.
    ///
    /// When enabled, the workspace must exist and canonicalize before Docker
    /// command construction.
    #[serde(default = "default_true")]
    pub mount_workspace: bool,

    /// Optional workspace root allowlist for fail-closed Docker mount validation: when `mount_workspace` is enabled, the workspace must exist and canonicalize even when this list is empty; every configured root must also exist and canonicalize; one invalid entry rejects the command before Docker starts; an empty list permits any canonical workspace.
    #[serde(default)]
    pub allowed_workspace_roots: Vec<String>,
}

fn default_docker_image() -> String {
    "alpine:3.20".into()
}

fn default_docker_network() -> String {
    "none".into()
}

fn default_docker_memory_limit_mb() -> Option<u64> {
    Some(512)
}

fn default_docker_cpu_limit() -> Option<f64> {
    Some(1.0)
}

impl Default for DockerRuntimeConfig {
    fn default() -> Self {
        Self {
            image: default_docker_image(),
            network: default_docker_network(),
            memory_limit_mb: default_docker_memory_limit_mb(),
            cpu_limit: default_docker_cpu_limit(),
            read_only_rootfs: true,
            mount_workspace: true,
            allowed_workspace_roots: Vec::new(),
        }
    }
}
