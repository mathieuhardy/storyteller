//! API integration tests: the contract of `docs/api.md`, as served.

mod common;

use axum::http::StatusCode;
use common::{error_code, slugs, TestServer};
use serde_json::json;

#[tokio::test]
async fn version_reports_api_core_and_schema() {
    let server = TestServer::new();
    let body = server.get_ok("/api/v1/version").await;
    assert_eq!(body["api_version"], "v1");
    assert_eq!(body["schema_version"], 1);
    assert!(body["core_version"].as_str().unwrap().starts_with("0."));
}

#[tokio::test]
async fn project_exposes_the_root_entry_and_its_metadata() {
    let server = TestServer::new();
    let body = server.get_ok("/api/v1/project").await;

    assert_eq!(body["entry"]["type"], "project");
    assert_eq!(body["entry"]["frontmatter"]["title"], "La Fêlure");
    assert_eq!(body["entry"]["frontmatter"]["status"], "writing");
    assert_eq!(body["schema_version"], 1);
    assert_eq!(body["enabled_types"].as_array().unwrap().len(), 11);
    assert_eq!(body["stats"]["entries"], 14);
    assert_eq!(body["stats"]["by_type"]["character"], 2);
    assert!(body["errors"].as_array().unwrap().is_empty());
    assert_eq!(body["root"], server.root().to_str().unwrap());
}

#[tokio::test]
async fn types_lists_enabled_types_with_french_labels() {
    let server = TestServer::new();
    let body = server.get_ok("/api/v1/types").await;
    let types = body.as_array().unwrap();
    assert_eq!(types.len(), 11);

    let character = types
        .iter()
        .find(|t| t["name"] == "character")
        .expect("character type");
    assert_eq!(character["label"], "Personnage");
    assert_eq!(character["folder"], "characters");
    assert_eq!(character["enabled"], true);

    // Common fields come first, so a generated form starts with type/title.
    let fields = character["fields"].as_array().unwrap();
    assert_eq!(fields[0]["name"], "type");
    assert_eq!(fields[1]["name"], "title");
    assert_eq!(fields[1]["label"], "Titre");
}

#[tokio::test]
async fn a_type_schema_describes_kinds_enums_and_link_targets() {
    let server = TestServer::new();
    let body = server.get_ok("/api/v1/types/chapter").await;
    let fields = body["fields"].as_array().unwrap();

    let field = |name: &str| {
        fields
            .iter()
            .find(|f| f["name"] == name)
            .unwrap_or_else(|| panic!("{name} missing"))
            .clone()
    };

    assert_eq!(field("order")["kind"], "number");
    assert_eq!(field("pov")["kind"], "link");
    assert_eq!(field("pov")["link_targets"][0], "character");
    assert_eq!(field("locations")["kind"], "link-list");
    assert_eq!(field("chapter_status")["kind"], "enum");
    assert_eq!(field("chapter_status")["enum_values"][0], "to-write");
    // Non-enum fields do not carry an empty `enum_values` noise field.
    assert!(field("order").get("enum_values").is_none());
    assert_eq!(field("order")["tier"], "mvp");
    assert_eq!(field("wordcount")["tier"], "optional");
}

#[tokio::test]
async fn a_field_can_accept_a_number_or_a_text() {
    let server = TestServer::new();
    let body = server.get_ok("/api/v1/types/character").await;
    let age = body["fields"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "age")
        .expect("age missing");
    assert_eq!(age["kind"], "number-or-text");
    assert_eq!(age["label"], "Âge");
}

