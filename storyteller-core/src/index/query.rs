//! Filtering, sorting and pagination over the index (`docs/api.md` §4).
//!
//! Filters are combined as the doc specifies: several `type` values are an OR,
//! several `tag` values an AND, and a `<field>` filter matches a frontmatter
//! value. Comparisons use the same normalized keys as link resolution, so
//! `?tag=Protagoniste` finds `protagoniste`.

use rusqlite::{types::Value as SqlValue, Connection, OptionalExtension};
use serde::Serialize;

use crate::error::Result;
use crate::model::EntrySummary;
use crate::normalize::normalize;

/// Default page size when the client asks for none.
pub const DEFAULT_PER_PAGE: usize = 50;
/// Hard cap, so a client cannot ask the server to materialize everything.
pub const MAX_PER_PAGE: usize = 200;

/// Sort instruction: a field name and a direction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortSpec {
    pub field: String,
    pub descending: bool,
}

impl SortSpec {
    /// Parses the `?sort=` syntax: `title`, or `-updated` for descending.
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        let (field, descending) = match raw.strip_prefix('-') {
            Some(rest) => (rest, true),
            None => (raw, false),
        };
        let field = field.trim();
        (!field.is_empty()).then(|| Self {
            field: field.to_string(),
            descending,
        })
    }
}

/// A list query.
#[derive(Debug, Clone, Default)]
pub struct ListQuery {
    /// Types to keep; empty means every type.
    pub types: Vec<String>,
    /// Tags an entry must **all** carry.
    pub tags: Vec<String>,
    /// Frontmatter `(key, value)` equality filters, ANDed together.
    pub fields: Vec<(String, String)>,
    /// Full-text filter (`docs/api.md` §4): on `/entities` it restricts the
    /// list; `/search` requires it and additionally ranks by relevance.
    pub q: Option<String>,
    pub sort: Option<SortSpec>,
    /// 1-based page number.
    pub page: usize,
    pub per_page: usize,
}

impl ListQuery {
    /// Clamps page and per_page into their allowed ranges.
    pub fn sanitized(mut self) -> Self {
        self.page = self.page.max(1);
        self.per_page = match self.per_page {
            0 => DEFAULT_PER_PAGE,
            n => n.min(MAX_PER_PAGE),
        };
        self
    }
}

/// One page of results, with the metadata the frontend needs.
#[derive(Debug, Clone, Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub page: usize,
    pub per_page: usize,
    /// Total number of matches, all pages considered.
    pub total: usize,
}

/// Columns that can be sorted on directly. Anything else is treated as a
/// frontmatter field name and sorted through the `fields` table.
const ENTRY_COLUMNS: [&str; 6] = ["title", "slug", "type", "created", "updated", "path"];

pub(super) fn list(conn: &Connection, query: &ListQuery) -> Result<Page<EntrySummary>> {
    let query = query.clone().sanitized();
    let (where_clause, mut params) = build_filters(&query);

    let total: i64 = conn.query_row(
        &format!("SELECT count(*) FROM entries {where_clause}"),
        rusqlite::params_from_iter(params.iter()),
        |row| row.get(0),
    )?;

    let order_clause = build_order(&query, &mut params);
    // Saturating all the way to SQL: a client asking for page 10^18 must get an
    // empty page. An overflowing cast would wrap to a negative offset, which
    // SQLite silently reads as 0 — i.e. it would serve the *first* page.
    let offset = (query.page - 1).saturating_mul(query.per_page);
    params.push(SqlValue::from(query.per_page as i64));
    params.push(SqlValue::from(i64::try_from(offset).unwrap_or(i64::MAX)));

    let sql = format!(
        "SELECT slug, path, type, title, excerpt, has_errors, updated
         FROM entries
         {where_clause}
         {order_clause}
         LIMIT ? OFFSET ?"
    );

    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        Ok(SummaryRow {
            slug: row.get(0)?,
            path: row.get(1)?,
            type_name: row.get(2)?,
            title: row.get(3)?,
            excerpt: row.get(4)?,
            has_errors: row.get::<_, i64>(5)? != 0,
            updated: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
        })
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?.into_summary(conn)?);
    }

    Ok(Page {
        items,
        page: query.page,
        per_page: query.per_page,
        total: total as usize,
    })
}

