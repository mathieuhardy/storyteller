# Glossary

Shared vocabulary for humans and agents on the **Storyteller** project. These terms are canonical: they are used identically throughout documentation and code. Definitions are listed alphabetically.

> Cross-cutting convention: [markdown is the sole source of truth](principles.md); everything derived (see [index](#index)) is a disposable, rebuildable cache.

| Term | In brief |
| --- | --- |
| [alias](#alias) | Alternative name for an entry, valid link target |
| [asset](#asset) | Media file stored in `assets/` |
| [backlink](#backlink) | Incoming link, automatically computed |
| [body](#body) | Free-form markdown text after the frontmatter |
| [entity](#entity) | Bible object, materialized by an entry |
| [entry](#entry) | Markdown file = frontmatter + body |
| [frontmatter](#frontmatter) | YAML block at the top of an entry |
| [index](#index) | Rebuildable SQLite cache from entries |
| [project](#project) | Self-contained markdown folder for a novel |
| [slug](#slug) | ASCII filename = entry identity |
| [source of truth](#source-of-truth) | The markdown files themselves |
| [stub](#stub) | Link target without an existing entry |
| [type](#type) | Entity schema (drives the fields) |
| [view](#view) | Filterable list/table of entries |
| [watcher](#watcher) | File system surveillance |
| [wikilink](#wikilink) | `[[Target]]` link between entries |

---

## alias

Alternative name for an [entry](#entry), declared in the common `aliases` field of the [frontmatter](#frontmatter). An alias is a valid target for link resolution: when renaming an entry, the app may keep the old [title](#entry) as an alias so that existing [wikilinks](#wikilink) continue to resolve. See [linking](linking.md).

## asset

Media file (image, map, attachment) stored in the `assets/` folder of the [project](#project), typically `assets/images/` or `assets/maps/`. An asset is referenced from an entry by a markdown image (`![[file]]` or `![](assets/...)`) or by a dedicated field; a map is a simple static image, without interactive pins. See [data model](data-model.md).

## backlink

Incoming link to an [entry](#entry): the set of entries that cite it via a [wikilink](#wikilink). Backlinks are automatically computed from the [index](#index) and displayed in a dedicated panel; they are not written in the markdown. See [linking](linking.md).

## body

The **free-form** markdown part of an [entry](#entry), located after the [frontmatter](#frontmatter). The body contains the prose, notes, and [wikilinks](#wikilink) written by the user; it is not subject to any template or imposed questionnaire, and the app preserves it as-is during writes. See [data model](data-model.md).

## entity

An object from the novel's world managed by Storyteller — character, location, faction, object, culture, system, species, chapter, note, concept, or the project itself. Each entity is materialized by exactly one [entry](#entry) and belongs to a [type](#type). See [data model](data-model.md).

## entry

Markdown file representing a single [entity](#entity): a YAML [frontmatter](#frontmatter) followed by a free-form [body](#body). The identity of an entry is its filename (ASCII kebab-case slug); link resolution proceeds by filename, then [title](#entry), then [alias](#alias). See [data model](data-model.md).

## frontmatter

YAML block at the top of an [entry](#entry), delimited by `---`, which holds the structured fields. It contains common fields (`type`, `title`, `aliases`, `tags`, `cover`, `created`, `updated`) and fields specific to the [type](#type); links are stored there as [wikilink](#wikilink) strings (e.g., `pov: "[[Aria]]"`). App writes are non-destructive: unknown keys, key order, and values are preserved. See [data model](data-model.md).

## index

Search and navigation cache (file `.storyteller/cache.sqlite`) built from the entries: it speeds up [views](#view), filters, full-text search, and [backlink](#backlink) computation. The index is disposable, gitignored, and 100% rebuildable; it is **never** a [source of truth](#source-of-truth). See [architecture](architecture.md).

## project

One novel = one self-contained markdown folder, portable and Git-versionable. It groups the `project.md` entry, subfolders by [type](#type), the `assets/` folder, and the technical `.storyteller/` folder (`config.yaml` with `enabled_types`/`schema_version`, and the [index](#index) `cache.sqlite`). Storyteller supports multi-project. See [data model](data-model.md).

## slug

Filename of an [entry](#entry), in ASCII `kebab-case` without extension (e.g., `aria-solane`). The slug is the **identity** of the entry: it serves as a stable anchor for link resolution and remains valid on all file systems (including mobile). The displayed [title](#entry) is distinct and can be modified without renaming the file. See [data model](data-model.md).

## source of truth

Cardinal principle of the project: **markdown files are the sole source of truth**. All derived data ([index](#index), [backlinks](#backlink), search results) is rebuildable from these files, which ensures that external editing (Obsidian, vim, mobile) remains safe. See [principles](principles.md).

## stub

[Wikilink](#wikilink) target that does not yet correspond to any existing [entry](#entry) (neither by filename, nor by [title](#entry), nor by [alias](#alias)). A stub is detected and signaled by the app, which offers to create the missing entry from the link. See [linking](linking.md).

## type

Schema of an [entity](#entity), declared by the required `type` field of the [frontmatter](#frontmatter). The type drives the expected fields of the entry and its folder; types are modular (activatable/deactivatable via `enabled_types`). The ~11 default types: `project`, `character`, `location`, `faction`, `object`, `culture`, `system`, `species`, `chapter`, `note`, `concept`. A project may also declare its own **custom types** in `.storyteller/types.yaml`, merged into the same catalog ([ADR 0017](adr/0017-custom-types.md)). See [data model](data-model.md).

## view

Filterable presentation of entries — list or table — built from the [index](#index), typically filtered by [type](#type), then sorted and refined by tags or fields. Views (and saved views) are derived displays; the link graph is out of MVP scope (v2). See [features](features.md).

## watcher

Component that monitors the [project](#project) file system to detect external entry modifications (editing by another tool) and trigger incremental [index](#index) updates. It ensures the app stays synchronized with the [source of truth](#source-of-truth). See [architecture](architecture.md).

## wikilink

Link between entries written `[[Target]]`, or `[[Target|display text]]` for a custom label. It can appear in the [body](#body) as well as in the [frontmatter](#frontmatter) (as a string, where the field name carries the semantics: `pov`, `parent`, `owner`, `leader`…). Resolution follows the order filename → [title](#entry) → [alias](#alias), otherwise the target becomes a [stub](#stub). See [linking](linking.md).

---

Related documents: [vision](vision.md) · [principles](principles.md) · [features](features.md) · [data model](data-model.md) · [linking](linking.md) · [architecture](architecture.md) · [API](api.md) · [roadmap](roadmap.md) · [usage](usage.md)
