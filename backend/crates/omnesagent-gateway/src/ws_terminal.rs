//! Terminal WebSocket bridge for OmnesAgent Web ADE.
//! Provides bidirectional streaming for interactive shell sessions over WebSocket.

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
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

/// WS /ws/terminal/:id — terminal WebSocket connection.
pub async fn handle_ws_terminal(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    // Auth check matching gateway pairing pattern
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

    ws.on_upgrade(move |socket| handle_terminal_socket(socket, id))
        .into_response()
}

async fn handle_terminal_socket(socket: WebSocket, session_id: String) {
    eprintln!("[ws_terminal] Terminal WebSocket connected: session_id={}", session_id);

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<String>(128);

    // Initial banner
    let welcome = serde_json::json!({
        "type": "stdout",
        "data": format!("OmnesAgent Gateway Terminal Session [{}]\r\n", session_id),
    });
    let _ = tx.send(welcome.to_string()).await;

    #[cfg(windows)]
    let mut cmd = Command::new("powershell.exe");
    #[cfg(windows)]
    cmd.arg("-NoLogo");

    #[cfg(not(windows))]
    let mut cmd = Command::new("/bin/sh");

    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            let err_msg = serde_json::json!({
                "type": "stdout",
                "data": format!("Failed to spawn shell: {}\r\n", e),
            });
            let _ = ws_sender.send(Message::Text(err_msg.to_string().into())).await;
            return;
        }
    };

    let mut stdin = child.stdin.take().expect("stdin piped");
    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");

    // Task 1: Pump messages from mpsc channel -> WebSocket
    let ws_pump_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Task 2: Read stdout -> mpsc
    let tx_stdout = tx.clone();
    let stdout_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let msg = serde_json::json!({
                "type": "stdout",
                "data": format!("{}\r\n", line),
            });
            if tx_stdout.send(msg.to_string()).await.is_err() {
                break;
            }
        }
    });

    // Task 3: Read stderr -> mpsc
    let tx_stderr = tx.clone();
    let stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let msg = serde_json::json!({
                "type": "stdout",
                "data": format!("{}\r\n", line),
            });
            if tx_stderr.send(msg.to_string()).await.is_err() {
                break;
            }
        }
    });

    // Main loop: Read WS messages -> write to stdin
    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(cmd_data) = json.get("data").and_then(|d| d.as_str()) {
                        let _ = stdin.write_all(cmd_data.as_bytes()).await;
                        let _ = stdin.flush().await;
                    }
                } else {
                    let _ = stdin.write_all(text.as_bytes()).await;
                    let _ = stdin.flush().await;
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    let _ = child.kill().await;
    stdout_task.abort();
    stderr_task.abort();
    ws_pump_task.abort();
    eprintln!("[ws_terminal] Terminal WebSocket closed: session_id={}", session_id);
}