pub(super) fn summary(conn: &Connection, slug: &str) -> Result<Option<EntrySummary>> {
    let row = conn
        .query_row(
            "SELECT slug, path, type, title, excerpt, has_errors, updated
             FROM entries WHERE slug = ?1 ORDER BY path LIMIT 1",
            [slug],
            |row| {
                Ok(SummaryRow {
                    slug: row.get(0)?,
                    path: row.get(1)?,
                    type_name: row.get(2)?,
                    title: row.get(3)?,
                    excerpt: row.get(4)?,
                    has_errors: row.get::<_, i64>(5)? != 0,
                    updated: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                })
            },
        )
        .optional()?;

    match row {
        Some(row) => Ok(Some(row.into_summary(conn)?)),
        None => Ok(None),
    }
}

struct SummaryRow {
    slug: String,
    path: String,
    type_name: String,
    title: String,
    excerpt: String,
    has_errors: bool,
    updated: String,
}

impl SummaryRow {
    fn into_summary(self, conn: &Connection) -> Result<EntrySummary> {
        Ok(EntrySummary {
            tags: tags_of(conn, &self.path)?,
            slug: self.slug,
            path: self.path,
            type_name: self.type_name,
            title: self.title,
            excerpt: self.excerpt,
            has_errors: self.has_errors,
            updated: self.updated,
        })
    }
}

fn tags_of(conn: &Connection, path: &str) -> Result<Vec<String>> {
    let mut statement =
        conn.prepare_cached("SELECT tag FROM tags WHERE path = ?1 ORDER BY rowid")?;
    let rows = statement.query_map([path], |row| row.get(0))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// Turns free text into a safe, forgiving FTS5 `MATCH` expression: each
/// whitespace-separated term becomes an individually-quoted prefix match
/// (`"word"*`), ANDed together. Quoting every term is what makes this safe
/// against FTS5 query-syntax injection (unbalanced quotes, `OR`/`NOT`,
/// column filters via `:`) from arbitrary user input — a quoted string
/// followed by `*` is FTS5's own syntax for "starts with", so prefix
/// matching still works per term.
pub(super) fn build_match_query(q: &str) -> Option<String> {
    let terms: Vec<String> = q
        .split_whitespace()
        .map(|term| format!("\"{}\"*", term.replace('"', "\"\"")))
        .collect();
    (!terms.is_empty()).then(|| terms.join(" AND "))
}

/// Builds the `WHERE` clause and its parameters.
pub(super) fn build_filters(query: &ListQuery) -> (String, Vec<SqlValue>) {
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<SqlValue> = Vec::new();

    if !query.types.is_empty() {
        let placeholders = vec!["?"; query.types.len()].join(", ");
        conditions.push(format!("type IN ({placeholders})"));
        params.extend(query.types.iter().map(|t| SqlValue::from(t.clone())));
    }

    // AND semantics: one EXISTS per tag.
    for tag in &query.tags {
        conditions.push(
            "EXISTS (SELECT 1 FROM tags t WHERE t.path = entries.path AND t.tag_key = ?)"
                .to_string(),
        );
        params.push(SqlValue::from(normalize(tag)));
    }

    for (key, value) in &query.fields {
        conditions.push(
            "EXISTS (SELECT 1 FROM fields f
                     WHERE f.path = entries.path AND f.key = ? AND f.value_key = ?)"
                .to_string(),
        );
        params.push(SqlValue::from(key.clone()));
        params.push(SqlValue::from(normalize(value)));
    }

    if let Some(q) = query.q.as_deref() {
        if let Some(match_query) = build_match_query(q) {
            conditions.push(
                "entries.path IN (SELECT path FROM entries_fts WHERE entries_fts MATCH ?)"
                    .to_string(),
            );
            params.push(SqlValue::from(match_query));
        }
    }

    let clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };
    (clause, params)
}

