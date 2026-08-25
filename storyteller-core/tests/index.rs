//! Index integration tests, against a realistic project fixture.
//!
//! These cover the M1 Definition of Done: the core parses a real project
//! without loss, the index rebuilds entirely from the files, and backlinks are
//! exact and bidirectional.

mod common;

use std::collections::BTreeSet;

use common::Fixture;
use storyteller_core::error::codes;
use storyteller_core::index::{Index, ListQuery, SortSpec};
use storyteller_core::links::Resolution;

fn slugs(items: &[storyteller_core::EntrySummary]) -> Vec<&str> {
    items.iter().map(|item| item.slug.as_str()).collect()
}

fn all(query: ListQuery) -> ListQuery {
    ListQuery {
        per_page: 200,
        ..query
    }
}

#[test]
fn scans_every_entry_and_nothing_else() {
    let fixture = Fixture::new();
    let project = fixture.project();
    let paths = project.entry_paths();

    assert!(
        !paths.iter().any(|p| p.starts_with("assets/")),
        "assets/ holds media, never entries: {paths:?}"
    );
    assert!(
        !paths.iter().any(|p| p.starts_with(".storyteller")),
        "the technical folder is not scanned: {paths:?}"
    );
    assert!(paths.contains(&"project.md".to_string()));
    assert_eq!(paths.len(), 14);
}

#[test]
fn index_rebuilds_entirely_from_the_files() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    assert_eq!(index.entry_count().unwrap(), 14);
    let counts = index.counts_by_type().unwrap();
    assert_eq!(counts["character"], 2);
    assert_eq!(counts["chapter"], 2);
    assert_eq!(counts["concept"], 2);
    assert_eq!(counts["location"], 2);
    // `notes/sans-type.md` declares no type and is counted as a note.
    assert_eq!(counts["note"], 2);
    assert_eq!(counts["project"], 1);
}

#[test]
fn deleting_the_cache_costs_nothing() {
    let fixture = Fixture::new();
    let before = {
        let (_project, index) = fixture.indexed();
        index.list(&all(ListQuery::default())).unwrap()
    };

    let cache = Index::path_in(fixture.root());
    assert!(cache.exists(), "the cache file should have been created");
    std::fs::remove_file(&cache).unwrap();

    let (_project, index) = fixture.indexed();
    let after = index.list(&all(ListQuery::default())).unwrap();

    assert_eq!(after.total, before.total);
    assert_eq!(slugs(&after.items), slugs(&before.items));
}

#[test]
fn rebuilding_twice_changes_nothing() {
    let fixture = Fixture::new();
    let project = fixture.project();
    let mut index = Index::open(project.root()).unwrap();

    let first = index.rebuild_from_project(&project).unwrap();
    let first_list = index.list(&all(ListQuery::default())).unwrap();
    let first_backlinks = index.backlinks("aria-solane").unwrap();

    let second = index.rebuild_from_project(&project).unwrap();
    let second_list = index.list(&all(ListQuery::default())).unwrap();
    let second_backlinks = index.backlinks("aria-solane").unwrap();

    assert_eq!(first.entries, second.entries);
    assert_eq!(first.links, second.links);
    assert_eq!(first.stubs, second.stubs);
    assert_eq!(slugs(&first_list.items), slugs(&second_list.items));
    assert_eq!(first_backlinks.len(), second_backlinks.len());
}

#[test]
fn the_cache_is_gitignored() {
    let fixture = Fixture::new();
    let _ = fixture.indexed();
    let gitignore =
        std::fs::read_to_string(fixture.root().join(".storyteller/.gitignore")).unwrap();
    assert!(gitignore.contains("cache.sqlite"));
}

#[test]
fn the_file_wins_over_the_index() {
    let fixture = Fixture::new();
    let (project, index) = fixture.indexed();

    fixture.write(
        "characters/kael-vantre.md",
        "---\ntype: character\ntitle: Kael Vantre le Sombre\n---\nRéécrit hors de l'app.\n",
    );

    // The index still holds the stale title — it is only a cache…
    assert_eq!(
        index.summary("kael-vantre").unwrap().unwrap().title,
        "Kael Vantre"
    );
    // …while a read goes to the file, the source of truth.
    let entry = project.read_entry("characters/kael-vantre.md").unwrap();
    assert_eq!(entry.title(), "Kael Vantre le Sombre");
}

