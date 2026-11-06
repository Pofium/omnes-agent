//! Memory subsystem configuration: storage backends (Postgres/pgvector,
//! Qdrant), the `[memory]` retrieval/policy/type knobs, and the semantic
//! validators (`validate_memory_semantics`, `validate_whatsapp_semantics`).
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::<MemoryType>` path is unchanged.

use super::*;

// ── Memory ───────────────────────────────────────────────────

/// Persistent storage configuration (`[storage]` section).
///
/// Storage is a two-tier alias-keyed map: `[storage.<backend>.<alias>]`,
/// parallel to `[providers.models.<type>.<alias>]`. Each backend has its own typed
/// config struct. `MemoryConfig.backend` carries a dotted reference (`"sqlite.default"`,
/// `"postgres.work"`) that resolves to one of these entries via
/// [`Config::resolve_active_storage`].
#[derive(Debug, Clone, Serialize, Deserialize, Default, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "storage"]
pub struct StorageConfig {
    /// SQLite storage instances (`[storage.sqlite.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub sqlite: HashMap<String, SqliteStorageConfig>,
    /// PostgreSQL storage instances (`[storage.postgres.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub postgres: HashMap<String, PostgresStorageConfig>,
    /// Qdrant storage instances (`[storage.qdrant.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub qdrant: HashMap<String, QdrantStorageConfig>,
    /// Markdown storage instances (`[storage.markdown.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub markdown: HashMap<String, MarkdownStorageConfig>,
    /// Lucid CLI sync instances (`[storage.lucid.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub lucid: HashMap<String, LucidStorageConfig>,
}

/// SQLite storage backend (`[storage.sqlite.<alias>]`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "storage_sqlite"]
#[serde(default)]
pub struct SqliteStorageConfig {
    /// Optional override for the SQLite database path.
    /// When unset, defaults to `<workspace_dir>/brain.db`.
    pub path: Option<String>,
    /// Maximum seconds to wait when opening the DB if it's locked.
    /// `None` waits indefinitely. Recommended max: 300.
    pub open_timeout_secs: Option<u64>,
}

/// PostgreSQL storage backend (`[storage.postgres.<alias>]`).
///
/// Holds connection parameters AND pgvector settings on one alias-keyed
/// entry; previously these lived in two separate sections.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "storage_postgres"]
#[serde(default)]
pub struct PostgresStorageConfig {
    /// Connection URL (e.g. `"postgres://user:pass@host/db"`).
    /// Accepts legacy aliases: dbURL, database_url, databaseUrl.
    #[serde(alias = "dbURL", alias = "database_url", alias = "databaseUrl")]
    #[secret]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub db_url: Option<String>,
    /// Database schema for the memory table.
    pub schema: String,
    /// Table name for memory entries.
    pub table: String,
    /// Optional connection timeout in seconds.
    pub connect_timeout_secs: Option<u64>,
    /// Enable pgvector extension for hybrid vector+keyword recall.
    pub vector_enabled: bool,
    /// Vector dimensions for pgvector embeddings.
    pub vector_dimensions: usize,
}

impl Default for PostgresStorageConfig {
    fn default() -> Self {
        Self {
            db_url: None,
            schema: default_storage_schema(),
            table: default_storage_table(),
            connect_timeout_secs: None,
            vector_enabled: false,
            vector_dimensions: default_pgvector_dimensions(),
        }
    }
}

/// Qdrant vector database backend (`[storage.qdrant.<alias>]`).
///
/// URL, collection, and API key are typed config values. Use schema-mirror env
/// overrides such as `OMNESAGENT_storage__qdrant__<alias>__url=...` for runtime
/// env injection.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "storage_qdrant"]
#[serde(default)]
pub struct QdrantStorageConfig {
    /// Qdrant server URL (e.g. `"http://localhost:6333"`).
    /// Use `OMNESAGENT_storage__qdrant__<alias>__url=...` for env injection.
    pub url: Option<String>,
    /// Collection name for storing memories.
    /// Use `OMNESAGENT_storage__qdrant__<alias>__collection=...` for env injection.
    pub collection: String,
    /// API key for Qdrant Cloud or secured instances.
    /// Use `OMNESAGENT_storage__qdrant__<alias>__api_key=...` for env injection.
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub api_key: Option<String>,
}

