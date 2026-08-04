# ADR 0001 — No Timeline or Narrative Chronology

- **Status**: Accepted
- **Date**: 2026-08-04

## Context

Many novel "bible" tools offer a **timeline**: a time axis on which events, scenes, and diegetic dates (those of the story) are placed to verify chronological consistency.

This feature is appealing but costly. It assumes a rich temporal model: imaginary calendars, fuzzy or relative dates ("three winters earlier"), simultaneous events, ellipses, flashbacks. It induces a second data structure to maintain alongside [entries](../glossary.md), a dedicated visualization interface, and significant input effort for the author. Yet Storyteller aims to store and organize the **context** of a novel, not to model its temporal mechanics.

A decision is needed: do we integrate a structured narrative time concept, or stay descriptive?

## Decision

**Storyteller has no timeline or narrative chronology.** Time is not an axis of the application.

- Dates present in [entries](../glossary.md) remain **simple descriptive fields** (free text or values in the [frontmatter](../data-model.md)), without temporal semantics interpreted by the app.
- The `created` / `updated` fields managed by the application are **technical metadata** (file creation/modification date), **never** a story chronology.
- The `order` field of the `chapter` type designates the **position in the manuscript**, not a point on a time axis. See the [data model](../data-model.md).

## Consequences

- The data model stays simpler and the product remains focused on context organization.
- No timeline interface to design, no imaginary calendar model to maintain.
- An author who wants to document a chronology can still do so freely in the **body** markdown of a [note](../glossary.md) or `concept` entry (e.g., a hand-written timeline), without tool support.
- Accepted consequence: no automatic temporal consistency checking, no "timeline" view. If a strong need emerged, it would be the subject of a new ADR explicitly reopening this.

## Alternatives Considered

- **Structured timeline with imaginary calendars**: the most powerful, but by far the most complex to model and maintain; outside the scope of an MVP centered on context. Rejected for cost and product drift risk.
- **"Smart" date field per entry** (sortable and correlatable diegetic dates): imposes a date syntax and app interpretation, thus a disguised timeline. Rejected for the same reason.
- **Free chronology in markdown body**: retained de facto as an unsupported solution — this is already what the free body of entries allows, with no app commitment.
