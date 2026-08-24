//! Serves the built SvelteKit frontend for self-host/Docker/Nix
//! (`docs/architecture.md` §5, M6) — the outer router's fallback, for
//! anything that isn't under `/api/v1`.
//!
//! Not involved in frontend *development*: `npm run dev` proxies `/api` to
//! this server directly (`frontend/README.md`), so a developer never hits
//! this module. It only matters for a packaged build, where the SvelteKit SPA
//! (`adapter-static`, `fallback: 'index.html'`) is embedded into the binary
//! at compile time via `rust-embed`'s `debug-embed` feature — forced on
//! deliberately so behavior is identical in `cargo build` and `--release`
//! (a reproducible release, not one that silently differs by profile).
//! `frontend/build/` is gitignored and untracked, so a checkout that never
//! ran `npm run build` simply embeds nothing: `cargo test`/`cargo build`
//! still succeed (see `resolve_path`'s tests for the *logic*, decoupled from
//! whatever happens to be on disk).

use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;

// `allow_missing`: a checkout that never ran `npm run build` (every Rust-only
// dev/test workflow) has no `frontend/build/` at all — that must not be a
// compile error, just an empty embed (see the module doc comment).
#[derive(Embed)]
#[folder = "../frontend/build"]
#[allow_missing = true]
struct Assets;

/// `index.html` — served for both `/` and any client-side route the SPA
/// itself will resolve (`docs/ui/README.md`).
const INDEX: &str = "index.html";

/// The outer router's fallback (anything not matched under `/api/v1`).
pub async fn serve(uri: Uri) -> Response {
    match resolve_path(uri.path(), |path| Assets::get(path).is_some()) {
        Some(path) => {
            // `resolve_path` only ever returns a path `exists` confirmed, so
            // this file is present — barring a race with... nothing: embedded
            // assets are `'static` and never change at runtime.
            let file = Assets::get(&path).expect("resolve_path returned an existing asset");
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], file.data).into_response()
        }
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

/// Decides what to serve for a request path, given a predicate for "this
/// exact path exists among the embedded assets" — pure and independent of
/// `rust_embed`/axum, so the SPA-fallback logic is unit-tested directly
/// against a fake asset set rather than whatever the local checkout happens
/// to have built.
///
/// - An exact asset match (`/_app/immutable/chunks/xyz.js`) is served as-is.
/// - A path with no matching asset but no file extension either (a
///   client-side route like `/entry/aria-solane`) falls back to `index.html`
///   — that's what makes it an SPA.
/// - A path that looks like it wanted a real file (`/favicon.ico`, a typo'd
///   asset path) but doesn't exist is a genuine 404, not silently masked as
///   the app shell.
fn resolve_path(request_path: &str, exists: impl Fn(&str) -> bool) -> Option<String> {
    let path = request_path.trim_start_matches('/');
    let candidate = if path.is_empty() { INDEX } else { path };

    if exists(candidate) {
        return Some(candidate.to_string());
    }
    let looks_like_a_file = candidate.rsplit('/').next().is_some_and(|last| last.contains('.'));
    if looks_like_a_file {
        return None;
    }
    exists(INDEX).then(|| INDEX.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn assets(paths: &[&str]) -> HashSet<String> {
        paths.iter().map(|p| p.to_string()).collect()
    }

    #[test]
    fn serves_an_exact_asset_match() {
        let files = assets(&["index.html", "_app/immutable/chunks/xyz.js"]);
        assert_eq!(
            resolve_path("/_app/immutable/chunks/xyz.js", |p| files.contains(p)),
            Some("_app/immutable/chunks/xyz.js".to_string())
        );
    }

    #[test]
    fn root_serves_index_html() {
        let files = assets(&["index.html"]);
        assert_eq!(resolve_path("/", |p| files.contains(p)), Some("index.html".to_string()));
    }

    #[test]
    fn a_client_side_route_falls_back_to_index_html() {
        let files = assets(&["index.html", "_app/immutable/chunks/xyz.js"]);
        assert_eq!(
            resolve_path("/entry/aria-solane", |p| files.contains(p)),
            Some("index.html".to_string())
        );
        assert_eq!(
            resolve_path("/type/character", |p| files.contains(p)),
            Some("index.html".to_string())
        );
    }

    #[test]
    fn a_missing_file_like_path_is_not_masked_as_the_app_shell() {
        let files = assets(&["index.html"]);
        assert_eq!(resolve_path("/favicon.ico", |p| files.contains(p)), None);
        assert_eq!(
            resolve_path("/_app/immutable/chunks/does-not-exist.js", |p| files.contains(p)),
            None
        );
    }

    #[test]
    fn no_embedded_frontend_is_a_clean_404_everywhere() {
        assert_eq!(resolve_path("/", |_| false), None);
        assert_eq!(resolve_path("/entry/aria-solane", |_| false), None);
    }
}
