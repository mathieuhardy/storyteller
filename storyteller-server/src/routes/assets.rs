//! Asset endpoints (`docs/api.md` §3, "Assets"). Plain files under `assets/` —
//! no frontmatter, no index entry.

use axum::extract::{Multipart, Path, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use storyteller_core::assets::AssetInfo;

use crate::error::{ApiError, ApiResult};
use crate::events::Event;
use crate::state::SharedState;

/// `GET /api/v1/assets` — every file under `assets/`, sorted by path.
pub async fn list(State(state): State<SharedState>) -> ApiResult<Json<Vec<AssetInfo>>> {
    let active = state.require_project()?;
    let assets = active.project().list_assets();
    Ok(Json(assets))
}

/// `GET /api/v1/assets/{path}` — serves the raw file with a guessed content type.
pub async fn serve(
    State(state): State<SharedState>,
    Path(path): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let active = state.require_project()?;
    let absolute = active.project().asset_absolute_path(&path)?;
    let bytes = std::fs::read(&absolute)
        .map_err(|e| ApiError::internal(format!("cannot read {}: {e}", absolute.display())))?;
    Ok(([(header::CONTENT_TYPE, guess_content_type(&path))], bytes))
}

/// `POST /api/v1/assets` — multipart upload. The first file part found (the one
/// carrying a filename) is saved under `assets/`; a stray non-file field is
/// skipped rather than rejected, since no particular field name is mandated.
pub async fn upload(
    State(state): State<SharedState>,
    mut multipart: Multipart,
) -> ApiResult<impl IntoResponse> {
    let active = state.require_project()?;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::bad_request(format!("invalid multipart body: {e}")))?
    {
        let Some(filename) = field.file_name().map(str::to_string) else {
            continue;
        };
        let bytes = field
            .bytes()
            .await
            .map_err(|e| ApiError::bad_request(format!("cannot read upload: {e}")))?;
        let relative = active.project().save_asset(&filename, &bytes)?;
        state.emit(Event::AssetsChanged {
            path: relative.clone(),
            change: "added",
        });
        return Ok((StatusCode::CREATED, Json(json!({ "path": relative }))));
    }
    Err(ApiError::bad_request("no file part in the upload"))
}

fn guess_content_type(path: &str) -> &'static str {
    let extension = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        _ => "application/octet-stream",
    }
}
