//! Project registry endpoints (`docs/api.md` §3, "Projects"): listing the known
//! projects and switching the active one at runtime.

mod common;

use axum::http::StatusCode;
use common::{error_code, make_project, TestServer};
use serde_json::json;

/// Launcher-only mode (`docs/api.md` §3): before `POST /projects/open`, every
/// route needing an active project answers a normalized `503 no_project`
/// rather than a generic 404 — distinct from `Project::open` failing on a path
/// that isn't a project at all (`opening_a_missing_folder_is_not_found`).
#[tokio::test]
async fn routes_needing_a_project_answer_503_before_one_is_open() {
    let server = TestServer::empty();
    for uri in ["/api/v1/project", "/api/v1/entities", "/api/v1/stubs"] {
        let (status, body) = server.get(uri).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{uri}: {body}");
        assert_eq!(error_code(&body), "no_project", "{uri}: {body}");
    }
}

/// The canonical form of a path, matching what the API reports as a project root.
fn canonical(path: &std::path::Path) -> String {
    std::fs::canonicalize(path)
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

#[tokio::test]
async fn projects_lists_the_active_project() {
    let server = TestServer::new();
    let body = server.get_ok("/api/v1/projects").await;

    let active = canonical(server.root());
    assert_eq!(body["active"], active);
    let items = body["items"].as_array().unwrap();
    assert!(
        items.iter().any(|item| item["path"] == active),
        "the active project is in the registry: {body}"
    );
    let record = items.iter().find(|item| item["path"] == active).unwrap();
    assert_eq!(record["name"], "la-felure");
    assert_eq!(record["entries"], 14);
}

#[tokio::test]
async fn open_switches_the_active_project() {
    let server = TestServer::new();
    let (_second_dir, second_root) = make_project("autre-roman");
    let second = canonical(&second_root);

    let (status, body) = server
        .send(
            "POST",
            "/api/v1/projects/open",
            &json!({ "path": second_root.to_str().unwrap() }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["root"], second, "open returns the new project payload");

    // The active project is now the second one, and both are remembered.
    let project = server.get_ok("/api/v1/project").await;
    assert_eq!(project["root"], second);

    let listed = server.get_ok("/api/v1/projects").await;
    assert_eq!(listed["active"], second);
    let paths: Vec<&str> = listed["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["path"].as_str().unwrap())
        .collect();
    assert!(paths.contains(&second.as_str()));
    assert!(paths.contains(&canonical(server.root()).as_str()));
    // Most-recently-opened first.
    assert_eq!(paths[0], second);
}

#[tokio::test]
async fn opening_a_missing_folder_is_not_found() {
    let server = TestServer::new();
    let (status, body) = server
        .send(
            "POST",
            "/api/v1/projects/open",
            &json!({ "path": "/does/not/exist/anywhere" }),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
    // The original project is still active — a failed open changes nothing.
    let project = server.get_ok("/api/v1/project").await;
    assert_eq!(project["root"], canonical(server.root()));
}
