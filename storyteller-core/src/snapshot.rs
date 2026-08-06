//! An in-memory parsed view of a project, kept alongside the index so that a
//! reindex only re-reads and re-parses the files that actually changed.
//!
//! The index (`docs/architecture.md` §4) can be rebuilt incrementally because
//! parsing markdown — not writing SQLite rows — is the cost that grows with the
//! project. A [`Snapshot`] holds every parsed [`Entry`] together with the file
//! fingerprint ([`FileStat`]: mtime + size + content hash) it was parsed from;
//! [`Snapshot::refresh`] compares those fingerprints against disk and re-parses
//! only the files whose bytes moved.
//!
//! It is still a **cache**, not truth (golden rule 3): dropping it and calling
//! [`Snapshot::scan`] again is always correct, just slower.

use std::collections::BTreeMap;
use std::path::Path;

use crate::index::FileStat;
use crate::model::Entry;
use crate::project::{self, Project};

/// What a [`Snapshot::refresh`] did to one path, for logging and change events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// A file that was absent from the snapshot and now parses.
    Created(String),
    /// A file whose bytes changed since it was last parsed.
    Updated(String),
    /// A file the snapshot held that is no longer a readable entry on disk.
    Deleted(String),
}

impl Change {
    pub fn path(&self) -> &str {
        match self {
            Change::Created(p) | Change::Updated(p) | Change::Deleted(p) => p,
        }
    }
}

/// A parsed project, addressable by entry path.
///
/// Entries are stored **without** the cross-file `duplicate_slug` flag; that
/// diagnostic depends on the whole set and is recomputed by [`Self::indexable`]
/// so that repeated refreshes never accumulate stale duplicates.
#[derive(Debug, Default, Clone)]
pub struct Snapshot {
    /// Raw parsed entries, keyed by their project-relative path.
    entries: BTreeMap<String, Entry>,
    /// Fingerprint each entry was parsed from, same key space as `entries`.
    stats: BTreeMap<String, FileStat>,
}

impl Snapshot {
    /// Parses the whole project cold (first launch, or after dropping the cache).
    pub fn scan(project: &Project) -> Self {
        let mut snapshot = Snapshot::default();
        for path in project.entry_paths() {
            snapshot.parse_into(project, &path);
        }
        snapshot
    }

    /// Re-reads only `changed` paths and updates the snapshot in place.
    ///
    /// Paths outside the entry space (assets, `.storyteller/`, non-markdown) are
    /// ignored. A path whose fingerprint is unchanged is a no-op — an editor that
    /// rewrites a file with identical bytes must not trigger a reindex. The
    /// returned changes are only those that moved the snapshot.
    pub fn refresh(&mut self, project: &Project, changed: &[String]) -> Vec<Change> {
        let mut changes = Vec::new();
        for path in dedup(changed) {
            if !is_entry_path(&path) {
                continue;
            }
            let existed = self.entries.contains_key(&path);
            match project
                .absolute_path(&path)
                .ok()
                .and_then(|abs| FileStat::of(&abs))
            {
                // Readable on disk: reparse only when the bytes actually moved.
                Some(stat) => {
                    if self.stats.get(&path) == Some(&stat) {
                        continue;
                    }
                    self.parse_into(project, &path);
                    changes.push(if existed {
                        Change::Updated(path)
                    } else {
                        Change::Created(path)
                    });
                }
                // Gone (deleted, moved out, or made unreadable).
                None => {
                    if existed {
                        self.entries.remove(&path);
                        self.stats.remove(&path);
                        changes.push(Change::Deleted(path));
                    }
                }
            }
        }
        changes
    }

    /// Entries with the cross-file `duplicate_slug` diagnostic applied, ready to
    /// hand to [`crate::index::Index::rebuild_from_snapshot`].
    pub fn indexable(&self) -> Vec<Entry> {
        let mut entries: Vec<Entry> = self.entries.values().cloned().collect();
        project::flag_duplicate_slugs(&mut entries);
        entries
    }

    /// Fingerprint recorded for a path, if the snapshot holds it.
    pub fn stat(&self, path: &str) -> Option<&FileStat> {
        self.stats.get(path)
    }

    /// Number of entries currently held.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Reads, parses and stores one entry, replacing any previous version.
    ///
    /// A file that cannot be fingerprinted or read is dropped from the snapshot:
    /// the same forgiving behaviour as a cold [`Project::scan`], which skips a
    /// file it cannot read rather than aborting.
    fn parse_into(&mut self, project: &Project, path: &str) {
        let Some(absolute) = project.absolute_path(path).ok() else {
            return;
        };
        let Some(stat) = FileStat::of(&absolute) else {
            self.entries.remove(path);
            self.stats.remove(path);
            return;
        };
        match project.read_entry(path) {
            Ok(entry) => {
                self.entries.insert(path.to_string(), entry);
                self.stats.insert(path.to_string(), stat);
            }
            Err(err) => {
                tracing::warn!("skipping {path}: {err}");
                self.entries.remove(path);
                self.stats.remove(path);
            }
        }
    }
}