impl Default for QdrantStorageConfig {
    fn default() -> Self {
        Self {
            url: None,
            collection: default_qdrant_collection(),
            api_key: None,
        }
    }
}

/// Markdown directory storage (`[storage.markdown.<alias>]`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "storage_markdown"]
#[serde(default)]
pub struct MarkdownStorageConfig {
    /// Optional override for the markdown root directory.
    /// When unset, defaults to `<workspace_dir>/memory/`.
    pub directory: Option<String>,
}

/// Lucid CLI sync backend (`[storage.lucid.<alias>]`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "storage_lucid"]
#[serde(default)]
pub struct LucidStorageConfig {
    /// Optional path to the lucid-memory binary. When unset, the bare `lucid`
    /// executable is resolved on `PATH`. A blank or whitespace-only value is
    /// rejected at config validation and remains invalid if startup continues,
    /// so an invalid explicit selector never falls through to `PATH`.
    pub binary_path: Option<String>,
    /// Recall (context) timeout override, in milliseconds. Lucid CLI cold
    /// starts (loading the local embedding model) can exceed 1.5s on ARM
    /// hosts; raise this if recalls still time out on your hardware.
    /// Unset falls back to the built-in default (3000ms). `0` is rejected
    /// at config validation; because startup warns and continues after
    /// validation errors, a `0` that reaches the runtime is treated as
    /// unset and falls back to the default.
    pub recall_timeout_ms: Option<u64>,
    /// Store timeout override, in milliseconds. Same cold-start
    /// consideration as `recall_timeout_ms`. Unset falls back to the
    /// built-in default (3000ms); `0` is rejected at validation and, if
    /// startup continues after the warning, is likewise treated as unset.
    pub store_timeout_ms: Option<u64>,
}

fn default_storage_schema() -> String {
    "public".into()
}

fn default_storage_table() -> String {
    "memories".into()
}

fn default_qdrant_collection() -> String {
    "omnesagent_memories".into()
}

/// Search strategy for memory recall.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, omnesagent_macros::ConfigEnum)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum SearchMode {
    /// Pure keyword search (FTS5 BM25)
    Bm25,
    /// Pure vector/semantic search
    Embedding,
    /// Weighted combination of keyword + vector (default)
    #[default]
    Hybrid,
}