#[test]
fn backlinks_are_exact_and_bidirectional() {
    let fixture = Fixture::new();
    let (project, index) = fixture.indexed();

    for entry in project.scan() {
        for link in index.outgoing_links(&entry.slug).unwrap() {
            let Some(target) = link.target_slug else {
                continue;
            };
            let backlinks = index.backlinks(&target).unwrap();
            assert!(
                backlinks.iter().any(|back| back.slug == entry.slug),
                "{} links to {target} but is missing from its backlinks",
                entry.slug
            );
        }
    }
}

#[test]
fn backlinks_report_the_field_that_carried_the_link() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();
    let backlinks = index.backlinks("aria-solane").unwrap();

    let pov = backlinks
        .iter()
        .find(|back| back.slug == "03-la-felure" && back.field.as_deref() == Some("pov"))
        .expect("the chapter declares Aria as pov");
    assert_eq!(pov.type_name, "chapter");
    assert_eq!(pov.title, "La Fêlure");
    assert_eq!(pov.context, "[[Aria Solane]]");

    let from_body = backlinks
        .iter()
        .find(|back| back.slug == "kael-vantre" && back.field.is_none())
        .expect("Kael mentions Aria in his body");
    assert!(
        from_body.context.contains("[[Aria Solane]]"),
        "a body backlink carries an excerpt: {:?}",
        from_body.context
    );
}

#[test]
fn a_broken_entry_still_produces_its_backlink() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    let backlinks = index.backlinks("aria-solane").unwrap();
    assert!(
        backlinks.iter().any(|back| back.slug == "note-cassee"),
        "an entry with invalid YAML must keep contributing its body links"
    );
}

#[test]
fn links_resolve_by_slug_title_and_alias() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    // By title, from a frontmatter field.
    let chapter = index.outgoing_links("03-la-felure").unwrap();
    let pov = chapter
        .iter()
        .find(|link| link.field.as_deref() == Some("pov"))
        .unwrap();
    assert_eq!(pov.resolution, Resolution::Resolved);
    assert_eq!(pov.target_slug.as_deref(), Some("aria-solane"));

    // By slug, from a body link written in kebab-case.
    let aria = index.outgoing_links("aria-solane").unwrap();
    assert!(aria.iter().any(|link| link.target_raw == "atelier-solane"
        && link.target_slug.as_deref() == Some("atelier-solane")));
}

#[test]
fn an_anchor_does_not_change_resolution() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    let link = index
        .outgoing_links("03-la-felure")
        .unwrap()
        .into_iter()
        .find(|link| link.target_raw == "Système du Verre Vivant" && link.field.is_none())
        .expect("the body links to the system with a #Limites anchor");
    assert_eq!(link.resolution, Resolution::Resolved);
    assert_eq!(link.target_slug.as_deref(), Some("systeme-du-verre-vivant"));
}

#[test]
fn unresolved_targets_are_grouped_as_stubs() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    let stubs = index.stubs().unwrap();
    let keys: BTreeSet<&str> = stubs.iter().map(|stub| stub.key.as_str()).collect();
    assert_eq!(keys, BTreeSet::from(["humain", "maitre orlan"]));

    let orlan = stubs.iter().find(|s| s.key == "maitre orlan").unwrap();
    assert_eq!(orlan.labels, ["Maître Orlan"], "original text is preserved");
    assert_eq!(orlan.count, 2, "cited from a body and from a leader field");
    assert_eq!(orlan.sources, ["aria-solane", "ordre-du-prisme"]);
}

#[test]
fn creating_the_missing_entry_turns_a_stub_into_a_link() {
    let fixture = Fixture::new();

    fixture.write(
        "characters/maitre-orlan.md",
        "---\ntype: character\ntitle: Maître Orlan\n---\nLe vieux maître.\n",
    );
    let (_project, index) = fixture.indexed();

    assert!(
        !index
            .stubs()
            .unwrap()
            .iter()
            .any(|stub| stub.key == "maitre orlan"),
        "the stub should be gone once the entry exists"
    );
    assert!(
        index
            .backlinks("maitre-orlan")
            .unwrap()
            .iter()
            .any(|back| back.slug == "ordre-du-prisme"),
        "and the links that pointed at it now resolve"
    );
}

