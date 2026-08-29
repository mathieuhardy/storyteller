//! Native folder picker via system dialogs (zenity/kdialog).
//!
//! Bypasses Tauri's dialog plugin which crashes on Linux with webkit2gtk.

use axum::Json;
use serde::Serialize;
use std::process::Command;

use crate::error::ApiError;

#[derive(Serialize)]
pub struct PickFolderResponse {
    pub path: Option<String>,
}

/// Opens a native folder picker dialog using zenity or kdialog.
pub async fn pick_folder() -> Result<Json<PickFolderResponse>, ApiError> {
    let path = tokio::task::spawn_blocking(|| {
        // Try zenity first (GTK-based, most common on Linux)
        if let Ok(output) = Command::new("zenity")
            .args(["--file-selection", "--directory", "--title=Select Project Folder"])
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(path);
                }
            }
            // User cancelled (exit code 1) or empty selection
            if output.status.code() == Some(1) {
                return None;
            }
        }

        // Fall back to kdialog (KDE)
        if let Ok(output) = Command::new("kdialog")
            .args(["--getexistingdirectory", "."])
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(path);
                }
            }
        }

        None
    })
    .await
    .map_err(|e| ApiError::internal(format!("dialog task failed: {e}")))?;

    Ok(Json(PickFolderResponse { path }))
}
