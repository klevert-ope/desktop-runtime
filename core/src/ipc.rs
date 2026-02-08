//! Typed IPC between webview and host: JSON envelope, single entry point, no string dispatch.
//!
//! The UI sends `{ id, name, ...args }`; the host returns `{ id, ok? | err? }`. Invalid messages
//! are ignored (no panic). Timeout is enforced in the UI (see `IPC_TIMEOUT_MS`).

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Timeout in ms for an IPC round-trip. Enforced in the UI (e.g. bridge.js); keep in sync with the frontend.
#[allow(dead_code)]
pub const IPC_TIMEOUT_MS: u64 = 30_000;

/// GitHub repo for update checks (owner/name). Set at build via `DESKTOP_RUNTIME_GITHUB_REPO` or derived from CARGO_PKG_REPOSITORY.
const GITHUB_REPO: &str = env!("GITHUB_REPO_FOR_UPDATES", "Set GITHUB_REPO_FOR_UPDATES via build.rs");

/// Allowed URL schemes for OpenUrl. Prevents file:// and other non-http(s) opens from the UI.
const ALLOWED_URL_SCHEMES: [&str; 2] = ["https://", "http://"];

// ---------------------------------------------------------------------------
// Envelope and command
// ---------------------------------------------------------------------------

/// Incoming message: `id` (correlation) + flattened command (`name` + args).
#[derive(Debug, Clone, Deserialize)]
pub struct IpcEnvelope {
    pub id: String,
    #[serde(flatten)]
    pub command: Command,
}

/// Commands the UI can send. Tagged with `name` for deserialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "name")]
pub enum Command {
    ReadConfig,
    WriteConfig { data: ConfigPayload },
    Ping,
    OpenFileDialog,
    GetVersion,
    CheckForUpdates,
    OpenUrl { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPayload {
    pub key: String,
    pub value: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Response
// ---------------------------------------------------------------------------

/// Outgoing response correlated by `id`. Exactly one of `ok` or `err` is set.
#[derive(Debug, Clone, Serialize)]
pub struct IpcResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ok: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub err: Option<String>,
}

impl IpcResponse {
    #[must_use]
    pub fn ok(id: String, data: serde_json::Value) -> Self {
        Self {
            id,
            ok: Some(data),
            err: None,
        }
    }

    #[must_use]
    pub fn err(id: String, message: String) -> Self {
        Self {
            id,
            ok: None,
            err: Some(message),
        }
    }
}

// ---------------------------------------------------------------------------
// Parse and handle
// ---------------------------------------------------------------------------

/// Parses a raw IPC message. Invalid JSON or missing required fields return `None` (ignored safely).
#[must_use]
pub fn parse_message(raw: &str) -> Option<IpcEnvelope> {
    serde_json::from_str(raw).ok()
}

/// Handles one command synchronously. Returns a JSON-serializable value on success or an error string.
pub fn handle_command(command: &Command) -> Result<serde_json::Value, String> {
    match command {
        Command::ReadConfig => Ok(serde_json::json!({ "config": {} })),
        Command::WriteConfig { data: _ } => Ok(serde_json::json!({ "written": true })),
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
            if !ALLOWED_URL_SCHEMES.iter().any(|s| url.starts_with(s)) {
                return Err("URL must be http:// or https://".to_string());
            }
            opener::open(url).map_err(|e| e.to_string())?;
            Ok(serde_json::json!({ "opened": true }))
        }
    }
}

// ---------------------------------------------------------------------------
// Update check
// ---------------------------------------------------------------------------

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

/// Compares two semver-like strings. Returns 1 if a > b, -1 if a < b, 0 if equal. Non-numeric segments treated as 0.
#[must_use]
pub fn semver_compare(a: &str, b: &str) -> i32 {
    let parse = |s: &str| {
        let parts: Vec<u64> = s.split('.').map(|p| p.parse::<u64>().unwrap_or(0)).collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };
    let (ma, mi, pa) = parse(a);
    let (mb, mj, pb) = parse(b);
    if ma != mb {
        return if ma > mb { 1 } else { -1 };
    }
    if mi != mj {
        return if mi > mj { 1 } else { -1 };
    }
    if pa != pb {
        return if pa > pb { 1 } else { -1 };
    }
    0
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_message_valid_ping() {
        let raw = r#"{"id":"abc-123","name":"Ping"}"#;
        let env = parse_message(raw).expect("valid");
        assert_eq!(env.id, "abc-123");
        assert!(matches!(env.command, Command::Ping));
    }

    #[test]
    fn parse_message_invalid_returns_none() {
        assert!(parse_message("").is_none());
        assert!(parse_message("{}").is_none());
        assert!(parse_message("not json").is_none());
    }

    #[test]
    fn semver_compare_equal() {
        assert_eq!(semver_compare("1.0.0", "1.0.0"), 0);
        assert_eq!(semver_compare("0.0.0", "0.0.0"), 0);
    }

    #[test]
    fn semver_compare_greater() {
        assert_eq!(semver_compare("2.0.0", "1.0.0"), 1);
        assert_eq!(semver_compare("1.1.0", "1.0.0"), 1);
        assert_eq!(semver_compare("1.0.1", "1.0.0"), 1);
    }

    #[test]
    fn semver_compare_less() {
        assert_eq!(semver_compare("1.0.0", "2.0.0"), -1);
        assert_eq!(semver_compare("1.0.0", "1.1.0"), -1);
        assert_eq!(semver_compare("1.0.0", "1.0.1"), -1);
    }

    #[test]
    fn open_url_rejects_non_http() {
        let cmd = Command::OpenUrl {
            url: "file:///etc/passwd".to_string(),
        };
        assert!(handle_command(&cmd).is_err());
        // https is allowed (actual open is not run in test to avoid system dependency)
        let cmd = Command::OpenUrl {
            url: "javascript:alert(1)".to_string(),
        };
        assert!(handle_command(&cmd).is_err());
    }
}
