//! Handy process lifecycle and remote CLI invocation.

use super::{detect::is_handy_running, HandyError};
use std::path::Path;
use std::time::Duration;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Launch the Handy desktop application detached.
pub fn launch_handy(exe_path: &Path, start_hidden: bool) -> Result<(), HandyError> {
    if !exe_path.exists() {
        return Err(HandyError::Other(format!(
            "Handy executable not found at {}",
            exe_path.display()
        )));
    }

    let mut cmd = std::process::Command::new(exe_path);
    if start_hidden {
        cmd.arg("--start-hidden");
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    cmd.spawn()
        .map_err(|e| HandyError::Other(format!("Failed to spawn Handy: {e}")))?;

    Ok(())
}

/// Stop running Handy process via taskkill.
pub async fn stop_handy() -> Result<(), HandyError> {
    if !is_handy_running() {
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {

        // 1. Try graceful WM_CLOSE
        let _ = tokio::process::Command::new("taskkill")
            .args(["/IM", "Handy.exe"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .await;

        tokio::time::sleep(Duration::from_millis(1500)).await;

        // 2. If still running, force termination
        if is_handy_running() {
            let _ = tokio::process::Command::new("taskkill")
                .args(["/F", "/IM", "Handy.exe"])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .await;
        }
    }

    Ok(())
}

/// Trigger dictation via Handy's single-instance CLI interface:
/// `Handy.exe --toggle-transcription`.
pub async fn toggle_transcription(exe_path: &Path) -> Result<(), HandyError> {
    if !is_handy_running() {
        launch_handy(exe_path, true)?;
        // Wait briefly for single-instance listener to bind
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }

    let mut cmd = tokio::process::Command::new(exe_path);
    cmd.arg("--toggle-transcription");

    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| HandyError::Other(format!("Failed to send toggle-transcription: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HandyError::Other(format!(
            "toggle-transcription failed: {stderr}"
        )));
    }

    Ok(())
}

/// Cancel ongoing transcription: `Handy.exe --cancel`.
pub async fn cancel_transcription(exe_path: &Path) -> Result<(), HandyError> {
    if !is_handy_running() {
        return Ok(());
    }

    let mut cmd = tokio::process::Command::new(exe_path);
    cmd.arg("--cancel");

    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let _ = cmd.output().await;
    Ok(())
}

/// Query installed speech models via `Handy.exe --list-models --json`.
pub async fn list_models(exe_path: &Path) -> Result<Vec<serde_json::Value>, HandyError> {
    let mut cmd = tokio::process::Command::new(exe_path);
    cmd.args(["--list-models", "--json"]);

    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = tokio::time::timeout(Duration::from_secs(15), cmd.output())
        .await
        .map_err(|_| HandyError::Other("list-models timed out".into()))?
        .map_err(|e| HandyError::Other(format!("Failed to run list-models: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HandyError::Other(format!("list-models failed: {stderr}")));
    }

    let json_text = String::from_utf8_lossy(&output.stdout);
    let trimmed = json_text.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str(trimmed)
        .map_err(|e| HandyError::Other(format!("Invalid list-models JSON: {e}")))
}

/// Transcribe an audio file offline using `Handy.exe --transcribe-file <path> --json`.
pub async fn transcribe_file(
    exe_path: &Path,
    wav_path: &Path,
    model: Option<&str>,
    timeout_secs: u64,
) -> Result<String, HandyError> {
    let mut cmd = tokio::process::Command::new(exe_path);
    cmd.arg("--transcribe-file").arg(wav_path).arg("--json");

    if let Some(m) = model {
        cmd.arg("--model").arg(m);
    }

    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let timeout_duration = Duration::from_secs(timeout_secs.max(10));
    let output = tokio::time::timeout(timeout_duration, cmd.output())
        .await
        .map_err(|_| HandyError::HandyTranscribeTimeout)?
        .map_err(|e| HandyError::Other(format!("Failed to execute transcribe-file: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("No model found") || stderr.contains("no models") {
            return Err(HandyError::HandyNoModel);
        }
        return Err(HandyError::Other(format!("Transcription failed: {stderr}")));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout_str.trim();

    // Parse JSON result from Handy
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Some(text) = val.get("text").and_then(|t| t.as_str()) {
            return Ok(text.to_string());
        }
        if let Some(transcription) = val.get("transcription").and_then(|t| t.as_str()) {
            return Ok(transcription.to_string());
        }
    }

    Ok(trimmed.to_string())
}