#[test]
fn deleting_an_entry_turns_its_backlinks_into_stubs() {
    let fixture = Fixture::new();
    fixture.remove("locations/atelier-solane.md");
    let (_project, index) = fixture.indexed();

    let stub = index
        .stubs()
        .unwrap()
        .into_iter()
        .find(|stub| stub.key == "atelier solane")
        .expect("the chapter still points at the deleted location");
    assert!(stub.sources.contains(&"03-la-felure".to_string()));
}

#[test]
fn an_ambiguous_target_is_never_silently_resolved() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    // `Le Prisme` is the title of a concept *and* of an object.
    let link = index
        .outgoing_links("heritage")
        .unwrap()
        .into_iter()
        .find(|link| link.target_raw == "Le Prisme")
        .unwrap();
    assert_eq!(link.resolution, Resolution::Ambiguous);
    assert_eq!(link.target_slug, None);
    assert_eq!(link.candidates, ["prisme-mere", "transmission"]);

    // An ambiguous target is not a stub: it needs disambiguation, not creation.
    assert!(!index
        .stubs()
        .unwrap()
        .iter()
        .any(|stub| stub.key == "le prisme"));
}

#[test]
fn a_title_beats_an_alias() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    // "Le Prisme" is also an alias of the faction, at a lower rank: the alias
    // must not even enter the ambiguity.
    let link = index
        .outgoing_links("heritage")
        .unwrap()
        .into_iter()
        .find(|link| link.target_raw == "Le Prisme")
        .unwrap();
    assert!(!link.candidates.contains(&"ordre-du-prisme".to_string()));
}

#[test]
fn embeds_are_not_links_and_never_create_stubs() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    let links = index.outgoing_links("aria-solane").unwrap();
    assert!(
        !links.iter().any(|link| link.target_raw.ends_with(".jpg")),
        "an ![[…]] embed is a media reference, not an entry link"
    );
    assert!(!index
        .stubs()
        .unwrap()
        .iter()
        .any(|stub| stub.key.ends_with(".jpg")));
}

#[test]
fn graph_includes_every_entry_as_a_node() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    let graph = index.graph().unwrap();
    assert_eq!(graph.nodes.len(), 14);
    assert!(graph.nodes.iter().any(|n| n.slug == "aria-solane"));
    // Every edge connects two entries that are actually nodes — a stub or an
    // ambiguous link never appears (no single target to draw a line to).
    assert!(graph.edges.iter().all(|edge| {
        graph.nodes.iter().any(|n| n.slug == edge.source)
            && graph.nodes.iter().any(|n| n.slug == edge.target)
    }));
}

#[test]
fn graph_collapses_repeated_links_between_the_same_pair() {
    let fixture = Fixture::new();
    fixture.write(
        "notes/liens.md",
        "---\ntype: note\ntitle: Liens\nrelated:\n  - \"[[Aria Solane]]\"\n---\n[[Aria Solane]] à nouveau.\n",
    );
    let (_project, index) = fixture.indexed();

    let graph = index.graph().unwrap();
    let count = graph
        .edges
        .iter()
        .filter(|e| e.source == "liens" && e.target == "aria-solane")
        .count();
    assert_eq!(count, 1, "two mentions of the same target is still one edge");
}

#[test]
fn graph_excludes_self_links() {
    let fixture = Fixture::new();
    fixture.write("notes/auto.md", "---\ntype: note\ntitle: Auto\n---\n[[Auto]]\n");
    let (_project, index) = fixture.indexed();

    let graph = index.graph().unwrap();
    assert!(!graph
        .edges
        .iter()
        .any(|e| e.source == "auto" && e.target == "auto"));
}

