//! Editor REST API (`/api/v1/editor/*`) — `BACKEND_SPEC.md` §3.2 and §9.
//!
//! Phase F0 subset: read a file into a gateway buffer, atomic write, buffer
//! registry listing/close. The editing data path itself is the
//! `/ws/editor/{buffer_id}` channel (`ws_editor.rs`); tree/symbols/search/git
//! endpoints arrive with phases F1+ per `PLAN_FILE_EDITOR_ZED.md` §6.

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::AppState;
use super::api::require_auth;
use omnesagent_editor::{EditorError, EditorService};

#[derive(Debug, Deserialize, Default)]
pub struct EditorFileQuery {
    /// Absolute path of the project file.
    pub path: Option<String>,
    /// Project root to register on first use (`BACKEND_SPEC` §9.9, invariant 5:
    /// buffers must live inside a registered working root).
    pub root: Option<String>,
    /// `PUT` only: overwrite even when an open dirty buffer would be reloaded.
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Deserialize)]
pub struct EditorWriteBody {
    pub content: String,
    #[serde(default)]
    pub create_parents: bool,
}

#[derive(Debug, Serialize)]
pub struct EditorFileResponse {
    pub buffer_id: u64,
    pub path: String,
    pub rev: u64,
    pub content: String,
    pub language: &'static str,
    pub eol: &'static str,
    pub encoding: &'static str,
    pub read_only: bool,
    pub dirty: bool,
    pub num_lines: u32,
    pub size_bytes: usize,
}

/// `GET /api/v1/editor/file?path=<abs>&root=<abs>` — open (or reuse) the
/// gateway buffer for the file and return its metadata + content.
pub async fn handle_editor_file_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<EditorFileQuery>,
) -> Response {
    if let Err(e) = require_auth(&state, &headers) {
        return e.into_response();
    }
    let Some(path) = q.path.as_deref().map(PathBuf::from) else {
        return editor_error_response(EditorError::InvalidOp("missing `path`".into()));
    };
    if let Some(root) = q.root.as_deref() {
        if let Err(e) = state.editor.register_root(std::path::Path::new(root)) {
            return editor_error_response(e);
        }
    }
    match open_and_describe(&state.editor, &path) {
        Ok(resp) => Json(resp).into_response(),
        Err(e) => editor_error_response(e),
    }
}

fn open_and_describe(
    svc: &EditorService,
    path: &std::path::Path,
) -> Result<EditorFileResponse, EditorError> {
    let (buffer_id, _created) = svc.open_file(path)?;
    let info = svc.info(buffer_id)?;
    let page = svc.rows(buffer_id, 0, info.num_lines)?;
    let content = page
        .rows
        .iter()
        .map(|r| r.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    Ok(EditorFileResponse {
        buffer_id: info.buffer_id,
        path: info.path,
        rev: info.rev,
        content,
        language: info.language,
        eol: info.eol,
        encoding: "utf-8",
        read_only: info.read_only,
        dirty: info.dirty,
        num_lines: info.num_lines,
        size_bytes: info.size_bytes,
    })
}

/// `PUT /api/v1/editor/file?path=<abs>&root=<abs>` — atomic write of client
/// content (temp file + rename). If a buffer is open for the path and dirty,
/// the write is refused (409) unless `force` — молчаливая потеря правок
/// запрещена (`BACKEND_SPEC` §9.6). A clean open buffer reloads from disk.
pub async fn handle_editor_file_put(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<EditorFileQuery>,
    Json(body): Json<EditorWriteBody>,
) -> Response {
    if let Err(e) = require_auth(&state, &headers) {
        return e.into_response();
    }
    let Some(path) = q.path.as_deref().map(PathBuf::from) else {
        return editor_error_response(EditorError::InvalidOp("missing `path`".into()));
    };
    if let Some(root) = q.root.as_deref() {
        if let Err(e) = state.editor.register_root(std::path::Path::new(root)) {
            return editor_error_response(e);
        }
    }

    // The path must sit inside a registered root even when the file does not
    // exist yet (creation case).
    if let Err(e) = state
        .editor
        .ensure_in_roots_for_write(&path, body.create_parents)
    {
        return editor_error_response(e);
    }

    if let Some(buffer_id) = state.editor.find_by_path(&path).unwrap_or(None) {
        let info = match state.editor.info(buffer_id) {
            Ok(info) => info,
            Err(e) => return editor_error_response(e),
        };
        if info.dirty && !q.force {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "error": "buffer_dirty",
                    "buffer_id": buffer_id,
                    "hint": "save or discard the open buffer first, or pass force=true"
                })),
            )
                .into_response();
        }
    }

    if body.create_parents {
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return editor_error_response(EditorError::Io(e));
            }
        }
    }

    if let Err(e) = atomic_write(&path, body.content.as_bytes()) {
        return editor_error_response(e);
    }

    // Refresh the open buffer (if any) from the new disk content.
    if let Some(buffer_id) = state.editor.find_by_path(&path).unwrap_or(None) {
        match state.editor.reload_from_disk(buffer_id) {
            Ok(info) => {
                return Json(serde_json::json!({
                    "path": info.path,
                    "buffer_id": info.buffer_id,
                    "rev": info.rev,
                    "size_bytes": info.size_bytes,
                    "reloaded": true,
                }))
                .into_response();
            }
            Err(e) => return editor_error_response(e),
        }
    }

    match std::fs::metadata(&path) {
        Ok(meta) => Json(serde_json::json!({
            "path": path.display().to_string(),
            "size_bytes": meta.len(),
            "reloaded": false,
        }))
        .into_response(),
        Err(e) => editor_error_response(EditorError::Io(e)),
    }
}

