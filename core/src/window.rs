//! Window and WebView setup helpers.
//!
//! Icon loading, init script, and related UI glue live here so main and
//! event handling stay focused on orchestration.
//! The app icon PNG is decoded once and reused for both window and tray.

use std::sync::OnceLock;
use tao::window::Icon;

/// Cached decoded icon (RGBA pixels, width, height). Decoded once at first use.
fn decoded_icon() -> Option<&'static (Vec<u8>, u32, u32)> {
    static CACHED: OnceLock<Option<(Vec<u8>, u32, u32)>> = OnceLock::new();
    CACHED.get_or_init(|| {
        let bytes = include_bytes!("../../packaging/icons/react.png");
        let img = image::load_from_memory(bytes).ok()?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        Some((rgba.into_raw(), w, h))
    }).as_ref()
}

/// Loads the application window icon from the embedded asset (uses shared decode).
#[must_use]
pub fn window_icon() -> Option<Icon> {
    let (rgba, width, height) = decoded_icon()?;
    Icon::from_rgba(rgba.clone(), *width, *height).ok()
}

/// Loads the tray icon from the embedded asset (same decode as window icon).
#[must_use]
pub fn tray_icon() -> Option<tray_icon::Icon> {
    let (rgba, width, height) = decoded_icon()?;
    tray_icon::Icon::from_rgba(rgba.clone(), *width, *height).ok()
}

/// Returns the init script: disables context menu, exposes `window.native` and IPC resolve helpers.
#[must_use]
pub fn init_script() -> &'static str {
    r#"
        document.addEventListener('contextmenu', function(e) { e.preventDefault(); });
        window.native = {
            send: function(msg) {
                if (window.ipc && typeof window.ipc.postMessage === 'function') {
                    window.ipc.postMessage(msg);
                }
            }
        };
        window.__ipcResolve = window.__ipcResolve || {};
        window.__resolveIpc = function(id, json) {
            if (window.__ipcResolve[id]) {
                window.__ipcResolve[id](json);
                delete window.__ipcResolve[id];
            }
        };
    "#
}
