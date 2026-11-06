//! Gateway security & transport config: the `[gateway]` server section
//! (pairing, rate limiting, TLS/mTLS, dashboard) plus the remote WSS
//! transport (`[wss]`, `[relay]`, `[enroll]`) and their nested sub-configs.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::<Type>` path is unchanged.

use super::*;

// ── Gateway security ─────────────────────────────────────────────

/// Longest supported dashboard WebSocket keepalive interval.
///
/// Longer periods do not protect typical intermediary idle deadlines, while
/// this bound keeps timer construction portable across supported targets.
pub const GATEWAY_WEBSOCKET_PING_INTERVAL_MAX_SECS: u64 = 86_400;

/// Gateway server configuration (`[gateway]` section).
///
/// Controls the HTTP gateway for webhook and pairing endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "gateway"]
#[allow(clippy::struct_excessive_bools)]
pub struct GatewayConfig {
    /// Gateway port (default: 42617)
    #[serde(default = "default_gateway_port")]
    pub port: u16,
    /// Gateway host (default: 127.0.0.1)
    #[serde(default = "default_gateway_host")]
    pub host: String,
    /// Require pairing before accepting requests (default: true)
    #[serde(default = "default_true")]
    pub require_pairing: bool,
    /// Allow binding to non-localhost without a tunnel (default: false)
    #[serde(default)]
    pub allow_public_bind: bool,
    /// Allow authenticated remote callers to use admin endpoints that are
    /// otherwise localhost-only. Currently this gates `POST /admin/reload`.
    /// When false (default), those endpoints reject any non-loopback peer.
    /// When true, a non-loopback request is accepted only if it also passes
    /// pairing authentication — which requires `require_pairing = true`; with
    /// pairing off a remote caller cannot be authenticated and is rejected, so
    /// this flag never exposes an anonymous remote reload. `/admin/shutdown`
    /// and the pairing-code endpoints stay localhost-only regardless.
    /// (default: false)
    #[serde(default)]
    pub allow_remote_admin: bool,
    /// Paired bearer tokens (managed automatically, not user-edited)
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub paired_tokens: Vec<String>,

    /// Max `/pair` requests per minute per client key.
    #[serde(default = "default_pair_rate_limit")]
    pub pair_rate_limit_per_minute: u32,

    /// Max `/webhook` requests per minute per client key.
    #[serde(default = "default_webhook_rate_limit")]
    pub webhook_rate_limit_per_minute: u32,

    /// Optional shared secret for the gateway's generic `POST /webhook` and
    /// `POST /sop/*` routes. Requests must send the exact value in
    /// `X-Webhook-Secret`. This is intentionally separate from
    /// `[channels.webhook.<alias>].secret`, which authenticates that channel's
    /// own listener with an HMAC signature.
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub webhook_secret: Option<String>,

    /// Trust proxy-forwarded client IP headers (`X-Forwarded-For`, `X-Real-IP`).
    /// Disabled by default; enable only behind a trusted reverse proxy.
    #[serde(default)]
    #[credential_class = "public_value"]
    pub trust_forwarded_headers: bool,

    /// Optional URL path prefix for reverse-proxy deployments.
    /// When set, all gateway routes are served under this prefix.
    /// Must start with `/` and must not end with `/`.
    #[serde(default)]
    pub path_prefix: Option<String>,

    /// Maximum distinct client keys tracked by gateway rate limiter maps.
    #[serde(default = "default_gateway_rate_limit_max_keys")]
    pub rate_limit_max_keys: usize,

    /// TTL for webhook idempotency keys.
    #[serde(default = "default_idempotency_ttl_secs")]
    pub idempotency_ttl_secs: u64,

    /// Maximum distinct idempotency keys retained in memory.
    #[serde(default = "default_gateway_idempotency_max_keys")]
    pub idempotency_max_keys: usize,

    /// Persist gateway WebSocket chat sessions to SQLite. Default: true.
    #[serde(default = "default_true")]
    pub session_persistence: bool,

    /// Auto-archive stale gateway sessions older than N hours. 0 = disabled. Default: 0.
    #[serde(default)]
    pub session_ttl_hours: u32,

    /// Send WebSocket ping frames every N seconds to keep dashboard chat
    /// connections alive. Range:
    /// `0..=GATEWAY_WEBSOCKET_PING_INTERVAL_MAX_SECS`. 0 = disabled. Default: 30.
    /// The value is read when each WebSocket connection opens; changing it
    /// requires reconnecting clients (or restarting the gateway) to affect
    /// existing connections.
    #[serde(default = "default_gateway_websocket_ping_interval_secs")]
    pub websocket_ping_interval_secs: u64,

