//! Security configuration: audit logging, OTP, e-stop, leak detection,
//! WebAuthn, Nevis IAM/SSO, and the sandbox backend.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::<SecurityType>` path is unchanged.

use super::*;

// ── Security Config ─────────────────────────────────────────────────

/// Security configuration for audit logging, OTP, e-stop, IAM/SSO, WebAuthn,
/// and the host's NAT64 egress boundary.
///
/// Sandbox backend and resource limits live on per-agent risk profiles
/// (see `RiskProfileConfig::sandbox_*` and `RiskProfileConfig::max_*`); the
/// runtime resolves them via `Config::active_risk_profile(agent_alias)`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "security"]
pub struct SecurityConfig {
    /// Network-specific RFC 6052 NAT64 prefixes deployed on this host's
    /// network, for example
    /// `[security] nat64_prefixes = ["2001:db8:122:344::/96"]`. Default: `[]`.
    ///
    /// A NAT64 translator delivers any IPv6 destination inside one of these
    /// prefixes to the IPv4 address embedded in it. The prefix is an
    /// organization's own choice and nothing in the address reveals it, so
    /// without this list an attacker-controlled hostname can resolve to an
    /// apparently-global IPv6 address that the local translator maps to
    /// `10.0.0.1` or `169.254.169.254`, and the outbound SSRF checks accept
    /// it. Declaring the prefixes in use makes the address they translate to
    /// part of the validation boundary for `http_request`, `web_fetch`, and
    /// `text_browser`.
    ///
    /// Each entry is `<ipv6>/<length>` with one of RFC 6052 §2.2's lengths —
    /// 32, 40, 48, 56, 64, or 96 — and no bits set beyond that length. A
    /// malformed entry fails construction of the tools that read this list
    /// rather than being skipped, so a typo cannot silently narrow the
    /// boundary.
    ///
    /// The default is correct for deployments that run no NAT64 translator and
    /// for those that use only the well-known `64:ff9b::/96` prefix, which is
    /// classified without configuration.
    ///
    /// RFC 8215 also standardizes `64:ff9b:1::/48` as the local-use NAT64
    /// block, and it still needs declaring. The public-address validator
    /// already denies the whole block as non-global, so the default is safe on
    /// its own. But that non-global check is precisely what the private-host
    /// opt-in relaxes, and the metadata gate that remains unconditional
    /// underneath it decodes only the well-known `/96`, not this block. A
    /// deployment that translates from `64:ff9b:1::/48` must therefore declare
    /// the specific prefix it uses here; otherwise, once a host is allowed
    /// through `allowed_private_hosts`, an address in that block reaches the
    /// translator's embedded IPv4 destination without that destination being
    /// classified.
    ///
    /// Declared prefixes may overlap, and an address inside several of them
    /// decodes to a *different* IPv4 destination under each. Validation is
    /// conservative: an address is accepted only when every declared
    /// translation it matches lands somewhere acceptable, so a nested prefix
    /// that decodes an address to a private or metadata destination denies it
    /// even when a broader prefix decodes the same address globally. If that
    /// refuses destinations you consider legitimate, narrow the declared
    /// prefixes to the translations actually deployed.
    #[serde(default)]
    pub nat64_prefixes: Vec<String>,

    /// Audit logging configuration
    #[serde(default)]
    #[nested]
    pub audit: AuditConfig,

    /// Outbound credential leak detection and redaction configuration. See
    /// `[security.leak_detection]` for the detector controls.
    #[serde(default)]
    #[nested]
    pub leak_detection: LeakDetectionConfig,

    /// OTP gating configuration for sensitive actions/domains.
    #[serde(default)]
    #[nested]
    pub otp: OtpConfig,

    /// Emergency-stop state machine configuration.
    #[serde(default)]
    #[nested]
    pub estop: EstopConfig,

    /// Nevis IAM integration for SSO/MFA authentication and role-based access.
    #[serde(default)]
    #[nested]
    pub nevis: NevisConfig,

    /// WebAuthn / FIDO2 hardware key authentication configuration.
    #[serde(default)]
    #[nested]
    pub webauthn: WebAuthnConfig,
}

