# ADR 0015 — Markdown rendering happens in `core`

- **Status**: Accepted
- **Date**: 2026-08-24

## Context

`docs/architecture.md` §6 flagged this as an open question from M1 onward: **where does `?render=html`
happen** — `storyteller-core` (Rust) or the frontend (JS)? Until decided, `?render=html` answers `501` rather
than guessing ([api.md](../api.md) §3), and the entry detail/editor screens fall back to raw text with
manually-highlighted `[[wikilinks]]` (`BodySection.svelte`, `BodyEditor.svelte`) as a stopgap.

The wrinkle that makes this a real decision, not a formality: [wikilinks](../linking.md) and asset embeds
(`[[Target]]`, `![[file]]`) are **not CommonMark syntax**. Rendering the body to HTML means resolving them —
by the exact same ranked algorithm (`filename → title → alias`, `docs/linking.md` §3) already implemented in
`core::links::Resolver` for the index and the `/links`/`/backlinks` endpoints. Whoever renders HTML either
**owns that resolution logic**, or **duplicates it**.

`core`'s own doc comment already states the stakes: "both binaries (`storyteller-server`, `storyteller-tauri`)
are thin wrappers over this crate, which is what keeps their behaviour identical." A frontend-side renderer
would need its own JS implementation of title/alias/slug ranking, ambiguity detection, and stub classification
— a second, drifting copy of `docs/linking.md` §3, in the one layer the architecture explicitly says should
carry **no business logic** (`docs/architecture.md` §3, restated in the entry editor's own comments).

## Decision

**Rendering happens in `storyteller-core`.** A new `render` module turns a raw body into HTML:

1. **Wikilinks and embeds are spliced first**, at their exact byte offsets (`links::extract_from_body` already
   gives these): `[[Target]]` becomes an `<a class="wikilink resolved" href="/entry/{slug}">Display</a>` (or
   a `<span class="wikilink stub|ambiguous">` when unresolved), and `![[file]]` becomes an `<img>` sourced from
   `/api/v1/assets/{path}` — resolved by matching the embed text against an asset's **filename**, since
   `![[file]]` addresses assets the same way a wikilink addresses entries: by name, not by full path
   (`docs/linking.md` §"Images and assets"). A `![[file]]` with no matching asset renders a visible
   "asset not found" block rather than a broken `<img>`, matching the workshop screen's diagnostic language.
2. The **spliced text is then handed to `pulldown-cmark`** (CommonMark + tables/strikethrough/task-lists).
   CommonMark passes inline/block raw HTML straight through untouched, so the crafted `<a>`/`<img>`/warning
   fragments survive parsing unchanged — no need for pulldown-cmark to know wikilinks exist at all.
3. Resolution itself is **not recomputed from scratch**: the route handler feeds `render_body` a lookup
   closure backed by `Index::outgoing_links` (already computed at index time) rather than rescanning every
   entry in the project through a fresh `Resolver`. `render_body` itself stays a pure function of `(body,
   resolve closure, asset-exists closure)` — decoupled from `Index`/`Project`, and unit-testable with plain
   closures, matching `links::rewrite_body_links`'s existing shape.

`GET /entities/{slug}?render=html` populates `Entry.html` (already defined but never populated, `api.md` §2)
with this output. It is never written back to the file — display-only, exactly as documented.

## Consequences

- **One resolution algorithm, one place.** The frontend gains real rendered content without ever
  reimplementing wikilink ranking; `BodySection.svelte`'s hand-rolled regex highlighter and
  `BodyEditor.svelte`'s preview mode can be replaced by injecting `entry.html` (`{@html}`), once that
  follow-up frontend change is made — out of scope for this ADR, which only lands the backend contract.
- **A frontend routing convention leaks into `core`.** The `<a href="/entry/{slug}">` and
  `<img src="/api/v1/assets/{path}">` forms bake in the app's own path scheme. This is deliberately accepted:
  per `architecture.md` §3, there is **one SvelteKit frontend**, target-independent — the same `/entry/{slug}`
  client-side route is shared whether it's fetching over HTTP (`storyteller-server`) or, later, Tauri IPC
  (`storyteller-tauri`, M7, transport still an open question of its own). If a second, differently-routed
  frontend ever appears, this convention needs revisiting — not a concern at MVP scale.
- **`![](assets/...)` (the plain-markdown image form) is left as pulldown-cmark renders it** — a
  project-relative `src`, not rewritten to the API's asset URL. Only the `![[file]]` wikilink-style embed
  (the form actually used in this project's own content) gets the full treatment. Documented gap, not a
  silent one: rewriting the plain form would need an event-level transform pass over pulldown-cmark's parser
  output, deferred until something other than `![[file]]` is actually authored.
- **No HTML sanitizer.** Per [ADR 0003](0003-single-user-local.md), there is exactly one, trusted author of
  the content being rendered back to themselves — there is no meaningfully new attack surface to sanitize
  against in a single-user local tool. This would need revisiting if multi-user/cloud ever entered scope,
  which [principles.md](../principles.md) rules out permanently.
- **New dependency**: `pulldown-cmark` (widely used, no transitive bloat, actively maintained CommonMark
  implementation) in `storyteller-core`.

## Alternatives Considered

- **Render in the frontend** (e.g. `markdown-it`/`marked` + a JS reimplementation of wikilink resolution).
  Rejected: duplicates `docs/linking.md` §3's ranked resolution algorithm in a second language, in the layer
  the architecture says should hold no business logic, and risks drifting from `core`'s behavior over time
  (two renderers of the same content, only one of which is authoritative for links).
- **Hybrid — `core` resolves links to a data structure, frontend renders markdown and injects link data.**
  Rejected: splits one concern (rendering the body) across two layers for no clear gain, and still requires
  the frontend to parse CommonMark itself just to know where to splice link data in.
- **Sanitize rendered HTML with a dedicated crate** (e.g. `ammonia`). Rejected for now: no threat model calls
  for it in a single-user local tool (see Consequences); adds a dependency and a rule-list to maintain for a
  risk that doesn't exist under `docs/principles.md`.
