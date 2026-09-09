//! Administrator authentication, session management, and password policy.
//!
//! Provides Argon2id password hashing, initial temporary password bootstrap,
//! HttpOnly session cookies, brute-force rate limiting, and password change enforcement.

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::AppState;

/// Session time-to-live: 24 hours.
pub const SESSION_TTL: Duration = Duration::from_secs(86400);

/// Minimum length for administrator passwords.
pub const MIN_PASSWORD_LENGTH: usize = 8;

/// Stored in `admin_auth.json` in the gateway data directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAuthData {
    pub password_hash: String,
    pub must_change_password: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

/// Active web session record.
#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub token: String,
    pub created_at: Instant,
    pub expires_at: Instant,
}

/// Central manager for administrator authentication and active web sessions.
#[derive(Debug)]
pub struct AdminAuthManager {
    auth_file_path: PathBuf,
    credentials_file_path: PathBuf,
    data: RwLock<AdminAuthData>,
    sessions: RwLock<HashMap<String, SessionRecord>>,
}

impl AdminAuthManager {
    /// Initializes or loads administrator authentication configuration from data directory.
    pub fn load_or_init(data_dir: &Path) -> anyhow::Result<(Arc<Self>, Option<String>)> {
        std::fs::create_dir_all(data_dir)?;
        let auth_file = data_dir.join("admin_auth.json");
        let creds_file = data_dir.join(".install-credentials");

        let (data, initial_password) = if auth_file.exists() {
            let content = std::fs::read_to_string(&auth_file)?;
            let auth_data: AdminAuthData = serde_json::from_str(&content)?;
            (auth_data, None)
        } else {
            // Generate a secure random initial password
            let temp_password = format!("omnes-{}", uuid::Uuid::new_v4().simple());
            let hash = hash_password(&temp_password)?;
            let auth_data = AdminAuthData {
                password_hash: hash,
                must_change_password: true,
                created_at: Utc::now(),
                last_login: None,
            };

            let json = serde_json::to_string_pretty(&auth_data)?;
            std::fs::write(&auth_file, json)?;

            // Write temporary credentials file for installation script (chmod 600 if unix)
            let _ = std::fs::write(&creds_file, format!("admin:{temp_password}\n"));
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&creds_file, std::fs::Permissions::from_mode(0o600));
            }

            (auth_data, Some(temp_password))
        };

        let manager = Arc::new(Self {
            auth_file_path: auth_file,
            credentials_file_path: creds_file,
            data: RwLock::new(data),
            sessions: RwLock::new(HashMap::new()),
        });

        Ok((manager, initial_password))
    }

    /// Mock instance for testing without disk persistence or password change requirements.
    pub fn mock() -> Arc<Self> {
        let hash = hash_password("mock-password").unwrap_or_default();
        Arc::new(Self {
            auth_file_path: PathBuf::from("admin_auth.json"),
            credentials_file_path: PathBuf::from(".install-credentials"),
            data: RwLock::new(AdminAuthData {
                password_hash: hash,
                must_change_password: false,
                created_at: Utc::now(),
                last_login: None,
            }),
            sessions: RwLock::new(HashMap::new()),
        })
    }

    /// Verifies supplied password against stored Argon2 hash.
    pub fn verify_password(&self, password: &str) -> bool {
        let hash = self.data.read().password_hash.clone();
        verify_password_hash(password, &hash)
    }

    /// Updates administrator password and resets the `must_change_password` requirement.
    pub fn change_password(&self, old_pw: &str, new_pw: &str) -> Result<(), String> {
        if !self.verify_password(old_pw) {
            return Err("Неверный текущий пароль".to_string());
        }
        if new_pw.len() < MIN_PASSWORD_LENGTH {
            return Err(format!(
                "Новый пароль должен содержать минимум {MIN_PASSWORD_LENGTH} символов"
            ));
        }
        if old_pw == new_pw {
            return Err("Новый пароль не должен совпадать с текущим".to_string());
        }

        let new_hash = hash_password(new_pw).map_err(|e| e.to_string())?;
        {
            let mut guard = self.data.write();
            guard.password_hash = new_hash;
            guard.must_change_password = false;

            if let Ok(json) = serde_json::to_string_pretty(&*guard) {
                let _ = std::fs::write(&self.auth_file_path, json);
            }
        }

        // Clean up temporary credentials file once password is changed
        let _ = std::fs::remove_file(&self.credentials_file_path);

        Ok(())
    }

    /// Creates a new active web session and returns the session token.
    pub fn create_session(&self) -> String {
        let token = uuid::Uuid::new_v4().to_string();
        let now = Instant::now();
        let record = SessionRecord {
            token: token.clone(),
            created_at: now,
            expires_at: now + SESSION_TTL,
        };

        let mut sessions = self.sessions.write();
        self.sweep_expired_sessions(&mut sessions, now);
        sessions.insert(token.clone(), record);

        // Update last login
        {
            let mut guard = self.data.write();
            guard.last_login = Some(Utc::now());
            if let Ok(json) = serde_json::to_string_pretty(&*guard) {
                let _ = std::fs::write(&self.auth_file_path, json);
            }
        }

        token
    }

    /// Validates an active session token.
    /// Returns `Some(must_change_password)` if session is valid, or `None` if invalid/expired.
    pub fn validate_session(&self, token: &str) -> Option<bool> {
        let now = Instant::now();
        let mut sessions = self.sessions.write();
        if let Some(record) = sessions.get(token) {
            if record.expires_at > now {
                let must_change = self.data.read().must_change_password;
                return Some(must_change);
            } else {
                sessions.remove(token);
            }
        }
        None
    }

    /// Destroys an active session (logout).
    pub fn destroy_session(&self, token: &str) {
        self.sessions.write().remove(token);
    }

    /// Current administrator authentication status.
    pub fn get_status(&self) -> (bool, Option<DateTime<Utc>>) {
        let guard = self.data.read();
        (guard.must_change_password, guard.last_login)
    }

    fn sweep_expired_sessions(&self, sessions: &mut HashMap<String, SessionRecord>, now: Instant) {
        sessions.retain(|_, s| s.expires_at > now);
    }
}

