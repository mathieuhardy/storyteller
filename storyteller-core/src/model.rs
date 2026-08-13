//! The in-memory shape of an entry.
//!
//! Serialization here *is* the API representation described in `docs/api.md` §2:
//! the server layer adds no reshaping, so the contract lives in one place.

use serde::{Deserialize, Serialize};

use crate::error::Diagnostic;
use crate::links::Backlink;

/// A dynamic frontmatter value.
///
/// We reuse `serde_json::Value` as the generic value tree rather than defining
/// yet another enum: it is ordered (`preserve_order`), directly serializable,
/// and lossless for everything YAML can express in a frontmatter. YAML-specific
/// niceties (comments, quoting style, anchors) are *not* carried here — they are
/// preserved for non-destructive writing via [`crate::parse::ParsedDocument::raw_frontmatter`].
pub type Value = serde_json::Value;

/// Frontmatter as an **ordered** key → value map.
///
/// Key order is the file's order, and unknown keys are kept: both are required
/// by the non-destructive writing rule (`docs/data-model.md` §6).
pub type Frontmatter = serde_json::Map<String, Value>;

/// Type name used when `type` is missing or unusable (`docs/data-model.md` §2).
pub const FALLBACK_TYPE: &str = "note";

/// A full entry: one markdown file, parsed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// Identity of the entry = filename without extension (`docs/glossary.md#slug`).
    pub slug: String,
    /// Path relative to the project root, always with `/` separators.
    pub path: String,
    /// Effective type. Mirrors `frontmatter.type`, or [`FALLBACK_TYPE`] when
    /// absent. An unknown-but-present type value is kept verbatim and flagged.
    #[serde(rename = "type")]
    pub type_name: String,
    pub frontmatter: Frontmatter,
    /// Raw markdown body, byte-for-byte as on disk.
    pub body: String,
    /// Incoming links, only when explicitly requested (`?include=backlinks`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backlinks: Option<Vec<Backlink>>,
    /// Per-entry diagnostics; empty when all is well. Never empty *and* fatal.
    pub errors: Vec<Diagnostic>,
}

impl Entry {
    /// Displayed name: `title` when present and non-empty, else the slug.
    pub fn title(&self) -> &str {
        self.frontmatter
            .get("title")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .unwrap_or(&self.slug)
    }

    /// Declared aliases (`aliases`), tolerating a single string instead of a list.
    pub fn aliases(&self) -> Vec<String> {
        string_list(self.frontmatter.get("aliases"))
    }

    /// Declared tags (`tags`), tolerating a single string instead of a list.
    pub fn tags(&self) -> Vec<String> {
        string_list(self.frontmatter.get("tags"))
    }

    pub fn created(&self) -> Option<&str> {
        self.frontmatter.get("created").and_then(Value::as_str)
    }

    pub fn updated(&self) -> Option<&str> {
        self.frontmatter.get("updated").and_then(Value::as_str)
    }

    /// First non-empty, non-heading body line, trimmed to `max_chars`.
    /// Used as the list excerpt; purely derived, never stored in the file.
    pub fn excerpt(&self, max_chars: usize) -> String {
        let line = self
            .body
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("---"))
            .unwrap_or("");
        truncate_chars(line, max_chars)
    }
}

/// A light entry, as returned by list endpoints (`docs/api.md` §4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntrySummary {
    pub slug: String,
    pub path: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub title: String,
    pub tags: Vec<String>,
    pub excerpt: String,
    /// `true` when the entry carries at least one diagnostic, so a view can
    /// badge it without fetching the whole entry.
    pub has_errors: bool,
    /// `updated` frontmatter value (RFC 3339), empty when absent — lets a list
    /// view show/sort recency without fetching the full entry.
    pub updated: String,
}

/// Reads a field as a list of strings, accepting a bare string as a 1-element
/// list. Anything else yields an empty list (tolerance, never a hard failure).
pub(crate) fn string_list(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::String(s)) if !s.trim().is_empty() => vec![s.trim().to_string()],
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max_chars).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry_with(frontmatter: Frontmatter, body: &str) -> Entry {
        Entry {
            slug: "aria-solane".into(),
            path: "characters/aria-solane.md".into(),
            type_name: "character".into(),
            frontmatter,
            body: body.into(),
            backlinks: None,
            errors: Vec::new(),
        }
    }

    #[test]
    fn title_falls_back_to_slug() {
        let mut fm = Frontmatter::new();
        assert_eq!(entry_with(fm.clone(), "").title(), "aria-solane");
        fm.insert("title".into(), Value::String("   ".into()));
        assert_eq!(entry_with(fm.clone(), "").title(), "aria-solane");
        fm.insert("title".into(), Value::String("Aria Solane".into()));
        assert_eq!(entry_with(fm, "").title(), "Aria Solane");
    }

    #[test]
    fn string_list_tolerates_a_bare_string() {
        let mut fm = Frontmatter::new();
        fm.insert("tags".into(), Value::String("pov".into()));
        assert_eq!(entry_with(fm, "").tags(), vec!["pov"]);
    }

    #[test]
    fn string_list_ignores_non_string_items() {
        let mut fm = Frontmatter::new();
        fm.insert(
            "tags".into(),
            serde_json::json!(["pov", 3, null, "", "  act-1  "]),
        );
        assert_eq!(entry_with(fm, "").tags(), vec!["pov", "act-1"]);
    }

    #[test]
    fn excerpt_skips_headings_and_blank_lines() {
        let entry = entry_with(Frontmatter::new(), "\n# Titre\n\nAria grandit à Verre.\n");
        assert_eq!(entry.excerpt(50), "Aria grandit à Verre.");
    }

    #[test]
    fn excerpt_truncates_on_char_boundaries() {
        let entry = entry_with(Frontmatter::new(), "ééééé");
        assert_eq!(entry.excerpt(3), "ééé…");
    }
}
