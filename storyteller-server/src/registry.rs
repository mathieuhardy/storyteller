//! Recently-opened projects registry (`docs/api.md` §3, "Projects").
//!
//! The [launcher](../../docs/ui/layout.md) lists the projects the user has
//! opened before. That list is a **machine preference**, like the UI theme — it
//! is not part of any project and is never written into the markdown folders
//! (golden rule 1: the files stay portable). It therefore lives in the OS config
//! directory, not in `.storyteller/`.
//!
//! Following the tolerance rule (golden rule 5), a missing or corrupt registry
//! file degrades to an empty list rather than failing the server: the worst case
//! is an empty "recent projects" panel, never a crash.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// Distinguishes Storyteller projects from standalone books (plain markdown folders).
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordKind {
    #[default]
    Project,
    Book,
}

/// One known project or book. `path` is the canonical absolute folder path (the same
/// string the API reports as the active project root).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub path: String,
    /// Display name — the folder's base name; the launcher shows it.
    pub name: String,
    /// Entry count at the last open, for the launcher's "N entries" hint.
    /// For books, this is the count of `.md` files.
    pub entries: usize,
    /// When the project was last opened, RFC 3339 / UTC.
    pub last_opened: String,
    /// Whether this is a Storyteller project or a standalone book.
    #[serde(default)]
    pub kind: RecordKind,
}

/// The persisted list of known projects, most-recently-opened first.
///
/// The `path` it was loaded from is remembered so [`save`](Self::save) writes
/// back to the same place. It is `None` when no location could be resolved, in
/// which case the registry keeps working in memory but never persists.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Registry {
    #[serde(default)]
    records: Vec<ProjectRecord>,
    #[serde(skip)]
    path: Option<PathBuf>,
}

impl Registry {
    /// Loads the registry from the OS config location (production path).
    pub fn load() -> Self {
        Self::load_from(registry_path())
    }

    /// Loads the registry from an explicit file, or nowhere when `path` is
    /// `None`. Used by tests and embeddings that must not touch the user's real
    /// config; it also isolates each test's registry from the others.
    ///
    /// Degrades to an empty registry on any problem (missing file, unreadable,
    /// malformed JSON): the launcher then simply shows nothing.
    pub fn load_from(path: Option<PathBuf>) -> Self {
        let mut registry = match &path {
            Some(path) => match std::fs::read_to_string(path) {
                Ok(raw) => serde_json::from_str(&raw).unwrap_or_else(|err| {
                    tracing::warn!(
                        "ignoring unreadable project registry {}: {err}",
                        path.display()
                    );
                    Self::default()
                }),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => Self::default(),
                Err(err) => {
                    tracing::warn!("cannot read project registry {}: {err}", path.display());
                    Self::default()
                }
            },
            None => Self::default(),
        };
        registry.path = path;
        registry
    }

    /// Records that `path` was just opened, then reorders most-recent-first.
    ///
    /// An existing entry for the same path is updated in place; RFC 3339 / UTC
    /// timestamps sort lexicographically in chronological order, so a plain
    /// string sort is enough to keep the list ordered.
    pub fn touch(&mut self, path: &str, name: &str, entries: usize) {
        let now = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_default();
        match self
            .records
            .iter_mut()
            .find(|record| record.path == path && record.kind == RecordKind::Project)
        {
            Some(record) => {
                record.name = name.to_string();
                record.entries = entries;
                record.last_opened = now;
            }
            None => self.records.push(ProjectRecord {
                path: path.to_string(),
                name: name.to_string(),
                entries,
                last_opened: now,
                kind: RecordKind::Project,
            }),
        }
        self.records
            .sort_by(|a, b| b.last_opened.cmp(&a.last_opened));
    }

    /// The known projects, most-recently-opened first.
    pub fn records(&self) -> &[ProjectRecord] {
        &self.records
    }

    /// Returns only Storyteller projects (kind = Project), most-recently-opened first.
    pub fn projects(&self) -> Vec<&ProjectRecord> {
        self.records
            .iter()
            .filter(|r| r.kind == RecordKind::Project)
            .collect()
    }

