# Storyteller

A local tool for storing and organizing all the **context** of a novel — its "bible": characters, locations, factions, objects, cultures, systems, species, chapters, notes, and concepts. Each project is a simple folder of markdown files, self-contained and portable, where **markdown remains the sole source of truth**.

## What it is / What it isn't

**It is** a narrative context manager: typed entries (YAML frontmatter + free-form markdown body), linked together by [wikilinks](docs/linking.md) `[[Name]]` with automatic backlinks and stub detection, viewable through filterable views.

**It is not** a word processor for writing the manuscript. Also **out of scope**: timeline/narrative chronology, interactive maps, link graph (planned for v2), real-time collaboration, authentication, AI/generation, cloud sync, guided questionnaires, and built-in versioning (left to Git). See [the vision](docs/vision.md) and [features](docs/features.md) for details.

## Status

**Pre-MVP — backend in progress.** The specs (M0) are stable and the read-only backend (M1) is in place: `storyteller-core` parses a project and builds its index, `storyteller-server` serves the read API. Next up is **M2**: non-destructive writing and the file watcher. See the [roadmap](docs/roadmap.md) for the milestone table.

There is **no interface yet** (M4) and nothing to install (M6).

### Trying the backend

```sh
cargo run -p storyteller-server -- --project tests/fixtures/sample-project
curl 'http://127.0.0.1:8787/api/v1/entities?type=character'
curl 'http://127.0.0.1:8787/api/v1/entities/aria-solane?include=backlinks'
```

Point `--project` at any folder of markdown files — a `.storyteller/` folder is created for the index, and nothing else is written. Endpoints served today are listed in [api.md](docs/api.md#implementation-status).

## Features

- **Typed, modular entities**: ~11 default types (project, character, location, faction, object, culture, system, species, chapter, note, concept), activatable/deactivatable per project.
- **Entries**: YAML frontmatter (fields driven by type) + free-form markdown body, no imposed templates.
- **Links**: wikilinks `[[Name]]`, automatic backlinks, stub detection and creation, aliases.
- **Views**: filterable lists and tables by type, backlinks panel, full-text search, sorting, and saved views.
- **Media**: images and maps stored as assets and referenced in entries.
- **Multi-project**: each project is a self-contained markdown folder.

Full details in [docs/features.md](docs/features.md).

## Technologies

- **Backend**: Rust (workspace `storyteller-core` + `storyteller-server` + `storyteller-tauri`).
- **Frontend**: SvelteKit + Shadcn (Tailwind avoided if possible).
- **Storage**: raw markdown = source of truth; disposable, rebuildable SQLite + FTS5 index ([ADR 0011](docs/adr/0011-index-sqlite-fts5.md)).
- **Deployment**: Docker and Nix flake(s).
- **Packaging**: AppImage and `.deb`; Android APK path to explore.

See [architecture](docs/architecture.md) for details.

## Installation

> Coming soon — packaging is milestone [M6](docs/roadmap.md). Until then, build from source (see [Status](#status)).

- **Docker** — coming soon (see [architecture](docs/architecture.md)).
- **Nix** — coming soon (see [architecture](docs/architecture.md)).
- **AppImage / .deb** — coming soon (see [architecture](docs/architecture.md)).

## Documentation

- [AGENTS.md](AGENTS.md) — entry router for code agents.
- [docs/vision.md](docs/vision.md) — the why and scope of the product.
- [docs/usage.md](docs/usage.md) — usage guide (screenshots coming).
- Full documentation index: see the [docs/](docs/) folder.