/// Whether a project-relative path is where an entry can live: a markdown file
/// outside `assets/` and the technical folder. Mirrors [`Project::entry_paths`],
/// and lets the [watcher](../../storyteller_server/watcher/index.html) pre-filter
/// filesystem events with the exact same rule.
pub fn is_entry_path(path: &str) -> bool {
    if project::is_internal_path(path) {
        return false;
    }
    if path == project::ASSETS_DIR || path.starts_with(&format!("{}/", project::ASSETS_DIR)) {
        return false;
    }
    Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case(project::ENTRY_EXTENSION))
}

/// Stable de-duplication of the changed-path list.
fn dedup(paths: &[String]) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    paths
        .iter()
        .filter(|p| seen.insert((*p).clone()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn write(root: &Path, relative: &str, contents: &str) {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    fn sample() -> (tempfile::TempDir, Project) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(
            root,
            "project.md",
            "---\ntype: project\ntitle: Roman\n---\n",
        );
        write(
            root,
            "characters/aria.md",
            "---\ntype: character\ntitle: Aria\n---\nElle vit à [[Cité de Verre]].\n",
        );
        let project = Project::open(root).unwrap();
        (dir, project)
    }

    fn owned(paths: &[&str]) -> Vec<String> {
        paths.iter().map(|p| p.to_string()).collect()
    }

    #[test]
    fn scan_parses_every_entry() {
        let (_dir, project) = sample();
        let snapshot = Snapshot::scan(&project);
        assert_eq!(snapshot.len(), 2);
        assert!(snapshot.stat("characters/aria.md").is_some());
    }

    #[test]
    fn refresh_ignores_paths_outside_the_entry_space() {
        let (_dir, project) = sample();
        let mut snapshot = Snapshot::scan(&project);
        let changes = snapshot.refresh(
            &project,
            &owned(&[
                "assets/images/map.png",
                ".storyteller/cache.sqlite",
                "characters/portrait.png",
            ]),
        );
        assert!(changes.is_empty(), "{changes:?}");
    }

    #[test]
    fn refresh_is_a_noop_when_bytes_are_unchanged() {
        let (_dir, project) = sample();
        let mut snapshot = Snapshot::scan(&project);
        let changes = snapshot.refresh(&project, &owned(&["characters/aria.md"]));
        assert!(changes.is_empty(), "identical bytes must not reindex");
    }

    #[test]
    fn refresh_detects_create_update_and_delete() {
        let (dir, project) = sample();
        let root = dir.path().to_path_buf();
        let mut snapshot = Snapshot::scan(&project);

        // Create.
        write(
            &root,
            "locations/cite-de-verre.md",
            "---\ntype: location\ntitle: Cité de Verre\n---\n",
        );
        let changes = snapshot.refresh(&project, &owned(&["locations/cite-de-verre.md"]));
        assert_eq!(
            changes,
            vec![Change::Created("locations/cite-de-verre.md".into())]
        );
        assert_eq!(snapshot.len(), 3);

        // Update.
        write(
            &root,
            "characters/aria.md",
            "---\ntype: character\ntitle: Aria Solane\n---\n",
        );
        let changes = snapshot.refresh(&project, &owned(&["characters/aria.md"]));
        assert_eq!(changes, vec![Change::Updated("characters/aria.md".into())]);

        // Delete.
        std::fs::remove_file(root.join("characters/aria.md")).unwrap();
        let changes = snapshot.refresh(&project, &owned(&["characters/aria.md"]));
        assert_eq!(changes, vec![Change::Deleted("characters/aria.md".into())]);
        assert_eq!(snapshot.len(), 2);
    }

    #[test]
    fn indexable_recomputes_duplicate_flags_each_time() {
        use crate::error::codes;
        let (dir, project) = sample();
        let root = dir.path().to_path_buf();
        let mut snapshot = Snapshot::scan(&project);

        // A second file with the same slug as `aria` must flag both — and only
        // once, no matter how many times we refresh and re-derive.
        write(
            &root,
            "notes/aria.md",
            "---\ntype: note\ntitle: Autre\n---\n",
        );
        snapshot.refresh(&project, &owned(&["notes/aria.md"]));

        for _ in 0..3 {
            let entries = snapshot.indexable();
            let aria = entries
                .iter()
                .find(|e| e.path == "characters/aria.md")
                .unwrap();
            let dupes = aria
                .errors
                .iter()
                .filter(|d| d.code == codes::DUPLICATE_SLUG)
                .count();
            assert_eq!(dupes, 1, "duplicate flag must not accumulate");
        }

        // Removing the clash clears the flag on the survivor.
        std::fs::remove_file(root.join("notes/aria.md")).unwrap();
        snapshot.refresh(&project, &owned(&["notes/aria.md"]));
        let entries = snapshot.indexable();
        let aria = entries
            .iter()
            .find(|e| e.path == "characters/aria.md")
            .unwrap();
        assert!(!aria.errors.iter().any(|d| d.code == codes::DUPLICATE_SLUG));
    }
}
