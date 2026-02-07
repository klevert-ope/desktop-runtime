// Single event loop, one WebView, embedded UI, typed IPC. No Tokio spawn; use Runtime::new_current_thread() only if async needed.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ipc;
mod protocol;

use crate::ipc::{handle_command, parse_message, IpcResponse};
use crate::protocol::serve;
use include_dir::include_dir;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::Icon;
use wry::http::Response;
use wry::WebViewBuilder;

/// Max pending IPC responses before we drop new ones (backpressure).
const MAX_PENDING_IPC: usize = 256;

static UI: include_dir::Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../ui/dist");

/// Window/taskbar icon (React logo), loaded from packaging/icons/react.png.
fn window_icon() -> Option<Icon> {
    let bytes = include_bytes!("../../packaging/icons/react.png");
    let img = image::load_from_memory(bytes).ok()?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Icon::from_rgba(rgba.into_raw(), width, height).ok()
}

/// Carries IPC response JSON back into the WebView for request correlation.
struct IpcUserEvent {
    response_json: String,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(
        if cfg!(debug_assertions) { "info" } else { "warn" },
    ))
    .init();

    let mut builder = EventLoopBuilder::<IpcUserEvent>::with_user_event();
    let event_loop = builder.build();
    let proxy = event_loop.create_proxy();
    let pending_ipc = Arc::new(AtomicUsize::new(0));

    let mut window_builder = tao::window::WindowBuilder::new()
        .with_title("Desktop Runtime")
        .with_inner_size(tao::dpi::LogicalSize::new(800.0, 600.0));
    if let Some(icon) = window_icon() {
        window_builder = window_builder.with_window_icon(Some(icon));
    }
    let window = match window_builder.build(&event_loop) {
        Ok(w) => w,
        Err(e) => {
            log::error!("Failed to create window: {}", e);
            std::process::exit(1);
        }
    };

    let ipc_proxy = proxy.clone();
    let pending_ipc_handler = Arc::clone(&pending_ipc);
    let ipc_handler = move |req: wry::http::Request<String>| {
        let body = req.body();
        if let Some(envelope) = parse_message(body) {
            let result = handle_command(&envelope.command);
            let resp = match result {
                Ok(data) => IpcResponse::ok(envelope.id, data),
                Err(e) => IpcResponse::err(envelope.id, e),
            };
            if let Ok(json) = serde_json::to_string(&resp) {
                if pending_ipc_handler.load(Ordering::Relaxed) >= MAX_PENDING_IPC {
                    log::warn!("IPC backpressure: dropping response (id={})", resp.id);
                    return;
                }
                pending_ipc_handler.fetch_add(1, Ordering::Relaxed);
                if ipc_proxy.send_event(IpcUserEvent {
                    response_json: json,
                }).is_err() {
                    pending_ipc_handler.fetch_sub(1, Ordering::Relaxed);
                    log::warn!("IPC send_event failed (event loop may be gone)");
                }
            }
        }
    };

    let app_protocol = "app".to_string();
    let handler = move |_: wry::WebViewId<'_>, request: wry::http::Request<Vec<u8>>| {
        let path = request.uri().path();
        let (body, mime_type) = serve(&UI, path)
            .map(|(b, m)| (b, m))
            .unwrap_or_else(|| {
                (
                    std::borrow::Cow::Borrowed(b"Not Found".as_slice()),
                    "text/plain",
                )
            });
        let status = if body.len() == 9 && body.as_ref() == b"Not Found" {
            404
        } else {
            200
        };
        Response::builder()
            .status(status)
            .header("Content-Type", mime_type)
            .header("Content-Security-Policy", "default-src 'self'; script-src 'self'; connect-src 'none';")
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

    let init_script = r#"
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
    "#;

    let navigation_allow = move |url: String| url.starts_with("app://") || url.contains("app.localhost");

    // Use a user-writable data directory on all platforms so the web engine never writes
    // under install or admin locations (Program Files, /Applications, /usr, AppImage, etc.).
    let mut web_context = {
        use std::path::PathBuf;
        #[cfg(target_os = "windows")]
        let user_data_dir = std::env::var("LOCALAPPDATA").ok().and_then(|local| {
            let path = PathBuf::from(local).join("Desktop Runtime").join("WebView2");
            std::fs::create_dir_all(&path).ok().map(|()| path)
        });
        #[cfg(target_os = "macos")]
        let user_data_dir = std::env::var("HOME").ok().and_then(|home| {
            let path = PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Desktop Runtime");
            std::fs::create_dir_all(&path).ok().map(|()| path)
        });
        #[cfg(target_os = "linux")]
        let user_data_dir = {
            let base = std::env::var("XDG_DATA_HOME")
                .ok()
                .map(PathBuf::from)
                .or_else(|| {
                    std::env::var("HOME")
                        .ok()
                        .map(|h| PathBuf::from(h).join(".local").join("share"))
                });
            base.and_then(|p| {
                let path = p.join("desktop-runtime");
                std::fs::create_dir_all(&path).ok().map(|()| path)
            })
        };
        wry::WebContext::new(user_data_dir)
    };

    let builder = WebViewBuilder::new_with_web_context(&mut web_context)
        .with_custom_protocol(app_protocol, handler)
        .with_url("app://localhost/index.html")
        .with_ipc_handler(ipc_handler)
        .with_initialization_script(init_script)
        .with_navigation_handler(navigation_allow)
        .with_devtools(false);

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    let webview = builder.build(&window).unwrap_or_else(|e| {
        log::error!("Failed to build webview: {}", e);
        std::process::exit(1);
    });

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let webview = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;
        let vbox = window.default_vbox().unwrap_or_else(|| {
            log::error!("Failed to get GTK vbox");
            std::process::exit(1);
        });
        builder.build_gtk(vbox).unwrap_or_else(|e| {
            log::error!("Failed to build webview: {}", e);
            std::process::exit(1);
        })
    };

    // Keep web_context alive for the lifetime of the webview (required on some platforms).
    let web_context = web_context;

    event_loop.run(move |event, _, control_flow| {
        let _ = &web_context;
        *control_flow = ControlFlow::Wait;

        if let tao::event::Event::UserEvent(IpcUserEvent { response_json }) = event {
            pending_ipc.fetch_sub(1, Ordering::Relaxed);
            // Single-buffer escape to avoid N small Vec allocations per IPC response (allocation churn / peak memory).
            let mut escaped = String::with_capacity(response_json.len() * 2);
            for c in response_json.chars() {
                match c {
                    '\\' => escaped.push_str("\\\\"),
                    '"' => escaped.push_str("\\\""),
                    '\n' => escaped.push_str("\\n"),
                    '\r' => escaped.push_str("\\r"),
                    other => escaped.push(other),
                }
            }
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
        }
    });
}
