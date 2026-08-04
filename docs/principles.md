# Guiding Principles

This document states the principles that **arbitrate all implementation choices** for Storyteller. When two technical solutions conflict, we decide in favor of the one that respects these principles — in the order listed in case of conflict.

Each principle is described by: a short explanation, a **concrete implication for the code**, and an **anti-pattern to reject**. These principles are non-negotiable on the fly: a structural challenge goes through an [ADR](adr/README.md).

For the vocabulary used here (entry, frontmatter, body, index, watcher, stub…), see the [glossary](glossary.md). For data mechanics, see the [data model](data-model.md); for links, [linking](linking.md).

---

## 1. Local-first / offline

The application works **entirely offline**, on the user's machine, without any mandatory remote service. No network request is needed to open a project, read, edit, or link entries. Self-hosting (Docker, binary) is just a way to run this same local software.

- **Concrete implication for the code**: no runtime dependency on an online service (telemetry, CDN, third-party API, license check). All resources (fonts, icons, front assets) are bundled in the binary or served locally. A project opens from a simple folder path.
- **Anti-pattern to reject**: "feature X requires a connection"; a blocking network call at startup; a font or lib loaded from an external CDN.

---

## 2. Markdown is the sole source of truth

The `.md` files on disk **are** the data. The SQLite [index](glossary.md) (`.storyteller/cache.sqlite`) is a **disposable cache**, 100% rebuildable from the files, and **never authoritative**. If the index and files diverge, the files always win.

- **Concrete implication for the code**: any write goes first to the markdown file, then updates the index (never the reverse). The index is `gitignored`. A **full reindex** command/routine exists and can delete then rebuild `cache.sqlite` without any business data loss. Deleting the cache should be a harmless operation.
- **Anti-pattern to reject**: storing in SQLite data that exists nowhere in the files (e.g., a user-entered field present only in the index); reading a "truth" value from the cache without the ability to regenerate it from disk.

---

## 3. Respect for files edited outside the app (non-destructive writing)

A project is edited by Storyteller as well as Obsidian, vim, a mobile editor, or Git. App writes must be **non-destructive**: they preserve everything they don't understand.

- **Concrete implication for the code**: when writing an entry, preserve **unknown YAML keys**, **existing key order**, formatting, and the **markdown body as-is** (including comments, whitespace, list style). Only write actually modified fields. A [watcher](glossary.md) detects external modifications and reindexes. A read→write round-trip without user modification produces an empty diff (idempotence).
- **Anti-pattern to reject**: rewriting the entire file by re-serializing from an in-memory model (which reorders keys and drops unknown fields); "cleaning" or reformatting the user's YAML/markdown along the way.

---

## 4. Reversibility / no lock-in

The user can **leave at any time without loss**. A project is a folder of ordinary markdown, readable and useful even if Storyteller disappears. We create no dependency on a proprietary format.

- **Concrete implication for the code**: no binary or proprietary format for business data (only `cache.sqlite`, which is disposable, uses one). [Wikilinks](glossary.md), [assets](glossary.md), and [frontmatter](glossary.md) remain standard markdown conventions, readable by other tools. Moving/copying the folder is enough to "export." Versioning is left to Git (out of application scope).
- **Anti-pattern to reject**: a homemade link syntax unreadable outside the app; burying business data in the filename or in an opaque database; any "export" step required to retrieve your data.

---

## 5. Modularity (typed entities)

[Entities](glossary.md) are **typed** (~11 [types](glossary.md) by default) and types are **activatable/deactivatable** per project via `.storyteller/config.yaml` (`enabled_types`). The [type](glossary.md) drives the proposed [frontmatter](glossary.md) schema, without ever constraining the [body](glossary.md).

- **Concrete implication for the code**: the engine is **driven by type schemas**, not hard-coded per type. Disabling a type deletes no files: entries remain readable and their links valid; the type just disappears from views/creation. Adding a type (v2: custom types) should not require recompiling the core.
- **Anti-pattern to reject**: a giant `match` / hard-coded branches for each type in business logic; making type deactivation a destructive operation.