/// Memory backend configuration (`[memory]` section).
///
/// Controls conversation memory storage, embeddings, hybrid search, response
/// caching, and memory snapshot/hydration. Backend-specific connection settings
/// live under `[storage.<backend>.<alias>]`; this section selects which storage
/// instance to use via the `backend` dotted reference.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "memory"]
#[allow(clippy::struct_excessive_bools)]
pub struct MemoryConfig {
    /// Dotted reference to the active storage instance: `<backend>.<alias>`
    /// (e.g. `"sqlite.default"`, `"postgres.work"`). Resolves through
    /// `Config.storage.<backend>.<alias>` at runtime. Bare backend names
    /// (`"sqlite"`) are treated as `"<backend>.default"`. Set to `"none"` to
    /// disable persistence entirely.
    #[serde(default = "default_memory_backend")]
    pub backend: String,
    /// Auto-save what *you* tell OmnesAgent into memory as conversation history — the agent's own replies are not saved. Turn off if you want memory to only hold things you explicitly record via the memory tool.
    #[serde(default = "default_auto_save")]
    pub auto_save: bool,
    /// Run the periodic hygiene pass that archives stale daily/session files and enforces retention windows. Leave on unless you want to manage cleanup yourself.
    #[serde(default = "default_hygiene_enabled")]
    pub hygiene_enabled: bool,
    /// Also extract atomic durable facts from each consolidated turn and store
    /// them as individual Core memories. Default off; the flip is sequenced in
    /// a later phase. SQLite-only: enabling this requires the sqlite memory
    /// backend globally and on every agent (validated at config load).
    #[serde(default)]
    pub consolidation_extract_facts: bool,
    /// Move daily/session files to the archive directory after this many days. Keeps the hot working set small without deleting history.
    #[serde(default = "default_archive_after_days")]
    pub archive_after_days: u32,
    /// Delete archived files permanently after this many days. Set high if you need long-term history; set low for privacy / disk-space reasons.
    #[serde(default = "default_purge_after_days")]
    pub purge_after_days: u32,
    /// Delete conversation rows older than this many days from the DB (sqlite backend only). Age is measured by `updated_at` (last write time). 0 = keep forever.
    #[serde(default = "default_conversation_retention_days")]
    pub conversation_retention_days: u32,
    /// Delete daily memory rows older than this many days from the DB. Age is measured by `updated_at` (last write time). 0 = keep forever.
    #[serde(default = "default_zero_retention")]
    pub daily_retention_days: u32,
    /// Delete core memory rows older than this many days from the DB. Age is measured by `created_at` (first-write time). Neither recall nor ordinary rewrites refresh `created_at` under the current SQLite upsert, so core retention is an absolute age limit from first write. Set this to a generously large window for durable core memories, or keep 0 = keep forever.
    #[serde(default = "default_zero_retention")]
    pub core_retention_days: u32,
    /// Source of embedding vectors for semantic search. `none` = keyword-only retrieval (no API calls, no vector cost); `openai` = OpenAI's embedding API; `custom:URL` = any OpenAI-compatible embedding endpoint (LiteLLM, local gateway, etc.).
    #[serde(default = "default_embedding_provider")]
    pub embedding_provider: String,
    /// Embedding model identifier — must match a model your chosen embedding model_provider serves (e.g. `text-embedding-3-small` for OpenAI). Changing this invalidates existing embeddings: the change is detected at startup and stale vectors are cleared automatically; run `omnesagent memory reindex` to re-embed (or set `auto_reindex_on_identity_change`).
    #[serde(default = "default_embedding_model")]
    pub embedding_model: String,
    /// Vector width produced by the embedding model — must match the model's native dimension or vectors won't store correctly. Look up the number on the model_provider's model page.
    #[serde(default = "default_embedding_dims")]
    pub embedding_dimensions: usize,
    /// Automatically re-embed all memories in the background when a change of embedding provider/model/dimensions is detected at startup (after the stale vectors have been cleared). Costs one embedding API call per memory, so it's off by default — leave it off for large stores and run `omnesagent memory reindex` explicitly instead.
    #[serde(default)]
    pub auto_reindex_on_identity_change: bool,
    /// Optional API key for the embedding endpoint. When set, embedding calls use this key instead of inheriting one from the seed model provider — decoupling embeddings from the chat model. Use it when the chat model runs on a provider that carries no usable embedding credential (e.g. an OAuth-only provider) while embeddings keep hitting an `openai`/`custom:` endpoint with their own key. Leave unset to inherit the seed provider's key (backward-compatible default).
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding_api_key: Option<String>,
    /// How heavily vector (semantic) similarity counts when `search_mode = hybrid`. Raise toward 1.0 to favor meaning-based matches; lower it to lean on keyword overlap instead.
    #[serde(default = "default_vector_weight")]
    pub vector_weight: f64,
    /// How heavily BM25 (keyword) overlap counts when `search_mode = hybrid`. Raise toward 1.0 for exact-term matching; lower it when paraphrases should still score well.
    #[serde(default = "default_keyword_weight")]
    pub keyword_weight: f64,
    /// How memories are retrieved: `bm25` = keyword-only (no embeddings, cheapest); `embedding` = vector similarity only (needs an embedding model_provider); `hybrid` = blended keyword + vector score using the weights above (most robust).
    #[serde(default)]
    pub search_mode: SearchMode,
    /// Minimum hybrid score (0.0–1.0) for a memory to be included in context.
    /// Memories scoring below this threshold are dropped to prevent irrelevant
    /// context from bleeding into conversations. Default: 0.4
    #[serde(default = "default_min_relevance_score")]
    pub min_relevance_score: f64,
    /// Max embedding cache entries before LRU eviction
    #[serde(default = "default_cache_size")]
    pub embedding_cache_size: usize,
    /// Max tokens per chunk for document splitting
    #[serde(default = "default_chunk_size")]
    pub chunk_max_tokens: usize,

