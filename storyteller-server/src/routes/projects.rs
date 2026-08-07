//! Project registry endpoints (`docs/api.md` §3, "Projects").
//!
//! The launcher lists previously-opened projects and switches between them at
//! runtime. The registry is a machine preference held outside any project
//! folder (see [`crate::registry`]); opening a project swaps the server's active
//! project, rebuilds its index and rewatches its folder (see
//! [`AppState::open`](crate::state::AppState::open)).

use std::path::PathBuf;

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::ApiResult;
use crate::registry::ProjectRecord;
use crate::routes::meta::{project_response, ProjectResponse};
use crate::state::SharedState;

#[derive(Serialize)]
pub struct ProjectsResponse {
    /// Known projects, most-recently-opened first.
    items: Vec<ProjectRecord>,
    /// Canonical path of the currently active project.
    active: String,
}

/// `GET /api/v1/projects` — the recent-projects registry plus which is active.
pub async fn list(State(state): State<SharedState>) -> Json<ProjectsResponse> {
    Json(ProjectsResponse {
        items: state.registry_records(),
        active: state.active_root(),
    })
}

/// Body of `POST /api/v1/projects/open`.
#[derive(Deserialize)]
pub struct OpenBody {
    /// Absolute folder path on the server machine. In browser/self-host mode the
    /// launcher supplies it as text (no OS folder picker without a native shell);
    /// a real picker arrives with the Tauri build (M7).
    path: String,
}

/// `POST /api/v1/projects/open` — switch the active project to `path`.
///
/// Returns the same payload as `GET /project` for the freshly opened project. A
/// non-existent or unreadable folder becomes a `404` via the core error mapping.
pub async fn open(
    State(state): State<SharedState>,
    Json(body): Json<OpenBody>,
) -> ApiResult<Json<ProjectResponse>> {
    let active = state.open(&PathBuf::from(body.path))?;
    Ok(Json(project_response(&active)?))
}
