//! Search endpoint (`docs/api.md` §3 "Search"; ADR 0011 — SQLite + FTS5).

use axum::extract::{RawQuery, State};
use axum::Json;
use storyteller_core::index::{Page, SearchResult};

use crate::error::{ApiError, ApiResult};
use crate::params;
use crate::state::SharedState;

/// `GET /api/v1/search?q=…` — full-text search on title/aliases/tags/body,
/// ranked by relevance; combinable with the usual `type`/`tag`/`<field>`
/// filters and pagination (`docs/api.md` §4). Unlike `/entities`, `q` is
/// required here.
pub async fn search(
    State(state): State<SharedState>,
    RawQuery(query): RawQuery,
) -> ApiResult<Json<Page<SearchResult>>> {
    let params = params::parse(query.as_deref())?;
    if params.list.q.as_deref().unwrap_or("").trim().is_empty() {
        return Err(ApiError::bad_request("`q` is required on /search"));
    }
    let page = state.require_project()?.index().search(&params.list)?;
    Ok(Json(page))
}
