//! Event loop and user events.
//!
//! Owns `UserEvent`, `run_event_loop`, and the JSON escape helper used only
//! when dispatching IPC responses back to the WebView.
//! IPC responses are batched: producers push to a queue and send `IpcFlush`;
//! the main loop drains the queue and delivers all in one `evaluate_script`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::storage;

/// User-defined events sent from background threads or IPC into the main loop.
#[allow(dead_code)]
pub enum UserEvent {
    /// Wake to drain the IPC response queue and deliver a batch to the WebView.
    IpcFlush,
    /// Request to show the window (after first load or fallback timeout).
    ShowWindow,
    /// Hide window (e.g. minimize to tray).
    HideWindow,
    /// Exit the application.
    Quit,
}

/// Escapes a JSON string for safe embedding inside a JS string (backslash, quote, newline, carriage return).
/// Avoids allocation when the string contains none of these characters.
#[must_use]
pub fn escape_json_for_js(s: &str) -> std::borrow::Cow<'_, str> {
    if !s.contains(['\\', '"', '\n', '\r']) {
        return std::borrow::Cow::Borrowed(s);
    }
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
    std::borrow::Cow::Owned(out)
}

/// Drains the IPC queue and runs one script to deliver all responses. Returns true if any were delivered.
/// Recovers from mutex poison so the queue can be drained and memory released.
fn drain_ipc_queue_and_deliver(
    queue: &Mutex<Vec<String>>,
    pending_ipc: &AtomicUsize,
    webview: &wry::WebView,
) -> bool {
    let batch: Vec<String> = {
        let mut q = queue.lock().unwrap_or_else(|e| {
            log::error!("IPC queue mutex was poisoned, recovering");
            e.into_inner()
        });
        std::mem::take(&mut *q)
    };
    let n = batch.len();
    if n == 0 {
        return false;
    }
    let to_sub = n.min(pending_ipc.load(Ordering::Relaxed));
    pending_ipc.fetch_sub(to_sub, Ordering::Relaxed);

    let mut script = String::from("if (window.__resolveIpc) { ");
    for response_json in batch {
        let escaped = escape_json_for_js(&response_json);
        script.push_str(&format!(
            r#"try {{ var r = JSON.parse("{}"); window.__resolveIpc(r.id, r); }} catch(e) {{}}"#,
            escaped
        ));
    }
    script.push_str(" }");
    if let Err(e) = webview.evaluate_script(&script) {
        log::warn!("IPC evaluate_script failed: {}", e);
    }
    true
}

/// Runs the tao event loop until exit.
///
/// Keeps `web_context`, `window`, and `_tray_icon` alive for the lifetime of `webview`.
/// Uses `ControlFlow::Poll` after draining IPC so the loop re-runs immediately
/// when there is pending work; otherwise `Wait` to avoid busy-waiting.
pub fn run_event_loop(
    event_loop: tao::event_loop::EventLoop<UserEvent>,
    webview: wry::WebView,
    window: tao::window::Window,
    _web_context: wry::WebContext,
    event_proxy: tao::event_loop::EventLoopProxy<UserEvent>,
    pending_ipc: Arc<AtomicUsize>,
    ipc_queue: Arc<Mutex<Vec<String>>>,
) {
    let mut tray_icon_holder: Option<tray_icon::TrayIcon> = None;
    let show_proxy = event_proxy.clone();
    let quit_proxy = event_proxy.clone();

    event_loop.run(move |event, _event_loop, control_flow| {
        *control_flow = tao::event_loop::ControlFlow::Wait;

        // Create tray icon on first run (required on macOS: event loop must be running).
        if tray_icon_holder.is_none()
            && let Some(icon) = crate::window::tray_icon()
        {
            let proxy = event_proxy.clone();
            let qp = quit_proxy.clone();
            let menu = tray_icon::menu::Menu::new();
            let show_id = tray_icon::menu::MenuId::new("show");
            let quit_id = tray_icon::menu::MenuId::new("quit");
            menu.append(&tray_icon::menu::MenuItem::with_id(
                show_id.clone(),
                "Show",
                true,
                None,
            ))
            .ok();
            menu.append(&tray_icon::menu::MenuItem::with_id(
                quit_id.clone(),
                "Quit",
                true,
                None,
            ))
            .ok();
            let sp = show_proxy.clone();
            tray_icon::TrayIconEvent::set_event_handler(Some(move |_| {
                let _ = sp.send_event(UserEvent::ShowWindow);
            }));
            tray_icon::menu::MenuEvent::set_event_handler(Some(
                move |event: tray_icon::menu::MenuEvent| {
                    if event.id == show_id {
                        let _ = proxy.send_event(UserEvent::ShowWindow);
                    } else if event.id == quit_id {
                        let _ = qp.send_event(UserEvent::Quit);
                    }
                },
            ));
            if let Ok(tray) = tray_icon::TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip("Desktop Runtime")
                .with_icon(icon)
                .build()
            {
                tray_icon_holder = Some(tray);
            }
        }

        if let tao::event::Event::UserEvent(ev) = event {
            match ev {
                UserEvent::ShowWindow => {
                    window.set_visible(true);
                }
                UserEvent::HideWindow => {
                    window.set_visible(false);
                }
                UserEvent::Quit => {
                    *control_flow = tao::event_loop::ControlFlow::Exit;
                }
                UserEvent::IpcFlush => {
                    let had_work = drain_ipc_queue_and_deliver(&ipc_queue, &pending_ipc, &webview);
                    if had_work {
                        *control_flow = tao::event_loop::ControlFlow::Poll;
                    }
                }
            }
            return;
        }

        if let tao::event::Event::WindowEvent {
            event: tao::event::WindowEvent::CloseRequested,
            ..
        } = event
        {
            if let Ok(pos) = window.outer_position() {
                let size = window.inner_size();
                storage::save_window_bounds(pos.x, pos.y, size.width, size.height);
            }
            *control_flow = tao::event_loop::ControlFlow::Exit;
            return;
        }

        if let tao::event::Event::MainEventsCleared = event {
            if drain_ipc_queue_and_deliver(&ipc_queue, &pending_ipc, &webview) {
                *control_flow = tao::event_loop::ControlFlow::Poll;
            }
            return;
        }
        if let tao::event::Event::RedrawEventsCleared = event {}
    });
}
