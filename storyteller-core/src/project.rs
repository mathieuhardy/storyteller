//! Reading a project folder — the source of truth.
//!
//! Everything here goes straight to disk. No caching, no index: the index is
//! built *from* this module (see [`crate::index`]), never the other way round.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use walkdir::WalkDir;

use crate::config::{ProjectConfig, STORYTELLER_DIR};
use crate::error::{codes, Diagnostic, Error, Result};
use crate::model::{Entry, Frontmatter};
use crate::parse::parse_document;
use crate::types;

/// Folder holding media; it contains no entries.
pub const ASSETS_DIR: &str = "assets";
/// Extension of an entry file.
pub const ENTRY_EXTENSION: &str = "md";

/// An open project: a self-contained markdown folder.
#[derive(Debug, Clone)]
pub struct Project {
    root: PathBuf,
    config: ProjectConfig,
}

impl Project {
    /// Opens a project folder and loads its configuration.
    ///
    /// A plain markdown folder with no `.storyteller/` is a valid project: the
    /// configuration then falls back to its defaults.
    pub fn open(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        if !root.is_dir() {
            return Err(Error::ProjectNotFound(root));
        }
        // Canonicalize once so that later containment checks are meaningful.
        let root = root.canonicalize().map_err(|e| Error::io(&root, e))?;
        let config = ProjectConfig::load(&root);
        Ok(Self { root, config })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn config(&self) -> &ProjectConfig {
        &self.config
    }

    /// Re-reads `.storyteller/config.yaml` from disk.
    pub fn reload_config(&mut self) {
        self.config = ProjectConfig::load(&self.root);
    }

    /// Absolute path of an entry, checked to stay inside the project.
    pub fn absolute_path(&self, relative: &str) -> Result<PathBuf> {
        let relative = Path::new(relative);
        if relative.is_absolute()
            || relative
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::Prefix(_)))
        {
            return Err(Error::PathEscapesProject(relative.to_path_buf()));
        }
        Ok(self.root.join(relative))
    }

    /// Lists entry files, sorted by path so that scans are deterministic.
    ///
    /// Skips hidden folders (including `.storyteller/`) and `assets/`: media
    /// live there, entries never do.
    pub fn entry_paths(&self) -> Vec<String> {
        let mut paths: Vec<String> = WalkDir::new(&self.root)
            .into_iter()
            .filter_entry(|dir_entry| {
                if dir_entry.depth() == 0 {
                    return true;
                }
                let name = dir_entry.file_name().to_string_lossy();
                if dir_entry.file_type().is_dir() {
                    return !name.starts_with('.') && name != ASSETS_DIR;
                }
                true
            })
            .filter_map(|result| match result {
                Ok(dir_entry) => Some(dir_entry),
                Err(err) => {
                    // An unreadable subfolder must not abort the whole scan.
                    tracing::warn!("skipping unreadable path: {err}");
                    None
                }
            })
            .filter(|dir_entry| dir_entry.file_type().is_file())
            .filter(|dir_entry| {
                dir_entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case(ENTRY_EXTENSION))
            })
            .filter_map(|dir_entry| relative_path(&self.root, dir_entry.path()))
            .collect();
        paths.sort();
        paths
    }

    /// Reads and parses one entry, by project-relative path.
    ///
    /// Never fails on bad *content* — only on a missing or unreadable file.
    pub fn read_entry(&self, relative_path: &str) -> Result<Entry> {
        let absolute = self.absolute_path(relative_path)?;
        let bytes = std::fs::read(&absolute).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => Error::EntryNotFound(relative_path.to_string()),
            _ => Error::io(&absolute, err),
        })?;
        Ok(entry_from_bytes(relative_path, &bytes))
    }

    /// Reads every entry of the project.
    ///
    /// Files that cannot be read at all are skipped with a warning rather than
    /// aborting the scan: one broken file must not make a project unopenable.
    pub fn scan(&self) -> Vec<Entry> {
        let mut entries: Vec<Entry> = self
            .entry_paths()
            .into_iter()
            .filter_map(|path| match self.read_entry(&path) {
                Ok(entry) => Some(entry),
                Err(err) => {
                    tracing::warn!("skipping {path}: {err}");
                    None
                }
            })
            .collect();
        flag_duplicate_slugs(&mut entries);
        entries
    }
}

