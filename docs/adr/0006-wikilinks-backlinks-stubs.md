# ADR 0006 — Wikilinks, Backlinks, and Stubs

- **Status**: Accepted
- **Date**: 2026-08-04

## Context

A novel bible is a network: a character belongs to a faction, lives in a location, wields an object governed by a system. A mechanism for **inter-entry links** is needed that is readable in markdown, robust to renaming, and navigable in both directions.

Several options: don't tool the links (plain text), link by **bare identifier** (reference by opaque id), or use **wikilinks** by name (`[[Name]]`) doubled with an [index](../glossary.md) inverse.

Bare id links are stable but unreadable to the eye and tedious to type by hand. Lack of tooled links deprives the author of navigation and missing reference detection.

## Decision

**Storyteller uses wikilinks, automatic backlinks, and stub detection.** Details in [linking](../linking.md).

- **Wikilinks**: `[[Target]]` and `[[Target|display text]]` in body and [frontmatter](../data-model.md). Images: `![[file]]` or `![](assets/...)`.
- **Frontmatter links** stored as wikilink strings (e.g., `pov: "[[Aria]]"`, `locations: ["[[Glass City]]"]`). It is **the field name** that carries the link semantics (`pov`, `parent`, `owner`, `leader`…).
- **Resolution**: filename → `title` → `aliases` → otherwise **[stub](../glossary.md)** (referenced but nonexistent target). See [linking](../linking.md).
- **Automatic [backlinks](../glossary.md)**: the inverse index lists, for each entry, who points to it; a backlinks panel is offered in views.
- **Renaming**: on name change, the app updates links and/or keeps the old title as [alias](../glossary.md).

## Consequences

- **Readability and external editing**: `[[Name]]` remains readable and hand-typeable (Obsidian, vim), consistent with markdown source of truth ([ADR 0002](0002-markdown-source-of-truth.md)).
- **Bidirectional navigation**: backlinks, computed from the index, show the network without the author manually maintaining inverse links.
- **Remaining work discovery**: stubs signal entries to create; an entry can be created directly from a stub.
- **Resolution cost**: name-based resolution requires handling **ambiguity** (two entries bearing the same name/alias) and link updates on rename — these cases are specified in [linking](../linking.md).
- **Hardening point**: a stable `id` field would make links insensitive to renaming; it is **deliberately not imposed** here and left to decide (see [data model](../data-model.md) and [roadmap](../roadmap.md)).
- The inverse index must be rebuildable from files, like all of the index ([ADR 0002](0002-markdown-source-of-truth.md)).

## Alternatives Considered

- **No tooled links** (plain text): no backlinks, no missing reference detection, manual navigation. Rejected.
- **Bare identifier links**: robust to renaming but unreadable and tedious to type manually, in tension with external editing. Rejected as the primary mechanism (the `id` remains a complementary hardening path, not a substitute for wikilinks).
- **Wikilinks + backlinks + stubs** (retained): readable, navigable both ways, revealing of remaining work, at the cost of explicit ambiguity and rename handling.
