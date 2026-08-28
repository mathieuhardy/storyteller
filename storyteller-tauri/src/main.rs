//! Desktop (and future mobile) shell.
//!
//! No native IPC command surface: the webview points at the *same* axum
//! router `storyteller-server` serves over HTTP, bound to an OS-assigned
//! loopback port and run in-process. `lib.rs`'s own doc comment already states
//! the intent — "that is what keeps the server and the Tauri webview behaving
//! identically" — this binary is that intent, made concrete
//! ([ADR 0019](../../docs/adr/0019-tauri-reuses-the-http-router.md)).

use std::path::PathBuf;

use storyteller_server::{router, AppState};
use tauri::{WebviewUrl, WebviewWindowBuilder};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "storyteller_tauri=info,storyteller_server=info,storyteller_core=info".into()
            }),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let project = std::env::var_os("STORYTELLER_PROJECT").map(PathBuf::from);

            // The window needs a real URL before it can open. A small
            // multi-thread runtime binds the port and hands the server off to
            // a background task; it is then leaked (`Box::leak`) rather than
            // shut down, since a desktop process has no "keep the app open,
            // stop the server" state — the whole process exits as one unit
            // when the window closes, taking the in-process server with it.
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("failed to start the embedded server's async runtime");
            let port = runtime.block_on(bootstrap_and_serve(project));
            Box::leak(Box::new(runtime));

            let url = format!("http://127.0.0.1:{port}/")
                .parse()
                .expect("the embedded server's own URL must be valid");
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
                .title("Storyteller")
                .inner_size(1280.0, 800.0)
                .build()
                .expect("failed to open the main window");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the Storyteller app");
}

/// Opens the project (or starts in launcher-only mode), starts the watcher,
/// and serves the API + embedded frontend on an OS-assigned loopback port —
/// exactly what `storyteller-server`'s own `main.rs` does, minus the
/// command-line parsing and the fixed `--bind` address.
async fn bootstrap_and_serve(project: Option<PathBuf>) -> u16 {
    let state = match &project {
        Some(path) => AppState::bootstrap(path)
            .unwrap_or_else(|e| panic!("opening project {}: {e}", path.display())),
        None => {
            tracing::info!("starting in launcher-only mode (no project)");
            AppState::bootstrap_empty()
        }
    };
    state.start_watcher();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind the embedded server to a local port");
    let port = listener
        .local_addr()
        .expect("a bound listener always has a local address")
        .port();
    tracing::info!("embedded server listening on http://127.0.0.1:{port}");

    let app = router(state);
    tokio::spawn(async move {
        if let Err(err) = axum::serve(listener, app).await {
            tracing::error!("embedded server stopped: {err}");
        }
    });

    port
}
