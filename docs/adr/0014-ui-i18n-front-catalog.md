# ADR 0014 — UI language via a front-side i18n catalog

- **Status**: Accepted
- **Date**: 2026-08-07

## Context

The interface must offer a **language switcher**, starting with **EN** and **FR** (see
[ui/i18n.md](../ui/i18n.md)). This governs the application **chrome** and the **displayed labels** of types
and fields — never the author's content, which stays as written (a decision aligned with
[ADR 0010](0010-frontmatter-keys-en-content-fr.md): frontmatter **keys** are English, **content** is French).

Today `GET /types` returns a single French `label` per type and field ([api.md](../api.md)). To render an EN
interface, the localized labels have to come from somewhere. Two families of solution exist: serve them from
the **backend** (localized responses), or hold them in a **frontend catalog**. We must decide before M4 so the
component layer and the type-driven forms are built on a stable assumption.

Relevant constraint: the MVP types are a **fixed, built-in set** of 11 ([data-model.md](../data-model.md)).
User-defined types and custom fields are out of scope for now ("later" in [features.md](../features.md) §B).

## Decision

**The frontend owns an i18n catalog (EN/FR) for both the chrome and the built-in type/field labels.** The
backend is not asked to localize; the language is a **local UI preference**, like the theme, never written to
the project files.

- The catalog holds every UI string: menus, buttons, section headers, state messages, tooltips, and the
  **labels of the built-in types and fields** keyed by their English `name` (`role` → "Rôle" / "Role").
- The frontend keys type/field labels off the **stable English `name`** returned by `GET /types` (not off the
  server's `label`). The server `label` is treated as a fallback only.
- The mono technical key (e.g. `pov`) stays visible next to the localized label, so the link to the `.md`
  file holds in any language ([ui/i18n.md](../ui/i18n.md)).
- Dates and numbers are **formatted** per the chosen language; **stored** dates remain ISO 8601 / RFC 3339.

## Consequences

- **No backend work** for i18n at MVP: no `Accept-Language` handling, no label catalog in `storyteller-core`.
  The server stays a content API; localization is a presentation concern, where it belongs for a
  local-first, single-user tool.
- Works **offline** and keeps the project **portable**: switching language changes nothing on disk.
- The frontend carries the EN/FR label dictionary for the 11 built-in types — acceptable precisely because
  that set is fixed and small.
- **Known limitation / revisit trigger:** if/when **user-defined types or custom fields** land, their labels
  cannot live in a built-in front catalog. That feature must revisit label localization (likely: labels
  carried in the data/config, or a backend-served catalog) and will need its **own ADR**. This ADR is scoped
  to the built-in types.
- `GET /types.label` may stay French-only without blocking the EN UI; if it is ever localized later, the
  frontend can prefer it over its catalog without a breaking change.

## Alternatives Considered

- **Backend-localized `/types`** (via `Accept-Language` or a `lang` param), with the EN/FR label catalog in
  `storyteller-core`. Single source of truth and the natural home if custom types arrive. Rejected for the
  MVP: it adds backend work now for a benefit that only materializes with a feature that is still "later",
  and it pushes a presentation concern into the content core.
- **Hybrid** — chrome translated on the front, type/field labels served by the backend. Rejected: it splits
  translation across two sources of truth for no MVP gain, complicating both sides.
