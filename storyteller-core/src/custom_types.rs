//! User-defined types, loaded from `.storyteller/types.yaml`
//! (`docs/data-model.md` §7, [ADR 0017](../../docs/adr/0017-custom-types.md)).
//!
//! A custom type extends the built-in catalog ([`crate::types::catalog`]) with
//! the project's own — a setting heavy on artifacts might want `artifact` or
//! `prophecy`, say. Every [`TypeSchema`]/[`FieldSchema`] elsewhere in the crate
//! is `&'static`, built once at compile time; a custom type is data read at
//! runtime, so its strings and slices are **leaked** (`Box::leak`) to get the
//! same `'static` shape rather than threading a second, owned type through
//! every function that already takes `&'static TypeSchema`. This trades a
//! small, bounded amount of memory — a handful of short strings per type,
//! reclaimed only when the process exits — for reusing the existing catalog
//! machinery unchanged; see the ADR for the alternatives considered. Loading a
//! project once, or a few times in a session via `POST /projects/open`, leaks
//! at most a few kilobytes — nowhere near the size of the project itself,
//! which the app already holds fully parsed in memory.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::Deserialize;

use crate::config::STORYTELLER_DIR;
use crate::error::{codes, Diagnostic};
use crate::types::{self, FieldKind, FieldSchema, Tier, TypeSchema};

/// File declaring custom types, alongside `config.yaml`.
pub const TYPES_FILE: &str = "types.yaml";

#[derive(Debug, Deserialize, Default)]
struct RawTypes {
    #[serde(default)]
    types: Vec<RawType>,
    #[serde(default)]
    field_extensions: HashMap<String, Vec<RawField>>,
}

#[derive(Debug, Deserialize)]
struct RawType {
    name: String,
    label: String,
    folder: String,
    #[serde(default)]
    fields: Vec<RawField>,
}

#[derive(Debug, Deserialize)]
struct RawField {
    name: String,
    label: String,
    kind: FieldKind,
    #[serde(default)]
    enum_values: Vec<String>,
    #[serde(default)]
    link_targets: Vec<String>,
}

/// Loads and validates `.storyteller/types.yaml`.
///
/// Returns the **merged catalog**: built-in types (with field extensions applied
/// if any) plus custom types. A missing file is not an error — most projects
/// have no custom types (golden rule 1: a plain markdown folder works out of the
/// box). A type or extension that fails validation is dropped with a diagnostic
/// rather than aborting the whole file (golden rule 5, tolerance for imperfect
/// data); the rest still loads.
pub fn load(project_root: &Path) -> (&'static [TypeSchema], Vec<Diagnostic>) {
    let path = project_root.join(STORYTELLER_DIR).join(TYPES_FILE);
    let raw = match std::fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            // No types.yaml: return unmodified built-in catalog
            return (types::merged_catalog(&HashMap::new(), &[]), Vec::new());
        }
        Err(err) => {
            return (
                types::merged_catalog(&HashMap::new(), &[]),
                vec![Diagnostic::warning(
                    codes::YAML_PARSE_ERROR,
                    format!("cannot read {}: {err}", path.display()),
                )],
            );
        }
    };

    let parsed: RawTypes = match serde_norway::from_str(&raw) {
        Ok(parsed) => parsed,
        Err(err) => {
            return (
                types::merged_catalog(&HashMap::new(), &[]),
                vec![Diagnostic::warning(
                    codes::YAML_PARSE_ERROR,
                    format!("invalid {}: {err} — no custom type loaded", path.display()),
                )],
            );
        }
    };

    let mut names: HashSet<String> = types::catalog().iter().map(|t| t.name.to_string()).collect();
    let mut folders: HashSet<String> = types::catalog()
        .iter()
        .map(|t| t.folder.to_string())
        .filter(|f| !f.is_empty())
        .collect();

    let mut accepted = Vec::new();
    let mut diagnostics = Vec::new();

    for raw_type in parsed.types {
        let label = raw_type.name.clone();
        match build(raw_type, &names, &folders) {
            Ok(schema) => {
                names.insert(schema.name.to_string());
                folders.insert(schema.folder.to_string());
                accepted.push(schema);
            }
            Err(message) => diagnostics.push(
                Diagnostic::warning(
                    codes::INVALID_TYPE_DEFINITION,
                    format!("custom type `{label}`: {message}"),
                )
                .with_field("types"),
            ),
        }
    }

    // Build and validate field extensions
    let custom = leak_slice(accepted);
    let (extensions, ext_diagnostics) = build_extensions(parsed.field_extensions, &names, custom);
    diagnostics.extend(ext_diagnostics);

    (types::merged_catalog(&extensions, custom), diagnostics)
}

