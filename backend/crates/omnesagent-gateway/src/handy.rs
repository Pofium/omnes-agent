//! Gateway HTTP handlers and SSE integration for Handy offline STT dictation.
//!
//! Provides endpoints:
//! - `GET  /api/voice/handy/status`
//! - `POST /api/voice/handy/install`
//! - `POST /api/voice/handy/install/cancel`
//! - `POST /api/voice/handy/launch`
//! - `POST /api/voice/handy/stop`
//! - `POST /api/voice/handy/toggle`
//! - `POST /api/voice/handy/uninstall`
//! - `GET  /api/voice/handy/models`
//! - `POST /api/voice/handy/transcribe`

use super::AppState;
use super::api::require_auth;
use axum::{
    extract::{ConnectInfo, Multipart, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use omnesagent_tools::handy::{
    detect::detect_handy,
    installer::HandyInstaller,
    launcher::{launch_handy, list_models, stop_handy, toggle_transcription, transcribe_file},
    manifest::fetch_manifest,
    HandyError,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;

/// Maximum payload limit for audio files sent to `/api/voice/handy/transcribe` (25 MB).
pub const HANDY_MAX_TRANSCRIBE_BYTES: usize = 25 * 1024 * 1024;

/// Check that the request originated from the local loopback interface.
fn check_local_only(addr: &SocketAddr) -> Result<(), Response> {
    if !addr.ip().is_loopback() {
        let body = serde_json::json!({
            "error": "local_only",
            "code": "LOCAL_ONLY",
            "message": "Handy management is only allowed from local loopback"
        });
        return Err((StatusCode::FORBIDDEN, Json(body)).into_response());
    }
    Ok(())
}

/// Helper to map `HandyError` to HTTP Response.
fn handy_error_response(err: HandyError) -> Response {
    let status = match err {
        HandyError::UnsupportedPlatform => StatusCode::BAD_REQUEST,
        HandyError::LocalOnly => StatusCode::FORBIDDEN,
        HandyError::JobAlreadyRunning => StatusCode::CONFLICT,
        HandyError::HostNotAllowed(_) => StatusCode::BAD_REQUEST,
        HandyError::NetworkError(_) => StatusCode::BAD_GATEWAY,
        HandyError::SizeLimitExceeded(_, _) => StatusCode::PAYLOAD_TOO_LARGE,
        HandyError::SignatureInvalid(_) => StatusCode::UNPROCESSABLE_ENTITY,
        HandyError::AuthenticodeInvalid(_) => StatusCode::UNPROCESSABLE_ENTITY,
        HandyError::DowngradeBlocked(_, _) => StatusCode::CONFLICT,
        HandyError::ElevationRequired => StatusCode::FORBIDDEN,
        HandyError::InstallTimeout => StatusCode::GATEWAY_TIMEOUT,
        HandyError::InstallerExitCode(_) => StatusCode::INTERNAL_SERVER_ERROR,
        HandyError::SmokeFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
        HandyError::DiskSpace(_) => StatusCode::INSUFFICIENT_STORAGE,
        HandyError::HandyNoModel => StatusCode::NOT_FOUND,
        HandyError::HandyTranscribeTimeout => StatusCode::GATEWAY_TIMEOUT,
        HandyError::Cancelled => StatusCode::OK,
        HandyError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    let body = serde_json::json!({
        "error": err.code(),
        "code": err.code(),
        "message": err.to_string(),
    });

    (status, Json(body)).into_response()
}

// ── GET /api/voice/handy/status ──────────────────────────────────────

pub async fn handle_handy_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if let Err(e) = require_auth(&state, &headers) {
        return e.into_response();
    }

    let handy_cfg = state.config.read().voice.handy.clone();
    let mut status = detect_handy(&handy_cfg);

    let installer = HandyInstaller::global();
    status.active_job = installer.current_job().await;

    // Attach latest manifest info if network is reachable / cached
    if status.supported {
        if let Ok(manifest) = fetch_manifest(&handy_cfg, false, status.version.as_deref()).await {
            status.latest = Some(manifest);
        }
    }

    (StatusCode::OK, Json(status)).into_response()
}

// ── POST /api/voice/handy/install ─────────────────────────────────────

#[derive(Debug, Deserialize, Default)]
pub struct InstallRequest {
    pub version: Option<String>,
    #[serde(default)]
    pub allow_downgrade: bool,
}

pub async fn handle_handy_install(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<InstallRequest>,
) -> Response {
    if let Err(e) = require_auth(&state, &headers) {
        return e.into_response();
    }
    if let Err(e) = check_local_only(&addr) {
        return e;
    }

    let config = state.config.read().voice.handy.clone();

    let installer = HandyInstaller::global();

    // Spawn listener to forward progress to gateway SSE event_tx
    let mut rx = installer.subscribe();
    let event_tx = state.event_tx.clone();
    tokio::spawn(async move {
        while let Ok(progress) = rx.recv().await {
            let event = serde_json::json!({
                "type": "handy.install.progress",
                "job_id": progress.job_id,
                "kind": progress.kind,
                "stage": progress.stage.to_string(),
                "percent": progress.percent,
                "downloaded_bytes": progress.downloaded_bytes,
                "total_bytes": progress.total_bytes,
                "message": progress.message,
                "error": progress.error,
            });
            let _ = event_tx.send(event);
        }
    });

    match installer
        .spawn_install(config, payload.version, payload.allow_downgrade)
        .await
    {
        Ok(job_id) => (
            StatusCode::ACCEPTED,
            Json(serde_json::json!({ "job_id": job_id })),
        )
            .into_response(),
        Err(err) => handy_error_response(err),
    }
}

// ── POST /api/voice/handy/install/cancel ──────────────────────────────

pub async fn handle_handy_install_cancel(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Response {
    if let Err(e) = require_auth(&state, &headers) {
        return e.into_response();
    }
    if let Err(e) = check_local_only(&addr) {
        return e;
    }

    match HandyInstaller::global().cancel_job().await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "status": "cancelled" }))).into_response(),
        Err(err) => handy_error_response(err),
    }
}

