//! Proxy-aware WebSocket connect helpers (`ws_connect_with_proxy` and the
//! `BoxedIo`/`ProxiedWsStream` transport types): resolve the effective proxy
//! for a service key and tunnel the TCP connection (HTTP CONNECT / SOCKS5)
//! before handing the stream to `tokio_tungstenite`.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::{ws_connect_with_proxy, BoxedIo, ProxiedWsStream}` are
//! unchanged.

use super::*;

// ── Proxy-aware WebSocket connect ────────────────────────────────
//
// `tokio_tungstenite::connect_async` does not honour proxy settings.
// The helpers below resolve the effective proxy URL for a given service
// key and, when a proxy is active, establish a tunnelled TCP connection
// (HTTP CONNECT for http/https proxies, SOCKS5 for socks5/socks5h)
// before handing the stream to `tokio_tungstenite` for the WebSocket
// handshake.

/// Combined async IO trait for boxed WebSocket transport streams.
trait AsyncReadWrite: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send {}
impl<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send> AsyncReadWrite for T {}

/// A boxed async IO stream used when a WebSocket connection is tunnelled
/// through a proxy. The concrete type varies depending on the proxy
/// kind (HTTP CONNECT vs SOCKS5) and the target scheme (ws vs wss).
///
/// We wrap in a newtype so we can implement `AsyncRead` and `AsyncWrite`
/// via delegation, since Rust trait objects cannot combine multiple
/// non-auto traits.
pub struct BoxedIo(Box<dyn AsyncReadWrite>);

impl tokio::io::AsyncRead for BoxedIo {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut *self.0).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for BoxedIo {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::Pin::new(&mut *self.0).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut *self.0).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut *self.0).poll_shutdown(cx)
    }
}

impl Unpin for BoxedIo {}

/// Convenience alias for the WebSocket stream returned by the proxy-aware
/// connect helpers.
pub type ProxiedWsStream = tokio_tungstenite::WebSocketStream<BoxedIo>;

/// Resolve the effective proxy URL for a WebSocket connection to the
/// given `ws_url`, taking into account the per-channel `proxy_url`
/// override, the runtime proxy config, scope and no_proxy list.
fn resolve_ws_proxy_url(
    service_key: &str,
    ws_url: &str,
    channel_proxy_url: Option<&str>,
) -> Option<String> {
    // 1. Explicit per-channel proxy always wins.
    if let Some(url) = normalize_proxy_url_option(channel_proxy_url) {
        return Some(url);
    }

    // 2. Consult the runtime proxy config.
    let cfg = runtime_proxy_config();
    if !cfg.should_apply_to_service(service_key) {
        return None;
    }

    // Check the no_proxy list against the WebSocket target host.
    if let Ok(parsed) = reqwest::Url::parse(ws_url)
        && let Some(host) = parsed.host_str()
    {
        let no_proxy_entries = cfg.normalized_no_proxy();
        if !no_proxy_entries.is_empty() {
            let host_lower = host.to_ascii_lowercase();
            let matches_no_proxy = no_proxy_entries.iter().any(|entry| {
                let entry = entry.trim().to_ascii_lowercase();
                if entry == "*" {
                    return true;
                }
                if host_lower == entry {
                    return true;
                }
                // Support ".example.com" matching "foo.example.com"
                if let Some(suffix) = entry.strip_prefix('.') {
                    return host_lower.ends_with(suffix) || host_lower == suffix;
                }
                // Support "example.com" also matching "foo.example.com"
                host_lower.ends_with(&format!(".{entry}"))
            });
            if matches_no_proxy {
                return None;
            }
        }
    }

    // For wss:// prefer https_proxy, for ws:// prefer http_proxy, fall
    // back to all_proxy in both cases.
    let is_secure = ws_url.starts_with("wss://") || ws_url.starts_with("wss:");
    let preferred = if is_secure {
        normalize_proxy_url_option(cfg.https_proxy.as_deref())
    } else {
        normalize_proxy_url_option(cfg.http_proxy.as_deref())
    };
    preferred.or_else(|| normalize_proxy_url_option(cfg.all_proxy.as_deref()))
}

