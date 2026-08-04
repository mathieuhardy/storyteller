# ADR 0004 — One Self-Contained Folder per Project

- **Status**: Accepted
- **Date**: 2026-08-04

## Context

An author may work on multiple novels at once. A decision is needed on the storage unit for a **project** and multi-project management.

Two approaches: a **central store** (a single database or application folder grouping all projects, indexed by identifier), or **one self-contained folder per project**, where all of a novel's content lives in a portable directory.

The central store eases some global operations but couples projects together, complicates selective backup, and makes data dependent on an application location. It conflicts with markdown source of truth ([ADR 0002](0002-markdown-source-of-truth.md)).

## Decision

**Each project is a self-contained, portable markdown folder.** Multi-project means simply opening multiple such folders.

Reference project structure:

```
my-novel/
  project.md                    (project entry)
  .storyteller/config.yaml      (enabled_types, schema_version)
  .storyteller/cache.sqlite     (index, gitignored)
  characters/ locations/ factions/ objects/ cultures/
  systems/ species/ chapters/ notes/ concepts/
  assets/images/  assets/maps/
```

- The `.storyteller/` folder holds project configuration (`config.yaml`: `enabled_types`, `schema_version`) and the [index](../glossary.md) (`cache.sqlite`, gitignored, rebuildable). See [data model](../data-model.md) and [architecture](../architecture.md).
- Moving, copying, archiving, or versioning a project amounts to manipulating its folder.

## Consequences

- **Total portability**: a project is moved, backed up (Git, copy), and archived in a single folder operation. Consistent with [ADR 0002](0002-markdown-source-of-truth.md).
- **Isolation**: projects have no cross-dependencies; deleting one project doesn't affect others.
- **Self-describing**: config lives in the project (`enabled_types`, `schema_version`), so the folder remains interpretable independently of the machine.
- **Simple multi-project**: no mandatory global registry; the app just opens a folder. A possible "recent projects" remains a client-side convenience, not authoritative.
- **Relative paths**: [wikilinks](../glossary.md) and asset references stay internal to the project, ensuring the folder works wherever it's moved. See [linking](../linking.md).
- Accepted consequence: truly cross-project operations (inter-project search, entry reuse between novels) are not MVP goals.

## Alternatives Considered

- **Central application store** (a single database or folder for all projects): simplifies some global views but couples projects, complicates selective backup, and ties data to the app location. Contrary to targeted portability. Rejected.
- **Per-project database**: would break markdown source of truth ([ADR 0002](0002-markdown-source-of-truth.md)). Rejected.
- **Self-contained folder per project** (retained): maximizes portability and isolation, at the cost of no tooled inter-project operations.
