# ADR 0017 — Custom types are declared in `.storyteller/types.yaml` and merged into the catalog at runtime

- **Status**: Accepted
- **Date**: 2026-08-25

## Context

M7 (`docs/roadmap.md`) lists **custom types** — types defined by the user beyond the 11 built-ins — as a v2 item. Every [`TypeSchema`]/[`FieldSchema`] in `storyteller-core::types` is a `&'static` value, hand-written as a `const CATALOG: &[TypeSchema] = &[...]` array built at compile time (`storyteller-core/src/types.rs`). `type_schema`, `folder_for`, `resolve_type` and `validate` all read from that single static array, and the frontend's forms, list columns, and type picker are already fully **schema-driven** from `GET /types`/`GET /types/{type}` (`docs/ui/screens.md` §3–4) — no type name is hardcoded client-side.

A custom type needs the same shape (`name`, `label`, `folder`, `fields`) but is *data*, known only at runtime, per project. Two questions: where does a user declare one, and how does runtime-loaded data fit into a type system built entirely around `&'static`?

## Decision

**Custom types are declared in `.storyteller/types.yaml`**, a new file alongside `config.yaml` (`docs/data-model.md` §5) — YAML, like `config.yaml`, not markdown: it is project *configuration* (a schema), not story *content*, so it does not compete with "markdown is the sole source of truth" (golden rule 1, which speaks to entries).

```yaml
types:
  - name: artifact
    label: Artéfact
    folder: artifacts
    fields:
      - name: origin
        label: Origine
        kind: text
      - name: rarity
        label: Rareté
        kind: enum
        enum_values: [common, rare, legendary]
      - name: owner
        label: Propriétaire
        kind: link
        link_targets: [character]
```

`kind` accepts the same values `FieldKind` already serializes as (`text`, `number`, `boolean`, `enum`, `list`, `link`, `link-list`, `image`, `image-list`, `number-or-text`, `list-or-text`). Every custom field is `Tier::Optional`, `required: false` — v1 does not expose those knobs; see Consequences.

`storyteller-core::custom_types::load(project_root)` parses the file and validates each declared type: `name`/field names snake_case, `name` and `folder` not colliding with a built-in or an earlier custom type in the same file, field names not shadowing a common field (`type`, `title`, `aliases`, `tags`, `cover`, `created`, `updated`), enum fields carrying values (and non-enum fields not carrying any). A type that fails validation is **dropped with a diagnostic**, not fatal to the file — golden rule 5, tolerance for imperfect data, applied to schema declarations the same way it already applies to entries. `link_targets` is *not* validated against known type names, matching `FieldSchema.link_targets`'s existing contract: "purely informative; a link to another type is not an error" (`types.rs` doc comment) — nothing at runtime enforces it, for built-ins or custom types alike.

**The accepted `TypeSchema`s are leaked to `'static`** (`Box::leak`) rather than making the type carry owned `String`/`Vec` fields. `Project` gains a `custom_types: &'static [TypeSchema]` field, loaded once in `Project::open`/`reload_config`; `types::type_schema`, `folder_for`, `resolve_type`, `validate` and the new `all_types` all take a `custom: &'static [TypeSchema]` parameter and search built-ins-then-custom. Every existing call site threads `project.custom_types()` through; nothing downstream (index, search, the API's `TypeResponse`, the frontend) needed to change shape — a custom type is just one more `TypeSchema` value flowing through code that already treats `TypeSchema` generically.

## Consequences

- **The frontend needed zero changes to create, edit, list, view, filter, or full-text-search entries of a custom type.** `GET /types/{type}` already drives form generation (`docs/ui/screens.md` §4) and the list/table columns (§3) from whatever schema comes back; the index groups entries by whatever `type` string they actually declare (`docs/architecture.md`). This is the payoff of M4's "no API bypass on front side" bet.
- **One real, documented gap: type *labels* in the nav/creation-picker fall back to the raw name for a custom type.** `frontend/src/lib/i18n/index.svelte.ts`'s `typeLabel(name)` looks up a compile-time `type.<name>` catalog key and falls back to `name` itself — it does not (yet) consult the schema's own `label` the way `fieldLabel` already does for fields. A custom `artifact` type shows "artifact" in the nav until a `field.<name>`-style frontend fallback is added. Left as a follow-up, not blocking: it's cosmetic, not a functional gap.
- **No GUI type-builder in this pass.** `types.yaml` is hand-authored — consistent with `config.yaml`'s `enabled_types` already being a hand-editable file with no dedicated settings screen (`docs/roadmap.md` M4 notes this same gap for type enable/disable). A "manage types" screen is a natural, separately-scoped follow-up.
- **`enabled_types` now defaults-in and validates against custom types too.** `ProjectConfig::load` takes the loaded custom types so a freshly declared custom type is immediately offered for creation without a second manual edit to `config.yaml`, and so `enabled_types` mentioning a custom type is never wrongly flagged `unknown_type`.
- **Leaking is a deliberate, bounded trade-off, not an oversight.** A custom type's strings/slices live for the rest of the process — reclaimed only on exit, not on the next `POST /projects/open`. Given custom-type declarations are small (a handful of short strings per type) and project switches are a rare, manual, single-user action (ADR 0003), this is bytes, not megabytes, over a realistic session — far smaller than the fully-parsed project the app already holds in memory per open project. The alternative (below) was rejected precisely to avoid a much larger, riskier change for a cost this small.
- **A malformed `types.yaml` degrades gracefully.** A YAML syntax error drops all custom types for that project (with a diagnostic); one bad type definition inside an otherwise-valid file drops only that one. Either way the project still opens — it never fails to load over a schema-declaration problem.

## Alternatives Considered

- **Make `TypeSchema`/`FieldSchema` own their data (`String`, `Vec<FieldSchema>`) everywhere, custom and built-in alike.** Rejected for this pass: the built-in catalog is ~300 lines of `const fn` builders (`text`, `field`, `enumeration`, `link`, `link_list`) producing a single `const CATALOG: &[TypeSchema] = &[...]` array — `String` cannot be constructed in a `const` context, so this would force every builder and the catalog itself off `const`, rippling through the ~15 call sites across `core` and `server` for a change whose only payoff is avoiding one `Box::leak`. Worth revisiting if custom types grow real per-request mutation (e.g. editing a type definition through the API, not just a file) rather than "loaded once per project open."
- **A separate, parallel `CustomTypeSchema` type, kept apart from the built-in `TypeSchema`.** Rejected: every consumer (`entry_from_bytes`, the `/types` routes, the editor's schema-driven form) would need to branch on or merge two shapes instead of iterating one slice — the entire appeal of "custom types just work" comes from them being the *same* `TypeSchema` the built-ins already are.
- **Declare custom types in `config.yaml` itself, instead of a new `types.yaml`.** Rejected for legibility: `config.yaml` is a short, hand-glanceable file (`schema_version`, `enabled_types`); a type's field list can run to a dozen lines each. Splitting keeps `config.yaml` skimmable and gives custom types their own obvious home, the same way `cache.sqlite` and `config.yaml` are already split by concern inside `.storyteller/`.
