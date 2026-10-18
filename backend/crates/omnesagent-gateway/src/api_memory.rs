//! Memory REST API handlers (/api/memory).

use super::AppState;
use super::api::require_auth;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use omnesagent_memory::MemoryEntry;
use serde::Deserialize;

pub const MEMORY_API_CONTENT_MAX_CHARS: usize = 4096;

#[derive(Deserialize)]
pub struct MemoryQuery {
    pub query: Option<String>,
    pub category: Option<String>,
    /// Filter memories created at or after (RFC 3339 / ISO 8601)
    pub since: Option<String>,
    /// Filter memories created at or before (RFC 3339 / ISO 8601)
    pub until: Option<String>,
    /// When set to a configured agent alias, the request goes through
    /// that agent's per-alias memory backend (so SQL backends filter by
    /// the agent's UUID, Markdown reads only that agent's directory,
    /// etc.). Omit for the install-wide view.
    #[serde(default)]
    pub agent: Option<String>,
}

#[derive(Deserialize)]
pub struct MemoryStoreBody {
    pub key: String,
    pub content: String,
    pub category: Option<String>,
    /// Configured agent alias to write under. When omitted the store goes
    /// to the install-wide memory backend (no per-agent attribution).
    #[serde(default)]
    pub agent: Option<String>,
}

#[derive(Deserialize)]
pub struct MemoryDeleteQuery {
    /// Configured agent alias to delete from. Omit for the install-wide
    /// backend.
    #[serde(default)]
    pub agent: Option<String>,
}

pub async fn resolve_memory_handle(
    state: &AppState,
    agent_alias: Option<&str>,
) -> Result<std::sync::Arc<dyn omnesagent_memory::Memory>, (StatusCode, Json<serde_json::Value>)> {
    let alias = match agent_alias.map(str::trim).filter(|s| !s.is_empty()) {
        Some(a) => a,
        None => return Ok(state.mem.clone()),
    };
    let config = state.config.read().clone();
    if config.agent(alias).is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!(
                "Unknown agent {alias:?} (no [agents.{alias}] entry configured)"
            )})),
        ));
    }
    let api_key = config
        .resolved_model_provider_for_agent(alias)
        .and_then(|(_, _, cfg)| cfg.api_key.clone());
    omnesagent_memory::create_memory_for_agent(&config, alias, api_key.as_deref())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({"error": format!("Failed to build per-agent memory: {e:#}")}),
                ),
            )
        })
}

/// GET /api/memory — list or search memory entries
pub async fn handle_api_memory_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<MemoryQuery>,
) -> impl IntoResponse {
    if let Err(e) = require_auth(&state, &headers) {
        return e.into_response();
    }

    let mem = match resolve_memory_handle(&state, params.agent.as_deref()).await {
        Ok(m) => m,
        Err(e) => return e.into_response(),
    };

    // Use recall when query or time range is provided
    if params.query.is_some() || params.since.is_some() || params.until.is_some() {
        let query = params.query.as_deref().unwrap_or("");
        let since = params.since.as_deref();
        let until = params.until.as_deref();
        match mem.recall(query, 50, None, since, until).await {
            Ok(entries) => {
                let entries = match params.category.as_deref() {
                    Some(cat) => entries
                        .into_iter()
                        .filter(|e| e.category.to_string() == cat)
                        .collect(),
                    None => entries,
                };
                Json(serde_json::json!({
                    "entries": sanitize_memory_entries_for_api(entries)
                }))
                .into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Memory recall failed: {e}")})),
            )
                .into_response(),
        }
    } else {
        // List mode
        let category = params.category.as_deref().map(|cat| match cat {
            "core" => omnesagent_memory::MemoryCategory::Core,
            "daily" => omnesagent_memory::MemoryCategory::Daily,
            "conversation" => omnesagent_memory::MemoryCategory::Conversation,
            other => omnesagent_memory::MemoryCategory::Custom(other.to_string()),
        });

        match mem.list(category.as_ref(), None).await {
            Ok(entries) => Json(serde_json::json!({
                "entries": sanitize_memory_entries_for_api(entries)
            }))
            .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Memory list failed: {e}")})),
            )
                .into_response(),
        }
    }
}

pub fn sanitize_memory_entries_for_api(entries: Vec<MemoryEntry>) -> Vec<MemoryEntry> {
    entries
        .into_iter()
        .map(|mut entry| {
            entry.content = truncate_with_ellipsis_total_chars(entry.content);
            entry
        })
        .collect()
}

pub fn truncate_with_ellipsis_total_chars(mut s: String) -> String {
    if s.char_indices().nth(MEMORY_API_CONTENT_MAX_CHARS).is_none() {
        return s;
    }

    let keep_chars = MEMORY_API_CONTENT_MAX_CHARS - 3;
    let cut_idx = s
        .char_indices()
        .nth(keep_chars)
        .map(|(idx, _)| idx)
        .unwrap_or(s.len());
    s.truncate(cut_idx);
    s.push_str("...");
    s
}

/// POST /api/memory — store a memory entry
pub async fn handle_api_memory_store(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<MemoryStoreBody>,
) -> impl IntoResponse {
    if let Err(e) = require_auth(&state, &headers) {
        return e.into_response();
    }

    let category = body
        .category
        .as_deref()
        .map(|cat| match cat {
            "core" => omnesagent_memory::MemoryCategory::Core,
            "daily" => omnesagent_memory::MemoryCategory::Daily,
            "conversation" => omnesagent_memory::MemoryCategory::Conversation,
            other => omnesagent_memory::MemoryCategory::Custom(other.to_string()),
        })
        .unwrap_or(omnesagent_memory::MemoryCategory::Core);

    let mem = match resolve_memory_handle(&state, body.agent.as_deref()).await {
        Ok(m) => m,
        Err(e) => return e.into_response(),
    };

    match mem.store(&body.key, &body.content, category, None).await {
        Ok(()) => Json(serde_json::json!({"status": "ok"})).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Memory store failed: {e}")})),
        )
            .into_response(),
    }
}

/// DELETE /api/memory/:key — delete a memory entry
pub async fn handle_api_memory_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(key): Path<String>,
    Query(query): Query<MemoryDeleteQuery>,
) -> impl IntoResponse {
    if let Err(e) = require_auth(&state, &headers) {
        return e.into_response();
    }

    let mem = match resolve_memory_handle(&state, query.agent.as_deref()).await {
        Ok(m) => m,
        Err(e) => return e.into_response(),
    };

    match mem.forget(&key).await {
        Ok(deleted) => {
            Json(serde_json::json!({"status": "ok", "deleted": deleted})).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Memory forget failed: {e}")})),
        )
            .into_response(),
    }
}
