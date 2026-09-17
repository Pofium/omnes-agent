//! Release manifest fetching, caching, SSRF validation, and signature verification.

use super::{HandyError, HandyLatestManifest, HANDY_AUTHENTICODE_SUBJECT_SUBSTR};
use omnesagent_config::schema::HandyConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Tauri updater manifest structure for `latest.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriManifest {
    pub version: String,
    pub notes: Option<String>,
    pub pub_date: Option<String>,
    pub platforms: HashMap<String, PlatformArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformArtifact {
    pub signature: Option<String>,
    pub url: String,
}

/// Cached manifest file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedManifest {
    fetched_at: u64,
    manifest: TauriManifest,
}

/// Validate that a URL is safe against SSRF attacks according to project rules:
/// - Must use https
/// - Hostname must be in the allowlist
/// - Hostname must not resolve to private, loopback, or reserved IP addresses
pub fn validate_download_url(url_str: &str, allowlist: &[String]) -> Result<reqwest::Url, HandyError> {
    let parsed = reqwest::Url::parse(url_str)
        .map_err(|e| HandyError::NetworkError(format!("Invalid URL: {e}")))?;

    if parsed.scheme() != "https" {
        return Err(HandyError::HostNotAllowed(format!(
            "Only HTTPS URLs are permitted, got {}",
            parsed.scheme()
        )));
    }

    let host_str = parsed
        .host_str()
        .ok_or_else(|| HandyError::HostNotAllowed("Missing host in URL".into()))?;

    let host_lower = host_str.to_lowercase();
    let is_allowed = allowlist.iter().any(|allowed| {
        let allowed_lower = allowed.to_lowercase();
        host_lower == allowed_lower || host_lower.ends_with(&format!(".{allowed_lower}"))
    });

    if !is_allowed {
        return Err(HandyError::HostNotAllowed(host_str.to_string()));
    }

    // Resolve hostname to IP and check for private / loopback addresses
    let socket_str = format!("{host_str}:443");
    if let Ok(addrs) = socket_str.to_socket_addrs() {
        for addr in addrs {
            let ip = addr.ip();
            if is_forbidden_ip(ip) {
                return Err(HandyError::HostNotAllowed(format!(
                    "Host {host_str} resolved to forbidden IP {ip}"
                )));
            }
        }
    }

    Ok(parsed)
}

/// Check if an IP address is loopback, private, link-local, or multicast.
fn is_forbidden_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_documentation()
                || v4.is_multicast()
                || v4.octets()[0] == 0 // 0.0.0.0/8
                || (v4.octets()[0] == 100 && (v4.octets()[1] & 0b1100_0000 == 0b0100_0000)) // 100.64.0.0/10 CGNAT
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_multicast()
                || (v6.segments()[0] & 0xfe00) == 0xfc00 // Unique local (fc00::/7)
                || (v6.segments()[0] & 0xffc0) == 0xfe80 // Link local (fe80::/10)
        }
    }
}

/// Get path to manifest cache file in `~/.omnesagent/cache/handy/manifest.json`.
pub fn get_manifest_cache_path() -> Option<PathBuf> {
    directories::UserDirs::new().map(|dirs| {
        dirs.home_dir()
            .join(".omnesagent")
            .join("cache")
            .join("handy")
            .join("manifest.json")
    })
}

/// Fetch the release manifest with a 10-minute local cache.
pub async fn fetch_manifest(
    config: &HandyConfig,
    force_refresh: bool,
    current_version: Option<&str>,
) -> Result<HandyLatestManifest, HandyError> {
    let cache_path = get_manifest_cache_path();

    if !force_refresh {
        if let Some(ref path) = cache_path {
            if path.exists() {
                if let Ok(data) = tokio::fs::read_to_string(path).await {
                    if let Ok(cached) = serde_json::from_str::<CachedManifest>(&data) {
                        let now = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        if now.saturating_sub(cached.fetched_at) < 600 {
                            return parse_platform_artifact(&cached.manifest, current_version);
                        }
                    }
                }
            }
        }
    }

    // Determine URL
    let manifest_url = if config.pin_version.is_empty() {
        "https://github.com/cjpais/Handy/releases/latest/download/latest.json".to_string()
    } else {
        format!(
            "https://github.com/cjpais/Handy/releases/download/v{}/latest.json",
            config.pin_version
        )
    };

    validate_download_url(&manifest_url, &config.download_host_allowlist)?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::custom({
            let allowlist = config.download_host_allowlist.clone();
            move |attempt| {
                if attempt.previous().len() >= 5 {
                    attempt.error("Too many redirects")
                } else if let Err(e) = validate_download_url(attempt.url().as_str(), &allowlist) {
                    attempt.error(format!("{e}"))
                } else {
                    attempt.follow()
                }
            }
        }))
        .build()
        .map_err(|e| HandyError::NetworkError(e.to_string()))?;

    let response = client
        .get(&manifest_url)
        .send()
        .await
        .map_err(|e| HandyError::NetworkError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(HandyError::NetworkError(format!(
            "HTTP {} fetching manifest from {}",
            response.status(),
            manifest_url
        )));
    }

    let manifest = response
        .json::<TauriManifest>()
        .await
        .map_err(|e| HandyError::NetworkError(format!("Invalid manifest JSON: {e}")))?;

    // Cache manifest
    if let Some(ref path) = cache_path {
        if let Some(parent) = path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let cached = CachedManifest {
            fetched_at: now,
            manifest: manifest.clone(),
        };
        if let Ok(json_str) = serde_json::to_string_pretty(&cached) {
            let _ = tokio::fs::write(path, json_str).await;
        }
    }

    parse_platform_artifact(&manifest, current_version)
}

/// Extract target Windows artifact from Tauri manifest.
fn parse_platform_artifact(
    manifest: &TauriManifest,
    current_version: Option<&str>,
) -> Result<HandyLatestManifest, HandyError> {
    let platform = manifest
        .platforms
        .get("windows-x86_64-nsis")
        .or_else(|| manifest.platforms.get("windows-x86_64"))
        .ok_or_else(|| {
            HandyError::Other("Manifest does not contain windows-x86_64 artifact".into())
        })?;

    let update_available = match current_version {
        Some(curr) => is_newer_version(&manifest.version, curr),
        None => true,
    };

    Ok(HandyLatestManifest {
        version: manifest.version.clone(),
        pub_date: manifest.pub_date.clone(),
        update_available,
        url: platform.url.clone(),
        signature: platform.signature.clone(),
        size_bytes: None,
    })
}

/// Check if `remote_ver` is newer than `local_ver`.
pub fn is_newer_version(remote_ver: &str, local_ver: &str) -> bool {
    fn parse_parts(v: &str) -> Vec<u32> {
        v.trim_start_matches('v')
            .split(['.', '-'])
            .filter_map(|p| p.parse::<u32>().ok())
            .collect()
    }

    let r_parts = parse_parts(remote_ver);
    let l_parts = parse_parts(local_ver);

    r_parts > l_parts
}

/// Verify Windows Authenticode signature on the downloaded executable.
pub async fn verify_authenticode(file_path: &Path) -> Result<(), HandyError> {
    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let script = format!(
            "$sig = Get-AuthenticodeSignature -LiteralPath '{}'; \
             $subj = if ($sig.SignerCertificate) {{ $sig.SignerCertificate.Subject }} else {{ '' }}; \
             $sig.Status.ToString() + '|' + $subj",
            file_path.display()
        );

        let output = tokio::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .await
            .map_err(|e| HandyError::AuthenticodeInvalid(format!("Failed to run PowerShell: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HandyError::AuthenticodeInvalid(format!(
                "Get-AuthenticodeSignature command failed: {stderr}"
            )));
        }

        let result = String::from_utf8_lossy(&output.stdout);
        let trimmed = result.trim();
        let parts: Vec<&str> = trimmed.splitn(2, '|').collect();

        let status = parts.first().copied().unwrap_or("");
        let subject = parts.get(1).copied().unwrap_or("");

        if !status.eq_ignore_ascii_case("Valid") {
            return Err(HandyError::AuthenticodeInvalid(format!(
                "Signature status is '{status}', expected 'Valid'"
            )));
        }

        if !subject.contains(HANDY_AUTHENTICODE_SUBJECT_SUBSTR) {
            return Err(HandyError::AuthenticodeInvalid(format!(
                "Signature subject '{subject}' does not contain expected '{HANDY_AUTHENTICODE_SUBJECT_SUBSTR}'"
            )));
        }

        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = file_path;
        Ok(())
    }
}
