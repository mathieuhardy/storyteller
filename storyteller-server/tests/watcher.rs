//! Watcher and change-event tests (M2, `docs/api.md` §5): writes through the API
//! announce precise `entity.*` events, and edits made *outside* the app reach
//! the index and announce `index.rebuilt`.

mod common;

use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{slugs, TestServer};
use serde_json::json;
use storyteller_server::events::Event;
use tower::ServiceExt;

/// Waits for the first event matching `name`, failing the test on timeout.
async fn wait_for(events: &mut tokio::sync::broadcast::Receiver<Event>, name: &str) -> Event {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match events.recv().await {
                Ok(event) if event.name() == name => break event,
                Ok(_) => continue,
                Err(err) => panic!("event channel closed before `{name}`: {err}"),
            }
        }
    })
    .await
    .unwrap_or_else(|_| panic!("`{name}` not received within timeout"))
}

#[tokio::test]
async fn creating_an_entry_broadcasts_entity_created() {
    let server = TestServer::new();
    let mut events = server.subscribe();

    let (status, _) = server
        .send(
            "POST",
            "/api/v1/entities",
            &json!({ "type": "character", "title": "Nouvelle Venue" }),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let event = events.try_recv().expect("a create event was broadcast");
    assert_eq!(event.name(), "entity.created");
    assert_eq!(event.data()["slug"], "nouvelle-venue");
    assert_eq!(event.data()["type"], "character");
}

#[tokio::test]
async fn updating_and_deleting_broadcast_matching_events() {
    let server = TestServer::new();
    let mut events = server.subscribe();

    let (status, _) = server
        .send(
            "PATCH",
            "/api/v1/entities/aria-solane",
            &json!({ "frontmatter": { "role": "protagoniste" } }),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let event = events.try_recv().expect("an update event");
    assert_eq!(event.name(), "entity.updated");
    assert_eq!(event.data()["slug"], "aria-solane");

    let (status, _) = server
        .send("DELETE", "/api/v1/entities/aria-solane", &json!({}))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let event = events.try_recv().expect("a delete event");
    assert_eq!(event.name(), "entity.deleted");
    assert_eq!(event.data()["slug"], "aria-solane");
}

#[tokio::test]
async fn an_external_create_reaches_the_index() {
    let server = TestServer::new();
    let mut events = server.subscribe();
    // The watcher watches the canonicalized project root the state holds.
    let root = server.state().project().root().to_path_buf();
    let _watcher = storyteller_server::watcher::spawn(server.state()).expect("watcher starts");

    // A new file appears the way Obsidian or vim would create it.
    std::fs::write(
        root.join("characters/newcomer.md"),
        "---\ntype: character\ntitle: Newcomer\n---\nArrivé de nulle part.\n",
    )
    .unwrap();

    let event = wait_for(&mut events, "index.rebuilt").await;
    assert_eq!(event.data()["reason"], "watch");

    let body = server.get_ok("/api/v1/entities?type=character").await;
    assert!(
        slugs(&body).contains(&"newcomer"),
        "external entry should be listed: {body}"
    );
}

#[tokio::test]
async fn an_external_delete_reaches_the_index() {
    let server = TestServer::new();
    let mut events = server.subscribe();
    let root = server.state().project().root().to_path_buf();
    let _watcher = storyteller_server::watcher::spawn(server.state()).expect("watcher starts");

    std::fs::remove_file(root.join("characters/aria-solane.md")).unwrap();

    wait_for(&mut events, "index.rebuilt").await;

    let (status, _) = server.get("/api/v1/entities/aria-solane").await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "the deleted entry should be gone from the index"
    );
}

#[tokio::test]
async fn the_event_stream_endpoint_is_a_text_event_stream() {
    let server = TestServer::new();
    // Do not read the (never-ending) body: status and content-type are enough.
    let response = server
        .router_clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("text/event-stream")
    );
}
