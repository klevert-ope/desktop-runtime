// app:// protocol: serve from embedded UI, CSP, no external URLs.

use include_dir::{Dir, File};
use std::borrow::Cow;

/// CSP for app://.
const CSP: &str = "default-src 'self'; script-src 'self'; connect-src 'none';";

fn mime(path: &str) -> &'static str {
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

/// Serve one request from embedded UI dir. Returns (body, mime_type). Path is URI path (e.g. "/" or "/assets/foo.js"). No filesystem access.
/// Rejects path traversal (e.g. "..") so only files inside the embedded UI tree are served.
pub fn serve(ui: &'static Dir, uri_path: &str) -> Option<(Cow<'static, [u8]>, &'static str)> {
    let path = uri_path.trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    let path = path.trim_end_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    // Reject path traversal; include_dir is embedded so ".." has no meaning but reject for safety.
    if path.contains("..") {
        return None;
    }

    let file: &File = ui.get_file(path)?;
    let contents = file.contents();
    let mime_type = mime(path);
    Some((Cow::Borrowed(contents), mime_type))
}

/// Response helper with CSP and Content-Type.
#[allow(dead_code)]
pub fn response(body: Cow<'static, [u8]>, mime_type: &'static str) -> http::Response<Cow<'static, [u8]>> {
    http::Response::builder()
        .status(200)
        .header("Content-Type", mime_type)
        .header("Content-Security-Policy", CSP)
        .header("X-Content-Type-Options", "nosniff")
        .body(body)
        .expect("response")
}
