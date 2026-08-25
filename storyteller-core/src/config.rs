//! Project configuration: `.storyteller/config.yaml` (`docs/data-model.md` §5).

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{codes, Diagnostic};
use crate::types::{self, TypeSchema};

/// Technical folder holding the config and the index.
pub const STORYTELLER_DIR: &str = ".storyteller";
/// Config file name, inside [`STORYTELLER_DIR`].
pub const CONFIG_FILE: &str = "config.yaml";
/// Schema version this build writes and understands.
///
/// Bumped **only** for breaking schema changes; additive field work keeps it
/// unchanged (`docs/data-model.md` §8).
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Effective configuration of a project.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectConfig {
    pub schema_version: u32,
    /// Types offered when creating an entry. Disabling a type **never** hides
    /// or deletes existing entries: they stay on disk and stay indexed
    /// (`docs/data-model.md` §7).
    pub enabled_types: Vec<String>,
    /// Problems met while loading. A broken config degrades to the defaults
    /// rather than preventing the project from opening.
    pub errors: Vec<Diagnostic>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            enabled_types: types::catalog()
                .iter()
                .map(|t| t.name.to_string())
                .collect(),
            errors: Vec::new(),
        }
    }
}

impl ProjectConfig {
    /// Defaults with a given catalog of custom types enabled alongside the
    /// built-ins — used by [`Self::load`] so a freshly declared custom type is
    /// offered for creation without the user having to also edit
    /// `enabled_types` by hand.
    fn default_for(custom: &'static [TypeSchema]) -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            enabled_types: types::all_types(custom).map(|t| t.name.to_string()).collect(),
            errors: Vec::new(),
        }
    }

    /// Whether entries of this type may be created.
    pub fn is_enabled(&self, type_name: &str) -> bool {
        self.enabled_types.iter().any(|t| t == type_name)
    }

    /// Path of the config file for a project root.
    pub fn path_in(project_root: &Path) -> PathBuf {
        project_root.join(STORYTELLER_DIR).join(CONFIG_FILE)
    }

    /// Persists `schema_version`/`enabled_types` to `.storyteller/config.yaml`,
    /// creating the folder on first write (a project may not have had one yet —
    /// `default()` covers it until now). `errors` is diagnostic-only and never
    /// written back.
    pub fn save(&self, project_root: &Path) -> std::io::Result<()> {
        let path = Self::path_in(project_root);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let saved = SavedConfig {
            schema_version: self.schema_version,
            enabled_types: &self.enabled_types,
        };
        let yaml = serde_norway::to_string(&saved)
            .unwrap_or_else(|_| "schema_version: 1\nenabled_types: []\n".to_string());
        std::fs::write(path, yaml)
    }

    /// Loads the config, falling back to defaults for anything unreadable.
    ///
    /// A project without `.storyteller/config.yaml` is perfectly valid — a
    /// plain markdown folder must work out of the box (golden rule 1). `custom`
    /// is the project's custom types ([`crate::custom_types::load`]), so that
    /// `enabled_types` validates and defaults against the full catalog, not
    /// just the built-ins.
    pub fn load(project_root: &Path, custom: &'static [TypeSchema]) -> Self {
        let path = Self::path_in(project_root);
        let raw = match std::fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Self::default_for(custom)
            }
            Err(err) => {
                let mut config = Self::default_for(custom);
                config.errors.push(Diagnostic::warning(
                    codes::YAML_PARSE_ERROR,
                    format!("cannot read {}: {err}", path.display()),
                ));
                return config;
            }
        };

        match serde_norway::from_str::<RawConfig>(&raw) {
            Ok(parsed) => {
                let mut config = Self::default_for(custom);
                if let Some(version) = parsed.schema_version {
                    config.schema_version = version;
                }
                if let Some(enabled) = parsed.enabled_types {
                    let (known, unknown): (Vec<_>, Vec<_>) = enabled
                        .into_iter()
                        .partition(|t| types::type_schema(t, custom).is_some());
                    for name in unknown {
                        config.errors.push(
                            Diagnostic::warning(
                                codes::UNKNOWN_TYPE,
                                format!("`enabled_types` mentions unknown type `{name}`"),
                            )
                            .with_field("enabled_types"),
                        );
                    }
                    config.enabled_types = known;
                }
                config
            }
            Err(err) => {
                let mut config = Self::default_for(custom);
                config.errors.push(Diagnostic::warning(
                    codes::YAML_PARSE_ERROR,
                    format!("invalid {}: {err} — using defaults", path.display()),
                ));
                config
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct RawConfig {
    schema_version: Option<u32>,
    enabled_types: Option<Vec<String>>,
}

/// Shape written to disk by [`ProjectConfig::save`] — deliberately narrower than
/// [`ProjectConfig`] itself: `errors` is a read-side diagnostic, never persisted.
#[derive(Serialize)]
struct SavedConfig<'a> {
    schema_version: u32,
    enabled_types: &'a [String],
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_config(contents: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let storyteller = dir.path().join(STORYTELLER_DIR);
        std::fs::create_dir_all(&storyteller).unwrap();
        std::fs::write(storyteller.join(CONFIG_FILE), contents).unwrap();
        dir
    }

    #[test]
    fn missing_config_enables_every_type() {
        let dir = tempfile::tempdir().unwrap();
        let config = ProjectConfig::load(dir.path(), &[]);
        assert_eq!(config.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(config.enabled_types.len(), types::catalog().len());
        assert!(config.errors.is_empty());
    }

    #[test]
    fn reads_enabled_types_and_version() {
        let dir = write_config("schema_version: 1\nenabled_types:\n  - character\n  - chapter\n");
        let config = ProjectConfig::load(dir.path(), &[]);
        assert_eq!(config.enabled_types, ["character", "chapter"]);
        assert!(config.is_enabled("chapter"));
        assert!(!config.is_enabled("faction"));
        assert!(config.errors.is_empty());
    }

    #[test]
    fn unknown_enabled_type_is_dropped_and_flagged() {
        let dir = write_config("enabled_types:\n  - character\n  - dragon\n");
        let config = ProjectConfig::load(dir.path(), &[]);
        assert_eq!(config.enabled_types, ["character"]);
        assert_eq!(config.errors[0].code, codes::UNKNOWN_TYPE);
    }

    #[test]
    fn broken_config_falls_back_to_defaults() {
        let dir = write_config("enabled_types: [character\n");
        let config = ProjectConfig::load(dir.path(), &[]);
        assert_eq!(config.enabled_types.len(), types::catalog().len());
        assert_eq!(config.errors[0].code, codes::YAML_PARSE_ERROR);
    }

    #[test]
    fn save_writes_a_file_that_load_reads_back() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = ProjectConfig::default();
        config.enabled_types.retain(|t| t != "faction");

        config.save(dir.path()).unwrap();
        let reloaded = ProjectConfig::load(dir.path(), &[]);
        assert!(!reloaded.is_enabled("faction"));
        assert!(reloaded.is_enabled("character"));
        assert_eq!(reloaded.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(reloaded.errors.is_empty());
    }

    #[test]
    fn save_creates_the_storyteller_folder_if_absent() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!dir.path().join(STORYTELLER_DIR).exists());
        ProjectConfig::default().save(dir.path()).unwrap();
        assert!(ProjectConfig::path_in(dir.path()).exists());
    }
}
