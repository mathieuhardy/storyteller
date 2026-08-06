# API — Backend ↔ Frontend Contract

This document describes the **contract** between the backend ([`storyteller-core`](architecture.md) exposed via [`storyteller-server`](architecture.md)) and the SvelteKit frontend. It is **implementation-independent**: it locks in request/response shapes, not internal details. See the [data model](data-model.md) for entry fields and [linking](linking.md) for the semantics of [wikilinks](glossary.md), [backlinks](glossary.md), and [stubs](glossary.md).

> **Cardinal reminder** — Markdown is the **sole [source of truth](glossary.md)**. The API only exposes an indexed read and writes **non-destructively** to the files. The [index](glossary.md) is a disposable cache; the API never exposes it as an authoritative database.

---

## 1. Style

- **JSON over HTTP**, **resource-oriented**. Standard HTTP verbs (`GET`/`POST`/`PATCH`/`DELETE`), plural paths (`/entities`, `/types`…), body and responses in `application/json` (except raw [asset](glossary.md) serving).
- **No authentication state**: single-user, local / self-host tool (no auth, no tokens, no permissions). See [principles](principles.md).
- Entry paths are expressed relative to the active [project](glossary.md) root (e.g., `characters/aria.md`).
- **Open question — final transport.** The same logical contract must work over HTTP (`storyteller-server` binary) or **Tauri IPC/commands** (`storyteller-tauri` webview). The endpoints below describe the reference HTTP form; their exact mapping to Tauri commands (and the equivalent SSE event mechanism) is decided in [architecture.md](architecture.md) / a dedicated ADR. The frontend consumes an **abstract client layer** to avoid hard-coding transport.