    /// Pairing dashboard configuration
    #[serde(default)]
    #[nested]
    pub pairing_dashboard: PairingDashboardConfig,

    /// Path to the web dashboard `dist` directory. When set, the gateway
    /// serves the compiled frontend from the filesystem instead of requiring
    /// it to be embedded in the binary. Accepts absolute paths or paths
    /// relative to the working directory. When omitted the gateway runs in
    /// API-only mode (no web dashboard) unless auto-detection finds it.
    #[serde(default)]
    pub web_dist_dir: Option<String>,

    /// TLS configuration for the gateway server (`[gateway.tls]`).
    #[serde(default)]
    #[nested]
    pub tls: Option<GatewayTlsConfig>,

    /// HTTP request timeout (seconds) for gateway routes other than the
    /// long-running cron-trigger endpoint. Default: 30s.
    #[serde(default = "default_gateway_request_timeout_secs")]
    pub request_timeout_secs: u64,

    /// HTTP request timeout (seconds) for `POST /api/cron/{id}/run`, which
    /// runs jobs synchronously and routinely exceeds the 30s default.
    /// Default: 600s (10 minutes).
    #[serde(default = "default_gateway_long_running_request_timeout_secs")]
    pub long_running_request_timeout_secs: u64,

    /// Poll GitHub for newer releases and show an "update available" indicator
    /// on the dashboard version tag. Read-only; does not install anything.
    /// (default: true)
    #[serde(default = "default_true")]
    pub check_updates: bool,

    /// Allow triggering a self-upgrade (binary swap via `omnesagent update`) from
    /// the dashboard. This is a remote-code-execution-adjacent surface: any
    /// authenticated dashboard user could replace the running binary. Keep off
    /// unless you trust every paired client. (default: false)
    #[serde(default)]
    pub allow_self_upgrade: bool,
}

fn default_gateway_port() -> u16 {
    42617
}

fn default_gateway_request_timeout_secs() -> u64 {
    30
}

fn default_gateway_long_running_request_timeout_secs() -> u64 {
    600
}

fn default_gateway_websocket_ping_interval_secs() -> u64 {
    30
}

fn default_gateway_host() -> String {
    "127.0.0.1".into()
}

fn default_pair_rate_limit() -> u32 {
    10
}

fn default_webhook_rate_limit() -> u32 {
    60
}

fn default_idempotency_ttl_secs() -> u64 {
    300
}

fn default_gateway_rate_limit_max_keys() -> usize {
    10_000
}

fn default_gateway_idempotency_max_keys() -> usize {
    10_000
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            port: default_gateway_port(),
            host: default_gateway_host(),
            require_pairing: true,
            allow_public_bind: false,
            allow_remote_admin: false,
            paired_tokens: Vec::new(),
            pair_rate_limit_per_minute: default_pair_rate_limit(),
            webhook_rate_limit_per_minute: default_webhook_rate_limit(),
            webhook_secret: None,
            trust_forwarded_headers: false,
            path_prefix: None,
            rate_limit_max_keys: default_gateway_rate_limit_max_keys(),
            idempotency_ttl_secs: default_idempotency_ttl_secs(),
            idempotency_max_keys: default_gateway_idempotency_max_keys(),
            session_persistence: true,
            session_ttl_hours: 0,
            websocket_ping_interval_secs: default_gateway_websocket_ping_interval_secs(),
            pairing_dashboard: PairingDashboardConfig::default(),
            web_dist_dir: None,
            tls: None,
            request_timeout_secs: default_gateway_request_timeout_secs(),
            long_running_request_timeout_secs: default_gateway_long_running_request_timeout_secs(),
            check_updates: true,
            allow_self_upgrade: false,
        }
    }
}

/// Pairing dashboard configuration (`[gateway.pairing_dashboard]`).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "gateway.pairing_dashboard"]
pub struct PairingDashboardConfig {
    /// Length of pairing codes (default: 8)
    #[serde(default = "default_pairing_code_length")]
    pub code_length: usize,
    /// Time-to-live for pending pairing codes in seconds (default: 3600)
    #[serde(default = "default_pairing_ttl")]
    pub code_ttl_secs: u64,
    /// Maximum concurrent pending pairing codes (default: 3)
    #[serde(default = "default_max_pending_codes")]
    pub max_pending_codes: usize,
    /// Maximum failed pairing attempts before lockout (default: 5)
    #[serde(default = "default_max_failed_attempts")]
    pub max_failed_attempts: u32,
    /// Lockout duration in seconds after max attempts (default: 300)
    #[serde(default = "default_pairing_lockout_secs")]
    pub lockout_secs: u64,
}

