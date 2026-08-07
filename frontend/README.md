# storyteller — frontend

SvelteKit + Tailwind front for **Storyteller** (milestone [M4](../docs/roadmap.md)). It is a pure
consumer of the [HTTP API](../docs/api.md); it holds **no business logic** (see
[architecture.md](../docs/architecture.md) §3). Visual direction, tokens and components are specified
under [docs/ui/](../docs/ui/README.md) ("Atelier").

## Stack

- **SvelteKit** (SPA via `@sveltejs/adapter-static`, `ssr = false`): the built site is served
  statically by `storyteller-server` (M6), or embedded by `storyteller-tauri` (M7).
- **Tailwind v4**, contained by component classes ([ADR 0013](../docs/adr/0013-tailwind-with-component-classes.md)):
  the [Atelier tokens](../docs/ui/foundations.md) are CSS custom properties in `src/app.css`,
  exposed to Tailwind's theme; components encapsulate their utilities.
- **i18n** EN/FR via a front catalog ([ADR 0014](../docs/adr/0014-ui-i18n-front-catalog.md)),
  keyed off the stable English type/field `name`.

## Layout

```
src/
  app.css                    design tokens (light/dark) + Tailwind theme
  lib/
    api/       client.ts     transport-agnostic API client (fetch → /api/v1)
               events.ts     SSE client (watcher change stream)
               types.ts      TypeScript mirror of the API JSON
    i18n/      en.ts/fr.ts   catalogs; index.svelte.ts = locale + t()
    stores/    theme.svelte.ts   system/light/dark preference
    components/                Topbar, Nav, Rail, Button, Icon, …
  routes/
    +page.svelte             launcher (recent projects, open a folder)
    (app)/                   application shell (topbar · nav · rail) + screens
```

## Develop

The dev server proxies `/api` to the backend, so both run same-origin (no CORS):

```sh
# 1. backend, in the repo root
cargo run -p storyteller-server -- --project tests/fixtures/sample-project

# 2. frontend, here
npm install
npm run dev        # http://localhost:5173
```

## Check & build

```sh
npm run check      # svelte-check (types)
npm run build      # static build into ./build
```
