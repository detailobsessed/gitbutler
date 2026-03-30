//! Cloudflare tunnel support.
//!
//! Two modes are supported:
//!
//! * **Quick tunnel** — no account needed. `cloudflared` is spawned
//!   with `--url` and assigns a random `trycloudflare.com` URL.  The URL is
//!   parsed from cloudflared's stderr output and returned.
//!
//! * **Named tunnel** — requires a pre-configured tunnel.
//!   `cloudflared` is invoked with `tunnel run --url ... <name>` and the URL
//!   is the supplied hostname (known upfront, no parsing needed).  See
//!   <https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/> for setup.
//!
//! Requires `cloudflared` to be installed. If it is not found a clear
//! error message with install instructions is printed.

use colored::Colorize as _;
use tokio::io::AsyncBufReadExt as _;

const TUNNEL_URL_TIMEOUT_SECS: u64 = 30;

/// Which kind of cloudflare tunnel to start.
pub enum Mode<'a> {
    /// Quick tunnel — no account needed, random `trycloudflare.com` URL.
    Quick,
    /// Named tunnel — requires `cloudflared tunnel login` + `cloudflared tunnel route dns`.
    /// Fields: `(tunnel_name, hostname)`.
    Named { name: &'a str, hostname: &'a str },
}

/// Spawn a cloudflared tunnel pointed at `http://127.0.0.1:{port}`.
///
/// Waits up to 30 seconds for cloudflared to become ready, then returns the
/// public URL and the child process. The child **must** be kept alive for the
/// tunnel to remain open; dropping it kills the process.
///
/// When `verbose` is true, cloudflared's output is forwarded to stderr.
pub async fn start(
    mode: Mode<'_>,
    port: u16,
    verbose: bool,
) -> anyhow::Result<(String, tokio::process::Child)> {
    let local_url = format!("http://127.0.0.1:{port}");

    let (mut child, url_source) = match &mode {
        Mode::Quick => {
            let child = spawn_cloudflared(&["tunnel", "--url", &local_url, "--no-autoupdate"])?;
            (child, UrlSource::ParseFromOutput)
        }
        Mode::Named { name, hostname } => {
            let child = spawn_cloudflared(&["tunnel", "run", "--url", &local_url, name])?;
            (child, UrlSource::Known(normalize_origin(hostname)?))
        }
    };

    let mut rx = merge_output(&mut child);

    // Wait for cloudflared to signal readiness.
    // For quick tunnels we parse the URL from output; for named tunnels we
    // wait for a "connection registered" log line.
    let url = tokio::time::timeout(
        std::time::Duration::from_secs(TUNNEL_URL_TIMEOUT_SECS),
        wait_for_ready(&mut rx, &url_source, verbose),
    )
    .await
    .map_err(|_| {
        let what = match url_source {
            UrlSource::ParseFromOutput => "cloudflared tunnel URL",
            UrlSource::Known(_) => "cloudflared to connect",
        };
        anyhow::anyhow!("Timed out after {TUNNEL_URL_TIMEOUT_SECS}s waiting for {what}")
    })??;

    // Keep draining cloudflared's output so its stdio pipes never fill up.
    // Without this, the write-blocking cloudflared would stall mid-proxy.
    tokio::spawn(async move {
        while let Some(line) = rx.recv().await {
            if verbose {
                eprintln!("{line}");
            }
        }
    });

    Ok((url, child))
}

enum UrlSource {
    /// Parse the tunnel URL from cloudflared's output (quick tunnel).
    ParseFromOutput,
    /// URL is already known (named tunnel) — just wait for connection.
    Known(String),
}

async fn wait_for_ready(
    rx: &mut tokio::sync::mpsc::Receiver<String>,
    url_source: &UrlSource,
    verbose: bool,
) -> anyhow::Result<String> {
    let mut seen = Vec::new();
    while let Some(line) = rx.recv().await {
        if verbose {
            eprintln!("{line}");
        }
        match url_source {
            UrlSource::ParseFromOutput => {
                if let Some(url) = extract_url(&line) {
                    return Ok(url);
                }
            }
            UrlSource::Known(url) => {
                if is_connected(&line) {
                    return Ok(url.clone());
                }
            }
        }
        seen.push(line);
    }
    // Process exited before becoming ready.
    let detail = if seen.is_empty() {
        "(no output)".to_string()
    } else {
        seen.join("\n")
    };
    let what = match url_source {
        UrlSource::ParseFromOutput => "reporting a tunnel URL",
        UrlSource::Known(_) => "the tunnel was established",
    };
    Err(anyhow::anyhow!(
        "cloudflared exited before {what}:\n{detail}"
    ))
}

/// Spawn `cloudflared` with the given arguments, returning the child process.
///
/// Translates a "not found" OS error into a user-friendly message with install
/// instructions; other spawn errors are returned as-is.
fn spawn_cloudflared(args: &[&str]) -> anyhow::Result<tokio::process::Child> {
    tokio::process::Command::new("cloudflared")
        .args(args)
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                let install_hint = if cfg!(target_os = "macos") {
                    "brew install cloudflared"
                } else if cfg!(target_os = "windows") {
                    "winget install --id Cloudflare.cloudflared"
                } else {
                    "https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/"
                };
                anyhow::anyhow!(
                    "{}\nInstall: {}",
                    "cloudflared is not installed.".bold(),
                    install_hint.cyan()
                )
            } else {
                anyhow::anyhow!("Failed to spawn cloudflared: {e}")
            }
        })
}

