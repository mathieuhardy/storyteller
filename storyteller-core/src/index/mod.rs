//! The index: a rebuildable SQLite cache over the project's entries.
//!
//! Golden rule 3 governs this module: **the index is never authoritative**.
//! It exists to make views, filters and backlinks fast. Deleting
//! `.storyteller/cache.sqlite` must always be a safe, consequence-free act.
//!
//! ```no_run
//! use storyteller_core::{index::Index, project::Project};
//!
//! let project = Project::open("/path/to/my-novel")?;
//! let mut index = Index::open(project.root())?;
//! index.rebuild(&project.scan())?;
//! # Ok::<(), storyteller_core::error::Error>(())
//! ```

mod query;
mod schema;

pub use query::{ListQuery, Page, SortSpec, DEFAULT_PER_PAGE, MAX_PER_PAGE};
pub use schema::INDEX_SCHEMA_VERSION;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::config::STORYTELLER_DIR;
use crate::error::{Diagnostic, Error, Result};
use crate::links::{
    self, Backlink, LinkContext, LinkOccurrence, OutgoingLink, Resolution, Resolver, Stub,
};
use crate::model::{Entry, EntrySummary, Value};
use crate::normalize::normalize;

/// Index file name, inside `.storyteller/`.
pub const CACHE_FILE: &str = "cache.sqlite";
/// How much context a backlink preview keeps.
const EXCERPT_CHARS: usize = 200;

/// Status stored for an asset embed (`![[…]]`), which resolves to no entry.
const EMBED_STATUS: &str = "embed";

/// Outcome of a rebuild, for logging and the `index.rebuilt` event.
#[derive(Debug, Clone, Serialize)]
pub struct RebuildReport {
    pub entries: usize,
    pub links: usize,
    pub stubs: usize,
    pub duration_ms: u128,
}

/// Handle on a project's index.
pub struct Index {
    conn: Connection,
}

impl Index {
    /// Opens (creating it if needed) the index of a project.
    ///
    /// Also makes sure the cache is gitignored: a project folder is meant to be
    /// committed as-is, and the cache has no business in history.
    pub fn open(project_root: &Path) -> Result<Self> {
        let dir = project_root.join(STORYTELLER_DIR);
        std::fs::create_dir_all(&dir).map_err(|e| Error::io(&dir, e))?;
        ensure_gitignored(&dir)?;

        let conn = Connection::open(dir.join(CACHE_FILE))?;
        schema::ensure_schema(&conn)?;
        Ok(Self { conn })
    }

