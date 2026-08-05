# Data Model

This document is the normative reference for **Storyteller** storage. It defines how an **entry** is written to disk, what fields each **type** carries, and how the app reads and rewrites these files without ever corrupting them. Link details (resolution, backlinks, stubs) are covered in [linking.md](linking.md); undecided points are referred to [the ADRs](adr/README.md).

> **Cardinal reminder.** **Markdown is the sole source of truth.** The [index](glossary.md) (`cache.sqlite`) is a disposable cache, 100% rebuildable from the files. All app writes are **non-destructive**.

---

## 1. Storage Principles

| Principle | Rule |
|---|---|
| **Source of truth** | The `.md` files are authoritative. The index is just a rebuildable cache; it can be deleted without loss. |
| **One entry = one file** | Each [entity](glossary.md) is a single `.md` file. No multi-file entries, no multi-entry files. |
| **One project = one folder** | A [project](glossary.md) is a self-contained markdown folder, portable and Git-versionable as-is. |
| **Faithful round-trip** | Reading then rewriting an entry without modification produces an identical file (same keys, same order, same body). See §6. |
| **Keys EN / content FR** | [Frontmatter](glossary.md) keys are in English `snake_case`; UI labels, values, and prose are in French. See [ADR 0010](adr/0010-frontmatter-keys-en-content-fr.md). |
| **Safe external editing** | An entry remains editable in Obsidian, vim, or a mobile editor; the app requires no proprietary format. |

An entry is always composed of two parts:

1. the **frontmatter** YAML delimited by `---`, whose fields are **driven by the type**;
2. the **body** markdown which is **free-form**: unconstrained prose, no imposed templates or questionnaires.

```markdown
---
type: character
title: Aria Solane
---
Free-form markdown body. Write anything here, with [[wikilinks]].
```

---

## 2. Common Fields for All Entries

Every type inherits these fields. The `type` field drives the schema; the others are shared.

| Field | Value | Tier | Note |
|---|---|---|---|
| `type` | enum (one of 11 types) | **MVP** (required) | Determines the schema and folder. Without it, the entry is treated as `note`. |
| `title` | text | **MVP** | Displayed name, freely modifiable. Distinct from filename (see §3). |
| `aliases` | list of texts | **MVP** | Alternative names; used for link resolution and as a safety net during renaming. |
| `tags` | list of texts | **MVP** | Free-form labels, cross-type. |
| `cover` | image (relative path) | opt | Main illustration for the entry (e.g., `assets/images/aria.jpg`). |
| `created` | datetime (ISO 8601 / RFC 3339) | **MVP** | **Managed by the app.** File creation timestamp (e.g., `2026-07-12T09:15:00Z`). |
| `updated` | datetime (ISO 8601 / RFC 3339) | **MVP** | **Managed by the app.** Last modification timestamp. |

### Dates Are Never a Narrative Chronology

`created` and `updated` describe **the file**, not the story being told. They are technical metadata (when the entry was created/modified in the tool). Storyteller **has no timeline**: no field models the order of novel events. The only existing ordering field, `order` on `chapter`, is a **position in the manuscript**, not a time axis (see §4).

---

## 3. Identity & Naming

### Filename (slug) vs `title`

The **identity** of an entry is its **filename**, an ASCII `kebab-case` slug (e.g., `aria-solane.md`). It is stable and serves as the anchor for links. The `title` (e.g., `Aria Solane`) is the **displayed** name, modifiable at any time without renaming the file.

### Link Resolution (summary)

A [wikilink](glossary.md) `[[Target]]` is resolved in this order:

1. **filename** (slug);
2. **`title`**;
3. **`aliases`**;
4. otherwise → **[stub](glossary.md)** (target not yet created, signaled by the app).

Full details (ambiguity, case, creation from stub) are in [linking.md](linking.md).

### Renaming & Alias Safety Net

Renaming an entry (changing its slug or `title`) can break incoming links. The app:

- **updates incoming links** to the new target;
- **and/or keeps the old name as an `alias`**, so that links not rewritten (or written by an external tool) continue to resolve.

This safety net ensures that a rename never silently produces a [stub](glossary.md).

### Filename Collision

