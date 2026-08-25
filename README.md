# Storyteller

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A local tool for storing and organizing all the **context** of a novel — its "bible": characters, locations, factions, objects, cultures, systems, species, chapters, notes, and concepts. Each project is a simple folder of markdown files, self-contained and portable, where **markdown remains the sole source of truth**.

## What it is / What it isn't

**It is** a narrative context manager: typed entries (YAML frontmatter + free-form markdown body), linked together by [wikilinks](docs/linking.md) `[[Name]]` with automatic backlinks and stub detection, viewable through filterable views, a full-text search, and a link graph — plus your own custom entry types when the 11 built-ins aren't enough.

**It is not** a word processor for writing the manuscript. Also **out of scope**: timeline/narrative chronology, interactive maps, real-time collaboration, authentication, AI/generation, cloud sync, guided questionnaires, and built-in versioning (left to Git). See [the vision](docs/vision.md) and [features](docs/features.md) for details.

## Status

**MVP complete (M0–M6), and most of v2 (M7) too.** `storyteller-core` parses a project non-destructively, resolves wikilinks, and builds a rebuildable SQLite+FTS5 index; `storyteller-server` exposes the full read/write/link/search API plus a file watcher; the SvelteKit frontend covers the dashboard, list/table views, entry detail, the editor, the links workshop, the media gallery, the link graph, and search. The application is packaged for self-host (Docker), installable via Nix, and packaged as a desktop app (AppImage/`.deb`, via `storyteller-tauri`). See the [roadmap](docs/roadmap.md) for the milestone table.

Left in M7: an Android APK spike. See the roadmap's ["Left Aside & Known Gaps"](docs/roadmap.md#left-aside--known-gaps) for smaller follow-ups and ideas under investigation.

### Running it

```sh
cargo run -p storyteller-server -- --project tests/fixtures/sample-project
curl 'http://127.0.0.1:8787/api/v1/entities?type=character'
curl 'http://127.0.0.1:8787/api/v1/entities/aria-solane?include=backlinks'
```

Point `--project` at any folder of markdown files — a `.storyteller/` folder is created for the index, and nothing else is written. Endpoints served today are listed in [api.md](docs/api.md#implementation-status).

For the frontend during development, see [frontend/README.md](frontend/README.md) — a dev server proxying to the backend above. For a packaged, self-contained build (API + built frontend in one binary), see [Installation](#installation) below.

## Features

- **Typed, modular entities**: ~11 default types (project, character, location, faction, object, culture, system, species, chapter, note, concept), activatable/deactivatable per project — plus **custom types** of your own, declared in `.storyteller/types.yaml`.
- **Entries**: YAML frontmatter (fields driven by type) + free-form markdown body, no imposed templates.
- **Links**: wikilinks `[[Name]]`, automatic backlinks, stub detection and creation, aliases, autocompletion in the editor.
- **Views**: filterable lists and tables by type, backlinks panel, full-text search, sorting, and saved views.
- **Link graph**: a force-directed view of every entry and how they connect, for exploring the whole network at once.
- **Media**: images and maps stored as assets and referenced in entries, plus a gallery for browsing them visually.
- **Multi-project**: each project is a self-contained markdown folder.
- **Runs your way**: browser-hosted (self-host), or as a native desktop app — same API, same frontend, either way.

Full details in [docs/features.md](docs/features.md).

## Technologies

- **Backend**: Rust (workspace `storyteller-core` + `storyteller-server` + `storyteller-tauri`).
- **Frontend**: SvelteKit + Tailwind, contained by component classes ([ADR 0013](docs/adr/0013-tailwind-with-component-classes.md)) — utilities stay inside components, not scattered across markup.
- **Storage**: raw markdown = source of truth; disposable, rebuildable SQLite + FTS5 index ([ADR 0011](docs/adr/0011-index-sqlite-fts5.md)).
- **Deployment**: Docker and Nix flake(s) for self-host; a native desktop app (`storyteller-tauri`) that runs the exact same HTTP API in-process ([ADR 0019](docs/adr/0019-tauri-reuses-the-http-router.md)).
- **Packaging**: AppImage and `.deb` (built and verified via `cargo tauri build`); Android APK path still to explore.

See [architecture](docs/architecture.md) for details.

## Installation

- **Docker** — self-host, one image bundling the API and the built frontend (`storyteller-server/src/frontend.rs`):

  ```sh
  docker build -t storyteller .
  docker run -p 8787:8787 -v /path/to/your/novel:/data storyteller
  ```

  or `PROJECT_DIR=/path/to/your/novel docker compose up --build`. Open `http://localhost:8787`.

- **Nix** — `nix build` / `nix run` (flake, `x86_64-linux`); see [flake.nix](flake.nix). The frontend's `npmDepsHash` is a placeholder until someone runs it once against network access to fill in the real hash — standard for a first Nix packaging pass.
- **AppImage / .deb** — desktop packages of `storyteller-tauri`, built with `cargo tauri build`:

  ```sh
  cargo install tauri-cli --version "^2.0" --locked   # one-time
  cd frontend && npm run build && cd ..
  cd storyteller-tauri && cargo tauri build
  ```

  Artifacts land under `target/release/bundle/{appimage,deb}/`. See
  [CONTRIBUTING.md](CONTRIBUTING.md#desktop-app-storyteller-tauri) for the Linux system
  dependencies this needs (webkit2gtk, gtk3, `patchelf`, …). Not yet published anywhere — build it
  yourself for now; see the roadmap's CI packaging item for the plan to change that.

## Documentation

- [AGENTS.md](AGENTS.md) — entry router for code agents, read-first order for the specs.
- [CONTRIBUTING.md](CONTRIBUTING.md) — building from source (Rust workspace, frontend, Docker, Nix, desktop app).
- [docs/vision.md](docs/vision.md) — the why and scope of the product.
- [docs/usage.md](docs/usage.md) — usage guide (screenshots coming).
- [docs/roadmap.md](docs/roadmap.md) — milestones, what's left, and ideas under investigation.
- Full documentation index: see the [docs/](docs/) folder.

## License

[MIT](LICENSE).
