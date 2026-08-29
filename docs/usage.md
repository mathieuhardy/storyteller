# Usage Guide

A walkthrough of the actual application — what you'll do, screen by screen. Screenshots aren't in
this pass (no browser was available to capture real ones while writing it); everything below
describes real, shipped behavior, not planned behavior. See [vision](vision.md) for the why,
[features](features.md) for the full catalog, and [data model](data-model.md) if you want the
exact frontmatter fields per type.

## Opening or creating a project

A **project** is just a folder of markdown files. On first launch you land on the launcher: open
an existing folder by path, or start a new one — either way, Storyteller creates a `.storyteller/`
folder inside it for its index and config, and touches nothing else. Recently opened projects are
remembered (in an OS preference file, never inside the project itself) so you can switch back to
one with a click.

Nothing about a project depends on Storyteller having opened it before: point it at any folder of
markdown files with YAML frontmatter and it works, `.storyteller/` or not.

## The dashboard

Opening a project lands you on its dashboard: the project's own root entry (`project.md`) as a
hero — cover, title, logline, status, genres — plus at-a-glance stats (entry count by type,
how many [stubs](glossary.md) are waiting, entries flagged with a diagnostic), your most recently
modified entries, and an index-health indicator that pulses when the file watcher picks up an
external change.

## Creating and editing entries

Every piece of context — a character, a location, a chapter, anything — is an **entry**: a
markdown file with YAML frontmatter (typed fields) and a free-form body. Create one from the "New
entry" button (pick a type, give it a title) or straight from a pending link (see
[Stubs](#the-links-workshop) below); edit it from its own page.

The editor's form is generated from the entry's type — text fields, numbers, enums, link fields
with autocomplete, image fields with upload, lists — so a `character` and a `chapter` don't look
alike, but the mechanism generating their forms is identical, including for a [custom
type](#custom-types) you've defined yourself. Only what you actually changed gets sent when you
save. Fields the app doesn't recognize (added by hand, or by another tool) are shown read-only in
a "preserved fields" section — they survive every save untouched.

**Nothing here is destructive.** Unknown YAML keys, key order, and the body text are preserved
byte-for-byte on every write. Editing a file in Storyteller, then in Obsidian or vim, then back in
Storyteller, never loses anything either side didn't touch.

## Linking entries

Write `[[Character Name]]` anywhere — in the body, or in a link-typed frontmatter field like
`pov` or `owner` — and Storyteller resolves it: by filename first, then by `title`, then by an
`alias`. A resolved link is clickable and navigates straight to the target. Each entry's page has
a rail panel showing **outgoing links** (with their resolution state) and, automatically,
**backlinks** — every other entry that mentions this one, grouped by source, with the field or
body excerpt that mentions it.

A link to something that doesn't exist yet becomes a **stub** rather than an error — it's tracked,
not broken. `[[Aria]]` and `[[Aria Solane]]` both resolving to the same character isn't an error
either, until *two different* entries could both plausibly be the target: that's flagged as
**ambiguous**, and resolved explicitly rather than guessed.

## The links workshop

The workshop (nav: "To create") is where stubs and ambiguous links get resolved:

- **Stubs tab** — every unresolved link target, grouped, with a count and the entries that mention
  it. "Create" opens a form pre-filled with the target's text as the title; pick a type, and the
  link resolves at the next reindex — no source file gets rewritten, the link just starts matching
  a real entry.
- **Ambiguous tab** — every link with more than one plausible target. Pick the right one, and
  Storyteller rewrites the link in place to point at it unambiguously (either just this occurrence,
  or every occurrence of the same ambiguous text) — a normal non-destructive edit of the source
  file, same as any other write.

## Browsing: list, table, and saved views

Each type gets its own list/table screen (nav: pick a type). Table view's columns are that type's
fields; list view shows cards with an excerpt. Both share a toolbar: sort (by any column), filters
(by tag, or by any field value — the filter picker adapts to the field's kind, offering enum
values directly for an enum field), and pagination. Filters live in the URL, so a filtered/sorted
view is just a link you can bookmark or share — or **save** it by name from the toolbar
(`SavedViewsMenu`) to reload later; saved views are a local browser preference, not written into
the project.

## Full-text search

The search box in the top bar queries title, aliases, tags, and body text across every entry,
accent-insensitive, ranked by relevance, with a highlighted snippet per result — combinable with
the same type/tag/field filters and pagination the list screens use.

## Media: assets and the gallery

Images (character portraits, location photos) and maps (always static images, never interactive)
live under a project's `assets/` folder and get referenced from entries via image fields (`cover`,
`portrait`, `map`, …) or embedded in the body with `![[file]]`. Upload through the entry editor's
image fields, or browse everything at once in the **gallery** (nav: "Media"): a grid of thumbnails
you can filter by filename, with a detail view offering a "copy link" action to paste
`![[file]]` straight into an entry's body.

## The link graph

The graph screen (nav: "Explore") shows the whole project as a network — every entry as a node,
every resolved link as an edge — laid out automatically so clusters and outliers are visible at a
glance. Hover a node to highlight its direct connections; click one to open that entry; scroll to
zoom, drag to pan. It's read-only: a way to explore the shape of your universe, not to edit it.

## Custom types

If the ~11 built-in types (character, location, faction, …) don't cover something your project
needs, declare your own in `.storyteller/types.yaml` — a name, a label, a folder, and a list of
fields with their kinds (text, number, enum, link, …). A custom type behaves exactly like a
built-in one everywhere: it gets a create form, a list/table screen, filtering, search, the works.

You can also **extend existing types** (built-in or custom) with additional fields via
`field_extensions` in the same file — useful when you want to track extra data on characters or
locations without creating an entirely new type.

See [data model §7](data-model.md#custom-types) for the exact file format.

## Editing files externally

Everything above is also true from outside the app. Edit a file in Obsidian, vim, or on your
phone via a sync tool, and Storyteller's watcher picks up the change and reindexes automatically —
no manual refresh, no risk of the app's view going stale. If you ever distrust the index, deleting
`.storyteller/cache.sqlite` and reopening the project rebuilds it from the files, which are always
the authority.

## Running it

Storyteller runs the same way regardless of how you launch it — same API, same frontend:

- **Self-host**, over a network, via Docker or Nix — see the root [README](../README.md#installation).
- **Desktop app** — a native window, via `storyteller-tauri` (AppImage/`.deb` on Linux today), which
  runs the exact same server in-process and just opens a window pointed at it.

---

See also: [vision](vision.md), [features](features.md), [data model](data-model.md),
[architecture](architecture.md).
