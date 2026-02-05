// Typed IPC: JSON envelope, single entry point, no string dispatch.

use serde::{Deserialize, Serialize};

/// IPC timeout (ms). Enforced in UI (bridge.js).
#[allow(dead_code)]
pub const IPC_TIMEOUT_MS: u64 = 30_000;

/// Envelope: `id` (UUID), `name` (command).
#[derive(Debug, Clone, Deserialize)]
pub struct IpcEnvelope {
    pub id: String,
    #[serde(flatten)]
    pub command: Command,
}

/// Typed command enum — no dynamic payloads, no string-based dispatch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "name")]
pub enum Command {
    ReadConfig,
    WriteConfig { data: ConfigPayload },
    Ping,
    /// Phase 8: File dialog (on demand only; no shell, no plugins).
    OpenFileDialog,
    /// Returns current app version (from CARGO_PKG_VERSION).
    GetVersion,
    /// Checks GitHub releases for newer version. Returns { current, latest?, url?, isNewer }.
    CheckForUpdates,
    /// Opens URL in default system browser.
    OpenUrl { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPayload {
    pub key: String,
    pub value: serde_json::Value,
}

/// Response correlated by `id`.
#[derive(Debug, Clone, Serialize)]
pub struct IpcResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ok: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub err: Option<String>,
}

impl IpcResponse {
    pub fn ok(id: String, data: serde_json::Value) -> Self {
        IpcResponse {
            id,
            ok: Some(data),
            err: None,
        }
    }
    pub fn err(id: String, message: String) -> Self {
        IpcResponse {
            id,
            ok: None,
            err: Some(message),
        }
    }
}

/// Parse incoming message; invalid messages return None (ignored safely).
pub fn parse_message(raw: &str) -> Option<IpcEnvelope> {
    serde_json::from_str(raw).ok()
}

/// Override via DESKTOP_RUNTIME_GITHUB_REPO at build.
const GITHUB_REPO: &str = env!("GITHUB_REPO_FOR_UPDATES", "Set GITHUB_REPO_FOR_UPDATES via build.rs");

/// Sync handler; no reflection.
pub fn handle_command(command: &Command) -> Result<serde_json::Value, String> {
    match command {
        Command::ReadConfig => {
            // Placeholder: return empty config
            Ok(serde_json::json!({ "config": {} }))
        }
        Command::WriteConfig { data: _ } => {
            // Placeholder: accept and acknowledge
            Ok(serde_json::json!({ "written": true }))
        }
        Command::Ping => Ok(serde_json::json!({ "pong": true })),
        Command::OpenFileDialog => {
            let path = rfd::FileDialog::new().pick_file();
            Ok(serde_json::json!({
                "path": path.map(|p| p.display().to_string())
            }))
        }
        Command::GetVersion => Ok(serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "releasesUrl": format!("https://github.com/{}/releases", GITHUB_REPO)
        })),
        Command::CheckForUpdates => check_for_updates(),
        Command::OpenUrl { url } => {
            opener::open(url).map_err(|e| e.to_string())?;
            Ok(serde_json::json!({ "opened": true }))
        }
    }
}

/// GitHub API latest release vs current version.
fn check_for_updates() -> Result<serde_json::Value, String> {
    let current = env!("CARGO_PKG_VERSION");
    let api_url = format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO);

    let resp = ureq::get(&api_url)
        .set("Accept", "application/vnd.github.v3+json")
        .set("User-Agent", "Desktop-Runtime-Update-Check")
        .call()
        .map_err(|e| e.to_string())?;

    let body: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
    let tag_name = body["tag_name"].as_str().ok_or("No tag_name in response")?;
    let latest = tag_name.trim_start_matches('v');
    let html_url = body["html_url"]
        .as_str()
        .unwrap_or("https://github.com")
        .to_string();

    let is_newer = semver_compare(latest, current) > 0;

    Ok(serde_json::json!({
        "current": current,
        "latest": latest,
        "url": html_url,
        "isNewer": is_newer
    }))
}

/// Compare semver strings. Returns 1 if a > b, -1 if a < b, 0 if equal.
fn semver_compare(a: &str, b: &str) -> i32 {
    let parse = |s: &str| {
        let parts: Vec<u64> = s
            .split('.')
            .map(|p| p.parse::<u64>().unwrap_or(0))
            .collect();
        (parts.get(0).copied().unwrap_or(0), parts.get(1).copied().unwrap_or(0), parts.get(2).copied().unwrap_or(0))
    };
    let (ma, mi, pa) = parse(a);
    let (mb, mj, pb) = parse(b);
    if ma > mb { return 1; }
    if ma < mb { return -1; }
    if mi > mj { return 1; }
    if mi < mj { return -1; }
    if pa > pb { return 1; }
    if pa < pb { return -1; }
    0
}
