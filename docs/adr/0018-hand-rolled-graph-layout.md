# ADR 0018 — The link graph uses a hand-rolled force layout, no charting/graph library

- **Status**: Accepted
- **Date**: 2026-08-25

## Context

[ADR 0008](0008-no-graph-in-mvp.md) deferred the link graph to v2 (M7), noting it is "a separate workstream: layout engine, performant rendering on large projects, filtering, interactions, and real interface design effort" — but also that the backlinks index already has the data, so v2 "just" needs a visualization layer. `GET /graph` (`docs/api.md` §3) now exists, sourced directly from the index's `links` table: every entry as a node, every **resolved**, deduplicated entry-to-entry link as an edge.

The remaining piece is positioning: a graph library's whole job is turning a node/edge list into readable, non-overlapping coordinates — usually via a **force-directed layout** (nodes repel each other, edges pull their endpoints together, iterate until it settles). The question is where that layout computation comes from.

## Decision

**A small, hand-rolled Fruchterman-Reingold force simulation** (`frontend/src/lib/graph/layout.ts`), run once per graph load — no `d3-force`, `cytoscape.js`, or similar dependency. It is a pure function, `(nodes, edges) → Map<slug, {x, y}>`: repulsion between every node pair, attraction along edges, a cooling "temperature" that shrinks step size over ~200 iterations, positions clamped to a frame. The frontend renders the result as plain SVG (`docs/ui/screens.md` §7) — circles for nodes, lines for edges, colored with the existing Atelier tokens (`--accent`/`--danger`/`--faint`, `docs/ui/foundations.md`) rather than a new per-type categorical palette, consistent with how the dashboard's distribution chart already treats every type with one accent hue.

## Consequences

- **Zero new frontend dependencies.** The entire feature — data, layout, rendering, interaction (hover to highlight neighbors, click to navigate, wheel/drag pan-zoom) — is core SvelteKit + plain SVG + `frontend/src/lib/graph/layout.ts`. No new package to keep updated, audit, or theme-match.
- **O(n²) per iteration is the accepted ceiling.** Every pair of nodes repels every other pair, each of the ~200 iterations. At the local/single-user scale this app targets (ADR 0003) — realistically tens to a few hundred entries — this runs well under a second in the browser. A project large enough to make this the bottleneck would already be straining the force-directed *concept* (an unreadable hairball of nodes), not just this implementation; that's a "revisit if it happens" problem, not a design-time one.
- **No stable, reproducible layout across reloads.** Positions are recomputed from a random initial placement every time the graph screen loads — panning/zooming state is not preserved either. Acceptable for a first cut: the graph is for *exploring* connectivity, not a diagram meant to be memorized node-by-node.
- **Simpler than a general-purpose graph library, but also less capable.** No built-in clustering, no WebGL renderer for very large graphs, no drag-to-reposition-and-pin. If the graph screen needs those later (dragging nodes, saved layouts, thousands of nodes), that is the point to reconsider a real graph library — this ADR does not foreclose it, the same way ADR 0008 didn't foreclose building the graph at all.

## Alternatives Considered

- **`d3-force`.** The standard choice for exactly this — battle-tested, handles edge cases (disconnected components, collision) better than a hand-rolled version. Rejected for this pass: it's a real dependency (and drags in d3's module ecosystem conventions) for a feature whose v1 scope is "show the network, click to navigate" — the ADR 0008 goal of "just a visualization layer," not a data-viz platform. Worth adopting if the graph screen grows real interaction requirements (pinning, clustering, incremental layout on partial updates).
- **A full graph UI library (`cytoscape.js`, `vis-network`).** Rejected: these bring their own rendering, event, and styling systems that would sit awkwardly next to the existing hand-styled Svelte components (`docs/ui/components.md`) — every other screen in the app is plain Svelte + the Atelier CSS tokens, not a third-party widget.
- **Server-side layout** (compute positions in `storyteller-core`, serve them alongside `nodes`/`edges`). Rejected: layout is a *display* concern with no bearing on the data model, and the client already has to re-layout on every window resize anyway; keeping it client-side matches "the GUI is designed after the docs, and consumes a stable API" (`AGENTS.md`) — `GET /graph`'s contract (nodes + edges) stays reusable by `storyteller-tauri` (M7) or any future consumer regardless of which layout algorithm renders it.