    // ── Response Cache (saves tokens on repeated prompts) ──────
    /// Enable LLM response caching to avoid paying for duplicate prompts
    #[serde(default)]
    pub response_cache_enabled: bool,
    /// TTL in minutes for cached responses (default: 60)
    #[serde(default = "default_response_cache_ttl")]
    pub response_cache_ttl_minutes: u32,
    /// Max number of cached responses before LRU eviction (default: 5000)
    #[serde(default = "default_response_cache_max")]
    pub response_cache_max_entries: usize,
    /// Max in-memory hot cache entries for the two-tier response cache (default: 256)
    #[serde(default = "default_response_cache_hot_entries")]
    pub response_cache_hot_entries: usize,

    // ── Memory Snapshot (soul backup to Markdown) ─────────────
    /// Enable periodic export of core memories to MEMORY_SNAPSHOT.md
    #[serde(default)]
    pub snapshot_enabled: bool,
    /// Run snapshot during hygiene passes (heartbeat-driven)
    #[serde(default)]
    pub snapshot_on_hygiene: bool,
    /// Auto-hydrate from MEMORY_SNAPSHOT.md when brain.db is missing
    #[serde(default = "default_true")]
    pub auto_hydrate: bool,

    // ── Retrieval Pipeline ─────────────────────────────────────
    /// Retrieval stages for per-agent recall. Only `"cache"` is active: it
    /// enables an in-process, per-handle hot cache over recall results and is
    /// omitted from the default so recall stays coherent across handles.
    /// `"fts"` and `"vector"` are reserved for when the backend exposes
    /// distinct FTS and vector operations; recall is a single hybrid backend
    /// call today, so those names have no effect (kept for forward compat).
    #[serde(default = "default_retrieval_stages")]
    pub retrieval_stages: Vec<String>,
    /// Candidate pool multiplier over the final recall limit before blend/rerank trimming.
    /// Values must be in 1..=20; runtime also enforces a bounded candidate pool.
    #[serde(default = "default_candidate_multiplier")]
    pub candidate_multiplier: usize,
    /// Enable the recall rerank stage: blend retrieval score with importance
    /// and recency, collapse near-duplicate entries, then trim back to the
    /// recall limit. The advanced strategy below runs when the candidate
    /// count reaches `rerank_threshold`.
    #[serde(default)]
    pub rerank_enabled: bool,
    /// Minimum candidate count to trigger the advanced rerank strategy.
    #[serde(default = "default_rerank_threshold")]
    pub rerank_threshold: usize,
    /// Advanced rerank strategy. Valid: "none", "mmr".
    #[serde(default = "default_rerank_strategy")]
    pub rerank_strategy: String,
    /// MMR relevance-vs-diversity weight, where 1.0 means relevance-only.
    #[serde(default = "default_mmr_lambda")]
    pub mmr_lambda: f64,
    /// Importance weight used by the recall blend.
    #[serde(default = "default_importance_weight")]
    pub importance_weight: f64,
    /// Recency weight used by the recall blend.
    #[serde(default = "default_recency_weight")]
    pub recency_weight: f64,
    /// Reserved (0.0-1.0): the FTS score above which recall would skip the
    /// vector stage. Inert until the backend exposes distinct FTS and vector
    /// operations; recall is a single hybrid call today, so this has no effect.
    #[serde(default = "default_fts_early_return_score")]
    pub fts_early_return_score: f64,
    /// Prefix injected memory entries with their storage date
    /// (`[from YYYY-MM-DD]`), so the model can tell recalled text from the
    /// live turn it is prepended to.
    #[serde(default = "default_true")]
    pub recall_date_stamp: bool,

    // ── Namespace Isolation ─────────────────────────────────────
    /// Default namespace for memory entries.
    #[serde(default = "default_namespace")]
    pub default_namespace: String,

    // ── Conflict Resolution ─────────────────────────────────────
    /// Cosine similarity threshold for conflict detection (0.0–1.0).
    #[serde(default = "default_conflict_threshold")]
    pub conflict_threshold: f64,
    /// Enable reversible supersede soft-hide machinery when wired.
    #[serde(default = "default_conflict_supersede_enabled")]
    pub conflict_supersede_enabled: bool,
    /// Enable write-time near-duplicate detection.
    #[serde(default)]
    pub dedup_on_write: bool,
    /// Jaccard threshold for text-only duplicate detection.
    #[serde(default = "default_dedup_jaccard_threshold")]
    pub dedup_jaccard_threshold: f64,
    /// Action to take when a duplicate is detected.
    #[serde(default)]
    pub dedup_action: MemoryDedupAction,

