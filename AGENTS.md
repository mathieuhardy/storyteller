# AGENTS.md — Entry Point for Code Agents

## Language Policy

**Always respond in French, but always write code and documentation in English.**

## Project Purpose

**Storyteller** is a local, single-user tool for storing and organizing all the **context** of a novel (the "bible": characters, world, chapters, resources). It is **not** for writing the manuscript itself, and **does not have** a narrative timeline. Entries are **typed, modular entities** whose source of truth is **raw markdown** (YAML frontmatter + free-form body), linked by **wikilinks** with automatic **backlinks**.

> **Docs before code.** The specs come first and stay authoritative: any code that deviates from them is either wrong, or must be accompanied by the doc update that makes it right. **The GUI is designed AFTER** the documentation, never before.

**Current state**: M0–M6 are done (specs, core + read-only API, non-destructive CRUD + watcher, links/stubs, SvelteKit frontend, FTS search + saved views, Docker/Nix packaging) — the M1–M6 MVP is complete. **M7** (v2+) is in progress: the media gallery, custom types ([ADR 0017](docs/adr/0017-custom-types.md)), the link graph ([ADR 0008](docs/adr/0008-no-graph-in-mvp.md), [ADR 0018](docs/adr/0018-hand-rolled-graph-layout.md)) and the desktop shell ([ADR 0019](docs/adr/0019-tauri-reuses-the-http-router.md), `storyteller-tauri` → AppImage/.deb) have landed; only the Android APK spike is not started. See the [roadmap](docs/roadmap.md) for the milestone table and [api.md](docs/api.md#implementation-status) for what the API serves today.

---

## Read First

Before any contribution, read the specs **in this order** — each doc assumes the previous one is understood:

1. [Vision](docs/vision.md) — the why, the audience, the product promise.
2. [Principles](docs/principles.md) — design invariants (including "markdown = source of truth").
3. [Data Model](docs/data-model.md) — types, frontmatter fields, project structure.
4. [Linking](docs/linking.md) — wikilinks, backlinks, stubs, aliases, resolution.
5. [Features](docs/features.md) — scope, priorities, what is **out of scope**.
6. [Architecture](docs/architecture.md) — Rust workspace, index/cache, watcher, frontend.
7. [API](docs/api.md) — HTTP contract between backend and frontend.

Cross-cutting references, consult as needed:

- [Glossary](docs/glossary.md) — **canonical terminology**; use exactly these terms in code and docs.
- [Roadmap](docs/roadmap.md) — **development order** (milestones M0 → M7) and work breakdown.
- [Usage](docs/usage.md) — end-user guide; **stub** until GUI milestone (M4), nothing to implement before then.

---

## Task to Doc Map

Before touching anything, open the relevant doc(s).

| What you want to do | Reference doc(s) |
| --- | --- |
| Modify entry format (frontmatter, fields, types) | [data-model.md](docs/data-model.md) (+ [linking.md](docs/linking.md)) |
| Touch links, backlinks, stubs, aliases, resolution | [linking.md](docs/linking.md) (+ [data-model.md](docs/data-model.md)) |
| Add/modify an endpoint, change an API contract | [api.md](docs/api.md) (+ [architecture.md](docs/architecture.md)) |
| Understand the index, SQLite cache, watcher | [architecture.md](docs/architecture.md) |
| Check scope, priorities, what's excluded | [features.md](docs/features.md) |
| Know the dev order, a milestone, a work package | [roadmap.md](docs/roadmap.md) |
| Use the right term (official vocabulary) | [glossary.md](docs/glossary.md) |
| Understand or record an architecture decision | [docs/adr/](docs/adr/README.md) |
| Start on frontend / views / editor | [features.md](docs/features.md) + [architecture.md](docs/architecture.md) |

---

## Golden Rules (invariants — non-negotiable)

1. **Markdown is the ONLY source of truth.** All application state derives from the files. Nothing important lives only in the database.
2. **NEVER break a file edited outside the app.** Writing is **non-destructive**: we preserve **unknown YAML keys**, **key order**, and the **body** as-is. A file created in Obsidian, vim, or a mobile editor must remain intact after a round-trip through Storyteller.
3. **The index is a disposable cache, never authoritative.** The `cache.sqlite` must be **100% rebuildable** from the markdown files. In case of divergence, **the file wins**.
4. **Local-first: zero required network dependency.** The tool works entirely offline. No network call should be required to read, write, index, or navigate.
5. **Tolerance for imperfect data.** Malformed frontmatter, missing field, broken link, unknown type: we **signal** (warning, stub, degraded state), we **don't block**, and we never lose data.

Any contribution that violates one of these rules must be rejected, regardless of its other merits.

---

## Repo Conventions

### Workspace Layout

Rust workspace + separate frontend:

```
storyteller/
  storyteller-core/     (lib) model, markdown/YAML parsing, index, links
  storyteller-server/   (bin) HTTP server, exposes the API — depends on core
  storyteller-tauri/    (bin) desktop webview, runs the server's router in-process (ADR 0019)   [M7, Android APK pending]
  frontend/             SvelteKit + Tailwind (ADR 0013), consumed by both binaries
  tests/fixtures/       reference project, shared by both crates' tests
  docs/                 specs (see "Read First")
```

`storyteller-core` depends on neither binary; both binaries (`server`, `tauri`) **share** `core`. See [architecture.md](docs/architecture.md) for details.

Inside `storyteller-core/src/`: `parse` (frontmatter/body), `model` (entry shape), `types` (type catalog), `custom_types` (user-declared types, `types.yaml`), `normalize` (match keys), `links` (extraction + resolution), `project` (reads the files), `index/` (SQLite cache), `config`, `error`.

### Build & Test

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full build-from-source walkthrough (prerequisites,
Docker, Nix, the desktop app's system dependencies). Quick reference:

```sh
cargo test                      # whole workspace
cargo clippy --all-targets      # must be warning-free
cargo fmt --all
cargo run -p storyteller-server -- --project /path/to/my-novel
cargo tauri dev                 # desktop shell, from storyteller-tauri/ (needs `cargo install tauri-cli`)
```

The server then serves `http://127.0.0.1:8787/api/v1/…`. `tests/fixtures/sample-project` is a ready-made project to point it at. `cargo build`/`cargo test` never require `npm run build` first — `storyteller-server` embeds nothing without it ([ADR 0016](docs/adr/0016-embed-frontend-in-server-binary.md)) and `storyteller-tauri` points its own `frontendDist` at a placeholder it never actually serves ([ADR 0019](docs/adr/0019-tauri-reuses-the-http-router.md)).

### Language

- **Frontmatter keys**: English, `snake_case` (e.g., `ruling_faction`, `chapter_status`).
- **UI labels and content** (values, titles, entry prose): **French**.
- **Documentation**: English.
- **Code comments**: recommended convention = **English** (aligned with the Rust/crates ecosystem and code identifiers). Stay consistent within a module; don't mix both in a file.

### Commits

**Conventional Commits** recommended: `type(scope): description`.

- Common types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `build`.
- Examples: `docs(data-model): clarify chapter type fields`, `feat(core): non-destructive frontmatter parsing`.
- Suggested scope = crate or domain (`core`, `server`, `tauri`, `frontend`, `data-model`, `linking`…).

---

## Scope

The exact scope and priorities live in [features.md](docs/features.md). **Read it before adding a feature.**

Quick reminder — **OUT OF SCOPE** (do not implement, not even "as a bonus"):

- Manuscript writing / prose editor.
- Timeline / narrative chronology.
- Interactive maps and pins (maps are **static images** referenced).
- Real-time collaboration, multi-user, authentication, permissions/secrets.
- AI / content generation.
- Cloud sync / SaaS.
- Guided questionnaires and imposed prose templates.
- Built-in versioning (left to **Git**, outside the app).

---

## Locked Decisions

Recorded architecture decisions are documented in [docs/adr/](docs/adr/README.md).

> **It is forbidden to reopen a locked decision without a new ADR.** If you think a decision should change, you **do not implement it**: you draft an ADR (status "proposed") that references and supersedes the existing ADR, with arguments. Code never precedes the decision.

---

## Definition of Done (generic)

A contribution is only "done" if **all** of the following are true:

- [ ] **Tests** present and passing for the added/modified behavior (non-destructive round-trip covered when relevant).
- [ ] **Documentation up to date**: any deviation from the specs is reflected in the relevant doc(s) in `docs/`.
- [ ] **Offline-safe**: no required network dependency introduced; works offline.
- [ ] **No secrets**: no secret, token, or credential in code, config, or history.
- [ ] **Golden rules respected**: markdown source of truth, non-destructive writing, rebuildable index, tolerance for imperfect data.
- [ ] **Canonical terminology** respected (see [glossary.md](docs/glossary.md)) and language/commit conventions applied.
