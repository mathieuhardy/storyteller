# Architecture

Technical outline of **Storyteller**, at the *outline* level: it fixes the main components and their responsibilities, without locking in implementation details. Undecided points are grouped in [Open Technical Questions](#6-open-technical-questions--adr) and referred to ADRs.

For the on-disk format, see [data model](data-model.md). For the exact endpoint contracts, see [API](api.md).

> Cardinal principle governing the entire architecture: **markdown is the sole source of truth**. The [index](glossary.md) is just a disposable cache, 100% rebuildable from the files. App writes are **non-destructive** (see [principles](principles.md)).

---

## 1. Overview

Storyteller is composed of a **shared Rust core**, exposed in two ways (an HTTP server for self-hosting, a Tauri webview for desktop/mobile), and a **single SvelteKit front** that consumes the same API in both cases. The [source of truth](glossary.md) always remains the markdown [project](glossary.md) folder; the SQLite [index](glossary.md) is just a local cache.

```
                        ┌──────────────────────────────┐
                        │   SvelteKit + Shadcn Front    │
                        │   (same views, same API)      │
                        └──────────────┬───────────────┘
                                       │  API (see api.md)
              ┌────────────────────────┴────────────────────────┐
              │                                                  │
   ┌──────────▼───────────┐                        ┌─────────────▼────────────┐
   │  storyteller-server  │                        │     storyteller-tauri     │
   │  HTTP binary         │                        │  Tauri v2 webview         │
   │  → Docker / self-host│                        │  → AppImage / .deb / APK  │
   └──────────┬───────────┘                        └─────────────┬────────────┘
              │                                                  │
              └────────────────────┬─────────────────────────────┘
                                   │  shared dependency
                       ┌───────────▼────────────┐
                       │    storyteller-core     │
                       │  parsing · links ·      │
                       │  index · watcher ·      │
                       │  search · non-destructive│
                       │  writing                │
                       └───────────┬─────────────┘
                                   │ reads / writes
          ┌────────────────────────┴─────────────────────────┐
          │  Project folder (SOURCE OF TRUTH)                 │
          │  *.md + assets/   .storyteller/config.yaml        │
          │  .storyteller/cache.sqlite  (index, gitignored)   │
          └───────────────────────────────────────────────────┘
```

Two deployment targets, **one core, one front**:

- **Server mode**: `storyteller-server` serves the API + built front; browser access, ideal for Docker / self-host.
- **Webview mode**: `storyteller-tauri` embeds the front and talks to core locally; produces desktop packages and a mobile path.

---

## 2. Rust Backend (workspace)

The backend is a **Cargo workspace** with three members. All business logic lives in the lib; the two binaries are just transport wrappers.

| Crate                 | Nature            | Role | Target |
|-----------------------|-------------------|------|--------|
| `storyteller-core`    | lib               | business logic | (consumed by binaries) |
| `storyteller-server`  | HTTP binary       | exposes API over network | Docker / self-host / browser |
| `storyteller-tauri`   | binary (Tauri v2) | embeds front + core in webview | AppImage / .deb / APK |

### storyteller-core (lib)

Library with no network I/O or UI. Responsibilities:

- **Parsing**: splits [frontmatter](glossary.md) YAML + [body](glossary.md) markdown; body remains free-form (no templates).
- **[Wikilink](glossary.md) extraction**: detection of `[[Target]]`, `[[Target|text]]`, images `![[...]]`, and links carried by frontmatter fields (see [linking](linking.md)).
- **Indexing**: building/updating the SQLite cache (entries, fields, links, [backlinks](glossary.md), [stubs](glossary.md)).
- **[Watcher](glossary.md)**: monitoring the project folder via the [`notify`](https://crates.io/crates/notify) crate → incremental re-indexing on external change (Obsidian, vim, mobile sync).
- **Search**: filtering/sorting queries and full-text search on top of the index.
- **Non-destructive writing**: serialization that **preserves unknown YAML keys, key order, and body** as-is (see [principles](principles.md) and [data model](data-model.md)).

`core` remains transport-agnostic: it knows neither HTTP nor Tauri, ensuring identical behavior in both modes.

### storyteller-server (HTTP binary)

HTTP wrapper on top of `core`. Serves the API (see [api.md](api.md)) and, in production, the statically built SvelteKit front. This is the target for the Docker container and self-host "browser access."

### storyteller-tauri (webview binary)

**Tauri v2** wrapper on top of the same `core`. Embeds the front in a webview and provides the native app experience (desktop, then Android path). Target for AppImage/.deb packages and the APK spike.

> The transport mode between front and core in Tauri context (local HTTP vs. IPC commands) is undecided → see §6 and ADR.

---

## 3. SvelteKit + Shadcn Frontend

**One front** SvelteKit, target-independent:

- Consumes the **same API** (see [api.md](api.md)) whether running in server mode (HTTP fetch to `storyteller-server`) or Tauri webview mode.
- Provides the MVP [views](glossary.md): filterable lists/tables by [type](glossary.md), [backlinks](glossary.md) panel, [entry](glossary.md) editor (frontmatter + markdown body). No graph in MVP (v2).
- Built statically for Docker (served by the server) or embedded in the Tauri bundle.

> **Resolved:** the Shadcn/Tailwind question is settled — Tailwind is adopted, contained by component classes ([ADR 0013](adr/0013-tailwind-with-component-classes.md)). The GUI design (["Atelier"](ui/README.md)) and the M4 scaffold followed this doc, as intended.

---

## 4. Index & Search

- **Engine**: **SQLite + FTS5**, stored in `.storyteller/cache.sqlite` — decided in [ADR 0011](adr/0011-index-sqlite-fts5.md). File is **gitignored, disposable, regenerable** at any time from the markdown (never a source of truth). The app writes `.storyteller/.gitignore` itself so the project folder stays committable as-is.
- **Index contents**: entries and their frontmatter fields, outgoing links, backlinks, stubs, plus an FTS table for full-text search (title, aliases, tags, and body).
- **Incremental reindexing**: the [watcher](glossary.md) (`notify`) triggers updates; we only reindex what changed, detected by **mtime + content hash**. Full rebuild possible cold (first launch, corrupted cache, `schema_version` change).

> SQLite schema details are an implementation matter (see `storyteller-core/src/index/schema.rs`) and carry **no migration guarantee**: a cache written by another layout is dropped and rebuilt. The on-disk format remains defined by [data-model.md](data-model.md).

---

## 5. Packaging & Deployment

| Target | Contents | Notes |
|--------|----------|-------|
| **Docker** | `storyteller-server` + built front, embedded ([ADR 0016](adr/0016-embed-frontend-in-server-binary.md)) | self-host image, browser access — **MVP (M6) ✅** |
| **Nix flake** | `packages.default` / `nix run` — same embedded-frontend binary | reproducible "native" install — **MVP (M6) ✅** |
| **AppImage / .deb** | Tauri bundle (`storyteller-tauri`) | desktop packaging via Tauri bundler — **v2 (M7)** |
| **Android APK** | Tauri mobile bundle | **feasibility spike, out of MVP (M7)** |

See the [roadmap](roadmap.md): milestone M6 = MVP packaging (Docker/Nix); M7 = desktop packaged (AppImage/.deb) and APK spike.

---

## 6. Open Technical Questions (→ ADR)

To be decided in [ADR](adr/README.md); do not prejudge here.

1. ~~**Shadcn without Tailwind**~~ — **Resolved** by [ADR 0013](adr/0013-tailwind-with-component-classes.md): Tailwind is adopted, contained by component classes.
2. **Primary target** — Tauri app vs. headless server: which one drives product/UX tradeoffs by default?
3. **Entity identity** — filename/title/alias (current default) vs. stable `id` field (hardening path). The **link rewriting policy** on rename is decided ([ADR 0012](adr/0012-rename-link-rewriting.md): rewrite breaking links to the new filename, keep no alias); the stable-`id` question stays open. See [linking](linking.md) and [data model](data-model.md).
4. ~~**Markdown rendering**~~ — **Resolved** by [ADR 0015](adr/0015-markdown-rendering-in-core.md): rendering happens in `core` (`pulldown-cmark` + wikilink/embed splicing), exposed via `?render=html`.
5. **Transport in Tauri mode** — local HTTP vs. Tauri IPC between front and core.

Decided since this document was first written: **index engine** → [ADR 0011](adr/0011-index-sqlite-fts5.md) (SQLite + FTS5); **rename link-rewriting policy** → [ADR 0012](adr/0012-rename-link-rewriting.md); **markdown rendering** → [ADR 0015](adr/0015-markdown-rendering-in-core.md) (in `core`).
