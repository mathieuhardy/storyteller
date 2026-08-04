# ADR 0002 — Markdown Is the Sole Source of Truth

- **Status**: Accepted
- **Date**: 2026-08-04

## Context

Storyteller must durably store the context of a novel: typed [entries](../glossary.md), links, [assets](../glossary.md). Two main storage families are possible.

1. A **database** (proprietary or embedded) as the authoritative store, with possibly a secondary markdown export.
2. **Raw markdown files** as the authoritative store, with a technical index rebuilt from them.

The stakes: data longevity (a novel is worked on for years), the author's freedom to edit entries with other tools (Obsidian, vim, mobile editor), portability and backup (Git, folder copy), and no proprietary lock-in.

## Decision

**Raw markdown is the sole source of truth.** See also [principles](../principles.md).

- Each [entry](../glossary.md) is a standalone `.md` file: [frontmatter](../data-model.md) YAML (type-driven fields) + free-form markdown **body**.
- The [index](../glossary.md) (`.storyteller/cache.sqlite`) is a **disposable cache**, 100% rebuildable from the files. It is never authoritative and is gitignored.
- App writes are **non-destructive**: preserve unknown YAML keys, existing key order, and body as-is, so that external editing remains safe. See [principles](../principles.md) and [architecture](../architecture.md).

## Consequences

- **Longevity and no lock-in**: data remains readable and editable without Storyteller, indefinitely.
- **Safe external editing**: Obsidian, a text editor, or a mobile editor can modify the files; a [watcher](../glossary.md) detects changes and rebuilds the index. See [architecture](../architecture.md).
- **Trivial backup and versioning**: a project is a folder; Git or a simple copy suffices (see [ADR 0004](0004-one-folder-per-project.md)).
- **Accepted technical constraints**: the engine must parse/serialize YAML non-destructively (preserving order and unknown keys), handle concurrency between app and external editor, and be able to regenerate the index entirely. Search and filters go through the index, which must stay consistent with the files.
- The index can be deleted without data loss: it rebuilds.

## Alternatives Considered

- **Proprietary authoritative database** (SQLite/Postgres as truth, markdown as export): better "ready-to-use" query performance and simple transactional writes, but creates a proprietary lock-in, makes external editing dangerous or impossible, and weakens longevity. Contrary to the product's purpose. Rejected.
- **Binary format or document database**: even more closed, no external editing possible. Rejected outright.
- **Authoritative markdown + cache index** (retained): requires engineering effort on non-destructive writing and index rebuild, but guarantees longevity, portability, and editing freedom.
