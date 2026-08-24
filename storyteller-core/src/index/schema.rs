//! SQLite schema of the index.
//!
//! The index is a **disposable cache**: every column here is derived from the
//! markdown files and can be thrown away at any time (golden rule 3). That is
//! also why a schema change needs no migration — we simply rebuild.
//!
//! Note what is deliberately *absent* from `entries`: the body. Full text
//! lives in the files; `entries` keeps only what views need (title, excerpt,
//! fields, links). The one place the body *is* duplicated is `entries_fts`
//! (search, M5) — an FTS5 index needs its own copy of the searchable text, but
//! it is exactly as disposable as the rest: rebuilt from the files, never
//! read back as a source of truth.

use rusqlite::Connection;

use crate::error::Result;

/// Version of the index layout. Bumping it discards existing caches.
///
/// Bumped to 2 for `entries_fts` (search, M5): an existing on-disk cache from
/// before this change has no such table, and would otherwise be reused as-is
/// (`ensure_schema` only rebuilds on a version mismatch).
pub const INDEX_SCHEMA_VERSION: i64 = 2;

const SCHEMA: &str = r#"
CREATE TABLE meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;

-- One row per entry file. Keyed by path, not slug: two files may share a
-- filename, which is a conflict we report rather than a row we drop.
CREATE TABLE entries (
    path         TEXT PRIMARY KEY,
    slug         TEXT NOT NULL,
    type         TEXT NOT NULL,
    title        TEXT NOT NULL,
    created      TEXT,
    updated      TEXT,
    excerpt      TEXT NOT NULL,
    frontmatter  TEXT NOT NULL,  -- JSON object, original key order
    errors       TEXT NOT NULL,  -- JSON array of diagnostics
    has_errors   INTEGER NOT NULL,
    mtime        INTEGER,        -- unix seconds, for incremental reindexing
    size         INTEGER,
    content_hash TEXT NOT NULL   -- sha256 of the file bytes
) STRICT;

CREATE INDEX entries_slug ON entries(slug);
CREATE INDEX entries_type ON entries(type);

CREATE TABLE aliases (
    path      TEXT NOT NULL REFERENCES entries(path) ON DELETE CASCADE,
    alias     TEXT NOT NULL,
    alias_key TEXT NOT NULL
) STRICT;

CREATE INDEX aliases_key ON aliases(alias_key);
CREATE INDEX aliases_path ON aliases(path);

CREATE TABLE tags (
    path    TEXT NOT NULL REFERENCES entries(path) ON DELETE CASCADE,
    tag     TEXT NOT NULL,
    tag_key TEXT NOT NULL
) STRICT;

CREATE INDEX tags_key ON tags(tag_key);
CREATE INDEX tags_path ON tags(path);

-- Flattened frontmatter values, for filtering and sorting. List values yield
-- one row per element, keeping their position.
CREATE TABLE fields (
    path       TEXT NOT NULL REFERENCES entries(path) ON DELETE CASCADE,
    key        TEXT NOT NULL,
    position   INTEGER NOT NULL,  -- 0 for scalars, index within a list otherwise
    value_text TEXT,              -- textual form, as written
    value_key  TEXT,              -- normalized form, for tolerant comparison
    value_num  REAL               -- numeric form when the value is a number
) STRICT;

CREATE INDEX fields_key ON fields(key, value_key);
CREATE INDEX fields_path ON fields(path);

-- links(source, target, context) of docs/linking.md §4.1, plus what a preview
-- needs. `target_slug` is NULL unless the target resolved to exactly one entry.
CREATE TABLE links (
    id          INTEGER PRIMARY KEY,
    source_path TEXT NOT NULL REFERENCES entries(path) ON DELETE CASCADE,
    source_slug TEXT NOT NULL,
    target_raw  TEXT NOT NULL,  -- as typed, never normalized
    target_key  TEXT NOT NULL,  -- normalized match key
    target_slug TEXT,
    context     TEXT NOT NULL,  -- 'body' or 'field:<name>'
    display     TEXT,
    anchor      TEXT,
    byte_offset INTEGER,        -- offset in the body, NULL for frontmatter links
    status      TEXT NOT NULL,  -- 'resolved' | 'stub' | 'ambiguous' | 'embed'
    is_embed    INTEGER NOT NULL,
    excerpt     TEXT NOT NULL,
    candidates  TEXT            -- JSON array of competing slugs, when ambiguous
) STRICT;

CREATE INDEX links_target_slug ON links(target_slug) WHERE target_slug IS NOT NULL;
CREATE INDEX links_target_key ON links(target_key);
CREATE INDEX links_source ON links(source_path);
CREATE INDEX links_status ON links(status);

-- Full-text search (docs/api.md "Search"; ADR 0011 chose SQLite+FTS5).
-- `remove_diacritics 2` makes search accent-insensitive, matching the rest of
-- the app's tolerant text comparison (see `normalize`) — content is French
-- (ADR 0010), where that matters a lot ("cite" should find "Cité").
CREATE VIRTUAL TABLE entries_fts USING fts5(
    path UNINDEXED,
    title,
    aliases,
    tags,
    body,
    tokenize = 'unicode61 remove_diacritics 2'
);
"#;

/// Applies pragmas, then creates the schema if the database is empty or stale.
///
/// A cache written by another layout is **dropped**, never migrated: rebuilding
/// from the files is always correct and costs a scan.
pub fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", true)?;

    if schema_version(conn)? == Some(INDEX_SCHEMA_VERSION) {
        return Ok(());
    }

    drop_everything(conn)?;
    conn.execute_batch(SCHEMA)?;
    conn.execute(
        "INSERT INTO meta(key, value) VALUES ('index_schema_version', ?1)",
        [INDEX_SCHEMA_VERSION.to_string()],
    )?;
    Ok(())
}

fn schema_version(conn: &Connection) -> Result<Option<i64>> {
    let exists: bool = conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'meta'",
        [],
        |_| Ok(true),
    ) == Ok(true);
    if !exists {
        return Ok(None);
    }
    let version: Option<String> = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'index_schema_version'",
            [],
            |row| row.get(0),
        )
        .ok();
    Ok(version.and_then(|v| v.parse().ok()))
}

fn drop_everything(conn: &Connection) -> Result<()> {
    // The FTS5 virtual table owns "shadow" tables (`entries_fts_data`,
    // `entries_fts_idx`, …) that also show up in `sqlite_master` as plain
    // tables. Dropping the virtual table first cascades to them correctly;
    // dropping a shadow table directly does not, so it must go first and the
    // generic loop below must not also touch them.
    conn.execute_batch("DROP TABLE IF EXISTS entries_fts")?;

    let names: Vec<(String, String)> = {
        let mut statement = conn.prepare(
            "SELECT type, name FROM sqlite_master
             WHERE type IN ('table', 'index', 'view')
               AND name NOT LIKE 'sqlite_%'
               AND name NOT LIKE 'entries_fts%'",
        )?;
        let rows = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect::<rusqlite::Result<_>>()?
    };
    // Tables first: dropping a table takes its indexes with it.
    for (_, name) in names.iter().filter(|(kind, _)| kind == "table") {
        conn.execute_batch(&format!("DROP TABLE IF EXISTS \"{name}\""))?;
    }
    for (kind, name) in names.iter().filter(|(kind, _)| kind != "table") {
        conn.execute_batch(&format!(
            "DROP {} IF EXISTS \"{name}\"",
            kind.to_uppercase()
        ))?;
    }
    Ok(())
}
