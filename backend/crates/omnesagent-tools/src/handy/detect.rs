//! Detection of installed Handy application, paths, versions, and process status.

use super::{HandyConfigSummary, HandyStatus};
use omnesagent_config::schema::HandyConfig;
use std::path::{Path, PathBuf};

/// Check if current platform is supported (Windows x64).
pub fn is_platform_supported() -> bool {
    cfg!(all(target_os = "windows", target_arch = "x86_64"))
}

/// Information retrieved from Windows Uninstall registry key.
#[derive(Debug, Clone, Default)]
pub struct RegistryHandyInfo {
    pub display_version: Option<String>,
    pub install_location: Option<PathBuf>,
    pub uninstall_string: Option<String>,
}

/// Read Handy info from Windows registry (`HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Handy`).
#[cfg(target_os = "windows")]
pub fn read_registry_info() -> Option<RegistryHandyInfo> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Handy")
        .ok()?;

    let display_version: Option<String> = key.get_value("DisplayVersion").ok();
    let install_location: Option<String> = key.get_value("InstallLocation").ok();
    let uninstall_string: Option<String> = key.get_value("UninstallString").ok();

    Some(RegistryHandyInfo {
        display_version,
        install_location: install_location.map(PathBuf::from),
        uninstall_string,
    })
}

#[cfg(not(target_os = "windows"))]
pub fn read_registry_info() -> Option<RegistryHandyInfo> {
    None
}

/// Get default installation directory for Handy (`%LOCALAPPDATA%\Handy`).
pub fn default_install_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|p| p.join("Handy"))
}

/// Check if `Handy.exe` process is currently running.
pub fn is_handy_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let output = std::process::Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq Handy.exe", "/FO", "CSV", "/NH"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            return text.to_lowercase().contains("handy.exe");
        }
        false
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

/// Check if a given Handy installation has the `portable` marker.
pub fn is_portable_installation(exe_path: &Path) -> bool {
    if let Some(parent) = exe_path.parent() {
        parent.join("portable").exists()
    } else {
        false
    }
}

/// Detect full Handy status on current machine given config.
pub fn detect_handy(config: &HandyConfig) -> HandyStatus {
    let supported = is_platform_supported();
    if !supported {
        return HandyStatus {
            supported: false,
            installed: false,
            version: None,
            install_path: None,
            exe_path: None,
            portable: false,
            running: false,
            custom_exe_path: config.custom_exe_path.clone(),
            latest: None,
            active_job: None,
            config: HandyConfigSummary {
                enabled: config.enabled,
                install_policy: config.install_policy.clone(),
                auto_start_with_os: config.auto_start_with_os,
            },
        };
    }

    let reg_info = read_registry_info();
    let running = is_handy_running();

    // 1. Check custom path if configured
    if let Some(ref custom) = config.custom_exe_path {
        let custom_path = PathBuf::from(custom);
        if custom_path.exists() && custom_path.is_file() {
            let parent = custom_path.parent().map(|p| p.to_string_lossy().to_string());
            let portable = is_portable_installation(&custom_path);
            return HandyStatus {
                supported: true,
                installed: true,
                version: reg_info.as_ref().and_then(|r| r.display_version.clone()),
                install_path: parent,
                exe_path: Some(custom_path.to_string_lossy().to_string()),
                portable,
                running,
                custom_exe_path: Some(custom.clone()),
                latest: None,
                active_job: None,
                config: HandyConfigSummary {
                    enabled: config.enabled,
                    install_policy: config.install_policy.clone(),
                    auto_start_with_os: config.auto_start_with_os,
                },
            };
        }
    }

    // 2. Check registry InstallLocation
    if let Some(ref reg) = reg_info {
        if let Some(ref loc) = reg.install_location {
            let exe = loc.join("Handy.exe");
            if exe.exists() {
                let portable = is_portable_installation(&exe);
                return HandyStatus {
                    supported: true,
                    installed: true,
                    version: reg.display_version.clone(),
                    install_path: Some(loc.to_string_lossy().to_string()),
                    exe_path: Some(exe.to_string_lossy().to_string()),
                    portable,
                    running,
                    custom_exe_path: None,
                    latest: None,
                    active_job: None,
                    config: HandyConfigSummary {
                        enabled: config.enabled,
                        install_policy: config.install_policy.clone(),
                        auto_start_with_os: config.auto_start_with_os,
                    },
                };
            }
        }
    }

    // 3. Check default path `%LOCALAPPDATA%\Handy\Handy.exe`
    if let Some(def_dir) = default_install_dir() {
        let exe = def_dir.join("Handy.exe");
        if exe.exists() {
            let portable = is_portable_installation(&exe);
            return HandyStatus {
                supported: true,
                installed: true,
                version: reg_info.as_ref().and_then(|r| r.display_version.clone()),
                install_path: Some(def_dir.to_string_lossy().to_string()),
                exe_path: Some(exe.to_string_lossy().to_string()),
                portable,
                running,
                custom_exe_path: None,
                latest: None,
                active_job: None,
                config: HandyConfigSummary {
                    enabled: config.enabled,
                    install_policy: config.install_policy.clone(),
                    auto_start_with_os: config.auto_start_with_os,
                },
            };
        }
    }

    // Not installed
    HandyStatus {
        supported: true,
        installed: false,
        version: None,
        install_path: None,
        exe_path: None,
        portable: false,
        running,
        custom_exe_path: config.custom_exe_path.clone(),
        latest: None,
        active_job: None,
        config: HandyConfigSummary {
            enabled: config.enabled,
            install_policy: config.install_policy.clone(),
            auto_start_with_os: config.auto_start_with_os,
        },
    }
}