// ── Argon2 Password Hashing Utilities ─────────────────────────────────────

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    use argon2::password_hash::SaltString;
    use argon2::password_hash::rand_core::OsRng;
    use argon2::{Argon2, PasswordHasher};

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Argon2 hash error: {e}"))?
        .to_string();
    Ok(hash)
}

pub fn verify_password_hash(password: &str, hash: &str) -> bool {
    use argon2::password_hash::{PasswordHash, PasswordVerifier};
    use argon2::Argon2;

    let Ok(parsed_hash) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

// ── HTTP API Handlers ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub status: &'static str,
    pub must_change_password: bool,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthStatusResponse {
    pub authenticated: bool,
    pub must_change_password: bool,
    pub last_login: Option<DateTime<Utc>>,
}

/// POST /api/auth/login
pub async fn handle_auth_login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<LoginRequest>,
) -> Response {
    let client_ip = addr.ip().to_string();

    // Rate-limiting check
    if let Err(rate_err) = state.auth_limiter.check_rate_limit(&client_ip) {
        let retry_after = rate_err.retry_after_secs;
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            Json(serde_json::json!({
                "error": "too_many_requests",
                "message": format!("Превышен лимит попыток входа. Повторите через {retry_after} сек."),
                "retry_after_secs": retry_after
            })),
        )
            .into_response();
    }

    state.auth_limiter.record_attempt(&client_ip);

    if !state.admin_auth.verify_password(&payload.password) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "unauthorized",
                "message": "Неверный пароль администратора"
            })),
        )
            .into_response();
    }

    let token = state.admin_auth.create_session();
    let (must_change, _) = state.admin_auth.get_status();

    let cookie_val = format!(
        "omnes_session={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        SESSION_TTL.as_secs()
    );

    (
        StatusCode::OK,
        [(header::SET_COOKIE, cookie_val)],
        Json(LoginResponse {
            status: "ok",
            must_change_password: must_change,
            token,
        }),
    )
        .into_response()
}

/// POST /api/auth/logout
pub async fn handle_auth_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if let Some(token) = extract_session_token(&headers) {
        state.admin_auth.destroy_session(&token);
    }

    let clear_cookie = "omnes_session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";
    (
        StatusCode::OK,
        [(header::SET_COOKIE, clear_cookie.to_string())],
        Json(serde_json::json!({ "status": "ok" })),
    )
        .into_response()
}

/// POST /api/auth/change-password
pub async fn handle_auth_change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> Response {
    let Some(token) = extract_session_token(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "unauthorized",
                "message": "Требуется авторизация"
            })),
        )
            .into_response();
    };

    if state.admin_auth.validate_session(&token).is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "session_expired",
                "message": "Сессия истекла"
            })),
        )
            .into_response();
    }

    match state.admin_auth.change_password(&payload.old_password, &payload.new_password) {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "ok",
                "message": "Пароль успешно изменён"
            })),
        )
            .into_response(),
        Err(err_msg) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "bad_request",
                "message": err_msg
            })),
        )
            .into_response(),
    }
}

