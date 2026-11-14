//! Plugin system configuration (the `[plugins]` section): `PluginsConfig`,
//! `PluginSecurityConfig` (`[plugins.security]`), `PluginLimitsConfig`
//! (`[plugins.limits]`), and the per-instance `PluginEntryConfig`
//! (`[plugins.entries]`).
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::{PluginsConfig, PluginEntryConfig, ...}` path is unchanged.

use super::*;

/// More than one canonical config row exists for the same plugin instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("duplicate plugin config entries for one instance key")]
pub struct DuplicatePluginConfigEntry;

/// Per-instance plugin config keyed by the host-derived `PluginInstanceId`
/// config-entry key; values are secret so they encrypt at rest under the same
/// adjacent `.secret_key` as every other secret.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "plugins.entries"]
pub struct PluginEntryConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub config: HashMap<String, String>,
    /// Destinations this plugin instance may reach. Default: empty,
    /// which means **no network reach at all** — a transport permission such as
    /// `http_client` grants the surface, this field grants the destinations.
    ///
    /// Entries are exact hosts (`api.example.com`, `10.0.0.5`) or explicit
    /// suffix patterns (`*.example.com`, which matches subdomains but **not**
    /// the apex — list the apex separately if it is needed). A bare domain
    /// never implies its subdomains, and there is no `*` meaning "anywhere".
    /// Ports are not part of an entry: granting a host grants every port on it.
    ///
    /// Deliberately a **plaintext sibling** of the `#[secret]` `config` map and
    /// never inside it: the allowlist is the thing an operator audits, so it
    /// has to stay readable in the file they are auditing.
    #[serde(default)]
    pub egress_hosts: Vec<String>,
    /// Subset of `egress_hosts` additionally permitted to resolve to a private,
    /// loopback, or link-local address (a self-hosted Gitea, a LAN Nextcloud).
    /// Mirrors the `allowed_private_hosts` semantics the host tools already use.
    ///
    /// This relaxes the *address class* only. It never widens `egress_hosts` —
    /// a host listed here but not there is still denied — and cloud metadata
    /// addresses stay refused regardless of what is listed, including through a
    /// configured `security.nat64_prefixes` translation. Unlike the tool-layer
    /// field, `*` is not accepted here.
    #[serde(default)]
    pub egress_allow_private: Vec<String>,
}

/// Plugin system configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "plugins"]
pub struct PluginsConfig {
    /// Enable the plugin system (default: false)
    #[serde(default)]
    pub enabled: bool,
    /// Directory where plugins are stored
    #[serde(default = "default_plugins_dir")]
    pub plugins_dir: String,
    /// Auto-discover and load plugins on startup (default: false)
    ///
    /// This gates the package-bound *tool* and *skill* instances the activation
    /// plan discovers from installed manifests. Explicit
    /// `[channels.plugin.<alias>]` declarations are operator-named, not
    /// discovered, so they activate without it.
    #[serde(default)]
    pub auto_discover: bool,
    /// Maximum number of logical plugin instances admitted across capabilities.
    ///
    /// This counts admitted *instances*, not installed packages: one package
    /// that provides both a channel binding and a tool consumes two. It
    /// replaces the never-enforced `plugins.max_plugins` key, which counted
    /// packages; because the units differ, an old `max_plugins` value is not
    /// carried over. See the plugin activation docs for the migration note.
    #[serde(default = "default_max_active_plugin_instances")]
    pub max_active_instances: usize,
    /// Plugin signature verification security settings
    #[serde(default)]
    #[nested]
    pub security: PluginSecurityConfig,
    /// Per-call WASM execution limits
    #[serde(default)]
    #[nested]
    pub limits: PluginLimitsConfig,
    #[serde(default)]
    #[nested]
    #[natural_key = "name"]
    pub entries: Vec<PluginEntryConfig>,
}

impl PluginsConfig {
    pub fn entry_config(
        &self,
        instance_key: &str,
    ) -> Result<Option<&HashMap<String, String>>, DuplicatePluginConfigEntry> {
        let mut matches = self
            .entries
            .iter()
            .filter(|entry| entry.name == instance_key);
        let first = matches.next().map(|entry| &entry.config);
        if matches.next().is_some() {
            return Err(DuplicatePluginConfigEntry);
        }
        Ok(first)
    }

    /// The granted egress allowlist and private-address carveout for `alias`,
    /// as `(egress_hosts, egress_allow_private)`.
    ///
    /// A missing entry returns two empty lists, which is deny-everything. This
    /// is the read the plugin host performs **per request**, so an operator's
    /// edit applies without re-instantiating the guest (resolved from live
    /// config at use time
    /// resolution). Both lists are returned as authored; they were validated by
    /// [`Config::validate`] at load, and the matcher re-canonicalizes nothing.
    #[must_use]
    pub fn entry_egress(&self, alias: &str) -> (Vec<String>, Vec<String>) {
        self.entries
            .iter()
            .find(|e| e.name == alias)
            .map(|e| (e.egress_hosts.clone(), e.egress_allow_private.clone()))
            .unwrap_or_default()
    }
}

