//! Autonomy/security policy surface: the auto-approve defaults and the
//! `impl RiskProfileConfig` policy helpers (effective tool allowlist, sandbox
//! config). The `RiskProfileConfig` type itself lives in `profiles_bundles`.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::{default_auto_approve, ...}` paths are unchanged.

use super::*;

//
// All policy fields live on per-agent `[risk_profiles.<alias>]` entries
// (see `RiskProfileConfig` below). `Config::active_risk_profile(agent_alias)`
// resolves the active profile for any callsite (agent-driven or non-agent
// contexts). Configs from older schema versions are folded into
// `risk_profiles.default` by the migration in `schema/v2.rs`.

pub fn default_auto_approve() -> Vec<String> {
    vec![
        "file_read".into(),
        "memory_recall".into(),
        "web_search_tool".into(),
        "web_fetch".into(),
        "calculator".into(),
        "glob_search".into(),
        "content_search".into(),
        "image_info".into(),
        "weather".into(),
        "tool_search".into(),
        "browser".into(),
        "browser_open".into(),
    ]
}

pub(crate) fn default_always_ask() -> Vec<String> {
    vec![]
}

impl RiskProfileConfig {
    /// Legacy serialized marker used by older operators to represent an
    /// explicit deny-all profile before `deny_all_tools` existed.
    pub const LEGACY_DENY_ALL_TOOLS_SENTINEL: &'static str = "__none__";

    /// Resolve the profile's effective tool allowlist without changing the
    /// persisted representation. `None` is unrestricted, while `Some([])` is
    /// explicit deny-all. Mixed legacy sentinel lists retain only real tool
    /// names.
    #[must_use]
    pub fn effective_allowed_tools(&self) -> Option<Vec<String>> {
        if self.deny_all_tools {
            return Some(Vec::new());
        }

        let real = self
            .allowed_tools
            .iter()
            .filter(|name| name.as_str() != Self::LEGACY_DENY_ALL_TOOLS_SENTINEL)
            .cloned()
            .collect::<Vec<_>>();
        if !self.allowed_tools.is_empty() && real.is_empty() {
            return Some(Vec::new());
        }
        (!real.is_empty()).then_some(real)
    }

    /// Merge the built-in default `auto_approve` entries into the current
    /// list, preserving any user-supplied additions.
    pub fn ensure_default_auto_approve(&mut self) {
        let defaults = default_auto_approve();
        for entry in defaults {
            if !self.auto_approve.iter().any(|existing| existing == &entry) {
                self.auto_approve.push(entry);
            }
        }
    }

    /// Synthesize a [`SandboxConfig`] from this profile's flattened sandbox
    /// fields. Sandbox config is stored flat on the profile; callsites that
    /// still want a `SandboxConfig` instance (sandbox detection in
    /// `omnesagent-runtime::security::detect`) can call this helper.
    #[must_use]
    pub fn sandbox_config(&self) -> SandboxConfig {
        let backend = self
            .sandbox_backend
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(parse_sandbox_backend)
            .unwrap_or_default();
        SandboxConfig {
            enabled: self.sandbox_enabled,
            backend,
            firejail_args: self.firejail_args.clone(),
        }
    }
}

fn parse_sandbox_backend(name: &str) -> SandboxBackend {
    match name.to_ascii_lowercase().as_str() {
        "auto" => SandboxBackend::Auto,
        "landlock" => SandboxBackend::Landlock,
        "firejail" => SandboxBackend::Firejail,
        "bubblewrap" => SandboxBackend::Bubblewrap,
        "docker" => SandboxBackend::Docker,
        "sandbox-exec" | "sandboxexec" | "seatbelt" => SandboxBackend::SandboxExec,
        "none" => SandboxBackend::None,
        _ => SandboxBackend::default(),
    }
}

pub(crate) fn is_valid_env_var_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}