fn default_pairing_code_length() -> usize {
    8
}
fn default_pairing_ttl() -> u64 {
    3600
}
fn default_max_pending_codes() -> usize {
    3
}
fn default_max_failed_attempts() -> u32 {
    5
}
fn default_pairing_lockout_secs() -> u64 {
    300
}

impl Default for PairingDashboardConfig {
    fn default() -> Self {
        Self {
            code_length: default_pairing_code_length(),
            code_ttl_secs: default_pairing_ttl(),
            max_pending_codes: default_max_pending_codes(),
            max_failed_attempts: default_max_failed_attempts(),
            lockout_secs: default_pairing_lockout_secs(),
        }
    }
}

/// TLS configuration for the gateway server (`[gateway.tls]`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "gateway.tls"]
pub struct GatewayTlsConfig {
    /// Enable TLS for the gateway (default: false).
    #[serde(default)]
    pub enabled: bool,
    /// Path to the PEM-encoded server certificate file.
    pub cert_path: String,
    /// Path to the PEM-encoded server private key file.
    pub key_path: String,
    /// Client certificate authentication (mutual TLS) settings.
    #[serde(default)]
    #[nested]
    pub client_auth: Option<GatewayClientAuthConfig>,
}

/// Client certificate authentication (mTLS) configuration (`[gateway.tls.client_auth]`).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "gateway.tls.client_auth"]
pub struct GatewayClientAuthConfig {
    /// Enable client certificate verification (default: false).
    #[serde(default)]
    pub enabled: bool,
    /// Path to the PEM-encoded CA certificate used to verify client certs.
    #[serde(default)]
    pub ca_cert_path: String,
    /// Reject connections that do not present a valid client certificate (default: true).
    #[serde(default = "default_true")]
    pub require_client_cert: bool,
    /// Optional SHA-256 fingerprints for certificate pinning.
    /// When non-empty, only client certs matching one of these fingerprints are accepted.
    #[serde(default)]
    pub pinned_certs: Vec<String>,
    /// Optional path to a revoked-fingerprint list (one SHA-256 hex per line). A
    /// client cert whose fingerprint is listed is refused at the handshake (A5);
    /// the file is re-read when it changes. Empty disables the check.
    #[serde(default)]
    pub crl_path: String,
}

impl Default for GatewayClientAuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ca_cert_path: String::new(),
            require_client_cert: default_true(),
            pinned_certs: Vec::new(),
            crl_path: String::new(),
        }
    }
}

/// Client certificate authentication (mTLS) for the remote WSS transport
/// (`[wss.client_auth]`).
///
/// This mirrors [`GatewayClientAuthConfig`]; the two are distinct structs only
/// because the `Configurable` derive binds the section prefix into the type. The
/// verification logic itself is single-sourced in the `omnesagent-tls` crate.
///
/// Unlike the gateway's variant there is no `require_client_cert` knob: the
/// remote WSS plane is *always* mutually authenticated (there is no
/// server-only-TLS path), so a client certificate is unconditionally required.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "wss.client_auth"]
pub struct WssClientAuthConfig {
    /// Use the bring-your-own CA below. When false (default) the daemon
    /// auto-generates its own CA. The WSS plane is always mutually authenticated
    /// either way; this does NOT toggle whether a client certificate is required.
    #[serde(default)]
    pub enabled: bool,
    /// Path to the PEM-encoded CA certificate used to verify client certificates.
    #[serde(default)]
    pub ca_cert_path: String,
    /// Optional SHA-256 fingerprints for certificate pinning. When non-empty,
    /// only client certs matching one of these fingerprints are accepted.
    #[serde(default)]
    pub pinned_certs: Vec<String>,
    /// Optional path to a revoked-fingerprint list (one SHA-256 hex per line). A
    /// client cert whose fingerprint is listed is refused at the handshake (A5);
    /// the file is re-read when it changes. Empty (default) makes the daemon use
    /// its ledger-materialized list at `<data_dir>/tls/revoked`.
    #[serde(default)]
    pub crl_path: String,
}

