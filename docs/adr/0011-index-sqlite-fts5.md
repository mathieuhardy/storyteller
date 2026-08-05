# ADR 0011 — SQLite + FTS5 as the Index Engine

- **Status**: Accepted
- **Date**: 2026-08-05

## Context

[Architecture](../architecture.md) §6 left the index engine open: **SQLite + FTS5** (expected) vs. **Tantivy**. Milestone [M1](../roadmap.md) cannot be built without choosing, since it delivers the [index](../glossary.md) itself, and the rule is that code never precedes the decision.

The constraints are set by decisions already taken. The index is a **disposable cache** ([ADR 0002](0002-markdown-source-of-truth.md)): it is rebuilt from the markdown files, so durability and migration guarantees matter far less than rebuild speed. It must serve two different needs from one store: **structured queries** (filter by [type](../glossary.md), tags, arbitrary frontmatter fields, sort, paginate — [API](../api.md) §4) and, from [M5](../roadmap.md), **full-text search** over titles, aliases, tags, and bodies. It must run **offline** on a single-user machine, embedded in a library with no server process, and it must ship to the same targets as the rest: Docker, Nix, and a Tauri bundle including an Android path ([ADR 0009](0009-rust-sveltekit-stack.md)).

## Decision

**We use SQLite with the FTS5 extension as the sole index engine**, in `.storyteller/cache.sqlite`.

- **One store for both needs**: relational tables for entries, aliases, tags, flattened frontmatter fields, and links/backlinks; an FTS5 virtual table for full-text search (M5).
- **SQLite is bundled** (compiled into the binary) rather than linked against a system library, so behaviour is identical across Docker, Nix, desktop, and mobile targets.
- **The schema is versioned but never migrated.** A cache written by another layout is **dropped and rebuilt** from the files. This is always correct, and it is the concrete expression of "the index is never authoritative".
- **Change detection** relies on **mtime + size + content hash** (sha256) recorded per entry, which is what lets the [watcher](../glossary.md) reindex incrementally (M2) instead of rescanning everything.

## Consequences

- **One dependency, two capabilities.** Structured filters and full-text search share a single file, a single transaction model, and a single rebuild path. No second index to keep consistent with the first.
- **Rebuild is the recovery strategy for everything.** Corrupted cache, schema change, divergence with the files: the answer is always "rebuild". No migration code, ever.
- **Joins are available**, which suits [backlinks](../glossary.md) (`links` joined to `entries`) and field filters expressed as correlated subqueries.
- **FTS5 ranking is basic** compared to a real search engine: no fuzzy matching, no custom scoring beyond BM25. Accepted — this is a personal bible of a few hundred entries, not a search product.
- **A C dependency enters the build.** Bundled SQLite needs a C compiler, which adds a constraint to the Nix and Android toolchains. Accepted as the price of a single, uniform engine.
- **The cache must be gitignored.** The application writes `.storyteller/.gitignore` itself, so a project folder stays committable as-is without the user having to know.

## Alternatives Considered

- **Tantivy** (Rust full-text search engine): better ranking and a pure-Rust build, but it only covers the *search* half of the problem. Structured filters, sorts, and backlink joins would still need a second store, so the cost is two engines and two rebuild paths for a benefit (search quality) that a few hundred entries never make visible. Rejected.
- **SQLite for structure + Tantivy for search**: the explicit two-engine version of the above. Rejected for the same reason, with the added risk of the two stores diverging.
- **In-memory index only, rebuilt at every launch**: attractive since the files are the source of truth anyway, and it removes the cache file entirely. Rejected because startup cost then grows with project size on every launch, and the [watcher](../glossary.md)'s incremental reindexing (M2) has nothing to be incremental against.
- **No index, querying the files directly**: simplest possible, but every filtered [view](../glossary.md) becomes a full scan and parse of the project. Rejected on performance grounds.