    /// Returns only standalone books (kind = Book), most-recently-opened first.
    pub fn books(&self) -> Vec<&ProjectRecord> {
        self.records
            .iter()
            .filter(|r| r.kind == RecordKind::Book)
            .collect()
    }

    /// Records that a book folder was just opened, then reorders most-recent-first.
    pub fn touch_book(&mut self, path: &str, name: &str, entries: usize) {
        let now = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_default();
        match self
            .records
            .iter_mut()
            .find(|record| record.path == path && record.kind == RecordKind::Book)
        {
            Some(record) => {
                record.name = name.to_string();
                record.entries = entries;
                record.last_opened = now;
            }
            None => self.records.push(ProjectRecord {
                path: path.to_string(),
                name: name.to_string(),
                entries,
                last_opened: now,
                kind: RecordKind::Book,
            }),
        }
        self.records
            .sort_by(|a, b| b.last_opened.cmp(&a.last_opened));
    }

    /// Removes a record by path. Returns `true` if found and removed.
    pub fn remove(&mut self, path: &str) -> bool {
        let before = self.records.len();
        self.records.retain(|r| r.path != path);
        self.records.len() < before
    }

    /// Persists the registry, creating the config directory if needed.
    ///
    /// Best-effort: a write failure is returned so the caller can log it, but it
    /// must never abort the open it accompanies — losing a "recent" entry is
    /// harmless next to refusing to switch projects.
    pub fn save(&self) -> std::io::Result<()> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }
}

/// Location of the registry file: `$XDG_CONFIG_HOME/storyteller/projects.json`,
/// falling back to `%APPDATA%` on Windows and `$HOME/.config` elsewhere.
///
/// Returns `None` only when no home can be resolved at all, in which case the
/// registry silently disables itself.
fn registry_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("APPDATA").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("storyteller").join("projects.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_registry() -> (tempfile::TempDir, Registry) {
        let dir = tempfile::tempdir().unwrap();
        let registry = Registry::load_from(Some(dir.path().join("projects.json")));
        (dir, registry)
    }

    #[test]
    fn touch_persists_and_reloads() {
        let (dir, mut registry) = temp_registry();
        registry.touch("/novels/a", "a", 12);
        registry.save().unwrap();

        let reloaded = Registry::load_from(Some(dir.path().join("projects.json")));
        assert_eq!(reloaded.records().len(), 1);
        assert_eq!(reloaded.records()[0].path, "/novels/a");
        assert_eq!(reloaded.records()[0].name, "a");
        assert_eq!(reloaded.records()[0].entries, 12);
    }

    #[test]
    fn touch_upserts_the_same_path() {
        let (_dir, mut registry) = temp_registry();
        registry.touch("/novels/a", "a", 1);
        registry.touch("/novels/a", "a-renamed", 5);
        assert_eq!(registry.records().len(), 1, "same path is updated in place");
        assert_eq!(registry.records()[0].name, "a-renamed");
        assert_eq!(registry.records()[0].entries, 5);
    }

    #[test]
    fn most_recently_touched_comes_first() {
        let (_dir, mut registry) = temp_registry();
        registry.touch("/novels/a", "a", 1);
        registry.touch("/novels/b", "b", 1);
        assert_eq!(registry.records()[0].path, "/novels/b");
        // Re-touching a moves it back to the front.
        registry.touch("/novels/a", "a", 1);
        assert_eq!(registry.records()[0].path, "/novels/a");
    }

    #[test]
    fn a_corrupt_file_degrades_to_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("projects.json");
        std::fs::write(&path, "{ this is not json").unwrap();
        let registry = Registry::load_from(Some(path));
        assert!(registry.records().is_empty());
    }

    #[test]
    fn a_pathless_registry_never_persists() {
        let mut registry = Registry::load_from(None);
        registry.touch("/novels/a", "a", 1);
        // No path means save is a no-op that still succeeds — it works in memory.
        assert!(registry.save().is_ok());
        assert_eq!(registry.records().len(), 1);
    }
}
