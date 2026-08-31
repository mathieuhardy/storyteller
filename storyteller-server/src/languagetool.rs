//! LanguageTool integration for book mode: `.storyteller/languagetool.yaml`
//!
//! Provides grammar and spelling checking via a local LanguageTool server.
//! Configuration (server URL, default language) is stored in the book's
//! `.storyteller/` directory.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use storyteller_core::config::STORYTELLER_DIR;
use storyteller_core::error::{codes, Diagnostic};

/// File name, inside [`STORYTELLER_DIR`], holding LanguageTool config.
pub const LANGUAGETOOL_FILE: &str = "languagetool.yaml";

/// LanguageTool configuration for a book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageToolConfig {
    /// URL of the LanguageTool server (e.g., "http://localhost:8081").
    #[serde(default = "default_server_url")]
    pub server_url: String,
    /// Default language code (e.g., "fr", "en-US").
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_server_url() -> String {
    "http://localhost:8081".to_string()
}

fn default_language() -> String {
    "fr".to_string()
}

impl Default for LanguageToolConfig {
    fn default() -> Self {
        Self {
            server_url: default_server_url(),
            language: default_language(),
        }
    }
}

/// Loaded LanguageTool config with any parse errors.
#[derive(Debug, Clone, Default)]
pub struct LoadedConfig {
    pub config: LanguageToolConfig,
    pub errors: Vec<Diagnostic>,
}

impl LoadedConfig {
    /// Path of the config file for a book root.
    pub fn path_in(book_root: &Path) -> PathBuf {
        book_root.join(STORYTELLER_DIR).join(LANGUAGETOOL_FILE)
    }

    /// Loads the config, falling back to defaults if missing or invalid.
    pub fn load(book_root: &Path) -> Self {
        let path = Self::path_in(book_root);
        let raw = match std::fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Self::default();
            }
            Err(err) => {
                return Self {
                    config: LanguageToolConfig::default(),
                    errors: vec![Diagnostic::warning(
                        codes::YAML_PARSE_ERROR,
                        format!("cannot read {}: {err}", path.display()),
                    )],
                };
            }
        };

        match serde_norway::from_str::<LanguageToolConfig>(&raw) {
            Ok(config) => Self {
                config,
                errors: Vec::new(),
            },
            Err(err) => Self {
                config: LanguageToolConfig::default(),
                errors: vec![Diagnostic::warning(
                    codes::YAML_PARSE_ERROR,
                    format!("invalid {}: {err} — using defaults", path.display()),
                )],
            },
        }
    }

    /// Saves config to `.storyteller/languagetool.yaml`.
    pub fn save(config: &LanguageToolConfig, book_root: &Path) -> std::io::Result<()> {
        let path = Self::path_in(book_root);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let yaml = serde_norway::to_string(config)
            .unwrap_or_else(|_| "server_url: http://localhost:8081\nlanguage: fr\n".to_string());
        std::fs::write(path, yaml)
    }
}

/// A single match from LanguageTool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LTMatch {
    pub message: String,
    #[serde(default)]
    pub short_message: Option<String>,
    pub offset: usize,
    pub length: usize,
    pub replacements: Vec<LTReplacement>,
    pub rule: LTRule,
    #[serde(default)]
    pub context: Option<LTContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LTReplacement {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LTRule {
    pub id: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LTContext {
    pub text: String,
    pub offset: usize,
    pub length: usize,
}

/// Response from LanguageTool /v2/check endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LTResponse {
    #[serde(default)]
    pub matches: Vec<LTMatch>,
}

/// Check text against a LanguageTool server using curl (blocking).
fn check_text_blocking(server_url: &str, text: &str, language: &str) -> Result<LTResponse, String> {
    let url = format!("{}/v2/check", server_url.trim_end_matches('/'));

    let output = std::process::Command::new("curl")
        .args([
            "-s",
            "-X", "POST",
            "-d", &format!("language={}", language),
            "--data-urlencode", &format!("text={}", text),
            &url,
        ])
        .output()
        .map_err(|e| format!("failed to run curl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("curl failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str::<LTResponse>(&stdout)
        .map_err(|e| format!("failed to parse LanguageTool response: {e}"))
}

/// Check text against a LanguageTool server.
pub async fn check_text(
    server_url: &str,
    text: &str,
    language: &str,
) -> Result<LTResponse, String> {
    let server_url = server_url.to_string();
    let text = text.to_string();
    let language = language.to_string();

    tokio::task::spawn_blocking(move || check_text_blocking(&server_url, &text, &language))
        .await
        .map_err(|e| format!("task failed: {e}"))?
}

/// Test connection to a LanguageTool server using curl (blocking).
fn test_connection_blocking(server_url: &str) -> Result<(), String> {
    let url = format!("{}/v2/check", server_url.trim_end_matches('/'));

    let output = std::process::Command::new("curl")
        .args([
            "-s",
            "-X", "POST",
            "-d", "language=en",
            "-d", "text=test",
            &url,
        ])
        .output()
        .map_err(|e| format!("failed to run curl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("curl failed: {}", stderr));
    }

    Ok(())
}

/// Test connection to a LanguageTool server.
pub async fn test_connection(server_url: &str) -> Result<(), String> {
    let server_url = server_url.to_string();

    tokio::task::spawn_blocking(move || test_connection_blocking(&server_url))
        .await
        .map_err(|e| format!("task failed: {e}"))?
}
