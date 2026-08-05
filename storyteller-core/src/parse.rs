//! Splitting an entry into frontmatter + body, tolerantly.
//!
//! Two invariants drive this module (`docs/principles.md`):
//!
//! * **The body is never touched.** It is a byte-exact slice of the source,
//!   starting right after the closing `---` line.
//! * **Nothing is ever rejected.** Broken YAML produces diagnostics and, when
//!   possible, a partially salvaged frontmatter — never a failed read.
//!
//! [`ParsedDocument::raw_frontmatter`] keeps the original YAML text so that
//! non-destructive writing (M2) can edit it surgically instead of re-serializing.

use crate::error::{codes, Diagnostic};
use crate::model::{Frontmatter, Value};

/// Result of parsing one markdown source.
#[derive(Debug, Clone)]
pub struct ParsedDocument {
    /// Interpreted frontmatter, ordered, possibly partial or empty.
    pub frontmatter: Frontmatter,
    /// Body, byte-for-byte as in the source.
    pub body: String,
    /// Exact YAML text between the `---` delimiters, `None` when the file has
    /// no frontmatter block at all.
    pub raw_frontmatter: Option<String>,
    /// Byte offset of [`Self::body`] within the source. Offsets of links found
    /// in the body are relative to the body, so `body_offset + link.offset`
    /// gives a position in the file.
    pub body_offset: usize,
    pub diagnostics: Vec<Diagnostic>,
}

const DELIMITER: &str = "---";

/// Parses a markdown source into frontmatter + body.
///
/// A frontmatter block is recognized only when the document *starts* with a
/// `---` line (a leading BOM is tolerated). Anything else is a body-only
/// document, which is valid: `type` then defaults to `note` upstream.
pub fn parse_document(source: &str) -> ParsedDocument {
    let mut diagnostics = Vec::new();

    let Some((raw_yaml, yaml_start, body_offset)) = locate_frontmatter(source) else {
        if starts_with_delimiter(source) {
            // Opening delimiter but no closing one: the user is mid-edit or the
            // file is truncated. Treat everything as body so nothing is lost.
            diagnostics.push(Diagnostic::error(
                codes::YAML_PARSE_ERROR,
                "unterminated frontmatter: no closing `---` line",
            ));
        }
        return ParsedDocument {
            frontmatter: Frontmatter::new(),
            body: source.to_string(),
            raw_frontmatter: None,
            body_offset: 0,
            diagnostics,
        };
    };

    let frontmatter = parse_frontmatter(raw_yaml, yaml_start, source, &mut diagnostics);

    ParsedDocument {
        frontmatter,
        body: source[body_offset..].to_string(),
        raw_frontmatter: Some(raw_yaml.to_string()),
        body_offset,
        diagnostics,
    }
}

/// Returns `(raw_yaml, yaml_start_offset, body_start_offset)`.
fn locate_frontmatter(source: &str) -> Option<(&str, usize, usize)> {
    let bom_len = if source.starts_with('\u{feff}') {
        '\u{feff}'.len_utf8()
    } else {
        0
    };
    let after_bom = &source[bom_len..];

    let first_line_end = line_end(after_bom, 0);
    if !is_delimiter_line(&after_bom[..first_line_end]) {
        return None;
    }
    let yaml_start = bom_len + skip_newline(after_bom, first_line_end);

    // Scan line by line for the closing delimiter.
    let mut cursor = yaml_start;
    while cursor < source.len() {
        let end = line_end(source, cursor);
        if is_delimiter_line(&source[cursor..end]) {
            let body_start = skip_newline(source, end);
            return Some((&source[yaml_start..cursor], yaml_start, body_start));
        }
        cursor = skip_newline(source, end);
    }
    None
}

fn starts_with_delimiter(source: &str) -> bool {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    is_delimiter_line(&source[..line_end(source, 0)])
}

/// A delimiter line is `---` possibly followed by trailing whitespace (and a
/// `\r` when the file uses CRLF endings).
fn is_delimiter_line(line: &str) -> bool {
    line.trim_end() == DELIMITER
}

/// Byte offset of the end of the line starting at `from` (excluding `\n`).
fn line_end(source: &str, from: usize) -> usize {
    source[from..]
        .find('\n')
        .map(|i| from + i)
        .unwrap_or(source.len())
}

/// Byte offset just after the newline ending at `line_end`.
fn skip_newline(source: &str, line_end: usize) -> usize {
    if line_end < source.len() {
        line_end + 1
    } else {
        source.len()
    }
}

