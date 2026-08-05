//! Non-destructive serialization of entries (`docs/data-model.md` §6).
//!
//! Three invariants govern every function here (golden rule 2):
//!
//! * **Untouched keys survive byte-for-byte.** Editing frontmatter copies the
//!   original text of every key it does not change — comments, quoting style and
//!   blank lines included. Only the keys actually modified are re-emitted.
//! * **The body is never reformatted.** It is passed through verbatim.
//! * **Writing is deterministic.** Same inputs, same bytes — clean Git diffs.
//!
//! Emitted values follow the house style of `docs/data-model.md` §6: lists one
//! element per line, minimal quoting, and **double-quoted wikilink strings**
//! (`"[[Aria]]"`), which YAML would otherwise read as a nested flow sequence.

use crate::model::{Frontmatter, Value};

/// Frontmatter/body delimiter.
const DELIMITER: &str = "---";

/// Current UTC time as an RFC 3339 string with second precision, the form used
/// for `created`/`updated` (`docs/data-model.md` §2).
///
/// Second precision matches the timestamps a user reads in the file; the
/// sub-second part carries no meaning here and would only add churn to diffs.
pub fn now_rfc3339() -> String {
    use time::format_description::well_known::Rfc3339;
    time::OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
        .format(&Rfc3339)
        .unwrap_or_default()
}

/// Assembles a full entry file from frontmatter YAML text and a body.
///
/// The body is written exactly as given. `yaml` is the text that sits between
/// the `---` delimiters; an empty `yaml` still produces a (degenerate but valid)
/// empty frontmatter block, which only happens on a freshly created stub.
pub fn assemble(yaml: &str, body: &str) -> String {
    let mut out = String::with_capacity(yaml.len() + body.len() + 8);
    out.push_str(DELIMITER);
    out.push('\n');
    out.push_str(yaml);
    if !yaml.is_empty() && !yaml.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(DELIMITER);
    out.push('\n');
    out.push_str(body);
    out
}

/// Serializes a whole frontmatter map to YAML, in key order (entry creation).
pub fn serialize_frontmatter(frontmatter: &Frontmatter) -> String {
    let mut out = String::new();
    for (key, value) in frontmatter {
        out.push_str(&serialize_pair(key, value));
    }
    out
}

/// Applies key upserts to existing frontmatter YAML **surgically**.
///
/// Every top-level key not named in `upserts` is copied verbatim, so unknown
/// keys, key order, quoting and inline comments are preserved. A key present in
/// `upserts` is re-emitted in place; a key that does not yet exist is appended
/// in the order given. Nothing is ever deleted — clearing a value is expressed
/// as setting it to `null`.
pub fn edit_frontmatter(raw: &str, upserts: &[(String, Value)]) -> String {
    let blocks = split_top_level(raw);
    let mut seen = Vec::new();
    let mut out = String::with_capacity(raw.len() + 64);

    for block in &blocks {
        match &block.key {
            Some(key) => {
                seen.push(key.clone());
                match upserts.iter().find(|(k, _)| k == key) {
                    Some((k, value)) => out.push_str(&serialize_pair(k, value)),
                    None => out.push_str(&block.text),
                }
            }
            // Preamble, continuation lines and comments: always verbatim.
            None => out.push_str(&block.text),
        }
    }

    for (key, value) in upserts {
        if !seen.iter().any(|k| k == key) {
            out.push_str(&serialize_pair(key, value));
        }
    }
    out
}

/// One top-level key and the lines that belong to it, or a keyless run of
/// preamble / continuation / comment lines.
struct Block {
    key: Option<String>,
    text: String,
}

/// Splits raw YAML into verbatim blocks, one per top-level key. Continuation
/// lines (indented list items, nested mappings), blank lines and comments attach
/// to the most recent key, so re-emitting an untouched key keeps them.
fn split_top_level(raw: &str) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();
    let mut current = Block {
        key: None,
        text: String::new(),
    };

    for line in raw.split_inclusive('\n') {
        match top_level_key(line) {
            Some(key) => {
                if !current.text.is_empty() || current.key.is_some() {
                    blocks.push(current);
                }
                current = Block {
                    key: Some(key),
                    text: line.to_string(),
                };
            }
            None => current.text.push_str(line),
        }
    }
    if !current.text.is_empty() || current.key.is_some() {
        blocks.push(current);
    }
    blocks
}

