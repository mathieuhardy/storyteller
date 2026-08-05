//! Shared server state.

use std::sync::{Arc, Mutex, MutexGuard};

use storyteller_core::index::{Index, RebuildReport};
use storyteller_core::project::Project;
use storyteller_core::Result;

/// The active project and its index.
///
/// Storyteller is single-user and local ([ADR 0003](../../docs/adr/0003-single-user-local.md)),
/// so one `Mutex` around the index is the right amount of machinery: queries are
/// sub-millisecond and there is no concurrent writer to arbitrate.
pub struct AppState {
    project: Project,
    index: Mutex<Index>,
}

pub type SharedState = Arc<AppState>;

impl AppState {
    /// Opens a project and builds its index cold (first launch, restart).
    pub fn open(project_root: &std::path::Path) -> Result<Self> {
        let project = Project::open(project_root)?;
        let mut index = Index::open(project.root())?;
        let report = index.rebuild_from_project(&project)?;
        tracing::info!(
            root = %project.root().display(),
            entries = report.entries,
            links = report.links,
            stubs = report.stubs,
            duration_ms = report.duration_ms,
            "project opened"
        );
        Ok(Self {
            project,
            index: Mutex::new(index),
        })
    }

    pub fn project(&self) -> &Project {
        &self.project
    }

    /// Locks the index.
    ///
    /// Poisoning is recovered from rather than propagated: the index is a
    /// disposable cache, so a panicked reader cannot have corrupted anything
    /// that matters.
    pub fn index(&self) -> MutexGuard<'_, Index> {
        self.index
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Rebuilds the index from the files.
    pub fn rebuild(&self) -> Result<RebuildReport> {
        self.index().rebuild_from_project(&self.project)
    }
}