/// Connect a WebSocket through the configured proxy (if any).
///
/// When no proxy applies, this is a thin wrapper around
/// `tokio_tungstenite::connect_async`. When a proxy is active the
/// function tunnels the TCP connection through the proxy before
/// performing the WebSocket upgrade.
///
/// `service_key` is the proxy-service selector (e.g. `"channel.discord"`).
/// `channel_proxy_url` is the optional per-channel proxy override.
pub async fn ws_connect_with_proxy(
    ws_url: &str,
    service_key: &str,
    channel_proxy_url: Option<&str>,
) -> anyhow::Result<(
    ProxiedWsStream,
    tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>>,
)> {
    let proxy_url = resolve_ws_proxy_url(service_key, ws_url, channel_proxy_url);

    match proxy_url {
        None => {
            // No proxy — establish TCP+TLS manually, wrap in BoxedIo, then
            // perform the WebSocket handshake over the wrapped stream.
            //
            // Previous implementation used `connect_async` followed by
            // `into_inner()` + `from_raw_socket` to normalize the return
            // type. That pattern discards data already buffered by the
            // tungstenite frame codec, causing channels (Slack Socket Mode,
            // Discord, etc.) to silently miss the first frames sent by the
            // server and all subsequent events.
            use tokio::net::TcpStream;

            let target = reqwest::Url::parse(ws_url)
                .with_context(|| format!("Invalid WebSocket URL: {ws_url}"))?;
            let target_host = target
                .host_str()
                .ok_or_else(|| {
                    ::omnesagent_log::record!(
                        WARN,
                        ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Reject)
                            .with_outcome(::omnesagent_log::EventOutcome::Failure)
                            .with_attrs(::serde_json::json!({"ws_url": ws_url})),
                        "WebSocket URL has no host"
                    );
                    anyhow::Error::msg(format!("WebSocket URL has no host: {ws_url}"))
                })?
                .to_string();
            let target_port = target
                .port_or_known_default()
                .unwrap_or(if target.scheme() == "wss" { 443 } else { 80 });

            let tcp = TcpStream::connect(format!("{target_host}:{target_port}"))
                .await
                .with_context(|| format!("TCP connect to {target_host}:{target_port}"))?;

            let is_secure = target.scheme() == "wss";
            let stream: BoxedIo = if is_secure {
                let mut root_store = rustls::RootCertStore::empty();
                root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
                let tls_config = std::sync::Arc::new(
                    rustls::ClientConfig::builder()
                        .with_root_certificates(root_store)
                        .with_no_client_auth(),
                );
                let connector = tokio_rustls::TlsConnector::from(tls_config);
                let server_name = rustls_pki_types::ServerName::try_from(target_host.clone())
                    .with_context(|| format!("Invalid TLS server name: {target_host}"))?;
                let tls_stream = connector
                    .connect(server_name, tcp)
                    .await
                    .with_context(|| format!("TLS handshake with {target_host}"))?;
                BoxedIo(Box::new(tls_stream))
            } else {
                BoxedIo(Box::new(tcp))
            };

            let default_port = if is_secure { 443 } else { 80 };
            let host_header = if target_port == default_port {
                target_host.clone()
            } else {
                format!("{target_host}:{target_port}")
            };

            let ws_request = tokio_tungstenite::tungstenite::http::Request::builder()
                .uri(ws_url)
                .header("Host", host_header)
                .header("Connection", "Upgrade")
                .header("Upgrade", "websocket")
                .header(
                    "Sec-WebSocket-Key",
                    tokio_tungstenite::tungstenite::handshake::client::generate_key(),
                )
                .header("Sec-WebSocket-Version", "13")
                .body(())
                .with_context(|| "Failed to build WebSocket upgrade request")?;

            let (ws_stream, response) =
                tokio_tungstenite::client_async(ws_request, stream)
                    .await
                    .with_context(|| format!("WebSocket handshake failed for {ws_url}"))?;

            Ok((ws_stream, response))
        }
        Some(proxy) => ws_connect_via_proxy(ws_url, &proxy).await,
    }
}

