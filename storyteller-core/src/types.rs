//! The type catalog: the 11 default types and their field schemas.
//!
//! This is the code mirror of `docs/data-model.md` §4. It drives
//! `GET /api/v1/types`, which the frontend uses to generate forms — hence the
//! French labels (`docs/adr/0010`: keys in English, UI labels in French).
//!
//! ## What "MVP" does *not* mean
//!
//! `docs/data-model.md` marks many fields **MVP**. That tier says the field is
//! *in scope for the MVP*, not that an entry is invalid without it — a character
//! sketched in two lines is a legitimate character. Only `type` is genuinely
//! required, and even then a missing `type` degrades to `note` rather than
//! failing (golden rule 5, tolerance for imperfect data). Validation therefore
//! flags *wrong* values, not *absent* ones — except `type` and `title`, which
//! views need to display anything at all.

use serde::{Deserialize, Serialize};

use crate::error::{codes, Diagnostic};
use crate::model::{Frontmatter, Value, FALLBACK_TYPE};

/// Value kind of a field (`docs/data-model.md` §4, "Value type legend").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FieldKind {
    Text,
    Number,
    Boolean,
    /// Closed list of values, see [`FieldSchema::enum_values`].
    Enum,
    /// List of free texts.
    List,
    /// A single wikilink string, e.g. `pov: "[[Aria]]"`.
    Link,
    /// A list of wikilink strings.
    LinkList,
    /// Relative path to an asset.
    Image,
    /// A list of relative paths to assets.
    ImageList,
    /// Number or text, indifferently (e.g. `age: 24` or `age: "la trentaine"`).
    NumberOrText,
    /// A list, or a single text when there is only one thing to say.
    ListOrText,
}

/// Whether a field is part of the MVP surface or an optional refinement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Mvp,
    Optional,
}

/// Schema of one frontmatter field.
#[derive(Debug, Clone, Serialize)]
pub struct FieldSchema {
    /// Frontmatter key: English, `snake_case`.
    pub name: &'static str,
    /// UI label, in French.
    pub label: &'static str,
    pub kind: FieldKind,
    pub tier: Tier,
    /// `true` only when an entry cannot be interpreted without it.
    pub required: bool,
    /// Allowed values when [`FieldKind::Enum`].
    #[serde(skip_serializing_if = "<[&str]>::is_empty")]
    pub enum_values: &'static [&'static str],
    /// Types the link is expected to point to. Purely informative: resolution
    /// never filters on type, and a link to another type is not an error
    /// (`docs/linking.md` §2 — the *field name* carries the semantics).
    #[serde(skip_serializing_if = "<[&str]>::is_empty")]
    pub link_targets: &'static [&'static str],
}

/// Schema of one entry type.
#[derive(Debug, Clone, Serialize)]
pub struct TypeSchema {
    /// Type name as written in `frontmatter.type`.
    pub name: &'static str,
    /// UI label, in French.
    pub label: &'static str,
    /// Folder holding entries of this type, relative to the project root.
    /// Empty for `project`, whose single entry lives at the root (`project.md`).
    pub folder: &'static str,
    /// Fields specific to this type. Common fields ([`common_fields`]) are
    /// inherited by every type and not repeated here.
    pub fields: &'static [FieldSchema],
}

impl TypeSchema {
    pub fn field(&self, name: &str) -> Option<&'static FieldSchema> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// Common + specific fields, in declaration order.
    pub fn all_fields(&self) -> impl Iterator<Item = &'static FieldSchema> {
        common_fields().iter().chain(self.fields.iter())
    }
}

const fn text(name: &'static str, label: &'static str, tier: Tier) -> FieldSchema {
    field(name, label, FieldKind::Text, tier)
}

const fn field(
    name: &'static str,
    label: &'static str,
    kind: FieldKind,
    tier: Tier,
) -> FieldSchema {
    FieldSchema {
        name,
        label,
        kind,
        tier,
        required: false,
        enum_values: &[],
        link_targets: &[],
    }
}

const fn enumeration(
    name: &'static str,
    label: &'static str,
    tier: Tier,
    enum_values: &'static [&'static str],
) -> FieldSchema {
    FieldSchema {
        enum_values,
        ..field(name, label, FieldKind::Enum, tier)
    }
}

const fn link(
    name: &'static str,
    label: &'static str,
    tier: Tier,
    link_targets: &'static [&'static str],
) -> FieldSchema {
    FieldSchema {
        link_targets,
        ..field(name, label, FieldKind::Link, tier)
    }
}

