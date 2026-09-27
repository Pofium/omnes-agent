//! WebSocket channel for open editor buffers: `/ws/editor/{buffer_id}`.
//!
//! Frame contract: `BACKEND_SPEC.md` §9.9. Phase F0 subset — `hello`,
//! `rows_snapshot` / `rows_request`, `rows_changed` (дельта строк), `edit_ops`
//! / `edit_ack` (optimistic client edits with base-rev gating), `save` /
//! `save_state`, `cursor` (accepted, relayed in F7 with presence), `error`.
//! `lsp_*`, `agent_proposal` and `folds` frames arrive with phases F5/F7/F1.
//!
//! Invariants honored here (§9.9): every row frame carries the buffer
//! revision; `rows_snapshot` is idempotent for a range; edits from the event
//! broadcast never touch the client's own undo stack; auth matches the other
//! WS channels.

use super::AppState;
use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::sync::mpsc;

use omnesagent_editor::{BufferEvent, EditOp};

/// WS `/ws/editor/{buffer_id}` — attach to an open buffer.
pub async fn handle_ws_editor(
    State(state): State<AppState>,
    Path(buffer_id): Path<u64>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    // Auth check matching the other WS channels (ws_terminal).
    if state.pairing.require_pairing() {
        let token = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|auth| auth.strip_prefix("Bearer "))
            .or_else(|| {
                headers
                    .get("sec-websocket-protocol")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|protos| {
                        protos
                            .split(',')
                            .map(|p| p.trim())
                            .find_map(|p| p.strip_prefix("bearer."))
                    })
            });

        let authorized = match token {
            Some(t) => state.pairing.is_authenticated(t),
            None => false,
        };
        if !authorized {
            return (
                StatusCode::UNAUTHORIZED,
                [("WWW-Authenticate", "Bearer")],
                "Unauthorized WebSocket request",
            )
                .into_response();
        }
    }

    // The buffer must already exist (opened via `GET /api/v1/editor/file`).
    if state.editor.info(buffer_id).is_err() {
        return (
            StatusCode::NOT_FOUND,
            "Unknown editor buffer — open the file via /api/v1/editor/file first",
        )
            .into_response();
    }

    ws.on_upgrade(move |socket| handle_editor_socket(socket, state.editor.clone(), buffer_id))
        .into_response()
}