/// Establish a WebSocket connection tunnelled through the given proxy URL.
async fn ws_connect_via_proxy(
    ws_url: &str,
    proxy_url: &str,
) -> anyhow::Result<(
    ProxiedWsStream,
    tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>>,
)> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt as _};
    use tokio::net::TcpStream;

    let target =
        reqwest::Url::parse(ws_url).with_context(|| format!("Invalid WebSocket URL: {ws_url}"))?;
    let target_host = target
        .host_str()
        .ok_or_else(|| {
            ::omnesagent_log::record!(
                WARN,
                ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Reject)
                    .with_outcome(::omnesagent_log::EventOutcome::Failure)
                    .with_attrs(::serde_json::json!({"ws_url": ws_url})),
                "WebSocket URL has no host"
            );
            anyhow::Error::msg(format!("WebSocket URL has no host: {ws_url}"))
        })?
        .to_string();
    let target_port = target
        .port_or_known_default()
        .unwrap_or(if target.scheme() == "wss" { 443 } else { 80 });

    let proxy = reqwest::Url::parse(proxy_url)
        .with_context(|| format!("Invalid proxy URL: {proxy_url}"))?;

    let stream: BoxedIo = match proxy.scheme() {
        "socks5" | "socks5h" | "socks" => {
            let proxy_addr = format!(
                "{}:{}",
                proxy.host_str().unwrap_or("127.0.0.1"),
                proxy.port_or_known_default().unwrap_or(1080)
            );
            let target_addr = format!("{target_host}:{target_port}");
            let socks_stream = if proxy.username().is_empty() {
                tokio_socks::tcp::Socks5Stream::connect(proxy_addr.as_str(), target_addr.as_str())
                    .await
                    .with_context(|| format!("SOCKS5 connect to {target_addr} via {proxy_addr}"))?
            } else {
                let password = proxy.password().unwrap_or("");
                tokio_socks::tcp::Socks5Stream::connect_with_password(
                    proxy_addr.as_str(),
                    target_addr.as_str(),
                    proxy.username(),
                    password,
                )
                .await
                .with_context(|| format!("SOCKS5 auth connect to {target_addr} via {proxy_addr}"))?
            };
            let tcp: TcpStream = socks_stream.into_inner();
            BoxedIo(Box::new(tcp))
        }
        "http" | "https" => {
            let proxy_host = proxy.host_str().unwrap_or("127.0.0.1");
            let proxy_port = proxy.port_or_known_default().unwrap_or(8080);
            let proxy_addr = format!("{proxy_host}:{proxy_port}");

            let mut tcp = TcpStream::connect(&proxy_addr)
                .await
                .with_context(|| format!("TCP connect to HTTP proxy {proxy_addr}"))?;

            // Send HTTP CONNECT request.
            let connect_req = format!(
                "CONNECT {target_host}:{target_port} HTTP/1.1\r\nHost: {target_host}:{target_port}\r\n\r\n"
            );
            tcp.write_all(connect_req.as_bytes()).await?;

            // Read the response (we only need the status line).
            let mut buf = vec![0u8; 4096];
            let mut total = 0usize;
            loop {
                let n = tcp.read(&mut buf[total..]).await?;
                if n == 0 {
                    anyhow::bail!("HTTP CONNECT proxy closed connection before response");
                }
                total += n;
                // Look for end of HTTP headers.
                if let Some(pos) = find_header_end(&buf[..total]) {
                    let status_line = std::str::from_utf8(&buf[..pos])
                        .unwrap_or("")
                        .lines()
                        .next()
                        .unwrap_or("");
                    if !status_line.contains("200") {
                        anyhow::bail!(
                            "HTTP CONNECT proxy returned non-200 response: {status_line}"
                        );
                    }
                    break;
                }
                if total >= buf.len() {
                    anyhow::bail!("HTTP CONNECT proxy response too large");
                }
            }

            BoxedIo(Box::new(tcp))
        }
        scheme => {
            anyhow::bail!("Unsupported proxy scheme '{scheme}' for WebSocket connections");
        }
    };

    // If the target is wss://, wrap in TLS.
    let is_secure = target.scheme() == "wss";
    let stream: BoxedIo = if is_secure {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let tls_config = std::sync::Arc::new(
            rustls::ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth(),
        );
        let connector = tokio_rustls::TlsConnector::from(tls_config);
        let server_name = rustls_pki_types::ServerName::try_from(target_host.clone())
            .with_context(|| format!("Invalid TLS server name: {target_host}"))?;

        // `stream` is `BoxedIo` — we need a concrete `AsyncRead + AsyncWrite`
        // for `TlsConnector::connect`. Since `BoxedIo` already satisfies
        // those bounds we can pass it directly.
        let tls_stream = connector
            .connect(server_name, stream)
            .await
            .with_context(|| format!("TLS handshake with {target_host}"))?;
        BoxedIo(Box::new(tls_stream))
    } else {
        stream
    };

    // Perform the WebSocket client handshake over the tunnelled stream.
    let ws_request = tokio_tungstenite::tungstenite::http::Request::builder()
        .uri(ws_url)
        .header("Host", format!("{target_host}:{target_port}"))
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header(
            "Sec-WebSocket-Key",
            tokio_tungstenite::tungstenite::handshake::client::generate_key(),
        )
        .header("Sec-WebSocket-Version", "13")
        .body(())
        .with_context(|| "Failed to build WebSocket upgrade request")?;

    let (ws_stream, response) = tokio_tungstenite::client_async(ws_request, stream)
        .await
        .with_context(|| format!("WebSocket handshake failed for {ws_url}"))?;

    Ok((ws_stream, response))
}

/// Find the `\r\n\r\n` boundary marking the end of HTTP headers.
fn find_header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4)
}