impl PluginsConfig {
    /// Resolve `plugins_dir` to an absolute path, expanding a leading `~/`.
    ///
    /// This is the single source of truth for the plugin directory: the CLI,
    /// runtime tool/skill discovery, and the gateway all resolve through it so
    /// that `omnesagent plugin install` and the agent agree on one location.
    /// Pure — performs no filesystem I/O.
    #[must_use]
    pub fn resolved_plugins_dir(&self) -> PathBuf {
        expand_tilde_path(&self.plugins_dir)
    }
}

/// Plugin signature verification configuration (`[plugins.security]`).
///
/// Controls Ed25519 signature verification for plugin manifests.
/// In `strict` mode, only plugins signed by a trusted publisher key are loaded.
/// In `permissive` mode, unsigned or untrusted plugins produce warnings but are
/// still loaded. In `disabled` mode (the default), no signature checking occurs.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "plugins.security"]
pub struct PluginSecurityConfig {
    /// Signature enforcement mode: "disabled", "permissive", or "strict".
    #[serde(default = "default_signature_mode")]
    pub signature_mode: String,
    /// Hex-encoded Ed25519 public keys of trusted plugin publishers.
    #[serde(default)]
    pub trusted_publisher_keys: Vec<String>,
}

fn default_signature_mode() -> String {
    "disabled".to_string()
}

impl Default for PluginSecurityConfig {
    fn default() -> Self {
        Self {
            signature_mode: default_signature_mode(),
            trusted_publisher_keys: Vec::new(),
        }
    }
}

/// Per-call WASM execution limits (`[plugins.limits]`).
///
/// Bounds a single plugin call so a runaway or malicious component traps
/// instead of hanging the host or exhausting memory. `call_fuel` caps
/// instructions per call; `call_timeout_ms` caps elapsed wall-clock time,
/// including time awaiting async host imports; the memory, table, and instance
/// ceilings bound a store's growth. Every value is operator-tunable and
/// validated as non-zero.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "plugins.limits"]
pub struct PluginLimitsConfig {
    /// Fuel budget per plugin call (wasmtime instruction units).
    #[serde(default = "default_plugin_call_fuel")]
    pub call_fuel: u64,
    /// Maximum linear memory a plugin store may grow to, in megabytes.
    #[serde(default = "default_plugin_max_memory_mb")]
    pub max_memory_mb: usize,
    /// Maximum table elements a plugin store may allocate.
    #[serde(default = "default_plugin_max_table_elements")]
    pub max_table_elements: usize,
    /// Maximum component instances a plugin store may create.
    #[serde(default = "default_plugin_max_instances")]
    pub max_instances: usize,
    /// Wall-clock deadline for one plugin export call, in milliseconds.
    #[serde(default = "default_plugin_call_timeout_ms")]
    pub call_timeout_ms: u64,
    /// Maximum live host-owned network connections per logical plugin instance,
    /// shared across every transport and every store belonging to it.
    ///
    /// Unlike the ceilings above this one spans calls rather than bounding a
    /// single call, because a connection outlives the call that opened it.
    #[serde(default = "default_plugin_max_connections_per_instance")]
    pub max_connections_per_instance: usize,
}

fn default_plugin_call_fuel() -> u64 {
    1_000_000_000
}

fn default_plugin_max_memory_mb() -> usize {
    256
}

fn default_plugin_max_table_elements() -> usize {
    100_000
}

fn default_plugin_max_instances() -> usize {
    64
}

fn default_plugin_call_timeout_ms() -> u64 {
    30_000
}

fn default_plugin_max_connections_per_instance() -> usize {
    16
}

impl Default for PluginLimitsConfig {
    fn default() -> Self {
        Self {
            call_fuel: default_plugin_call_fuel(),
            max_memory_mb: default_plugin_max_memory_mb(),
            max_table_elements: default_plugin_max_table_elements(),
            max_instances: default_plugin_max_instances(),
            call_timeout_ms: default_plugin_call_timeout_ms(),
            max_connections_per_instance: default_plugin_max_connections_per_instance(),
        }
    }
}

fn default_plugins_dir() -> String {
    default_path_under_config_dir("plugins")
}

fn default_max_active_plugin_instances() -> usize {
    50
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            plugins_dir: default_plugins_dir(),
            auto_discover: false,
            max_active_instances: default_max_active_plugin_instances(),
            security: PluginSecurityConfig::default(),
            limits: PluginLimitsConfig::default(),
            entries: Vec::new(),
        }
    }
}