#[tokio::test]
async fn patch_type_disables_it_without_touching_existing_entries() {
    let server = TestServer::new();

    let (status, body) = server
        .send("PATCH", "/api/v1/types/faction", &json!({"enabled": false}))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["enabled"], false);

    // Reflected immediately in the type list and the project's enabled_types.
    let types = server.get_ok("/api/v1/types?all=true").await;
    let faction = types
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "faction")
        .unwrap();
    assert_eq!(faction["enabled"], false);
    let project = server.get_ok("/api/v1/project").await;
    assert!(!project["enabled_types"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t == "faction"));

    // Existing entries of the now-disabled type are still listed and readable.
    let entities = server.get_ok("/api/v1/entities?type=faction").await;
    assert!(!entities["items"].as_array().unwrap().is_empty());

    // Creating a new one is refused while it stays disabled.
    let (status, body) = server
        .send(
            "POST",
            "/api/v1/entities",
            &json!({"type": "faction", "title": "Nouvelle Faction"}),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");

    // Re-enabling restores creation.
    server
        .send("PATCH", "/api/v1/types/faction", &json!({"enabled": true}))
        .await;
    let (status, _) = server
        .send(
            "POST",
            "/api/v1/entities",
            &json!({"type": "faction", "title": "Nouvelle Faction"}),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn patch_an_unknown_type_is_a_normalized_404() {
    let server = TestServer::new();
    let (status, body) = server
        .send("PATCH", "/api/v1/types/dragon", &json!({"enabled": false}))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&body), "not_found");
}

#[tokio::test]
async fn an_unknown_type_is_a_normalized_404() {
    let server = TestServer::new();
    let (status, body) = server.get("/api/v1/types/dragon").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&body), "not_found");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("dragon"));
}

#[tokio::test]
async fn entities_returns_a_paginated_list() {
    let server = TestServer::new();
    let body = server.get_ok("/api/v1/entities?per_page=5").await;
    assert_eq!(body["total"], 14);
    assert_eq!(body["page"], 1);
    assert_eq!(body["per_page"], 5);
    assert_eq!(slugs(&body).len(), 5);

    let item = &body["items"][0];
    for key in [
        "slug",
        "path",
        "type",
        "title",
        "tags",
        "excerpt",
        "has_errors",
        "updated",
    ] {
        assert!(item.get(key).is_some(), "{key} missing from a list item");
    }
    assert!(item.get("body").is_none(), "a list stays light");
}

#[tokio::test]
async fn entities_filters_by_type_tag_and_field() {
    let server = TestServer::new();

    let body = server.get_ok("/api/v1/entities?type=character").await;
    assert_eq!(slugs(&body), ["aria-solane", "kael-vantre"]);

    let body = server.get_ok("/api/v1/entities?tag=pov").await;
    assert_eq!(slugs(&body), ["aria-solane"]);

    // The documented `?pov=[[Aria]]` form, percent-encoded.
    let body = server
        .get_ok("/api/v1/entities?pov=%5B%5BAria%20Solane%5D%5D")
        .await;
    assert_eq!(slugs(&body), ["03-la-felure"]);
}

#[tokio::test]
async fn entities_sorts_on_a_frontmatter_field() {
    let server = TestServer::new();
    let body = server
        .get_ok("/api/v1/entities?type=chapter&sort=-order")
        .await;
    assert_eq!(slugs(&body), ["04-le-four", "03-la-felure"]);
}

#[tokio::test]
async fn a_bad_pagination_parameter_is_a_400() {
    let server = TestServer::new();
    let (status, body) = server.get("/api/v1/entities?page=deux").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&body), "bad_request");
}

#[tokio::test]
async fn an_entry_exposes_frontmatter_body_and_no_backlinks_by_default() {
    let server = TestServer::new();
    let body = server.get_ok("/api/v1/entities/aria-solane").await;

    assert_eq!(body["slug"], "aria-solane");
    assert_eq!(body["path"], "characters/aria-solane.md");
    assert_eq!(body["type"], "character");
    assert_eq!(body["frontmatter"]["title"], "Aria Solane");
    // Links in frontmatter stay raw wikilink strings; the API never resolves
    // them in place.
    assert_eq!(body["frontmatter"]["home"], "[[Cité de Verre]]");
    assert_eq!(body["frontmatter"]["factions"][0], "[[Ordre du Prisme]]");
    assert!(body["body"].as_str().unwrap().contains("[[Kael Vantre]]"));
    assert!(body["errors"].as_array().unwrap().is_empty());
    assert!(
        body.get("backlinks").is_none(),
        "backlinks are opt-in via ?include=backlinks"
    );
}

