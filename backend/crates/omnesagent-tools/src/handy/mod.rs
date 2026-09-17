//! Handy offline speech-to-text dictation integration.
//!
//! Provides detection, manifest fetching, secure download, silent installation,
//! process management, transcription triggering, and Windows autostart configuration
//! for the third-party Handy application (https://github.com/cjpais/handy).

pub mod autostart;
pub mod detect;
pub mod installer;
pub mod launcher;
pub mod manifest;

use serde::{Deserialize, Serialize};
use std::fmt;

/// Public minisign verification key for the official Handy Tauri updater artifacts.
/// Source: `https://raw.githubusercontent.com/cjpais/Handy/master/src-tauri/tauri.conf.json`
pub const HANDY_MINISIGN_PUBKEY: &str =
    "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEJBQjcyMDk1MjA2NjAxRjkKUldUNUFXWWdsU0MzdXRRZi8zYzhqV2FaNUVDbDd2Rk5VM1IvWWowVXdmRFNKQ1BrMXF5RFFsLy8K";

/// Expected Authenticode subject DN substring for verified Handy binaries.
pub const HANDY_AUTHENTICODE_SUBJECT_SUBSTR: &str = "CN=YNYNG LLC";

/// Overall status of Handy on the current host.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HandyStatus {
    /// Whether host OS is supported (Windows x64).
    pub supported: bool,
    /// Whether Handy is currently detected as installed.
    pub installed: bool,
    /// Detected installed version (e.g. "0.9.6").
    pub version: Option<String>,
    /// Install root directory (e.g. `%LOCALAPPDATA%\Handy`).
    pub install_path: Option<String>,
    /// Full path to Handy executable (`Handy.exe`).
    pub exe_path: Option<String>,
    /// Whether this is a portable installation (`portable` marker file present).
    pub portable: bool,
    /// Whether Handy.exe is currently running.
    pub running: bool,
    /// Custom path configured in `voice.handy.custom_exe_path` if any.
    pub custom_exe_path: Option<String>,
    /// Latest release info from manifest if available.
    pub latest: Option<HandyLatestManifest>,
    /// Active background install/update/uninstall job if running.
    pub active_job: Option<HandyJobStatus>,
    /// Current configuration summary.
    pub config: HandyConfigSummary,
}

/// Summary of latest available release.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HandyLatestManifest {
    pub version: String,
    pub pub_date: Option<String>,
    pub update_available: bool,
    pub url: String,
    pub signature: Option<String>,
    pub size_bytes: Option<u64>,
}

/// Snapshot of active or recently completed install/update/uninstall job.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HandyJobStatus {
    pub job_id: String,
    pub kind: String, // "install" | "update" | "uninstall"
    pub stage: InstallPhase,
    pub percent: u8,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub message: String,
    pub error: Option<String>,
}

/// Phases of the installation state machine.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InstallPhase {
    Idle,
    Download,
    Verify,
    Install,
    Postcheck,
    Done,
    Failed,
}

impl fmt::Display for InstallPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Download => write!(f, "download"),
            Self::Verify => write!(f, "verify"),
            Self::Install => write!(f, "install"),
            Self::Postcheck => write!(f, "postcheck"),
            Self::Done => write!(f, "done"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Configuration summary for status responses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct HandyConfigSummary {
    pub enabled: bool,
    pub install_policy: String,
    pub auto_start_with_os: bool,
}

/// Error codes matching ТЗ_Handy_интеграция.md §12.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandyError {
    UnsupportedPlatform,
    LocalOnly,
    JobAlreadyRunning,
    HostNotAllowed(String),
    NetworkError(String),
    SizeLimitExceeded(u64, u64),
    SignatureInvalid(String),
    AuthenticodeInvalid(String),
    DowngradeBlocked(String, String),
    ElevationRequired,
    InstallTimeout,
    InstallerExitCode(i32),
    SmokeFailed(String),
    DiskSpace(String),
    HandyNoModel,
    HandyTranscribeTimeout,
    Cancelled,
    Other(String),
}

impl HandyError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedPlatform => "UNSUPPORTED_PLATFORM",
            Self::LocalOnly => "LOCAL_ONLY",
            Self::JobAlreadyRunning => "JOB_ALREADY_RUNNING",
            Self::HostNotAllowed(_) => "HOST_NOT_ALLOWED",
            Self::NetworkError(_) => "NETWORK_ERROR",
            Self::SizeLimitExceeded(_, _) => "SIZE_LIMIT_EXCEEDED",
            Self::SignatureInvalid(_) => "SIGNATURE_INVALID",
            Self::AuthenticodeInvalid(_) => "AUTHENTICODE_INVALID",
            Self::DowngradeBlocked(_, _) => "DOWNGRADE_BLOCKED",
            Self::ElevationRequired => "ELEVATION_REQUIRED",
            Self::InstallTimeout => "INSTALL_TIMEOUT",
            Self::InstallerExitCode(_) => "INSTALLER_EXIT_CODE",
            Self::SmokeFailed(_) => "SMOKE_FAILED",
            Self::DiskSpace(_) => "DISK_SPACE",
            Self::HandyNoModel => "HANDY_NO_MODEL",
            Self::HandyTranscribeTimeout => "HANDY_TRANSCRIBE_TIMEOUT",
            Self::Cancelled => "CANCELLED",
            Self::Other(_) => "INTERNAL_ERROR",
        }
    }
}

impl fmt::Display for HandyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => write!(f, "Handy is only supported on Windows x64"),
            Self::LocalOnly => write!(f, "Handy management operations are only allowed for local requests"),
            Self::JobAlreadyRunning => write!(f, "An install or update job is already running"),
            Self::HostNotAllowed(host) => write!(f, "Download host is not allowed: {host}"),
            Self::NetworkError(msg) => write!(f, "Network error: {msg}"),
            Self::SizeLimitExceeded(got, max) => write!(f, "Download size {got} exceeds limit {max}"),
            Self::SignatureInvalid(msg) => write!(f, "Minisign signature check failed: {msg}"),
            Self::AuthenticodeInvalid(msg) => write!(f, "Authenticode certificate check failed: {msg}"),
            Self::DowngradeBlocked(installed, target) => write!(f, "Downgrade from {installed} to {target} is blocked"),
            Self::ElevationRequired => write!(f, "Handy was installed with elevation; please update it manually"),
            Self::InstallTimeout => write!(f, "Installer execution timed out"),
            Self::InstallerExitCode(code) => write!(f, "Installer failed with exit code {code}"),
            Self::SmokeFailed(msg) => write!(f, "Handy post-install smoke test failed: {msg}"),
            Self::DiskSpace(msg) => write!(f, "Insufficient disk space: {msg}"),
            Self::HandyNoModel => write!(f, "No speech-to-text models found in Handy. Please download a model in Handy."),
            Self::HandyTranscribeTimeout => write!(f, "Handy transcription timed out"),
            Self::Cancelled => write!(f, "Job was cancelled by user"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for HandyError {}
