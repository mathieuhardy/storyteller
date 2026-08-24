//! Reading a project folder — the source of truth.
//!
//! Everything here goes straight to disk. No caching, no index: the index is
//! built *from* this module (see [`crate::index`]), never the other way round.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use walkdir::WalkDir;

use crate::config::{ProjectConfig, STORYTELLER_DIR};
use crate::error::{codes, Diagnostic, Error, Result};
use crate::links::{self, LinkOccurrence};
use crate::model::{Entry, Frontmatter, Value};
use crate::normalize::normalize;
use crate::parse::parse_document;
use crate::{types, write};

/// Keys the app owns and callers may not set through create/update: they would
/// otherwise be able to forge `created`/`updated` out of sync with reality.
const MANAGED_KEYS: [&str; 2] = ["created", "updated"];

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

    /// Enables or disables a type for **creation** (`PATCH /types/{type}`,
    /// `docs/api.md` §3). Disabling never hides or deletes existing entries of
    /// that type (`docs/data-model.md` §7) — it only changes what
    /// `POST /entities` and the type picker offer going forward.
    pub fn set_type_enabled(&mut self, type_name: &str, enabled: bool) -> Result<()> {
        if types::type_schema(type_name).is_none() {
            return Err(Error::UnknownType(type_name.to_string()));
        }
        let already = self.config.is_enabled(type_name);
        if enabled && !already {
            self.config.enabled_types.push(type_name.to_string());
        } else if !enabled && already {
            self.config.enabled_types.retain(|t| t != type_name);
        }
        self.config
            .save(&self.root)
            .map_err(|e| Error::io(ProjectConfig::path_in(&self.root), e))
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

    /// Creates an entry: derives its [slug](../glossary.md#slug) from `title`,
    /// writes the file in the type's folder, and stamps `created`/`updated`.
    ///
    /// `frontmatter` carries the caller's extra fields (link fields, tags…); the
    /// app-managed keys and the mandatory `type`/`title` are set here and cannot
    /// be forged through it. Refuses to overwrite an existing file — a fresh slug
    /// is a fresh identity (`docs/data-model.md` §3).
    pub fn create_entry(
        &self,
        type_name: &str,
        title: &str,
        frontmatter: &Frontmatter,
        body: &str,
        now: &str,
    ) -> Result<Entry> {
        let folder = types::folder_for(type_name)
            .ok_or_else(|| Error::UnknownType(type_name.to_string()))?;
        let slug = write::slugify(title).ok_or_else(|| Error::InvalidTitle(title.to_string()))?;
        let relative = if folder.is_empty() {
            format!("{slug}.{ENTRY_EXTENSION}")
        } else {
            format!("{folder}/{slug}.{ENTRY_EXTENSION}")
        };
        let absolute = self.absolute_path(&relative)?;
        if absolute.exists() {
            return Err(Error::EntryExists(slug));
        }

        let mut built = Frontmatter::new();
        built.insert("type".into(), Value::from(type_name));
        built.insert("title".into(), Value::from(title));
        for (key, value) in frontmatter {
            if key == "type" || key == "title" || MANAGED_KEYS.contains(&key.as_str()) {
                continue;
            }
            built.insert(key.clone(), value.clone());
        }
        built.insert("created".into(), Value::from(now));
        built.insert("updated".into(), Value::from(now));

        let contents = write::assemble(&write::serialize_frontmatter(&built), body);
        self.write_file(&absolute, &contents)?;
        Ok(entry_from_bytes(&relative, contents.as_bytes()))
    }

    /// Updates an entry non-destructively: merges the provided frontmatter keys,
    /// optionally replaces the body, and refreshes `updated`.
    ///
    /// Untouched keys, unknown keys, key order and (when `body` is `None`) the
    /// body are preserved byte-for-byte (`docs/data-model.md` §6). `created` is
    /// immutable and `updated` is always set to `now`, whatever the caller sent.
    pub fn update_entry(
        &self,
        relative: &str,
        frontmatter: &Frontmatter,
        body: Option<&str>,
        now: &str,
    ) -> Result<Entry> {
        let absolute = self.absolute_path(relative)?;
        let source = self.read_source(&absolute, relative)?;
        let document = parse_document(&source);

        let mut upserts: Vec<(String, Value)> = frontmatter
            .iter()
            .filter(|(key, _)| !MANAGED_KEYS.contains(&key.as_str()))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        upserts.push(("updated".to_string(), Value::from(now)));

        let raw = document.raw_frontmatter.as_deref().unwrap_or("");
        let yaml = write::edit_frontmatter(raw, &upserts);
        let contents = write::assemble(&yaml, body.unwrap_or(&document.body));
        self.write_file(&absolute, &contents)?;
        Ok(entry_from_bytes(relative, contents.as_bytes()))
    }

    /// Deletes an entry file. Links that targeted it turn into [stubs](../glossary.md)
    /// at the next reindex — the app never rewrites other files to hide the gap.
    pub fn delete_entry(&self, relative: &str) -> Result<()> {
        let absolute = self.absolute_path(relative)?;
        std::fs::remove_file(&absolute).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => Error::EntryNotFound(relative.to_string()),
            _ => Error::io(&absolute, err),
        })
    }

    /// Renames an entry — new slug and/or new title — rewriting the wikilinks
    /// that would otherwise break ([ADR 0012](../adr/0012-rename-link-rewriting.md)).
    ///
    /// Per that ADR: incoming links are rewritten to point at the new filename
    /// and **no alias is kept**. Only links that would stop resolving are
    /// touched; a link already reaching the entry by an untouched alias is left
    /// alone. Every rewrite is a non-destructive edit of the referencing file.
    pub fn rename_entry(
        &self,
        relative: &str,
        new_slug: Option<&str>,
        new_title: Option<&str>,
        now: &str,
    ) -> Result<RenameOutcome> {
        let old_absolute = self.absolute_path(relative)?;
        let source = self.read_source(&old_absolute, relative)?;
        let document = parse_document(&source);
        let old_slug = slug_of(relative);
        let old_title = title_of(&document.frontmatter, &old_slug);

        let new_slug = match new_slug {
            Some(requested) => write::slugify(requested)
                .ok_or_else(|| Error::InvalidTitle(requested.to_string()))?,
            None => old_slug.clone(),
        };
        let slug_changed = new_slug != old_slug;
        let effective_title = new_title.map(str::to_string);

        let new_relative = match relative.rfind('/') {
            Some(cut) => format!("{}/{new_slug}.{ENTRY_EXTENSION}", &relative[..cut]),
            None => format!("{new_slug}.{ENTRY_EXTENSION}"),
        };
        let new_absolute = self.absolute_path(&new_relative)?;

        let entries = self.scan();
        if slug_changed
            && entries
                .iter()
                .any(|entry| entry.path != relative && entry.slug == new_slug)
        {
            return Err(Error::EntryExists(new_slug));
        }

        // Two resolvers describe the project before and after the rename; a link
        // is rewritten only when it resolves to the entry now but would not once
        // the identity changes.
        let before = links::Resolver::new(&entries);
        let mut after = links::Resolver::default();
        for entry in &entries {
            if entry.path == relative {
                let title = effective_title.as_deref().unwrap_or(&old_title);
                after.insert(&new_slug, title, &entry.aliases());
            } else {
                after.insert(&entry.slug, entry.title(), &entry.aliases());
            }
        }

        let rewrite = |occurrence: &LinkOccurrence| -> Option<String> {
            if before.resolve(&occurrence.target_raw).slug.as_deref() != Some(old_slug.as_str()) {
                return None;
            }
            if after.resolve(&occurrence.target_raw).slug.as_deref() == Some(new_slug.as_str()) {
                return None;
            }
            Some(rewritten_inner(&old_slug, &new_slug, occurrence))
        };

        // Referencing files first, so a failure there aborts before the entry
        // itself moves and leaves the project half-renamed.
        let mut rewritten_paths = Vec::new();
        for entry in &entries {
            if entry.path == relative {
                continue;
            }
            let entry_absolute = self.absolute_path(&entry.path)?;
            let Ok(entry_source) = self.read_source(&entry_absolute, &entry.path) else {
                continue;
            };
            if let Some(updated) = rewrite_source(&entry_source, &rewrite) {
                self.write_file(&entry_absolute, &updated)?;
                rewritten_paths.push(entry.path.clone());
            }
        }

        // Finally the entry itself: bump `updated`, set the new title if any,
        // then move the file when the slug changed.
        let mut own_upserts = Vec::new();
        if let Some(title) = &effective_title {
            own_upserts.push(("title".to_string(), Value::from(title.clone())));
        }
        own_upserts.push(("updated".to_string(), Value::from(now)));
        let raw = document.raw_frontmatter.as_deref().unwrap_or("");
        let contents = write::assemble(&write::edit_frontmatter(raw, &own_upserts), &document.body);
        self.write_file(&new_absolute, &contents)?;
        if slug_changed {
            std::fs::remove_file(&old_absolute).map_err(|e| Error::io(&old_absolute, e))?;
        }

        Ok(RenameOutcome {
            entry: entry_from_bytes(&new_relative, contents.as_bytes()),
            rewritten_paths,
        })
    }

    /// Reads a file as UTF-8 text, mapping a missing file to [`Error::EntryNotFound`].
    ///
    /// A non-UTF-8 file is refused rather than lossily decoded: editing it would
    /// risk persisting mangled bytes (golden rule 2).
    fn read_source(&self, absolute: &Path, relative: &str) -> Result<String> {
        let bytes = std::fs::read(absolute).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => Error::EntryNotFound(relative.to_string()),
            _ => Error::io(absolute, err),
        })?;
        String::from_utf8(bytes).map_err(|_| {
            Error::io(
                absolute,
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "file is not valid UTF-8; fix its encoding before editing",
                ),
            )
        })
    }

    /// Writes a file, creating its parent folder (e.g. the type subfolder on the
    /// very first entry of that type).
    fn write_file(&self, absolute: &Path, contents: &str) -> Result<()> {
        if let Some(parent) = absolute.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
        }
        std::fs::write(absolute, contents).map_err(|e| Error::io(absolute, e))
    }
}

