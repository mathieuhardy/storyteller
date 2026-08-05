//! Entry point of the HTTP server.
//!
//! Local-first: it binds the loopback interface by default. There is no auth,
//! no tokens, no permissions ([ADR 0003](../../docs/adr/0003-single-user-local.md)),
//! so exposing it beyond the machine is an explicit choice the operator makes
//! with `--bind`.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Context};
use storyteller_server::{router, AppState};
use tower_http::trace::TraceLayer;

const DEFAULT_BIND: &str = "127.0.0.1:8787";

const USAGE: &str = "\
storyteller-server — serves the Storyteller read API over HTTP

USAGE:
    storyteller-server --project <PATH> [--bind <ADDR>]

OPTIONS:
    -p, --project <PATH>  Project folder to open (env: STORYTELLER_PROJECT)
    -b, --bind <ADDR>     Address to listen on (env: STORYTELLER_BIND)
                          [default: 127.0.0.1:8787]
    -h, --help            Print this help
";

struct Options {
    project: PathBuf,
    bind: SocketAddr,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "storyteller_server=info,storyteller_core=info".into()),
        )
        .init();

    let options = match parse_options()? {
        Some(options) => options,
        // `--help` is a success, not an error.
        None => {
            print!("{USAGE}");
            return Ok(());
        }
    };

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(serve(options))
}

async fn serve(options: Options) -> anyhow::Result<()> {
    let state = Arc::new(
        AppState::open(&options.project)
            .with_context(|| format!("opening project {}", options.project.display()))?,
    );

    let app = router(state).layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind(options.bind)
        .await
        .with_context(|| format!("binding {}", options.bind))?;

    tracing::info!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("serving")?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutting down");
}

/// Parses CLI options, falling back to environment variables.
///
/// Returns `Ok(None)` when help was requested.
fn parse_options() -> anyhow::Result<Option<Options>> {
    let mut project = std::env::var_os("STORYTELLER_PROJECT").map(PathBuf::from);
    let mut bind = std::env::var("STORYTELLER_BIND").ok();

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "-p" | "--project" => {
                project = Some(PathBuf::from(
                    args.next().context("--project needs a path")?,
                ))
            }
            "-b" | "--bind" => bind = Some(args.next().context("--bind needs an address")?),
            other => bail!("unexpected argument `{other}`\n\n{USAGE}"),
        }
    }

    let Some(project) = project else {
        bail!("no project folder given\n\n{USAGE}");
    };
    let bind = bind.unwrap_or_else(|| DEFAULT_BIND.to_string());

    Ok(Some(Options {
        project,
        bind: bind
            .parse()
            .with_context(|| format!("invalid address `{bind}`"))?,
    }))
}
