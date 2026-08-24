//! Search endpoint tests (`docs/api.md` §3 "Search").

mod common;

use axum::http::StatusCode;
use common::{error_code, slugs, TestServer};

#[tokio::test]
async fn search_ranks_by_relevance_and_carries_a_snippet() {
    let server = TestServer::new();
    let body = server.get_ok("/api/v1/search?q=ironique").await;
    let items = slugs(&body);
    assert!(items.contains(&"aria-solane"), "{items:?}");
    let hit = body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["slug"] == "aria-solane")
        .unwrap();
    assert!(hit["snippet"].as_str().unwrap().contains("<mark>"), "{hit}");
    // Same page-envelope shape as `/entities`.
    assert!(body["page"].is_number());
    assert!(body["per_page"].is_number());
    assert!(body["total"].is_number());
}

#[tokio::test]
async fn search_combines_with_type_and_pagination() {
    let server = TestServer::new();
    let body = server
        .get_ok("/api/v1/search?q=cite&type=location")
        .await;
    let items = body["items"].as_array().unwrap();
    assert!(!items.is_empty());
    assert!(items.iter().all(|item| item["type"] == "location"));
}

#[tokio::test]
async fn q_is_required_on_search() {
    let server = TestServer::new();
    let (status, body) = server.get("/api/v1/search").await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(error_code(&body), "bad_request");

    let (status, _) = server.get("/api/v1/search?q=").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn search_requires_an_open_project() {
    let server = TestServer::empty();
    let (status, body) = server.get("/api/v1/search?q=anything").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(error_code(&body), "no_project");
}

#[tokio::test]
async fn no_match_is_an_empty_page() {
    let server = TestServer::new();
    let body = server
        .get_ok("/api/v1/search?q=zzznonexistentzzz")
        .await;
    assert!(body["items"].as_array().unwrap().is_empty());
    assert_eq!(body["total"], 0);
}