/// Outcome of a rename: the entry at its new identity, plus the paths of the
/// referencing files that were rewritten (for the change events, M2 watcher).
#[derive(Debug, Clone)]
pub struct RenameOutcome {
    pub entry: Entry,
    pub rewritten_paths: Vec<String>,
}

/// Displayed title of a frontmatter, falling back to the slug (mirrors
/// [`Entry::title`], usable before an `Entry` exists).
fn title_of(frontmatter: &Frontmatter, slug: &str) -> String {
    frontmatter
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .unwrap_or(slug)
        .to_string()
}

/// New inner text of a rewritten wikilink: the new slug, the original `#anchor`,
/// and a `|display` that keeps prose readable.
///
/// A `|display` is synthesized only when the source text was human prose — i.e.
/// it matched by title or alias (`[[Aria Solane]]` → `[[aria-solane|Aria Solane]]`).
/// A link that was already an identifier (the old or new slug) is rewritten bare
/// (`[[aria]]` → `[[aria-solane]]`), and an explicit display is always kept.
fn rewritten_inner(old_slug: &str, new_slug: &str, occurrence: &LinkOccurrence) -> String {
    let mut inner = new_slug.to_string();
    if let Some(anchor) = &occurrence.anchor {
        inner.push('#');
        inner.push_str(anchor);
    }
    let target_key = normalize(&occurrence.target_raw);
    let display = occurrence.display.clone().or_else(|| {
        (target_key != normalize(new_slug) && target_key != normalize(old_slug))
            .then(|| occurrence.target_raw.clone())
    });
    if let Some(display) = display {
        inner.push('|');
        inner.push_str(&display);
    }
    inner
}

