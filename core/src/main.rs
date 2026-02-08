//! Desktop runtime: single event loop, one WebView, embedded UI, typed IPC.
//!
//! No Tokio spawn in the main loop; use `Runtime::new_current_thread()` only if async is needed.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ipc;
mod protocol;

use crate::ipc::{handle_command, parse_message, IpcResponse};
use crate::protocol::{serve, ServeResult};
use include_dir::include_dir;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::thread;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::Icon;
use wry::http::Response;
use wry::WebViewBuilder;

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Max pending IPC responses before dropping new ones (backpressure).
const MAX_PENDING_IPC: usize = 256;

/// Initial window size (logical).
const WINDOW_WIDTH: f64 = 800.0;
const WINDOW_HEIGHT: f64 = 600.0;

/// Minimum window size (logical).
const WINDOW_MIN_WIDTH: f64 = 400.0;
const WINDOW_MIN_HEIGHT: f64 = 300.0;

/// Seconds to wait before showing the window if the first page load never fires.
const SHOW_WINDOW_FALLBACK_SECS: u64 = 3;

/// Env var: set to "1" to enable WebView DevTools (avoids tao event-loop warnings when off).
const ENV_DEVTOOLS: &str = "DESKTOP_RUNTIME_DEVTOOLS";

/// Embedded UI (must match ui/dist at build time).
static UI: include_dir::Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../ui/dist");

// ---------------------------------------------------------------------------
// User events
// ---------------------------------------------------------------------------

enum UserEvent {
    Ipc { response_json: String },
    ShowWindow,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Exits the process with code 1 after logging. Use for unrecoverable startup failures.
fn exit_fatal(msg: &str) -> ! {
    log::error!("{}", msg);
    std::process::exit(1);
}

fn window_icon() -> Option<Icon> {
    let bytes = include_bytes!("../../packaging/icons/react.png");
    let img = image::load_from_memory(bytes).ok()?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Icon::from_rgba(rgba.into_raw(), width, height).ok()
}

/// Escapes a JSON string for safe embedding inside a JS string (backslash, quote, newline, carriage return).
fn escape_json_for_js(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            other => out.push(other),
        }
    }
    out
}