Since identity **is** the filename, two files sharing a name (e.g. `characters/aria.md` and `notes/aria.md`) claim the same identity. This is a conflict, not a supported layout, and the app resolves it by **signaling, never by hiding**:

- both entries remain listed and indexed — dropping one would be data loss;
- each carries a `duplicate_slug` diagnostic naming the other file(s);
- [wikilinks](glossary.md) to that name resolve as **ambiguous** (see [linking](linking.md) §3.3), so the user disambiguates or renames.

### Stable `id` Path → ADR

A stable `id` field (independent of filename) is a **hardening path** for identity, not imposed in MVP. The decision is deferred to an [ADR](adr/README.md); do not add an `id` until it is decided.

---

## 4. Catalog of the 11 Types

Value type legend: **text**, **number**, **enum** (closed list), **list** (of texts), **link** `[[…]]` (wikilink string), **link-list** (list of wikilink strings), **image** (relative path), **boolean**.

> Links in frontmatter are **always** stored as wikilink strings (e.g., `pov: "[[Aria]]"`, `locations: ["[[Glass City]]"]`). It is **the field name** that carries the link semantics (`pov`, `parent`, `owner`, `leader`…). See [linking.md](linking.md).

Each type inherits the **common fields** (§2); the tables below list only the **specific** fields.

### 4.1 `project`

Root entry of the project (file `project.md`, see §5).

| Field | Value | Tier |
|---|---|---|
| `logline` | text | **MVP** |
| `genres` | list | **MVP** |
| `status` | enum: `idea`, `draft`, `writing`, `revision`, `complete` | **MVP** |

**Body:** short and long summaries of the novel, intent note, pitch, any framing text.

### 4.2 `character`

| Field | Value | Tier |
|---|---|---|
| `role` | text | **MVP** |
| `desire` | text | **MVP** |
| `wound` | text | **MVP** |
| `fear` | text | **MVP** |
| `motivation` | text | **MVP** |
| `need` | text | opt |
| `lie` | text | opt |
| `flaw` | text | opt |
| `arc` | text | opt |
| `species` | link `[[Species]]` | opt |
| `culture` | link `[[Culture]]` | opt |
| `factions` | link-list `[[Faction]]` | opt |
| `home` | link `[[Location]]` | opt |
| `status` | text | opt |
| `portrait` | image | opt |
| `age` | number or text | opt |
| `gender` | text | opt |
| `pronouns` | text | opt |
| `appearance` | text | opt |

**Body:** biography, voice, prose relationships, characterization notes, evolution through the plot.

### 4.3 `location`

| Field | Value | Tier |
|---|---|---|
| `location_kind` | text/enum (city, region, building, planet…) | **MVP** |
| `parent` | link `[[Location]]` | opt |
| `ruling_faction` | link `[[Faction]]` | opt |
| `culture` | link `[[Culture]]` | opt |
| `population` | number or text | opt |
| `climate` | text | opt |
| `map` | image | opt |

**Body:** sensory description, geography, location history, points of interest. The **map** is a **static image** referenced (`map`); no interactive pins (out of scope).

### 4.4 `faction`

| Field | Value | Tier |
|---|---|---|
| `org_kind` | text/enum (guild, kingdom, company, order…) | **MVP** |
| `leader` | link `[[Character]]` | **MVP** |
| `parent` | link `[[Faction]]` | opt |
| `headquarters` | link `[[Location]]` | opt |
| `allies` | link-list `[[Faction]]` | opt |
| `rivals` | link-list `[[Faction]]` | opt |
| `ideology` | text | opt |
| `goals` | list or text | opt |
| `members` | link-list `[[Character]]` | opt |

**Body:** history, internal structure, resources, methods, role in the plot.

### 4.5 `object`

| Field | Value | Tier |
|---|---|---|
| `owner` | link `[[Character]]` or `[[Faction]]` | **MVP** |
| `object_kind` | text/enum | opt |
| `location` | link `[[Location]]` | opt |
| `system` | link `[[System]]` | opt |
| `powers` | list or text | opt |
| `image` | image | opt |

**Body:** description, origin, stakes, detailed powers and constraints.

### 4.6 `culture`

