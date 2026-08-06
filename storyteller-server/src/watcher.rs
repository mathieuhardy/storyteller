//! File→index synchronization (`docs/architecture.md` §4, `docs/api.md` §5).
//!
//! Storyteller's truth is the markdown on disk (golden rule 1), so an edit made
//! outside the app — Obsidian, vim, a `git pull`, a mobile sync — must reach the
//! index just like a write through the API does. This watcher monitors the
//! project folder via [`notify`], debounces the bursts an editor or a checkout
//! produces, and hands the changed paths to
//! [`AppState::reindex_and_announce`](crate::state::AppState::reindex_and_announce).
//!
//! Only entry files matter here: changes under `.storyteller/` (our own cache
//! writes included) and `assets/` are filtered out, which is also what keeps the
//! index's own writes from waking the watcher in a loop.

use std::path::{Component, Path, PathBuf};
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;

use notify_debouncer_full::notify::{RecommendedWatcher, RecursiveMode, Watcher as _};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use storyteller_core::snapshot;

use crate::state::SharedState;

/// How long to coalesce a burst of filesystem events before reindexing. Long
/// enough to fold a "save" (rename + write + chmod) into one pass, short enough
/// that an external edit feels live in the GUI.
const DEBOUNCE: Duration = Duration::from_millis(400);

/// Reason string carried by the `index.rebuilt` event this watcher emits.
const REASON: &str = "watch";

/// A running watcher. Dropping it stops watching and lets the worker thread end.
pub struct Watcher {
    // Order matters: the debouncer owns the event sender, so dropping it first
    // disconnects the channel and unblocks the worker's receive loop.
    _debouncer: Debouncer<RecommendedWatcher, FileIdMap>,
    worker: Option<JoinHandle<()>>,
}

impl Drop for Watcher {
    fn drop(&mut self) {
        // `_debouncer` is dropped after this body, so take the handle and let the
        // thread wind down on its own; joining here would block on the still-open
        // channel. The process outlives this in practice (shutdown), so a detached
        // join is fine.
        self.worker.take();
    }
}

/// Starts watching `state`'s project folder. Failing to set up the watcher is
/// returned to the caller, which decides whether to serve without live sync.
pub fn spawn(state: SharedState) -> anyhow::Result<Watcher> {
    let root = state.project().root().to_path_buf();

    let (tx, rx) = mpsc::channel::<DebounceEventResult>();
    let mut debouncer = new_debouncer(DEBOUNCE, None, tx)?;
    debouncer.watcher().watch(&root, RecursiveMode::Recursive)?;
    debouncer.cache().add_root(&root, RecursiveMode::Recursive);

    let worker = std::thread::Builder::new()
        .name("storyteller-watcher".into())
        .spawn(move || run(state, root, rx))?;

    tracing::info!("watching project folder for external changes");
    Ok(Watcher {
        _debouncer: debouncer,
        worker: Some(worker),
    })
}

/// Receive loop: ends when the debouncer (and its sender) is dropped.
fn run(state: SharedState, root: PathBuf, rx: mpsc::Receiver<DebounceEventResult>) {
    for result in rx {
        let events = match result {
            Ok(events) => events,
            Err(errors) => {
                tracing::warn!(?errors, "watch error");
                continue;
            }
        };

        let changed = changed_entry_paths(&root, &events);
        if changed.is_empty() {
            continue;
        }

        match state.reindex_and_announce(&changed, REASON) {
            Ok(changes) if !changes.is_empty() => {
                tracing::info!(count = changes.len(), "reindexed external changes");
            }
            // Every candidate path turned out unchanged (e.g. the echo of our own
            // write): nothing to announce.
            Ok(_) => {}
            Err(err) => tracing::error!("reindex after external change failed: {err}"),
        }
    }
    tracing::debug!("watcher worker stopped");
}

/// Project-relative entry paths touched by a batch of debounced events.
///
/// Kept forgiving: a path we cannot relativize, or one outside the entry space,
/// is simply dropped. Final change detection (and duplicate handling) is the
/// snapshot's job — this only decides what is worth handing to it.
fn changed_entry_paths(
    root: &Path,
    events: &[notify_debouncer_full::DebouncedEvent],
) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    for event in events {
        for absolute in &event.paths {
            if let Some(relative) = relativize(root, absolute) {
                // Same rule the snapshot enforces, applied early so `.storyteller/`
                // writes (our own index) never reach `reindex` — no feedback loop.
                if snapshot::is_entry_path(&relative) && !paths.contains(&relative) {
                    paths.push(relative);
                }
            }
        }
    }
    paths
}

/// Absolute path → project-relative path with `/` separators, or `None` if it
/// escapes the root.
fn relativize(root: &Path, absolute: &Path) -> Option<String> {
    let relative = absolute.strip_prefix(root).ok()?;
    let mut parts = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            // A rename can surface odd components; anything non-plain means we do
            // not trust the path enough to reindex from it.
            _ => return None,
        }
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}
