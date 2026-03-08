use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::oneshot;
use tracing::{error, info, warn};

/// Ensure cloudflared is installed, then spawn the tunnel.
/// Returns None if cloudflared can't be installed.
pub async fn spawn_tunnel(port: u16) -> Option<oneshot::Receiver<String>> {
    if !ensure_cloudflared().await {
        return None;
    }
    let (tx, rx) = oneshot::channel();
    tokio::spawn(run_tunnel(port, tx));
    Some(rx)
}

/// Check if cloudflared is installed; if not, attempt to install it.
async fn ensure_cloudflared() -> bool {
    // Check if already installed
    if Command::new("cloudflared")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .is_ok()
    {
        return true;
    }

    eprintln!("  cloudflared not found. Installing...");

    let install_result = if cfg!(target_os = "macos") {
        Command::new("brew")
            .args(["install", "cloudflared"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await
    } else if cfg!(target_os = "linux") {
        // Download binary directly from GitHub releases
        let arch = if cfg!(target_arch = "aarch64") {
            "cloudflared-linux-arm64"
        } else {
            "cloudflared-linux-amd64"
        };
        let url =
            format!("https://github.com/cloudflare/cloudflared/releases/latest/download/{arch}");
        Command::new("sh")
            .args([
                "-c",
                &format!(
                    "curl -sL '{url}' -o /tmp/cloudflared && chmod +x /tmp/cloudflared && sudo mv /tmp/cloudflared /usr/local/bin/cloudflared"
                ),
            ])
            .status()
            .await
    } else if cfg!(target_os = "windows") {
        Command::new("winget")
            .args([
                "install",
                "--id",
                "Cloudflare.cloudflared",
                "--accept-source-agreements",
                "--accept-package-agreements",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await
    } else {
        eprintln!("  Unsupported OS for auto-install.");
        return false;
    };

    match install_result {
        Ok(status) if status.success() => {
            eprintln!("  cloudflared installed successfully.");
            true
        }
        Ok(status) => {
            error!("cloudflared install exited with: {status}");
            print_manual_instructions();
            false
        }
        Err(e) => {
            error!("Failed to install cloudflared: {e}");
            print_manual_instructions();
            false
        }
    }
}

fn print_manual_instructions() {
    eprintln!();
    eprintln!("  Could not auto-install cloudflared. Install manually:");
    eprintln!("    macOS:   brew install cloudflared");
    eprintln!(
        "    Linux:   https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/"
    );
    eprintln!("    Windows: winget install Cloudflare.cloudflared");
    eprintln!();
}

async fn run_tunnel(port: u16, url_tx: oneshot::Sender<String>) {
    let mut child = match Command::new("cloudflared")
        .args(["tunnel", "--url", &format!("http://localhost:{}", port)])
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            error!("Failed to start cloudflared: {e}");
            print_manual_instructions();
            return;
        }
    };

    let mut url_tx = Some(url_tx);

    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if let Some(url) = extract_tunnel_url(&line) {
                info!(url = %url, "Cloudflare tunnel active");
                if let Some(tx) = url_tx.take() {
                    let _ = tx.send(url);
                }
            }
            if line.contains("ERR") {
                warn!("cloudflared: {line}");
            }
        }
    }

    match child.wait().await {
        Ok(status) => warn!("cloudflared exited with: {status}"),
        Err(e) => error!("cloudflared process error: {e}"),
    }
}

fn extract_tunnel_url(line: &str) -> Option<String> {
    if line.contains(".trycloudflare.com") {
        line.split_whitespace()
            .find(|word| word.starts_with("https://") && word.contains(".trycloudflare.com"))
            .map(|s| s.to_string())
    } else {
        None
    }
}