fn is_snake_case(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Validates and builds field extensions for built-in and custom types.
///
/// Returns a map from type name to the leaked slice of extension fields, plus
/// any diagnostics for invalid extensions (which are dropped).
fn build_extensions(
    extensions: HashMap<String, Vec<RawField>>,
    known_types: &HashSet<String>,
    custom_types: &'static [TypeSchema],
) -> (HashMap<String, &'static [FieldSchema]>, Vec<Diagnostic>) {
    let mut result = HashMap::new();
    let mut diagnostics = Vec::new();
    let common: HashSet<&str> = types::common_fields().iter().map(|f| f.name).collect();

    for (type_name, raw_fields) in extensions {
        // Check that the target type exists
        if !known_types.contains(&type_name) {
            diagnostics.push(
                Diagnostic::warning(
                    codes::INVALID_FIELD_EXTENSION,
                    format!("field_extensions: unknown type `{type_name}`"),
                )
                .with_field("field_extensions"),
            );
            continue;
        }

        // Get existing fields from the target type to check for shadowing.
        // Check built-ins first, then custom types.
        let existing_fields: HashSet<&str> = types::type_schema(&type_name, types::catalog())
            .or_else(|| custom_types.iter().find(|t| t.name == type_name))
            .map(|s| s.fields.iter().map(|f| f.name).collect())
            .unwrap_or_default();

        let mut field_names: HashSet<String> = HashSet::new();
        let mut fields = Vec::new();
        let mut type_valid = true;

        for raw_field in raw_fields {
            if let Err(message) = validate_extension_field(
                &raw_field,
                &type_name,
                &common,
                &existing_fields,
                &field_names,
            ) {
                diagnostics.push(
                    Diagnostic::warning(codes::INVALID_FIELD_EXTENSION, message)
                        .with_field("field_extensions"),
                );
                type_valid = false;
                break;
            }

            field_names.insert(raw_field.name.clone());
            fields.push(FieldSchema {
                name: leak_str(raw_field.name),
                label: leak_str(raw_field.label),
                kind: raw_field.kind,
                tier: Tier::Optional,
                required: false,
                enum_values: leak_slice(raw_field.enum_values.into_iter().map(leak_str).collect()),
                link_targets: leak_slice(
                    raw_field.link_targets.into_iter().map(leak_str).collect(),
                ),
            });
        }

        if type_valid && !fields.is_empty() {
            result.insert(type_name, leak_slice(fields));
        }
    }

    (result, diagnostics)
}

/// Validates a single extension field.
fn validate_extension_field(
    raw_field: &RawField,
    type_name: &str,
    common: &HashSet<&str>,
    existing_fields: &HashSet<&str>,
    declared_fields: &HashSet<String>,
) -> Result<(), String> {
    if !is_snake_case(&raw_field.name) {
        return Err(format!(
            "field_extensions[{type_name}]: field `{}` must be snake_case",
            raw_field.name
        ));
    }
    if common.contains(raw_field.name.as_str()) {
        return Err(format!(
            "field_extensions[{type_name}]: field `{}` shadows a common field",
            raw_field.name
        ));
    }
    if existing_fields.contains(raw_field.name.as_str()) {
        return Err(format!(
            "field_extensions[{type_name}]: field `{}` shadows an existing field",
            raw_field.name
        ));
    }
    if declared_fields.contains(&raw_field.name) {
        return Err(format!(
            "field_extensions[{type_name}]: field `{}` declared twice",
            raw_field.name
        ));
    }
    if raw_field.label.trim().is_empty() {
        return Err(format!(
            "field_extensions[{type_name}]: field `{}` has no label",
            raw_field.name
        ));
    }
    let is_enum = raw_field.kind == FieldKind::Enum;
    if is_enum && raw_field.enum_values.is_empty() {
        return Err(format!(
            "field_extensions[{type_name}]: field `{}` is an enum without values",
            raw_field.name
        ));
    }
    if !is_enum && !raw_field.enum_values.is_empty() {
        return Err(format!(
            "field_extensions[{type_name}]: field `{}` is not an enum but declares values",
            raw_field.name
        ));
    }
    Ok(())
}

/// Validates one declared type against the names/folders already taken (built-in
/// plus custom types accepted earlier in the same file) and builds its
/// `'static` [`TypeSchema`].
fn build(
    raw: RawType,
    names: &HashSet<String>,
    folders: &HashSet<String>,
) -> Result<TypeSchema, String> {
    if !is_snake_case(&raw.name) {
        return Err(format!("name `{}` must be snake_case", raw.name));
    }
    if names.contains(&raw.name) {
        return Err(format!("name `{}` is already taken", raw.name));
    }
    if raw.label.trim().is_empty() {
        return Err("missing label".to_string());
    }
    if raw.folder.trim().is_empty() {
        return Err("missing folder".to_string());
    }
    if folders.contains(&raw.folder) {
        return Err(format!(
            "folder `{}` is already used by another type",
            raw.folder
        ));
    }

    let common: HashSet<&str> = types::common_fields().iter().map(|f| f.name).collect();
    let mut field_names = HashSet::new();
    let mut fields = Vec::with_capacity(raw.fields.len());
    for raw_field in raw.fields {
        if !is_snake_case(&raw_field.name) {
            return Err(format!("field `{}` must be snake_case", raw_field.name));
        }
        if common.contains(raw_field.name.as_str()) {
            return Err(format!(
                "field `{}` shadows a common field",
                raw_field.name
            ));
        }
        if !field_names.insert(raw_field.name.clone()) {
            return Err(format!("field `{}` declared twice", raw_field.name));
        }
        if raw_field.label.trim().is_empty() {
            return Err(format!("field `{}` has no label", raw_field.name));
        }
        let is_enum = raw_field.kind == FieldKind::Enum;
        if is_enum && raw_field.enum_values.is_empty() {
            return Err(format!(
                "field `{}` is an enum without values",
                raw_field.name
            ));
        }
        if !is_enum && !raw_field.enum_values.is_empty() {
            return Err(format!(
                "field `{}` is not an enum but declares values",
                raw_field.name
            ));
        }

        fields.push(FieldSchema {
            name: leak_str(raw_field.name),
            label: leak_str(raw_field.label),
            kind: raw_field.kind,
            tier: Tier::Optional,
            required: false,
            enum_values: leak_slice(raw_field.enum_values.into_iter().map(leak_str).collect()),
            link_targets: leak_slice(raw_field.link_targets.into_iter().map(leak_str).collect()),
        });
    }

    Ok(TypeSchema {
        name: leak_str(raw.name),
        label: leak_str(raw.label),
        folder: leak_str(raw.folder),
        fields: leak_slice(fields),
    })
}

fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn leak_slice<T>(v: Vec<T>) -> &'static [T] {
    Box::leak(v.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUILTIN_COUNT: usize = 11;

    fn write_types(dir: &Path, contents: &str) {
        let storyteller = dir.join(STORYTELLER_DIR);
        std::fs::create_dir_all(&storyteller).unwrap();
        std::fs::write(storyteller.join(TYPES_FILE), contents).unwrap();
    }

    #[test]
    fn missing_file_is_not_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let (catalog, diagnostics) = load(dir.path());
        // Returns built-in types only
        assert_eq!(catalog.len(), BUILTIN_COUNT);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn loads_a_well_formed_custom_type() {
        let dir = tempfile::tempdir().unwrap();
        write_types(
            dir.path(),
            r#"
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
"#,
        );
        let (catalog, diagnostics) = load(dir.path());
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        // 11 built-ins + 1 custom
        assert_eq!(catalog.len(), BUILTIN_COUNT + 1);
        // Custom types are appended at the end
        let schema = &catalog[BUILTIN_COUNT];
        assert_eq!(schema.name, "artifact");
        assert_eq!(schema.label, "Artéfact");
        assert_eq!(schema.folder, "artifacts");
        assert_eq!(schema.fields.len(), 3);
        assert_eq!(schema.field("rarity").unwrap().enum_values, ["common", "rare", "legendary"]);

        assert!(types::type_schema("artifact", catalog).is_some());
        assert_eq!(types::folder_for("artifact", catalog), Some("artifacts"));
    }

    #[test]
    fn rejects_a_name_colliding_with_a_built_in_type() {
        let dir = tempfile::tempdir().unwrap();
        write_types(
            dir.path(),
            "types:\n  - name: character\n    label: Doublon\n    folder: doublons\n",
        );
        let (catalog, diagnostics) = load(dir.path());
        // Only built-ins, no custom type added
        assert_eq!(catalog.len(), BUILTIN_COUNT);
        assert_eq!(diagnostics[0].code, codes::INVALID_TYPE_DEFINITION);
    }

    #[test]
    fn rejects_a_folder_colliding_with_a_built_in_type() {
        let dir = tempfile::tempdir().unwrap();
        write_types(
            dir.path(),
            "types:\n  - name: gizmo\n    label: Gadget\n    folder: characters\n",
        );
        let (catalog, diagnostics) = load(dir.path());
        assert_eq!(catalog.len(), BUILTIN_COUNT);
        assert_eq!(diagnostics[0].code, codes::INVALID_TYPE_DEFINITION);
    }

    #[test]
    fn one_bad_type_does_not_block_the_others() {
        let dir = tempfile::tempdir().unwrap();
        write_types(
            dir.path(),
            r#"
types:
  - name: "Not Snake Case"
    label: Cassé
    folder: broken
  - name: gizmo
    label: Gadget
    folder: gizmos
"#,
        );
        let (catalog, diagnostics) = load(dir.path());
        // 11 built-ins + 1 valid custom
        assert_eq!(catalog.len(), BUILTIN_COUNT + 1);
        assert_eq!(catalog[BUILTIN_COUNT].name, "gizmo");
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn rejects_an_enum_field_without_values() {
        let dir = tempfile::tempdir().unwrap();
        write_types(
            dir.path(),
            "types:\n  - name: gizmo\n    label: Gadget\n    folder: gizmos\n    fields:\n      - name: state\n        label: État\n        kind: enum\n",
        );
        let (catalog, diagnostics) = load(dir.path());
        assert_eq!(catalog.len(), BUILTIN_COUNT);
        assert_eq!(diagnostics[0].code, codes::INVALID_TYPE_DEFINITION);
    }

    #[test]
    fn rejects_a_field_shadowing_a_common_field() {
        let dir = tempfile::tempdir().unwrap();
        write_types(
            dir.path(),
            "types:\n  - name: gizmo\n    label: Gadget\n    folder: gizmos\n    fields:\n      - name: tags\n        label: Étiquettes\n        kind: list\n",
        );
        let (catalog, diagnostics) = load(dir.path());
        assert_eq!(catalog.len(), BUILTIN_COUNT);
        assert_eq!(diagnostics[0].code, codes::INVALID_TYPE_DEFINITION);
    }

    #[test]
    fn malformed_yaml_is_reported_and_yields_no_custom_type() {
        let dir = tempfile::tempdir().unwrap();
        write_types(dir.path(), "types: [not, a, mapping");
        let (catalog, diagnostics) = load(dir.path());
        // Still returns built-ins even on parse error
        assert_eq!(catalog.len(), BUILTIN_COUNT);
        assert_eq!(diagnostics[0].code, codes::YAML_PARSE_ERROR);
    }

    // --- Field extensions tests ---

    #[test]
    fn field_extension_adds_fields_to_builtin_type() {
        let dir = tempfile::tempdir().unwrap();
        write_types(
            dir.path(),
            r#"
field_extensions:
  character:
    - name: profession
      label: Profession
      kind: text
    - name: birthplace
      label: Lieu de naissance
      kind: link
      link_targets: [location]
"#,
        );
        let (catalog, diagnostics) = load(dir.path());
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(catalog.len(), BUILTIN_COUNT);

        let character = types::type_schema("character", catalog).unwrap();
        assert!(character.field("profession").is_some());
        assert!(character.field("birthplace").is_some());
        assert_eq!(
            character.field("birthplace").unwrap().link_targets,
            &["location"]
        );
        // Original fields are still there
        assert!(character.field("role").is_some());
    }

    #[test]
    fn field_extension_with_enum() {
        let dir = tempfile::tempdir().unwrap();
        write_types(
            dir.path(),
            r#"
field_extensions:
  faction:
    - name: alignment
      label: Alignement
      kind: enum
      enum_values: [good, neutral, evil]
"#,
        );
        let (catalog, diagnostics) = load(dir.path());
        assert!(diagnostics.is_empty(), "{diagnostics:?}");

        let faction = types::type_schema("faction", catalog).unwrap();
        let alignment = faction.field("alignment").unwrap();
        assert_eq!(alignment.kind, FieldKind::Enum);
        assert_eq!(alignment.enum_values, &["good", "neutral", "evil"]);
    }

    #[test]
    fn field_extension_rejects_unknown_type() {
        let dir = tempfile::tempdir().unwrap();
        write_types(
            dir.path(),
            r#"
field_extensions:
  unicorn:
    - name: horn_color
      label: Couleur de corne
      kind: text
"#,
        );
        let (catalog, diagnostics) = load(dir.path());
        assert_eq!(catalog.len(), BUILTIN_COUNT);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, codes::INVALID_FIELD_EXTENSION);
        assert!(diagnostics[0].message.contains("unicorn"));
    }

    #[test]
    fn field_extension_rejects_shadowing_existing_field() {
        let dir = tempfile::tempdir().unwrap();
        write_types(
            dir.path(),
            r#"
field_extensions:
  character:
    - name: role
      label: Rôle (doublon)
      kind: text
"#,
        );
        let (catalog, diagnostics) = load(dir.path());
        assert_eq!(catalog.len(), BUILTIN_COUNT);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, codes::INVALID_FIELD_EXTENSION);
        assert!(diagnostics[0].message.contains("shadows"));
    }

    #[test]
    fn field_extension_rejects_shadowing_common_field() {
        let dir = tempfile::tempdir().unwrap();
        write_types(
            dir.path(),
            r#"
field_extensions:
  character:
    - name: tags
      label: Étiquettes (doublon)
      kind: list
"#,
        );
        let (catalog, diagnostics) = load(dir.path());
        assert_eq!(catalog.len(), BUILTIN_COUNT);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, codes::INVALID_FIELD_EXTENSION);
        assert!(diagnostics[0].message.contains("common"));
    }

    #[test]
    fn field_extension_can_target_custom_type() {
        let dir = tempfile::tempdir().unwrap();
        write_types(
            dir.path(),
            r#"
types:
  - name: artifact
    label: Artéfact
    folder: artifacts

field_extensions:
  artifact:
    - name: rarity
      label: Rareté
      kind: enum
      enum_values: [common, rare, legendary]
"#,
        );
        let (catalog, diagnostics) = load(dir.path());
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        // 11 built-ins + 1 custom (extended)
        assert_eq!(catalog.len(), BUILTIN_COUNT + 1);

        let artifact = types::type_schema("artifact", catalog).unwrap();
        assert!(artifact.field("rarity").is_some());
    }

    #[test]
    fn field_extension_and_custom_types_work_together() {
        let dir = tempfile::tempdir().unwrap();
        write_types(
            dir.path(),
            r#"
types:
  - name: artifact
    label: Artéfact
    folder: artifacts
    fields:
      - name: origin
        label: Origine
        kind: text

field_extensions:
  character:
    - name: profession
      label: Profession
      kind: text
"#,
        );
        let (catalog, diagnostics) = load(dir.path());
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(catalog.len(), BUILTIN_COUNT + 1);

        // Custom type exists with its field
        let artifact = types::type_schema("artifact", catalog).unwrap();
        assert!(artifact.field("origin").is_some());

        // Built-in type is extended
        let character = types::type_schema("character", catalog).unwrap();
        assert!(character.field("profession").is_some());
        assert!(character.field("role").is_some()); // Original field still present
    }
}