const fn link_list(
    name: &'static str,
    label: &'static str,
    tier: Tier,
    link_targets: &'static [&'static str],
) -> FieldSchema {
    FieldSchema {
        link_targets,
        ..field(name, label, FieldKind::LinkList, tier)
    }
}

use FieldKind::{Boolean, Image, ImageList, List, ListOrText, Number, NumberOrText};
use Tier::{Mvp, Optional};

/// Fields shared by every type (`docs/data-model.md` §2).
///
/// `created` and `updated` describe **the file**, never the story: Storyteller
/// has no narrative chronology ([ADR 0001](../../docs/adr/0001-no-timeline.md)).
pub fn common_fields() -> &'static [FieldSchema] {
    const COMMON: &[FieldSchema] = &[
        FieldSchema {
            required: true,
            ..text("type", "Type", Mvp)
        },
        text("title", "Titre", Mvp),
        field("aliases", "Alias", List, Mvp),
        field("tags", "Étiquettes", List, Mvp),
        field("cover", "Couverture", Image, Optional),
        text("created", "Créé le", Mvp),
        text("updated", "Modifié le", Mvp),
    ];
    COMMON
}

/// The 11 default types, in the order of `docs/data-model.md` §4.
pub fn catalog() -> &'static [TypeSchema] {
    const CATALOG: &[TypeSchema] = &[
        TypeSchema {
            name: "project",
            label: "Projet",
            folder: "",
            fields: &[
                text("logline", "Pitch", Mvp),
                field("genres", "Genres", List, Mvp),
                enumeration(
                    "status",
                    "Statut",
                    Mvp,
                    &["idea", "draft", "writing", "revision", "complete"],
                ),
            ],
        },
        TypeSchema {
            name: "character",
            label: "Personnage",
            folder: "characters",
            fields: &[
                text("role", "Rôle", Mvp),
                text("desire", "Désir", Mvp),
                text("wound", "Blessure", Mvp),
                text("fear", "Peur", Mvp),
                text("motivation", "Motivation", Mvp),
                text("need", "Besoin", Optional),
                text("lie", "Mensonge", Optional),
                text("flaw", "Faille", Optional),
                text("arc", "Arc", Optional),
                link("species", "Espèce", Optional, &["species"]),
                link("culture", "Culture", Optional, &["culture"]),
                link_list("factions", "Factions", Optional, &["faction"]),
                link("home", "Domicile", Optional, &["location"]),
                text("status", "État", Optional),
                field("portrait", "Portrait", Image, Optional),
                field("age", "Âge", NumberOrText, Optional),
                text("gender", "Genre", Optional),
                text("pronouns", "Pronoms", Optional),
                text("appearance", "Apparence", Optional),
            ],
        },
        TypeSchema {
            name: "location",
            label: "Lieu",
            folder: "locations",
            fields: &[
                text("location_kind", "Nature du lieu", Mvp),
                link("parent", "Lieu parent", Optional, &["location"]),
                link(
                    "ruling_faction",
                    "Faction dirigeante",
                    Optional,
                    &["faction"],
                ),
                link("culture", "Culture", Optional, &["culture"]),
                field("population", "Population", NumberOrText, Optional),
                text("climate", "Climat", Optional),
                field("map", "Carte", Image, Optional),
            ],
        },
        TypeSchema {
            name: "faction",
            label: "Faction",
            folder: "factions",
            fields: &[
                text("org_kind", "Nature de l'organisation", Mvp),
                link("leader", "Chef", Mvp, &["character"]),
                link("parent", "Faction parente", Optional, &["faction"]),
                link("headquarters", "Siège", Optional, &["location"]),
                link_list("allies", "Alliés", Optional, &["faction"]),
                link_list("rivals", "Rivaux", Optional, &["faction"]),
                text("ideology", "Idéologie", Optional),
                field("goals", "Objectifs", ListOrText, Optional),
                link_list("members", "Membres", Optional, &["character"]),
            ],
        },
        TypeSchema {
            name: "object",
            label: "Objet",
            folder: "objects",
            fields: &[
                link("owner", "Propriétaire", Mvp, &["character", "faction"]),
                text("object_kind", "Nature de l'objet", Optional),
                link("location", "Lieu", Optional, &["location"]),
                link("system", "Système", Optional, &["system"]),
                field("powers", "Pouvoirs", ListOrText, Optional),
                field("image", "Image", Image, Optional),
            ],
        },
        TypeSchema {
            name: "culture",
            label: "Culture",
            folder: "cultures",
            fields: &[
                link("homeland", "Terre d'origine", Mvp, &["location"]),
                link_list("species", "Espèces", Optional, &["species"]),
                text("language", "Langue", Optional),
                link("government", "Gouvernement", Optional, &["faction"]),
                field("values", "Valeurs", List, Optional),
                text("religion", "Religion", Optional),
            ],
        },
        TypeSchema {
            name: "system",
            label: "Système",
            folder: "systems",
            fields: &[
                enumeration(
                    "system_kind",
                    "Nature du système",
                    Mvp,
                    &["magic", "technology", "politics", "economy", "religion"],
                ),
                field("rules", "Règles", List, Mvp),
                field("limits", "Limites", List, Mvp),
                text("cost", "Coût", Mvp),
                text("source", "Source", Optional),
                link_list("practitioners", "Praticiens", Optional, &["character"]),
            ],
        },
        TypeSchema {
            name: "species",
            label: "Espèce",
            folder: "species",
            fields: &[
                text("classification", "Classification", Mvp),
                field("intelligent", "Intelligente", Boolean, Optional),
                link("habitat", "Habitat", Optional, &["location"]),
                field("abilities", "Capacités", List, Optional),
                text("lifespan", "Longévité", Optional),
                field("image", "Image", Image, Optional),
            ],
        },
        TypeSchema {
            name: "chapter",
            label: "Chapitre",
            folder: "chapters",
            // `order` is a position in the manuscript, never a time axis.
            fields: &[
                field("order", "Position", Number, Mvp),
                link("pov", "Point de vue", Mvp, &["character"]),
                link_list("locations", "Lieux", Mvp, &["location"]),
                enumeration(
                    "chapter_status",
                    "Statut du chapitre",
                    Mvp,
                    &["to-write", "draft", "written", "revised"],
                ),
                text("summary", "Résumé", Mvp),
                link_list("characters", "Personnages", Optional, &["character"]),
                field("plotlines", "Intrigues", List, Optional),
                field("act", "Acte", NumberOrText, Optional),
                field("part", "Partie", NumberOrText, Optional),
                field("wordcount", "Nombre de mots", Number, Optional),
            ],
        },
        TypeSchema {
            name: "note",
            label: "Note",
            folder: "notes",
            fields: &[
                text("resource_kind", "Nature de la ressource", Optional),
                text("source_url", "URL source", Optional),
                link_list("related", "Liens connexes", Optional, &[]),
                field("attachments", "Pièces jointes", ImageList, Optional),
            ],
        },
        TypeSchema {
            name: "concept",
            label: "Concept",
            folder: "concepts",
            fields: &[
                text("category", "Catégorie", Optional),
                link_list("related", "Liens connexes", Optional, &[]),
            ],
        },
    ];
    CATALOG
}

