// Ensures ui/dist exists at compile time; runs npm install/build if missing.

use std::process::Command;

fn main() {
    // Override with DESKTOP_RUNTIME_GITHUB_REPO; else derived from CARGO_PKG_REPOSITORY.
    let repo = std::env::var("DESKTOP_RUNTIME_GITHUB_REPO")
        .ok()
        .or_else(|| {
            std::env::var("CARGO_PKG_REPOSITORY").ok().and_then(|url| {
                let s = url
                    .strip_prefix("https://github.com/")
                    .or_else(|| url.strip_prefix("https://www.github.com/"))?;
                Some(s.trim_end_matches('/').strip_suffix(".git").unwrap_or(s).to_string())
            })
        })
        .unwrap_or_else(|| "klevert-ope/desktop-runtime".to_string());
    println!("cargo:rustc-env=GITHUB_REPO_FOR_UPDATES={}", repo);
    let ui_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("ui");
    let dist = ui_dir.join("dist");
    if !dist.join("index.html").exists() {
        let _ = Command::new("npm")
            .args(["install"])
            .current_dir(&ui_dir)
            .status();
        let _ = Command::new("npm")
            .args(["run", "build"])
            .current_dir(&ui_dir)
            .status();
    }
    println!("cargo:rerun-if-changed=../packaging/icons/react.png");
    println!("cargo:rerun-if-changed=../ui/package.json");
    println!("cargo:rerun-if-changed=../ui/index.html");
    println!("cargo:rerun-if-changed=../ui/vite.config.js");
    println!("cargo:rerun-if-changed=../ui/src");
}