    // ── Memory Budget / Pinning ────────────────────────────────
    /// Maximum Core rows before budget compaction. 0 = unbounded.
    #[serde(default)]
    pub core_max_rows: u64,
    /// Maximum Core bytes before budget compaction. 0 = unbounded.
    #[serde(default)]
    pub core_max_bytes: u64,
    /// Maximum Daily rows before budget compaction. 0 = unbounded.
    #[serde(default)]
    pub daily_max_rows: u64,
    /// Eviction ordering for budget compaction.
    #[serde(default)]
    pub evict_order: MemoryEvictOrder,
    /// Namespaces protected from budget eviction.
    #[serde(default)]
    pub pin_namespaces: Vec<String>,
    /// Pin entries at or above this importance. >1.0 means disabled.
    #[serde(default = "default_pin_min_importance")]
    pub pin_min_importance: f64,

    // ── Audit Trail ─────────────────────────────────────────────
    /// Enable audit logging of memory operations.
    #[serde(default)]
    pub audit_enabled: bool,
    /// Retention period for audit entries in days (default: 30).
    #[serde(default = "default_audit_retention_days")]
    pub audit_retention_days: u32,

    // ── Policy Engine ───────────────────────────────────────────
    /// Memory policy configuration.
    #[serde(default)]
    #[nested]
    pub policy: MemoryPolicyConfig,

    /// Typed memory configuration (`[memory.types]` section).
    #[serde(default)]
    #[nested]
    pub types: MemoryTypesConfig,
    // Backend-specific config fields (sqlite_open_timeout_secs, qdrant.*,
    // postgres.*) live on `[storage.<backend>.<alias>]`. The `backend` field
    // carries a dotted alias reference and the runtime looks up the typed
    // config via `Config::resolve_active_storage`.
}

/// Typed memory configuration (`[memory.types]` section).
///
/// Behaviour-neutral by default: `enabled` gates MemoryKind assignment on new
/// consolidation writes and defaults off; the flip is sequenced in a later
/// phase. SQLite-only: enabling requires the sqlite memory backend globally
/// and on every agent (validated at config load).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "memory.types"]
pub struct MemoryTypesConfig {
    /// Assign a first-class MemoryKind to new consolidation writes.
    #[serde(default)]
    pub enabled: bool,
}

/// Memory policy configuration (`[memory.policy]` section).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "memory.policy"]
pub struct MemoryPolicyConfig {
    /// Maximum entries per namespace (0 = unlimited).
    #[serde(default)]
    pub max_entries_per_namespace: usize,
    /// Maximum entries per category (0 = unlimited).
    #[serde(default)]
    pub max_entries_per_category: usize,
    /// Retention days by category (overrides global). Keys: "core", "daily", "conversation".
    #[serde(default)]
    pub retention_days_by_category: std::collections::HashMap<String, u32>,
    /// Namespaces that are read-only (writes are rejected).
    #[serde(default)]
    pub read_only_namespaces: Vec<String>,
    /// Content scan mode for durable memory writes: "off", "on", or "strict".
    #[serde(default = "default_memory_threat_scan")]
    pub threat_scan: String,
    /// Re-scan stored entries at recall/read time and withhold flagged entries.
    #[serde(default = "default_true")]
    pub threat_scan_load_time: bool,
    /// Behavior when a write-time content scan matches: "reject" or
    /// "block-on-read".
    #[serde(default = "default_memory_threat_scan_on_hit")]
    pub threat_scan_on_hit: String,
    /// Redact configured secret/PII categories before persistence.
    #[serde(default)]
    pub redact_on_write: bool,
    /// Redaction categories applied when `redact_on_write` is true.
    #[serde(default = "default_memory_redact_categories")]
    pub redact_categories: Vec<String>,
}