| Field | Value | Tier |
|---|---|---|
| `homeland` | link `[[Location]]` | **MVP** |
| `species` | link-list `[[Species]]` | opt |
| `language` | text | opt |
| `government` | link `[[Faction]]` | opt |
| `values` | list | opt |
| `religion` | text | opt |

**Body:** customs, rites, taboos, myths, aesthetics, relationship to power.

### 4.7 `system`

World rules system (magic, technology, politics, economy, religion).

| Field | Value | Tier |
|---|---|---|
| `system_kind` | enum: `magic`, `technology`, `politics`, `economy`, `religion` | **MVP** |
| `rules` | list | **MVP** |
| `limits` | list | **MVP** |
| `cost` | text | **MVP** |
| `source` | text | opt |
| `practitioners` | link-list `[[Character]]` | opt |

**Body:** explanation of how it works, examples, exceptions, narrative consequences.

### 4.8 `species`

| Field | Value | Tier |
|---|---|---|
| `classification` | text | **MVP** |
| `intelligent` | boolean | opt |
| `habitat` | link `[[Location]]` | opt |
| `abilities` | list | opt |
| `lifespan` | text | opt |
| `image` | image | opt |

**Body:** morphology, behavior, ecology, place in the world.

### 4.9 `chapter`

| Field | Value | Tier |
|---|---|---|
| `order` | number | **MVP** |
| `pov` | link `[[Character]]` | **MVP** |
| `locations` | link-list `[[Location]]` | **MVP** |
| `chapter_status` | enum: `to-write`, `draft`, `written`, `revised` | **MVP** |
| `summary` | text | **MVP** |
| `characters` | link-list `[[Character]]` | opt |
| `plotlines` | list | opt |
| `act` | number or text | opt |
| `part` | number or text | opt |
| `wordcount` | number | opt |

> `order` is the **position in the manuscript**, **not** a time axis. Storyteller describes context, it does not model story chronology (out of scope). The manuscript itself is not written here (no prose editor).

**Body:** detailed synopsis, beats, staging notes, points to verify. **Not** the chapter text.

### 4.10 `note`

| Field | Value | Tier |
|---|---|---|
| `resource_kind` | text/enum | opt |
| `source_url` | text (URL) | opt |
| `related` | link-list | opt |
| `attachments` | list of images/paths | opt |

**Body:** the main content of the note. Catch-all type for research, inspiration, reusable resources.

### 4.11 `concept`

Minimal type: **common fields only**.

| Field | Value | Tier |
|---|---|---|
| `category` | text | opt |
| `related` | link-list | opt |

**Body:** definition and development of the idea, theme, or motif.

---

## 5. Project Directory Structure

```text
my-novel/
  project.md                    # project type entry (root)
  .storyteller/
    config.yaml                 # enabled_types, schema_version
    cache.sqlite                # index (gitignored, rebuildable)
  characters/
  locations/
  factions/
  objects/
  cultures/
  systems/
  species/
  chapters/
  notes/
  concepts/
  assets/
    images/
    maps/
```

### Role of `project.md`

`project.md` is the **root entry** (type `project`) placed at the folder root. It carries the novel's framing (logline, genres, status) and serves as the entry point. It's an entry like any other, just located at the root rather than in a type subfolder.

### Role of `.storyteller/config.yaml`

The technical folder `.storyteller/` holds project configuration and index. `config.yaml` declares the **enabled types** and **schema version**. `cache.sqlite` is the [index](glossary.md): **gitignored** and **rebuildable** — its loss has no consequences.

Example `config.yaml`:

```yaml
schema_version: 1
enabled_types:
  - project
  - character
  - location
  - faction
  - object
  - culture
  - system
  - species
  - chapter
  - note
  - concept
```

---

## 6. Frontmatter & File Conventions

### Relative Image Paths

All media are referenced by **relative paths** to the project folder (e.g., `cover: assets/images/aria.jpg`, `map: assets/maps/glass-city.png`). Never absolute paths or local URLs: the project must remain portable. In the body, images via `![[file]]` or `![](assets/...)`.

### Non-destructive Writing (faithful round-trip)

When the app rewrites an entry, it **preserves**:

- **unknown YAML keys** (fields added by the user or another tool) — kept as-is;
- **existing key order**;
- the **markdown body byte-for-byte** (spaces, line breaks, formatting).