/// Builds an [`Entry`] from raw file bytes.
pub fn entry_from_bytes(relative_path: &str, bytes: &[u8]) -> Entry {
    let slug = slug_of(relative_path);

    let Ok(source) = std::str::from_utf8(bytes) else {
        // Deliberately *not* lossy-decoded: a mangled body must never become
        // the app's view of the file, or a later write would persist the damage.
        return Entry {
            slug,
            path: relative_path.to_string(),
            type_name: crate::model::FALLBACK_TYPE.to_string(),
            frontmatter: Frontmatter::new(),
            body: String::new(),
            backlinks: None,
            errors: vec![Diagnostic::error(
                codes::ENCODING_ERROR,
                "file is not valid UTF-8; fix its encoding to make it readable",
            )],
        };
    };

    let document = parse_document(source);
    let (type_name, mut errors) = types::resolve_type(&document.frontmatter);
    errors.extend(document.diagnostics);
    errors.extend(types::validate(&type_name, &document.frontmatter));

    Entry {
        slug,
        path: relative_path.to_string(),
        type_name,
        frontmatter: document.frontmatter,
        body: document.body,
        backlinks: None,
        errors,
    }
}

/// Slug of an entry = filename without extension (`docs/glossary.md#slug`).
pub fn slug_of(relative_path: &str) -> String {
    let file_name = relative_path.rsplit('/').next().unwrap_or(relative_path);
    file_name
        .strip_suffix(&format!(".{ENTRY_EXTENSION}"))
        .unwrap_or(file_name)
        .to_string()
}

/// Flags entries sharing a slug.
///
/// The slug *is* the identity, so a collision is a genuine conflict: links to
/// it resolve as ambiguous. Both files stay visible — hiding one would be data
/// loss — and each carries a diagnostic naming the other paths.
fn flag_duplicate_slugs(entries: &mut [Entry]) {
    let mut paths_by_slug: HashMap<&str, Vec<String>> = HashMap::new();
    for entry in entries.iter() {
        paths_by_slug
            .entry(entry.slug.as_str())
            .or_default()
            .push(entry.path.clone());
    }
    let duplicates: HashMap<String, Vec<String>> = paths_by_slug
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(slug, paths)| (slug.to_string(), paths))
        .collect();

    for entry in entries.iter_mut() {
        if let Some(paths) = duplicates.get(&entry.slug) {
            let others: Vec<&str> = paths
                .iter()
                .map(String::as_str)
                .filter(|p| *p != entry.path)
                .collect();
            entry.errors.push(Diagnostic::warning(
                codes::DUPLICATE_SLUG,
                format!(
                    "filename `{}` is also used by: {}",
                    entry.slug,
                    others.join(", ")
                ),
            ));
        }
    }
}

/// Project-relative path with `/` separators, or `None` if outside the root.
fn relative_path(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    let parts: Vec<String> = relative
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    Some(parts.join("/"))
}