impl Default for MemoryPolicyConfig {
    fn default() -> Self {
        Self {
            max_entries_per_namespace: 0,
            max_entries_per_category: 0,
            retention_days_by_category: std::collections::HashMap::new(),
            read_only_namespaces: Vec::new(),
            threat_scan: default_memory_threat_scan(),
            threat_scan_load_time: true,
            threat_scan_on_hit: default_memory_threat_scan_on_hit(),
            redact_on_write: false,
            redact_categories: default_memory_redact_categories(),
        }
    }
}

fn default_memory_threat_scan() -> String {
    "on".into()
}

fn default_memory_threat_scan_on_hit() -> String {
    "reject".into()
}

fn default_memory_redact_categories() -> Vec<String> {
    ["secret", "api_key", "private_key", "email", "phone"]
        .into_iter()
        .map(String::from)
        .collect()
}

fn default_retrieval_stages() -> Vec<String> {
    // No "cache": the hot cache is opt-in so activating the retrieval decorator
    // does not change default per-agent recall. "fts"/"vector" are reserved and
    // inert (recall is a single hybrid backend call today).
    vec!["fts".into(), "vector".into()]
}
fn default_rerank_threshold() -> usize {
    5
}

/// Largest allowed multiplier for the bounded query-time rerank candidate pool.
pub const MAX_MEMORY_RERANK_CANDIDATE_MULTIPLIER: usize = 20;

fn default_candidate_multiplier() -> usize {
    4
}
fn default_rerank_strategy() -> String {
    "none".into()
}
fn default_mmr_lambda() -> f64 {
    0.7
}
fn default_importance_weight() -> f64 {
    0.2
}
fn default_recency_weight() -> f64 {
    0.1
}
fn default_fts_early_return_score() -> f64 {
    0.85
}

fn default_namespace() -> String {
    "default".into()
}
fn default_conflict_threshold() -> f64 {
    0.85
}
fn default_conflict_supersede_enabled() -> bool {
    true
}
fn default_dedup_jaccard_threshold() -> f64 {
    0.80
}
fn default_pin_min_importance() -> f64 {
    1.01
}

/// Surface `[memory]` knobs that the schema accepts but that have no runtime
/// consumer yet, so an operator who sets them learns they currently have no
/// effect instead of debugging missing behavior (same class of check as the
/// `wire_api_not_supported_for_family` warning).
///
/// A PR that wires a consumer for one of these knobs must drop its check
/// here in the same change; this list mirrors what is inert on the current
/// tree, not what is planned.
///
/// Called from `Config::collect_warnings`, so each warning reaches both the
/// CLI (via `validate()`'s tracing emission) and the gateway dashboard.
pub fn validate_memory_semantics(
    memory: &MemoryConfig,
) -> Vec<crate::validation_warnings::ValidationWarning> {
    let mut inert: Vec<(&'static str, &'static str)> = Vec::new();

    // The staged retrieval pipeline (`RetrievalPipeline`) exists but is not
    // wired into the production recall path, so its tuning knobs are inert.
    if memory.retrieval_stages != default_retrieval_stages() {
        inert.push((
            "memory.retrieval_stages",
            "the staged retrieval pipeline is not wired into the recall path yet",
        ));
    }
    if (memory.fts_early_return_score - default_fts_early_return_score()).abs() > f64::EPSILON {
        inert.push((
            "memory.fts_early_return_score",
            "the staged retrieval pipeline is not wired into the recall path yet",
        ));
    }

    // The proposed rerank stage was never landed, so these knobs remain inert.
    // `DefaultMemoryStrategy::new` logs the same fact at agent start; this
    // config-time copy reaches `config validate` and dashboard callers too.
    if memory.rerank_enabled {
        inert.push((
            "memory.rerank_enabled",
            "the rerank stage is not yet implemented",
        ));
    }
    if memory.rerank_threshold != default_rerank_threshold() {
        inert.push((
            "memory.rerank_threshold",
            "the rerank stage is not yet implemented",
        ));
    }

    inert
        .into_iter()
        .map(|(path, reason)| {
            crate::validation_warnings::ValidationWarning::new(
                "memory_config_knob_inert",
                format!("{path} is set but {reason}; this setting currently has no effect"),
                path,
            )
        })
        .collect()
}

