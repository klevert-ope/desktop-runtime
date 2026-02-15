//! Application configuration and compile-time constants.
//!
//! Centralizes window dimensions, IPC limits, env vars, and embedded UI path
//! so the rest of the crate stays decoupled from concrete values.

use include_dir::include_dir;

/// Max pending IPC responses before dropping new ones (backpressure).
/// Also bounds IPC queue memory: at most this many response strings are queued at once.
pub const MAX_PENDING_IPC: usize = 256;

/// Number of worker threads for blocking IPC commands (e.g. file dialog, update check).
pub const IPC_WORKER_POOL_SIZE: usize = 4;

/// Initial window size (logical).
pub const WINDOW_WIDTH: f64 = 800.0;

/// Initial window height (logical).
pub const WINDOW_HEIGHT: f64 = 600.0;

/// Minimum window width (logical).
pub const WINDOW_MIN_WIDTH: f64 = 400.0;

/// Minimum window height (logical).
pub const WINDOW_MIN_HEIGHT: f64 = 300.0;

/// Seconds to wait before showing the window if the first page load never fires.
pub const SHOW_WINDOW_FALLBACK_SECS: u64 = 3;

/// Env var: set to `"1"` to enable WebView DevTools.
pub const ENV_DEVTOOLS: &str = "DESKTOP_RUNTIME_DEVTOOLS";

/// Embedded UI directory (must match `ui/dist` at build time).
pub static UI: include_dir::Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../ui/dist");
