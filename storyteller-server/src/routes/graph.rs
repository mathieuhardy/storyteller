//! Link graph endpoint (`docs/api.md` §3, "Graph"; [ADR 0008](../../../docs/adr/0008-no-graph-in-mvp.md)).

use axum::extract::State;
use axum::Json;
use storyteller_core::links::Graph;

use crate::error::ApiResult;
use crate::state::SharedState;

/// `GET /api/v1/graph` — every entry and every resolved entry-to-entry link,
/// straight from the index.
pub async fn get(State(state): State<SharedState>) -> ApiResult<Json<Graph>> {
    Ok(Json(state.require_project()?.index().graph()?))
}