/// `GET /api/v1/editor/raw?path=<abs>&root=<abs>` — raw bytes of a project
/// file inside the registered roots (image previews, binary assets). Served
/// with the guessed MIME type; never opens an editor buffer.
pub async fn handle_editor_file_raw(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<EditorFileQuery>,
) -> Response {
    if let Err(e) = require_auth(&state, &headers) {
        return e.into_response();
    }
    let Some(path) = q.path.as_deref().map(PathBuf::from) else {
        return editor_error_response(EditorError::InvalidOp("missing `path`".into()));
    };
    if let Some(root) = q.root.as_deref() {
        if let Err(e) = state.editor.register_root(std::path::Path::new(root)) {
            return editor_error_response(e);
        }
    }
    match state.editor.read_raw(&path) {
        Ok(bytes) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream().to_string();
            Response::builder()
                .status(StatusCode::OK)
                .header(axum::http::header::CONTENT_TYPE, mime)
                .header(axum::http::header::CACHE_CONTROL, "no-store")
                .body(axum::body::Body::from(bytes))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Err(e) => editor_error_response(e),
    }
}

#[derive(Debug, Serialize)]
pub struct EditorBuffersResponse {
    pub buffers: Vec<omnesagent_editor::BufferInfo>,
    pub roots: Vec<String>,
}

/// `GET /api/v1/editor/buffers` — open buffer registry.
pub async fn handle_editor_buffers_get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if let Err(e) = require_auth(&state, &headers) {
        return e.into_response();
    }
    Json(EditorBuffersResponse {
        buffers: state.editor.list(),
        roots: state.editor.roots(),
    })
    .into_response()
}

#[derive(Debug, Deserialize, Default)]
pub struct EditorCloseQuery {
    pub buffer_id: Option<u64>,
    pub path: Option<String>,
    #[serde(default)]
    pub force: bool,
}

/// `DELETE /api/v1/editor/buffer?buffer_id=<n>|path=<abs>&force=<bool>` —
/// close a buffer; refuses dirty buffers unless `force`.
pub async fn handle_editor_buffer_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<EditorCloseQuery>,
) -> Response {
    if let Err(e) = require_auth(&state, &headers) {
        return e.into_response();
    }
    let buffer_id = if let Some(id) = q.buffer_id {
        id
    } else if let Some(path) = q.path.as_deref() {
        match state.editor.find_by_path(std::path::Path::new(path)) {
            Ok(Some(id)) => id,
            Ok(None) => {
                return editor_error_response(EditorError::NotFound(path.to_string()));
            }
            Err(e) => return editor_error_response(e),
        }
    } else {
        return editor_error_response(EditorError::InvalidOp(
            "pass `buffer_id` or `path`".into(),
        ));
    };
    match state.editor.close(buffer_id, q.force) {
        Ok(info) => Json(serde_json::json!({ "closed": true, "buffer": info })).into_response(),
        Err(e) => editor_error_response(e),
    }
}

/// Atomic write: temp file in the target directory + rename (replaces the
/// destination on both POSIX and Windows), `BACKEND_SPEC` §9.6.
pub(crate) fn atomic_write(path: &std::path::Path, bytes: &[u8]) -> Result<(), EditorError> {
    let dir = path
        .parent()
        .ok_or_else(|| EditorError::InvalidOp(format!("no parent dir: {}", path.display())))?;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp = dir.join(format!(
        ".{}.omnes-tmp-{nanos}",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
    ));
    std::fs::write(&tmp, bytes)?;
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(EditorError::Io(e));
    }
    Ok(())
}