The app does not reorder, reformat, or delete what it doesn't understand. This is what makes external editing (Obsidian, vim, mobile) safe.

### Deterministic YAML Serialization

For clean, stable Git diffs:

- **lists** are serialized **one element per line**;
- writing is **deterministic** (same inputs → same bytes);
- favor a readable, minimal form (no superfluous quotes, except for wikilink strings that need them).

```yaml
factions:
  - "[[Order of the Prism]]"
  - "[[Glassblowers Guild]]"
```

---

## 7. Type Modularity

Types are **activatable/deactivatable** via `enabled_types` in `config.yaml`.

- Disabling a type **removes it from the creation UI** (no longer offered to create such an entry).
- Entries of a disabled type **remain on disk** and **remain indexed**; they are not deleted or ignored. They are simply **hidden** from creation and, depending on the view, filterable.
- Re-enabling the type makes them fully available again.

Modularity is a display/creation preference, **never** a destructive operation on data.

---

## 8. Extensibility & Schema Migration

| Situation | Behavior |
|---|---|
| **Adding a field** | **Additive, no migration.** A new optional field appears; existing entries without it remain valid. |
| **Unknown field** | **Preserved** on write (see §6). The app never deletes a field it doesn't know. |
| **`schema_version`** | Reserved for **breaking changes** (incompatible change in semantics or structure). As long as we stay additive, we don't increment it. |

Any evolution requiring a true migration (thus a `schema_version` bump) must be documented in an [ADR](adr/README.md) before implementation.

---

## 9. Complete Entry Examples

### Example A — rich `character`

File: `characters/aria-solane.md`

```markdown
---
type: character
title: Aria Solane
aliases:
  - The Glassmaker
  - Aria
tags:
  - protagonist
  - pov
cover: assets/images/aria-solane.jpg
created: 2026-07-12T09:15:00Z
updated: 2026-08-01T18:40:00Z
role: protagonist
desire: find her disappeared mother's workshop
wound: abandoned as a child after the glassworks fire
fear: losing those she loves again
motivation: prove she deserves the family legacy
need: learn to trust
lie: "« I don't need anyone »"
flaw: pride
arc: from isolation to solidarity
species: "[[Human]]"
culture: "[[Glass City Glassblowers]]"
factions:
  - "[[Order of the Prism]]"
home: "[[Glass City]]"
status: alive
age: 24
gender: female
pronouns: she
appearance: ash-colored hair, burn scar on right forearm
---

Aria grew up in the alleys of [[Glass City]], taken in by the
[[Order of the Prism]] after the fire that took her mother. A prodigy
glassmaker but solitary, she refuses help from other members.

## Voice
Dry, ironic, never finishes her sentences when emotional.

## Relationships
- Wary rivalry with [[Kael Vantre]].
- Owes everything, without admitting it, to [[Master Orlan]].
```

### Example B — `chapter`

File: `chapters/03-the-crack.md`

```markdown
---
type: chapter
title: The Crack
tags:
  - act-1
created: 2026-07-20T11:00:00Z
updated: 2026-08-03T16:20:00Z
order: 3
pov: "[[Aria Solane]]"
locations:
  - "[[Glass City]]"
  - "[[Solane Workshop]]"
chapter_status: draft
summary: Aria discovers an impossible crack in a work sealed by her mother.
characters:
  - "[[Aria Solane]]"
  - "[[Kael Vantre]]"
plotlines:
  - solane-legacy
  - prism-threat
act: 1
wordcount: 2400
---

## Synopsis
Aria inspects the pieces left by her mother and spots a crack that
should never have existed. [[Kael Vantre]] catches her there.

## Beats
1. Return to [[Solane Workshop]] at dusk.
2. Discovering the crack; memory flash.
3. Confrontation with [[Kael Vantre]] — partial revelation.

## To verify
- Consistency with rules set in [[Living Glass System]].
```

---

## See Also

- [Linking, backlinks and stubs](linking.md) — resolution, ambiguity, creation from stub.
- [Principles](principles.md) — source of truth, non-destruction.
- [Architecture](architecture.md) — index, watcher, Rust workspace.
- [Glossary](glossary.md) — canonical term definitions.
- [ADR](adr/README.md) — pending decisions (including stable `id` path).
