# ADR 0010 — Frontmatter Keys in English, Content in French

- **Status**: Accepted
- **Date**: 2026-08-04

## Context

Storyteller stores each [entity](../glossary.md) as a markdown file whose [frontmatter](../glossary.md) YAML holds the structured fields. Two audiences coexist: **French-speaking authors** (who read and write prose, values, UI labels) and **code / third-party tools** (Rust backend, Obsidian-type editors, scripts) that manipulate field keys.

The language of frontmatter **keys** (`type`, `title`, `role`, `pov`…) must be decided independently of **content** language. Keys serve as stable identifiers shared between the on-disk format, [index](../glossary.md), API, and code; content is creative data intended for a French-speaking readership.

Mixing the two (accented French keys, or content forced to English) creates friction: fragile accents and spaces in YAML, inconsistencies between code and files, and an unnatural writing experience for the author.

## Decision

**Frontmatter keys are in English `snake_case` (ASCII); UI labels, values, and prose are in French.**

- Field keys (common and type-specific) are stable ASCII identifiers: `type`, `title`, `aliases`, `tags`, `role`, `pov`, `location_kind`, `chapter_status`…
- **Values** remain in French (e.g., `role: protagoniste`, `chapter_status: brouillon`), as does the entire markdown [body](../glossary.md).
- The **interface** translates keys to French labels for display; the stored key never changes.

See [data model](../data-model.md) (§ common fields and type catalog).

## Consequences

- **Stability and interoperability**: ASCII keys are robust in YAML, consistent with Rust code identifiers, and compatible with third-party tools (Obsidian, vim) without accent or encoding issues.
- **Author experience preserved**: the author reads and writes French only where it matters (values, prose, UI labels).
- **Label layer to maintain**: the interface must provide an English key → French label mapping (and associated documentation). Modest, localized cost.
- **French enums**: since enumeration values are in French, any technical mapping (e.g., sorting, icons) is done on the value as written; we avoid duplicating linguistic variants.

## Alternatives Considered

- **Everything in French (including keys)**: more "natural" looking in the raw file, but accented/spaced keys are fragile in YAML, misaligned with code identifiers, and awkward for third-party tools. Rejected.
- **Everything in English (including content)**: maximum code consistency, but imposes a foreign language on the author for their creative data — contrary to the target audience. Rejected.
- **Keys EN / content FR** (retained): separates the stable technical identifier from creative data, at the cost of a simple label layer in the UI.