#[test]
fn diagnostics_are_attached_to_the_offending_entry() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    let broken = index.diagnostics("note-cassee").unwrap();
    assert!(broken.iter().any(|d| d.code == codes::YAML_PARSE_ERROR));

    let untyped = index.diagnostics("sans-type").unwrap();
    assert!(untyped
        .iter()
        .any(|d| d.code == codes::MISSING_REQUIRED_FIELD && d.field.as_deref() == Some("type")));

    assert!(
        index.diagnostics("aria-solane").unwrap().is_empty(),
        "a well-formed entry carries no diagnostic"
    );
}

#[test]
fn a_broken_entry_does_not_break_the_listing() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    let page = index.list(&all(ListQuery::default())).unwrap();
    let broken = page
        .items
        .iter()
        .find(|item| item.slug == "note-cassee")
        .expect("it must appear in the list");
    assert!(broken.has_errors);
    assert_eq!(broken.title, "Note cassée", "salvaged from broken YAML");
}

#[test]
fn unknown_frontmatter_keys_survive_indexing() {
    let fixture = Fixture::new();
    let (project, _index) = fixture.indexed();
    let entry = project.read_entry("characters/aria-solane.md").unwrap();
    assert_eq!(entry.frontmatter["obsidian_note_id"], "91f3c0");
}

#[test]
fn list_filters_by_type() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    let page = index
        .list(&all(ListQuery {
            types: vec!["character".into()],
            ..Default::default()
        }))
        .unwrap();
    assert_eq!(slugs(&page.items), ["aria-solane", "kael-vantre"]);
    assert_eq!(page.total, 2);

    // Repeated `type` is an OR.
    let page = index
        .list(&all(ListQuery {
            types: vec!["character".into(), "chapter".into()],
            ..Default::default()
        }))
        .unwrap();
    assert_eq!(page.total, 4);
}

#[test]
fn list_filters_by_tag_with_and_semantics() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    let page = index
        .list(&all(ListQuery {
            tags: vec!["pov".into()],
            ..Default::default()
        }))
        .unwrap();
    assert_eq!(slugs(&page.items), ["aria-solane"]);

    let page = index
        .list(&all(ListQuery {
            tags: vec!["pov".into(), "protagoniste".into()],
            ..Default::default()
        }))
        .unwrap();
    assert_eq!(slugs(&page.items), ["aria-solane"]);

    let page = index
        .list(&all(ListQuery {
            tags: vec!["pov".into(), "antagoniste".into()],
            ..Default::default()
        }))
        .unwrap();
    assert!(page.items.is_empty(), "no entry carries both tags");
}

#[test]
fn tag_and_field_filters_ignore_case_and_accents() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    let page = index
        .list(&all(ListQuery {
            tags: vec!["PROTAGONISTE".into()],
            ..Default::default()
        }))
        .unwrap();
    assert_eq!(slugs(&page.items), ["aria-solane"]);

    let page = index
        .list(&all(ListQuery {
            fields: vec![("status".into(), "VIVANTE".into())],
            ..Default::default()
        }))
        .unwrap();
    assert_eq!(slugs(&page.items), ["aria-solane"]);
}

#[test]
fn list_filters_on_a_frontmatter_link_field() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    // The documented `?pov=[[Aria Solane]]` form: the raw wikilink string.
    let page = index
        .list(&all(ListQuery {
            fields: vec![("pov".into(), "[[Aria Solane]]".into())],
            ..Default::default()
        }))
        .unwrap();
    assert_eq!(slugs(&page.items), ["03-la-felure"]);
}

#[test]
fn list_filters_on_a_list_field_element() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    let page = index
        .list(&all(ListQuery {
            fields: vec![("locations".into(), "[[Atelier Solane]]".into())],
            ..Default::default()
        }))
        .unwrap();
    assert_eq!(slugs(&page.items), ["03-la-felure", "04-le-four"]);
}

#[test]
fn list_restricts_to_a_full_text_match() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    // `q` on `/entities` restricts the list (`docs/api.md` §4) — same shape
    // as any other filter, unlike the dedicated `/search` endpoint which also
    // ranks by relevance and returns a snippet.
    let page = index
        .list(&all(ListQuery {
            q: Some("ironique".into()),
            ..Default::default()
        }))
        .unwrap();
    assert_eq!(slugs(&page.items), ["aria-solane"]);

    let page = index
        .list(&all(ListQuery {
            q: Some("zzznonexistentzzz".into()),
            ..Default::default()
        }))
        .unwrap();
    assert!(page.items.is_empty());
}

