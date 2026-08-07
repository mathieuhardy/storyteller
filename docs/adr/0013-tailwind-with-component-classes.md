# ADR 0013 — Tailwind, contained by component classes

- **Status**: Accepted
- **Date**: 2026-08-07

## Context

[ADR 0009](0009-rust-sveltekit-stack.md) fixed the stack as **SvelteKit + Shadcn** but left one point
explicitly undecided: the [principles](../principles.md) ask to **avoid Tailwind if possible**, yet the
shadcn-svelte ecosystem is built on Tailwind (its components are Tailwind utilities over Bits UI / Melt
primitives). That open point must be resolved **before** GUI work (M4) starts.

The [UI design work](../ui/README.md) is now done: a direction ("Atelier") with a token system
([foundations.md](../ui/foundations.md)), a component catalogue, and reference mockups written in **plain CSS
driven by custom properties**. Whatever styling approach we pick must carry those tokens and both themes
(light/dark) faithfully.

The concrete worry with Tailwind is not the utility model itself but its failure mode: **class soup** —
dozens of utilities inlined on every element, markup that becomes unreadable and impossible to keep
consistent. We need Tailwind's velocity and its ecosystem (shadcn-svelte) without that outcome.

## Decision

**We use Tailwind CSS, but utilities are contained — they do not sprawl across application markup.** The
discipline is:

1. **Components encapsulate utilities.** UI primitives are Svelte components (the shadcn-svelte model:
   Bits UI / Melt headless behaviour + Tailwind styling *inside* the component). Application code uses
   `<Button variant="primary">`, not a string of thirty classes. Utility clusters live in one place, behind
   a typed prop surface.
2. **Recurring clusters become semantic classes.** When the same utility cluster repeats, it is extracted
   with `@apply` into a named class under `@layer components` (e.g. `.field-label`, `.link-resolved`), so the
   intent is named and edited once. Inline utilities remain fine for genuinely local, one-off layout.
3. **Design tokens drive the theme.** The [foundations.md](../ui/foundations.md) tokens (colours, radii,
   shadows, type) are declared as CSS custom properties and exposed to Tailwind's theme, so values are never
   hard-coded as literal utilities and both themes switch through the tokens (`prefers-color-scheme` +
   `data-theme` override). Semantic link/diagnostic states map to token-backed classes, not ad-hoc colours.

The rule of thumb: **if a class list stops being readable at a glance, it becomes a component or an
`@apply` class.** Markup should read like the design, not like a stylesheet.

## Consequences

- The **Shadcn/Tailwind open point of [ADR 0009](0009-rust-sveltekit-stack.md) is resolved**: shadcn-svelte
  is adopted as intended, on Tailwind.
- We gain component velocity and a large, documented ecosystem; accessibility primitives (focus, dialogs,
  combobox) come from Bits UI / Melt rather than being hand-rolled.
- We accept a **build-time CSS toolchain** (Tailwind + PostCSS) and a Tailwind config as part of the frontend.
- The [principles](../principles.md) preference "avoid Tailwind if possible" is **consciously overridden**
  here, in exchange for the ecosystem — the containment discipline above is the price we pay to keep it sane.
  This note documents that trade-off so it is not silently reopened.
- The reference mockups stay in **plain CSS**; they are a visual/token reference, not the component source.
  Their tokens are ported into the Tailwind theme rather than copied as literals.
- A reviewable line exists: PRs that inline large utility clusters in app markup (instead of a component or
  `@apply` class) are rejected on that basis.

## Alternatives Considered

- **Bits UI / Melt + vanilla CSS, no Tailwind.** Would honour the "avoid Tailwind" preference literally and
  transpose the plain-CSS mockups directly. Rejected: it forgoes the shadcn-svelte ecosystem and puts the
  full component/styling burden on us, for a purity gain we judged not worth the velocity cost.
- **Fully bespoke components, headless lib only where critical.** Maximum control, zero UI dependency, but
  the most work and the a11y burden entirely on us. Rejected for the MVP on cost grounds.
- **Tailwind used freely, utilities inlined everywhere.** The default Tailwind experience. Rejected: it is
  exactly the class-soup failure mode we want to avoid — unreadable markup, no single place to keep recurring
  patterns consistent.