/// Builds the `ORDER BY` clause.
///
/// `path` is always the last tie-breaker: two entries with the same title must
/// not swap places between two identical requests.
pub(super) fn build_order(query: &ListQuery, params: &mut Vec<SqlValue>) -> String {
    let Some(sort) = &query.sort else {
        return "ORDER BY title COLLATE NOCASE ASC, path ASC".to_string();
    };
    let direction = if sort.descending { "DESC" } else { "ASC" };

    if ENTRY_COLUMNS.contains(&sort.field.as_str()) {
        let collation = if sort.field == "title" {
            " COLLATE NOCASE"
        } else {
            ""
        };
        // Entries missing the value sort last in both directions: an entry
        // without `updated` is not "the oldest", it is simply unknown.
        return format!(
            "ORDER BY \"{}\"{collation} {direction} NULLS LAST, path ASC",
            sort.field
        );
    }

    // Sorting on a frontmatter field: numbers numerically, the rest as text.
    params.push(SqlValue::from(sort.field.clone()));
    params.push(SqlValue::from(sort.field.clone()));
    format!(
        "ORDER BY
            (SELECT f.value_num FROM fields f
             WHERE f.path = entries.path AND f.key = ? ORDER BY f.position LIMIT 1)
            {direction} NULLS LAST,
            (SELECT f.value_key FROM fields f
             WHERE f.path = entries.path AND f.key = ? ORDER BY f.position LIMIT 1)
            {direction} NULLS LAST,
            path ASC"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sort_syntax() {
        assert_eq!(
            SortSpec::parse("title"),
            Some(SortSpec {
                field: "title".into(),
                descending: false
            })
        );
        assert_eq!(
            SortSpec::parse("-updated"),
            Some(SortSpec {
                field: "updated".into(),
                descending: true
            })
        );
        assert_eq!(SortSpec::parse("  -order  ").unwrap().field, "order");
        assert_eq!(SortSpec::parse(""), None);
        assert_eq!(SortSpec::parse("-"), None);
    }

    #[test]
    fn sanitizes_pagination() {
        let query = ListQuery {
            page: 0,
            per_page: 0,
            ..Default::default()
        }
        .sanitized();
        assert_eq!(query.page, 1);
        assert_eq!(query.per_page, DEFAULT_PER_PAGE);

        let query = ListQuery {
            per_page: 10_000,
            ..Default::default()
        }
        .sanitized();
        assert_eq!(query.per_page, MAX_PER_PAGE);
    }

    #[test]
    fn filter_placeholders_match_parameter_count() {
        let query = ListQuery {
            types: vec!["character".into(), "faction".into()],
            tags: vec!["pov".into()],
            fields: vec![("status".into(), "alive".into())],
            ..Default::default()
        };
        let (clause, params) = build_filters(&query);
        assert_eq!(clause.matches('?').count(), params.len());
        assert!(clause.starts_with("WHERE type IN (?, ?)"));
    }

    #[test]
    fn no_filters_yields_no_where_clause() {
        let (clause, params) = build_filters(&ListQuery::default());
        assert!(clause.is_empty());
        assert!(params.is_empty());
    }

    #[test]
    fn order_by_field_adds_its_parameters() {
        let mut params = Vec::new();
        let clause = build_order(
            &ListQuery {
                sort: Some(SortSpec {
                    field: "order".into(),
                    descending: false,
                }),
                ..Default::default()
            },
            &mut params,
        );
        assert_eq!(params.len(), 2, "one per correlated subquery");
        assert!(clause.contains("value_num"));
    }

    #[test]
    fn order_by_column_adds_no_parameter() {
        let mut params = Vec::new();
        let clause = build_order(
            &ListQuery {
                sort: Some(SortSpec {
                    field: "updated".into(),
                    descending: true,
                }),
                ..Default::default()
            },
            &mut params,
        );
        assert!(params.is_empty());
        assert!(clause.contains("\"updated\" DESC"));
    }
}