/// Outbound credential leak detection configuration.
///
/// These settings control the final guardrail pass over outbound channel
/// responses before they are delivered. Deterministic credential patterns
/// include API keys, private keys, database URLs, bot tokens, and related
/// token syntax. The high-entropy pass is a separate heuristic for standalone
/// opaque tokens.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "security.leak_detection"]
pub struct LeakDetectionConfig {
    /// Enable outbound credential leak detection and redaction.
    #[serde(default = "default_leak_detection_enabled")]
    pub enabled: bool,

    /// Detection sensitivity from 0.0 to 1.0; higher is more aggressive.
    #[serde(default = "default_leak_detection_sensitivity")]
    pub sensitivity: f64,

    /// Enable high-entropy token redaction; deterministic patterns still run when false.
    #[serde(default = "default_leak_detection_high_entropy_tokens")]
    pub high_entropy_tokens: bool,
}

fn default_leak_detection_enabled() -> bool {
    true
}

fn default_leak_detection_sensitivity() -> f64 {
    0.7
}

fn default_leak_detection_high_entropy_tokens() -> bool {
    true
}

impl Default for LeakDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: default_leak_detection_enabled(),
            sensitivity: default_leak_detection_sensitivity(),
            high_entropy_tokens: default_leak_detection_high_entropy_tokens(),
        }
    }
}

/// WebAuthn / FIDO2 hardware key authentication configuration (`[security.webauthn]`).
///
/// Enables registration and authentication via hardware security keys
/// (YubiKey, SoloKey, etc.) and platform authenticators (Touch ID, Windows Hello).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "security.webauthn"]
pub struct WebAuthnConfig {
    /// Enable WebAuthn authentication. Default: false.
    #[serde(default)]
    pub enabled: bool,
    /// Relying Party ID (domain name, e.g. "example.com"). Default: "localhost".
    #[serde(default = "default_webauthn_rp_id")]
    pub rp_id: String,
    /// Relying Party origin URL (e.g. `"https://example.com"`). Default: `"http://localhost:42617"`.
    #[serde(default = "default_webauthn_rp_origin")]
    pub rp_origin: String,
    /// Relying Party display name. Default: "OmnesAgent".
    #[serde(default = "default_webauthn_rp_name")]
    pub rp_name: String,
}

impl Default for WebAuthnConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rp_id: default_webauthn_rp_id(),
            rp_origin: default_webauthn_rp_origin(),
            rp_name: default_webauthn_rp_name(),
        }
    }
}

fn default_webauthn_rp_id() -> String {
    "localhost".into()
}

fn default_webauthn_rp_origin() -> String {
    "http://localhost:42617".into()
}

fn default_webauthn_rp_name() -> String {
    "OmnesAgent".into()
}

/// OTP validation strategy.
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum OtpMethod {
    /// Time-based one-time password (RFC 6238).
    #[default]
    Totp,
    /// Future method for paired-device confirmations.
    Pairing,
    /// Future method for local CLI challenge prompts.
    CliPrompt,
}

/// Security OTP configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "security.otp"]
#[serde(deny_unknown_fields)]
pub struct OtpConfig {
    /// Enable OTP gating. Defaults to disabled for backward compatibility.
    #[serde(default)]
    pub enabled: bool,

    /// OTP method.
    #[serde(default)]
    pub method: OtpMethod,

    /// TOTP time-step in seconds.
    #[serde(default = "default_otp_token_ttl_secs")]
    pub token_ttl_secs: u64,

    /// Reuse window for recently validated OTP codes.
    #[serde(default = "default_otp_cache_valid_secs")]
    pub cache_valid_secs: u64,

    /// Tool/action names gated by OTP. Empty or malformed entries are rejected
    /// at config load; an entry that does not match a known gated action is
    /// accepted but logged at WARN, since it cannot be enforced.
    #[serde(default = "default_otp_gated_actions")]
    pub gated_actions: Vec<String>,

    /// Explicit domain patterns gated by OTP.
    #[serde(default)]
    pub gated_domains: Vec<String>,

