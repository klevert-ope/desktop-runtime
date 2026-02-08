//! Build script: injects env for the crate and ensures the embedded UI is built.
//!
//! ## Environment (input)
//!
//! - `DESKTOP_RUNTIME_GITHUB_REPO` – Optional. `owner/repo` for update checks. If unset, derived from
//!   `CARGO_PKG_REPOSITORY` or defaults to `klevert-ope/desktop-runtime`.
//!
//! ## Emitted
//!
//! - `cargo:rustc-env=GITHUB_REPO_FOR_UPDATES=<repo>` – Consumed by `core/src/ipc.rs`.
//! - `cargo:rerun-if-changed=<path>` – So the crate rebuilds when UI or icons change.
//!
//! ## UI build
//!
//! If `../ui/dist/index.html` is missing, runs `npm install` and `npm run build` in `../ui`.
//! Failures are reported and the build fails so CI catches a broken frontend.

use std::path::Path;
use std::process::Command;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default GitHub repo (owner/name) when not set via env or CARGO_PKG_REPOSITORY.
const DEFAULT_GITHUB_REPO: &str = "klevert-ope/desktop-runtime";

/// Path to the UI app (relative to CARGO_MANIFEST_DIR).
const UI_DIR: &str = "../ui";

/// Path to the built UI entry (we check this to decide whether to run npm).
const DIST_INDEX: &str = "dist/index.html";

/// Paths that trigger a rerun of the build script when changed.
const RERUN_IF_CHANGED: &[&str] = &[
    "../packaging/icons/react.png",
    "../ui/package.json",
    "../ui/package-lock.json",
    "../ui/index.html",
    "../ui/vite.config.js",
    "../ui/src",
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns the GitHub repo (owner/name) for update checks: env override, then CARGO_PKG_REPOSITORY, then default.
fn github_repo_for_updates() -> String {
    if let Ok(repo) = std::env::var("DESKTOP_RUNTIME_GITHUB_REPO") {
        return repo.trim_end_matches('/').to_string();
    }
    if let Ok(url) = std::env::var("CARGO_PKG_REPOSITORY") {
        let s = url
            .strip_prefix("https://github.com/")
            .or_else(|| url.strip_prefix("https://www.github.com/"))
            .map(|s| s.trim_end_matches('/').strip_suffix(".git").unwrap_or(s).to_string());
        if let Some(repo) = s {
            return repo;
        }
    }
    DEFAULT_GITHUB_REPO.to_string()
}

/// Runs `npm install` in `ui_dir`. On failure, panics with a clear message.
fn npm_install(ui_dir: &Path) {
    let status = Command::new("npm")
        .args(["install"])
        .current_dir(ui_dir)
        .status();
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => panic!(
            "npm install failed in {} (exit code: {:?})",
            ui_dir.display(),
            s.code()
        ),
        Err(e) => panic!("failed to run npm install: {}", e),
    }
}

/// Runs `npm run build` in `ui_dir`. On failure, panics with a clear message.
fn npm_run_build(ui_dir: &Path) {
    let status = Command::new("npm")
        .args(["run", "build"])
        .current_dir(ui_dir)
        .status();
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => panic!(
            "npm run build failed in {} (exit code: {:?})",
            ui_dir.display(),
            s.code()
        ),
        Err(e) => panic!("failed to run npm run build: {}", e),
    }
}

/// Ensures `ui_dir/dist/index.html` exists by running npm install and npm run build if missing.
fn ensure_ui_build(ui_dir: &Path) {
    let dist_index = ui_dir.join(DIST_INDEX);
    if dist_index.exists() {
        return;
    }
    npm_install(ui_dir);
    npm_run_build(ui_dir);
    if !dist_index.exists() {
        panic!(
            "expected {} to exist after npm run build",
            dist_index.display()
        );
    }
}

fn main() {
    let repo = github_repo_for_updates();
    println!("cargo:rustc-env=GITHUB_REPO_FOR_UPDATES={}", repo);

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set by cargo");
    let ui_dir = Path::new(&manifest_dir).join(UI_DIR);
    ensure_ui_build(&ui_dir);

    for path in RERUN_IF_CHANGED {
        println!("cargo:rerun-if-changed={}", path);
    }
}
