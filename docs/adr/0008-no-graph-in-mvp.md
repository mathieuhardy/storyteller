# ADR 0008 — No Link Graph in MVP

- **Status**: Accepted
- **Date**: 2026-08-04

## Context

Since entries are connected by [wikilinks](../glossary.md) and [backlinks](../glossary.md) ([ADR 0006](0006-wikilinks-backlinks-stubs.md)), it's tempting to display this network as a **graph**: a cloud of nodes (entries) and edges (links) that can be visually navigated.

A graph is an appealing comfort feature, but it's also a separate workstream: layout engine, performant rendering on large projects, filtering, interactions, and real interface design effort. It's not part of the MVP's core value, which is to **store and organize** context with list/table views and a backlinks panel.

## Decision

**The link graph is not in the MVP; it is deferred to v2.**

- The MVP is limited to **filterable list/table views by type** and a **backlinks panel**. See [features](../features.md).
- The graph is explicitly listed in the [roadmap](../roadmap.md) at v2 milestone (M7 and beyond).

## Consequences

- **Effort focused** on the core: data model, links/backlinks/stubs, views and search. The MVP ships faster and more solid.
- **Foundations already in place**: the backlinks inverse index ([ADR 0006](0006-wikilinks-backlinks-stubs.md)) already provides the data for a future graph; adding it in v2 won't require a model overhaul, just a visualization layer.
- Accepted consequence: no visual network overview in MVP; navigation uses links and backlinks.
- Decision **reversible at no structural cost**: it closes no doors, it just defers a display layer.

## Alternatives Considered

- **Graph from MVP**: lengthens scope and diverts effort from the core, for a comfort feature. Rejected for MVP.
- **No graph at all**: too restrictive long-term when the data already exists. Rejected.
- **Graph deferred to v2** (retained): deliver essential value first, then add visualization on foundations already ready.
