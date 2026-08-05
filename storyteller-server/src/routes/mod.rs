//! HTTP routing.
//!
//! M1 exposes the **read** surface of `docs/api.md`. Write endpoints (M2),
//! link/stub endpoints (M3), search (M5) and the SSE stream (which needs the
//! watcher, M2) are deliberately absent rather than stubbed.

mod entities;
mod meta;
mod types;

use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;

use crate::error::ApiError;
use crate::state::SharedState;

/// Version prefix of every route (`docs/api.md` §6).
pub const API_PREFIX: &str = "/api/v1";

/// Builds the application router.
pub fn router(state: SharedState) -> Router {
    let api = Router::new()
        .route("/version", get(meta::version))
        .route("/project", get(meta::project))
        .route("/types", get(types::list))
        .route("/types/{type_name}", get(types::get))
        .route("/entities", get(entities::list))
        .route("/entities/{slug}", get(entities::get))
        .route("/entities/{slug}/backlinks", get(entities::backlinks))
        .with_state(state);

    Router::new()
        .nest(API_PREFIX, api)
        .fallback(not_found)
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
        // M1 is read-only: pointing at the milestone is more useful than a bare 405.
        message: format!("{method} {uri} is not available: the M1 API is read-only"),
    }
}