/// Surface WhatsApp chat-policy keys that are accepted but never consulted.
///
/// `self_chat_mode` is read only by the Web transport inside its
/// `mode == Personal` block, so under `mode = "business"` it validates cleanly
/// and has no effect. `mode` selects OmnesAgent's policy posture, not a WhatsApp
/// account type: both modes drive the same linked-device session, and the
/// self-chat affordance is scoped to the personal branch by design.
///
/// `dm_policy` and `group_policy` are consulted under BOTH modes, so they are
/// not reported here.
///
/// `allowed_groups` is separate. It is the group-identity gate and is consulted
/// under both modes, so it is never inert. `is_group_chat_allowed` returns true
/// for an empty list only under `group_policy = "all"`; under every other policy
/// an empty list admits no group. That closure is reported below by its own
/// warning rather than as an inert key.
///
/// Warnings only, no behaviour change to the validator itself. But be precise
/// about WHICH reliance is reported, because this sentence used to promise more
/// than the function delivers: under `mode = "business"` the inert-key warning
/// covers `self_chat_mode` ONLY. `dm_policy` and `group_policy` are now live in
/// business mode, so an operator who was relying on the old permissive business
/// behaviour gets NO warning here and can start silently dropping DMs or groups
/// under the default `allowlist`. That upgrade note belongs in the channel book
/// and the release notes; `config validate` will not surface it for them.
///
/// Called from `Config::collect_warnings`, so this reaches the CLI and the
/// gateway dashboard on the same path as the other warnings.
pub fn validate_whatsapp_semantics(
    alias: &str,
    wa: &WhatsAppConfig,
) -> Vec<crate::validation_warnings::ValidationWarning> {
    let mut out = Vec::new();
    // Gate on the backend the runtime will actually select, not on whether a
    // Web selector is merely present. `is_web_config` is true whenever any Web
    // selector is set, including alongside `phone_number_id`, and that
    // combination runs as Cloud. The Cloud transport consults none of the keys
    // below, so diagnosing them against a Cloud channel reports a Web gate that
    // never runs. The ambiguity itself is already surfaced separately at
    // startup, so it is not restated here.
    if !wa.enabled || wa.backend_type() != "web" {
        return out;
    }

    if wa.mode != WhatsAppWebMode::Personal {
        let mut inert: Vec<(&'static str, String)> = Vec::new();
        if wa.self_chat_mode {
            inert.push((
                "self_chat_mode",
                "self-chat handling is unchanged by this setting".to_string(),
            ));
        }
        for (key, effect) in inert {
            out.push(crate::validation_warnings::ValidationWarning::new(
                "whatsapp_chat_policy_inert",
                format!(
                    "channels.whatsapp.{alias}.{key} is set but the Web transport only \
                     consults it under mode = \"personal\"; as configured, {effect}"
                ),
                format!("channels.whatsapp.{alias}.{key}"),
            ));
        }
    }

    // The empty group-identity gate moved from admit-all to policy-selected.
    // The warning is therefore a migration notice about a capability the
    // operator is LOSING, not a fail-open alarm, and only `allowlist` loses
    // anything. Two policies must stay quiet, because neither changed:
    //
    //   group_policy = "all"      admits every group before and after.
    //   group_policy = "ignore"   rejects every group before and after. It is
    //                             mode-independent in the chat-type gate, so
    //                             this holds for business as well as personal;
    //                             only the gate doing the rejecting moved.
    //
    // The predicate is shared with the Web transport's startup notice rather
    // than restated here, so `config validate` and the runtime cannot drift
    // into disagreeing about which configurations changed.
    if wa.allowed_groups.is_empty()
        && whatsapp_empty_group_list_is_newly_closed(&wa.mode, &wa.group_policy)
    {
        let remedy = whatsapp_empty_group_list_remedy(&wa.group_policy);
        out.push(crate::validation_warnings::ValidationWarning::new(
            "whatsapp_empty_group_list_serves_no_group",
            format!(
                "channels.whatsapp.{alias}.allowed_groups is empty and \
                 group_policy is \"allowlist\", so this channel answers NO \
                 group. An empty list used to admit every group at the \
                 identity gate; that gate is now decided by group_policy. To \
                 restore group access, {remedy}."
            ),
            format!("channels.whatsapp.{alias}.allowed_groups"),
        ));
    }

    out
}

/// Write-time duplicate handling policy for memory entries.
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum MemoryDedupAction {
    /// Reject an incoming near-duplicate.
    #[default]
    Reject,
    /// Merge an incoming near-duplicate into a survivor.
    Merge,
}

/// Memory budget eviction order.
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum MemoryEvictOrder {
    /// Evict the lowest-value rows first.
    #[default]
    Value,
    /// Evict the oldest rows first.
    Oldest,
}

fn default_audit_retention_days() -> u32 {
    30
}

fn default_pgvector_dimensions() -> usize {
    1536
}

fn default_embedding_provider() -> String {
    "none".into()
}
fn default_auto_save() -> bool {
    true
}
fn default_hygiene_enabled() -> bool {
    true
}
fn default_archive_after_days() -> u32 {
    7
}
fn default_purge_after_days() -> u32 {
    30
}
fn default_conversation_retention_days() -> u32 {
    30
}
fn default_zero_retention() -> u32 {
    0
}
fn default_embedding_model() -> String {
    "text-embedding-3-small".into()
}
fn default_embedding_dims() -> usize {
    1536
}
fn default_vector_weight() -> f64 {
    0.7
}
fn default_keyword_weight() -> f64 {
    0.3
}
fn default_min_relevance_score() -> f64 {
    0.4
}
fn default_cache_size() -> usize {
    10_000
}
fn default_chunk_size() -> usize {
    512
}
fn default_response_cache_ttl() -> u32 {
    60
}
fn default_response_cache_max() -> usize {
    5_000
}

fn default_response_cache_hot_entries() -> usize {
    256
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            backend: "sqlite".into(),
            auto_save: true,
            hygiene_enabled: default_hygiene_enabled(),
            consolidation_extract_facts: false,
            archive_after_days: default_archive_after_days(),
            purge_after_days: default_purge_after_days(),
            conversation_retention_days: default_conversation_retention_days(),
            daily_retention_days: default_zero_retention(),
            core_retention_days: default_zero_retention(),
            embedding_provider: default_embedding_provider(),
            embedding_model: default_embedding_model(),
            embedding_dimensions: default_embedding_dims(),
            auto_reindex_on_identity_change: false,
            embedding_api_key: None,
            vector_weight: default_vector_weight(),
            keyword_weight: default_keyword_weight(),
            search_mode: SearchMode::default(),
            min_relevance_score: default_min_relevance_score(),
            embedding_cache_size: default_cache_size(),
            chunk_max_tokens: default_chunk_size(),
            response_cache_enabled: false,
            response_cache_ttl_minutes: default_response_cache_ttl(),
            response_cache_max_entries: default_response_cache_max(),
            response_cache_hot_entries: default_response_cache_hot_entries(),
            snapshot_enabled: false,
            snapshot_on_hygiene: false,
            auto_hydrate: true,
            retrieval_stages: default_retrieval_stages(),
            candidate_multiplier: default_candidate_multiplier(),
            rerank_enabled: false,
            rerank_threshold: default_rerank_threshold(),
            rerank_strategy: default_rerank_strategy(),
            mmr_lambda: default_mmr_lambda(),
            importance_weight: default_importance_weight(),
            recency_weight: default_recency_weight(),
            fts_early_return_score: default_fts_early_return_score(),
            recall_date_stamp: true,
            default_namespace: default_namespace(),
            conflict_threshold: default_conflict_threshold(),
            conflict_supersede_enabled: default_conflict_supersede_enabled(),
            dedup_on_write: false,
            dedup_jaccard_threshold: default_dedup_jaccard_threshold(),
            dedup_action: MemoryDedupAction::default(),
            core_max_rows: 0,
            core_max_bytes: 0,
            daily_max_rows: 0,
            evict_order: MemoryEvictOrder::default(),
            pin_namespaces: Vec::new(),
            pin_min_importance: default_pin_min_importance(),
            audit_enabled: false,
            audit_retention_days: default_audit_retention_days(),
            policy: MemoryPolicyConfig::default(),
            types: MemoryTypesConfig::default(),
        }
    }
}
