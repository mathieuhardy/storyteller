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

use std::collections::HashSet;
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
/// A missing file is not an error — most projects have no custom types
/// (golden rule 1: a plain markdown folder works out of the box). A type that
/// fails validation is dropped with a diagnostic rather than aborting the
/// whole file (golden rule 5, tolerance for imperfect data); the rest still
/// loads.
pub fn load(project_root: &Path) -> (&'static [TypeSchema], Vec<Diagnostic>) {
    let path = project_root.join(STORYTELLER_DIR).join(TYPES_FILE);
    let raw = match std::fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return (&[], Vec::new()),
        Err(err) => {
            return (
                &[],
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
                &[],
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

    (leak_slice(accepted), diagnostics)
}

fn is_snake_case(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
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

    fn write_types(dir: &Path, contents: &str) {
        let storyteller = dir.join(STORYTELLER_DIR);
        std::fs::create_dir_all(&storyteller).unwrap();
        std::fs::write(storyteller.join(TYPES_FILE), contents).unwrap();
    }

    #[test]
    fn missing_file_is_not_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let (custom, diagnostics) = load(dir.path());
        assert!(custom.is_empty());
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
        let (custom, diagnostics) = load(dir.path());
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(custom.len(), 1);
        let schema = &custom[0];
        assert_eq!(schema.name, "artifact");
        assert_eq!(schema.label, "Artéfact");
        assert_eq!(schema.folder, "artifacts");
        assert_eq!(schema.fields.len(), 3);
        assert_eq!(schema.field("rarity").unwrap().enum_values, ["common", "rare", "legendary"]);

        assert!(types::type_schema("artifact", custom).is_some());
        assert_eq!(types::folder_for("artifact", custom), Some("artifacts"));
    }

    #[test]
    fn rejects_a_name_colliding_with_a_built_in_type() {
        let dir = tempfile::tempdir().unwrap();
        write_types(
            dir.path(),
            "types:\n  - name: character\n    label: Doublon\n    folder: doublons\n",
        );
        let (custom, diagnostics) = load(dir.path());
        assert!(custom.is_empty());
        assert_eq!(diagnostics[0].code, codes::INVALID_TYPE_DEFINITION);
    }

    #[test]
    fn rejects_a_folder_colliding_with_a_built_in_type() {
        let dir = tempfile::tempdir().unwrap();
        write_types(
            dir.path(),
            "types:\n  - name: gizmo\n    label: Gadget\n    folder: characters\n",
        );
        let (custom, diagnostics) = load(dir.path());
        assert!(custom.is_empty());
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
        let (custom, diagnostics) = load(dir.path());
        assert_eq!(custom.len(), 1);
        assert_eq!(custom[0].name, "gizmo");
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn rejects_an_enum_field_without_values() {
        let dir = tempfile::tempdir().unwrap();
        write_types(
            dir.path(),
            "types:\n  - name: gizmo\n    label: Gadget\n    folder: gizmos\n    fields:\n      - name: state\n        label: État\n        kind: enum\n",
        );
        let (custom, diagnostics) = load(dir.path());
        assert!(custom.is_empty());
        assert_eq!(diagnostics[0].code, codes::INVALID_TYPE_DEFINITION);
    }

    #[test]
    fn rejects_a_field_shadowing_a_common_field() {
        let dir = tempfile::tempdir().unwrap();
        write_types(
            dir.path(),
            "types:\n  - name: gizmo\n    label: Gadget\n    folder: gizmos\n    fields:\n      - name: tags\n        label: Étiquettes\n        kind: list\n",
        );
        let (custom, diagnostics) = load(dir.path());
        assert!(custom.is_empty());
        assert_eq!(diagnostics[0].code, codes::INVALID_TYPE_DEFINITION);
    }

    #[test]
    fn malformed_yaml_is_reported_and_yields_no_custom_type() {
        let dir = tempfile::tempdir().unwrap();
        write_types(dir.path(), "types: [not, a, mapping");
        let (custom, diagnostics) = load(dir.path());
        assert!(custom.is_empty());
        assert_eq!(diagnostics[0].code, codes::YAML_PARSE_ERROR);
    }
}
