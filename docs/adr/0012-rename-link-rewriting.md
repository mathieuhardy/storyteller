# ADR 0012 — Renaming Rewrites Breaking Links, Without Keeping an Alias

- **Status**: Accepted
- **Date**: 2026-08-05

## Context

[Linking](../linking.md) §7.2 left the **rename link-rewriting policy** open and forbade assuming a default: when an entry is renamed (new [slug](../glossary.md#slug) and/or new `title`), incoming [wikilinks](../glossary.md) may stop resolving. [Architecture](../architecture.md) §6 lists the same question. Milestone [M2](../roadmap.md) delivers rename, so the decision can no longer be deferred.

Two levers exist, and they are independent:

1. **Rewrite incoming links** in the referencing files to point at the new identity, or leave them.
2. **Keep the old name as an [alias](../glossary.md)** on the renamed entry as a safety net, or not.

The forces: the project's cardinal rule is that **markdown is the [source of truth](../glossary.md)** and app writes are **non-destructive** ([ADR 0002](0002-markdown-source-of-truth.md)); resolution already ranks **filename first**, then `title`, then `aliases` ([linking](../linking.md) §3.2); and the tool's audience is a single author who wants clean, predictable Git diffs, not a growing tail of alias cruft.

## Decision

**On rename, we rewrite the incoming links that would otherwise break, targeting the new filename, and we keep no alias.**

- **Only breaking links are touched.** A link is rewritten only when it resolves to the entry *before* the rename but would *not* after it. A link that still reaches the entry another way — typically by an untouched `title` or `alias` — is left exactly as written. This keeps rename diffs minimal.
- **Rewrites target the new slug.** The filename is unique per project ([data model](data-model.md) §3), so it is the stable anchor. `[[old-slug]]` becomes `[[new-slug]]`.
- **Readable prose keeps its text.** When the original link was human prose (it matched by `title` or `alias`, e.g. `[[Aria Solane]]`), the rewrite preserves it as display text: `[[new-slug|Aria Solane]]` — the disambiguation form of [linking](../linking.md) §3.3. An existing `|display` and any `#anchor` are always preserved; a link that was already an identifier is rewritten bare.
- **No alias is added.** Renaming does not append the old name to `aliases`.
- **Every rewrite is a non-destructive edit** of the referencing file: unknown keys, key order and the untouched body survive byte-for-byte ([data model](data-model.md) §6). Asset embeds (`![[…]]`) are never rewritten — they point at media, not entries.

## Consequences

- **No silent stubs from an app rename.** Every link the rename would have orphaned is repaired in the same operation, so the [stub](../glossary.md) list stays a signal about *authoring gaps*, not about the app's own edits.
- **Clean files over time.** Without an accumulating alias list, an entry's frontmatter reflects what the author wrote, not its rename history.
- **A rename writes several files.** Repairing links means editing every referencing entry, each a separate non-destructive write. For a single-user local tool over a few hundred entries this is cheap, and the diffs are small because only breaking links change.
- **External renames still degrade gracefully.** This ADR governs renames performed *through the app*. A rename done in Obsidian or vim is still just an external edit: the [watcher](../glossary.md) reindexes and any now-unresolved link surfaces as a stub ([linking](../linking.md) §7.3) — the app never rewrites files on its own initiative in reaction to an external change.
- **Losing the old name is deliberate.** Because no alias is kept, a link written *later* by an external tool under the old name will not resolve and will appear as a stub. That is the accepted cost of clean files; the alternative (a permanent alias) is what we rejected.

## Alternatives Considered

- **Alias safety-net only** (rewrite nothing; append the old name to `aliases`): the most conservative and fully non-destructive of other files, and it never orphans a link. Rejected as the default because aliases accumulate with every rename, frontmatter drifts from what the author means, and the "real" name of an entry becomes ambiguous. It remains the natural behavior for *title* edits that the user does not frame as a rename, but that is a UI affordance, not this policy.
- **Rewrite links *and* keep an alias**: rewrite breaking links *and* append the old name as a backstop. Belt-and-braces, but it pays the alias-accumulation cost in full while the rewrite already removed the need for it. Rejected as redundant.
- **Rewrite *all* incoming links, not just breaking ones**: canonicalize every reference to the new slug on each rename. Rejected because it churns files whose links still resolve perfectly well, inflating diffs and reformatting author-written prose for no correctness gain.
- **A stable `id` field independent of the filename**: would make links immune to renaming entirely. A real hardening path, but it is explicitly deferred to its own decision ([data model](data-model.md) §3, [linking](../linking.md) §7.3) and does not block M2.
