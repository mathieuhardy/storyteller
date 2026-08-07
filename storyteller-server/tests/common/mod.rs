//! Test helpers: a throwaway copy of the reference project, served by the real
//! router. Requests go through `oneshot`, so no port is bound.
//!
//! Compiled into each test binary, so a helper only one binary uses reads as
//! dead code in the others; that is expected, not a smell.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::Value;
use storyteller_server::events::Event;
use storyteller_server::registry::Registry;
use storyteller_server::SharedState;
use tokio::sync::broadcast;
use tower::ServiceExt;

pub struct TestServer {
    _dir: tempfile::TempDir,
    root: PathBuf,
    state: SharedState,
    router: Router,
}

impl TestServer {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("temp dir");
        let root = dir.path().join("la-felure");
        copy_dir(&fixture_source(), &root);

        // Registry lives inside the throwaway dir, so tests never read or write
        // the developer's real `~/.config/storyteller` and stay isolated from
        // each other.
        let registry = Registry::load_from(Some(dir.path().join("projects.json")));
        let state = storyteller_server::AppState::bootstrap_with_registry(&root, registry)
            .expect("open project");
        Self {
            _dir: dir,
            root,
            state: state.clone(),
            router: storyteller_server::router(state),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The shared state, for tests that drive the watcher or events directly.
    pub fn state(&self) -> SharedState {
        self.state.clone()
    }

    /// Subscribes to the change-event stream (`docs/api.md` §5).
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.state.subscribe()
    }

    /// A clone of the router, for driving a request directly (e.g. an SSE stream
    /// whose body must not be collected).
    pub fn router_clone(&self) -> Router {
        self.router.clone()
    }

    /// Issues a GET and returns the status plus the parsed JSON body.
    pub async fn get(&self, uri: &str) -> (StatusCode, Value) {
        let response = self
            .router
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .expect("response");

        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|e| panic!("{uri} returned a non-JSON body: {e}"))
        };
        (status, json)
    }

    /// Issues a GET expected to succeed.
    pub async fn get_ok(&self, uri: &str) -> Value {
        let (status, body) = self.get(uri).await;
        assert_eq!(status, StatusCode::OK, "{uri} → {body}");
        body
    }

    /// Issues a request carrying a JSON body, returning status and parsed body
    /// (`Value::Null` for an empty response, e.g. a 204).
    pub async fn send(&self, method: &str, uri: &str, body: &Value) -> (StatusCode, Value) {
        let response = self
            .router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(body).unwrap()))
                    .unwrap(),
            )
            .await
            .expect("response");

        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|e| panic!("{method} {uri} returned a non-JSON body: {e}"))
        };
        (status, json)
    }
}

fn fixture_source() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/sample-project")
}

/// A throwaway copy of the reference project under a fresh temp dir, for tests
/// that need a *second* project to switch to (`POST /projects/open`). The
/// returned `TempDir` must be kept alive for the copy to survive.
pub fn make_project(name: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().join(name);
    copy_dir(&fixture_source(), &root);
    (dir, root)
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// Slugs of a list response, in order.
pub fn slugs(body: &Value) -> Vec<&str> {
    body["items"]
        .as_array()
        .expect("items array")
        .iter()
        .map(|item| item["slug"].as_str().expect("slug"))
        .collect()
}

/// Normalized error code of an error response.
pub fn error_code(body: &Value) -> &str {
    body["error"]["code"].as_str().expect("error.code")
}