/// Whether a project-relative path belongs to the technical folder.
pub fn is_internal_path(relative_path: &str) -> bool {
    relative_path == STORYTELLER_DIR || relative_path.starts_with(&format!("{STORYTELLER_DIR}/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_project() -> tempfile::TempDir {
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
        write(
            root,
            "locations/cite-de-verre.md",
            "---\ntype: location\ntitle: Cité de Verre\nlocation_kind: ville\n---\n",
        );
        // Must be ignored by the scan.
        write(root, "assets/images/notes.md", "---\ntype: note\n---\n");
        write(root, ".storyteller/config.yaml", "schema_version: 1\n");
        write(root, ".obsidian/workspace.md", "---\ntype: note\n---\n");
        write(root, "characters/portrait.png", "not markdown");
        dir
    }

    fn write(root: &Path, relative: &str, contents: &str) {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    #[test]
    fn open_rejects_a_non_directory() {
        let err = Project::open("/definitely/not/here").unwrap_err();
        assert!(matches!(err, Error::ProjectNotFound(_)));
    }

    #[test]
    fn entry_paths_skips_assets_hidden_folders_and_non_markdown() {
        let dir = sample_project();
        let project = Project::open(dir.path()).unwrap();
        assert_eq!(
            project.entry_paths(),
            [
                "characters/aria.md",
                "locations/cite-de-verre.md",
                "project.md"
            ]
        );
    }

    #[test]
    fn scan_reads_every_entry_with_its_type() {
        let dir = sample_project();
        let project = Project::open(dir.path()).unwrap();
        let entries = project.scan();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].slug, "aria");
        assert_eq!(entries[0].type_name, "character");
        assert_eq!(entries[0].title(), "Aria");
        assert!(entries.iter().all(|e| e.errors.is_empty()));
    }

    #[test]
    fn read_entry_reports_a_missing_file() {
        let dir = sample_project();
        let project = Project::open(dir.path()).unwrap();
        let err = project.read_entry("characters/nobody.md").unwrap_err();
        assert!(matches!(err, Error::EntryNotFound(_)));
    }

    #[test]
    fn absolute_path_refuses_to_escape_the_project() {
        let dir = sample_project();
        let project = Project::open(dir.path()).unwrap();
        for attempt in ["../secret.md", "characters/../../secret.md", "/etc/passwd"] {
            assert!(
                matches!(
                    project.absolute_path(attempt),
                    Err(Error::PathEscapesProject(_))
                ),
                "{attempt} should be refused"
            );
        }
        assert!(project.absolute_path("characters/aria.md").is_ok());
    }

    #[test]
    fn slug_is_the_filename_without_extension() {
        assert_eq!(slug_of("characters/aria-solane.md"), "aria-solane");
        assert_eq!(slug_of("project.md"), "project");
        assert_eq!(slug_of("weird.name.md"), "weird.name");
    }

    #[test]
    fn entry_without_type_degrades_to_note() {
        let entry = entry_from_bytes("notes/vrac.md", b"---\ntitle: Vrac\n---\ntexte\n");
        assert_eq!(entry.type_name, "note");
        assert_eq!(entry.errors[0].code, codes::MISSING_REQUIRED_FIELD);
    }

    #[test]
    fn entry_with_broken_yaml_is_still_readable() {
        let entry = entry_from_bytes(
            "factions/faction-x.md",
            b"---\ntype: faction\ntitle: Faction X\nleader: \"[[??\n---\n# Faction X\n",
        );
        assert_eq!(entry.type_name, "faction");
        assert_eq!(entry.title(), "Faction X");
        assert_eq!(entry.body, "# Faction X\n");
        assert!(entry
            .errors
            .iter()
            .any(|e| e.code == codes::YAML_PARSE_ERROR));
    }

    #[test]
    fn non_utf8_file_is_flagged_and_left_untouched() {
        let entry = entry_from_bytes("notes/latin1.md", b"---\ntitle: Cit\xe9\n---\n");
        assert_eq!(entry.errors[0].code, codes::ENCODING_ERROR);
        assert!(entry.body.is_empty());
        assert!(entry.frontmatter.is_empty());
    }

    #[test]
    fn duplicate_slugs_are_flagged_on_both_entries() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "characters/aria.md",
            "---\ntype: character\ntitle: A\n---\n",
        );
        write(
            dir.path(),
            "notes/aria.md",
            "---\ntype: note\ntitle: B\n---\n",
        );
        let project = Project::open(dir.path()).unwrap();
        let entries = project.scan();
        assert_eq!(entries.len(), 2, "neither file may be hidden");
        for entry in &entries {
            let diagnostic = entry
                .errors
                .iter()
                .find(|e| e.code == codes::DUPLICATE_SLUG)
                .unwrap_or_else(|| panic!("{} not flagged", entry.path));
            assert!(!diagnostic.message.contains(&entry.path));
        }
    }

    #[test]
    fn internal_paths_are_recognized() {
        assert!(is_internal_path(".storyteller"));
        assert!(is_internal_path(".storyteller/cache.sqlite"));
        assert!(!is_internal_path("characters/aria.md"));
    }
}
