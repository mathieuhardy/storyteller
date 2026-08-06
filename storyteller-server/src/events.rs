//! Change events pushed to the GUI over SSE (`docs/api.md` §5).
//!
//! The frontend treats these as cache invalidations: an `entity.*` event names
//! one entry to reload, `index.rebuilt` tells a view to refresh wholesale. They
//! are best-effort — a client that missed some (reconnected, lagged) recovers by
//! reloading, because the index remains the single queryable truth.

use serde_json::{json, Value};
use storyteller_core::index::RebuildReport;

/// One change to broadcast. `name`/`data` map straight onto the SSE
/// `event:`/`data:` fields.
#[derive(Debug, Clone)]
pub enum Event {
    /// A new entry appeared (created through the API or on disk).
    EntityCreated {
        slug: String,
        path: String,
        type_name: String,
    },
    /// An existing entry was modified.
    EntityUpdated {
        slug: String,
        path: String,
        type_name: String,
    },
    /// An entry was deleted or moved out of the project.
    EntityDeleted { slug: String, path: String },
    /// The index was rebuilt; views should refresh. Emitted after an external
    /// change the watcher picked up (`docs/api.md` §5, "index.rebuilt").
    IndexRebuilt {
        reason: &'static str,
        duration_ms: u128,
        count: usize,
    },
}

impl Event {
    /// The SSE `event:` name.
    pub fn name(&self) -> &'static str {
        match self {
            Event::EntityCreated { .. } => "entity.created",
            Event::EntityUpdated { .. } => "entity.updated",
            Event::EntityDeleted { .. } => "entity.deleted",
            Event::IndexRebuilt { .. } => "index.rebuilt",
        }
    }

    /// The SSE `data:` payload.
    pub fn data(&self) -> Value {
        match self {
            Event::EntityCreated {
                slug,
                path,
                type_name,
            }
            | Event::EntityUpdated {
                slug,
                path,
                type_name,
            } => json!({ "slug": slug, "path": path, "type": type_name }),
            Event::EntityDeleted { slug, path } => json!({ "slug": slug, "path": path }),
            Event::IndexRebuilt {
                reason,
                duration_ms,
                count,
            } => json!({ "reason": reason, "duration_ms": duration_ms, "count": count }),
        }
    }

    /// The `index.rebuilt` event for a rebuild triggered by `reason`.
    pub fn index_rebuilt(reason: &'static str, report: &RebuildReport) -> Self {
        Event::IndexRebuilt {
            reason,
            duration_ms: report.duration_ms,
            count: report.entries,
        }
    }
}