All routes are prefixed by API version: `/api/v1/…` (see [§6](#6-errors--versioning)).

### Implementation Status

This document is the **target contract**; it is delivered milestone by milestone (see [roadmap](roadmap.md)). What is not built answers `501 not_implemented` when it is a *parameter* of a live endpoint, and `404` when the *route* does not exist yet.

| Delivered in | Surface |
|---|---|
| **M1** ✅ | `GET /version`, `GET /project`, `GET /types`, `GET /types/{type}`, `GET /entities`, `GET /entities/{slug}` (`?include=backlinks`), `GET /entities/{slug}/backlinks`. Filters, sort and pagination of [§4](#4-filtering-sorting-pagination) except `q`. |
| **M2** (CRUD) ✅ | `POST`/`PATCH`/`DELETE /entities` and `POST /entities/{slug}/rename` (non-destructive writing; rename per [ADR 0012](adr/0012-rename-link-rewriting.md)). After a write the index is rebuilt so the change is immediately visible. |
| **M2** (watcher) | The SSE stream of [§5](#5-event-stream-sse) and file→index sync on external edits — pending; it needs the [watcher](glossary.md). |
| **M3** ✅ | `GET /entities/{slug}/links`, `GET /stubs`, creation from a stub (via `POST /entities` with the `title` pre-filled from the link text — no source rewrite). |
| **M4** | `/assets` endpoints, `PATCH /types/{type}`, `/projects` registry. |
| **M5** | `GET /search`, and `q` on `/entities`. |

`?render=html` stays unimplemented until markdown rendering is settled ([architecture](architecture.md) §6).

---

## 2. Entity JSON Representation

An [entry](glossary.md) is serialized as follows (the `frontmatter` keys are **in English, snake_case**; values and prose are in French):

```json
{
  "slug": "aria",
  "path": "characters/aria.md",
  "type": "character",
  "frontmatter": {
    "type": "character",
    "title": "Aria",
    "aliases": ["The Glassmaker"],
    "tags": ["protagonist"],
    "role": "protagonist",
    "desire": "find her sister",
    "species": "[[Human]]",
    "factions": ["[[Glass Guild]]"],
    "created": "2026-07-01T10:00:00Z",
    "updated": "2026-08-04T09:12:00Z"
  },
  "body": "# Aria\n\nAria grew up in [[Glass City]]…",
  "html": "<h1>Aria</h1><p>Aria grew up in …</p>",
  "backlinks": [
    { "slug": "glass-city", "path": "locations/glass-city.md", "type": "location", "title": "Glass City", "field": null, "context": "…Aria returns there in chapter 3…" }
  ],
  "errors": []
}
```

| Field | Type | Presence | Description |
|---|---|---|---|
| `slug` | string | always | Entry identity = **filename** ([slug](glossary.md#slug) ASCII kebab-case, no extension). |
| `path` | string | always | `.md` file path relative to project root. |
| `type` | string | always | Entry [type](glossary.md) (mirrors `frontmatter.type`, required). |
| `frontmatter` | object | always | Complete YAML [frontmatter](glossary.md), key→value. Contains [common fields](data-model.md) (`type`, `title`, `aliases`, `tags`, `cover`, `created`, `updated`) + type fields. **Unknown keys and order are preserved** (non-destructive writing). |
| `body` | string | always | **Raw** markdown [body](glossary.md), as-is (source of truth for editing). |
| `html` | string | **optional** | HTML render of body (wikilinks resolved to `<a>`, images to `<img>`). Provided on request (see `?render=html`) for display; never rewritten to file. |
| `backlinks` | array | **optional** | Incoming [backlinks](glossary.md), included on request (`?include=backlinks`). Detailed form in [§ backlinks](#entity-backlinks). |
| `errors` | array | always | Per-entry diagnostics list (empty if all is well); see [error tolerance](#6-errors--versioning). |

**Links in frontmatter**: stored as **wikilink strings** (`pov: "[[Aria]]"`, `locations: ["[[Glass City]]"]`). The backend **does not resolve them** in `frontmatter` (raw value preserved); resolution (existing target / stub / ambiguous) is requested via the [`links`](#links--outgoing) / [`backlinks`](#entity-backlinks) endpoints. The **field name** carries the semantics (`pov`, `parent`, `owner`, `leader`…). See [linking.md](linking.md).

---

## 3. Resources & Endpoints

### Projects

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/v1/projects` | List known projects (recently opened/registered folders). |
| `POST` | `/api/v1/projects/open` | Open a project by folder path; becomes the active project, triggers scan + index (re)build. |
| `GET` | `/api/v1/project` | `project.md` entry + metadata: `enabled_types`, `schema_version` (from `.storyteller/config.yaml`), stats (entry count per type). |

### Entities (entries)

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/v1/entities` | **List** with filters/sort/pagination (see [§4](#4-filtering-sorting-pagination)). Returns a light list (`slug`, `path`, `type`, `title`, `tags`, excerpt). |
| `GET` | `/api/v1/entities/{slug}` | **Get** a full entry. Options: `?include=backlinks`, `?render=html`. |
| `POST` | `/api/v1/entities` | **Create** an entry. Body: `{ type, title, frontmatter?, body? }`. Backend generates the slug (ASCII kebab-case), writes the file in the type folder, initializes `created`/`updated`. Can **materialize a stub**. |
| `PATCH` | `/api/v1/entities/{slug}` | **Update** (**non-destructive** write): `{ frontmatter?, body? }`. Merges provided keys, **preserves** unknown keys / order / untouched body; updates `updated`. |
| `POST` | `/api/v1/entities/{slug}/rename` | **Rename**: `{ new_title?, new_slug? }`. Updates incoming [wikilinks](glossary.md) and/or keeps the old title as [alias](glossary.md) (policy detailed in [linking.md](linking.md)). |
| `DELETE` | `/api/v1/entities/{slug}` | **Delete** the file. Links that targeted it become [stubs](glossary.md). |

### Types

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/v1/types` | List **enabled** types (`enabled_types`) and their field schemas (name, FR label, kind, required/optional, link semantics if any). Base for dynamically generating forms. |
| `GET` | `/api/v1/types/{type}` | Detailed schema of a type (mirrors the [catalog](data-model.md)). |
| `PATCH` | `/api/v1/types/{type}` | Enable/disable a type: `{ enabled: bool }` (updates `.storyteller/config.yaml`). |

### Links, Backlinks & Stubs

<a id="entity-backlinks"></a>

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/v1/entities/{slug}/backlinks` | Incoming [backlinks](glossary.md). Each element: `{ slug, path, type, title, field, context }` — `field` = frontmatter field name that sourced the link (`null` if link is from body), `context` = excerpt around the link. See [linking.md](linking.md). |
| `GET` | `/api/v1/entities/{slug}/links` | **Outgoing** links from the entry (body + frontmatter), each **resolved**: `{ target_raw, target_slug?, field, resolution }` where `resolution ∈ resolved \| stub \| ambiguous`. |
| `GET` | `/api/v1/stubs` | List all project [stubs](glossary.md): wikilink targets without a corresponding entry, with count and list of referencing entries (for suggesting creation). |

<a id="links--outgoing"></a>

### Search

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/v1/search?q=…` | **Full-text** search (FTS) on titles, aliases, tags, and body. Parameters: `q` (query), plus [§4](#4-filtering-sorting-pagination) filters/pagination (`type`, `tag`, `page`…). Returns light entries + highlighted excerpts (`snippet`). |

### Assets

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/v1/assets` | List [assets](glossary.md) (`assets/images/`, `assets/maps/`): `{ path, kind, size, width?, height? }`. |
| `GET` | `/api/v1/assets/{path}` | **Serve** raw file (image/static map), appropriate `Content-Type`. A map is a referenced static image (no interactive pins — [out of scope](principles.md)). |
| `POST` | `/api/v1/assets` | **Upload** an asset (`multipart/form-data`) to assets folder; returns the `path` to reference via `![[file]]` or `![](assets/…)`. |

---

## 4. Filtering, Sorting, Pagination

Common query parameters for `GET /entities` and `GET /search`, to feed [views](glossary.md) (filterable lists/tables):

| Parameter | Example | Effect |
|---|---|---|
| `type` | `?type=character` | Filter by type (repeatable: `?type=character&type=faction`). |
| `tag` | `?tag=protagonist` | Filter by tag (repeatable; logical AND). |
| `<field>` | `?status=writing` · `?pov=[[Aria]]` | Filter on a frontmatter value (parameter name = field's snake_case key). |
| `q` | `?q=glass` | Full-text filter (on `/entities`, restricts list; required on `/search`). |
| `sort` | `?sort=title` · `?sort=-updated` | Sort by field (`title`, `created`, `updated`, `order`…); prefix `-` = descending. |
| `page` / `per_page` | `?page=2&per_page=50` | Pagination (default: `page=1`, `per_page` capped server-side). |

List response wrapped with pagination metadata:

```json
{
  "items": [ { "slug": "aria", "type": "character", "title": "Aria", "tags": ["protagonist"] } ],
  "page": 1,
  "per_page": 50,
  "total": 128
}
```

> These filter/sort combinations are the basis for **saved views** (milestone [M5](roadmap.md)): a view = a persisted set of parameters on the frontend side.

---

## 5. Event Stream (SSE)

The [watcher](glossary.md) monitors project files; the API pushes changes so the GUI refreshes **without polling** (external editing: Obsidian, vim, mobile).

- **Endpoint**: `GET /api/v1/events` — **Server-Sent Events** stream (`text/event-stream`).
- Each event: `event:` = type, `data:` = JSON payload.

| `event` | `data` | Meaning |
|---|---|---|
| `entity.created` | `{ slug, path, type }` | New entry detected on disk. |
| `entity.updated` | `{ slug, path, type }` | Entry modified (by app or externally). |
| `entity.deleted` | `{ slug, path }` | Entry deleted / moved out of project. |
| `assets.changed` | `{ path, change }` | Asset added/modified/deleted. |
| `index.rebuilt` | `{ reason, duration_ms, count }` | Index rebuilt (at startup, after full scan, or after burst of changes). Signal to refresh views. |

The frontend treats these events as cache invalidations (reload affected entry/view). The equivalent in Tauri mode (native event channel) is decided in [architecture.md](architecture.md).

---

## 6. Errors & Versioning

### HTTP Codes

| Code | Case |
|---|---|
| `200` | Success (read/update). |
| `201` | Creation succeeded (`POST /entities`, asset upload). |
| `204` | Deletion succeeded. |
| `400` | Invalid request (malformed parameter/body, missing required field). |
| `404` | Resource not found (nonexistent slug/type/asset). |
| `409` | Conflict (slug already taken on create, rename collision). |
| `422` | Content refused (e.g., provided frontmatter not serializable). |
| `500` | Internal error (disk I/O, etc.). |
| `501` | Endpoint or parameter **documented here but not implemented yet** (code `not_implemented`, message naming the milestone). Preferred over silently ignoring a parameter, which the client cannot detect. |

Normalized error body:

```json
{ "error": { "code": "not_found", "message": "Entry not found: characters/unknown.md", "details": {} } }
```

### Error Tolerance (key principle)

An entry with **invalid YAML does NOT fail a list**. The backend is **resilient**:

- In `GET /entities`, an unreadable entry is **included** with partial/empty `frontmatter`, raw `body` preserved, and a diagnostic in its `errors` field.
- On `GET /entities/{slug}`, the entry is returned with `200` with non-empty `errors` rather than `500`.

```json
{
  "slug": "faction-x",
  "path": "factions/faction-x.md",
  "type": "faction",
  "frontmatter": { "type": "faction", "title": "Faction X" },
  "body": "leader: [[??\n\n# Faction X …",
  "errors": [
    { "code": "yaml_parse_error", "message": "invalid YAML frontmatter line 4", "field": null, "severity": "error" }
  ]
}
```

Per-entry diagnostic codes (non-exhaustive): `yaml_parse_error`, `unknown_type`, `missing_required_field`, `invalid_field_value`, `duplicate_slug`, `encoding_error`. `severity ∈ error \| warning`. This ensures external editing never breaks the app and users see **where** to fix.

| Code | Meaning |
|---|---|
| `yaml_parse_error` | Frontmatter YAML is invalid. Readable `key: value` pairs are salvaged; the body is untouched. |
| `unknown_type` | `type` names a type outside the [catalog](data-model.md). The value is **kept as-is**, never rewritten. |
| `missing_required_field` | `type` absent (entry treated as `note`) or `title` absent (the [slug](glossary.md#slug) is displayed instead). |
| `invalid_field_value` | A value contradicts its declared kind (unknown `enum` value, text where a number is expected…). |
| `duplicate_slug` | Two files share a filename, hence an identity (see [data model](data-model.md) §3). Both entries stay listed; links to that name resolve as `ambiguous`. |
| `encoding_error` | The file is not valid UTF-8. Its content is **not** decoded lossily, so a later write can never persist mangled text. |

> **Absent MVP fields are not errors.** The **MVP** tier in the [data model](data-model.md) marks a field as *in scope*, not as required. A character with only a `title` is valid; validation reports **wrong** values, not blank ones.

### API Versioning

- Version in path: `/api/v1/…`. **Backward-compatible** changes (new optional fields, new endpoints) stay in `v1`; a breaking change increments to `v2`.
- Distinct from `schema_version` of the project (in `.storyteller/config.yaml`), which versions **type schemas**, not the HTTP contract.
- `GET /api/v1/version` returns `{ api_version, core_version, schema_version }` so the frontend can check compatibility.

---

## See Also

- [Data model](data-model.md) — fields per type, common fields, frontmatter.
- [Linking](linking.md) — resolution, aliases, ambiguity, stubs, rename.
- [Architecture](architecture.md) — transport (HTTP vs. Tauri IPC), watcher, index.
- [Glossary](glossary.md) · [Roadmap](roadmap.md).