#[tokio::test]
async fn frontmatter_keeps_its_key_order_and_unknown_keys() {
    let server = TestServer::new();
    let body = server.get_ok("/api/v1/entities/aria-solane").await;
    let keys: Vec<&str> = body["frontmatter"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();

    assert_eq!(&keys[..4], ["type", "title", "aliases", "tags"]);
    assert_eq!(
        keys.last(),
        Some(&"obsidian_note_id"),
        "a key written by another tool must survive: {keys:?}"
    );
}

#[tokio::test]
async fn html_is_absent_unless_render_html_is_requested() {
    let server = TestServer::new();
    let body = server.get_ok("/api/v1/entities/aria-solane").await;
    assert!(body.get("html").is_none());
}

#[tokio::test]
async fn render_html_resolves_links_stubs_and_a_missing_embed() {
    let server = TestServer::new();
    let body = server
        .get_ok("/api/v1/entities/aria-solane?render=html")
        .await;
    let html = body["html"].as_str().unwrap();

    // Resolved wikilink → a real link to the entry's slug.
    assert!(
        html.contains(r#"<a class="wikilink resolved" href="/entry/cite-de-verre">Cité de Verre</a>"#),
        "{html}"
    );
    // Unresolved wikilink → a stub span, not a broken link.
    assert!(
        html.contains(r#"<span class="wikilink stub" data-target="Maître Orlan">Maître Orlan</span>"#),
        "{html}"
    );
    // `![[aria-solane.jpg]]` has no matching file under `assets/` in the
    // fixture → a visible "not found" block, not a broken `<img>`.
    assert!(html.contains("asset-embed missing"), "{html}");
    assert!(!html.contains("<img"), "{html}");
    // Plain markdown still renders normally around the spliced fragments.
    assert!(html.contains("<h2>Voix</h2>"), "{html}");
}

#[tokio::test]
async fn render_html_resolves_an_uploaded_embed() {
    let server = TestServer::new();
    server
        .post_multipart("/api/v1/assets", "aria-solane.jpg", b"fake-jpeg")
        .await;

    let body = server
        .get_ok("/api/v1/entities/aria-solane?render=html")
        .await;
    let html = body["html"].as_str().unwrap();
    assert!(
        html.contains(r#"<img class="asset-embed" src="/api/v1/assets/assets/aria-solane.jpg""#),
        "{html}"
    );
}

#[tokio::test]
async fn an_unsupported_render_value_is_a_400() {
    let server = TestServer::new();
    let (status, body) = server.get("/api/v1/entities/aria-solane?render=pdf").await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(error_code(&body), "bad_request");
}

#[tokio::test]
async fn include_backlinks_attaches_them_to_the_entry() {
    let server = TestServer::new();
    let body = server
        .get_ok("/api/v1/entities/aria-solane?include=backlinks")
        .await;
    let backlinks = body["backlinks"].as_array().unwrap();
    assert!(!backlinks.is_empty());
    assert!(backlinks.iter().any(|back| back["slug"] == "03-la-felure"));
}

#[tokio::test]
async fn backlinks_carry_source_type_field_and_context() {
    let server = TestServer::new();
    let body = server
        .get_ok("/api/v1/entities/aria-solane/backlinks")
        .await;
    let backlinks = body.as_array().unwrap();

    let pov = backlinks
        .iter()
        .find(|back| back["slug"] == "03-la-felure" && back["field"] == "pov")
        .expect("the chapter declares Aria as pov");
    assert_eq!(pov["type"], "chapter");
    assert_eq!(pov["title"], "La Fêlure");
    assert_eq!(pov["path"], "chapters/03-la-felure.md");

    let from_body = backlinks
        .iter()
        .find(|back| back["slug"] == "kael-vantre")
        .expect("Kael mentions Aria in prose");
    assert!(
        from_body["field"].is_null(),
        "a body link has no field: {from_body}"
    );
    assert!(from_body["context"].as_str().unwrap().contains("Aria"));
}

#[tokio::test]
async fn backlinks_of_an_unknown_entry_are_a_404_not_an_empty_list() {
    let server = TestServer::new();
    let (status, body) = server.get("/api/v1/entities/personne/backlinks").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&body), "not_found");
}

#[tokio::test]
async fn outgoing_links_carry_their_field_and_resolution_status() {
    let server = TestServer::new();
    let body = server.get_ok("/api/v1/entities/aria-solane/links").await;
    let links = body.as_array().unwrap();

    let find = |raw: &str| {
        links
            .iter()
            .find(|l| l["target_raw"] == raw)
            .unwrap_or_else(|| panic!("link to {raw} missing: {body}"))
            .clone()
    };

    // A resolved frontmatter link carries its field name and the matched slug.
    let home = find("Cité de Verre");
    assert_eq!(home["field"], "home");
    assert_eq!(home["resolution"], "resolved");
    assert_eq!(home["target_slug"], "cite-de-verre");

    // A body link to a missing entry is a stub, with no field and no slug.
    let orlan = find("Maître Orlan");
    assert_eq!(orlan["resolution"], "stub");
    assert!(
        orlan["field"].is_null(),
        "a body link has no field: {orlan}"
    );
    assert!(orlan.get("target_slug").is_none() || orlan["target_slug"].is_null());

    // The species points at an unknown entry too.
    assert_eq!(find("Humain")["resolution"], "stub");
    assert_eq!(find("Humain")["field"], "species");
}

#[tokio::test]
async fn an_ambiguous_link_lists_its_candidates() {
    let server = TestServer::new();
    // `objects/prisme-mere` and `concepts/transmission` both title "Le Prisme",
    // so `[[Le Prisme]]` cannot resolve to one of them.
    let body = server.get_ok("/api/v1/entities/heritage/links").await;
    let prisme = body
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["target_raw"] == "Le Prisme")
        .expect("Héritage links to Le Prisme")
        .clone();
    assert_eq!(prisme["resolution"], "ambiguous");
    assert!(prisme["target_slug"].is_null());
    let candidates: Vec<&str> = prisme["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .collect();
    assert_eq!(candidates, ["prisme-mere", "transmission"]);
}

#[tokio::test]
async fn links_of_an_unknown_entry_are_a_404() {
    let server = TestServer::new();
    let (status, body) = server.get("/api/v1/entities/personne/links").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&body), "not_found");
}