// ── POST /api/voice/handy/launch ──────────────────────────────────────

pub async fn handle_handy_launch(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Response {
    if let Err(e) = require_auth(&state, &headers) {
        return e.into_response();
    }
    if let Err(e) = check_local_only(&addr) {
        return e;
    }

    let config = state.config.read().voice.handy.clone();

    let status = detect_handy(&config);
    let exe_path = match status.exe_path {
        Some(p) => PathBuf::from(p),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "not_installed",
                    "message": "Handy is not installed"
                })),
            )
                .into_response();
        }
    };

    match launch_handy(&exe_path, config.start_hidden) {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "running": true }))).into_response(),
        Err(err) => handy_error_response(err),
    }
}

// ── POST /api/voice/handy/stop ────────────────────────────────────────

pub async fn handle_handy_stop(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Response {
    if let Err(e) = require_auth(&state, &headers) {
        return e.into_response();
    }
    if let Err(e) = check_local_only(&addr) {
        return e;
    }

    match stop_handy().await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "stopped": true }))).into_response(),
        Err(err) => handy_error_response(err),
    }
}

// ── POST /api/voice/handy/toggle ──────────────────────────────────────

pub async fn handle_handy_toggle(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Response {
    if let Err(e) = require_auth(&state, &headers) {
        return e.into_response();
    }
    if let Err(e) = check_local_only(&addr) {
        return e;
    }

    let config = state.config.read().voice.handy.clone();

    let status = detect_handy(&config);
    let exe_path = match status.exe_path {
        Some(p) => PathBuf::from(p),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "not_installed",
                    "message": "Handy is not installed"
                })),
            )
                .into_response();
        }
    };

    match toggle_transcription(&exe_path).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "dispatched": true }))).into_response(),
        Err(err) => handy_error_response(err),
    }
}