/// Builds a merged catalog containing built-in types (with extensions applied)
/// and custom types (also with extensions applied if any).
///
/// For each type that has extensions, a new `TypeSchema` is created with the
/// original fields plus the extension fields, and leaked. Types without
/// extensions use the original static schema. Custom types are appended at the
/// end.
pub fn merged_catalog(
    extensions: &std::collections::HashMap<String, &'static [FieldSchema]>,
    custom: &'static [TypeSchema],
) -> &'static [TypeSchema] {
    let mut merged: Vec<TypeSchema> = Vec::with_capacity(catalog().len() + custom.len());

    // Apply extensions to built-in types
    for builtin in catalog() {
        merged.push(apply_extensions(builtin, extensions));
    }

    // Apply extensions to custom types
    for custom_type in custom {
        merged.push(apply_extensions(custom_type, extensions));
    }

    Box::leak(merged.into_boxed_slice())
}

/// Applies field extensions to a type schema, returning a new schema if
/// extensions exist or the original if not.
fn apply_extensions(
    schema: &TypeSchema,
    extensions: &std::collections::HashMap<String, &'static [FieldSchema]>,
) -> TypeSchema {
    if let Some(ext_fields) = extensions.get(schema.name) {
        let mut all_fields: Vec<FieldSchema> =
            Vec::with_capacity(schema.fields.len() + ext_fields.len());
        all_fields.extend(schema.fields.iter().cloned());
        all_fields.extend(ext_fields.iter().cloned());

        TypeSchema {
            name: schema.name,
            label: schema.label,
            folder: schema.folder,
            fields: Box::leak(all_fields.into_boxed_slice()),
        }
    } else {
        schema.clone()
    }
}

/// Iterates all types in the catalog.
///
/// The parameter should be the merged catalog from [`merged_catalog`], which
/// contains built-in types (possibly with extensions) plus any custom types.
/// For tests that only need built-ins, pass `catalog()` directly.
pub fn all_types(
    merged: &'static [TypeSchema],
) -> impl Iterator<Item = &'static TypeSchema> {
    merged.iter()
}

