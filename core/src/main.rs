//! Desktop runtime: single event loop, one WebView, embedded UI, typed IPC.
//!
//! No Tokio spawn in the main loop; use `Runtime::new_current_thread()` only if async is needed.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod event_loop;
mod ipc;
mod paths;
mod protocol;
mod storage;
mod window;

#[cfg(test)]
mod protocol_tests;

use crate::config::{
    ENV_DEVTOOLS, IPC_WORKER_POOL_SIZE, MAX_PENDING_IPC, SHOW_WINDOW_FALLBACK_SECS, UI,
    WINDOW_HEIGHT, WINDOW_MIN_HEIGHT, WINDOW_MIN_WIDTH, WINDOW_WIDTH,
};
use crate::event_loop::{run_event_loop, UserEvent};
use crate::ipc::{handle_command, is_blocking_command, parse_message, IpcResponse};
use crate::paths::user_data_dir;
use crate::protocol::{serve, ServeResult};
use crate::window::{init_script, window_icon};
use tao::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;
use tao::event_loop::EventLoopBuilder;
use wry::http::Response;
use wry::WebViewBuilder;

/// Exits the process with code 1 after logging. Use for unrecoverable startup failures.
fn exit_fatal(msg: &str) -> ! {
    log::error!("{}", msg);
    std::process::exit(1);
}

/// Pushes one IPC response JSON to the queue and sends `IpcFlush` only when this is the first item
/// (so the event loop is woken once per batch). Recovers from mutex poison so a panicking thread
/// cannot leave the queue permanently locked and cause unbounded growth or deadlock.
fn push_ipc_and_wake(proxy: &tao::event_loop::EventLoopProxy<UserEvent>, queue: &Mutex<Vec<String>>, json: String) {
    let was_first = {
        let mut q = queue.lock().unwrap_or_else(|e| {
            log::error!("IPC queue mutex was poisoned, recovering");
            e.into_inner()
        });
        q.push(json);
        q.len() == 1
    };
    if was_first {
        let _ = proxy.send_event(UserEvent::IpcFlush);
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .init();

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let pending_ipc = Arc::new(AtomicUsize::new(0));
    let ipc_queue: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let ipc_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(IPC_WORKER_POOL_SIZE)
        .build()
        .unwrap_or_else(|e| exit_fatal(&format!("IPC worker pool: {}", e)));

    let window = {
        let mut b = tao::window::WindowBuilder::new()
            .with_title("Desktop Runtime")
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .with_min_inner_size(LogicalSize::new(WINDOW_MIN_WIDTH, WINDOW_MIN_HEIGHT))
            .with_visible(false);
        if let Some(icon) = window_icon() {
            b = b.with_window_icon(Some(icon));
        }
        if let Some(bounds) = storage::load_window_bounds() {
            b = b
                .with_position(PhysicalPosition::new(bounds.x, bounds.y))
                .with_inner_size(PhysicalSize::new(bounds.width, bounds.height));
        }
        b.build(&event_loop).unwrap_or_else(|e| {
            exit_fatal(&format!("Failed to create window: {}", e));
        })
    };

    let ipc_proxy = proxy.clone();
    let pending_ipc_handler = Arc::clone(&pending_ipc);
    let ipc_queue_handler = Arc::clone(&ipc_queue);
    let ipc_handler = move |req: wry::http::Request<String>| {
        let body = req.body();
        let Some(envelope) = parse_message(body) else { return };

        if is_blocking_command(&envelope.command) {
            if pending_ipc_handler.load(Ordering::Relaxed) >= MAX_PENDING_IPC {
                log::warn!("IPC backpressure: dropping blocking request (id={})", envelope.id);
                return;
            }
            pending_ipc_handler.fetch_add(1, Ordering::Relaxed);
            let worker_proxy = ipc_proxy.clone();
            let worker_pending = Arc::clone(&pending_ipc_handler);
            let worker_queue = Arc::clone(&ipc_queue_handler);
            ipc_pool.spawn(move || {
                let resp = match handle_command(&envelope.command) {
                    Ok(data) => IpcResponse::ok(envelope.id, data),
                    Err(e) => IpcResponse::err(envelope.id, e),
                };
                if let Ok(json) = serde_json::to_string(&resp) {
                    push_ipc_and_wake(&worker_proxy, &worker_queue, json);
                } else {
                    worker_pending.fetch_sub(1, Ordering::Relaxed);
                }
            });
            return;
        }

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
            push_ipc_and_wake(&ipc_proxy, &ipc_queue_handler, json);
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
            .unwrap()
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

    run_event_loop(event_loop, webview, window, web_context, proxy, pending_ipc, ipc_queue);
}
