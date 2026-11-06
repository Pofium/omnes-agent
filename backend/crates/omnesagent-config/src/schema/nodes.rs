//! Dynamic node discovery config (the `[nodes]` section): external
//! processes/devices connecting over `/ws/nodes`, plus LAN-local mDNS peer
//! discovery (`[nodes.mdns]`).
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::{NodesConfig, MdnsConfig}` are unchanged.

use super::*;

// ── Nodes (Dynamic Node Discovery) ───────────────────────────────

fn default_mdns_announce_interval_secs() -> u64 {
    30
}

fn default_mdns_peer_ttl_secs() -> u64 {
    90
}

fn default_mdns_max_peers() -> usize {
    16
}

/// Configuration for LAN-local mDNS peer discovery (`[nodes.mdns]`).
///
/// This config controls only discovery behavior. The advertised gateway
/// endpoint is derived from the running gateway's actual host, port, and path
/// prefix at startup so `[nodes.mdns]` does not duplicate gateway listen state.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "nodes.mdns"]
pub struct MdnsConfig {
    /// Enable mDNS local peer discovery.
    #[serde(default)]
    pub enabled: bool,
    /// Human-readable node name advertised to LAN peers. Defaults to a stable
    /// local fallback when unset.
    #[serde(default)]
    pub node_name: Option<String>,
    /// Maximum number of unauthenticated LAN peer hints retained in memory.
    #[serde(default = "default_mdns_max_peers")]
    pub max_peers: usize,
    /// How often this node re-broadcasts its presence, in seconds.
    #[serde(default = "default_mdns_announce_interval_secs")]
    pub announce_interval_secs: u64,
    /// Seconds after the last announcement before a peer is evicted.
    #[serde(default = "default_mdns_peer_ttl_secs")]
    pub peer_ttl_secs: u64,
}

impl Default for MdnsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            node_name: None,
            max_peers: default_mdns_max_peers(),
            announce_interval_secs: default_mdns_announce_interval_secs(),
            peer_ttl_secs: default_mdns_peer_ttl_secs(),
        }
    }
}

/// Configuration for the dynamic node discovery system (`[nodes]`).
///
/// When enabled, external processes/devices can connect via WebSocket
/// at `/ws/nodes` and advertise their capabilities at runtime.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "nodes"]
pub struct NodesConfig {
    /// Enable dynamic node discovery endpoint.
    #[serde(default)]
    pub enabled: bool,
    /// Maximum number of concurrent node connections.
    #[serde(default = "default_max_nodes")]
    pub max_nodes: usize,
    /// Optional bearer token for node authentication.
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub auth_token: Option<String>,
    /// LAN-local mDNS peer discovery.
    #[serde(default)]
    #[nested]
    pub mdns: MdnsConfig,
}

fn default_max_nodes() -> usize {
    16
}

impl Default for NodesConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_nodes: default_max_nodes(),
            auth_token: None,
            mdns: MdnsConfig::default(),
        }
    }
}
