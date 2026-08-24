//! Asset endpoint tests (`docs/api.md` §3, "Assets").

mod common;

use axum::http::StatusCode;
use common::{error_code, TestServer};

#[tokio::test]
async fn lists_files_under_assets_including_non_images() {
    let server = TestServer::new();
    let body = server.get_ok("/api/v1/assets").await;
    let assets = body.as_array().unwrap();
    // The fixture ships a plain README under `assets/images/` — proof that the
    // index's own "skip assets/" rule (`docs/roadmap.md` M1) and this listing
    // are two different things: entries never live there, but any file does.
    let readme = assets
        .iter()
        .find(|a| a["path"] == "assets/images/README.md")
        .unwrap_or_else(|| panic!("{assets:?}"));
    assert_eq!(readme["kind"], "file");
    assert!(readme["size"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn serves_an_uploaded_asset_with_a_guessed_content_type() {
    let server = TestServer::new();
    server
        .post_multipart("/api/v1/assets", "aria-solane.jpg", b"fake-jpeg-bytes")
        .await;

    let (status, content_type, bytes) =
        server.get_raw("/api/v1/assets/assets/aria-solane.jpg").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type, "image/jpeg");
    assert_eq!(bytes, b"fake-jpeg-bytes");
}

#[tokio::test]
async fn a_missing_asset_is_a_normalized_404() {
    let server = TestServer::new();
    let (status, body) = server.get("/api/v1/assets/assets/images/nope.jpg").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&body), "not_found");
}

#[tokio::test]
async fn serving_outside_assets_is_refused_even_for_a_real_file() {
    let server = TestServer::new();
    let (status, body) = server.get("/api/v1/assets/../project.md").await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
}

#[tokio::test]
async fn upload_saves_the_file_and_broadcasts_assets_changed() {
    let server = TestServer::new();
    let mut events = server.subscribe();

    let (status, body) = server
        .post_multipart("/api/v1/assets", "portrait.png", b"fake-png-bytes")
        .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["path"], "assets/portrait.png");

    let (status, content_type, bytes) = server.get_raw("/api/v1/assets/assets/portrait.png").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type, "image/png");
    assert_eq!(bytes, b"fake-png-bytes");

    let event = events.recv().await.unwrap();
    assert_eq!(event.name(), "assets.changed");
    assert_eq!(event.data()["path"], "assets/portrait.png");
    assert_eq!(event.data()["change"], "added");
}

#[tokio::test]
async fn upload_refuses_to_overwrite_an_existing_asset() {
    let server = TestServer::new();
    let (first_status, first_body) = server
        .post_multipart("/api/v1/assets", "dup.png", b"one")
        .await;
    assert_eq!(first_status, StatusCode::CREATED, "{first_body}");

    let (second_status, second_body) = server
        .post_multipart("/api/v1/assets", "dup.png", b"two")
        .await;
    assert_eq!(second_status, StatusCode::CONFLICT, "{second_body}");
    assert_eq!(error_code(&second_body), "conflict");
}

#[tokio::test]
async fn upload_flattens_a_path_like_filename() {
    let server = TestServer::new();
    let (status, body) = server
        .post_multipart("/api/v1/assets", "../../evil.png", b"x")
        .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["path"], "assets/evil.png");
}

#[tokio::test]
async fn assets_endpoints_require_an_open_project() {
    let server = TestServer::empty();
    let (status, body) = server.get("/api/v1/assets").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(error_code(&body), "no_project");

    let (status, _) = server
        .post_multipart("/api/v1/assets", "x.png", b"x")
        .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}
