//! Shared server state.
//!
//! The server keeps **one active project** at a time but can switch to another
//! at runtime (`POST /projects/open`, `docs/api.md` §3). The active project, its
//! parsed [`Snapshot`] and its [`Index`] therefore live behind a swappable
//! [`Active`], while the things that must survive a switch — the SSE broadcast
//! channel and the recent-projects [`Registry`] — stay on [`AppState`].

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};

use storyteller_core::index::{Index, RebuildReport};
use storyteller_core::project::Project;
use storyteller_core::snapshot::{Change, Snapshot};
use storyteller_core::Result;
use tokio::sync::broadcast;

use crate::events::Event;
use crate::registry::{ProjectRecord, Registry};

/// How many change events the broadcast buffer holds. A GUI reloads on reconnect,
/// so a slow client that lags past this simply refreshes — nothing is corrupted.
const EVENT_CAPACITY: usize = 256;

/// One open project: its parsed snapshot and its index.
///
/// Storyteller is single-user and local ([ADR 0003](../../docs/adr/0003-single-user-local.md)),
/// so a `Mutex` around each is the right amount of machinery: queries are
/// sub-millisecond and there is no concurrent writer to arbitrate. When both are
/// needed, `snapshot` is always locked before `index`.
pub struct Active {
    project: Project,
    snapshot: Mutex<Snapshot>,
    index: Mutex<Index>,
}

impl Active {
    /// Opens a project and builds its index cold (first launch, project switch).
    fn open(project_root: &Path) -> Result<Self> {
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
        Ok(Self {
            project,
            snapshot: Mutex::new(snapshot),
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

    /// Like [`reindex`](Self::reindex) but reports what moved and the rebuild
    /// report, so the watcher path can announce `index.rebuilt` over SSE.
    fn reindex_reporting(
        &self,
        changed: &[String],
    ) -> Result<(Vec<Change>, Option<RebuildReport>)> {
        let mut snapshot = self.snapshot();
        let changes = snapshot.refresh(&self.project, changed);
        let report = if changes.is_empty() {
            None
        } else {
            Some(self.index().rebuild_from_snapshot(&snapshot)?)
        };
        Ok((changes, report))
    }

    /// Rebuilds everything from a fresh scan (cold path: forced refresh).
    pub fn rebuild(&self) -> Result<RebuildReport> {
        let mut snapshot = self.snapshot();
        *snapshot = Snapshot::scan(&self.project);
        self.index().rebuild_from_snapshot(&snapshot)
    }

    /// Entry count, treating an index read error as zero: this only feeds the
    /// registry hint and an event count, never a decision.
    fn entry_count(&self) -> usize {
        self.index().entry_count().unwrap_or(0)
    }
}

/// The server's shared state: the swappable active project plus the pieces that
/// outlive a project switch.
pub struct AppState {
    active: RwLock<Arc<Active>>,
    events: broadcast::Sender<Event>,
    registry: Mutex<Registry>,
    watcher: Mutex<Option<crate::watcher::Watcher>>,
}

pub type SharedState = Arc<AppState>;

impl AppState {
    /// Opens the initial project and returns the shared state. The caller starts
    /// the file watcher once it holds the `Arc` (see [`start_watcher`](Self::start_watcher)).
    ///
    /// Uses the OS config location for the recent-projects registry.
    pub fn bootstrap(project_root: &Path) -> Result<SharedState> {
        Self::bootstrap_with_registry(project_root, Registry::load())
    }

    /// Like [`bootstrap`](Self::bootstrap) but with a registry loaded from an
    /// explicit location — for tests and embeddings that must not read or write
    /// the user's real config (see [`Registry::load_from`]).
    pub fn bootstrap_with_registry(
        project_root: &Path,
        mut registry: Registry,
    ) -> Result<SharedState> {
        let active = Arc::new(Active::open(project_root)?);
        let (events, _) = broadcast::channel(EVENT_CAPACITY);
        record_open(&mut registry, &active);
        Ok(Arc::new(Self {
            active: RwLock::new(active),
            events,
            registry: Mutex::new(registry),
            watcher: Mutex::new(None),
        }))
    }

    /// The active project, as a cheap `Arc` clone. Callers hold it for the span
    /// of a request; a concurrent [`open`](Self::open) only affects the *next*
    /// `current()`, never one already taken.
    pub fn current(&self) -> Arc<Active> {
        self.active
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// (Re)starts the file watcher on the active project's folder. Replacing the
    /// stored handle drops the previous watcher, which stops its worker thread.
    /// A watcher that fails to start is not fatal: the API still serves and
    /// reindexes its own writes; only external edits go unnoticed.
    pub fn start_watcher(self: &Arc<Self>) {
        let watcher = match crate::watcher::spawn(self.clone()) {
            Ok(watcher) => Some(watcher),
            Err(err) => {
                tracing::warn!("file watcher disabled: {err:#}");
                None
            }
        };
        *self.watcher.lock().unwrap_or_else(|p| p.into_inner()) = watcher;
    }

    /// Switches the active project (`POST /projects/open`): opens `root`, swaps
    /// it in, rewatches it, records it in the registry, and announces a rebuild.
    pub fn open(self: &Arc<Self>, root: &Path) -> Result<Arc<Active>> {
        let active = Arc::new(Active::open(root)?);
        *self.active.write().unwrap_or_else(|p| p.into_inner()) = active.clone();
        self.start_watcher();
        {
            let mut registry = self.registry.lock().unwrap_or_else(|p| p.into_inner());
            record_open(&mut registry, &active);
        }
        // Opening a project is, from the GUI's angle, a wholesale refresh.
        self.emit(Event::IndexRebuilt {
            reason: "open",
            duration_ms: 0,
            count: active.entry_count(),
        });
        Ok(active)
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
        let active = self.current();
        let (changes, report) = active.reindex_reporting(changed)?;
        if let Some(report) = report {
            self.emit(Event::index_rebuilt(reason, &report));
        }
        Ok(changes)
    }

    /// Snapshot of the recent-projects registry, most-recently-opened first.
    pub fn registry_records(&self) -> Vec<ProjectRecord> {
        self.registry
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .records()
            .to_vec()
    }

    /// Canonical path of the active project's folder.
    pub fn active_root(&self) -> String {
        self.current().project().root().display().to_string()
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

/// Records `active` as just-opened in `registry` and persists it (best-effort).
fn record_open(registry: &mut Registry, active: &Active) {
    let root = active.project().root();
    registry.touch(
        &root.display().to_string(),
        &project_name(root),
        active.entry_count(),
    );
    if let Err(err) = registry.save() {
        tracing::warn!("cannot persist project registry: {err}");
    }
}

/// A project's display name: the folder's base name.
fn project_name(root: &Path) -> String {
    root.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.display().to_string())
}
