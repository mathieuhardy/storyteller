//! Entry endpoints (`docs/api.md` §3, "Entities").

use axum::extract::{Path, RawQuery, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use storyteller_core::index::Page;
use storyteller_core::links::{Backlink, OutgoingLink, Stub};
use storyteller_core::model::Frontmatter;
use storyteller_core::write::{now_rfc3339, slugify};
use storyteller_core::{types, Entry, EntrySummary};

use crate::error::{ApiError, ApiResult};
use crate::events::Event;
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

/// `GET /api/v1/entities/{slug}/links` — outgoing links, each resolved to
/// `resolved` / `stub` / `ambiguous` (`docs/api.md` §3, `docs/linking.md` §3).
pub async fn links(
    State(state): State<SharedState>,
    Path(slug): Path<String>,
) -> ApiResult<Json<Vec<OutgoingLink>>> {
    // An unknown slug is a 404, not an empty list, matching `backlinks`.
    ensure_exists(&state, &slug)?;
    Ok(Json(state.index().outgoing_links(&slug)?))
}

/// `GET /api/v1/stubs` — every unresolved link target in the project, grouped by
/// normalized key, for the "to create" list (`docs/linking.md` §6).
pub async fn stubs(State(state): State<SharedState>) -> ApiResult<Json<Vec<Stub>>> {
    Ok(Json(state.index().stubs()?))
}

/// Body of `POST /api/v1/entities`.
#[derive(Deserialize)]
pub struct CreateBody {
    #[serde(rename = "type")]
    type_name: String,
    title: String,
    /// Extra frontmatter fields; `type`/`title`/`created`/`updated` set here are
    /// ignored in favour of the managed values.
    #[serde(default)]
    frontmatter: Frontmatter,
    #[serde(default)]
    body: String,
}

/// `POST /api/v1/entities` — create an entry (`docs/api.md` §3).
///
/// The type must be known and enabled for creation; the slug (derived from the
/// title) must be free across the whole project. On success the file is written
/// and the index rebuilt, so the entry is immediately listable.
pub async fn create(
    State(state): State<SharedState>,
    Json(body): Json<CreateBody>,
) -> ApiResult<impl IntoResponse> {
    if types::type_schema(&body.type_name).is_none() {
        return Err(ApiError::bad_request(format!(
            "unknown type `{}`; create needs one of the catalog types",
            body.type_name
        )));
    }
    if !state.project().config().is_enabled(&body.type_name) {
        return Err(ApiError::bad_request(format!(
            "type `{}` is disabled for creation in this project",
            body.type_name
        )));
    }
    let slug = slugify(&body.title).ok_or_else(|| {
        ApiError::unprocessable(format!(
            "cannot derive a slug from `{}`: it has no usable characters",
            body.title
        ))
    })?;
    if state.index().path_of(&slug)?.is_some() {
        return Err(ApiError::conflict(format!(
            "an entry already claims the slug `{slug}`"
        )));
    }

    let entry = state.project().create_entry(
        &body.type_name,
        &body.title,
        &body.frontmatter,
        &body.body,
        &now_rfc3339(),
    )?;
    state.reindex(std::slice::from_ref(&entry.path))?;
    state.emit(Event::EntityCreated {
        slug: entry.slug.clone(),
        path: entry.path.clone(),
        type_name: entry.type_name.clone(),
    });
    Ok((StatusCode::CREATED, Json(entry)))
}

/// Body of `PATCH /api/v1/entities/{slug}`.
#[derive(Deserialize)]
pub struct UpdateBody {
    #[serde(default)]
    frontmatter: Frontmatter,
    /// Replaces the body when present; omit to leave it byte-for-byte.
    body: Option<String>,
}

/// `PATCH /api/v1/entities/{slug}` — non-destructive update (`docs/api.md` §3).
pub async fn update(
    State(state): State<SharedState>,
    Path(slug): Path<String>,
    Json(body): Json<UpdateBody>,
) -> ApiResult<Json<Entry>> {
    let path = path_of(&state, &slug)?;
    let entry = state.project().update_entry(
        &path,
        &body.frontmatter,
        body.body.as_deref(),
        &now_rfc3339(),
    )?;
    state.reindex(std::slice::from_ref(&entry.path))?;
    state.emit(Event::EntityUpdated {
        slug: entry.slug.clone(),
        path: entry.path.clone(),
        type_name: entry.type_name.clone(),
    });
    Ok(Json(entry))
}

/// `DELETE /api/v1/entities/{slug}` — remove the file (`docs/api.md` §3).
///
/// Links that targeted it become stubs at the next index pass; no other file is
/// touched.
pub async fn delete(
    State(state): State<SharedState>,
    Path(slug): Path<String>,
) -> ApiResult<StatusCode> {
    let path = path_of(&state, &slug)?;
    state.project().delete_entry(&path)?;
    state.reindex(std::slice::from_ref(&path))?;
    state.emit(Event::EntityDeleted { slug, path });
    Ok(StatusCode::NO_CONTENT)
}

/// Body of `POST /api/v1/entities/{slug}/rename`.
#[derive(Deserialize)]
pub struct RenameBody {
    new_title: Option<String>,
    new_slug: Option<String>,
}

/// `POST /api/v1/entities/{slug}/rename` — rename, rewriting breaking links
/// ([ADR 0012](../../docs/adr/0012-rename-link-rewriting.md)).
pub async fn rename(
    State(state): State<SharedState>,
    Path(slug): Path<String>,
    Json(body): Json<RenameBody>,
) -> ApiResult<Json<Entry>> {
    if body.new_title.is_none() && body.new_slug.is_none() {
        return Err(ApiError::bad_request(
            "rename needs at least one of `new_title` or `new_slug`",
        ));
    }
    let path = path_of(&state, &slug)?;
    let outcome = state.project().rename_entry(
        &path,
        body.new_slug.as_deref(),
        body.new_title.as_deref(),
        &now_rfc3339(),
    )?;

    // The entry file plus every referencing file that was rewritten changed.
    let mut changed = vec![path.clone(), outcome.entry.path.clone()];
    changed.extend(outcome.rewritten_paths.iter().cloned());
    state.reindex(&changed)?;

    // A slug change moves the file: the old identity is gone, a new one appears.
    if outcome.entry.path == path {
        state.emit(Event::EntityUpdated {
            slug: outcome.entry.slug.clone(),
            path: outcome.entry.path.clone(),
            type_name: outcome.entry.type_name.clone(),
        });
    } else {
        state.emit(Event::EntityDeleted {
            slug,
            path: path.clone(),
        });
        state.emit(Event::EntityCreated {
            slug: outcome.entry.slug.clone(),
            path: outcome.entry.path.clone(),
            type_name: outcome.entry.type_name.clone(),
        });
    }
    // Referencing files had their links rewritten.
    for rewritten in &outcome.rewritten_paths {
        if let Some(event) = updated_event(&state, rewritten) {
            state.emit(event);
        }
    }
    Ok(Json(outcome.entry))
}

/// Builds an `entity.updated` event for a path, reading its type from the index.
/// Returns `None` if the path is not indexed (nothing to announce).
fn updated_event(state: &SharedState, path: &str) -> Option<Event> {
    let slug = storyteller_core::project::slug_of(path);
    let summary = state.index().summary(&slug).ok().flatten()?;
    Some(Event::EntityUpdated {
        slug,
        path: path.to_string(),
        type_name: summary.type_name,
    })
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
