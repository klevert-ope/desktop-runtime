// Single event loop, one WebView, embedded UI, typed IPC. No Tokio spawn; use Runtime::new_current_thread() only if async needed.

mod ipc;
mod protocol;

use crate::ipc::{handle_command, parse_message, IpcResponse};
use crate::protocol::serve;
use include_dir::include_dir;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::Icon;
use wry::http::Response;
use wry::WebViewBuilder;

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
    let mut builder = EventLoopBuilder::<IpcUserEvent>::with_user_event();
    let event_loop = builder.build();
    let proxy = event_loop.create_proxy();

    let mut window_builder = tao::window::WindowBuilder::new()
        .with_title("Desktop Runtime")
        .with_inner_size(tao::dpi::LogicalSize::new(800.0, 600.0));
    if let Some(icon) = window_icon() {
        window_builder = window_builder.with_window_icon(Some(icon));
    }
    let window = window_builder.build(&event_loop).expect("create window");

    let ipc_proxy = proxy.clone();
    let ipc_handler = move |req: wry::http::Request<String>| {
        let body = req.body();
        if let Some(envelope) = parse_message(body) {
            let result = handle_command(&envelope.command);
            let resp = match result {
                Ok(data) => IpcResponse::ok(envelope.id, data),
                Err(e) => IpcResponse::err(envelope.id, e),
            };
            if let Ok(json) = serde_json::to_string(&resp) {
                let _ = ipc_proxy.send_event(IpcUserEvent {
                    response_json: json,
                });
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
            .unwrap()
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

    let builder = WebViewBuilder::new()
        .with_custom_protocol(app_protocol, handler)
        .with_url("app://localhost/index.html")
        .with_ipc_handler(ipc_handler)
        .with_initialization_script(init_script)
        .with_navigation_handler(navigation_allow)
        .with_devtools(false);

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    let webview = builder.build(&window).expect("build webview");

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let webview = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;
        let vbox = window.default_vbox().expect("vbox");
        builder.build_gtk(vbox).expect("build webview")
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let tao::event::Event::UserEvent(IpcUserEvent { response_json }) = event {
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
            let _ = webview.evaluate_script(&script);
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
