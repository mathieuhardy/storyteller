# Linking, Backlinks, and Stubs

Specification of the linking algorithm that the backend (`storyteller-core`) must implement: **wikilink** syntax, target **resolution**, **backlinks**, **stub** detection, and rename handling.

This document is normative for link behavior. It complements the [data model](data-model.md) (types, fields, frontmatter) and uses terms defined in the [glossary](glossary.md). Cardinal principle reminder: **markdown is the sole source of truth**; the link [index](glossary.md) is a disposable cache, entirely rebuildable from the files.

---

## 1. Wikilink Syntax

A **wikilink** connects an **entry** to a **target**, designated by readable text (never by a file path).

| Form | Example | Meaning |
| --- | --- | --- |
| `[[Target]]` | `[[Aria]]` | Link to the target; displayed text is the target itself. |
| `[[Target\|display text]]` | `[[Aria\|the captain]]` | Link to `Target`, rendered with `display text`. |
| `[[Target#section]]` | `[[Aria#Youth]]` | *(optional)* Link to an anchor (heading) in the target's **body**. |

Parsing rules:

- The **target** is the part to the left of the first `|` (and to the left of the first `#` if present). Only it is used for **resolution** (§3).
- The `#section` is a display anchor only: it does **not** modify target resolution. If the entry exists but the section is absent, the link remains resolved (anchor is ignored at render). Support for jumping to the section may be deferred (v2) without changing resolution.
- Leading/trailing whitespace around the target and display text is ignored.
- The target is **case-insensitive and accent-insensitive** for resolution (see normalization, §3), but its original text is **preserved as-is** in the file (non-destructive writing).
- A wikilink **never crosses** **project** boundaries: resolution scope is the current project (§3).

### Embeds (media)

Images and **assets** are not inter-entry links but content references. Two forms coexist:

| Form | Example | Usage |
| --- | --- | --- |
| `![[file]]` | `![[world-map.png]]` | Wikilink-style embed; resolved to a file in the `assets/` folder. |
| `![](assets/...)` | `![](assets/maps/world-map.png)` | Standard markdown image, path relative to project root. |

- These embeds are **detected and indexed** (to track **asset** usage), but they **do not create stubs**: a missing image is an "asset not found," signaled separately, not an entry to create.
- The `!` prefix unambiguously distinguishes an embed from a wikilink: `[[x]]` = link to entry, `![[x]]` = asset embed.

---

## 2. Links in Frontmatter

Typed relationships live in the **frontmatter** YAML and are stored **as wikilink strings**. It is the **field name** that carries the link semantics — not the content.

```yaml
---
type: chapter
title: "The Departure"
pov: "[[Aria]]"
locations: ["[[Glass City]]", "[[Free Port]]"]
---
```

| Field (example) | Type on the type… | Semantics carried by name |
| --- | --- | --- |
| `pov` | chapter | point of view → **Character** |
| `parent` | location, faction | hierarchical parent → same type |
| `owner` | object | owner → **Character** or **Faction** |
| `leader` | faction | leader → **Character** |
| `home` | character | home location → **Location** |
| `factions[]` | character | memberships → list of **Factions** |

Rules:

- Simple value: a wikilink string (`pov: "[[Aria]]"`). Multiple values: a **list** of wikilink strings (`factions: ["[[The Watchers]]"]`). The `[]` suffix in this document denotes a list field; it does not appear in the YAML.
- The `[[Target|displayed]]` form is allowed in frontmatter, but the displayed text has little use there; resolution uses only the **target**.
- **A single resolution mechanism** applies to **body** and **frontmatter**: every `[[…]]` is extracted then resolved by the §3 algorithm. The only difference is the **context** recorded (§4): `body` or `field:<name>`.
- **Non-destructive** writing: the app preserves unknown YAML keys, key order, and body. Modifying a target via the UI rewrites only the affected value.
- A link field value that is not a valid wikilink string (plain text, empty string) is **not** interpreted as a link: it is kept as-is and ignored by link indexing.

---

## 3. Target Resolution

Goal: associate a textual target with **at most one** entry in the project, or classify it as **stub** or **ambiguous**. Resolution is **scoped to the current project**.

### 3.1 Normalization

We compare **normalized match keys**, never raw strings. The `normalize(s)` function:

1. Unicode **NFC**.
2. `trim` + reduce multiple internal spaces to a single space.
3. **Case folding** (Unicode case folding → lowercase).
4. **Accent/diacritic folding** (e.g., `é`→`e`, `ç`→`c`).

Example: `«  Glass  City »` and `glass city` produce the same key.

> Normalization is only for **comparison**. Original texts (filename, `title`, `aliases`) remain displayed as-is.

### 3.2 Resolution Order

For a target `c`, we compute `k = normalize(c)` then search, **in this order**, among project entries:

1. **Filename** — `normalize(slug_without_extension)` == `k`. (An entry's identity is its filename; this is the strongest match.)
2. **`title`** — `normalize(title)` == `k`.
3. **`aliases`** — `k` is in `{ normalize(alias) }`.
4. Otherwise → **STUB**.

As soon as a rank produces **exactly one** entry, resolution stops and returns that entry (status `resolved`). Ranks express **priority**: a filename beats a homonymous `title`, which beats an `alias`.

### 3.3 Ambiguous Case (≥ 2 entries)

If, at the reached rank, **≥ 2 entries** match (two identical `title`s, or a `title` and another's `alias` at the same rank), the target is **ambiguous** — a status **distinct** from stub.

- An ambiguous target is **not** silently resolved to one of the candidates.
- It is **signaled** in the UI (the link carries an "ambiguous" indicator) and listed for **manual resolution**.
- The app **suggests disambiguation** by offering to rewrite the link as `[[target|Displayed]]` where `target` becomes the **filename** of the chosen candidate (filename being unique per project), while keeping the desired display text. Example: ambiguous `[[Aria]]` → `[[aria-val|Aria]]` or `[[aria-dark|Aria]]`.

### 3.4 Link Statuses

| Status | Condition | UI Treatment |
| --- | --- | --- |
| `resolved` | exactly 1 entry | active link |
| `stub` | 0 entries (§6) | "to create" link, detected |
| `ambiguous` | ≥ 2 entries | signaled link, manual resolution (§3.3) |

---

## 4. Backlinks (inverse index)

A **backlink** is the inverse of a link: if entry A cites `[[B]]`, then A appears in B's backlinks.

### 4.1 Conceptual Model

The **index** materializes a relation:

```
links(source, target, context)
```

| Column | Content |
| --- | --- |
| `source` | entry containing the link (filename). |
| `target` | resolved target (filename) or **stub** key if unresolved. |
| `context` | `body` **or** `field:<name>` (e.g., `field:pov`, `field:locations`). |

Additional useful information to store per occurrence: display text, position/offset in the file (for preview), resolution status (§3.4). This schema is **conceptual**: the concrete implementation (SQLite tables) is covered in [architecture](architecture.md). Like all of the **index**, it is **rebuildable** entirely by re-parsing the files.

### 4.2 "Mentioned In" Panel

For an entry X, the backlinks panel lists all `links` rows where `target = X`, grouped by `source`, with:

- the source entry and its **type**;
- the **context** (body, or field name — e.g., "via `pov`");
- a context excerpt (preview around the occurrence).

This panel updates on each **index** recalculation by the **watcher**.

---

## 5. "Where X Appears"

Derived view, built **entirely from X's backlinks** — no new data is stored.

- **Filtering by source type**: backlinks where `source.type = chapter` → "X appears in these chapters."
- **Filtering by field**: backlinks whose `context` is `field:pov`, `field:characters`, or `field:locations` → *declared* presence of a character/location in a chapter, distinct from a mere body mention.
- Combinable: "chapters where X is the **pov**" = `source.type = chapter` **and** `context = field:pov`.

### Derived Fields (displayable, not stored)

Some relationships don't need to be written on both sides: they can be **read** from backlinks.

> Example: the **members** of a **faction** F are not stored in F. They are deduced from characters whose `factions[]` contains `[[F]]` — i.e., F's backlinks at context `field:factions`. The UI can display a **computed** "Members" block, without duplicating information or risking desynchronization.

This principle avoids redundancy and respects the single **source of truth**: the link is written in only one place (character side), the other side is a read.

---

## 6. Stubs

A **stub** is a wikilink target **without a corresponding entry** in the project (§3 resolution reached rank 4).

### 6.1 Detection

During indexing, every link with `stub` status (§3.4) is recorded with its normalized key. Multiple occurrences of the same unresolved target share the **same** stub (grouped by normalized key), while preserving the original texts encountered.

### 6.2 Stub List

The UI exposes a **stub list** for the project: orphan targets, occurrence count, and source entries that mention them. It's a TODO list of entries to create.

### 6.3 "Create Entry" Action

From a stub (in body, frontmatter, or stub list), the **create entry** action:

- creates a new entry whose **`title` is pre-filled** with the **link text** (original target, case/accents preserved);
- lets the user choose the **type** (and thus the folder and frontmatter schema, cf. [data model](data-model.md));
- derives the **filename** (ASCII kebab-case slug) from this title;
- once created, the **watcher** recalculates the **index** and links pointing to this target change from `stub` to `resolved`, without rewriting source files.

---

## 7. Renaming and Broken Links

### 7.1 Identity and Title Renaming

- The **identity** of an entry is its **filename**. Links resolve primarily on it (§3.2).
- Renaming the **`title`** (without renaming the file) is safe: the app **keeps the old title as an `alias`**, so that wikilinks written under the old title continue to resolve (rank 3).

### 7.2 File Renaming (via the app)

Renaming the file changes identity. The policy is fixed by [ADR 0012](adr/0012-rename-link-rewriting.md): the app **rewrites the incoming wikilinks that would otherwise break** to point at the new filename, and **keeps no alias**.

- Only links that would stop resolving are rewritten; a link still reaching the entry by an untouched `title` or `alias` is left as written.
- Rewrites target the new **filename**; human-readable prose keeps its text as `[[new-slug|Displayed]]` (§3.3), and asset embeds are never touched.
- Each rewrite is a **non-destructive** edit of the referencing file (unknown keys, key order and body preserved).

This never silently produces a [stub](glossary.md) from an app-driven rename. The alternative safety-net (append the old name as an alias, rewrite nothing) was considered and rejected as the default — see the ADR.

### 7.3 External Editing and Broken Links

Editing outside the app (Obsidian, vim, mobile) is a first-class case — markdown remains the **source of truth**.

- The **watcher** detects on-disk changes and **recalculates the index** (no data is lost since the index is a cache).
- A link whose target no longer resolves (entry deleted or renamed outside the app) becomes a **signaled stub**: it is not silently repaired, it appears in the stub list (§6.2).
- The app **never rewrites** a file on its own initiative in reaction to external editing; any rewrite remains subject to the §7.2 policy.

> Hardening path (not imposed): a stable `id` field, independent of filename, would make links robust to external renaming. Decision deferred to an ADR (see [ADR folder](adr/README.md)); this document does not depend on it.

---

## See Also

- [Data model](data-model.md) — types, link fields, frontmatter.
- [Architecture](architecture.md) — index and watcher implementation.
- [Glossary](glossary.md) — definitions of wikilink, backlink, stub, alias, index, watcher.
