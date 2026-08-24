//! Query-string parsing (`docs/api.md` §4).
//!
//! Reserved parameters are listed once, here: every *other* parameter is a
//! frontmatter field filter, which is what makes `?status=writing` work without
//! the server knowing the field exists.

use std::collections::BTreeSet;

use storyteller_core::index::{ListQuery, SortSpec};

use crate::error::{ApiError, ApiResult};

/// Parameters that mean something to the API itself.
const RESERVED: [&str; 8] = [
    "type", "tag", "q", "sort", "page", "per_page", "include", "render",
];

/// A parsed query string.
#[derive(Debug, Default)]
pub struct ListParams {
    pub list: ListQuery,
    /// Extra sections the client asked to be included in the response.
    pub include: BTreeSet<String>,
    /// `?render=` value, when present — only `html` is accepted.
    render: Option<String>,
}

impl ListParams {
    pub fn wants_backlinks(&self) -> bool {
        self.include.contains("backlinks")
    }

    /// `?render=html` (`docs/api.md` §2, ADR 0015): render `body` to HTML.
    pub fn wants_html(&self) -> bool {
        self.render.as_deref() == Some("html")
    }
}

/// Parses a raw query string into filters, sort, pagination and includes.
pub fn parse(raw: Option<&str>) -> ApiResult<ListParams> {
    let pairs: Vec<(String, String)> = form_urlencoded::parse(raw.unwrap_or_default().as_bytes())
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();

    let mut params = ListParams::default();
    params.list.page = 1;

    for (key, value) in pairs {
        match key.as_str() {
            "type" => params.list.types.push(value),
            "tag" => params.list.tags.push(value),
            "sort" => {
                params.list.sort = SortSpec::parse(&value).or(params.list.sort);
            }
            "page" => params.list.page = parse_number(&key, &value)?,
            "per_page" => params.list.per_page = parse_number(&key, &value)?,
            "include" => {
                params
                    .include
                    .extend(value.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string));
            }
            // On `/entities`, `q` restricts the list (`docs/api.md` §4); on
            // `/search` it is additionally required — that check lives in the
            // search route, since an empty/absent `q` is perfectly valid here.
            "q" => params.list.q = Some(value),
            // Rendering side was an open architecture question; resolved by
            // ADR 0015 (in `core`). Only `html` is a served value.
            "render" => {
                if value != "html" {
                    return Err(ApiError::bad_request(format!(
                        "unknown `render` value `{value}`; supported: html"
                    )));
                }
                params.render = Some(value);
            }
            _ => params.list.fields.push((key, value)),
        }
    }

    params.list = params.list.sanitized();
    validate_includes(&params.include)?;
    Ok(params)
}

fn parse_number(key: &str, value: &str) -> ApiResult<usize> {
    value.parse().map_err(|_| {
        ApiError::bad_request(format!("`{key}` must be a positive integer, got `{value}`"))
    })
}

fn validate_includes(include: &BTreeSet<String>) -> ApiResult<()> {
    for name in include {
        if name != "backlinks" {
            return Err(ApiError::bad_request(format!(
                "unknown `include` section `{name}`; supported: backlinks"
            )));
        }
    }
    Ok(())
}

/// True when the parameter name is handled by the API rather than treated as a
/// field filter. Exposed for tests and documentation.
pub fn is_reserved(name: &str) -> bool {
    RESERVED.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_repeatable_and_field_filters() {
        let params = parse(Some("type=character&type=faction&tag=pov&status=vivante")).unwrap();
        assert_eq!(params.list.types, ["character", "faction"]);
        assert_eq!(params.list.tags, ["pov"]);
        assert_eq!(
            params.list.fields,
            [("status".to_string(), "vivante".to_string())]
        );
    }

    #[test]
    fn decodes_percent_escapes() {
        let params = parse(Some("pov=%5B%5BAria%20Solane%5D%5D")).unwrap();
        assert_eq!(
            params.list.fields,
            [("pov".to_string(), "[[Aria Solane]]".to_string())]
        );
    }

    #[test]
    fn parses_sort_and_pagination() {
        let params = parse(Some("sort=-updated&page=3&per_page=10")).unwrap();
        assert!(params.list.sort.unwrap().descending);
        assert_eq!(params.list.page, 3);
        assert_eq!(params.list.per_page, 10);
    }

    #[test]
    fn defaults_pagination_when_absent() {
        let params = parse(None).unwrap();
        assert_eq!(params.list.page, 1);
        assert_eq!(
            params.list.per_page,
            storyteller_core::index::DEFAULT_PER_PAGE
        );
        assert!(params.list.sort.is_none());
    }

    #[test]
    fn rejects_a_non_numeric_page() {
        let error = parse(Some("page=deux")).unwrap_err();
        assert_eq!(error.status, axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn parses_include_sections() {
        let params = parse(Some("include=backlinks")).unwrap();
        assert!(params.wants_backlinks());
        assert!(!parse(None).unwrap().wants_backlinks());
    }

    #[test]
    fn rejects_an_unknown_include_section() {
        let error = parse(Some("include=graph")).unwrap_err();
        assert_eq!(error.status, axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn parses_q() {
        let params = parse(Some("q=verre")).unwrap();
        assert_eq!(params.list.q.as_deref(), Some("verre"));
        assert!(parse(None).unwrap().list.q.is_none());
    }

    #[test]
    fn parses_render_html() {
        let params = parse(Some("render=html")).unwrap();
        assert!(params.wants_html());
        assert!(!parse(None).unwrap().wants_html());
    }

    #[test]
    fn rejects_an_unknown_render_value() {
        let error = parse(Some("render=pdf")).unwrap_err();
        assert_eq!(error.status, axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn reserved_names_are_not_field_filters() {
        for name in RESERVED {
            assert!(is_reserved(name));
        }
        assert!(!is_reserved("status"));
    }
}
