# Roadmap — Milestones & Dependencies

This document orders **Storyteller** work into milestones (M0→M7), with their dependencies and *Definition of Done* (DoD). For the functional detail of what is covered at each stage, see [features](features.md).

Framework reminder: [markdown is the sole source of truth](principles.md), the [index](glossary.md) is a disposable cache, and the GUI is designed **after** this documentation. The milestones below reflect this order.

## Overview

| Milestone | State | Content | Depends on | Definition of Done |
|---------|-------|---------|------------|-------------------|
| **M0** | ✅ done | Specs: all design documentation (vision, principles, data model, linking, architecture, API, glossary, roadmap, usage, ADR). | — | Docs cover data model, links, and API without ambiguity; locked decisions are traced in ADR; a code agent can start M1 without blocking questions. |
| **M1** | ✅ done | `storyteller-core` + index + **read-only** API: frontmatter/body parsing, type catalog, [index](glossary.md) building (`cache.sqlite`), read endpoints (list by type, entry, backlinks). | M0 | The core parses a real project without loss; the index rebuilds entirely from files; the API exposes entries, their typed fields, and backlinks in read mode; the index is rebuildable and gitignored. |
| **M2** | ✅ done | Entry CRUD: **non-destructive** writing (unknown YAML keys, key order, and body preserved), creation, update, deletion, **rename** with link updates, **watcher** for file→index sync. | M1 | Create/edit/delete an entry via API modifies markdown without breaking external edits; rename updates links and/or keeps old title as [alias](glossary.md); watcher reindexes external changes. |
| **M3** | ✅ done | Full links: [wikilink](glossary.md) resolution (file → title → alias), automatic [backlinks](glossary.md), [stub](glossary.md) detection, ambiguity handling, **create entry from stub**. | M1, M2 | Backlinks are exact and bidirectional; stubs are listed; resolution ambiguity is signaled; promoting a stub creates a real entry via M2 CRUD and resolves the link. |
| **M4** | ✅ done | SvelteKit + Shadcn frontend: filterable list/table [views](glossary.md) by type, backlinks panel, entry editor (frontmatter + body). | M1, M2, M3 | GUI consumes a stable M1–M3 API; navigate, filter, and edit entries; links and backlinks are clickable; no API bypass on front side. |
| **M5** | ✅ done | FTS search + advanced filters/sort + **saved views**. | M1 (index), M4 | Full-text search queries the index; filters and sorts combine; a view can be named, saved, and reloaded. |
| **M6** | ✅ done | Packaging & self-host **MVP**: `storyteller-server` distributed via **Docker** and installable via **Nix**. | M4 | The application launches in self-host Docker and installs via Nix; index rebuilds; a reproducible release is produced. |
| **M7** | — | v2+: packaged desktop (`storyteller-tauri` → AppImage/.deb), link graph, media gallery, custom types, Android APK spike. | M6 | Each v2 item is scoped (spec or spike); none blocks the M1–M6 MVP. |

### M1 — What Landed