// ── POST /api/voice/handy/uninstall ───────────────────────────────────

#[derive(Debug, Deserialize, Default)]
pub struct UninstallRequest {
    #[serde(default)]
    pub delete_app_data: bool,
}

pub async fn handle_handy_uninstall(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<UninstallRequest>,
) -> Response {
    if let Err(e) = require_auth(&state, &headers) {
        return e.into_response();
    }
    if let Err(e) = check_local_only(&addr) {
        return e;
    }

    match HandyInstaller::global().spawn_uninstall(payload.delete_app_data).await {
        Ok(job_id) => (
            StatusCode::ACCEPTED,
            Json(serde_json::json!({ "job_id": job_id })),
        )
            .into_response(),
        Err(err) => handy_error_response(err),
    }
}

// ── GET /api/voice/handy/models ───────────────────────────────────────

pub async fn handle_handy_models(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if let Err(e) = require_auth(&state, &headers) {
        return e.into_response();
    }

    let config = state.config.read().voice.handy.clone();

    let status = detect_handy(&config);
    let exe_path = match status.exe_path {
        Some(p) => PathBuf::from(p),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "not_installed",
                    "message": "Handy is not installed"
                })),
            )
                .into_response();
        }
    };

    match list_models(&exe_path).await {
        Ok(models) => (StatusCode::OK, Json(models)).into_response(),
        Err(err) => handy_error_response(err),
    }
}

// ── POST /api/voice/handy/transcribe ──────────────────────────────────

pub async fn handle_handy_transcribe(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    if let Err(e) = require_auth(&state, &headers) {
        return e.into_response();
    }
    if let Err(e) = check_local_only(&addr) {
        return e;
    }

    let config = state.config.read().voice.handy.clone();

    let status = detect_handy(&config);
    let exe_path = match status.exe_path {
        Some(p) => PathBuf::from(p),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "not_installed",
                    "message": "Handy is not installed"
                })),
            )
                .into_response();
        }
    };

    let home = match directories::UserDirs::new() {
        Some(u) => u.home_dir().to_path_buf(),
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Cannot resolve home dir" })),
            )
                .into_response();
        }
    };

    let tmp_dir = home
        .join(".omnesagent")
        .join("data")
        .join("handy")
        .join("tmp");
    let _ = tokio::fs::create_dir_all(&tmp_dir).await;

    let temp_filename = format!("{}.wav", uuid::Uuid::new_v4());
    let temp_path = tmp_dir.join(&temp_filename);

    let mut model_name: Option<String> = None;
    let mut file_written = false;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "model" {
            if let Ok(text) = field.text().await {
                model_name = Some(text);
            }
        } else if name == "file" || name == "audio" {
            if let Ok(bytes) = field.bytes().await {
                if bytes.len() > HANDY_MAX_TRANSCRIBE_BYTES {
                    return (
                        StatusCode::PAYLOAD_TOO_LARGE,
                        Json(serde_json::json!({ "error": "Audio file exceeds 25MB limit" })),
                    )
                        .into_response();
                }
                if let Err(e) = tokio::fs::write(&temp_path, &bytes).await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": format!("Write error: {e}") })),
                    )
                        .into_response();
                }
                file_written = true;
            }
        }
    }

    if !file_written {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "No audio file uploaded in multipart request" })),
        )
            .into_response();
    }

    let result = transcribe_file(
        &exe_path,
        &temp_path,
        model_name.as_deref(),
        config.transcribe_timeout_secs,
    )
    .await;

    // Always clean up temporary WAV file
    let _ = tokio::fs::remove_file(&temp_path).await;

    match result {
        Ok(text) => (StatusCode::OK, Json(serde_json::json!({ "text": text }))).into_response(),
        Err(err) => handy_error_response(err),
    }
}