/// Spawn two tasks that each read one stream and forward lines to a shared channel.
///
/// Using `chain` would read stdout to EOF before touching stderr; this
/// interleaves both so neither stream starves the other.
fn merge_output(child: &mut tokio::process::Child) -> tokio::sync::mpsc::Receiver<String> {
    let (tx, rx) = tokio::sync::mpsc::channel(64);

    let stdout = child.stdout.take().expect("stdout is piped");
    let tx_out = tx.clone();
    tokio::spawn(async move {
        let mut lines = tokio::io::BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if tx_out.send(line).await.is_err() {
                break;
            }
        }
    });

    let stderr = child.stderr.take().expect("stderr is piped");
    tokio::spawn(async move {
        let mut lines = tokio::io::BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if tx.send(line).await.is_err() {
                break;
            }
        }
    });

    rx
}

/// Normalise an arbitrary hostname or URL string into a canonical `https://<host>` origin.
///
/// Accepts:
/// * A bare hostname: `my-tunnel.example.com`
/// * A URL with a scheme: `https://my-tunnel.example.com` or `http://my-tunnel.example.com`
///
/// Any path, query, or fragment component is stripped; the scheme is always
/// normalised to `https://`.  Returns an error when the input cannot be parsed
/// as a URL or contains no host component.
///
/// The returned string exactly matches what a browser sends in the `Origin`
/// header (scheme + host + optional non-default port), so it is safe to use
/// directly for CORS / origin-allow-list comparisons.
pub fn normalize_origin(input: &str) -> anyhow::Result<String> {
    // If the caller omitted the scheme, prepend https:// so url::Url can parse it.
    let with_scheme;
    let to_parse = if input.contains("://") {
        input
    } else {
        with_scheme = format!("https://{input}");
        &with_scheme
    };

    let parsed =
        url::Url::parse(to_parse).map_err(|e| anyhow::anyhow!("invalid origin {input:?}: {e}"))?;

    anyhow::ensure!(
        matches!(parsed.scheme(), "http" | "https"),
        "invalid origin {:?}: scheme must be http or https, got {:?}",
        input,
        parsed.scheme()
    );

    let host = parsed
        .host_str()
        .filter(|h| !h.is_empty())
        .ok_or_else(|| anyhow::anyhow!("invalid origin {input:?}: no host component found"))?;

    // Reconstruct as https://<host> (with port only when non-default).
    let origin = match parsed.port() {
        Some(port) => format!("https://{host}:{port}"),
        None => format!("https://{host}"),
    };
    Ok(origin)
}

/// Returns `true` when a cloudflared log line confirms the tunnel is connected.
fn is_connected(line: &str) -> bool {
    // cloudflared emits this once the first connection to the Cloudflare edge
    // is registered and ready to accept traffic.
    line.contains("Registered tunnel connection")
        || line.contains("Connection registered")
        || line.contains("connsReady=1")
}

/// Extract a `https://*.trycloudflare.com` URL from a cloudflared log line.
fn extract_url(line: &str) -> Option<String> {
    let marker = ".trycloudflare.com";
    let end = line.find(marker)? + marker.len();
    let start = line[..end].rfind("https://")?;
    Some(line[start..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tunnel_url_from_log_line() {
        let line = "2024-01-01T00:00:00Z INF |  https://example-words-here.trycloudflare.com  |";
        assert_eq!(
            extract_url(line),
            Some("https://example-words-here.trycloudflare.com".into())
        );
    }

    #[test]
    fn returns_none_for_unrelated_lines() {
        assert_eq!(extract_url("INFO starting tunnel"), None);
    }

    #[test]
    fn detects_registered_connection() {
        assert!(is_connected(
            "2024-01-01T00:00:00Z INF Registered tunnel connection connIndex=0"
        ));
        assert!(is_connected("INF connsReady=1"));
        assert!(!is_connected("INF Starting tunnel tunnelID=abc"));
    }

    #[test]
    fn normalize_origin_cases() {
        // Bare hostname — scheme is inferred as https.
        assert_eq!(
            normalize_origin("my-tunnel.example.com").unwrap(),
            "https://my-tunnel.example.com"
        );
        // https:// is preserved as-is.
        assert_eq!(
            normalize_origin("https://my-tunnel.example.com").unwrap(),
            "https://my-tunnel.example.com"
        );
        // http:// is accepted and normalised to https.
        assert_eq!(
            normalize_origin("http://my-tunnel.example.com").unwrap(),
            "https://my-tunnel.example.com"
        );
        // Path, query, and fragment are stripped.
        assert_eq!(
            normalize_origin("https://my-tunnel.example.com/path?q=1#frag").unwrap(),
            "https://my-tunnel.example.com"
        );
        // Query without a path is also stripped.
        assert_eq!(
            normalize_origin("https://my-tunnel.example.com?q=1").unwrap(),
            "https://my-tunnel.example.com"
        );
        // Trailing slash is stripped.
        assert_eq!(
            normalize_origin("https://my-tunnel.example.com/").unwrap(),
            "https://my-tunnel.example.com"
        );
        // Non-default port is preserved (browsers include it in the Origin header).
        assert_eq!(
            normalize_origin("https://my-tunnel.example.com:8443").unwrap(),
            "https://my-tunnel.example.com:8443"
        );
    }

    #[test]
    fn normalize_origin_rejects_invalid() {
        // Empty string.
        assert!(normalize_origin("").is_err());
        // Scheme only, no host.
        assert!(normalize_origin("https://").is_err());
        // Non-http/https scheme.
        assert!(normalize_origin("ftp://my-tunnel.example.com").is_err());
    }
}