#[tokio::test]
async fn stubs_group_unresolved_targets_with_their_sources() {
    let server = TestServer::new();
    let body = server.get_ok("/api/v1/stubs").await;
    let stubs = body.as_array().unwrap();

    let orlan = stubs
        .iter()
        .find(|s| s["key"] == "maitre orlan")
        .expect("Maître Orlan is a stub");
    assert_eq!(orlan["labels"][0], "Maître Orlan");
    // Cited both as the faction leader and in Aria's body.
    assert_eq!(orlan["count"], 2);
    let sources: Vec<&str> = orlan["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap())
        .collect();
    assert!(sources.contains(&"aria-solane"));
    assert!(sources.contains(&"ordre-du-prisme"));

    // The ambiguous target is not a stub; the malformed `[[??` never resolves to
    // one either (it is not a whole-string wikilink in frontmatter).
    assert!(stubs.iter().all(|s| s["key"] != "le prisme"));
}

#[tokio::test]
async fn creating_an_entry_from_a_stub_resolves_the_link() {
    let server = TestServer::new();

    // Aria links to the missing `[[Maître Orlan]]`.
    let before = server.get_ok("/api/v1/entities/aria-solane/links").await;
    assert!(before
        .as_array()
        .unwrap()
        .iter()
        .any(|l| l["target_raw"] == "Maître Orlan" && l["resolution"] == "stub"));

    // Promote the stub via the ordinary create path, title pre-filled with the
    // link text (`docs/linking.md` §6.3).
    let (status, _) = server
        .send(
            "POST",
            "/api/v1/entities",
            &json!({ "type": "character", "title": "Maître Orlan" }),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    // No source file was rewritten, yet the link now resolves to the new entry.
    let after = server.get_ok("/api/v1/entities/aria-solane/links").await;
    let orlan = after
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["target_raw"] == "Maître Orlan")
        .expect("the link is still there")
        .clone();
    assert_eq!(orlan["resolution"], "resolved");
    assert_eq!(orlan["target_slug"], "maitre-orlan");

    // And it has left the stub list.
    let stubs = server.get_ok("/api/v1/stubs").await;
    assert!(stubs
        .as_array()
        .unwrap()
        .iter()
        .all(|s| s["key"] != "maitre orlan"));
}

#[tokio::test]
async fn a_broken_entry_is_served_with_200_and_diagnostics() {
    let server = TestServer::new();
    let body = server.get_ok("/api/v1/entities/note-cassee").await;

    assert_eq!(body["frontmatter"]["title"], "Note cassée");
    let errors = body["errors"].as_array().unwrap();
    assert!(
        errors.iter().any(|e| e["code"] == "yaml_parse_error"),
        "expected a yaml_parse_error: {errors:?}"
    );
    assert_eq!(errors[0]["severity"], "error");
    assert!(
        body["body"].as_str().unwrap().contains("[[Aria Solane]]"),
        "the body is preserved as-is"
    );
}

#[tokio::test]
async fn an_entry_without_type_is_served_as_a_note() {
    let server = TestServer::new();
    let body = server.get_ok("/api/v1/entities/sans-type").await;
    assert_eq!(body["type"], "note");
    let errors = body["errors"].as_array().unwrap();
    assert!(errors
        .iter()
        .any(|e| e["code"] == "missing_required_field" && e["field"] == "type"));
}

#[tokio::test]
async fn an_unknown_entry_is_a_normalized_404() {
    let server = TestServer::new();
    let (status, body) = server.get("/api/v1/entities/personne").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&body), "not_found");
    assert!(body["error"]["details"].is_object());
}

