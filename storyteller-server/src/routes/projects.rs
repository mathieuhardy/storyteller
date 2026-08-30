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
use crate::registry::{ProjectRecord, RecordKind};
use crate::routes::meta::{project_response, ProjectResponse};
use crate::state::SharedState;

#[derive(Serialize)]
pub struct ProjectsResponse {
    /// Known Storyteller projects, most-recently-opened first.
    items: Vec<ProjectRecord>,
    /// Known standalone books, most-recently-opened first.
    books: Vec<ProjectRecord>,
    /// Canonical path of the currently active project or book.
    active: String,
    /// What kind of item is active: "project", "book", or null.
    active_kind: Option<&'static str>,
}

/// `GET /api/v1/projects` — the recent-projects registry plus which is active.
pub async fn list(State(state): State<SharedState>) -> Json<ProjectsResponse> {
    let active_kind = match state.active_kind() {
        Some(RecordKind::Project) => Some("project"),
        Some(RecordKind::Book) => Some("book"),
        None => None,
    };
    Json(ProjectsResponse {
        items: state.registry_projects(),
        books: state.registry_books(),
        active: state.active_root(),
        active_kind,
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

/// Body of `DELETE /api/v1/projects/recent`.
#[derive(Deserialize)]
pub struct RemoveBody {
    /// Path of the record to remove from the recent list.
    path: String,
}

/// `DELETE /api/v1/projects/recent` — remove a project or book from the recent list.
///
/// This only removes it from the launcher's recent list; it does not delete the
/// folder on disk.
pub async fn remove_recent(
    State(state): State<SharedState>,
    Json(body): Json<RemoveBody>,
) -> ApiResult<axum::http::StatusCode> {
    if state.registry_remove(&body.path) {
        Ok(axum::http::StatusCode::NO_CONTENT)
    } else {
        Err(crate::error::ApiError::not_found("record not in recent list"))
    }
}
