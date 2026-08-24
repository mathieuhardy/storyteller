//! Type schema endpoints (`docs/api.md` §3, "Types").
//!
//! These feed dynamic form generation on the frontend: field names in English,
//! labels in French ([ADR 0010](../../../docs/adr/0010-frontmatter-keys-en-content-fr.md)).

use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;
use storyteller_core::types::{self, FieldSchema, TypeSchema};

use crate::error::{ApiError, ApiResult};
use crate::state::SharedState;

/// A type with its full field list and whether it is currently offered.
#[derive(Serialize)]
pub struct TypeResponse {
    name: &'static str,
    label: &'static str,
    folder: &'static str,
    /// `false` when the type is absent from `enabled_types`: it is not offered
    /// for creation, yet its existing entries stay listed and indexed.
    enabled: bool,
    /// Common fields first, then the type's own — the order a form should use.
    fields: Vec<&'static FieldSchema>,
}

impl TypeResponse {
    fn build(schema: &'static TypeSchema, enabled: bool) -> Self {
        Self {
            name: schema.name,
            label: schema.label,
            folder: schema.folder,
            enabled,
            fields: schema.all_fields().collect(),
        }
    }
}

/// `GET /api/v1/types` — the enabled types.
///
/// The full catalog is reachable with `?all=true`, which a view needs to render
/// entries of a type that has since been disabled. Returns all types as disabled
/// if no project is open (launcher-only mode).
pub async fn list(
    State(state): State<SharedState>,
    axum::extract::Query(query): axum::extract::Query<ListTypesQuery>,
) -> Json<Vec<TypeResponse>> {
    let active = state.current();
    let types = types::catalog()
        .iter()
        .map(|schema| {
            let enabled = active
                .as_ref()
                .map(|a| a.project().config().is_enabled(schema.name))
                .unwrap_or(false);
            (schema, enabled)
        })
        .filter(|(_, enabled)| query.all || *enabled)
        .map(|(schema, enabled)| TypeResponse::build(schema, enabled))
        .collect();
    Json(types)
}

#[derive(serde::Deserialize, Default)]
pub struct ListTypesQuery {
    #[serde(default)]
    all: bool,
}

/// `GET /api/v1/types/{type}` — one type's schema.
pub async fn get(
    State(state): State<SharedState>,
    Path(type_name): Path<String>,
) -> ApiResult<Json<TypeResponse>> {
    let schema = types::type_schema(&type_name)
        .ok_or_else(|| ApiError::not_found(format!("unknown type: {type_name}")))?;
    let enabled = state
        .current()
        .map(|a| a.project().config().is_enabled(schema.name))
        .unwrap_or(false);
    Ok(Json(TypeResponse::build(schema, enabled)))
}

#[derive(serde::Deserialize)]
pub struct SetEnabledBody {
    enabled: bool,
}

/// `PATCH /api/v1/types/{type}` — enable/disable a type for creation
/// (`docs/api.md` §3). Persisted to `.storyteller/config.yaml`; existing entries
/// of that type stay untouched and indexed either way (`docs/data-model.md` §7).
pub async fn set_enabled(
    State(state): State<SharedState>,
    Path(type_name): Path<String>,
    Json(body): Json<SetEnabledBody>,
) -> ApiResult<Json<TypeResponse>> {
    let schema = types::type_schema(&type_name)
        .ok_or_else(|| ApiError::not_found(format!("unknown type: {type_name}")))?;
    let active = state.require_project()?;
    active.set_type_enabled(&type_name, body.enabled)?;
    Ok(Json(TypeResponse::build(schema, body.enabled)))
}
