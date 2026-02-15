//! Update checking, download, and install.
//!
//! Isolated from generic command handling so protocol and network concerns
//! stay in one place.

use std::fs;
use std::io::Write;
use std::path::Path;

/// GitHub repo (owner/name) for update checks. Set at build via `DESKTOP_RUNTIME_GITHUB_REPO` or derived from CARGO_PKG_REPOSITORY.
pub(super) const GITHUB_REPO: &str =
    env!("GITHUB_REPO_FOR_UPDATES", "Set GITHUB_REPO_FOR_UPDATES via build.rs");

/// Preferred asset extensions per platform (first match wins).
#[cfg(target_os = "windows")]
const ASSET_EXTENSIONS: &[&str] = &[".msi", ".exe"];

#[cfg(target_os = "macos")]
const ASSET_EXTENSIONS: &[&str] = &[".pkg", ".dmg", ".app.tar.gz"];

#[cfg(target_os = "linux")]
const ASSET_EXTENSIONS: &[&str] = &[".AppImage", ".appimage", ".deb"];

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
const ASSET_EXTENSIONS: &[&str] = &[];

fn pick_asset_url(assets: &serde_json::Value) -> Option<String> {
    let arr = assets.as_array()?;
    for ext in ASSET_EXTENSIONS {
        for a in arr {
            let name = a["name"].as_str()?;
            if name.ends_with(ext) {
                return a["browser_download_url"].as_str().map(String::from);
            }
        }
    }
    arr.first()
        .and_then(|a| a["browser_download_url"].as_str())
        .map(String::from)
}

/// Fetches latest release info from GitHub and returns a JSON-serializable value.
pub(super) fn check_for_updates() -> Result<serde_json::Value, String> {
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
        .ok_or("No html_url in response")?
        .to_string();
    let asset_url = body.get("assets").and_then(pick_asset_url);

    let is_newer = semver_compare(latest, current) > 0;

    Ok(serde_json::json!({
        "current": current,
        "latest": latest,
        "url": html_url,
        "assetUrl": asset_url,
        "isNewer": is_newer
    }))
}

/// Downloads an update from the given URL to a temp file. Returns the local path.
pub(super) fn download_update(url: &str) -> Result<serde_json::Value, String> {
    if !url.starts_with("https://") {
        return Err("Download URL must be https://".to_string());
    }
    let resp = ureq::get(url)
        .set("User-Agent", "Desktop-Runtime-Update-Check")
        .call()
        .map_err(|e| e.to_string())?;

    let mut reader = resp.into_reader();
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut reader, &mut bytes).map_err(|e| e.to_string())?;

    let ext = Path::new(url)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let file_name = format!("desktop-runtime-update.{}", ext);
    let temp_dir = std::env::temp_dir();
    let dest = temp_dir.join(&file_name);

    let mut file = fs::File::create(&dest).map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "path": dest.display().to_string()
    }))
}

/// Launches the installer at the given path using the system default handler.
pub(super) fn install_update(path: &str) -> Result<serde_json::Value, String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err("Installer file not found".to_string());
    }
    #[cfg(target_os = "linux")]
    {
        if path.ends_with(".AppImage") || path.ends_with(".appimage") {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms).map_err(|e| e.to_string())?;
        }
    }
    opener::open(path).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "launched": true }))
}

/// Compares two semver-like strings. Returns 1 if a > b, -1 if a < b, 0 if equal. Non-numeric segments treated as 0.
#[must_use]
pub fn semver_compare(a: &str, b: &str) -> i32 {
    let mut ai = a.split('.').map(|p| p.parse::<u64>().unwrap_or(0));
    let mut bi = b.split('.').map(|p| p.parse::<u64>().unwrap_or(0));
    loop {
        let va = ai.next();
        let vb = bi.next();
        match (va, vb) {
            (None, None) => return 0,
            (Some(a_seg), Some(b_seg)) if a_seg != b_seg => return if a_seg > b_seg { 1 } else { -1 },
            (Some(_), Some(_)) => {}
            (Some(x), None) => return if x > 0 { 1 } else { 0 },
            (None, Some(y)) => return if y > 0 { -1 } else { 0 },
        }
    }
}
