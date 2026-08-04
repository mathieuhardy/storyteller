# ADR 0007 — Static Maps Without Interactive Pins

- **Status**: Accepted
- **Date**: 2026-08-04

## Context

Fiction worlds often rely on **maps**. A bible tool can either just display a map image, or offer **interactive pins**: clickable markers placed on the map and linked to location [entries](../glossary.md), with coordinates, zoom, layers.

Interactive pins require storing coordinates and an overlay model. But this data doesn't fit well in **flat markdown**: it would require a proprietary coordinate format, tight coupling between the image and an interactive state, and a dedicated editing interface — all hard to maintain in a readable, hand-editable way, in tension with markdown source of truth ([ADR 0002](0002-markdown-source-of-truth.md)).

## Decision

**Maps are referenced static images; no interactive pins.**

- A map is an [asset](../glossary.md) image (in `assets/maps/`) referenced from an entry, typically via the `map` field of the `location` type. See [data model](../data-model.md).
- The connection between a map and the locations it represents uses existing mechanisms: [wikilinks](../glossary.md) and [backlinks](../glossary.md) between entries ([ADR 0006](0006-wikilinks-backlinks-stubs.md)), not positioned markers.
- Interactive pins are **out of scope**.

## Consequences

- **Simplicity and markdown compatibility**: a map is just an image plus text links; everything remains readable and editable outside the app.
- **No proprietary coordinate format** to define or maintain; no marker editing interface.
- **Link-based navigation**: we connect a map and its locations via wikilinks/backlinks rather than visual position.
- Accepted consequence: no hover or click on map points to open an entry. An author who wants to visually annotate a map does so in their image editor beforehand.
- Reversibility: if pins were ever desired, a storage model compatible with markdown source of truth would be needed, decided in a new ADR.

## Alternatives Considered

- **Interactive pins with coordinates**: visually rich, but poorly representable in flat markdown and costly (coordinate format, image/state coupling, dedicated editor). Rejected for incompatibility with source of truth principle and for cost.
- **Map as referenced static image** (retained): minimal, markdown-compatible, linked to entries via existing links.