/// Rewrites the links of one referencing file, returning its new bytes when
/// anything changed. Body and frontmatter share the same decision closure.
fn rewrite_source(source: &str, rewrite: &links::Rewrite) -> Option<String> {
    let document = parse_document(source);
    let new_body = links::rewrite_body_links(&document.body, rewrite);

    // A body-only file has no frontmatter to touch and must not gain one.
    let Some(raw) = document.raw_frontmatter.as_deref() else {
        return new_body;
    };

    let mut upserts: Vec<(String, Value)> = Vec::new();
    for (key, value) in &document.frontmatter {
        if let Some(new_value) = links::rewrite_value_links(value, rewrite) {
            upserts.push((key.clone(), new_value));
        }
    }
    if new_body.is_none() && upserts.is_empty() {
        return None;
    }

    let yaml = write::edit_frontmatter(raw, &upserts);
    Some(write::assemble(&yaml, &new_body.unwrap_or(document.body)))
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
            html: None,
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
        html: None,
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
pub(crate) fn flag_duplicate_slugs(entries: &mut [Entry]) {
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

    const NOW: &str = "2026-08-05T12:00:00Z";

    fn read(root: &Path, relative: &str) -> String {
        std::fs::read_to_string(root.join(relative)).unwrap()
    }

    #[test]
    fn create_writes_a_file_in_the_type_folder_with_timestamps() {
        let dir = sample_project();
        let project = Project::open(dir.path()).unwrap();
        let mut fm = Frontmatter::new();
        fm.insert("role".into(), Value::from("mentor"));
        fm.insert(
            "factions".into(),
            serde_json::json!(["[[Ordre du Prisme]]"]),
        );

        let entry = project
            .create_entry("character", "Maître Orlan", &fm, "Un vieux verrier.\n", NOW)
            .unwrap();

        assert_eq!(entry.slug, "maitre-orlan");
        assert_eq!(entry.path, "characters/maitre-orlan.md");
        assert_eq!(entry.created(), Some(NOW));
        assert_eq!(entry.updated(), Some(NOW));
        let on_disk = read(project.root(), "characters/maitre-orlan.md");
        assert!(on_disk.starts_with("---\ntype: character\ntitle: Maître Orlan\n"));
        assert!(on_disk.contains("factions:\n  - \"[[Ordre du Prisme]]\"\n"));
        assert!(on_disk.ends_with("---\nUn vieux verrier.\n"));
    }

    #[test]
    fn create_refuses_a_taken_slug_and_an_unsluggable_title() {
        let dir = sample_project();
        let project = Project::open(dir.path()).unwrap();
        project
            .create_entry("character", "Sylve", &Frontmatter::new(), "", NOW)
            .unwrap();
        assert!(matches!(
            project.create_entry("character", "Sylve", &Frontmatter::new(), "", NOW),
            Err(Error::EntryExists(_))
        ));
        assert!(matches!(
            project.create_entry("character", "  ", &Frontmatter::new(), "", NOW),
            Err(Error::InvalidTitle(_))
        ));
        assert!(matches!(
            project.create_entry("dragon", "Smaug", &Frontmatter::new(), "", NOW),
            Err(Error::UnknownType(_))
        ));
    }

    #[test]
    fn update_merges_keys_and_preserves_everything_else() {
        let dir = sample_project();
        let root = dir.path().to_path_buf();
        write(
            &root,
            "characters/aria.md",
            "---\ntype: character\ntitle: Aria\ntags:\n  - pov\n# hand-written note\nobsidian_id: xyz\nupdated: 2020-01-01T00:00:00Z\n---\nCorps intact avec [[Cité de Verre]].\n",
        );
        let project = Project::open(&root).unwrap();
        let mut fm = Frontmatter::new();
        fm.insert("role".into(), Value::from("protagoniste"));
        // A caller cannot forge `updated`: it is always set to `now`.
        fm.insert("updated".into(), Value::from("1999-01-01T00:00:00Z"));
        project
            .update_entry("characters/aria.md", &fm, None, NOW)
            .unwrap();

        let on_disk = read(&root, "characters/aria.md");
        assert!(
            on_disk.contains("# hand-written note\nobsidian_id: xyz\n"),
            "{on_disk}"
        );
        assert!(on_disk.contains("role: protagoniste\n"));
        assert!(on_disk.contains(&format!("updated: {NOW}\n")));
        assert!(!on_disk.contains("1999"));
        assert!(
            on_disk.ends_with("---\nCorps intact avec [[Cité de Verre]].\n"),
            "body untouched: {on_disk}"
        );
    }

    #[test]
    fn set_type_enabled_persists_and_leaves_existing_entries_alone() {
        let dir = sample_project();
        let root = dir.path().to_path_buf();
        let mut project = Project::open(&root).unwrap();
        assert!(project.config().is_enabled("character"));

        project.set_type_enabled("character", false).unwrap();
        assert!(!project.config().is_enabled("character"));
        // Existing entries of a disabled type are neither hidden nor deleted.
        assert!(root.join("characters/aria.md").exists());
        assert!(project.scan().iter().any(|e| e.type_name == "character"));

        // Persisted: a fresh load sees the same state.
        let reloaded = Project::open(&root).unwrap();
        assert!(!reloaded.config().is_enabled("character"));

        project.set_type_enabled("character", true).unwrap();
        assert!(project.config().is_enabled("character"));
    }

    #[test]
    fn set_type_enabled_rejects_an_unknown_type() {
        let dir = sample_project();
        let mut project = Project::open(dir.path()).unwrap();
        assert!(matches!(
            project.set_type_enabled("dragon", false),
            Err(Error::UnknownType(_))
        ));
    }

    #[test]
    fn delete_removes_the_file() {
        let dir = sample_project();
        let project = Project::open(dir.path()).unwrap();
        project.delete_entry("characters/aria.md").unwrap();
        assert!(!dir.path().join("characters/aria.md").exists());
        assert!(matches!(
            project.delete_entry("characters/aria.md"),
            Err(Error::EntryNotFound(_))
        ));
    }

    #[test]
    fn rename_by_slug_rewrites_only_the_links_that_would_break() {
        let dir = sample_project();
        let root = dir.path().to_path_buf();
        // Slug `aria`, title `La Verrière`: `[[aria]]` reaches the entry by
        // filename, `[[La Verrière]]` by title. Renaming the slug breaks the
        // first (identifier gone) but not the second (the title still resolves).
        write(
            &root,
            "characters/aria.md",
            "---\ntype: character\ntitle: La Verrière\n---\n",
        );
        write(
            &root,
            "chapters/ch1.md",
            "---\ntype: chapter\ntitle: Ch1\npov: \"[[aria]]\"\n---\nVoir [[aria]], [[La Verrière]] et [[Cité de Verre]].\n",
        );
        let project = Project::open(&root).unwrap();

        let outcome = project
            .rename_entry("characters/aria.md", Some("aria-solane"), None, NOW)
            .unwrap();
        assert_eq!(outcome.entry.path, "characters/aria-solane.md");
        assert!(!root.join("characters/aria.md").exists());
        assert!(root.join("characters/aria-solane.md").exists());
        assert_eq!(outcome.rewritten_paths, ["chapters/ch1.md"]);

        let chapter = read(&root, "chapters/ch1.md");
        // Identifier links become the new bare slug; the title link is untouched.
        assert!(chapter.contains("pov: \"[[aria-solane]]\"\n"), "{chapter}");
        assert!(
            chapter.contains("Voir [[aria-solane]], [[La Verrière]] et [[Cité de Verre]]."),
            "{chapter}"
        );
    }

    #[test]
    fn rename_by_title_alone_updates_title_and_rewrites_orphaned_links() {
        let dir = sample_project();
        let root = dir.path().to_path_buf();
        // Slug and title differ, so a title link is not covered by the slug rank.
        write(
            &root,
            "characters/glass.md",
            "---\ntype: character\ntitle: The Glassmaker\n---\n",
        );
        write(
            &root,
            "chapters/ch1.md",
            "---\ntype: chapter\ntitle: Ch1\n---\nVoir [[The Glassmaker]] et [[glass]].\n",
        );
        let project = Project::open(&root).unwrap();
        project
            .rename_entry("characters/glass.md", None, Some("Aria Solane"), NOW)
            .unwrap();

        // Slug unchanged, title updated in place.
        assert!(read(&root, "characters/glass.md").contains("title: Aria Solane\n"));
        // The old-title link would orphan (no alias kept), so it is rewritten to
        // the still-valid slug with readable display; the slug link stays as-is.
        let chapter = read(&root, "chapters/ch1.md");
        assert!(chapter.contains("[[glass|The Glassmaker]]"), "{chapter}");
        assert!(chapter.contains("et [[glass]]."), "{chapter}");
    }
}
