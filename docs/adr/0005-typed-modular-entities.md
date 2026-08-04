# ADR 0005 — Typed, Modular Entities

- **Status**: Accepted
- **Date**: 2026-08-04

## Context

The content of a novel bible is heterogeneous: characters, locations, factions, objects, cultures, systems, species, chapters, notes, concepts. A decision is needed on [entry](../glossary.md) structure.

Three main models:

1. **Fixed imposed sections**: a frozen set of headings, identical for everyone, with required fields. Structuring but rigid, and often unsuited to a given project.
2. **100% free wiki**: just text, no structure. Flexible but no queries, no reliable filters, no possible validation.
3. **Typed entities**: each entry carries a `type` that drives a field schema, while keeping a free body.

Storyteller must both enable usable views/filters (thus structure) and stay adaptable from one novel to another (thus modular).

## Decision

**Entities are typed and modular.**

- Each [entry](../glossary.md) carries a `type` field (required) that **drives its [frontmatter](../data-model.md) schema**; the markdown **body** remains **free**. No prose templates, no guided questionnaires.
- Types are **activatable/deactivatable** per project via `enabled_types` in `.storyteller/config.yaml`.
- A catalog of **~11 default types**: `project`, `character`, `location`, `faction`, `object`, `culture`, `system`, `species`, `chapter`, `note`, `concept`. Field details in [data model](../data-model.md).
- The `concept` type serves as a **structured catch-all** (common fields only) for anything that doesn't fit a dedicated type, without falling back to formless wiki.

## Consequences

- **Views and filters possible**: because entries are typed, the app can offer filterable lists/tables by type and fields. See [features](../features.md).
- **Adaptability**: each project enables only the useful types; a contemporary chamber drama can disable `species` or `system`.
- **Free body preserved**: structure lives in frontmatter; prose remains entirely free, consistent with rejecting imposed templates.
- **Escape-hatch flexibility**: `concept` (and `note`) absorbs unforeseen cases without forcing a new type.
- **Constraints**: type schemas must be defined and versioned (`schema_version`), and enable/disable as well as unknown fields must be handled properly, in line with non-destructive writing ([ADR 0002](0002-markdown-source-of-truth.md)).
- The question of a stable `id` per entry and that of **user-defined custom types** are not decided here: see open questions in [data model](../data-model.md) and [roadmap](../roadmap.md) (v2).

## Alternatives Considered

- **Fixed imposed sections**: too rigid, unsuited to varied projects, and contrary to rejecting templates/questionnaires. Rejected.
- **100% free wiki**: no exploitable structure for views, filters, and validations. Rejected.
- **Typed, modular entities** (retained): compromise between structure (type-driven frontmatter) and freedom (free body, activatable types, catch-all type).