    /// Domain-category presets expanded into `gated_domains`.
    #[serde(default)]
    pub gated_domain_categories: Vec<String>,

    /// Maximum number of OTP challenge attempts before lockout.
    #[serde(default = "default_otp_challenge_max_attempts")]
    pub challenge_max_attempts: u32,
}

fn default_otp_token_ttl_secs() -> u64 {
    30
}

fn default_otp_cache_valid_secs() -> u64 {
    300
}

fn default_otp_challenge_max_attempts() -> u32 {
    3
}

impl Default for OtpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            method: OtpMethod::Totp,
            token_ttl_secs: default_otp_token_ttl_secs(),
            cache_valid_secs: default_otp_cache_valid_secs(),
            gated_actions: default_otp_gated_actions(),
            gated_domains: Vec::new(),
            gated_domain_categories: Vec::new(),
            challenge_max_attempts: default_otp_challenge_max_attempts(),
        }
    }
}

/// Emergency stop configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "security.estop"]
#[serde(deny_unknown_fields)]
pub struct EstopConfig {
    /// Enable emergency stop controls.
    #[serde(default)]
    pub enabled: bool,

    /// File path used to persist estop state.
    #[serde(default = "default_estop_state_file")]
    pub state_file: String,

    /// Require a valid OTP before resume operations.
    #[serde(default = "default_true")]
    pub require_otp_to_resume: bool,
}

fn default_estop_state_file() -> String {
    default_path_under_config_dir("estop-state.json")
}

impl Default for EstopConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            state_file: default_estop_state_file(),
            require_otp_to_resume: true,
        }
    }
}

/// Nevis IAM integration configuration.
///
/// When `enabled` is true, OmnesAgent validates incoming requests against a Nevis
/// Security Suite instance and maps Nevis roles to tool/workspace permissions.
#[derive(Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "security.nevis"]
#[serde(deny_unknown_fields)]
pub struct NevisConfig {
    /// Enable Nevis IAM integration. Defaults to false for backward compatibility.
    #[serde(default)]
    pub enabled: bool,

    /// Base URL of the Nevis instance (e.g. `https://nevis.example.com`).
    #[serde(default)]
    pub instance_url: String,

    /// Nevis realm to authenticate against.
    #[serde(default = "default_nevis_realm")]
    pub realm: String,

    /// OAuth2 client ID registered in Nevis.
    #[serde(default)]
    pub client_id: String,

    /// OAuth2 client secret. Encrypted via SecretStore when stored on disk.
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub client_secret: Option<String>,

    /// Token validation strategy: `"local"` (JWKS) or `"remote"` (introspection).
    #[serde(default = "default_nevis_token_validation")]
    pub token_validation: String,

    /// JWKS endpoint URL for local token validation.
    #[serde(default)]
    pub jwks_url: Option<String>,

    /// Nevis role to OmnesAgent permission mappings.
    #[serde(default)]
    pub role_mapping: Vec<NevisRoleMappingConfig>,

    /// Require MFA verification for all Nevis-authenticated requests.
    #[serde(default)]
    pub require_mfa: bool,

    /// Session timeout in seconds.
    #[serde(default = "default_nevis_session_timeout_secs")]
    pub session_timeout_secs: u64,
}

impl std::fmt::Debug for NevisConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NevisConfig")
            .field("enabled", &self.enabled)
            .field("instance_url", &self.instance_url)
            .field("realm", &self.realm)
            .field("client_id", &self.client_id)
            .field(
                "client_secret",
                &self.client_secret.as_ref().map(|_| "[REDACTED]"),
            )
            .field("token_validation", &self.token_validation)
            .field("jwks_url", &self.jwks_url)
            .field("role_mapping", &self.role_mapping)
            .field("require_mfa", &self.require_mfa)
            .field("session_timeout_secs", &self.session_timeout_secs)
            .finish()
    }
}

