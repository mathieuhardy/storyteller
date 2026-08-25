# Architecture Decision Records (ADR)

This folder contains the **ADRs** (*Architecture Decision Records*) for the Storyteller project: a written, dated, and immutable trace of structural product and architecture decisions. Each file documents **a single decision**: its context, the chosen option, its consequences, and the alternatives considered.

## What ADRs Are For

- Give code agents and reviewers the **why** of a decision, not just the **what**.
- Avoid endlessly reopening debates already settled.
- Make the decision history visible: when a decision is superseded, the old one remains readable to understand the project's evolution.

For overall product context, see [vision](../vision.md) and [principles](../principles.md). For technical details, see [data model](../data-model.md), [linking](../linking.md), and [architecture](../architecture.md).

## Possible Statuses

| Status | Meaning |
| --- | --- |
| **Proposed** | Decision under discussion, not yet enacted. |
| **Accepted** | Decision in effect, it has authority. |
| **Superseded** | A more recent ADR takes over (link to it is indicated). |
| **Deprecated** | Decision abandoned without a direct replacement. |

## Procedure

1. **Incrementing numbering**: each ADR receives a four-digit number (`0001`, `0002`, …) assigned in creation order, never reused.
2. **One decision per file**: name the file `NNNN-title-in-kebab-case.md`.
3. **Immutability**: we **do not modify** an accepted ADR. To reverse a decision, we create a **new** ADR that supersedes it, then change the old one's status to "Superseded" with a link to the new one. The only allowed edits on an accepted ADR are typo fixes and status/cross-link updates.
4. **Template**: start from [`_template.md`](_template.md) to draft a new ADR.

## ADR Index

| # | Title | Status |
| --- | --- | --- |
| [0001](0001-no-timeline.md) | No timeline or narrative chronology | Accepted |
| [0002](0002-markdown-source-of-truth.md) | Markdown is the sole source of truth | Accepted |
| [0003](0003-single-user-local.md) | Single-user, local / self-host | Accepted |
| [0004](0004-one-folder-per-project.md) | One self-contained folder per project | Accepted |
| [0005](0005-typed-modular-entities.md) | Typed, modular entities | Accepted |
| [0006](0006-wikilinks-backlinks-stubs.md) | Wikilinks, backlinks, and stubs | Accepted |
| [0007](0007-static-maps-no-pins.md) | Static maps without interactive pins | Accepted |
| [0008](0008-no-graph-in-mvp.md) | No link graph in MVP | Accepted |
| [0009](0009-rust-sveltekit-stack.md) | Rust + SvelteKit + Shadcn stack | Accepted |
| [0010](0010-frontmatter-keys-en-content-fr.md) | Frontmatter keys in English, content in French | Accepted |
| [0011](0011-index-sqlite-fts5.md) | SQLite + FTS5 as the index engine | Accepted |
| [0012](0012-rename-link-rewriting.md) | Renaming rewrites breaking links, without keeping an alias | Accepted |
| [0013](0013-tailwind-with-component-classes.md) | Tailwind, contained by component classes | Accepted |
| [0014](0014-ui-i18n-front-catalog.md) | UI language via a front-side i18n catalog | Accepted |
| [0015](0015-markdown-rendering-in-core.md) | Markdown rendering happens in `core` | Accepted |
| [0016](0016-embed-frontend-in-server-binary.md) | The built frontend is embedded into `storyteller-server` at compile time | Accepted |
| [0017](0017-custom-types.md) | Custom types are declared in `.storyteller/types.yaml` and merged into the catalog at runtime | Accepted |
| [0018](0018-hand-rolled-graph-layout.md) | The link graph uses a hand-rolled force layout, no charting/graph library | Accepted |
| [0019](0019-tauri-reuses-the-http-router.md) | `storyteller-tauri` reuses the HTTP router in-process, no native IPC command surface | Accepted |