/// Returns the user data directory for the web engine. Prefers platform user dirs; falls back to temp so we never use install path.
fn user_data_dir() -> std::path::PathBuf {
    use std::path::PathBuf;

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

/// Builds the init script: disable context menu, expose `window.native` and IPC resolve helpers.
fn init_script() -> &'static str {
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

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(if cfg!(debug_assertions) {
            "info"
        } else {
            "warn"
        }),
    )
    .init();

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let pending_ipc = Arc::new(AtomicUsize::new(0));

    let window = {
        let mut b = tao::window::WindowBuilder::new()
            .with_title("Desktop Runtime")
            .with_inner_size(tao::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .with_min_inner_size(tao::dpi::LogicalSize::new(WINDOW_MIN_WIDTH, WINDOW_MIN_HEIGHT))
            .with_visible(false);
        if let Some(icon) = window_icon() {
            b = b.with_window_icon(Some(icon));
        }
        b.build(&event_loop).unwrap_or_else(|e| exit_fatal(&format!("Failed to create window: {}", e)))
    };

    let ipc_proxy = proxy.clone();
    let pending_ipc_handler = Arc::clone(&pending_ipc);
    let ipc_handler = move |req: wry::http::Request<String>| {
        let body = req.body();
        if let Some(envelope) = parse_message(body) {
            let resp = match handle_command(&envelope.command) {
                Ok(data) => IpcResponse::ok(envelope.id, data),
                Err(e) => IpcResponse::err(envelope.id, e),
            };
            if let Ok(json) = serde_json::to_string(&resp) {
                if pending_ipc_handler.load(Ordering::Relaxed) >= MAX_PENDING_IPC {
                    log::warn!("IPC backpressure: dropping response (id={})", resp.id);
                    return;
                }
                pending_ipc_handler.fetch_add(1, Ordering::Relaxed);
                if ipc_proxy
                    .send_event(UserEvent::Ipc {
                        response_json: json,
                    })
                    .is_err()
                {
                    pending_ipc_handler.fetch_sub(1, Ordering::Relaxed);
                    log::warn!("IPC send_event failed (event loop may be gone)");
                }
            }
        }
    };

    let protocol_handler = move |_: wry::WebViewId<'_>, request: wry::http::Request<Vec<u8>>| {
        let path = request.uri().path();
        let (status, body, mime_type) = match serve(&UI, path) {
            ServeResult::Found { body, mime_type } => (200, body, mime_type),
            ServeResult::NotFound => (
                404,
                std::borrow::Cow::Borrowed(b"Not Found".as_slice()),
                "text/plain",
            ),
        };
        Response::builder()
            .status(status)
            .header("Content-Type", mime_type)
            .header("Content-Security-Policy", protocol::CSP)
            .header("X-Content-Type-Options", "nosniff")
            .body(body)
            .unwrap_or_else(|e| {
                log::error!("Protocol response build failed: {}", e);
                Response::builder()
                    .status(500)
                    .body(std::borrow::Cow::Borrowed(b"Internal Server Error".as_slice()))
                    .expect("fallback response")
            })
    };

    let navigation_allow = move |url: String| url.starts_with("app://") || url.contains("app.localhost");

    let show_window_proxy = proxy.clone();
    let shown = Arc::new(AtomicUsize::new(0));
    let on_page_load = {
        let p = show_window_proxy.clone();
        let s = Arc::clone(&shown);
        move |_event: wry::PageLoadEvent, _url: String| {
            if s.fetch_add(1, Ordering::Relaxed) == 0 {
                let _ = p.send_event(UserEvent::ShowWindow);
            }
        }
    };
    {
        let p = proxy.clone();
        let s = Arc::clone(&shown);
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(SHOW_WINDOW_FALLBACK_SECS));
            if s.fetch_add(1, Ordering::Relaxed) == 0 {
                let _ = p.send_event(UserEvent::ShowWindow);
            }
        });
    }

    let mut web_context = wry::WebContext::new(Some(user_data_dir()));
    let devtools = std::env::var(ENV_DEVTOOLS).as_deref() == Ok("1");

    let builder = WebViewBuilder::new_with_web_context(&mut web_context)
        .with_custom_protocol("app".to_string(), protocol_handler)
        .with_url("app://localhost/index.html")
        .with_ipc_handler(ipc_handler)
        .with_initialization_script(init_script())
        .with_navigation_handler(navigation_allow)
        .with_on_page_load_handler(on_page_load)
        .with_devtools(devtools);

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    let webview = builder.build(&window).unwrap_or_else(|e| {
        exit_fatal(&format!("Failed to build webview: {}", e));
    });

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let webview = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;
        let vbox = window.default_vbox().unwrap_or_else(|| exit_fatal("Failed to get GTK vbox"));
        builder.build_gtk(vbox).unwrap_or_else(|e| {
            exit_fatal(&format!("Failed to build webview: {}", e));
        })
    };

    run_event_loop(event_loop, webview, window, web_context, pending_ipc);
}

/// Runs the tao event loop until exit. Keeps `web_context` and `window` alive for the lifetime of `webview`.
fn run_event_loop(
    event_loop: tao::event_loop::EventLoop<UserEvent>,
    webview: wry::WebView,
    window: tao::window::Window,
    _web_context: wry::WebContext,
    pending_ipc: Arc<AtomicUsize>,
) {
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let tao::event::Event::UserEvent(UserEvent::ShowWindow) = event {
            window.set_visible(true);
            return;
        }

        if let tao::event::Event::UserEvent(UserEvent::Ipc { response_json }) = event {
            pending_ipc.fetch_sub(1, Ordering::Relaxed);
            let escaped = escape_json_for_js(&response_json);
            let script = format!(
                r#"if (window.__resolveIpc) {{ try {{ var r = JSON.parse("{}"); window.__resolveIpc(r.id, r); }} catch(e) {{}} }}"#,
                escaped
            );
            if let Err(e) = webview.evaluate_script(&script) {
                log::warn!("IPC evaluate_script failed: {}", e);
            }
            return;
        }

        if let tao::event::Event::WindowEvent {
            event: tao::event::WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
            return;
        }

        if let tao::event::Event::MainEventsCleared = event {
            return;
        }
        if let tao::event::Event::RedrawEventsCleared = event {
            return;
        }
    });
}
