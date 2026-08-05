//! # storyteller-server
//!
//! HTTP wrapper over [`storyteller_core`] (`docs/architecture.md` §2). It holds
//! **no** business logic: it parses requests, calls the core, and serializes the
//! result. That is what keeps the server and the Tauri webview behaving
//! identically.
//!
//! M1–M2 serve the **read** and **write** surface of `docs/api.md`:
//!
//! | Route | Purpose |
//! |---|---|
//! | `GET /api/v1/version` | API, core and schema versions |
//! | `GET /api/v1/project` | `project.md` + `enabled_types`, `schema_version`, stats |
//! | `GET /api/v1/types` | enabled types and their field schemas |
//! | `GET /api/v1/types/{type}` | one type's schema |
//! | `GET /api/v1/entities` | filtered / sorted / paginated list |
//! | `POST /api/v1/entities` | create an entry |
//! | `GET /api/v1/entities/{slug}` | one entry, `?include=backlinks` |
//! | `PATCH /api/v1/entities/{slug}` | non-destructive update |
//! | `DELETE /api/v1/entities/{slug}` | delete an entry |
//! | `POST /api/v1/entities/{slug}/rename` | rename, rewriting breaking links |
//! | `GET /api/v1/entities/{slug}/backlinks` | incoming links |

pub mod error;
pub mod params;
pub mod routes;
pub mod state;

pub use routes::router;
pub use state::{AppState, SharedState};
