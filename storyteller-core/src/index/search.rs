//! Full-text search (`docs/api.md` §3 "Search"; engine decided by [ADR
//! 0011](../../../docs/adr/0011-index-sqlite-fts5.md): SQLite + FTS5).
//!
//! Shares `query::build_filters`/`build_order` with the plain `/entities`
//! list for `type`/`tag`/`<field>` filters and explicit `sort=` — the only
//! things specific to search are the FTS `MATCH` join (needed to select
//! `snippet()`/`rank`, which a subquery can't expose) and the relevance
//! default order.

use rusqlite::{types::Value as SqlValue, Connection};
use serde::Serialize;

use crate::error::Result;
use crate::model::EntrySummary;

use super::query::{build_filters, build_match_query, build_order, ListQuery, Page};

/// A search hit: the same shape as a list item, plus a highlighted excerpt
/// from whichever indexed column matched best (title, aliases, tags, body).
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    #[serde(flatten)]
    pub entry: EntrySummary,
    pub snippet: String,
}

/// Highlight markers around a matched term in `snippet`, and the ellipsis
/// marking elided text — plain and simple rather than configurable, since
/// nothing in `docs/api.md` calls for anything else.
const SNIPPET_OPEN: &str = "<mark>";
const SNIPPET_CLOSE: &str = "</mark>";
const SNIPPET_ELLIPSIS: &str = "…";
/// Roughly how many tokens of context a snippet carries.
const SNIPPET_TOKENS: i64 = 12;

/// Runs a full-text search. `query.q` is assumed non-empty — the server layer
/// validates that, matching "required on `/search`" (`docs/api.md` §4).
///
/// The FTS condition stays a *subquery* (`entries.path IN (SELECT path FROM
/// entries_fts WHERE entries_fts MATCH ?)`, same shape `build_filters` already
/// uses for `q` on plain `/entities`) rather than a real `JOIN`: a join would
/// bring `entries_fts`'s own `title`/columns into the same name scope as
/// `entries`, making `build_order`'s shared, unqualified `ORDER BY "title"`
/// ambiguous. `snippet()`/`rank` — which do need a live FTS5 query in scope —
/// are instead computed by their own small correlated subqueries.
pub(super) fn search(conn: &Connection, query: &ListQuery) -> Result<Page<SearchResult>> {
    let query = query.clone().sanitized();
    let match_query = query
        .q
        .as_deref()
        .and_then(build_match_query)
        .unwrap_or_default();

    // `query.q` is set, so this already carries the FTS subquery condition.
    let (where_clause, filter_params) = build_filters(&query);

    let total: i64 = conn.query_row(
        &format!("SELECT count(*) FROM entries {where_clause}"),
        rusqlite::params_from_iter(filter_params.iter()),
        |row| row.get(0),
    )?;

    // Params in the exact order their `?` appear in `sql` below: the
    // snippet subquery's MATCH comes first (it's in the SELECT list, which
    // precedes WHERE in the text), then the WHERE clause's own params.
    let mut params: Vec<SqlValue> = vec![SqlValue::from(match_query.clone())];
    params.extend(filter_params);

    // Relevance by default (via its own correlated subquery, for the same
    // JOIN-vs-subquery reason as the snippet); an explicit `sort=` overrides
    // it, same as a saved view would expect if it names a column instead.
    let order_clause = if query.sort.is_some() {
        build_order(&query, &mut params)
    } else {
        params.push(SqlValue::from(match_query));
        "ORDER BY (SELECT rank FROM entries_fts
                    WHERE entries_fts.path = entries.path AND entries_fts MATCH ?) ASC,
                  path ASC"
            .to_string()
    };

    let offset = (query.page - 1).saturating_mul(query.per_page);
    params.push(SqlValue::from(query.per_page as i64));
    params.push(SqlValue::from(i64::try_from(offset).unwrap_or(i64::MAX)));

    let sql = format!(
        "SELECT entries.slug, entries.path, entries.type, entries.title,
                entries.excerpt, entries.has_errors, entries.updated,
                (SELECT snippet(entries_fts, -1, '{SNIPPET_OPEN}', '{SNIPPET_CLOSE}', '{SNIPPET_ELLIPSIS}', {SNIPPET_TOKENS})
                 FROM entries_fts WHERE entries_fts.path = entries.path AND entries_fts MATCH ?)
         FROM entries
         {where_clause}
         {order_clause}
         LIMIT ? OFFSET ?"
    );

    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        Ok(SearchRow {
            slug: row.get(0)?,
            path: row.get(1)?,
            type_name: row.get(2)?,
            title: row.get(3)?,
            excerpt: row.get(4)?,
            has_errors: row.get::<_, i64>(5)? != 0,
            updated: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
            snippet: row.get(7)?,
        })
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?.into_result(conn)?);
    }

    Ok(Page {
        items,
        page: query.page,
        per_page: query.per_page,
        total: total as usize,
    })
}

struct SearchRow {
    slug: String,
    path: String,
    type_name: String,
    title: String,
    excerpt: String,
    has_errors: bool,
    updated: String,
    snippet: String,
}

impl SearchRow {
    fn into_result(self, conn: &Connection) -> Result<SearchResult> {
        Ok(SearchResult {
            snippet: self.snippet,
            entry: EntrySummary {
                tags: tags_of(conn, &self.path)?,
                slug: self.slug,
                path: self.path,
                type_name: self.type_name,
                title: self.title,
                excerpt: self.excerpt,
                has_errors: self.has_errors,
                updated: self.updated,
            },
        })
    }
}

fn tags_of(conn: &Connection, path: &str) -> Result<Vec<String>> {
    let mut statement =
        conn.prepare_cached("SELECT tag FROM tags WHERE path = ?1 ORDER BY rowid")?;
    let rows = statement.query_map([path], |row| row.get(0))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}
