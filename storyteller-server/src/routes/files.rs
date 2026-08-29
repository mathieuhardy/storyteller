//! Raw file endpoints for Book Mode — bypasses the Entry system for direct
//! markdown editing. Security: paths are validated to stay within the project.

use std::path::{Path, PathBuf};

use axum::extract::{Path as AxumPath, RawQuery, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::state::SharedState;

/// A file or directory entry in the listing.
#[derive(Debug, Serialize)]
pub struct FileEntry {
    /// Relative path from project root.
    pub path: String,
    /// Base name of the file or directory.
    pub name: String,
    /// `true` if this is a directory.
    pub is_dir: bool,
}

/// Query parameters for `GET /files`.
#[derive(Debug, Deserialize, Default)]
pub struct ListQuery {
    /// Directory path relative to project root (default: root).
    path: Option<String>,
}

/// `GET /api/v1/files` — list files and directories.
///
/// Returns the contents of a directory, sorted with directories first, then
/// files, both alphabetically. Only `.md` files are included for files.
pub async fn list(
    State(state): State<SharedState>,
    RawQuery(query): RawQuery,
) -> ApiResult<Json<Vec<FileEntry>>> {
    let params = parse_list_query(query.as_deref());

    let active = state.require_project()?;
    let root = PathBuf::from(active.project().root());
    let target_path = match &params.path {
        Some(p) if !p.is_empty() => root.join(p),
        _ => root.clone(),
    };

    // Validate the path stays within the project.
    validate_path(&root, &target_path)?;

    if !target_path.is_dir() {
        return Err(ApiError::not_found("directory not found"));
    }

    let mut entries = Vec::new();
    let read_dir = std::fs::read_dir(&target_path).map_err(|e| {
        ApiError::internal(format!("failed to read directory: {e}"))
    })?;

    for entry in read_dir.flatten() {
        let file_type = entry.file_type().ok();
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files/directories.
        if file_name.starts_with('.') {
            continue;
        }

        let is_dir = file_type.map(|ft| ft.is_dir()).unwrap_or(false);
        let is_file = file_type.map(|ft| ft.is_file()).unwrap_or(false);

        // For files, only include .md files.
        if is_file && !file_name.ends_with(".md") {
            continue;
        }

        let relative_path = entry
            .path()
            .strip_prefix(&root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file_name.clone());

        if is_dir || is_file {
            entries.push(FileEntry {
                path: relative_path,
                name: file_name,
                is_dir,
            });
        }
    }

    // Sort: directories first, then files, both alphabetically.
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(Json(entries))
}

/// Response for reading a file.
#[derive(Debug, Serialize)]
pub struct FileContent {
    pub path: String,
    pub content: String,
}

/// `GET /api/v1/files/{*path}` — read raw file content.
pub async fn read(
    State(state): State<SharedState>,
    AxumPath(file_path): AxumPath<String>,
) -> ApiResult<Json<FileContent>> {
    let active = state.require_project()?;
    let root = PathBuf::from(active.project().root());
    let target_path = root.join(&file_path);

    validate_path(&root, &target_path)?;

    if !target_path.is_file() {
        return Err(ApiError::not_found(format!("file not found: {file_path}")));
    }

    let content = std::fs::read_to_string(&target_path).map_err(|e| {
        ApiError::internal(format!("failed to read file: {e}"))
    })?;

    Ok(Json(FileContent {
        path: file_path,
        content,
    }))
}

/// Body for `PUT /api/v1/files/{*path}`.
#[derive(Debug, Deserialize)]
pub struct WriteBody {
    pub content: String,
}

/// `PUT /api/v1/files/{*path}` — write raw file content.
///
/// Restricted to `.md` files for security.
pub async fn write(
    State(state): State<SharedState>,
    AxumPath(file_path): AxumPath<String>,
    Json(body): Json<WriteBody>,
) -> ApiResult<StatusCode> {
    // Only allow .md files for writing.
    if !file_path.ends_with(".md") {
        return Err(ApiError::bad_request(
            "only .md files can be written via this endpoint",
        ));
    }

    let active = state.require_project()?;
    let root = PathBuf::from(active.project().root());
    let target_path = root.join(&file_path);

    validate_path(&root, &target_path)?;

    // Ensure parent directory exists.
    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            return Err(ApiError::not_found("parent directory does not exist"));
        }
    }

    std::fs::write(&target_path, &body.content).map_err(|e| {
        ApiError::internal(format!("failed to write file: {e}"))
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Parses the query string for the list endpoint.
fn parse_list_query(raw: Option<&str>) -> ListQuery {
    let pairs = form_urlencoded::parse(raw.unwrap_or_default().as_bytes());
    let mut params = ListQuery::default();
    for (key, value) in pairs {
        if key == "path" {
            params.path = Some(value.into_owned());
        }
    }
    params
}

/// Validates that a path does not escape the project root via `..` or symlinks.
fn validate_path(root: &Path, target: &Path) -> ApiResult<()> {
    // Canonicalize both paths if they exist, otherwise check for .. segments.
    let canonical_root = root.canonicalize().map_err(|_| {
        ApiError::internal("failed to resolve project root")
    })?;

    // For target, we need to handle the case where it might not exist yet.
    // Check the existing prefix first.
    let mut check_path = target.to_path_buf();
    while !check_path.exists() {
        if let Some(parent) = check_path.parent() {
            check_path = parent.to_path_buf();
        } else {
            break;
        }
    }

    if check_path.exists() {
        let canonical_target = check_path.canonicalize().map_err(|_| {
            ApiError::bad_request("invalid path")
        })?;

        if !canonical_target.starts_with(&canonical_root) {
            return Err(ApiError::bad_request("path escapes project root"));
        }
    }

    // Also check for .. in the path string as a safety measure.
    let path_str = target.to_string_lossy();
    if path_str.contains("..") {
        return Err(ApiError::bad_request("path traversal not allowed"));
    }

    Ok(())
}