    /// Opens a throwaway in-memory index (tests, one-shot tooling).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        schema::ensure_schema(&conn)?;
        Ok(Self { conn })
    }

    /// Path of the cache file for a project root.
    pub fn path_in(project_root: &Path) -> PathBuf {
        project_root.join(STORYTELLER_DIR).join(CACHE_FILE)
    }

    /// Rebuilds the whole index from the given entries.
    ///
    /// This is the reference path: everything the index knows comes from here,
    /// in one transaction, so a crash mid-rebuild leaves the previous cache
    /// intact rather than a half-written one.
    pub fn rebuild(&mut self, entries: &[Entry]) -> Result<RebuildReport> {
        self.rebuild_with(entries, |_| None)
    }

    /// Rebuilds from a project, recording file metadata (mtime, size, hash) so
    /// that a later incremental pass can tell what actually changed (M2).
    pub fn rebuild_from_project(
        &mut self,
        project: &crate::project::Project,
    ) -> Result<RebuildReport> {
        let entries = project.scan();
        self.rebuild_with(&entries, |entry| {
            project
                .absolute_path(&entry.path)
                .ok()
                .and_then(|path| FileStat::of(&path))
        })
    }

    fn rebuild_with(
        &mut self,
        entries: &[Entry],
        stat_of: impl Fn(&Entry) -> Option<FileStat>,
    ) -> Result<RebuildReport> {
        let started = std::time::Instant::now();
        let resolver = Resolver::new(entries);

        // One transaction: an interrupted rebuild leaves the previous cache in
        // place instead of a half-written one.
        let transaction = self.conn.transaction()?;
        for table in ["links", "fields", "tags", "aliases", "entries"] {
            transaction.execute(&format!("DELETE FROM {table}"), [])?;
        }

        let mut link_count = 0;
        for entry in entries {
            insert_entry(&transaction, entry, stat_of(entry))?;
            link_count += insert_links(&transaction, entry, &resolver)?;
        }
        transaction.commit()?;

        let report = RebuildReport {
            entries: entries.len(),
            links: link_count,
            stubs: self.stubs()?.len(),
            duration_ms: started.elapsed().as_millis(),
        };
        tracing::info!(
            entries = report.entries,
            links = report.links,
            stubs = report.stubs,
            duration_ms = report.duration_ms,
            "index rebuilt"
        );
        Ok(report)
    }

    /// Number of indexed entries.
    pub fn entry_count(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT count(*) FROM entries", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Entry count per type, including types absent from `enabled_types`:
    /// disabling a type hides it from *creation*, never from the index.
    pub fn counts_by_type(&self) -> Result<BTreeMap<String, usize>> {
        let mut statement = self
            .conn
            .prepare("SELECT type, count(*) FROM entries GROUP BY type ORDER BY type")?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Paginated, filtered, sorted listing (`docs/api.md` §4).
    pub fn list(&self, query: &ListQuery) -> Result<Page<EntrySummary>> {
        query::list(&self.conn, query)
    }

    /// Path of the entry owning `slug`.
    ///
    /// On a slug collision the lowest path wins, deterministically; both files
    /// stay listed and both carry a `duplicate_slug` diagnostic.
    pub fn path_of(&self, slug: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row(
                "SELECT path FROM entries WHERE slug = ?1 ORDER BY path LIMIT 1",
                [slug],
                |row| row.get(0),
            )
            .optional()?)
    }

    /// Light view of an entry, straight from the index.
    pub fn summary(&self, slug: &str) -> Result<Option<EntrySummary>> {
        query::summary(&self.conn, slug)
    }

    /// Incoming links of an entry (`docs/linking.md` §4.2).
    ///
    /// Asset embeds are excluded: they reference media, not entries.
    pub fn backlinks(&self, slug: &str) -> Result<Vec<Backlink>> {
        let mut statement = self.conn.prepare(
            "SELECT e.slug, e.path, e.type, e.title, l.context, l.excerpt
             FROM links l
             JOIN entries e ON e.path = l.source_path
             WHERE l.target_slug = ?1 AND l.is_embed = 0
             ORDER BY e.path, l.byte_offset IS NULL DESC, l.byte_offset, l.context",
        )?;
        let rows = statement.query_map([slug], |row| {
            let context: String = row.get(4)?;
            Ok(Backlink {
                slug: row.get(0)?,
                path: row.get(1)?,
                type_name: row.get(2)?,
                title: row.get(3)?,
                field: LinkContext::from_stored(&context)
                    .field()
                    .map(str::to_string),
                context: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Outgoing links of an entry, each with its resolution status.
    pub fn outgoing_links(&self, slug: &str) -> Result<Vec<OutgoingLink>> {
        let mut statement = self.conn.prepare(
            "SELECT target_raw, target_slug, context, status, candidates
             FROM links
             WHERE source_slug = ?1 AND is_embed = 0
             ORDER BY byte_offset IS NULL DESC, byte_offset, context, target_raw",
        )?;
        let rows = statement.query_map([slug], |row| {
            let context: String = row.get(2)?;
            let status: String = row.get(3)?;
            let candidates: Option<String> = row.get(4)?;
            Ok(OutgoingLink {
                target_raw: row.get(0)?,
                target_slug: row.get(1)?,
                field: LinkContext::from_stored(&context)
                    .field()
                    .map(str::to_string),
                resolution: resolution_from_str(&status),
                candidates: candidates
                    .and_then(|json| serde_json::from_str(&json).ok())
                    .unwrap_or_default(),
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Every unresolved target, grouped by normalized key (`docs/linking.md` §6).
    pub fn stubs(&self) -> Result<Vec<Stub>> {
        let mut statement = self.conn.prepare(
            "SELECT target_key, target_raw, source_slug
             FROM links
             WHERE status = 'stub' AND is_embed = 0
             ORDER BY target_key, target_raw, source_slug",
        )?;
        let rows: Vec<(String, String, String)> = statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<rusqlite::Result<_>>()?;

        let mut grouped: Vec<Stub> = Vec::new();
        for (key, label, source) in rows {
            match grouped.last_mut().filter(|stub| stub.key == key) {
                Some(stub) => {
                    stub.count += 1;
                    if !stub.labels.contains(&label) {
                        stub.labels.push(label);
                    }
                    if !stub.sources.contains(&source) {
                        stub.sources.push(source);
                    }
                }
                None => grouped.push(Stub {
                    key,
                    labels: vec![label],
                    count: 1,
                    sources: vec![source],
                }),
            }
        }
        Ok(grouped)
    }

    /// Diagnostics recorded for an entry.
    pub fn diagnostics(&self, slug: &str) -> Result<Vec<Diagnostic>> {
        let raw: Option<String> = self
            .conn
            .query_row(
                "SELECT errors FROM entries WHERE slug = ?1 ORDER BY path LIMIT 1",
                [slug],
                |row| row.get(0),
            )
            .optional()?;
        Ok(raw
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default())
    }

    /// File metadata recorded at index time, for incremental reindexing (M2).
    pub fn file_stat(&self, path: &str) -> Result<Option<FileStat>> {
        Ok(self
            .conn
            .query_row(
                "SELECT mtime, size, content_hash FROM entries WHERE path = ?1",
                [path],
                |row| {
                    Ok(FileStat {
                        mtime: row.get(0)?,
                        size: row.get(1)?,
                        content_hash: row.get(2)?,
                    })
                },
            )
            .optional()?)
    }
}

/// Identity of a file's content, for change detection (mtime + hash).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileStat {
    /// Unix seconds, `None` when the platform did not provide it.
    pub mtime: Option<i64>,
    pub size: Option<i64>,
    /// sha256 of the file bytes; the authority when mtime is unreliable
    /// (mobile sync, checkouts) — see `docs/architecture.md` §4.
    pub content_hash: String,
}

impl FileStat {
    fn of(path: &Path) -> Option<Self> {
        let bytes = std::fs::read(path).ok()?;
        let metadata = std::fs::metadata(path).ok()?;
        let mtime = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64);
        Some(Self {
            mtime,
            size: Some(bytes.len() as i64),
            content_hash: content_hash(&bytes),
        })
    }
}

/// sha256 of file bytes, hex-encoded.
pub fn content_hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn insert_entry(conn: &Connection, entry: &Entry, stat: Option<FileStat>) -> Result<()> {
    let frontmatter = serde_json::to_string(&entry.frontmatter).unwrap_or_else(|_| "{}".into());
    let errors = serde_json::to_string(&entry.errors).unwrap_or_else(|_| "[]".into());
    let stat = stat.unwrap_or(FileStat {
        mtime: None,
        size: None,
        // Without file metadata, hash the parsed content: still a usable
        // fingerprint, just not byte-identical to the file's.
        content_hash: content_hash(entry.body.as_bytes()),
    });

    conn.execute(
        "INSERT INTO entries
            (path, slug, type, title, created, updated, excerpt,
             frontmatter, errors, has_errors, mtime, size, content_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            entry.path,
            entry.slug,
            entry.type_name,
            entry.title(),
            entry.created(),
            entry.updated(),
            entry.excerpt(EXCERPT_CHARS),
            frontmatter,
            errors,
            i64::from(!entry.errors.is_empty()),
            stat.mtime,
            stat.size,
            stat.content_hash,
        ],
    )?;

    for alias in entry.aliases() {
        conn.execute(
            "INSERT INTO aliases(path, alias, alias_key) VALUES (?1, ?2, ?3)",
            params![entry.path, alias, normalize(&alias)],
        )?;
    }
    for tag in entry.tags() {
        conn.execute(
            "INSERT INTO tags(path, tag, tag_key) VALUES (?1, ?2, ?3)",
            params![entry.path, tag, normalize(&tag)],
        )?;
    }
    for (key, value) in &entry.frontmatter {
        insert_field(conn, &entry.path, key, value)?;
    }
    Ok(())
}

/// Flattens one frontmatter value into `fields` rows.
fn insert_field(conn: &Connection, path: &str, key: &str, value: &Value) -> Result<()> {
    match value {
        Value::Array(items) => {
            for (position, item) in items.iter().enumerate() {
                insert_scalar_field(conn, path, key, position as i64, item)?;
            }
        }
        // Nested mappings are flattened with a dotted key, mirroring how link
        // extraction names them.
        Value::Object(map) => {
            for (nested_key, nested) in map {
                insert_field(conn, path, &format!("{key}.{nested_key}"), nested)?;
            }
        }
        scalar => insert_scalar_field(conn, path, key, 0, scalar)?,
    }
    Ok(())
}

fn insert_scalar_field(
    conn: &Connection,
    path: &str,
    key: &str,
    position: i64,
    value: &Value,
) -> Result<()> {
    let (text, number) = match value {
        Value::Null => (None, None),
        Value::Bool(b) => (Some(b.to_string()), None),
        Value::Number(n) => (Some(n.to_string()), n.as_f64()),
        Value::String(s) => (Some(s.clone()), None),
        // A list inside a list, or a mapping inside a list: store its JSON so
        // the value is still visible, without pretending it is filterable.
        other => (Some(other.to_string()), None),
    };

    conn.execute(
        "INSERT INTO fields(path, key, position, value_text, value_key, value_num)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            path,
            key,
            position,
            text,
            text.as_deref().map(normalize),
            number
        ],
    )?;
    Ok(())
}

fn insert_links(conn: &Connection, entry: &Entry, resolver: &Resolver) -> Result<usize> {
    let occurrences = links::extract_from_frontmatter(&entry.frontmatter)
        .into_iter()
        .chain(links::extract_from_body(&entry.body));

    let mut count = 0;
    for occurrence in occurrences {
        let key = occurrence.target_key();
        // Embeds point at assets: they never resolve to an entry and never
        // create a stub (`docs/linking.md` §1, "Embeds").
        let (status, target_slug, candidates) = if occurrence.is_embed {
            (EMBED_STATUS.to_string(), None, None)
        } else {
            let resolved = resolver.resolve_key(&key);
            let candidates = match resolved.resolution {
                Resolution::Ambiguous => serde_json::to_string(&resolved.candidates).ok(),
                _ => None,
            };
            (
                resolved.resolution.as_str().to_string(),
                resolved.slug,
                candidates,
            )
        };

        conn.execute(
            "INSERT INTO links
                (source_path, source_slug, target_raw, target_key, target_slug,
                 context, display, anchor, byte_offset, status, is_embed, excerpt, candidates)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                entry.path,
                entry.slug,
                occurrence.target_raw,
                key,
                target_slug,
                occurrence.context.as_stored(),
                occurrence.display,
                occurrence.anchor,
                occurrence.offset.map(|o| o as i64),
                status,
                i64::from(occurrence.is_embed),
                excerpt_for(entry, &occurrence),
                candidates,
            ],
        )?;
        count += 1;
    }
    Ok(count)
}

/// Preview text shown next to a backlink.
fn excerpt_for(entry: &Entry, occurrence: &LinkOccurrence) -> String {
    match occurrence.offset {
        Some(offset) => links::body_excerpt(&entry.body, offset, EXCERPT_CHARS),
        // A frontmatter link has no surrounding prose; the field name (carried
        // separately) plus the target is all the context there is.
        None => format!("[[{}]]", occurrence.target_raw),
    }
}

fn resolution_from_str(status: &str) -> Resolution {
    match status {
        "resolved" => Resolution::Resolved,
        "ambiguous" => Resolution::Ambiguous,
        _ => Resolution::Stub,
    }
}

/// Writes `.storyteller/.gitignore` so the cache never reaches Git.
fn ensure_gitignored(storyteller_dir: &Path) -> Result<()> {
    let path = storyteller_dir.join(".gitignore");
    if path.exists() {
        return Ok(());
    }
    let contents = format!(
        "# Index: disposable cache, rebuilt from the markdown files.\n\
         {CACHE_FILE}\n{CACHE_FILE}-shm\n{CACHE_FILE}-wal\n"
    );
    std::fs::write(&path, contents).map_err(|e| Error::io(&path, e))
}
