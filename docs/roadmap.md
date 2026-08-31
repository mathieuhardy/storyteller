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
| **M7** | ✅ done | v2+: **packaged desktop** (`storyteller-tauri` → AppImage/.deb), **link graph**, **media gallery**, **custom types**, **field extensions**, **character relationships**. | M6 | Each v2 item is scoped (spec or spike); none blocks the M1–M6 MVP. |

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

### M7 — What Landed So Far {#m7}

- **Media gallery** (`/gallery`, [docs/ui/screens.md](ui/screens.md#galerie)): visual browsing of everything
  under `assets/`, on top of M4's already-shipped `/assets` endpoints — no backend change needed. Grid of
  thumbnails (images) / generic-file cards, client-side filename filter (same reasoning as the Chantier's
  client-side aggregation — fine at local/single-user scale), upload via the existing `POST /assets` flow,
  and a detail modal with a "copy `![[path]]`" action for pasting into an entry body. No delete action —
  removing a still-referenced asset needs usage-checking that's out of scope for this pass.
- **Custom types** ([ADR 0017](adr/0017-custom-types.md), [data-model.md §7](data-model.md#custom-types)):
  a project can declare its own types in `.storyteller/types.yaml`, merged into the same catalog the 11
  built-ins live in (`storyteller-core::types`), leaked to `'static` at project-open so every existing
  `TypeSchema`-consuming function — parsing, validation, the index, the `/types` routes — needed no shape
  change, only a `custom_types` parameter threaded through. The frontend needed **zero changes**: creation,
  editing, listing, filtering, and search of a custom-typed entry all work through the existing
  schema-driven screens (M4). One known, documented gap: the nav/type-picker's `typeLabel()` falls back to
  the raw type name for a custom type rather than its own `label` — cosmetic, left as a follow-up. No GUI
  type-builder in this pass; `types.yaml` is hand-authored, like `config.yaml`'s `enabled_types` already is.
- **Link graph** (`/graph`, [docs/ui/screens.md](ui/screens.md#graphe)): the visualization layer
  [ADR 0008](adr/0008-no-graph-in-mvp.md) deferred to v2, now that its stated prerequisite — the backlinks
  index — has existed since M1. `GET /graph` (new, [ADR 0008](adr/0008-no-graph-in-mvp.md)) serves the whole
  project as nodes (every entry) and edges (every resolved, deduplicated entry-to-entry link) in one query
  against the index's `links` table. The frontend lays it out with a hand-rolled force simulation and
  renders it as plain SVG — no charting/graph library added ([ADR 0018](adr/0018-hand-rolled-graph-layout.md)).
  Hover highlights a node's direct neighbors, click navigates to the entry, wheel/drag zoom and pan.
- **Packaged desktop** (`storyteller-tauri`, [ADR 0019](adr/0019-tauri-reuses-the-http-router.md)): a Tauri v2
  shell that runs `storyteller-server`'s own router in-process on a loopback port and points its window at
  it — no separate IPC command surface, no duplicated API, the frontend unaware it isn't in a browser tab.
  `cargo tauri build` produces `Storyteller_0.1.0_amd64.AppImage` and `Storyteller_0.1.0_amd64.deb`; both
  were built **and run** in this environment (unlike M6's Nix packaging, this is verified end-to-end, not
  just authored to standard patterns).
- **Native folder picker** for the desktop app: `tauri-plugin-dialog` integrated, the launcher detects
  Tauri context via `window.__TAURI__` and shows a "Browse…" button that opens the native OS folder picker
  — the browser-hosted app's launcher stays exactly as it was (typed/pasted path only).
- **Type settings screen** (`/settings/types`): lists every type — built-in and custom alike — with a
  toggle wired to the existing `PATCH /types/{type}` endpoint, persisting enable/disable to
  `.storyteller/config.yaml`. No backend work needed; the endpoint existed since M4.
- **CI: release workflow** (`.github/workflows/release.yml`): GitHub Actions pipeline triggered on
  `v*` tags that builds desktop apps for Linux (AppImage, `.deb`), Windows (`.msi`), and macOS
  (`.dmg`) via `cargo tauri build`, then creates a draft GitHub Release with all artifacts attached.
  Includes `workflow_dispatch` for manual testing without pushing a tag. The `release` job only runs
  on actual tag pushes (skipped on manual dispatch).
- **Nix flake: Tauri desktop package**: `flake.nix` now builds `storyteller-tauri` as default
  (`nix build`, `nix profile install .`), with proper NixOS desktop integration: `.desktop` file,
  icon in `share/icons/hicolor/`, and the binary renamed to `storyteller`. The previous
  `storyteller-server` package remains available as `nix build .#storyteller-server`. The
  `npmDepsHash` placeholder is now filled in with the real hash.
- **Field extensions** ([ADR 0017](adr/0017-custom-types.md), [data-model.md §7](data-model.md#field-extensions)):
  `types.yaml` now supports a `field_extensions` section to add fields to existing types (built-in
  or custom) without redefining them. Same validation rules as custom type fields (snake_case, no
  shadowing common/existing fields, enum values required). Extensions are merged into the type's
  schema at load time — extended fields appear in forms, validation, and `/types/{type}`
  indistinguishably from native fields.
- **Character-to-character relationship fields**: `character` now has `family`, `allies`, and
  `rivals` link-list fields pointing to other characters, mirroring `faction`'s existing
  `allies`/`rivals` shape. Pure schema addition — no backend work beyond what link-list fields
  already do.

M7 is done.

### Book mode — What Landed

A distraction-free manuscript editor, alongside the existing "storyteller" worldbuilding mode:
opening a plain folder of markdown files as a "book" (`POST /books/open`) switches the interface to
a focused writing environment, entirely separate from the indexed project/link machinery above.

- **Standalone book mode**: backend (`storyteller-server/src/routes/books.rs`, `BookState` in
  `state.rs`) serves a folder of `.md` files with no `.storyteller/config.yaml`, no index, no
  snapshot, and no watcher/SSE. Frontend at `/book`.
- **Left panel**: a filesystem tree showing only `.md` files (and folders that contain at least
  one, recursively) — `FileTree.svelte`.
- **Center**: a raw markdown editor (`RawEditor.svelte`) with resizable width presets — `medium`
  (90ch), `wide` (110ch), `full` — plus line-height and line-break display settings.
- **Word count**: live, derived from editor content, shown in the topbar (`WordCount.svelte`).
- **Post-save replacements**: literal find/replace rules (e.g. `--` → `—`, `...` → `…`), authored
  through a settings popover (`ReplacementsSettings.svelte`) and persisted server-side in the one
  deliberate exception to "no `.storyteller/`": an optional `.storyteller/replacements.yaml`
  (`crate::replacements` in `storyteller-server`). Applied server-side on every save, before the
  file is written; since books have no watcher/SSE to otherwise notify the editor, `PUT
  /books/files/{path}` returns the post-replacement content and the frontend syncs its buffer from
  that response — no external script, no polling.

Book mode is done.

## Backlog

Features planned for implementation, not yet scheduled.

### Editor search (Ctrl+F)

In-editor text search for markdown editing (both storyteller entry editor and book mode):

- Toolbar appears on `Ctrl+F`, showing: occurrence count, current match index, Previous/Next buttons.
- Highlights all matches in the editor, scrolls to the current one.
- Available everywhere markdown is edited: entry body editor (`/entry/{slug}/edit`) and book raw editor (`/book`).

### Undo/redo (Ctrl+Z / Ctrl+Shift+Z)

Local undo stack for the markdown editor:

- `Ctrl+Z` undoes the last modification, `Ctrl+Shift+Z` (or `Ctrl+Y`) redoes.
- Stack capped at 100 entries to bound memory.
- Scope: in-editor changes only (not cross-session or server-side history).

### LanguageTool integration (book mode)

Grammar and spelling check via a local LanguageTool server, displayed in a right drawer:

- **Drawer panel** (300px, toggle): lists detected issues with message, context, and suggestions.
- **Apply suggestion**: click replaces text at the error offset, marks file dirty.
- **Settings popover**: server URL (e.g. `http://localhost:8081`), language selector (fr, en-US, …), "Test connection" button.
- **Config persistence**: `.storyteller/languagetool.yaml` (shared if project synced).
- **Scope**: book mode only (no storyteller entry editor).

MVP excludes: in-editor highlighting, auto-check on idle, click-to-navigate. Deferred to a later phase.

## Critical Path

The milestone order is not arbitrary: each stage builds on invariants set by the previous one.

- **M1 is the foundation.** Markdown parsing and the [index](glossary.md) condition everything else; nothing solid gets built until the core faithfully reads a project.
- **M2 and M3 build on M1 parsing.** CRUD (M2) rewrites exactly the entries that M1 can read, and link resolution (M3) uses the same parsed model.
- **M3 reuses M2's `create` to promote a stub.** "Create from stub" is not a parallel write path: it's M2 CRUD called to materialize a previously absent target, then re-resolve the link.
- **M4 (GUI) starts only once M1–M3 API is stable.** Consistent with "GUI is designed after the docs": the front is a pure consumer of an API frozen for read, write, and links, with no duplicated business logic.
- **M5 depends on the index (M1).** FTS search, filters, and sorts query the rebuildable cache; saved views are then exposed in the front (M4).
- **M6 assumes the front (M4).** MVP packaging (Docker/Nix) packages a complete application: `storyteller-server` serves the built front. Desktop packaging via `storyteller-tauri` (AppImage/.deb), which embeds the same front in a webview, is v2 (M7).

In summary, the critical path is **M1 → M2 → M3 → M4 → M6**, with **M5** grafted onto M1's index and delivered via M4's front.

## Ideas Under Investigation

Exploratory paths — not scheduled, need ADR or spike first. Recorded so they aren't lost.

### Optional local AI drawer *(uncertain)*

An opt-in connection to a local AI backend (Ollama, llama.cpp) for content suggestions. **This
reopens a stated Non-Goal** — needs its own ADR before any implementation. Constraints: strictly
opt-in, local-only (no cloud), never a required dependency.

**Status: uncertain** — keeping as an idea to revisit, not committed.

## Out of Scope

Permanently excluded (see [features.md](features.md)):

- Timeline/chronology with imaginary calendars ([ADR 0001](adr/0001-no-timeline.md))
- Interactive map pins ([ADR 0007](adr/0007-static-maps-no-pins.md))
- Real-time collaboration / multi-user / auth
- Cloud sync / SaaS
- Guided questionnaires / imposed templates
- Built-in versioning (use Git)