#[test]
fn list_sorts_by_column_and_by_frontmatter_field() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    let chapters = ListQuery {
        types: vec!["chapter".into()],
        ..Default::default()
    };

    let page = index
        .list(&all(ListQuery {
            sort: SortSpec::parse("order"),
            ..chapters.clone()
        }))
        .unwrap();
    assert_eq!(slugs(&page.items), ["03-la-felure", "04-le-four"]);

    let page = index
        .list(&all(ListQuery {
            sort: SortSpec::parse("-order"),
            ..chapters.clone()
        }))
        .unwrap();
    assert_eq!(slugs(&page.items), ["04-le-four", "03-la-felure"]);

    let page = index
        .list(&all(ListQuery {
            sort: SortSpec::parse("-updated"),
            ..chapters
        }))
        .unwrap();
    assert_eq!(slugs(&page.items), ["04-le-four", "03-la-felure"]);
}

#[test]
fn sorting_puts_entries_without_the_field_last() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    // Only chapters carry `order`; every other entry must land at the end,
    // whatever the direction.
    for sort in ["order", "-order"] {
        let page = index
            .list(&all(ListQuery {
                sort: SortSpec::parse(sort),
                ..Default::default()
            }))
            .unwrap();
        let ordered = slugs(&page.items);
        let last_chapter = ordered
            .iter()
            .position(|slug| !slug.starts_with("03-") && !slug.starts_with("04-"))
            .unwrap();
        assert!(
            last_chapter <= 2,
            "chapters should come first with sort={sort}: {ordered:?}"
        );
    }
}

#[test]
fn list_paginates_without_losing_or_repeating_entries() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    let mut seen = Vec::new();
    for page_number in 1..=4 {
        let page = index
            .list(&ListQuery {
                page: page_number,
                per_page: 5,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(page.total, 14, "total counts every match, not the page");
        assert_eq!(page.per_page, 5);
        seen.extend(slugs(&page.items).into_iter().map(str::to_string));
    }

    let unique: BTreeSet<&String> = seen.iter().collect();
    assert_eq!(seen.len(), 14);
    assert_eq!(unique.len(), 14);
}

#[test]
fn an_absurd_page_number_yields_an_empty_page() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();
    let page = index
        .list(&ListQuery {
            page: usize::MAX,
            per_page: 50,
            ..Default::default()
        })
        .unwrap();
    assert!(page.items.is_empty());
    assert_eq!(
        page.total, 14,
        "the total still describes the whole match set"
    );
}

#[test]
fn per_page_is_capped() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();
    let page = index
        .list(&ListQuery {
            per_page: 10_000,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(page.per_page, storyteller_core::index::MAX_PER_PAGE);
}

#[test]
fn summaries_carry_tags_and_an_excerpt() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    let aria = index.summary("aria-solane").unwrap().unwrap();
    assert_eq!(aria.type_name, "character");
    assert_eq!(aria.tags, ["protagoniste", "pov"]);
    assert!(aria.excerpt.starts_with("Aria a grandi"));
    assert!(!aria.has_errors);

    assert!(index.summary("inconnu").unwrap().is_none());
}

#[test]
fn file_metadata_is_recorded_for_incremental_reindexing() {
    let fixture = Fixture::new();
    let (_project, index) = fixture.indexed();

    let stat = index
        .file_stat("characters/aria-solane.md")
        .unwrap()
        .expect("stat recorded");
    assert!(stat.size.unwrap() > 0);
    assert!(stat.mtime.is_some());
    assert_eq!(stat.content_hash.len(), 64, "hex-encoded sha256");
}

#[test]
fn the_config_declares_every_type_as_enabled() {
    let fixture = Fixture::new();
    let project = fixture.project();
    let config = project.config();
    assert_eq!(config.schema_version, 1);
    assert!(config.is_enabled("character"));
    assert!(config.errors.is_empty());
}
