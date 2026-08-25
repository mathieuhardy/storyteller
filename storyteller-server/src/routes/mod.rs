//! HTTP routing.
//!
//! M1–M3 expose the **read**, **write** and **link** surface of `docs/api.md`;
//! the M2 watcher adds the SSE change stream (`/events`). Search (M5) is
//! deliberately absent rather than stubbed.

mod assets;
mod entities;
mod events;
mod graph;
pub(crate) mod meta;
mod projects;
mod search;
mod types;

use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Router;

use crate::error::ApiError;
use crate::frontend;
use crate::state::SharedState;

/// Version prefix of every route (`docs/api.md` §6).
pub const API_PREFIX: &str = "/api/v1";

/// Builds the application router.
pub fn router(state: SharedState) -> Router {
    let api = Router::new()
        .route("/version", get(meta::version))
        .route("/project", get(meta::project))
        .route("/types", get(types::list))
        .route("/types/{type_name}", get(types::get).patch(types::set_enabled))
        .route("/entities", get(entities::list).post(entities::create))
        .route(
            "/entities/{slug}",
            get(entities::get)
                .patch(entities::update)
                .delete(entities::delete),
        )
        .route("/entities/{slug}/rename", post(entities::rename))
        .route("/entities/{slug}/backlinks", get(entities::backlinks))
        .route("/entities/{slug}/links", get(entities::links))
        .route("/stubs", get(entities::stubs))
        .route("/graph", get(graph::get))
        .route("/search", get(search::search))
        .route("/assets", get(assets::list).post(assets::upload))
        .route("/assets/{*path}", get(assets::serve))
        .route("/projects", get(projects::list))
        .route("/projects/open", post(projects::open))
        .route("/events", get(events::stream))
        // Its own fallback: an unmatched path *under* `/api/v1` is a JSON
        // 404, never the frontend shell below it.
        .fallback(not_found)
        .with_state(state);

    Router::new()
        .nest(API_PREFIX, api)
        // Anything not under `/api/v1` is the built SvelteKit SPA (M6,
        // self-host/Docker/Nix) — a no-op (clean 404s) when nothing is
        // embedded, e.g. in `cargo test`/local Rust-only dev.
        .fallback(frontend::serve)
        .method_not_allowed_fallback(method_not_allowed)
}

/// Unknown routes answer with the normalized error body, not an empty 404.
async fn not_found(uri: axum::http::Uri) -> ApiError {
    ApiError::not_found(format!("no such endpoint: {uri}"))
}

async fn method_not_allowed(method: axum::http::Method, uri: axum::http::Uri) -> ApiError {
    ApiError {
        status: StatusCode::METHOD_NOT_ALLOWED,
        code: "method_not_allowed",
        message: format!("{method} is not allowed on {uri}"),
    }
}
