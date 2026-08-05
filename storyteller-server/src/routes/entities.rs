//! Entry endpoints (`docs/api.md` §3, "Entities").

use axum::extract::{Path, RawQuery, State};
use axum::Json;
use storyteller_core::index::Page;
use storyteller_core::links::Backlink;
use storyteller_core::{Entry, EntrySummary};

use crate::error::{ApiError, ApiResult};
use crate::params;
use crate::state::SharedState;

/// `GET /api/v1/entities` — filtered, sorted, paginated list.
///
/// Served from the index: this is exactly what it is a cache for.
pub async fn list(
    State(state): State<SharedState>,
    RawQuery(query): RawQuery,
) -> ApiResult<Json<Page<EntrySummary>>> {
    let params = params::parse(query.as_deref())?;
    let page = state.index().list(&params.list)?;
    Ok(Json(page))
}

/// `GET /api/v1/entities/{slug}` — one full entry.
///
/// The index only supplies the *path*; the content is re-read from the file, so
/// an edit made in Obsidian a second ago is already visible here. On divergence
/// the file wins (golden rule 3).
pub async fn get(
    State(state): State<SharedState>,
    Path(slug): Path<String>,
    RawQuery(query): RawQuery,
) -> ApiResult<Json<Entry>> {
    let params = params::parse(query.as_deref())?;
    let mut entry = read_entry(&state, &slug)?;

    if params.wants_backlinks() {
        entry.backlinks = Some(state.index().backlinks(&slug)?);
    }
    Ok(Json(entry))
}

/// `GET /api/v1/entities/{slug}/backlinks` — incoming links.
pub async fn backlinks(
    State(state): State<SharedState>,
    Path(slug): Path<String>,
) -> ApiResult<Json<Vec<Backlink>>> {
    // Checked first so an unknown slug is a 404 rather than an empty list.
    ensure_exists(&state, &slug)?;
    Ok(Json(state.index().backlinks(&slug)?))
}

fn read_entry(state: &SharedState, slug: &str) -> ApiResult<Entry> {
    let path = path_of(state, slug)?;
    match state.project().read_entry(&path) {
        Ok(entry) => Ok(entry),
        // Indexed but gone from disk: the cache is stale, and the file is right.
        Err(storyteller_core::Error::EntryNotFound(_)) => Err(ApiError::not_found(format!(
            "entry `{slug}` is indexed at {path} but the file is gone; the index needs a rebuild"
        ))),
        Err(other) => Err(other.into()),
    }
}

fn ensure_exists(state: &SharedState, slug: &str) -> ApiResult<()> {
    path_of(state, slug).map(|_| ())
}

fn path_of(state: &SharedState, slug: &str) -> ApiResult<String> {
    state
        .index()
        .path_of(slug)?
        .ok_or_else(|| ApiError::not_found(format!("entry not found: {slug}")))
}
