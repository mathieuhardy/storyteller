//! Full-text search tests (`docs/api.md` §3 "Search"; ADR 0011).

mod common;

use common::Fixture;
use storyteller_core::index::{ListQuery, SearchResult, SortSpec};

fn slugs(items: &[SearchResult]) -> Vec<&str> {
    items.iter().map(|item| item.entry.slug.as_str()).collect()
}

fn search(q: &str) -> ListQuery {
    ListQuery {
        q: Some(q.to_string()),
        per_page: 200,
        ..Default::default()
    }
}

#[test]
fn finds_a_body_match_with_a_highlighted_snippet() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    // "ironique" is in aria-solane's body ("Voix" section), not in any
    // frontmatter field — FTS only indexes title/aliases/tags/body, not
    // arbitrary frontmatter (`docs/api.md` "Search").
    let page = index.search(&search("ironique")).unwrap();
    assert!(slugs(&page.items).contains(&"aria-solane"), "{:?}", slugs(&page.items));
    let hit = page.items.iter().find(|r| r.entry.slug == "aria-solane").unwrap();
    assert!(hit.snippet.contains("<mark>"), "{}", hit.snippet);
}

#[test]
fn matching_is_accent_insensitive() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    // "Cité de Verre" is a title; searching without the accent must still
    // find it (French content, ADR 0010) — the FTS table strips diacritics.
    let page = index.search(&search("cite")).unwrap();
    assert!(
        slugs(&page.items).contains(&"cite-de-verre"),
        "{:?}",
        slugs(&page.items)
    );
}

#[test]
fn is_prefix_tolerant_per_term() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    // "prism" should find "Prisme" (title and body) via prefix matching.
    let page = index.search(&search("prism")).unwrap();
    assert!(!page.items.is_empty());
}

#[test]
fn combines_with_type_and_tag_filters() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    let query = ListQuery {
        types: vec!["location".into()],
        ..search("cite")
    };
    let page = index.search(&query).unwrap();
    assert!(slugs(&page.items).contains(&"cite-de-verre"));
    assert!(page.items.iter().all(|r| r.entry.type_name == "location"));

    // The same query, but asking for a type absent from the matches, is empty.
    let query = ListQuery {
        types: vec!["faction".into()],
        ..search("cite")
    };
    assert!(index.search(&query).unwrap().items.is_empty());
}

#[test]
fn an_explicit_sort_overrides_relevance_ranking() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    let query = ListQuery {
        sort: Some(SortSpec {
            field: "title".into(),
            descending: false,
        }),
        ..search("e")
    };
    let page = index.search(&query).unwrap();
    let titles: Vec<String> = page
        .items
        .iter()
        .map(|r| r.entry.title.clone())
        .collect();
    let mut sorted = titles.clone();
    sorted.sort_by_key(|t| t.to_lowercase());
    assert_eq!(titles, sorted, "{titles:?}");
}

#[test]
fn no_match_is_an_empty_page_not_an_error() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    let page = index.search(&search("zzznonexistentzzz")).unwrap();
    assert!(page.items.is_empty());
    assert_eq!(page.total, 0);
}

#[test]
fn unbalanced_quotes_and_fts_operators_do_not_error() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    // Free text, not FTS5 query syntax: a naive MATCH with these characters
    // would be a syntax error. Terms are individually quoted, so this must
    // still run (even if it matches nothing).
    for q in ["\"unterminated", "OR NOT AND", "a:b", "*"] {
        index.search(&search(q)).unwrap();
    }
}