/// WebSocket Secure (WSS) transport for remote TUI-to-daemon connections (`[wss]`).
///
/// When enabled, the daemon listens for TLS-encrypted WebSocket connections
/// on the configured bind address and port. TUI clients connect via
/// `--connect wss://host:port`.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "wss"]
pub struct WssConfig {
    /// Enable the WSS listener (default: false).
    #[serde(default)]
    pub enabled: bool,
    /// Bind address for the WSS listener (default: "0.0.0.0").
    #[serde(default = "default_wss_bind")]
    pub bind: String,
    /// Port for the WSS listener (default: 9781).
    #[serde(default = "default_wss_port")]
    pub port: u16,
    /// Path to the PEM-encoded server certificate file.
    #[serde(default)]
    pub cert_path: String,
    /// Path to the PEM-encoded server private key file.
    #[serde(default)]
    pub key_path: String,
    /// Extra Subject Alternative Names for the AUTO-GENERATED server certificate
    /// (hostnames and/or IPs a remote client uses to reach this daemon, e.g.
    /// `["zero", "192.168.2.168"]`). `localhost` and `127.0.0.1` are always
    /// included. Each entry that parses as an IP becomes an IP SAN, else a DNS
    /// SAN. Changing this list regenerates the server leaf (the CA is untouched).
    /// Ignored when you bring your own server certificate via `cert_path`.
    #[serde(default)]
    pub sans: Vec<String>,
    /// Client certificate authentication (mutual TLS) for the remote WSS plane.
    /// When the listener is enabled, the remote plane is always mutually
    /// authenticated; this section supplies the CA and pinning policy.
    #[serde(default)]
    #[nested]
    pub client_auth: Option<WssClientAuthConfig>,
    /// Ceiling on sockets past `accept()` that have not yet finished the TLS
    /// handshake and WebSocket upgrade (default: 256). The remote plane is
    /// Internet-facing, so this bounds the state an UNAUTHENTICATED peer can
    /// hold; beyond it, new sockets are dropped at accept rather than queued.
    #[serde(default = "default_wss_max_pending_handshakes")]
    pub max_pending_handshakes: usize,
    /// One absolute deadline, in seconds, covering TLS accept AND the
    /// WebSocket upgrade (default: 10). A single budget for the whole setup
    /// sequence, not a fresh window per phase; the heartbeat only starts once
    /// a session is established.
    #[serde(default = "default_wss_handshake_timeout_secs")]
    pub handshake_timeout_secs: u64,
    /// Ceiling on concurrently established WSS sessions (default: 64). Bounds
    /// the steady state that survives authentication; with the 32 MiB parser
    /// envelope this is also the aggregate parser-memory ceiling (64 x 32 MiB
    /// = 2 GiB by default), so raise it as a deliberate host-memory decision.
    #[serde(default = "default_wss_max_sessions")]
    pub max_sessions: usize,
    /// Ceiling on concurrent sessions presenting ONE client certificate
    /// (default: 8). `max_sessions` alone lets a single admitted (or stolen,
    /// pre-revocation) credential occupy every session and its parser
    /// envelope; this bounds what one credential can reserve.
    #[serde(default = "default_wss_max_sessions_per_client")]
    pub max_sessions_per_client: usize,
    /// How long, in seconds, a partially-received message may be held by the
    /// parser while control frames keep the connection alive (default: 60).
    /// Also the deadline for the slowest legitimate full-size request; raise
    /// it on slow links rather than disabling the bound.
    #[serde(default = "default_wss_incomplete_message_timeout_secs")]
    pub incomplete_message_timeout_secs: u64,
}

impl Default for WssConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind: default_wss_bind(),
            port: default_wss_port(),
            cert_path: String::new(),
            key_path: String::new(),
            sans: Vec::new(),
            client_auth: None,
            max_pending_handshakes: default_wss_max_pending_handshakes(),
            handshake_timeout_secs: default_wss_handshake_timeout_secs(),
            max_sessions: default_wss_max_sessions(),
            max_sessions_per_client: default_wss_max_sessions_per_client(),
            incomplete_message_timeout_secs: default_wss_incomplete_message_timeout_secs(),
        }
    }
}

