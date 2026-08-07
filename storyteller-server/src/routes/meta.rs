//! Project and version endpoints (`docs/api.md` §3 and §6).

use std::collections::BTreeMap;

use axum::extract::State;
use axum::Json;
use serde::Serialize;
use storyteller_core::error::Diagnostic;
use storyteller_core::Entry;

use crate::error::ApiResult;
use crate::state::{Active, SharedState};

/// HTTP contract version, distinct from the project's `schema_version`.
pub const API_VERSION: &str = "v1";

#[derive(Serialize)]
pub struct VersionResponse {
    api_version: &'static str,
    core_version: &'static str,
    schema_version: u32,
}

/// `GET /api/v1/version` — lets the frontend check compatibility.
pub async fn version(State(state): State<SharedState>) -> Json<VersionResponse> {
    Json(VersionResponse {
        api_version: API_VERSION,
        core_version: storyteller_core::CORE_VERSION,
        schema_version: state.current().project().config().schema_version,
    })
}

#[derive(Serialize)]
pub struct ProjectResponse {
    /// Absolute path of the open project folder.
    root: String,
    /// The `project.md` root entry, `null` when the folder has none — a project
    /// without it is degraded, not invalid.
    entry: Option<Entry>,
    enabled_types: Vec<String>,
    schema_version: u32,
    stats: Stats,
    /// Configuration problems, if any.
    errors: Vec<Diagnostic>,
}

#[derive(Serialize)]
pub struct Stats {
    entries: usize,
    /// Entry count per type, every type on disk included.
    by_type: BTreeMap<String, usize>,
}

/// `GET /api/v1/project` — the root entry plus project metadata.
pub async fn project(State(state): State<SharedState>) -> ApiResult<Json<ProjectResponse>> {
    Ok(Json(project_response(&state.current())?))
}

/// Builds the `GET /project` payload for an active project. Shared with
/// `POST /projects/open`, which returns the same shape for the project it opens.
pub fn project_response(active: &Active) -> ApiResult<ProjectResponse> {
    let project = active.project();
    let config = project.config();

    let entry = match project.read_entry("project.md") {
        Ok(entry) => Some(entry),
        Err(storyteller_core::Error::EntryNotFound(_)) => None,
        Err(other) => return Err(other.into()),
    };

    let index = active.index();
    Ok(ProjectResponse {
        root: project.root().display().to_string(),
        entry,
        enabled_types: config.enabled_types.clone(),
        schema_version: config.schema_version,
        stats: Stats {
            entries: index.entry_count()?,
            by_type: index.counts_by_type()?,
        },
        errors: config.errors.clone(),
    })
}
