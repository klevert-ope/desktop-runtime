//! Platform-specific paths and directory resolution.
//!
//! Keeps filesystem and environment concerns in one place so the rest of the
//! runtime does not depend on platform-specific env vars or paths.
//! User data dir is computed once at first use to avoid repeated env and I/O at startup.

use std::path::PathBuf;
use std::sync::OnceLock;

static USER_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

fn compute_user_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    let preferred = std::env::var("LOCALAPPDATA").ok().map(|local| {
        PathBuf::from(local).join("Desktop Runtime").join("WebView2")
    });

    #[cfg(target_os = "macos")]
    let preferred = std::env::var("HOME").ok().map(|home| {
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("Desktop Runtime")
    });

    #[cfg(target_os = "linux")]
    let preferred = std::env::var("XDG_DATA_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".local").join("share"))
        })
        .map(|p| p.join("desktop-runtime"));

    preferred
        .and_then(|path| std::fs::create_dir_all(&path).ok().map(|()| path))
        .unwrap_or_else(|| {
            let fallback = std::env::temp_dir().join("Desktop-Runtime");
            if std::fs::create_dir_all(&fallback).is_err() {
                log::warn!("Could not create user data dir; using temp_dir as-is");
            }
            fallback
        })
}

/// Returns the user data directory for the web engine (cached after first use).
///
/// Prefers platform user dirs; falls back to temp so we never use the install path.
#[must_use]
pub fn user_data_dir() -> PathBuf {
    USER_DATA_DIR
        .get_or_init(compute_user_data_dir)
        .clone()
}

/// Returns the app config directory (parent of user_data_dir on Windows, same on macOS/Linux).
#[must_use]
#[allow(dead_code)]
pub fn app_config_dir() -> PathBuf {
    let base = user_data_dir();
    base.parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| base)
}