/// Nominated-relay client (`[relay]`).
///
/// When enabled, the daemon keeps a persistent outbound connection to a relay
/// and registers `node_id`, so clients behind NAT can reach it *through* the
/// relay. The relay is a blind forwarder: the inner client<->daemon mTLS still
/// terminates at the daemon's WSS listener and is never decrypted by the relay.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "relay"]
pub struct RelayConfig {
    /// Enable the relay bridge (default: false).
    #[serde(default)]
    pub enabled: bool,
    /// Relay address to connect to, as `host:port`.
    #[serde(default)]
    pub url: String,
    /// Opaque node-id this daemon registers under (clients dial this id). Leave
    /// empty (recommended) to auto-mint and persist a random 128-bit capability at
    /// `<data_dir>/relay/node_id`; set it only to pin a specific id. It must stay
    /// unguessable and is decoupled from the cert - do NOT use a name or a cert
    /// thumbprint (a guessable/derivable id lets attackers probe and flood you).
    #[serde(default)]
    pub node_id: String,
    /// Relay account token presented at registration (admission credential).
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    pub token: String,
    /// PEM CA to trust for the relay's OWN (outer) TLS certificate. When set,
    /// this explicit CA takes precedence over remembered/TOFU relay pins. Empty
    /// uses the built-in public roots unless a stored pin or `tofu` is active.
    #[serde(default)]
    pub relay_ca_path: String,
    /// Server name to expect on the relay's outer certificate. Empty derives it
    /// from the host portion of `url`.
    #[serde(default)]
    pub relay_host: String,
    /// Skip verification of the relay's outer certificate (self-signed dev only).
    #[serde(default)]
    pub relay_insecure: bool,
    /// Trust-on-first-use for the relay's OUTER certificate (default false): accept
    /// the first leaf seen and pin it at `<data_dir>/relay/relay_pin` thereafter
    /// (the pin is also handed to enrolling clients). Opt-in; ignored once a pin is
    /// stored or `relay_ca_path` is set. A configured relay CA also bypasses any
    /// previously stored TOFU pin. The outer TLS is a metadata boundary, not the
    /// RPC boundary (the inner mTLS is); see threat A2.
    #[serde(default)]
    pub tofu: bool,
    /// PEM cert/key the daemon presents to the relay on the OUTER TLS layer
    /// (outer-mTLS variant), required only when the relay sets
    /// `outer_client_auth = required`. Empty presents no outer client cert. This is
    /// separate from the inner mTLS client identity.
    #[serde(default)]
    pub outer_client_cert: String,
    #[serde(default)]
    pub outer_client_key: String,
    /// Auto-rotate the auto-minted node-id every N days (default 0 = never).
    /// Rotation mints a fresh id, registers it alongside the old one for a short
    /// grace window (so in-flight clients keep working), then retires the old id;
    /// the new id reaches clients in-band on their next certificate renewal. Only
    /// applies when `node_id` is empty (an operator-pinned id is never rotated).
    #[serde(default)]
    pub node_id_rotation_days: u64,
}

fn default_wss_bind() -> String {
    "0.0.0.0".into()
}

fn default_wss_port() -> u16 {
    9781
}

fn default_wss_max_pending_handshakes() -> usize {
    256
}

fn default_wss_handshake_timeout_secs() -> u64 {
    10
}

fn default_wss_max_sessions() -> usize {
    64
}

fn default_wss_max_sessions_per_client() -> usize {
    8
}

fn default_wss_incomplete_message_timeout_secs() -> u64 {
    60
}

fn default_enroll_bind() -> String {
    "0.0.0.0".into()
}

fn default_enroll_port() -> u16 {
    9782
}

/// Certificate enrollment endpoint (`[enroll]`).
///
/// The dedicated, narrowly scoped bootstrap surface a *certless* client reaches
/// for its FIRST certificate. It is server-authenticated TLS (the daemon proves
/// itself; the client confirms the CA via the pairing short-auth-string) plus a
/// pairing-code gate. It accepts exactly one operation: submit a CSR, receive a
/// signed cert + the CA chain + the relay profile. This is NOT a fallback on the
/// always-mTLS RPC plane (that plane stays mutually authenticated with no
/// weakenable path); it is a separate minimal endpoint with its own auth model.
/// The daemon owns the CA, so this endpoint works with no gateway.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "enroll"]
pub struct EnrollConfig {
    /// Enable the enrollment endpoint (default: false). Requires `[wss]` enabled
    /// (it hands out certs for that mutually authenticated plane) and a daemon CA
    /// private key (auto-generated, or BYO with key); when the CA key is absent
    /// the endpoint fails closed and certs must be provisioned out of band.
    #[serde(default)]
    pub enabled: bool,
    /// Address the enrollment endpoint binds on.
    #[serde(default = "default_enroll_bind")]
    pub bind: String,
    /// Port the enrollment endpoint listens on.
    #[serde(default = "default_enroll_port")]
    pub port: u16,
    /// Reserved for a future migration flow. The first FOSS release rejects any
    /// non-empty value because code-less enrollment needs an explicit client-side
    /// trust anchor before certs can be cached. Leave empty to require the normal
    /// pairing-code-gated path.
    #[serde(default)]
    pub allow_unpaired_enrollment: String,
}