/// The key of a `key: …` line at column zero, or `None` for anything else
/// (indented, blank, comment, list item). Mirrors the salvage reader in
/// [`crate::parse`] so both agree on what a top-level key looks like.
fn top_level_key(line: &str) -> Option<String> {
    if line.starts_with(char::is_whitespace) || line.trim_start().starts_with('#') {
        return None;
    }
    let (key, _) = line.split_once(':')?;
    let key = key.trim();
    if key.is_empty() || !key.chars().all(is_plain_key_char) {
        return None;
    }
    Some(key.to_string())
}

fn is_plain_key_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')
}

/// Renders one `key: value` pair, list values one element per line.
fn serialize_pair(key: &str, value: &Value) -> String {
    match value {
        Value::Array(items) if items.iter().all(is_scalar) => {
            if items.is_empty() {
                return format!("{key}: []\n");
            }
            let mut out = format!("{key}:\n");
            for item in items {
                out.push_str("  - ");
                out.push_str(&emit_scalar(item));
                out.push('\n');
            }
            out
        }
        // Nested mappings, or lists holding them, are rare in what the app
        // writes; fall back to the YAML emitter and indent under the key.
        Value::Array(_) | Value::Object(_) => {
            let nested = serde_norway::to_string(value).unwrap_or_default();
            let mut out = format!("{key}:\n");
            for line in nested.lines() {
                out.push_str("  ");
                out.push_str(line);
                out.push('\n');
            }
            out
        }
        Value::Null => format!("{key}:\n"),
        scalar => format!("{key}: {}\n", emit_scalar(scalar)),
    }
}

fn is_scalar(value: &Value) -> bool {
    !matches!(value, Value::Array(_) | Value::Object(_))
}

/// A single scalar in inline form.
fn emit_scalar(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => emit_string(s),
        // Non-scalars never reach here (guarded by `is_scalar`); stay total.
        other => other.to_string(),
    }
}

/// A string scalar, quoted only when YAML would otherwise misread it.
fn emit_string(s: &str) -> String {
    if needs_quoting(s) {
        double_quote(s)
    } else {
        s.to_string()
    }
}

/// Whether a bare string would be misparsed and therefore needs quotes.
///
/// Deliberately errs toward quoting: an over-quoted value is still correct,
/// whereas an under-quoted `[[Aria]]` silently becomes a nested list.
fn needs_quoting(s: &str) -> bool {
    if s.is_empty() || s != s.trim() {
        return true;
    }
    // Reserved words YAML would read as a bool/null rather than text.
    if matches!(
        s.to_ascii_lowercase().as_str(),
        "true" | "false" | "null" | "yes" | "no" | "on" | "off" | "~"
    ) {
        return true;
    }
    // A number-looking string must stay a string.
    if s.parse::<i64>().is_ok() || s.parse::<f64>().is_ok() {
        return true;
    }
    let first = s.chars().next().unwrap_or(' ');
    // `[` and `{` start flow collections — this is what forces `"[[Aria]]"`.
    if "!&*?|>%@`\"'#,[]{}:-".contains(first) {
        return true;
    }
    s.contains(": ") || s.contains(" #") || s.contains('\n') || s.contains('\t')
}