`storyteller-core` (parsing, type catalog, normalization, link resolution, SQLite index) and `storyteller-server` (read endpoints). The engine choice is recorded in [ADR 0011](adr/0011-index-sqlite-fts5.md); the exact endpoint coverage is tabulated in [api.md](api.md#implementation-status).

Two points from M3 were **necessarily** built early, because backlinks cannot be computed without them: ranked target resolution (filename → `title` → `aliases`) and stub/ambiguity classification. They live in `core` and are covered by tests; what M3 still owned was the **exposure** (`GET /entities/{slug}/links`, `GET /stubs`) and **creation from a stub**, delivered in M3 (below).

### M2 — What Landed

Non-destructive entry CRUD and rename: `POST`/`PATCH`/`DELETE /entities` and `POST /entities/{slug}/rename`, preserving unknown YAML keys, key order and untouched body, with rename rewriting only the links that would otherwise break ([ADR 0012](adr/0012-rename-link-rewriting.md)). After each write the index is rebuilt so the change is immediately visible.

**Watcher (was deferred within M2, now landed):** the [watcher](glossary.md) closes the file→index sync on *external* edits and the [SSE stream](api.md#5-event-stream-sse). An in-memory snapshot re-parses only the files whose bytes moved (mtime + content hash), so an edit from Obsidian, vim or a `git pull` reindexes incrementally and is announced as `index.rebuilt`; writes through the API announce the matching `entity.*` event. `GET /events` serves the stream. This completes M2's Definition of Done.

### M3 — What Landed

Link exposure on top of M1's resolver: `GET /entities/{slug}/links` (outgoing links, each `resolved` / `stub` / `ambiguous`, with candidate slugs when ambiguous) and `GET /stubs` (unresolved targets grouped by normalized key, with their sources). **Creation from a stub** reuses M2's `POST /entities`: creating the entry with its `title` pre-filled from the link text makes the link resolve at the next rebuild, with **no source file rewritten** ([linking.md](linking.md) §6.3).

### M4 — What Landed

The **first slice** of the GUI. Delivered:

- **Backend registry**: `GET /projects` and `POST /projects/open` — the recent-projects registry (a machine preference in the OS config dir, not inside any project) and **runtime switching** of the active project (rescan + index rebuild + watcher restart). Recorded in [api.md](api.md#implementation-status).
- **Launcher-only mode**: the server can now start without `--project`; routes return `503 no_project` until a project is opened via `POST /projects/open`. This enables first-time user experience.
- **Frontend scaffold** (`frontend/`): SvelteKit SPA (`adapter-static`, `ssr` off) + Tailwind v4 carrying the ["Atelier" tokens](ui/foundations.md) in both themes ([ADR 0013](adr/0013-tailwind-with-component-classes.md)); a transport-agnostic API client, an [SSE](api.md#5-event-stream-sse) client, and the EN/FR i18n catalog ([ADR 0014](adr/0014-ui-i18n-front-catalog.md)).
- **App shell + launcher**: the topbar / type-nav / collapsible links-rail shell and the project launcher, consuming `/projects`, `/project` and `/types`. See [frontend/README.md](../frontend/README.md).
- **Dashboard screen**: hero section (project cover, title, logline, status, genres), stats grid (entries, chapters, stubs, errors), recent entries card, distribution by type chart, stubs preview, and index health indicator with watcher pulse. Components in `frontend/src/lib/components/dashboard/`.
- **List/table screen** (`/type/{type}`): Table and List views of a type's entries, entirely URL-driven (`sort=`, `tag=`, `<field>=`, `page=`, `per_page=`) so the toolbar, filter chips and pager just read/rewrite the query string. Sortable column headers plus a toolbar sort menu, an "add a filter" popover generated from the type's field schema (enum/boolean fields offer their values directly), removable filter chips, and pagination. `EntrySummary` gained an `updated` timestamp (`docs/api.md` §3) so the Modified column and default `sort=-updated` have real data; per-type columns beyond that (role, status…) need a richer list endpoint and are deferred. Components in `frontend/src/lib/components/list/`.
- **Entry detail screen** (`/entry/{slug}`): header, diagnostic banner, schema-driven attributes with a preserved-fields block, rendered body (`?render=html`, below), and the rail panel (backlinks, outgoing links, metadata). Components in `frontend/src/lib/components/entry/`.
- **Entry editor** (`/entry/{slug}/edit`, `/type/{type}/new`): creation and edition, form generated from the type schema, link autocomplete with inline stub creation, markdown body editor (toolbar + edit/preview — the preview stays a raw-text stopgap deliberately: it previews *unsaved* text, which `?render=html` can't see), and a save that diffs against the loaded entry so only changed fields are sent. Components in `frontend/src/lib/components/editor/`.
- **Links workshop** (`/stubs`): stubs tab (create-from-stub modal, type picker) and an ambiguous-links tab (built by aggregating `GET /entities/{slug}/links` across every entry, since there is no dedicated aggregate endpoint) with non-destructive disambiguation rewrites. Components in `frontend/src/lib/components/workshop/`.
- **`PATCH /types/{type}`**: enable/disable a type for creation, persisted to `.storyteller/config.yaml` (`ProjectConfig::save`, `Project::set_type_enabled`); existing entries of a disabled type stay untouched and indexed. `Active`'s `Project` moved behind a `Mutex` (matching `snapshot`/`index`) to allow this in-place mutation. No dedicated settings screen consumes it yet — the client function (`setTypeEnabled`) is exposed for one to pick up later.
- **`/assets` endpoints**: list every file under `assets/`, serve one with a guessed `Content-Type`, and upload via `multipart/form-data` (flattened to a bare filename, `409` on collision), broadcasting `assets.changed`. New `storyteller-core::assets` module; no image-dimension dependency added (`width`/`height` stay unpopulated). Along the way, fixed a real gap this surfaced: routes guarded by `require_project` were answering a generic `404` instead of the documented `503 no_project` in launcher-only mode — `AppState::require_project` now returns that directly.
- **Editor image fields**: with `/assets` landed, the entry editor's `image`/`image-list` fields (`cover`, `portrait`, `map`, `attachments`…) are no longer excluded — `ImageFieldEditor.svelte` uploads and shows thumbnails. Closes the gap flagged when the editor screen first landed.
- **Markdown rendering** (`?render=html`): the last open item, resolved by [ADR 0015](adr/0015-markdown-rendering-in-core.md) — rendering happens in `core` (new `render` module, `pulldown-cmark`), so wikilink resolution is never duplicated in JS. Wikilinks/embeds are spliced to `<a>`/`<span>`/`<img>` at their exact byte spans before parsing; resolution is read from the already-indexed outgoing links, not a fresh project-wide scan. The entry detail screen's body now injects this HTML (`BodySection.svelte`), replacing its raw-text wikilink highlighter — this closes M4.

M4 is done.

### M5 — What Landed

- **Full-text search**: `entries_fts`, a new SQLite FTS5 virtual table (ADR 0011's engine choice), indexing title/aliases/tags/body — deliberately not arbitrary frontmatter, matching [api.md](api.md#3-search)'s own scope. Accent-insensitive (`remove_diacritics 2`, content is French per [ADR 0010](adr/0010-frontmatter-keys-en-content-fr.md)); free text is turned into a per-term, individually-quoted prefix match (`"word"*`), which is both forgiving of partial typing and safe against FTS5 query-syntax injection from arbitrary input. The index schema bumped to v2 so an existing on-disk cache gets rebuilt with the new table rather than silently missing it.
- **`GET /search?q=…`**: ranked by relevance (FTS5 `rank`) unless an explicit `sort=` is given; combines with the usual `type`/`tag`/`<field>` filters and pagination; returns each hit with a highlighted `snippet`. **`q` on `GET /entities`** restricts the list the same way, without ranking or a snippet — both share `ListQuery`/`build_filters`.
- **Search screen** (`/search`): the topbar's search box (previously disabled, "M5" badge) now submits here. Results ranked by relevance with highlighted snippets, combinable with the same URL-driven filters as the list/table screen, paginated with the same `Pager`.
- **Saved views**: "a view = a persisted set of parameters on the frontend side" ([api.md](api.md#4-filtering-sorting-pagination)) — `SavedViewsMenu.svelte` on the list/table screen's toolbar names and stores the current `sort=`/`tag=`/`<field>=` combination in `localStorage` (a UI preference, like the theme/language, never written to the project), and reloads it later. `docs/features.md` §D previously listed this as v2; updated to MVP to match this milestone's actual DoD.

M5 is done.

### M6 — What Landed

- **Frontend embedded in the server binary** ([ADR 0016](adr/0016-embed-frontend-in-server-binary.md)): `storyteller-server/src/frontend.rs` uses `rust-embed` to bake `frontend/build/` (the built SvelteKit SPA) into the binary at compile time, forced identical across debug/release via `debug-embed`. `#[allow_missing]` keeps this from requiring a frontend build for ordinary `cargo build`/`cargo test`. The router's outer fallback serves it (exact asset, or `index.html` for a client-side route); the API sub-router got its own explicit fallback so unmatched `/api/v1/*` paths stay a JSON `404` rather than falling through to the SPA shell.
- **`Dockerfile`**: three stages — `node` builds the frontend, `rust:1-bookworm` compiles the server with that frontend staged into place first, a `debian:bookworm-slim` runtime stage holds only the resulting binary (~126 MB image). Runs as root deliberately: `/data` is an arbitrary host-bind-mounted project folder owned by whatever uid the operator has, and a fixed non-root image uid would only be able to write it by coincidence — same single-operator trust boundary as running the binary directly (ADR 0003), the container adds packaging, not a security boundary between users. `docker-compose.yml` for a one-line `PROJECT_DIR=… docker compose up --build`. Built and run end-to-end against the sample fixture during development (index rebuild, entry create/read, search, `?render=html`, SPA routes, real API 404s) — not just a syntax check.
- **`flake.nix`**: added `packages.default`/`storyteller-server`/`frontend` and `apps.default` (`nix run`) alongside the pre-existing dev shell, same embed-before-compile structure as the Dockerfile (`buildNpmPackage` output copied into place via `postPatch` before `buildRustPackage`). **Not verified end-to-end** — this session has no `nix` binary available to actually run `nix build`. `frontend`'s `npmDepsHash` is `pkgs.lib.fakeHash`, a deliberate placeholder: a real value can only come from Nix actually fetching `frontend/package-lock.json`'s dependency tree, which needs network access this session doesn't have either. Whoever runs `nix build` first will see the mismatch error report the real hash to paste in — the standard, expected way to fill this in.

M6 is done (Docker verified live; Nix authored to standard nixpkgs patterns but unverified — see above).

## Critical Path

The milestone order is not arbitrary: each stage builds on invariants set by the previous one.

- **M1 is the foundation.** Markdown parsing and the [index](glossary.md) condition everything else; nothing solid gets built until the core faithfully reads a project.
- **M2 and M3 build on M1 parsing.** CRUD (M2) rewrites exactly the entries that M1 can read, and link resolution (M3) uses the same parsed model.
- **M3 reuses M2's `create` to promote a stub.** "Create from stub" is not a parallel write path: it's M2 CRUD called to materialize a previously absent target, then re-resolve the link.
- **M4 (GUI) starts only once M1–M3 API is stable.** Consistent with "GUI is designed after the docs": the front is a pure consumer of an API frozen for read, write, and links, with no duplicated business logic.
- **M5 depends on the index (M1).** FTS search, filters, and sorts query the rebuildable cache; saved views are then exposed in the front (M4).
- **M6 assumes the front (M4).** MVP packaging (Docker/Nix) packages a complete application: `storyteller-server` serves the built front. Desktop packaging via `storyteller-tauri` (AppImage/.deb), which embeds the same front in a webview, is v2 (M7).

In summary, the critical path is **M1 → M2 → M3 → M4 → M6**, with **M5** grafted onto M1's index and delivered via M4's front.

## Deferred to v2 / Later

Outside MVP M1–M6, scheduled for **M7** or beyond:

- **Packaged desktop** (`storyteller-tauri` → AppImage/.deb): the same application embedded in a Tauri webview.
- **Interactive link graph** (network view of entries and backlinks).
- **Media gallery** for [assets](glossary.md) (images, maps).
- **Custom types** defined by the user beyond the ~11 default types.
- **Android APK spike** (mobile packaging via `storyteller-tauri`).

Remain **out of scope** permanently (see [features](features.md)): manuscript writing/prose editor, timeline/chronology, interactive maps and pins, real-time collaboration/multi-user/auth, AI/generation, cloud sync/SaaS, guided questionnaires and imposed templates, built-in versioning (left to Git).
