//! Post-save replacement rules for book mode: `.storyteller/replacements.yaml`
//! (`docs/roadmap.md` "Book mode"). Literal find/replace pairs applied to a
//! file's content server-side on every save so the writer doesn't have to
//! type special characters by hand (`--` -> `—`, `...` -> `…`). This is
//! book-only, server-side config — books never touch storyteller-core's
//! `Project`/index, so it stays out of `storyteller-core`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use storyteller_core::config::STORYTELLER_DIR;
use storyteller_core::error::{codes, Diagnostic};

/// File name, inside [`STORYTELLER_DIR`], holding a book's replacement rules.
pub const REPLACEMENTS_FILE: &str = "replacements.yaml";

/// A single literal find/replace pair, applied with plain substring
/// semantics (`str::replace`), not a regex.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplacementRule {
    pub find: String,
    pub replace: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RawReplacements {
    #[serde(default)]
    replacements: Vec<ReplacementRule>,
}

/// A book's replacement rules, loaded from (or about to be saved to)
/// `.storyteller/replacements.yaml`.
#[derive(Debug, Clone, Default)]
pub struct BookReplacements {
    pub rules: Vec<ReplacementRule>,
    /// Problems met while loading. A broken file degrades to no rules rather
    /// than failing the save/read it's attached to.
    pub errors: Vec<Diagnostic>,
}

impl BookReplacements {
    /// Path of the replacements file for a book root.
    pub fn path_in(book_root: &Path) -> PathBuf {
        book_root.join(STORYTELLER_DIR).join(REPLACEMENTS_FILE)
    }

    /// Loads the rules, falling back to none for anything unreadable.
    ///
    /// A book without `.storyteller/replacements.yaml` is perfectly valid —
    /// this feature is opt-in.
    pub fn load(book_root: &Path) -> Self {
        let path = Self::path_in(book_root);
        let raw = match std::fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Self::default(),
            Err(err) => {
                return Self {
                    rules: Vec::new(),
                    errors: vec![Diagnostic::warning(
                        codes::YAML_PARSE_ERROR,
                        format!("cannot read {}: {err}", path.display()),
                    )],
                };
            }
        };

        match serde_norway::from_str::<RawReplacements>(&raw) {
            Ok(parsed) => Self {
                rules: parsed.replacements,
                errors: Vec::new(),
            },
            Err(err) => Self {
                rules: Vec::new(),
                errors: vec![Diagnostic::warning(
                    codes::YAML_PARSE_ERROR,
                    format!("invalid {}: {err} — no replacements loaded", path.display()),
                )],
            },
        }
    }

    /// Persists `rules` to `.storyteller/replacements.yaml`, creating the
    /// folder on first write.
    pub fn save(rules: &[ReplacementRule], book_root: &Path) -> std::io::Result<()> {
        let path = Self::path_in(book_root);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let raw = RawReplacements {
            replacements: rules.to_vec(),
        };
        let yaml = serde_norway::to_string(&raw).unwrap_or_else(|_| "replacements: []\n".to_string());
        std::fs::write(path, yaml)
    }

    /// Applies the rules in list order, plain substring replacement.
    pub fn apply(&self, content: &str) -> String {
        self.rules
            .iter()
            .fold(content.to_string(), |acc, rule| acc.replace(&rule.find, &rule.replace))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_is_a_no_op() {
        let dir = tempfile::tempdir().unwrap();
        let replacements = BookReplacements::load(dir.path());
        assert!(replacements.rules.is_empty());
        assert!(replacements.errors.is_empty());
        assert_eq!(replacements.apply("hello -- world"), "hello -- world");
    }

    #[test]
    fn malformed_yaml_degrades_to_empty_with_a_diagnostic() {
        let dir = tempfile::tempdir().unwrap();
        let path = BookReplacements::path_in(dir.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "not: [valid: yaml: at: all").unwrap();

        let replacements = BookReplacements::load(dir.path());
        assert!(replacements.rules.is_empty());
        assert_eq!(replacements.errors.len(), 1);
        assert_eq!(replacements.errors[0].code, codes::YAML_PARSE_ERROR);
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let rules = vec![
            ReplacementRule {
                find: "--".to_string(),
                replace: "—".to_string(),
            },
            ReplacementRule {
                find: "...".to_string(),
                replace: "…".to_string(),
            },
        ];
        BookReplacements::save(&rules, dir.path()).unwrap();

        let loaded = BookReplacements::load(dir.path());
        assert_eq!(loaded.rules, rules);
        assert!(loaded.errors.is_empty());
    }

    #[test]
    fn apply_runs_rules_in_order_with_substring_semantics() {
        let replacements = BookReplacements {
            rules: vec![
                ReplacementRule {
                    find: "--".to_string(),
                    replace: "—".to_string(),
                },
                ReplacementRule {
                    find: "<<".to_string(),
                    replace: "«".to_string(),
                },
            ],
            errors: Vec::new(),
        };

        assert_eq!(
            replacements.apply("<<hello -- world>>"),
            "«hello — world>>"
        );
    }
}
