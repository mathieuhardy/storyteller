//! Normalized API errors (`docs/api.md` §6).
//!
//! Careful not to confuse the two error levels: a
//! [`storyteller_core::Diagnostic`] rides *inside* a `200` response, on the
//! entry it concerns. What lives here is the other kind — the request itself
//! could not be served.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use storyteller_core::Error as CoreError;

/// An error that becomes an HTTP status plus a normalized body.
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
}

impl ApiError {
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            message: message.into(),
        }
    }

    /// No project is currently open. The client should use `POST /projects/open`
    /// to open one first.
    pub fn no_project() -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "no_project",
            message: "no project open — use POST /projects/open first".into(),
        }
    }

    /// No book is currently open. The client should use `POST /books/open`
    /// to open one first.
    pub fn no_book() -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "no_book",
            message: "no book open — use POST /books/open first".into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "bad_request",
            message: message.into(),
        }
    }

    /// The request collides with the current state: a slug already taken on
    /// create, a rename target already in use (`docs/api.md` §6).
    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: "conflict",
            message: message.into(),
        }
    }

    /// The request is well-formed but its content is refused (a title with no
    /// sluggable characters, unserializable frontmatter).
    pub fn unprocessable(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "unprocessable",
            message: message.into(),
        }
    }

    /// Documented in the API contract but not built yet: the client asked for a
    /// later milestone's feature. Saying so beats silently ignoring it.
    pub fn not_implemented(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_IMPLEMENTED,
            code: "not_implemented",
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if self.status.is_server_error() {
            tracing::error!(code = self.code, message = %self.message, "request failed");
        }
        let body = json!({
            "error": {
                "code": self.code,
                "message": self.message,
                "details": {},
            }
        });
        (self.status, Json(body)).into_response()
    }
}

impl From<CoreError> for ApiError {
    fn from(error: CoreError) -> Self {
        match error {
            CoreError::EntryNotFound(what) => {
                ApiError::not_found(format!("entry not found: {what}"))
            }
            CoreError::EntryExists(slug) => {
                ApiError::conflict(format!("an entry already claims the slug `{slug}`"))
            }
            CoreError::InvalidTitle(title) => ApiError::unprocessable(format!(
                "cannot derive a slug from `{title}`: it has no usable characters"
            )),
            CoreError::UnknownType(name) => ApiError::not_found(format!("unknown type: {name}")),
            CoreError::AssetNotFound(path) => ApiError::not_found(format!("no asset at: {path}")),
            CoreError::AssetExists(path) => {
                ApiError::conflict(format!("an asset already exists at: {path}"))
            }
            CoreError::ProjectNotFound(path) => {
                ApiError::not_found(format!("project not found: {}", path.display()))
            }
            // A traversal attempt is a malformed request, not a missing resource.
            CoreError::PathEscapesProject(path) => {
                ApiError::bad_request(format!("path escapes the project: {}", path.display()))
            }
            other => ApiError::internal(other.to_string()),
        }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