fn parse_frontmatter(
    raw_yaml: &str,
    yaml_start: usize,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Frontmatter {
    if raw_yaml.trim().is_empty() {
        return Frontmatter::new();
    }

    match serde_norway::from_str::<serde_norway::Value>(raw_yaml) {
        Ok(serde_norway::Value::Mapping(mapping)) => yaml_mapping_to_frontmatter(&mapping),
        Ok(serde_norway::Value::Null) => Frontmatter::new(),
        Ok(_) => {
            diagnostics.push(Diagnostic::error(
                codes::YAML_PARSE_ERROR,
                "frontmatter is not a YAML mapping (expected `key: value` pairs)",
            ));
            Frontmatter::new()
        }
        Err(err) => {
            let line = yaml_error_line(&err, yaml_start, source);
            diagnostics.push(Diagnostic::error(
                codes::YAML_PARSE_ERROR,
                match line {
                    Some(line) => format!("invalid YAML frontmatter line {line}: {err}"),
                    None => format!("invalid YAML frontmatter: {err}"),
                },
            ));
            salvage_frontmatter(raw_yaml)
        }
    }
}

/// Converts a YAML error position (relative to the frontmatter) into a line
/// number in the file, so the user can jump straight to it.
fn yaml_error_line(err: &serde_norway::Error, yaml_start: usize, source: &str) -> Option<usize> {
    let within_yaml = err.location()?.line();
    let lines_before = source[..yaml_start].matches('\n').count();
    Some(lines_before + within_yaml)
}

/// Best-effort recovery from broken YAML: keep the top-level `key: value` pairs
/// that parse on their own, drop the rest.
///
/// This is what makes a half-broken entry still browsable — the user sees the
/// title and type in a list instead of an opaque "unreadable file" row
/// (`docs/api.md` §6).
fn salvage_frontmatter(raw_yaml: &str) -> Frontmatter {
    let mut out = Frontmatter::new();
    for line in raw_yaml.lines() {
        // Top level only: indented lines belong to a structure we just failed
        // to parse, so guessing their shape would invent data.
        if line.starts_with(char::is_whitespace) || line.trim_start().starts_with('#') {
            continue;
        }
        let Some((key, _)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() || !key.chars().all(is_plain_key_char) {
            continue;
        }
        if let Ok(serde_norway::Value::Mapping(m)) =
            serde_norway::from_str::<serde_norway::Value>(line)
        {
            for (k, v) in yaml_mapping_to_frontmatter(&m) {
                // A later duplicate does not overwrite an earlier salvaged key,
                // matching YAML's own "first wins" behaviour on our reader.
                out.entry(k).or_insert(v);
            }
        }
    }
    out
}

fn is_plain_key_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')
}

fn yaml_mapping_to_frontmatter(mapping: &serde_norway::Mapping) -> Frontmatter {
    let mut out = Frontmatter::new();
    for (key, value) in mapping {
        out.insert(yaml_key_to_string(key), yaml_to_json(value));
    }
    out
}

/// Frontmatter keys are strings by convention (`snake_case`, English). A
/// non-string key is stringified rather than dropped: we never lose data.
fn yaml_key_to_string(key: &serde_norway::Value) -> String {
    match key {
        serde_norway::Value::String(s) => s.clone(),
        other => match yaml_to_json(other) {
            Value::String(s) => s,
            v => v.to_string(),
        },
    }
}

fn yaml_to_json(value: &serde_norway::Value) -> Value {
    match value {
        serde_norway::Value::Null => Value::Null,
        serde_norway::Value::Bool(b) => Value::Bool(*b),
        serde_norway::Value::Number(n) => number_to_json(n),
        serde_norway::Value::String(s) => Value::String(s.clone()),
        serde_norway::Value::Sequence(items) => {
            Value::Array(items.iter().map(yaml_to_json).collect())
        }
        serde_norway::Value::Mapping(m) => Value::Object(yaml_mapping_to_frontmatter(m)),
        // `!tag value`: the tag is display metadata we have no use for; keep the
        // value so the data survives the round-trip through the index.
        serde_norway::Value::Tagged(tagged) => yaml_to_json(&tagged.value),
    }
}

fn number_to_json(n: &serde_norway::Number) -> Value {
    if let Some(i) = n.as_i64() {
        return Value::from(i);
    }
    if let Some(u) = n.as_u64() {
        return Value::from(u);
    }
    match n.as_f64().and_then(serde_json::Number::from_f64) {
        Some(f) => Value::Number(f),
        // NaN and infinities have no JSON form: keep the YAML spelling.
        None => Value::String(n.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_frontmatter_and_body() {
        let source = "---\ntype: character\ntitle: Aria\n---\nCorps libre.\n";
        let doc = parse_document(source);
        assert_eq!(doc.frontmatter["type"], "character");
        assert_eq!(doc.frontmatter["title"], "Aria");
        assert_eq!(doc.body, "Corps libre.\n");
        assert_eq!(
            doc.raw_frontmatter.unwrap(),
            "type: character\ntitle: Aria\n"
        );
        assert!(doc.diagnostics.is_empty());
    }

    #[test]
    fn body_is_a_byte_exact_slice() {
        let source = "---\ntype: note\n---\n\n\n  ## Titre\ttabulé  \n\n\ntexte\n\n";
        let doc = parse_document(source);
        assert_eq!(doc.body, &source[doc.body_offset..]);
        assert_eq!(source, format!("---\ntype: note\n---\n{}", doc.body));
    }

    #[test]
    fn keeps_frontmatter_key_order() {
        let source = "---\nzeta: 1\nalpha: 2\nmiddle: 3\n---\n";
        let keys: Vec<_> = parse_document(source).frontmatter.keys().cloned().collect();
        assert_eq!(keys, ["zeta", "alpha", "middle"]);
    }

    #[test]
    fn keeps_unknown_keys() {
        let source = "---\ntype: character\nobsidian_plugin_state: {a: 1}\n---\n";
        let doc = parse_document(source);
        assert!(doc.frontmatter.contains_key("obsidian_plugin_state"));
    }

    #[test]
    fn handles_document_without_frontmatter() {
        let source = "# Juste du markdown\n\n[[Aria]]\n";
        let doc = parse_document(source);
        assert!(doc.frontmatter.is_empty());
        assert_eq!(doc.body, source);
        assert_eq!(doc.raw_frontmatter, None);
        assert!(doc.diagnostics.is_empty());
    }

    #[test]
    fn handles_empty_frontmatter() {
        let doc = parse_document("---\n---\ncorps\n");
        assert!(doc.frontmatter.is_empty());
        assert_eq!(doc.body, "corps\n");
        assert!(doc.diagnostics.is_empty());
    }

    #[test]
    fn handles_crlf_line_endings() {
        let source = "---\r\ntype: note\r\ntitle: Aria\r\n---\r\ncorps\r\n";
        let doc = parse_document(source);
        assert_eq!(doc.frontmatter["title"], "Aria");
        assert_eq!(doc.body, "corps\r\n");
    }

    #[test]
    fn handles_missing_trailing_newline() {
        let doc = parse_document("---\ntype: note\n---");
        assert_eq!(doc.body, "");
        assert!(doc.diagnostics.is_empty());
    }

    #[test]
    fn tolerates_leading_bom() {
        let doc = parse_document("\u{feff}---\ntype: note\n---\ncorps\n");
        assert_eq!(doc.frontmatter["type"], "note");
        assert_eq!(doc.body, "corps\n");
    }

    #[test]
    fn unterminated_frontmatter_is_reported_not_lost() {
        let source = "---\ntype: character\ntitle: Aria\n";
        let doc = parse_document(source);
        assert_eq!(doc.body, source, "no byte may be dropped");
        assert_eq!(doc.diagnostics.len(), 1);
        assert_eq!(doc.diagnostics[0].code, codes::YAML_PARSE_ERROR);
    }

    #[test]
    fn broken_yaml_salvages_readable_keys() {
        // The doc's own example (docs/api.md §6): a truncated wikilink breaks
        // the block, yet `type` and `title` must still surface.
        let source = "---\ntype: faction\ntitle: Faction X\nleader: \"[[??\n---\n# Faction X\n";
        let doc = parse_document(source);
        assert_eq!(doc.frontmatter["type"], "faction");
        assert_eq!(doc.frontmatter["title"], "Faction X");
        assert!(!doc.frontmatter.contains_key("leader"));
        assert_eq!(doc.diagnostics[0].code, codes::YAML_PARSE_ERROR);
        assert_eq!(doc.body, "# Faction X\n", "body survives intact");
    }

    #[test]
    fn broken_yaml_reports_a_file_line_number() {
        let source = "---\ntype: faction\n  bad: [\n---\n";
        let doc = parse_document(source);
        let message = &doc.diagnostics[0].message;
        assert!(
            message.contains("line 3") || message.contains("line 4"),
            "unexpected message: {message}"
        );
    }

    #[test]
    fn non_mapping_frontmatter_is_reported() {
        let doc = parse_document("---\n- a\n- b\n---\ncorps\n");
        assert!(doc.frontmatter.is_empty());
        assert_eq!(doc.diagnostics[0].code, codes::YAML_PARSE_ERROR);
        assert_eq!(doc.body, "corps\n");
    }

    #[test]
    fn a_delimiter_inside_the_body_does_not_split_again() {
        let source = "---\ntype: note\n---\navant\n---\naprès\n";
        let doc = parse_document(source);
        assert_eq!(doc.body, "avant\n---\naprès\n");
    }

    #[test]
    fn converts_scalar_kinds() {
        let source =
            "---\nn: 3\nf: 1.5\nb: true\nempty:\nlist:\n  - a\n  - b\nnested:\n  k: v\n---\n";
        let fm = parse_document(source).frontmatter;
        assert_eq!(fm["n"], 3);
        assert_eq!(fm["f"], 1.5);
        assert_eq!(fm["b"], true);
        assert_eq!(fm["empty"], Value::Null);
        assert_eq!(fm["list"], serde_json::json!(["a", "b"]));
        assert_eq!(fm["nested"], serde_json::json!({"k": "v"}));
    }

    #[test]
    fn dates_stay_strings() {
        // No timeline in Storyteller: `created`/`updated` are opaque RFC 3339
        // strings, never parsed into a temporal type.
        let fm = parse_document("---\ncreated: 2026-07-12T09:15:00Z\n---\n").frontmatter;
        assert_eq!(fm["created"], "2026-07-12T09:15:00Z");
    }
}