#[tokio::test]
async fn the_file_is_read_at_request_time() {
    let server = TestServer::new();
    std::fs::write(
        server.root().join("characters/kael-vantre.md"),
        "---\ntype: character\ntitle: Kael le Sombre\n---\nRéécrit dans vim.\n",
    )
    .unwrap();

    // No reindex in between: a full entry read goes to the source of truth.
    let body = server.get_ok("/api/v1/entities/kael-vantre").await;
    assert_eq!(body["frontmatter"]["title"], "Kael le Sombre");
    assert!(body["body"].as_str().unwrap().contains("vim"));
}

#[tokio::test]
async fn entities_q_restricts_the_list_to_a_full_text_match() {
    let server = TestServer::new();
    // "ironique" is only in aria-solane's body (docs/api.md §4: `q` on
    // `/entities` restricts the list, same shape as any other filter).
    let body = server.get_ok("/api/v1/entities?q=ironique").await;
    assert_eq!(slugs(&body), ["aria-solane"]);
}

#[tokio::test]
async fn create_writes_an_entry_and_makes_it_immediately_readable() {
    let server = TestServer::new();
    let (status, body) = server
        .send(
            "POST",
            "/api/v1/entities",
            &json!({
                "type": "character",
                "title": "Maître Orlan",
                "frontmatter": { "role": "mentor", "factions": ["[[Ordre du Prisme]]"] },
                "body": "Un vieux verrier.\n"
            }),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["slug"], "maitre-orlan");
    assert_eq!(body["path"], "characters/maitre-orlan.md");
    assert_eq!(body["type"], "character");
    assert_eq!(body["frontmatter"]["role"], "mentor");
    assert!(body["frontmatter"]["created"]
        .as_str()
        .unwrap()
        .ends_with("Z"));
    assert_eq!(
        body["frontmatter"]["created"], body["frontmatter"]["updated"],
        "a fresh entry has equal created/updated"
    );

    // Reachable through the read API, so the index was rebuilt.
    let read = server.get_ok("/api/v1/entities/maitre-orlan").await;
    assert_eq!(read["frontmatter"]["title"], "Maître Orlan");
    assert!(read["body"].as_str().unwrap().contains("vieux verrier"));
}

