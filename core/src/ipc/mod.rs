//! Typed IPC between webview and host: JSON envelope, single entry point, no string dispatch.
//!
//! The UI sends `{ id, name, ...args }`; the host returns `{ id, ok? | err? }`. Invalid messages
//! are ignored (no panic). Timeout is enforced in the UI (see `IPC_TIMEOUT_MS`).

mod updates;

use crate::storage;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Timeout in ms for an IPC round-trip. Enforced in the UI (e.g. bridge.js); keep in sync with the frontend.
#[allow(dead_code)]
pub const IPC_TIMEOUT_MS: u64 = 30_000;

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

/// File filter for dialogs: human-readable name and list of extensions (e.g. `["png", "jpg"]`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// Commands the UI can send. Tagged with `name` for deserialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "name")]
pub enum Command {
    ReadConfig,
    WriteConfig { data: ConfigPayload },
    Ping,
    OpenFileDialog,
    OpenFileDialogWithFilters { filters: Vec<FileFilter> },
    SaveFileDialog {
        #[serde(default)]
        default_name: Option<String>,
        #[serde(default)]
        filters: Option<Vec<FileFilter>>,
    },
    OpenFolderDialog,
    GetVersion,
    CheckForUpdates,
    DownloadUpdate { url: String },
    InstallUpdate { path: String },
    OpenUrl { url: String },
    GetSystemInfo,
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

/// True for commands that may block (I/O, network, dialogs). Run these on a worker thread.
#[must_use]
pub fn is_blocking_command(command: &Command) -> bool {
    matches!(
        command,
        Command::OpenFileDialog
            | Command::OpenFileDialogWithFilters { .. }
            | Command::SaveFileDialog { .. }
            | Command::OpenFolderDialog
            | Command::CheckForUpdates
            | Command::DownloadUpdate { .. }
            | Command::InstallUpdate { .. }
            | Command::OpenUrl { .. }
    )
}

/// Parses a raw IPC message. Invalid JSON or missing required fields return `None` (ignored safely).
#[must_use]
pub fn parse_message(raw: &str) -> Option<IpcEnvelope> {
    serde_json::from_str(raw).ok()
}

/// Handles one command synchronously. Returns a JSON-serializable value on success or an error string.
pub fn handle_command(command: &Command) -> Result<serde_json::Value, String> {
    match command {
        Command::ReadConfig => Ok(serde_json::json!({ "config": storage::get_full_config() })),
        Command::WriteConfig { data } => {
            storage::set_value(data.key.clone(), data.value.clone());
            Ok(serde_json::json!({ "written": true }))
        }
        Command::Ping => Ok(serde_json::json!({ "pong": true })),
        Command::OpenFileDialog => {
            let path = rfd::FileDialog::new().pick_file();
            Ok(serde_json::json!({
                "path": path.map(|p| p.display().to_string())
            }))
        }
        Command::OpenFileDialogWithFilters { filters } => {
            let mut dlg = rfd::FileDialog::new();
            for f in filters {
                let exts: Vec<&str> = f.extensions.iter().map(String::as_str).collect();
                dlg = dlg.add_filter(&f.name, &exts);
            }
            let path = dlg.pick_file();
            Ok(serde_json::json!({
                "path": path.map(|p| p.display().to_string())
            }))
        }
        Command::SaveFileDialog {
            default_name,
            filters,
        } => {
            let mut dlg = rfd::FileDialog::new();
            if let Some(name) = default_name {
                dlg = dlg.set_file_name(name);
            }
            if let Some(f) = filters {
                for filter in f {
                    let exts: Vec<&str> = filter.extensions.iter().map(String::as_str).collect();
                    dlg = dlg.add_filter(&filter.name, &exts);
                }
            }
            let path = dlg.save_file();
            Ok(serde_json::json!({
                "path": path.map(|p| p.display().to_string())
            }))
        }
        Command::OpenFolderDialog => {
            let path = rfd::FileDialog::new().pick_folder();
            Ok(serde_json::json!({
                "path": path.map(|p| p.display().to_string())
            }))
        }
        Command::GetVersion => Ok(serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "releasesUrl": format!("https://github.com/{}/releases", updates::GITHUB_REPO)
        })),
        Command::CheckForUpdates => updates::check_for_updates(),
        Command::DownloadUpdate { url } => updates::download_update(url),
        Command::InstallUpdate { path } => updates::install_update(path),
        Command::OpenUrl { url } => {
            if !ALLOWED_URL_SCHEMES.iter().any(|s| url.starts_with(s)) {
                return Err("URL must be http:// or https://".to_string());
            }
            opener::open(url).map_err(|e| e.to_string())?;
            Ok(serde_json::json!({ "opened": true }))
        }
        Command::GetSystemInfo => {
            let info = serde_json::json!({
                "os": std::env::consts::OS,
                "arch": std::env::consts::ARCH,
                "family": std::env::consts::FAMILY,
                "hostname": std::env::var("COMPUTERNAME")
                    .ok()
                    .or_else(|| std::env::var("HOSTNAME").ok())
                    .unwrap_or_else(|| "unknown".to_string()),
                "appVersion": env!("CARGO_PKG_VERSION"),
            });
            Ok(serde_json::json!({ "info": info }))
        }
    }
}

#[cfg(test)]
mod tests;