impl NevisConfig {
    /// Validate that required fields are present when Nevis is enabled.
    ///
    /// Call at config load time to fail fast on invalid configuration rather
    /// than deferring errors to the first authentication request.
    pub fn validate(&self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        if self.instance_url.trim().is_empty() {
            return Err("nevis.instance_url is required when Nevis IAM is enabled".into());
        }

        if self.client_id.trim().is_empty() {
            return Err("nevis.client_id is required when Nevis IAM is enabled".into());
        }

        if self.realm.trim().is_empty() {
            return Err("nevis.realm is required when Nevis IAM is enabled".into());
        }

        match self.token_validation.as_str() {
            "local" | "remote" => {}
            other => {
                return Err(format!(
                    "nevis.token_validation has invalid value '{other}': \
                     expected 'local' or 'remote'"
                ));
            }
        }

        if self.token_validation == "local" && self.jwks_url.is_none() {
            return Err("nevis.jwks_url is required when token_validation is 'local'".into());
        }

        if self.session_timeout_secs == 0 {
            return Err("nevis.session_timeout_secs must be greater than 0".into());
        }

        Ok(())
    }
}

fn default_nevis_realm() -> String {
    "master".into()
}

fn default_nevis_token_validation() -> String {
    "local".into()
}

fn default_nevis_session_timeout_secs() -> u64 {
    3600
}

impl Default for NevisConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            instance_url: String::new(),
            realm: default_nevis_realm(),
            client_id: String::new(),
            client_secret: None,
            token_validation: default_nevis_token_validation(),
            jwks_url: None,
            role_mapping: Vec::new(),
            require_mfa: false,
            session_timeout_secs: default_nevis_session_timeout_secs(),
        }
    }
}

/// Maps a Nevis role to OmnesAgent tool permissions and workspace access.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct NevisRoleMappingConfig {
    /// Nevis role name (case-insensitive).
    pub nevis_role: String,

    /// Tool names this role can access. Use `"all"` for unrestricted tool access.
    #[serde(default)]
    pub omnesagent_permissions: Vec<String>,

    /// Workspace names this role can access. Use `"all"` for unrestricted.
    #[serde(default)]
    pub workspace_access: Vec<String>,
}

/// Sandbox configuration for OS-level isolation
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "security.sandbox"]
pub struct SandboxConfig {
    /// Enable sandboxing (None = auto-detect, Some = explicit)
    #[serde(default)]
    pub enabled: Option<bool>,

    /// Sandbox backend to use
    #[serde(default)]
    pub backend: SandboxBackend,

    /// Custom Firejail arguments (when backend = firejail)
    #[serde(default)]
    pub firejail_args: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: None, // Auto-detect
            backend: SandboxBackend::Auto,
            firejail_args: Vec::new(),
        }
    }
}

/// Sandbox backend selection
#[derive(Debug, Clone, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum SandboxBackend {
    /// Auto-detect best available (default)
    #[default]
    Auto,
    /// Landlock (Linux kernel LSM, native)
    Landlock,
    /// Firejail (user-space sandbox)
    Firejail,
    /// Bubblewrap (user namespaces)
    Bubblewrap,
    /// Docker container isolation
    Docker,
    /// macOS sandbox-exec (Seatbelt)
    #[serde(alias = "sandbox-exec")]
    SandboxExec,
    /// No sandboxing (application-layer only)
    None,
}

/// Audit logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "security.audit"]
pub struct AuditConfig {
    /// Enable audit logging
    #[serde(default = "default_audit_enabled")]
    pub enabled: bool,

    /// Path to audit log file (relative to omnesagent dir)
    #[serde(default = "default_audit_log_path")]
    pub log_path: String,

    /// Maximum log size in MB before rotation
    #[serde(default = "default_audit_max_size_mb")]
    pub max_size_mb: u32,

    /// Sign events with HMAC for tamper evidence
    #[serde(default)]
    pub sign_events: bool,
}

fn default_audit_enabled() -> bool {
    true
}

fn default_audit_log_path() -> String {
    "audit.log".to_string()
}

fn default_audit_max_size_mb() -> u32 {
    100
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: default_audit_enabled(),
            log_path: default_audit_log_path(),
            max_size_mb: default_audit_max_size_mb(),
            sign_events: false,
        }
    }
}