/// Looks up a type by name, built-in or custom.
pub fn type_schema(name: &str, custom: &'static [TypeSchema]) -> Option<&'static TypeSchema> {
    all_types(custom).find(|t| t.name == name)
}

/// Folder of a type, or `None` for an unknown type.
pub fn folder_for(type_name: &str, custom: &'static [TypeSchema]) -> Option<&'static str> {
    type_schema(type_name, custom).map(|t| t.folder)
}

/// Effective type of an entry, and the diagnostics that go with it.
///
/// * `type` missing → [`FALLBACK_TYPE`] + warning;
/// * `type` present but unknown → kept **verbatim** + warning. Discarding it
///   would lose user data, and the type may simply come from a newer version.
pub fn resolve_type(
    frontmatter: &Frontmatter,
    custom: &'static [TypeSchema],
) -> (String, Vec<Diagnostic>) {
    let mut diagnostics = Vec::new();
    let declared = frontmatter
        .get("type")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty());

    match declared {
        None => {
            diagnostics.push(
                Diagnostic::warning(
                    codes::MISSING_REQUIRED_FIELD,
                    format!("missing `type`, entry treated as `{FALLBACK_TYPE}`"),
                )
                .with_field("type"),
            );
            (FALLBACK_TYPE.to_string(), diagnostics)
        }
        Some(name) if type_schema(name, custom).is_some() => (name.to_string(), diagnostics),
        Some(name) => {
            diagnostics.push(
                Diagnostic::warning(codes::UNKNOWN_TYPE, format!("unknown type `{name}`"))
                    .with_field("type"),
            );
            (name.to_string(), diagnostics)
        }
    }
}

/// Checks frontmatter values against the schema of `type_name`.
///
/// Only reports what is actually wrong (see the module note on the MVP tier):
/// a missing `title`, and values that contradict their declared kind. Unknown
/// keys are never reported — they are legitimate and preserved.
pub fn validate(
    type_name: &str,
    frontmatter: &Frontmatter,
    custom: &'static [TypeSchema],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let has_title = frontmatter
        .get("title")
        .and_then(Value::as_str)
        .is_some_and(|t| !t.trim().is_empty());
    if !has_title {
        diagnostics.push(
            Diagnostic::warning(
                codes::MISSING_REQUIRED_FIELD,
                "missing `title`, the slug is displayed instead",
            )
            .with_field("title"),
        );
    }

    // An unknown type has no schema to validate against; `resolve_type` already
    // warned about it.
    let Some(schema) = type_schema(type_name, custom) else {
        return diagnostics;
    };

    for field in schema.all_fields() {
        let Some(value) = frontmatter.get(field.name) else {
            continue;
        };
        // An explicitly empty value is "not filled in", not a wrong value.
        if value.is_null() {
            continue;
        }
        if let Some(problem) = check_value(field, value) {
            diagnostics.push(
                Diagnostic::warning(
                    codes::INVALID_FIELD_VALUE,
                    format!("field `{}`: {problem}", field.name),
                )
                .with_field(field.name),
            );
        }
    }

    diagnostics
}

/// Returns a human-readable problem, or `None` when the value is acceptable.
fn check_value(field: &FieldSchema, value: &Value) -> Option<String> {
    match field.kind {
        FieldKind::Enum => {
            let text = value.as_str()?;
            if field.enum_values.contains(&text.trim()) {
                None
            } else {
                Some(format!(
                    "`{text}` is not one of: {}",
                    field.enum_values.join(", ")
                ))
            }
        }
        Number => {
            (!value.is_number()).then(|| format!("expected a number, got {}", kind_of(value)))
        }
        Boolean => {
            (!value.is_boolean()).then(|| format!("expected a boolean, got {}", kind_of(value)))
        }
        // Everything else is deliberately permissive: a text field holding a
        // number, or a list field holding a single item, is workable data.
        _ => None,
    }
}

fn kind_of(value: &Value) -> &'static str {
    match value {
        Value::Null => "nothing",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "text",
        Value::Array(_) => "a list",
        Value::Object(_) => "a mapping",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fm(pairs: &[(&str, Value)]) -> Frontmatter {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    #[test]
    fn catalog_holds_the_eleven_documented_types() {
        let names: Vec<_> = catalog().iter().map(|t| t.name).collect();
        assert_eq!(
            names,
            [
                "project",
                "character",
                "location",
                "faction",
                "object",
                "culture",
                "system",
                "species",
                "chapter",
                "note",
                "concept"
            ]
        );
    }

    #[test]
    fn every_type_has_a_french_label_and_a_folder() {
        for schema in catalog() {
            assert!(!schema.label.is_empty(), "{} has no label", schema.name);
            if schema.name == "project" {
                assert_eq!(schema.folder, "", "project.md lives at the root");
            } else {
                assert!(!schema.folder.is_empty(), "{} has no folder", schema.name);
            }
        }
    }

    #[test]
    fn field_names_are_snake_case_and_unique_per_type() {
        for schema in catalog() {
            let mut seen = std::collections::HashSet::new();
            for field in schema.all_fields() {
                assert!(
                    field
                        .name
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                    "{}.{} is not snake_case",
                    schema.name,
                    field.name
                );
                assert!(!field.label.is_empty());
                assert!(
                    seen.insert(field.name),
                    "{} declares {} twice",
                    schema.name,
                    field.name
                );
            }
        }
    }

    #[test]
    fn enum_fields_declare_their_values_and_others_do_not() {
        for schema in catalog() {
            for field in schema.all_fields() {
                match field.kind {
                    FieldKind::Enum => assert!(
                        !field.enum_values.is_empty(),
                        "{}.{} is an enum without values",
                        schema.name,
                        field.name
                    ),
                    _ => assert!(
                        field.enum_values.is_empty(),
                        "{}.{} is not an enum but declares values",
                        schema.name,
                        field.name
                    ),
                }
            }
        }
    }

    #[test]
    fn link_targets_reference_existing_types() {
        for schema in catalog() {
            for field in schema.all_fields() {
                for target in field.link_targets {
                    assert!(
                        type_schema(target, catalog()).is_some(),
                        "{}.{} points to unknown type {target}",
                        schema.name,
                        field.name
                    );
                }
            }
        }
    }

    #[test]
    fn missing_type_degrades_to_note() {
        let (type_name, diagnostics) = resolve_type(&fm(&[]), catalog());
        assert_eq!(type_name, FALLBACK_TYPE);
        assert_eq!(diagnostics[0].code, codes::MISSING_REQUIRED_FIELD);
    }

    #[test]
    fn unknown_type_is_kept_verbatim_and_flagged() {
        let (type_name, diagnostics) = resolve_type(&fm(&[("type", "prophecy".into())]), catalog());
        assert_eq!(type_name, "prophecy");
        assert_eq!(diagnostics[0].code, codes::UNKNOWN_TYPE);
    }

    #[test]
    fn known_type_is_clean() {
        let (type_name, diagnostics) = resolve_type(&fm(&[("type", "character".into())]), catalog());
        assert_eq!(type_name, "character");
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn validate_flags_bad_enum_value() {
        let diagnostics = validate(
            "chapter",
            &fm(&[
                ("title", "Le Fêlure".into()),
                ("chapter_status", "en-cours".into()),
            ]),
            catalog(),
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, codes::INVALID_FIELD_VALUE);
        assert_eq!(diagnostics[0].field.as_deref(), Some("chapter_status"));
    }

    #[test]
    fn validate_flags_bad_number_and_boolean() {
        let diagnostics = validate(
            "chapter",
            &fm(&[("title", "X".into()), ("order", "troisième".into())]),
            catalog(),
        );
        assert_eq!(diagnostics[0].field.as_deref(), Some("order"));

        let diagnostics = validate(
            "species",
            &fm(&[("title", "X".into()), ("intelligent", "oui".into())]),
            catalog(),
        );
        assert_eq!(diagnostics[0].field.as_deref(), Some("intelligent"));
    }

    #[test]
    fn validate_accepts_empty_and_unknown_fields() {
        let diagnostics = validate(
            "chapter",
            &fm(&[
                ("title", "X".into()),
                ("order", Value::Null),
                ("my_own_field", "peu importe".into()),
            ]),
            catalog(),
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn validate_is_permissive_on_mvp_fields_left_blank() {
        // A character with nothing but a title is valid: the MVP tier is scope,
        // not a constraint.
        let diagnostics = validate("character", &fm(&[("title", "Aria".into())]), catalog());
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn validate_reports_missing_title() {
        let diagnostics = validate("character", &fm(&[("type", "character".into())]), catalog());
        assert_eq!(diagnostics[0].field.as_deref(), Some("title"));
        assert_eq!(diagnostics[0].code, codes::MISSING_REQUIRED_FIELD);
    }

    #[test]
    fn validate_tolerates_number_or_text_fields() {
        for age in [Value::from(24), Value::from("la trentaine")] {
            let diagnostics =
                validate("character", &fm(&[("title", "A".into()), ("age", age)]), catalog());
            assert!(diagnostics.is_empty(), "{diagnostics:?}");
        }
    }
}
