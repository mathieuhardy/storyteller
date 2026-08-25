//! Error types.
//!
//! Two levels coexist, and the distinction matters:
//!
//! * [`Error`] — the operation could not be performed at all (I/O failure,
//!   corrupted index, project not found).
//! * [`Diagnostic`] — the operation succeeded in a *degraded* way for one entry
//!   (invalid YAML, unknown type, bad field value). Per the golden rule
//!   "tolerance for imperfect data", a bad entry never fails a whole listing;
//!   it is returned with diagnostics attached. See `docs/api.md` §6.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A fatal error: the requested operation could not be completed.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("project not found or not a directory: {0}")]
    ProjectNotFound(PathBuf),

    #[error("entry not found: {0}")]
    EntryNotFound(String),

    #[error("an entry already claims this identity: {0}")]
    EntryExists(String),

    #[error("cannot derive a slug from title: {0}")]
    InvalidTitle(String),

    #[error("unknown type: {0}")]
    UnknownType(String),

    #[error("path escapes the project root: {0}")]
    PathEscapesProject(PathBuf),

    #[error("no asset at this path: {0}")]
    AssetNotFound(String),

    #[error("an asset already exists at this path: {0}")]
    AssetExists(String),

    #[error("i/o error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("index error: {0}")]
    Index(#[from] rusqlite::Error),

    #[error("config error in {path}: {message}")]
    Config { path: PathBuf, message: String },
}

impl Error {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io {
            path: path.into(),
            source,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Severity of a per-entry [`Diagnostic`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// The entry is degraded: some data could not be interpreted.
    Error,
    /// The entry is usable but something is off (missing or odd field).
    Warning,
}

/// Canonical diagnostic codes (`docs/api.md` §6, "Error Tolerance").
pub mod codes {
    pub const YAML_PARSE_ERROR: &str = "yaml_parse_error";
    pub const UNKNOWN_TYPE: &str = "unknown_type";
    pub const MISSING_REQUIRED_FIELD: &str = "missing_required_field";
    pub const INVALID_FIELD_VALUE: &str = "invalid_field_value";
    /// Several files share the same filename, hence the same identity.
    pub const DUPLICATE_SLUG: &str = "duplicate_slug";
    /// The file is not valid UTF-8; its content cannot be interpreted.
    pub const ENCODING_ERROR: &str = "encoding_error";
    /// A custom type in `.storyteller/types.yaml` is malformed (bad name,
    /// folder/name collision, enum without values…) and was dropped.
    pub const INVALID_TYPE_DEFINITION: &str = "invalid_type_definition";
}

/// A non-fatal, per-entry problem. Surfaced in the entry's `errors` array so
/// the user sees *where* to fix things, without ever blocking a read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// One of [`codes`].
    pub code: String,
    pub message: String,
    /// Frontmatter field concerned, `None` when the problem is document-wide.
    pub field: Option<String>,
    pub severity: Severity,
}

impl Diagnostic {
    pub fn error(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
            field: None,
            severity: Severity::Error,
        }
    }

    pub fn warning(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
            field: None,
            severity: Severity::Warning,
        }
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }
}