---

## 6. Single-user & simplicity (no "just in case" complexity)

Storyteller is **single-user, local**. No auth, no collaboration, no permissions, no multi-tenant. We reject any anticipated complexity that doesn't serve the MVP.

- **Concrete implication for the code**: no authentication layer, user management, roles, or secrets handling. No speculative abstraction "for later" (interfaces with a single implementation, extension points without real need). We code the single-user case directly and readably.
- **Anti-pattern to reject**: introducing account/permission concepts "in case we do multi-user"; building a generic plugin system before having a second real use case; over-abstracting for a hypothetical future.

---

## 7. Tolerance for imperfect data

A real project contains invalid YAML, broken links, missing fields, half-edited files. **None of this should block project opening.** We degrade gracefully and **signal**, we don't crash.

- **Concrete implication for the code**: parsing is **resilient and localized** — an entry with invalid frontmatter is indexed in degraded mode (at minimum by filename and body), never silently rejected or fatal for the rest of the project. Errors are **collected and surfaced** (per-entry diagnostics) for display. An unresolved link becomes a signaled [stub](glossary.md), not a blocking error. No field is required to *open* an entry (see also [linking](linking.md)).
- **Anti-pattern to reject**: an `unwrap`/`panic` on user content; a single corrupted entry that fails the entire indexing; "auto-correcting" the user's file silently instead of signaling.

---

## 8. Separation of context / manuscript

Storyteller stores the **context** of a novel (the "bible": [characters, locations, factions…](data-model.md)), **not the manuscript** or its prose. There is **no timeline / narrative chronology**.

- **Concrete implication for the code**: date fields managed by the app (`created`/`updated`) are **technical metadata**, never a narrative time axis. The `order` field of a chapter is a **position in the manuscript**, not a date. No long-form prose editor, no prose templates or imposed guided questionnaires: the [body](glossary.md) remains free-form markdown.
- **Anti-pattern to reject**: introducing a timeline or "by story date" sorting; turning `order` into a time axis; adding fields that push the user to write the novel *in* the tool.

---

## 9. Mobile-friendly portability

The format and architecture must remain **usable on mobile** (Android APK path via `storyteller-tauri`). Raw markdown is chosen precisely for this portability.

- **Concrete implication for the code**: the business core ([`storyteller-core`](architecture.md)) is platform-agnostic and reusable behind the HTTP server as well as the Tauri webview; no business logic lives in the front. No dependency on desktop-specific paths/tools. Files (names as ASCII kebab-case [slugs](glossary.md#slug), relative paths) remain valid on mobile file systems.
- **Anti-pattern to reject**: putting business logic in the SvelteKit frontend (making it non-reusable on mobile); assuming a desktop-type file system or absolute paths; filenames with non-portable characters.

---

## 10. Doc-first (any structural decision goes through an ADR)

Structural decisions are **documented before** being coded. Documentation precedes code (this project is first a spec). An architecture decision is traced in an [ADR](adr/README.md).

- **Concrete implication for the code**: a structural choice (format, schema, link resolution, index strategy, stable `id` field…) is the subject of an ADR **before** implementation; code references the relevant ADR. [Canonical conventions](data-model.md) and [glossary](glossary.md) terms are the single reference: code aligns to them instead of reinventing.
- **Anti-pattern to reject**: deciding a structural question "quietly" in code without an ADR; letting code and docs diverge (YAML key, term, link behavior); inventing a decision on an undecided point — refer instead to an ADR or the "open questions" section.

---

## See Also

- [Vision](vision.md) — the product's why
- [Features](features.md) — what the MVP does
- [Data Model](data-model.md) — types, fields, frontmatter conventions
- [Linking](linking.md) — wikilinks, stubs, aliases
- [Architecture](architecture.md) — Rust workspace, index, watcher
- [Architecture Decision Records (ADR)](adr/README.md)