fn double_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Derives an ASCII `kebab-case` slug from a title (`docs/data-model.md` §3).
///
/// Diacritics are folded (`Cité` → `cite`), runs of non-alphanumeric characters
/// collapse to a single `-`, and the result is lowercased. Returns `None` when
/// nothing sluggable remains (e.g. a title made only of punctuation or of a
/// script with no ASCII folding), which the caller reports rather than writing a
/// file with an empty name.
pub fn slugify(title: &str) -> Option<String> {
    use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

    let folded: String = title
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .collect::<String>()
        .to_lowercase();

    let mut out = String::with_capacity(folded.len());
    let mut pending_dash = false;
    for c in folded.chars() {
        if c.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.push(c);
        } else {
            pending_dash = true;
        }
    }
    (!out.is_empty()).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_document;

    fn fm(pairs: &[(&str, Value)]) -> Frontmatter {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    #[test]
    fn assemble_wraps_frontmatter_and_body() {
        assert_eq!(
            assemble("type: note\n", "corps\n"),
            "---\ntype: note\n---\ncorps\n"
        );
    }

    #[test]
    fn no_op_edit_is_byte_identical() {
        let raw = "type: character\ntitle: Aria\naliases:\n  - La Verrière\n# a comment\nage: 24\n";
        assert_eq!(edit_frontmatter(raw, &[]), raw);
    }

    #[test]
    fn edit_replaces_a_key_in_place_and_keeps_the_rest() {
        let raw = "type: character\ntitle: Aria\nage: 24\n";
        let edited = edit_frontmatter(raw, &[("title".into(), Value::from("Aria Solane"))]);
        assert_eq!(edited, "type: character\ntitle: Aria Solane\nage: 24\n");
    }

    #[test]
    fn edit_appends_an_absent_key() {
        let raw = "type: note\n";
        let edited = edit_frontmatter(
            raw,
            &[("updated".into(), Value::from("2026-08-05T00:00:00Z"))],
        );
        assert_eq!(edited, "type: note\nupdated: 2026-08-05T00:00:00Z\n");
    }

    #[test]
    fn edit_preserves_unknown_keys_comments_and_order() {
        let raw = "zeta: 1\n# kept\nobsidian_id: abc\nalpha: 2\n";
        let edited = edit_frontmatter(raw, &[("alpha".into(), Value::from(9))]);
        assert_eq!(edited, "zeta: 1\n# kept\nobsidian_id: abc\nalpha: 9\n");
    }

    #[test]
    fn edit_replaces_a_multiline_list_value() {
        let raw = "tags:\n  - old-a\n  - old-b\nnext: x\n";
        let edited = edit_frontmatter(raw, &[("tags".into(), serde_json::json!(["new"]))]);
        assert_eq!(edited, "tags:\n  - new\nnext: x\n");
    }

    #[test]
    fn serialize_quotes_wikilinks_and_lays_lists_one_per_line() {
        let out = serialize_frontmatter(&fm(&[
            ("type", Value::from("chapter")),
            ("pov", Value::from("[[Aria Solane]]")),
            (
                "locations",
                serde_json::json!(["[[Cité de Verre]]", "[[Port]]"]),
            ),
        ]));
        assert_eq!(
            out,
            "type: chapter\npov: \"[[Aria Solane]]\"\nlocations:\n  - \"[[Cité de Verre]]\"\n  - \"[[Port]]\"\n"
        );
    }

    #[test]
    fn serialize_keeps_number_looking_strings_as_strings() {
        assert_eq!(serialize_pair("v", &Value::from("42")), "v: \"42\"\n");
        assert_eq!(serialize_pair("v", &Value::from(42)), "v: 42\n");
        assert_eq!(serialize_pair("v", &Value::from(true)), "v: true\n");
        assert_eq!(serialize_pair("v", &Value::from("true")), "v: \"true\"\n");
        assert_eq!(serialize_pair("v", &Value::Null), "v:\n");
    }

    #[test]
    fn round_trips_a_realistic_frontmatter_through_parse() {
        let source = "---\ntype: character\ntitle: Aria Solane\naliases:\n  - La Verrière\n  - Aria\nspecies: \"[[Humain]]\"\nage: 24\n---\nCorps avec [[Cité de Verre]].\n";
        let doc = parse_document(source);
        let rebuilt = assemble(&doc.raw_frontmatter.unwrap(), &doc.body);
        assert_eq!(
            rebuilt, source,
            "an untouched read/write must not alter bytes"
        );
    }

    #[test]
    fn slugify_folds_accents_and_kebab_cases() {
        assert_eq!(slugify("Aria Solane").as_deref(), Some("aria-solane"));
        assert_eq!(slugify("Cité de Verre").as_deref(), Some("cite-de-verre"));
        assert_eq!(
            slugify("  L'Ordre du Prisme !").as_deref(),
            Some("l-ordre-du-prisme")
        );
        assert_eq!(slugify("03 — La Fêlure").as_deref(), Some("03-la-felure"));
    }

    #[test]
    fn slugify_rejects_a_title_with_nothing_sluggable() {
        assert_eq!(slugify("   "), None);
        assert_eq!(slugify("——"), None);
    }
}