#[tokio::test]
async fn create_rejects_a_taken_slug_a_bad_title_and_an_unknown_type() {
    let server = TestServer::new();

    let (status, body) = server
        .send(
            "POST",
            "/api/v1/entities",
            &json!({ "type": "character", "title": "Aria Solane" }),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(error_code(&body), "conflict");

    let (status, _) = server
        .send(
            "POST",
            "/api/v1/entities",
            &json!({ "type": "character", "title": "   " }),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let (status, _) = server
        .send(
            "POST",
            "/api/v1/entities",
            &json!({ "type": "dragon", "title": "Smaug" }),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn update_merges_frontmatter_and_preserves_unknown_keys() {
    let server = TestServer::new();
    let (status, body) = server
        .send(
            "PATCH",
            "/api/v1/entities/aria-solane",
            &json!({ "frontmatter": { "status": "disparue" } }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["frontmatter"]["status"], "disparue");
    // A key written by another tool survives the write (non-destructive).
    assert_eq!(body["frontmatter"]["obsidian_note_id"], "91f3c0");

    // `updated` moved forward; the file on disk kept its body.
    let on_disk = std::fs::read_to_string(server.root().join("characters/aria-solane.md")).unwrap();
    assert!(on_disk.contains("status: disparue\n"), "{on_disk}");
    assert!(on_disk.contains("## Voix"), "body preserved: {on_disk}");
}

#[tokio::test]
async fn delete_removes_the_entry_and_returns_204() {
    let server = TestServer::new();
    let (status, body) = server
        .send("DELETE", "/api/v1/entities/kael-vantre", &json!({}))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(body, serde_json::Value::Null);

    let (status, _) = server.get("/api/v1/entities/kael-vantre").await;
    assert_eq!(status, StatusCode::NOT_FOUND, "the entry is gone");
}

#[tokio::test]
async fn rename_moves_the_entry_and_rewrites_breaking_links() {
    let server = TestServer::new();
    // `03-la-felure` declares `pov: "[[Aria Solane]]"` (a title link) and cites
    // her in prose. Renaming the *title* would orphan those, so they are rewritten.
    let (status, body) = server
        .send(
            "POST",
            "/api/v1/entities/aria-solane/rename",
            &json!({ "new_title": "Aria la Verrière" }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["slug"], "aria-solane",
        "slug unchanged on a title rename"
    );
    assert_eq!(body["frontmatter"]["title"], "Aria la Verrière");

    let chapter = std::fs::read_to_string(server.root().join("chapters/03-la-felure.md")).unwrap();
    assert!(
        chapter.contains("[[aria-solane|Aria Solane]]"),
        "the orphaned title link is rewritten to the slug: {chapter}"
    );
}

#[tokio::test]
async fn rename_needs_at_least_one_target() {
    let server = TestServer::new();
    let (status, _) = server
        .send("POST", "/api/v1/entities/aria-solane/rename", &json!({}))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn writing_to_an_unknown_slug_is_a_404() {
    let server = TestServer::new();
    for (method, uri) in [
        ("PATCH", "/api/v1/entities/personne"),
        ("DELETE", "/api/v1/entities/personne"),
    ] {
        let (status, _) = server.send(method, uri, &json!({})).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{method} {uri}");
    }
}

#[tokio::test]
async fn an_unknown_endpoint_returns_the_normalized_error_body() {
    let server = TestServer::new();
    let (status, body) = server.get("/api/v1/graph").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&body), "not_found");
}

#[tokio::test]
async fn routes_are_versioned() {
    let server = TestServer::new();
    // Unprefixed paths never hit the API — they fall through to the frontend
    // fallback (M6), which 404s here since test builds embed no frontend
    // (`frontend/build/` is gitignored and untracked). `get_raw`, not `get`:
    // that fallback answers plain text, not the API's JSON error shape.
    let (status, _, _) = server.get_raw("/entities").await;
    assert_eq!(status, StatusCode::NOT_FOUND, "routes live under /api/v1");
}

#[tokio::test]
async fn a_path_traversal_attempt_is_refused() {
    let server = TestServer::new();
    // Whatever the slug looks like, an entry is only ever reached through the
    // index, which knows project-relative paths only.
    let (status, _) = server.get("/api/v1/entities/..%2F..%2Fetc%2Fpasswd").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
