# Vision — Storyteller

**Storyteller** is a tool for storing and organizing all the **context** of a novel: its "bible." Characters, locations, factions, systems, chapters, notes, and resources live in the same place, linked together, and **owned locally** by the author.

Storyteller **is not for writing the manuscript**. It's for never losing track of the universe around the manuscript.

## The Problem

The context of a novel always ends up scattered:

- character sheets in a word processor,
- a world map in an image folder,
- worldbuilding notes in a note-taking app,
- consistency reminders scribbled in the manuscript margins,
- and an author's memory that has to piece it all together.

Result: you **forget** that a character has green eyes in chapter 3, you **search** for ten minutes for the name of a faction, you **duplicate** information, and you **dread** migrating everything when the current tool shuts down or changes its terms. The universe grows faster than the ability to retrieve it.

## The Target User

A **solo author** who wants to remain in control of their data:

- **fantasy** or **SF** writer with a dense world (magic, factions, species, geography);
- but also **contemporary fiction**, thriller, or literary fiction author who mainly needs to track characters, locations, and narrative threads.

This profile is **local-first**: they want their files on their machine, readable and editable without the application, without an account, without imposed cloud. Storyteller is self-hosted and never requires a connection to a third-party service.

## The Value Proposition

A **living, interconnected bible**, whose format remains open.

- **Typed and modular.** Each entry has a type (character, location, faction…) that defines its useful fields. Types can be enabled or disabled per project — a contemporary novel doesn't need the *species* type.
- **Interconnected.** [Wikilinks](linking.md) `[[Name]]` weave the universe; [backlinks](glossary.md) automatically show "who mentions whom," and [stubs](glossary.md) signal what remains to be created.
- **Owned locally.** Each project is a simple **folder of markdown files**, self-contained and portable. [Markdown is the sole source of truth](principles.md): the application is just a comfortable view of files that belong to you, editable in Storyteller as well as Obsidian, vim, or on mobile.
- **Non-destructive.** The app preserves what it doesn't understand (unknown keys, order, body text): editing a file by hand always remains safe.

## Non-Goals

Storyteller remains deliberately focused. It does **not** aim to:

- **replace the manuscript editor** — we store the context, we don't write prose here;
- offer a **timeline / narrative chronology** — decided, out of scope;
- provide **interactive maps** with pins — a map is a referenced static image;
- enable **real-time collaboration**, multi-user, authentication, or permission management;
- do **AI generation**, **cloud sync** as SaaS, or impose **guided questionnaires / prose templates**.

Versioning is left to **Git**.

## MVP Success Criteria

At the end of the MVP, an author should be able to, on their machine:

1. **Create a project** as a self-contained, portable markdown folder.
2. **Create, edit, and rename typed entries** (YAML frontmatter + free-form markdown body) without the app ever corrupting hand-edited content.
3. **Link entries** via `[[wikilink]]`, see **backlinks** and **stubs**, and create an entry directly from a pending link.
4. **Navigate** the universe via filterable lists/tables by type, with the backlinks panel.
5. **Find** any information through full-text search, filters, and sorting.
6. **Enable/disable types** per project, and **edit files externally** with confidence (the index rebuilds itself).

---

For the detailed capabilities, see [features](features.md). For the delivery order and milestones, see [roadmap](roadmap.md).
