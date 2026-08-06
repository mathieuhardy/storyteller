//! Shared server state.

use std::sync::{Arc, Mutex, MutexGuard};

use storyteller_core::index::{Index, RebuildReport};
use storyteller_core::project::Project;
use storyteller_core::snapshot::{Change, Snapshot};
use storyteller_core::Result;
use tokio::sync::broadcast;

use crate::events::Event;

/// How many change events the broadcast buffer holds. A GUI reloads on reconnect,
/// so a slow client that lags past this simply refreshes — nothing is corrupted.
const EVENT_CAPACITY: usize = 256;

/// The active project, its index, and the parsed snapshot that lets a reindex
/// touch only the files that changed.
///
/// Storyteller is single-user and local ([ADR 0003](../../docs/adr/0003-single-user-local.md)),
/// so a `Mutex` around each is the right amount of machinery: queries are
/// sub-millisecond and there is no concurrent writer to arbitrate. When both
/// are needed, `snapshot` is always locked before `index`.
pub struct AppState {
    project: Project,
    snapshot: Mutex<Snapshot>,
    index: Mutex<Index>,
    events: broadcast::Sender<Event>,
}

pub type SharedState = Arc<AppState>;

impl AppState {
    /// Opens a project and builds its index cold (first launch, restart).
    pub fn open(project_root: &std::path::Path) -> Result<Self> {
        let project = Project::open(project_root)?;
        let snapshot = Snapshot::scan(&project);
        let mut index = Index::open(project.root())?;
        let report = index.rebuild_from_snapshot(&snapshot)?;
        tracing::info!(
            root = %project.root().display(),
            entries = report.entries,
            links = report.links,
            stubs = report.stubs,
            duration_ms = report.duration_ms,
            "project opened"
        );
        let (events, _) = broadcast::channel(EVENT_CAPACITY);
        Ok(Self {
            project,
            snapshot: Mutex::new(snapshot),
            index: Mutex::new(index),
            events,
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

    fn snapshot(&self) -> MutexGuard<'_, Snapshot> {
        self.snapshot
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Incrementally reindexes the given paths after a write made *through* the
    /// API. The snapshot re-parses only what changed; the index is rebuilt only
    /// if something actually moved. The caller emits the precise `entity.*` event.
    pub fn reindex(&self, changed: &[String]) -> Result<()> {
        let mut snapshot = self.snapshot();
        let changes = snapshot.refresh(&self.project, changed);
        if !changes.is_empty() {
            self.index().rebuild_from_snapshot(&snapshot)?;
        }
        Ok(())
    }

    /// Reindexes external changes picked up by the [watcher](crate::watcher) and,
    /// when anything moved, announces a rebuild over SSE (`index.rebuilt`).
    ///
    /// Rebuilding only on a real change is what breaks the feedback loop: our own
    /// index write touches `.storyteller/`, which the watcher also sees, but that
    /// path carries no entry, so `refresh` reports no change and we stop.
    pub fn reindex_and_announce(
        &self,
        changed: &[String],
        reason: &'static str,
    ) -> Result<Vec<Change>> {
        let mut snapshot = self.snapshot();
        let changes = snapshot.refresh(&self.project, changed);
        if !changes.is_empty() {
            let report = self.index().rebuild_from_snapshot(&snapshot)?;
            self.emit(Event::index_rebuilt(reason, &report));
        }
        Ok(changes)
    }

    /// Rebuilds everything from a fresh scan (cold path: forced refresh).
    pub fn rebuild(&self) -> Result<RebuildReport> {
        let mut snapshot = self.snapshot();
        *snapshot = Snapshot::scan(&self.project);
        self.index().rebuild_from_snapshot(&snapshot)
    }

    /// Subscribes to the change-event stream (SSE, `docs/api.md` §5).
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.events.subscribe()
    }

    /// Broadcasts a change event. A send with no live subscriber is not an
    /// error: events are best-effort notifications, never a write path.
    pub fn emit(&self, event: Event) {
        let _ = self.events.send(event);
    }
}
