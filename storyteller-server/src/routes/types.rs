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
    /// `true` for a type declared in `.storyteller/types.yaml`, `false` for one
    /// of the 11 built-ins ([ADR 0017](../../../docs/adr/0017-custom-types.md)).
    custom: bool,
    /// Common fields first, then the type's own — the order a form should use.
    fields: Vec<&'static FieldSchema>,
}

impl TypeResponse {
    fn build(schema: &'static TypeSchema, enabled: bool, custom: bool) -> Self {
        Self {
            name: schema.name,
            label: schema.label,
            folder: schema.folder,
            enabled,
            custom,
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
    let custom_types = active
        .as_ref()
        .map(|a| a.project().custom_types())
        .unwrap_or(&[]);
    let types = types::all_types(custom_types)
        .map(|schema| {
            let enabled = active
                .as_ref()
                .map(|a| a.project().config().is_enabled(schema.name))
                .unwrap_or(false);
            let custom = types::catalog().iter().all(|t| t.name != schema.name);
            (schema, enabled, custom)
        })
        .filter(|(_, enabled, _)| query.all || *enabled)
        .map(|(schema, enabled, custom)| TypeResponse::build(schema, enabled, custom))
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
    let active = state.current();
    let custom_types = active
        .as_ref()
        .map(|a| a.project().custom_types())
        .unwrap_or(&[]);
    let schema = types::type_schema(&type_name, custom_types)
        .ok_or_else(|| ApiError::not_found(format!("unknown type: {type_name}")))?;
    let enabled = active
        .map(|a| a.project().config().is_enabled(schema.name))
        .unwrap_or(false);
    let custom = types::catalog().iter().all(|t| t.name != schema.name);
    Ok(Json(TypeResponse::build(schema, enabled, custom)))
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
    let active = state.require_project()?;
    let custom_types = active.project().custom_types();
    let schema = types::type_schema(&type_name, custom_types)
        .ok_or_else(|| ApiError::not_found(format!("unknown type: {type_name}")))?;
    let custom = types::catalog().iter().all(|t| t.name != schema.name);
    active.set_type_enabled(&type_name, body.enabled)?;
    Ok(Json(TypeResponse::build(schema, body.enabled, custom)))
}