/// Map editor errors onto HTTP status codes (mirrors `browse_error_response`).
pub(crate) fn editor_error_response(err: EditorError) -> Response {
    let status = match &err {
        EditorError::NotFound(_) => StatusCode::NOT_FOUND,
        EditorError::NotInRoot { .. } => StatusCode::FORBIDDEN,
        EditorError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        EditorError::RevMismatch { .. } => StatusCode::CONFLICT,
        EditorError::DirtyClose { .. } => StatusCode::CONFLICT,
        EditorError::ReadOnly { .. } => StatusCode::FORBIDDEN,
        EditorError::TooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
        EditorError::InvalidOp(_) => StatusCode::BAD_REQUEST,
    };
    (status, Json(serde_json::json!({ "error": format!("{err}") }))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::{delete as axum_delete, get as axum_get, put as axum_put};
    use tower::ServiceExt;

    fn editor_router(state: AppState) -> axum::Router {
        axum::Router::new()
            .route(
                "/api/v1/editor/file",
                axum_get(handle_editor_file_get).put(handle_editor_file_put),
            )
            .route("/api/v1/editor/raw", axum_get(handle_editor_file_raw))
            .route("/api/v1/editor/buffers", axum_get(handle_editor_buffers_get))
            .route("/api/v1/editor/buffer", axum_delete(handle_editor_buffer_delete))
            .with_state(state)
    }

    fn request(method: &str, uri: &str, body: Option<String>) -> axum::http::Request<axum::body::Body> {
        let mut builder = axum::http::Request::builder()
            .method(method)
            .uri(uri);
        if body.is_some() {
            builder = builder.header("content-type", "application/json");
        }
        builder
            .body(axum::body::Body::from(body.unwrap_or_default()))
            .unwrap()
    }

    fn qencode(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for b in s.bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    out.push(b as char)
                }
                _ => out.push_str(&format!("%{b:02X}")),
            }
        }
        out
    }

    #[tokio::test]
    async fn editor_file_open_write_close_roundtrip() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let file = root.join("sample.rs");
        std::fs::write(&file, "fn main() {}\n").unwrap();

        let state = crate::api::tests::test_state(omnesagent_config::schema::Config::default());
        let router = editor_router(state.clone());
        let root_str = root.display().to_string();
        let file_str = file.display().to_string();

        // Open: registers the root and creates the buffer.
        let res = router
            .clone()
            .oneshot(request(
                "GET",
                &format!(
                    "/api/v1/editor/file?path={}&root={}",
                    qencode(&file_str),
                    qencode(&root_str)
                ),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["language"], "rust");
        // "fn main() {}\n" — trailing newline yields a final empty line.
        assert_eq!(json["num_lines"], 2);
        let buffer_id = json["buffer_id"].as_u64().unwrap();

        // Edit the buffer through the service so it becomes dirty.
        state
            .editor
            .apply_edits(
                buffer_id,
                json["rev"].as_u64().unwrap(),
                &[omnesagent_editor::EditOp {
                    start: omnesagent_editor::EditorPos { line: 0, col: 0 },
                    end: omnesagent_editor::EditorPos { line: 0, col: 0 },
                    text: "// ".into(),
                }],
            )
            .unwrap();

        // PUT while dirty → 409 (молчаливая потеря правок запрещена).
        let res = router
            .clone()
            .oneshot(request(
                "PUT",
                &format!(
                    "/api/v1/editor/file?path={}",
                    qencode(&file_str)
                ),
                Some(r#"{"content":"fn main() {}"}"#.into()),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CONFLICT);

        // PUT with force → writes to disk and reloads the buffer.
        let res = router
            .clone()
            .oneshot(request(
                "PUT",
                &format!(
                    "/api/v1/editor/file?path={}&force=true",
                    qencode(&file_str)
                ),
                Some(r#"{"content":"fn main() {}\n"}"#.into()),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "fn main() {}\n");
        assert!(!state.editor.info(buffer_id).unwrap().dirty);

        // Close, then the buffer list is empty.
        let res = router
            .clone()
            .oneshot(request(
                "DELETE",
                &format!("/api/v1/editor/buffer?buffer_id={buffer_id}"),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let res = router
            .oneshot(request("GET", "/api/v1/editor/buffers", None))
            .await
            .unwrap();
        let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["buffers"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn editor_raw_serves_bytes_with_mime_inside_roots() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let file = root.join("logo.png");
        let payload: &[u8] = b"\x89PNG\r\n\x1a\nfakedata";
        std::fs::write(&file, payload).unwrap();

        let state = crate::api::tests::test_state(omnesagent_config::schema::Config::default());
        let router = editor_router(state);
        let res = router
            .oneshot(request(
                "GET",
                &format!(
                    "/api/v1/editor/raw?path={}&root={}",
                    qencode(&file.display().to_string()),
                    qencode(&root.display().to_string())
                ),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok()),
            Some("image/png")
        );
        let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&body[..], payload);
    }

    #[tokio::test]
    async fn editor_rejects_paths_outside_registered_roots() {
        let state = crate::api::tests::test_state(omnesagent_config::schema::Config::default());
        let router = editor_router(state);
        let outside = "Z:\\definitely\\not\\a\\root\\file.txt"
            .replace('\\', std::path::MAIN_SEPARATOR_STR);
        // The file does not exist and the drive is not a registered root —
        // fail closed with 404 (not-found), never leak IO details.
        let res = router
            .oneshot(request(
                "GET",
                &format!("/api/v1/editor/file?path={}", qencode(&outside)),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}