/// GET /api/auth/me
pub async fn handle_auth_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    let token = extract_session_token(&headers);
    let session_status = token.as_deref().and_then(|t| state.admin_auth.validate_session(t));

    match session_status {
        Some(must_change) => {
            let (_, last_login) = state.admin_auth.get_status();
            (
                StatusCode::OK,
                Json(AuthStatusResponse {
                    authenticated: true,
                    must_change_password: must_change,
                    last_login,
                }),
            )
                .into_response()
        }
        None => (
            StatusCode::UNAUTHORIZED,
            Json(AuthStatusResponse {
                authenticated: false,
                must_change_password: false,
                last_login: None,
            }),
        )
            .into_response(),
    }
}

// ── Session Middleware ─────────────────────────────────────────────────────

/// Axum middleware protecting `/api/*` and `/ws/*` endpoints with session authorization
/// and password change policy enforcement.
pub async fn require_web_auth(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();

    // 1. Always permit public endpoints
    if is_public_path(path) {
        return next.run(request).await;
    }

    // 2. Allow requests with valid desktop pairing token
    if let Some(auth_hdr) = headers.get(header::AUTHORIZATION).and_then(|h| h.to_str().ok()) {
        if let Some(token) = auth_hdr.strip_prefix("Bearer ").map(str::trim) {
            if state.pairing.is_authenticated(token) {
                return next.run(request).await;
            }
        }
    }

    // 3. Extract and validate web session
    let token = extract_session_token(&headers);
    let validation = token.as_deref().and_then(|t| state.admin_auth.validate_session(t));

    match validation {
        Some(must_change_password) => {
            // If password change is required, block all API calls except change-password, me, and logout
            if must_change_password && !is_auth_mgmt_path(path) {
                return (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({
                        "error": "password_change_required",
                        "message": "Перед продолжением работы необходимо сменить первичный пароль администратора."
                    })),
                )
                    .into_response();
            }
            next.run(request).await
        }
        None => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "unauthorized",
                "message": "Требуется авторизация."
            })),
        )
            .into_response(),
    }
}

fn is_public_path(path: &str) -> bool {
    matches!(
        path,
        "/health"
            | "/api/health"
            | "/api/auth/login"
            | "/api/auth/me"
            | "/favicon.png"
            | "/manifest.json"
    ) || path.starts_with("/_app/")
        || path == "/"
        || path == "/index.html"
}

fn is_auth_mgmt_path(path: &str) -> bool {
    matches!(
        path,
        "/api/auth/change-password" | "/api/auth/logout" | "/api/auth/me"
    )
}

fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    // 1. From Cookie: omnes_session=<token>
    if let Some(cookie_hdr) = headers.get(header::COOKIE).and_then(|h| h.to_str().ok()) {
        for cookie in cookie_hdr.split(';') {
            let cookie = cookie.trim();
            if let Some(val) = cookie.strip_prefix("omnes_session=") {
                let token = val.trim();
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
    }

    // 2. From X-Session-Token header
    if let Some(val) = headers.get("x-session-token").and_then(|h| h.to_str().ok()) {
        let token = val.trim();
        if !token.is_empty() {
            return Some(token.to_string());
        }
    }

    // 3. From Authorization: Bearer <session_id>
    if let Some(auth_hdr) = headers.get(header::AUTHORIZATION).and_then(|h| h.to_str().ok()) {
        if let Some(token) = auth_hdr.strip_prefix("Bearer ").map(str::trim) {
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "SuperSecretPassword123!";
        let hash = hash_password(password).expect("hash failed");
        assert!(verify_password_hash(password, &hash));
        assert!(!verify_password_hash("WrongPassword", &hash));
    }

    #[test]
    fn test_admin_auth_lifecycle() {
        let temp_dir = tempfile::tempdir().unwrap();
        let (manager, init_pw) = AdminAuthManager::load_or_init(temp_dir.path()).unwrap();
        let temp_pw = init_pw.expect("initial password should be generated");

        assert!(manager.verify_password(&temp_pw));
        let (must_change, _) = manager.get_status();
        assert!(must_change);

        let token = manager.create_session();
        assert_eq!(manager.validate_session(&token), Some(true));

        // Change password validation
        assert!(manager.change_password("wrong", "NewPassword123!").is_err());
        assert!(manager.change_password(&temp_pw, "short").is_err());
        assert!(manager.change_password(&temp_pw, &temp_pw).is_err());

        // Successful password change
        assert!(manager.change_password(&temp_pw, "NewPassword123!").is_ok());
        let (must_change_after, _) = manager.get_status();
        assert!(!must_change_after);
        assert!(manager.verify_password("NewPassword123!"));
        assert_eq!(manager.validate_session(&token), Some(false));

        // Destroy session
        manager.destroy_session(&token);
        assert_eq!(manager.validate_session(&token), None);
    }
}
