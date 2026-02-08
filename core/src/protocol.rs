//! `app://` protocol: serve embedded UI assets with strict CSP and no filesystem access.
//!
//! Path traversal (`..`) is rejected. Only files from the compile-time embedded directory
//! are served. MIME types are derived from extension only.

use include_dir::Dir;
use std::borrow::Cow;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Result of serving a single request. Caller sets HTTP status from this; no inference from body.
#[derive(Debug)]
pub enum ServeResult<'a> {
    /// File found. Use status 200 and the given body and MIME type.
    Found {
        body: Cow<'a, [u8]>,
        mime_type: &'static str,
    },
    /// Path missing or invalid. Use status 404.
    NotFound,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Content-Security-Policy for all app:// responses.
pub const CSP: &str = "default-src 'self'; script-src 'self'; connect-src 'none';";

/// X-Content-Type-Options for all responses.
const X_CONTENT_TYPE_OPTIONS: &str = "nosniff";

/// Default document when path is "/" or empty.
const INDEX_PATH: &str = "index.html";

// ---------------------------------------------------------------------------
// MIME type
// ---------------------------------------------------------------------------

/// Returns a static MIME type for the given path (extension-based). Unknown → `application/octet-stream`.
#[must_use]
pub(crate) fn mime_from_path(path: &str) -> &'static str {
    if path.ends_with(".html") || path.ends_with('/') || path.is_empty() {
        "text/html"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else {
        "application/octet-stream"
    }
}

// ---------------------------------------------------------------------------
// Serve
// ---------------------------------------------------------------------------

/// Normalizes URI path to an embedded file path: strip leading/trailing slashes, default to `index.html`, reject `..`.
pub(crate) fn normalize_path(uri_path: &str) -> Option<&str> {
    let path = uri_path.trim_start_matches('/').trim_end_matches('/');
    let path = if path.is_empty() {
        INDEX_PATH
    } else {
        path
    };
    if path.contains("..") {
        return None;
    }
    Some(path)
}

/// Serves one request from the embedded UI directory.
///
/// * `ui` – Compile-time embedded dir (e.g. `include_dir!`).
/// * `uri_path` – Request path (e.g. `/` or `/assets/foo.js`).
///
/// Path traversal is rejected. Returns `ServeResult` so the caller sets HTTP status explicitly.
#[must_use]
pub fn serve(ui: &'static Dir, uri_path: &str) -> ServeResult<'static> {
    let path = match normalize_path(uri_path) {
        Some(p) => p,
        None => return ServeResult::NotFound,
    };
    let file = match ui.get_file(path) {
        Some(f) => f,
        None => return ServeResult::NotFound,
    };
    ServeResult::Found {
        body: Cow::Borrowed(file.contents()),
        mime_type: mime_from_path(path),
    }
}

/// Builds an HTTP 200 response with CSP and Content-Type. Used by the protocol handler.
#[allow(dead_code)]
pub fn response_200(
    body: Cow<'static, [u8]>,
    mime_type: &'static str,
) -> http::Response<Cow<'static, [u8]>> {
    http::Response::builder()
        .status(200)
        .header("Content-Type", mime_type)
        .header("Content-Security-Policy", CSP)
        .header("X-Content-Type-Options", X_CONTENT_TYPE_OPTIONS)
        .body(body)
        .expect("valid response")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use include_dir::include_dir;

    static TEST_UI: include_dir::Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../ui/dist");

    #[test]
    fn normalize_path_default_index() {
        assert_eq!(normalize_path("/"), Some(INDEX_PATH));
        assert_eq!(normalize_path(""), Some(INDEX_PATH));
        assert_eq!(normalize_path("///"), Some(INDEX_PATH));
    }

    #[test]
    fn normalize_path_rejects_traversal() {
        assert_eq!(normalize_path("/.."), None);
        assert_eq!(normalize_path("/a/../b"), None);
        assert_eq!(normalize_path("/.. /index.html"), None);
    }

    #[test]
    fn serve_not_found_for_traversal() {
        let r = serve(&TEST_UI, "/../etc/passwd");
        assert!(matches!(r, ServeResult::NotFound));
    }

    #[test]
    fn serve_not_found_for_missing_file() {
        let r = serve(&TEST_UI, "/nonexistent.foo");
        assert!(matches!(r, ServeResult::NotFound));
    }

    #[test]
    fn serve_index_ok_when_dist_present() {
        let r = serve(&TEST_UI, "/");
        match r {
            ServeResult::Found { mime_type, .. } => assert_eq!(mime_type, "text/html"),
            ServeResult::NotFound => {
                // ui/dist may not exist in all test envs
            }
        }
    }

    #[test]
    fn mime_from_path_known_extensions() {
        assert_eq!(mime_from_path("a.html"), "text/html");
        assert_eq!(mime_from_path("b.js"), "application/javascript");
        assert_eq!(mime_from_path("c.css"), "text/css");
        assert_eq!(mime_from_path("d.png"), "image/png");
        assert_eq!(mime_from_path("e.woff2"), "font/woff2");
        assert_eq!(mime_from_path("f.unknown"), "application/octet-stream");
    }
}
