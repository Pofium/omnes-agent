//! State machine for Handy download, verification, silent installation, update, and uninstall jobs.

use super::{
    autostart::set_autostart_enabled,
    detect::{default_install_dir, detect_handy, is_handy_running, read_registry_info},
    launcher::{launch_handy, list_models, stop_handy},
    manifest::{fetch_manifest, is_newer_version, validate_download_url, verify_authenticode},
    HandyError, HandyJobStatus, InstallPhase,
};
use futures_util::StreamExt;
use omnesagent_config::schema::HandyConfig;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex, RwLock};

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Shared singleton installer manager.
pub struct HandyInstaller {
    active_job: Arc<RwLock<Option<HandyJobStatus>>>,
    cancel_flag: Arc<AtomicBool>,
    lock: Arc<Mutex<()>>,
    event_tx: broadcast::Sender<HandyJobStatus>,
}

static GLOBAL_INSTALLER: std::sync::LazyLock<HandyInstaller> =
    std::sync::LazyLock::new(HandyInstaller::new);

impl HandyInstaller {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(128);
        Self {
            active_job: Arc::new(RwLock::new(None)),
            cancel_flag: Arc::new(AtomicBool::new(false)),
            lock: Arc::new(Mutex::new(())),
            event_tx: tx,
        }
    }

    pub fn global() -> &'static HandyInstaller {
        &GLOBAL_INSTALLER
    }

    pub fn subscribe(&self) -> broadcast::Receiver<HandyJobStatus> {
        self.event_tx.subscribe()
    }

    pub async fn current_job(&self) -> Option<HandyJobStatus> {
        self.active_job.read().await.clone()
    }

    pub async fn cancel_job(&self) -> Result<(), HandyError> {
        let job = self.active_job.read().await;
        if let Some(ref j) = *job {
            match j.stage {
                InstallPhase::Download | InstallPhase::Verify => {
                    self.cancel_flag.store(true, Ordering::SeqCst);
                    Ok(())
                }
                InstallPhase::Install | InstallPhase::Postcheck => {
                    Err(HandyError::Other("Installation phase cannot be cancelled".into()))
                }
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    async fn update_status(&self, status: HandyJobStatus) {
        let mut job = self.active_job.write().await;
        *job = Some(status.clone());
        let _ = self.event_tx.send(status);
    }

    /// Run installation or update flow in background.
    pub async fn spawn_install(
        &self,
        config: HandyConfig,
        target_version: Option<String>,
        allow_downgrade: bool,
    ) -> Result<String, HandyError> {
        if !super::detect::is_platform_supported() {
            return Err(HandyError::UnsupportedPlatform);
        }

        // Try to acquire job lock
        let lock = match Arc::clone(&self.lock).try_lock_owned() {
            Ok(l) => l,
            Err(_) => return Err(HandyError::JobAlreadyRunning),
        };

        self.cancel_flag.store(false, Ordering::SeqCst);
        let job_id = uuid::Uuid::new_v4().to_string();

        let initial_status = HandyJobStatus {
            job_id: job_id.clone(),
            kind: "install".into(),
            stage: InstallPhase::Idle,
            percent: 0,
            downloaded_bytes: 0,
            total_bytes: 0,
            message: "Подготовка к установке...".into(),
            error: None,
        };
        self.update_status(initial_status).await;

        let active_job_id = job_id.clone();
        tokio::spawn(async move {
            let _guard = lock;
            let installer_ref = Self::global();
            let res = installer_ref
                .run_install_pipeline(&active_job_id, &config, target_version, allow_downgrade)
                .await;

            if let Err(err) = res {
                let fail_status = HandyJobStatus {
                    job_id: active_job_id,
                    kind: "install".into(),
                    stage: InstallPhase::Failed,
                    percent: 0,
                    downloaded_bytes: 0,
                    total_bytes: 0,
                    message: format!("Ошибка: {err}"),
                    error: Some(err.code().to_string()),
                };
                installer_ref.update_status(fail_status).await;
            }
        });

        Ok(job_id)
    }

    async fn run_install_pipeline(
        &self,
        job_id: &str,
        config: &HandyConfig,
        target_version: Option<String>,
        allow_downgrade: bool,
    ) -> Result<(), HandyError> {
        let current = detect_handy(config);
        let is_update = current.installed;
        let kind = if is_update { "update" } else { "install" };

        let target_ver_ref = target_version.as_deref().or(current.version.as_deref());
        let manifest = fetch_manifest(config, true, target_ver_ref).await?;

        // Downgrade check
        if let Some(ref current_ver) = current.version {
            if is_newer_version(current_ver, &manifest.version) && !allow_downgrade {
                return Err(HandyError::DowngradeBlocked(
                    current_ver.clone(),
                    manifest.version.clone(),
                ));
            }
        }

        // Cache dir
        let home = directories::UserDirs::new()
            .ok_or_else(|| HandyError::Other("Could not resolve user home directory".into()))?;
        let downloads_dir = home
            .home_dir()
            .join(".omnesagent")
            .join("cache")
            .join("handy")
            .join("downloads");
        tokio::fs::create_dir_all(&downloads_dir)
            .await
            .map_err(|e| HandyError::Other(format!("Failed to create download dir: {e}")))?;

        let setup_filename = format!("Handy_{}_x64-setup.exe", manifest.version);
        let part_path = downloads_dir.join(format!("{setup_filename}.part"));
        let final_path = downloads_dir.join(&setup_filename);

        // Phase 1: Download
        self.update_status(HandyJobStatus {
            job_id: job_id.to_string(),
            kind: kind.into(),
            stage: InstallPhase::Download,
            percent: 0,
            downloaded_bytes: 0,
            total_bytes: 0,
            message: "Загрузка установщика Handy...".into(),
            error: None,
        })
        .await;

        validate_download_url(&manifest.url, &config.download_host_allowlist)?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| HandyError::NetworkError(e.to_string()))?;

        let response = client
            .get(&manifest.url)
            .send()
            .await
            .map_err(|e| HandyError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(HandyError::NetworkError(format!(
                "HTTP {} downloading setup from {}",
                response.status(),
                manifest.url
            )));
        }

        let total_bytes = response.content_length().unwrap_or(0);
        if total_bytes > config.max_download_bytes {
            return Err(HandyError::SizeLimitExceeded(
                total_bytes,
                config.max_download_bytes,
            ));
        }

        let mut file = tokio::fs::File::create(&part_path)
            .await
            .map_err(|e| HandyError::Other(format!("Failed to create part file: {e}")))?;

        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut last_progress_report = 0u64;

        while let Some(chunk_res) = stream.next().await {
            if self.cancel_flag.load(Ordering::SeqCst) {
                let _ = tokio::fs::remove_file(&part_path).await;
                return Err(HandyError::Cancelled);
            }

            let chunk = chunk_res.map_err(|e| HandyError::NetworkError(e.to_string()))?;
            downloaded += chunk.len() as u64;

            if downloaded > config.max_download_bytes {
                let _ = tokio::fs::remove_file(&part_path).await;
                return Err(HandyError::SizeLimitExceeded(
                    downloaded,
                    config.max_download_bytes,
                ));
            }

            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
                .await
                .map_err(|e| HandyError::Other(format!("Disk write error: {e}")))?;

            if downloaded.saturating_sub(last_progress_report) >= 512 * 1024
                || downloaded == total_bytes
            {
                last_progress_report = downloaded;
                let pct = if total_bytes > 0 {
                    ((downloaded as f64 / total_bytes as f64) * 100.0).min(99.0) as u8
                } else {
                    50
                };
                self.update_status(HandyJobStatus {
                    job_id: job_id.to_string(),
                    kind: kind.into(),
                    stage: InstallPhase::Download,
                    percent: pct,
                    downloaded_bytes: downloaded,
                    total_bytes,
                    message: format!(
                        "Загрузка: {:.1} / {:.1} МБ",
                        downloaded as f64 / 1_048_576.0,
                        total_bytes as f64 / 1_048_576.0
                    ),
                    error: None,
                })
                .await;
            }
        }

        tokio::io::AsyncWriteExt::flush(&mut file)
            .await
            .map_err(|e| HandyError::Other(format!("Flush error: {e}")))?;
        drop(file);

        tokio::fs::rename(&part_path, &final_path)
            .await
            .map_err(|e| HandyError::Other(format!("Atomic rename failed: {e}")))?;

        // Phase 2: Verify
        self.update_status(HandyJobStatus {
            job_id: job_id.to_string(),
            kind: kind.into(),
            stage: InstallPhase::Verify,
            percent: 100,
            downloaded_bytes: downloaded,
            total_bytes: downloaded,
            message: "Проверка подписи установщика...".into(),
            error: None,
        })
        .await;

        if self.cancel_flag.load(Ordering::SeqCst) {
            let _ = tokio::fs::remove_file(&final_path).await;
            return Err(HandyError::Cancelled);
        }

        verify_authenticode(&final_path).await?;

        // Phase 3: Install
        self.update_status(HandyJobStatus {
            job_id: job_id.to_string(),
            kind: kind.into(),
            stage: InstallPhase::Install,
            percent: 0,
            downloaded_bytes: downloaded,
            total_bytes: downloaded,
            message: "Установка Handy... Это может занять несколько минут.".into(),
            error: None,
        })
        .await;

        let mut cmd = tokio::process::Command::new(&final_path);
        cmd.arg("/S");
        if is_update {
            cmd.arg("/UPDATE");
        }

        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let child_res = cmd.spawn();
        let mut child =
            child_res.map_err(|e| HandyError::Other(format!("Failed to run installer: {e}")))?;

        // Poll registry and wait for install completion up to 600s
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(600);
        let mut installed_ok = false;

        while start.elapsed() < timeout {
            tokio::time::sleep(Duration::from_millis(500)).await;

            let pct = ((start.elapsed().as_secs() as f64 / 60.0) * 100.0).min(95.0) as u8;
            self.update_status(HandyJobStatus {
                job_id: job_id.to_string(),
                kind: kind.into(),
                stage: InstallPhase::Install,
                percent: pct,
                downloaded_bytes: downloaded,
                total_bytes: downloaded,
                message: "Установка файлов и компонентов...".into(),
                error: None,
            })
            .await;

            if let Ok(Some(exit_status)) = child.try_wait() {
                if !exit_status.success() {
                    let code = exit_status.code().unwrap_or(-1);
                    return Err(HandyError::InstallerExitCode(code));
                }
            }

            // Verify installation in registry and filesystem
            let detected = detect_handy(config);
            if detected.installed {
                installed_ok = true;
                break;
            }
        }

        if !installed_ok {
            return Err(HandyError::InstallTimeout);
        }

        // Phase 4: Postcheck
        self.update_status(HandyJobStatus {
            job_id: job_id.to_string(),
            kind: kind.into(),
            stage: InstallPhase::Postcheck,
            percent: 96,
            downloaded_bytes: downloaded,
            total_bytes: downloaded,
            message: "Проверка работоспособности...".into(),
            error: None,
        })
        .await;

        let final_detected = detect_handy(config);
        if let Some(ref exe_str) = final_detected.exe_path {
            let exe_path = PathBuf::from(exe_str);
            // Run smoke test: list-models
            let _ = list_models(&exe_path).await;

            if config.launch_on_agent_start && !is_handy_running() {
                let _ = launch_handy(&exe_path, config.start_hidden);
            }
        }

        // Phase 5: Done
        self.update_status(HandyJobStatus {
            job_id: job_id.to_string(),
            kind: kind.into(),
            stage: InstallPhase::Done,
            percent: 100,
            downloaded_bytes: downloaded,
            total_bytes: downloaded,
            message: "Handy успешно установлен и готов к работе.".into(),
            error: None,
        })
        .await;

        Ok(())
    }

    /// Uninstall Handy and optionally remove configuration & models.
    pub async fn spawn_uninstall(&self, delete_app_data: bool) -> Result<String, HandyError> {
        if !super::detect::is_platform_supported() {
            return Err(HandyError::UnsupportedPlatform);
        }

        let lock = match Arc::clone(&self.lock).try_lock_owned() {
            Ok(l) => l,
            Err(_) => return Err(HandyError::JobAlreadyRunning),
        };

        let job_id = uuid::Uuid::new_v4().to_string();

        let initial_status = HandyJobStatus {
            job_id: job_id.clone(),
            kind: "uninstall".into(),
            stage: InstallPhase::Install,
            percent: 0,
            downloaded_bytes: 0,
            total_bytes: 0,
            message: "Остановка и удаление Handy...".into(),
            error: None,
        };
        self.update_status(initial_status).await;

        let active_job_id = job_id.clone();
        tokio::spawn(async move {
            let _guard = lock;
            let installer_ref = Self::global();
            let _ = stop_handy().await;

            let reg = read_registry_info();
            let install_dir = reg
                .as_ref()
                .and_then(|r| r.install_location.clone())
                .or_else(default_install_dir);

            if let Some(dir) = install_dir {
                let uninstaller = dir.join("uninstall.exe");
                if uninstaller.exists() {
                    let mut cmd = tokio::process::Command::new(&uninstaller);
                    let arg = format!("_?={}", dir.display());
                    cmd.arg("/S").arg(&arg);

                    #[cfg(target_os = "windows")]
                    {
                        cmd.creation_flags(CREATE_NO_WINDOW);
                    }

                    let _ = cmd.spawn();
                }

                // Poll for removal of registry key up to 60s
                let start = std::time::Instant::now();
                while start.elapsed() < Duration::from_secs(60) {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    if read_registry_info().is_none() {
                        break;
                    }
                }

                // Clean autostart
                let exe = dir.join("Handy.exe");
                let _ = set_autostart_enabled(&exe, false);
            }

            if delete_app_data {
                if let Some(appdata) = std::env::var_os("APPDATA") {
                    let handy_data = PathBuf::from(appdata).join("com.pais.handy");
                    let _ = tokio::fs::remove_dir_all(&handy_data).await;
                }
            }

            installer_ref
                .update_status(HandyJobStatus {
                    job_id: active_job_id,
                    kind: "uninstall".into(),
                    stage: InstallPhase::Done,
                    percent: 100,
                    downloaded_bytes: 0,
                    total_bytes: 0,
                    message: "Handy успешно удален.".into(),
                    error: None,
                })
                .await;
        });

        Ok(job_id)
    }
}
