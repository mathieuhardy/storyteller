# Features — Prioritized Catalog

This document lists **Storyteller** features grouped by domain, with a priority for each. It serves as the scope reference: it says **what**, not **when** or **how**. For development order, see the [roadmap](roadmap.md); for type and field details, see the [data model](data-model.md).

## Priority Legend

| Tag | Meaning |
| --- | --- |
| **MVP** | Product core, essential for first real usage (milestones M0–M6, excluding advanced options). |
| **v2** | Improvement added once the base product is usable. |
| **later** | Exploratory path, not scheduled, to be investigated (ADR or spike). |

> The tag expresses the **product maturity** of a feature. The **build sequence** (milestones M0→M7) is described in the [roadmap](roadmap.md) and may deliver some "v2" refinements during an earlier milestone. When in doubt about an undecided decision, refer to the [ADRs](adr/README.md) rather than inventing.

---

## A. Projects & Files

| Feature | Priority |
| --- | --- |
| Create / open a project (self-contained markdown folder) | MVP |
| Portable, movable project (relative paths, no state outside the folder) | MVP |
| Multi-project management (list, open, switch between folders) | MVP |
| Technical folder `.storyteller/` with `config.yaml` (`enabled_types`, `schema_version`) | MVP |
| Directory structure by type (`characters/`, `locations/`, `chapters/`, …) | MVP |
| Markdown = source of truth; **non-destructive** writing (preserves unknown YAML keys, key order, body intact) | MVP |
| Index `cache.sqlite`: disposable cache, gitignored, 100% rebuildable | MVP |
| Index rebuild on demand from files | MVP |
| File system `watcher`: detect external edits and re-index | MVP |
| `created` / `updated` fields managed by the app (never a narrative chronology) | MVP |
| Versioning left to Git (git-friendly structure, `.gitignore` provided) | MVP |
| Schema migrations driven by `schema_version` | v2 |

## B. Entities & Entries

| Feature | Priority |
| --- | --- |
| ~11 default `type`s (`project`, `character`, `location`, `faction`, `object`, `culture`, `system`, `species`, `chapter`, `note`, `concept`) | MVP |
| `entry` = `frontmatter` YAML (fields defined by `type`) + free-form `body` markdown | MVP |
| Common fields for all entries (`type`, `title`, `aliases`, `tags`, `cover`, `created`, `updated`) | MVP |
| Core and optional fields specific to each `type` (see [data model](data-model.md)) | MVP |
| CRUD for entries (create, read, update, delete) | MVP |
| Modify `frontmatter` without altering unknown keys or order | MVP |
| Rename an entry (file = ASCII kebab-case `slug`) | MVP |
| Light `frontmatter` validation by `type` schema (required fields, value types) | MVP |
| Respect `enabled_types`: only show/offer active `type`s | MVP |
| Free-form `tags` and multiple `aliases` per entry | MVP |
| UI for enabling/disabling `type`s (`/settings/types` screen) | MVP |
| User-defined custom `type`s | v2 |
| Custom fields added to a `type` | later |

> Type modularity is a locked decision: `enabled_types` exists and is respected from MVP; only the **toggle UI** is deferred to v2.

## C. Links & Navigation

| Feature | Priority |
| --- | --- |
| `wikilink`s in `body`: `[[Target]]` and `[[Target|display text]]` | MVP |
| Links in `frontmatter` stored as `wikilink` strings (semantics carried by **field name**: `pov`, `parent`, `owner`, `leader`, …) | MVP |
| Link resolution: filename → `title` → `alias` → otherwise `stub` | MVP |
| Automatic `backlink`s ("Mentioned in" panel) | MVP |
| `stub` detection (unresolved targets) | MVP |
| Create entry from a `stub` | MVP |
| Ambiguity handling (multiple possible targets for a link) | MVP |
| On rename: keep old `title` as `alias` (links continue to resolve) | MVP |
| Embedded images: `![[file]]` and `![](assets/…)` | MVP |
| Click-navigation on a `wikilink` (opens entry or `stub`) | MVP |
| Automatic `wikilink` rewriting in all files on rename | MVP |
| `wikilink` autocompletion in editor | MVP |
| Link graph (visualized relationships) | v2 |

## D. Views & Search

| Feature | Priority |
| --- | --- |
| List `view` by `type` | MVP |
| Table `view` by `type` (columns = `type` fields) | MVP |
| Filtering by `type` | MVP |
| Filtering by fields and `tags` | MVP |
| Sorting by field | MVP |
| Full-text search (FTS) on `title`, `aliases`, `tags`, and `body` | MVP |
| `backlink`s panel on each entry | MVP |
| Saved `view`s (reusable filters + sort) | MVP (M5) |
| Advanced combined queries/filters | v2 |
| Link graph as navigation mode | v2 |

## E. Media & Assets

| Feature | Priority |
| --- | --- |
| `assets/images/` and `assets/maps/` folders in project | MVP |
| `asset`s (images) referenced from entries | MVP |
| Media fields (`cover`, `portrait`, `image`, `map`) | MVP |
| Maps = **static** referenced image (`map` field) | MVP |
| Add `asset` via app (copy to `assets/`) | MVP |
| Media gallery (visual browsing of `asset`s) | v2 |
| Interactive pins/points on maps | later |

## F. Interop / Export

| Feature | Priority |
| --- | --- |
| Safe external editing (Obsidian, vim, mobile editor) thanks to non-destructive writing | MVP |
| Portable raw markdown, no proprietary lock | MVP |
| `wikilink` syntax compatible with Obsidian-like tools | MVP |
| "Bible" export (HTML / PDF / EPUB) | later |
| Import from another tool (Obsidian, World Anvil, Campfire, …) | later |

> Export and import are not locked decisions: treat them as paths to investigate in [ADR](adr/README.md) before any development.

## G. Platform / Packaging

| Feature | Priority |
| --- | --- |
| Rust backend: `storyteller-core` (lib) + `storyteller-server` (HTTP binary) | MVP |
| HTTP API (read-only first, then CRUD) | MVP |
| SvelteKit + Shadcn frontend (Tailwind, contained — ADR 0013) | MVP |
| Single-user, local / self-host, no auth or permissions | MVP |
| Self-host deployment via Docker | MVP |
| Nix flake(s) for developing and installing without Docker | MVP |
| Desktop application via `storyteller-tauri` (webview) | v2 |
| AppImage package | v2 |
| `.deb` package | v2 |
| Android APK spike (Tauri mobile) | later |

---

## OUT OF SCOPE

The following items are **explicitly out of scope** for Storyteller (the tool stores and organizes the **context** of a novel, it doesn't write the manuscript):

- Manuscript writing / prose editor.
- Timeline / narrative chronology.
- Interactive maps and clickable pins.
- Real-time collaboration, multi-user, authentication, permissions/secrets.
- AI / content generation.
- Cloud sync / SaaS.
- Guided questionnaires and imposed prose templates.
- Built-in versioning (left to Git).

For the implementation order of what **is** in scope, see the [roadmap](roadmap.md).