async fn handle_editor_socket(
    socket: WebSocket,
    editor: std::sync::Arc<omnesagent_editor::EditorService>,
    buffer_id: u64,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<String>(128);

    let Ok(mut events) = editor.subscribe(buffer_id) else {
        return;
    };

    // `hello` — buffer identity, revision, settings and the shared style
    // table (one syntax theme for gateway and client, `BACKEND_SPEC` §9.4).
    let Ok(info) = editor.info(buffer_id) else {
        return;
    };
    let hello = json!({
        "type": "hello",
        "buffer_id": info.buffer_id,
        "path": info.path,
        "rev": info.rev,
        "language": info.language,
        "eol": info.eol,
        "encoding": "utf-8",
        "read_only": info.read_only,
        "total_lines": info.num_lines,
        "settings": {
            "tab_size": 4,
            "hard_tabs": false,
            "soft_wrap": "none",
        },
        "styles": omnesagent_editor::highlight::style_table(),
    });
    if tx.send(hello.to_string()).await.is_err() {
        return;
    }

    // Latest revision this connection has been told about; guards against
    // duplicate row frames after our own `edit_ops` application.
    let mut last_sent_rev = info.rev;

    let ws_pump = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    loop {
        tokio::select! {
            event = events.recv() => {
                match event {
                    Ok(BufferEvent::Edited { rev, start_line, inval_count, new_count }) => {
                        if rev > last_sent_rev {
                            if let Ok(page) = editor.rows(buffer_id, start_line, new_count) {
                                let frame = json!({
                                    "type": "rows_changed",
                                    "rev": rev,
                                    "start_line": start_line,
                                    "inval_count": inval_count,
                                    "new_count": new_count,
                                    "total_lines": page.total_lines,
                                    "rows": page.rows,
                                });
                                if tx.send(frame.to_string()).await.is_err() { break; }
                                last_sent_rev = rev;
                            }
                        }
                    }
                    Ok(BufferEvent::Saved { rev }) => {
                        let frame = json!({
                            "type": "save_state",
                            "rev": rev,
                            "dirty": false,
                            "saved_at": chrono_now_rfc3339(),
                        });
                        if tx.send(frame.to_string()).await.is_err() { break; }
                        last_sent_rev = last_sent_rev.max(rev);
                    }
                    Ok(BufferEvent::Reloaded { rev }) => {
                        // Content was replaced from outside; push a fresh
                        // snapshot of the head of the file so the client can
                        // reconcile (full resync on demand is idempotent).
                        if let Ok(page) = editor.rows(buffer_id, 0, 400) {
                            let frame = json!({
                                "type": "rows_snapshot",
                                "rev": rev,
                                "from": page.from,
                                "total_lines": page.total_lines,
                                "rows": page.rows,
                                "reason": "reloaded",
                            });
                            if tx.send(frame.to_string()).await.is_err() { break; }
                            last_sent_rev = rev;
                        }
                    }
                    Ok(BufferEvent::Closed) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        // Buffer closed (or we lagged) — end the session; the
                        // client reconnects and re-resyncs if needed.
                        let _ = tx
                            .send(json!({"type": "error", "code": "buffer_closed",
                                         "message": "buffer closed"}).to_string())
                            .await;
                        break;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            inbound = ws_receiver.next() => {
                let Some(Ok(msg)) = inbound else { break };
                match msg {
                    Message::Text(text) => {
                        let Ok(frame) = serde_json::from_str::<Value>(&text) else {
                            let _ = tx.send(json!({"type": "error", "code": "bad_frame",
                                                     "message": "frame is not valid JSON"}).to_string()).await;
                            continue;
                        };
                        match frame.get("type").and_then(|t| t.as_str()) {
                            Some("rows_request") => {
                                let from = frame.get("from").and_then(Value::as_u64).unwrap_or(0) as u32;
                                let count = frame.get("count").and_then(Value::as_u64).unwrap_or(200) as u32;
                                if let Ok(page) = editor.rows(buffer_id, from, count) {
                                    let resp = json!({
                                        "type": "rows_snapshot",
                                        "rev": page.rev,
                                        "from": page.from,
                                        "total_lines": page.total_lines,
                                        "rows": page.rows,
                                    });
                                    if tx.send(resp.to_string()).await.is_err() { break; }
                                    last_sent_rev = last_sent_rev.max(page.rev);
                                }
                            }
                            Some("edit_ops") => {
                                let base_rev = frame.get("base_rev").and_then(Value::as_u64).unwrap_or(0);
                                let op_id = frame.get("op_id").and_then(Value::as_u64).unwrap_or(0);
                                let ops: Vec<EditOp> = frame
                                    .get("ops")
                                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                                    .unwrap_or_default();
                                match editor.apply_edits(buffer_id, base_rev, &ops) {
                                    Ok(outcome) => {
                                        let ack = json!({
                                            "type": "edit_ack",
                                            "op_id": op_id,
                                            "applied": true,
                                            "rev": outcome.rev,
                                        });
                                        if tx.send(ack.to_string()).await.is_err() { break; }
                                        // `rows_changed` for every client (including
                                        // this one) is produced by the Edited event.
                                    }
                                    Err(e) => {
                                        let applied = !matches!(e, omnesagent_editor::EditorError::RevMismatch { .. });
                                        let ack = json!({
                                            "type": "edit_ack",
                                            "op_id": op_id,
                                            "applied": applied,
                                            "error": format!("{e}"),
                                            "rev": editor.info(buffer_id).map(|i| i.rev).unwrap_or(0),
                                        });
                                        if tx.send(ack.to_string()).await.is_err() { break; }
                                    }
                                }
                            }
                            Some("save") => {
                                match editor.save(buffer_id) {
                                    Ok((info, _)) => {
                                        let frame = json!({
                                            "type": "save_state",
                                            "rev": info.rev,
                                            "dirty": false,
                                            "saved_at": chrono_now_rfc3339(),
                                        });
                                        if tx.send(frame.to_string()).await.is_err() { break; }
                                        last_sent_rev = last_sent_rev.max(info.rev);
                                    }
                                    Err(e) => {
                                        let _ = tx.send(json!({
                                            "type": "error", "code": "save_failed",
                                            "message": format!("{e}"),
                                        }).to_string()).await;
                                    }
                                }
                            }
                            Some("cursor") => {
                                // F0: accepted but not relayed; multi-client
                                // presence lands with F7 (`BACKEND_SPEC` §9.7).
                            }
                            Some("ping") => {
                                let _ = tx.send(json!({"type": "pong"}).to_string()).await;
                            }
                            _ => {
                                let _ = tx.send(json!({"type": "error", "code": "unknown_frame",
                                                         "message": "unsupported frame type"}).to_string()).await;
                            }
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        }
    }

    ws_pump.abort();
}

/// RFC 3339 UTC timestamp for `save_state.saved_at`.
fn chrono_now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}
