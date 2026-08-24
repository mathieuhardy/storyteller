# ADR 0016 — The built frontend is embedded into `storyteller-server` at compile time

- **Status**: Accepted
- **Date**: 2026-08-24

## Context

M6 packages `storyteller-server` for self-host (Docker) and Nix (`docs/architecture.md` §5): "`storyteller-server` + built front" served together over HTTP. The server needs to serve the SvelteKit SPA's static output (`frontend/build/`, `adapter-static`) alongside the JSON API, for anyone hitting the container/binary in a browser.

Two ways to get the static files in front of the server at runtime:

1. **A runtime directory**: the binary reads `frontend/build/` from a path on disk (a flag/env var), served via `tower-http`'s `ServeDir`.
2. **Compile-time embedding**: the static files are baked into the binary itself, served from memory.

## Decision

**Embed the built frontend into the `storyteller-server` binary at compile time**, via `rust-embed` (`#[derive(Embed)] #[folder = "../frontend/build"]`, `storyteller-server/src/frontend.rs`). The `debug-embed` feature is forced on so this happens identically in `cargo build` and `--release` — a build's behavior shouldn't depend on which profile compiled it.

`#[allow_missing = true]` keeps this from breaking the ordinary Rust-only workflow: a checkout that never ran `npm run build` (every `cargo test`/`cargo build` outside of packaging) simply embeds nothing, and the outer router's frontend fallback then 404s cleanly — it does not fail to compile, and it does not need a frontend to exist for the *Rust* half of the project to build and test.

The router splits accordingly: the API sub-router gets its own explicit `.fallback` (a JSON 404, for anything unmatched under `/api/v1`), and the *outer* router's fallback serves the frontend — an exact asset match as-is, a path with no extension and no match as `index.html` (the SPA takes over client-side), anything else a plain 404. That dispatch logic is a pure function (`resolve_path`) independent of `rust_embed`, unit-tested against a fake asset set rather than whatever happens to be built locally.

## Consequences

- **One artifact, matching "a reproducible release is produced."** The Docker image's runtime stage is just the compiled binary — no second layer/volume carrying static files that could drift out of sync with the API version serving them. The Nix package is a single derivation output for the same reason.
- **Build order is now real: frontend before backend.** `npm run build` must run before `cargo build -p storyteller-server` for the embed to pick up real content — both the Dockerfile (multi-stage: a `node` stage feeds its `frontend/build` into the `rust` stage before compiling) and the Nix flake (a `buildNpmPackage` output copied into place via `postPatch` before `buildRustPackage` compiles) encode this explicitly.
- **A rebuild-detection gap for local packaging testing.** `rust-embed`'s macro reads a directory, not a single tracked file; if `frontend/build/` changes (e.g. rebuilt) without any Rust source file changing, `cargo build` may not notice and needs a touched file (`touch storyteller-server/src/frontend.rs`) to force re-embedding. Irrelevant for Docker/Nix (each is a fresh build), but worth knowing when testing a packaged binary by hand.
- **The existing test suite runs before any frontend is built** (`frontend/build/` is gitignored, never committed), so `routes_are_versioned` (`storyteller-server/tests/api.rs`) documents and relies on the empty-embed case; a Nix package build disables `cargo test` for the same reason (a *real* frontend would be staged there, invalidating that one assertion for a reason unrelated to the package's correctness) — see `flake.nix`'s comment.

## Alternatives Considered

- **Serve from a runtime directory** (`tower-http::services::ServeDir` + a `--static-dir`/env var). Rejected: two artifacts (binary + a files directory) to keep paired and versioned together in the container/Nix output, for no MVP benefit — this is a single-operator self-host tool (ADR 0003), not a CDN-fronted service that would want to update static assets independently of the binary.
- **`include_dir!`** instead of `rust-embed`. Equivalent in spirit (compile-time embedding); `rust-embed` was chosen for its built-in `allow_missing` escape hatch, which is precisely what keeps the ordinary Rust-only dev/test workflow from acquiring a hard dependency on the frontend having been built.
