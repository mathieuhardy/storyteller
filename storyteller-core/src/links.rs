//! Wikilinks: extraction, resolution, backlinks (`docs/linking.md`).
//!
//! One single mechanism handles body and frontmatter: every `[[…]]` is
//! extracted, then resolved by the same ranked algorithm. Only the recorded
//! [`LinkContext`] differs.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::model::{Entry, Frontmatter, Value};
use crate::normalize::normalize;

/// Where a link was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkContext {
    /// In the free-form markdown body.
    Body,
    /// In a frontmatter field, by name (`pov`, `locations`, …). The field name
    /// carries the relationship semantics.
    Field(String),
}

impl LinkContext {
    /// Stored/serialized form: `body` or `field:<name>` (`docs/linking.md` §4.1).
    pub fn as_stored(&self) -> String {
        match self {
            LinkContext::Body => "body".to_string(),
            LinkContext::Field(name) => format!("field:{name}"),
        }
    }

    /// Parses back the stored form.
    pub fn from_stored(stored: &str) -> Self {
        match stored.strip_prefix("field:") {
            Some(name) => LinkContext::Field(name.to_string()),
            None => LinkContext::Body,
        }
    }

    /// Field name, or `None` when the link comes from the body — this is the
    /// `field` member of the API's backlink object.
    pub fn field(&self) -> Option<&str> {
        match self {
            LinkContext::Body => None,
            LinkContext::Field(name) => Some(name),
        }
    }
}

/// One wikilink occurrence, as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkOccurrence {
    /// Target exactly as typed, without `[[`/`]]`, `|display` or `#anchor`.
    pub target_raw: String,
    /// Display text of the `[[Target|display]]` form.
    pub display: Option<String>,
    /// `#section` anchor. Display-only: it never affects resolution
    /// (`docs/linking.md` §1).
    pub anchor: Option<String>,
    pub context: LinkContext,
    /// Byte offset of the `[[` (or `![[`) within the body, `None` for
    /// frontmatter links.
    pub offset: Option<usize>,
    /// `![[…]]` form: an asset embed, not a link between entries. Embeds are
    /// indexed to track asset usage but **never** produce stubs.
    pub is_embed: bool,
}

impl LinkOccurrence {
    /// Normalized match key of the target (`docs/linking.md` §3.1).
    pub fn target_key(&self) -> String {
        normalize(&self.target_raw)
    }
}

/// Status of a resolved target (`docs/linking.md` §3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Resolution {
    /// Exactly one entry matches.
    Resolved,
    /// No entry matches: an entry to create.
    Stub,
    /// Two or more entries match at the same rank; never silently picked.
    Ambiguous,
}

impl Resolution {
    pub fn as_str(self) -> &'static str {
        match self {
            Resolution::Resolved => "resolved",
            Resolution::Stub => "stub",
            Resolution::Ambiguous => "ambiguous",
        }
    }
}

/// An outgoing link, resolved. Shape of `GET /entities/{slug}/links`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutgoingLink {
    /// Target as written in the file.
    pub target_raw: String,
    /// Slug of the matched entry, when [`Resolution::Resolved`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_slug: Option<String>,
    /// Frontmatter field the link comes from, `null` when it comes from the body.
    pub field: Option<String>,
    pub resolution: Resolution,
    /// Candidate slugs when [`Resolution::Ambiguous`], for manual disambiguation.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<String>,
}

/// An incoming link. Shape of `GET /entities/{slug}/backlinks`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backlink {
    /// Entry that cites the target.
    pub slug: String,
    pub path: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub title: String,
    /// Frontmatter field that sourced the link, `null` when it is a body link.
    pub field: Option<String>,
    /// Excerpt around the occurrence, for preview.
    pub context: String,
}

/// A link target with no matching entry, grouped by normalized key
/// (`docs/linking.md` §6). Shape of `GET /stubs`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stub {
    /// Normalized key shared by every occurrence.
    pub key: String,
    /// Original texts encountered, preserved as written — the first one is what
    /// "create entry" pre-fills as `title`.
    pub labels: Vec<String>,
    pub count: usize,
    /// Slugs of the entries mentioning it.
    pub sources: Vec<String>,
}

/// A node in the [link graph](../../docs/adr/0018-hand-rolled-graph-layout.md) —
/// every entry, so isolated ones are visible too. Shape of `GET /graph`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub slug: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub title: String,
    pub has_errors: bool,
}

/// One resolved, entry-to-entry link. Unlike [`OutgoingLink`], a stub or an
/// ambiguous link has no single target entry and is never an edge; a repeated
/// link between the same two entries (several fields, several mentions) is
/// collapsed to one — the graph shows *that* two entries are connected, not
/// how many times.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
}

/// The whole project as a network: every entry, and every resolved
/// entry-to-entry link between them. Shape of `GET /graph`
/// ([ADR 0008](../../docs/adr/0008-no-graph-in-mvp.md)).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Graph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

/// Extracts every wikilink from a markdown body.
///
/// Offsets are relative to the body. Note that occurrences inside fenced code
/// blocks are *not* skipped: doing so would need a full markdown parse here,
/// and a spurious stub is a visible, harmless signal — never data loss.
pub fn extract_from_body(body: &str) -> Vec<LinkOccurrence> {
    let bytes = body.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;

    while let Some(found) = find_from(bytes, i, b"[[") {
        let Some(close) = find_from(bytes, found + 2, b"]]") else {
            break;
        };
        let inner = &body[found + 2..close];
        let is_embed = found > 0 && bytes[found - 1] == b'!';
        let start = if is_embed { found - 1 } else { found };

        if let Some(mut occurrence) = parse_inner(inner) {
            occurrence.context = LinkContext::Body;
            occurrence.offset = Some(start);
            occurrence.is_embed = is_embed;
            out.push(occurrence);
        }
        i = close + 2;
    }

    out
}

/// Extracts the wikilinks carried by frontmatter fields.
///
/// A value counts as a link only when the **whole** string is a wikilink; plain
/// text or an empty string is kept as-is and ignored (`docs/linking.md` §2).
/// Nested mappings are walked with a dotted path so that a link buried in a
/// user-invented structure is still indexed.
pub fn extract_from_frontmatter(frontmatter: &Frontmatter) -> Vec<LinkOccurrence> {
    let mut out = Vec::new();
    for (key, value) in frontmatter {
        collect_field_links(key, value, &mut out);
    }
    out
}

fn collect_field_links(path: &str, value: &Value, out: &mut Vec<LinkOccurrence>) {
    match value {
        Value::String(text) => {
            if let Some(occurrence) = parse_standalone(text) {
                out.push(LinkOccurrence {
                    context: LinkContext::Field(path.to_string()),
                    ..occurrence
                });
            }
        }
        // A list field records the same context as a scalar one: `field:locations`,
        // without an index. The relationship is what matters, not the slot.
        Value::Array(items) => {
            for item in items {
                collect_field_links(path, item, out);
            }
        }
        Value::Object(map) => {
            for (key, item) in map {
                collect_field_links(&format!("{path}.{key}"), item, out);
            }
        }
        _ => {}
    }
}

/// Parses a string that must be *exactly* one wikilink, e.g. `"[[Aria]]"`.
fn parse_standalone(text: &str) -> Option<LinkOccurrence> {
    let trimmed = text.trim();
    let inner = trimmed.strip_prefix("[[")?.strip_suffix("]]")?;
    // `[[a]] et [[b]]` is not a link value: it is prose in a field.
    if inner.contains("[[") || inner.contains("]]") {
        return None;
    }
    parse_inner(inner)
}

/// Parses the inside of `[[…]]` into target / display / anchor.
fn parse_inner(inner: &str) -> Option<LinkOccurrence> {
    // Split on `|` first: a `#` on the right-hand side is display text.
    let (target_part, display) = match inner.split_once('|') {
        Some((target, display)) => (target, Some(display.trim().to_string())),
        None => (inner, None),
    };

    let (target, anchor) = match target_part.split_once('#') {
        Some((target, anchor)) => (target, Some(anchor.trim().to_string())),
        None => (target_part, None),
    };

    let target = target.trim();
    if target.is_empty() {
        return None;
    }

    Some(LinkOccurrence {
        target_raw: target.to_string(),
        display: display.filter(|d| !d.is_empty()),
        anchor: anchor.filter(|a| !a.is_empty()),
        context: LinkContext::Body,
        offset: None,
        is_embed: false,
    })
}

/// Decides, for one occurrence, the new inner text of its wikilink, or `None`
/// to leave it untouched. The inner text is what sits between `[[` and `]]`
/// (target, optional `#anchor`, optional `|display`).
pub type Rewrite<'a> = dyn Fn(&LinkOccurrence) -> Option<String> + 'a;

/// Rewrites wikilinks in a markdown body, returning the new body when anything
/// changed. Asset embeds (`![[…]]`) are never rewritten — they point at media,
/// not entries (`docs/linking.md` §1). Occurrences are replaced back-to-front so
/// that earlier offsets stay valid.
pub fn rewrite_body_links(body: &str, rewrite: &Rewrite) -> Option<String> {
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    for occurrence in extract_from_body(body) {
        if occurrence.is_embed {
            continue;
        }
        let Some(start) = occurrence.offset else {
            continue;
        };
        let Some(rel_end) = body[start..].find("]]") else {
            continue;
        };
        if let Some(inner) = rewrite(&occurrence) {
            edits.push((start, start + rel_end + 2, format!("[[{inner}]]")));
        }
    }
    if edits.is_empty() {
        return None;
    }

    let mut out = body.to_string();
    for (start, end, replacement) in edits.into_iter().rev() {
        out.replace_range(start..end, &replacement);
    }
    Some(out)
}

/// Rewrites the wikilink strings carried by a frontmatter value, returning the
/// new value when anything changed. Only whole-string wikilinks count as links
/// (`docs/linking.md` §2), so plain text is passed through untouched.
pub fn rewrite_value_links(value: &Value, rewrite: &Rewrite) -> Option<Value> {
    match value {
        Value::String(text) => {
            let occurrence = parse_standalone(text)?;
            let inner = rewrite(&occurrence)?;
            Some(Value::String(format!("[[{inner}]]")))
        }
        Value::Array(items) => {
            let mut changed = false;
            let rewritten: Vec<Value> = items
                .iter()
                .map(|item| match rewrite_value_links(item, rewrite) {
                    Some(new_item) => {
                        changed = true;
                        new_item
                    }
                    None => item.clone(),
                })
                .collect();
            changed.then(|| Value::Array(rewritten))
        }
        Value::Object(map) => {
            let mut changed = false;
            let mut rewritten = crate::model::Frontmatter::new();
            for (key, item) in map {
                match rewrite_value_links(item, rewrite) {
                    Some(new_item) => {
                        changed = true;
                        rewritten.insert(key.clone(), new_item);
                    }
                    None => {
                        rewritten.insert(key.clone(), item.clone());
                    }
                }
            }
            changed.then(|| Value::Object(rewritten))
        }
        _ => None,
    }
}

fn find_from(haystack: &[u8], from: usize, needle: &[u8; 2]) -> Option<usize> {
    if from >= haystack.len() {
        return None;
    }
    haystack[from..]
        .windows(2)
        .position(|w| w == needle)
        .map(|i| from + i)
}

/// Ranked lookup table over a project's entries (`docs/linking.md` §3.2).
///
/// Ranks are searched in order — filename, then `title`, then `aliases` — and
/// the first rank that yields **exactly one** entry wins. This is what makes a
/// filename beat a homonymous title.
#[derive(Debug, Default)]
pub struct Resolver {
    by_slug: HashMap<String, Vec<String>>,
    by_title: HashMap<String, Vec<String>>,
    by_alias: HashMap<String, Vec<String>>,
}

impl Resolver {
    /// Builds a resolver from every entry of the project.
    pub fn new<'a>(entries: impl IntoIterator<Item = &'a Entry>) -> Self {
        let mut resolver = Self::default();
        for entry in entries {
            resolver.insert(&entry.slug, entry.title(), &entry.aliases());
        }
        resolver
    }

    /// Registers one entry's match keys.
    pub fn insert(&mut self, slug: &str, title: &str, aliases: &[String]) {
        push_unique(&mut self.by_slug, normalize(slug), slug);
        push_unique(&mut self.by_title, normalize(title), slug);
        for alias in aliases {
            push_unique(&mut self.by_alias, normalize(alias), slug);
        }
    }

    /// Resolves a raw target.
    pub fn resolve(&self, target_raw: &str) -> ResolvedTarget {
        self.resolve_key(&normalize(target_raw))
    }

    /// Resolves an already normalized key.
    pub fn resolve_key(&self, key: &str) -> ResolvedTarget {
        for rank in [&self.by_slug, &self.by_title, &self.by_alias] {
            let Some(candidates) = rank.get(key) else {
                continue;
            };
            return match candidates.as_slice() {
                [] => continue,
                [only] => ResolvedTarget {
                    resolution: Resolution::Resolved,
                    slug: Some(only.clone()),
                    candidates: Vec::new(),
                },
                many => ResolvedTarget {
                    resolution: Resolution::Ambiguous,
                    slug: None,
                    // Sorted, so the disambiguation list a user sees does not
                    // depend on the order files happened to be scanned in.
                    candidates: {
                        let mut candidates = many.to_vec();
                        candidates.sort();
                        candidates
                    },
                },
            };
        }
        ResolvedTarget {
            resolution: Resolution::Stub,
            slug: None,
            candidates: Vec::new(),
        }
    }
}

/// Outcome of resolving one target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTarget {
    pub resolution: Resolution,
    /// Matched slug, only when [`Resolution::Resolved`].
    pub slug: Option<String>,
    /// Competing slugs, only when [`Resolution::Ambiguous`].
    pub candidates: Vec<String>,
}

fn push_unique(map: &mut HashMap<String, Vec<String>>, key: String, slug: &str) {
    if key.is_empty() {
        return;
    }
    let slugs = map.entry(key).or_default();
    // Same entry reachable twice by the same key (e.g. an alias equal to the
    // title) must not make it ambiguous with itself.
    if !slugs.iter().any(|s| s == slug) {
        slugs.push(slug.to_string());
    }
}

/// Builds the preview excerpt for a body occurrence: the line holding the link,
/// trimmed and capped.
pub fn body_excerpt(body: &str, offset: usize, max_chars: usize) -> String {
    let line_start = body[..offset.min(body.len())]
        .rfind('\n')
        .map_or(0, |i| i + 1);
    let line_end = body[line_start..]
        .find('\n')
        .map_or(body.len(), |i| line_start + i);
    let line = body[line_start..line_end].trim();

    if line.chars().count() <= max_chars {
        return line.to_string();
    }
    let mut out: String = line.chars().take(max_chars).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn targets(occurrences: &[LinkOccurrence]) -> Vec<&str> {
        occurrences.iter().map(|o| o.target_raw.as_str()).collect()
    }

    #[test]
    fn extracts_plain_links() {
        let occurrences = extract_from_body("Aria vit à [[Cité de Verre]] avec [[Kael]].");
        assert_eq!(targets(&occurrences), ["Cité de Verre", "Kael"]);
        assert!(occurrences.iter().all(|o| o.context == LinkContext::Body));
        assert!(occurrences.iter().all(|o| !o.is_embed));
    }

    #[test]
    fn extracts_display_text_and_anchor() {
        let occurrences = extract_from_body("[[Aria|la capitaine]] et [[Aria#Jeunesse]]");
        assert_eq!(occurrences[0].target_raw, "Aria");
        assert_eq!(occurrences[0].display.as_deref(), Some("la capitaine"));
        assert_eq!(occurrences[1].target_raw, "Aria");
        assert_eq!(occurrences[1].anchor.as_deref(), Some("Jeunesse"));
    }

    #[test]
    fn hash_after_pipe_belongs_to_display_text() {
        let occurrences = extract_from_body("[[Aria|la #1]]");
        assert_eq!(occurrences[0].target_raw, "Aria");
        assert_eq!(occurrences[0].display.as_deref(), Some("la #1"));
        assert_eq!(occurrences[0].anchor, None);
    }

    #[test]
    fn trims_whitespace_around_target_and_display() {
        let occurrences = extract_from_body("[[  Aria  |  la capitaine  ]]");
        assert_eq!(occurrences[0].target_raw, "Aria");
        assert_eq!(occurrences[0].display.as_deref(), Some("la capitaine"));
    }

    #[test]
    fn marks_embeds() {
        let occurrences = extract_from_body("![[carte-du-monde.png]] puis [[Aria]]");
        assert!(occurrences[0].is_embed);
        assert!(!occurrences[1].is_embed);
    }

    #[test]
    fn offsets_point_at_the_opening_bracket() {
        let body = "abc [[Aria]] def ![[map.png]]";
        let occurrences = extract_from_body(body);
        assert_eq!(&body[occurrences[0].offset.unwrap()..][..2], "[[");
        assert_eq!(&body[occurrences[1].offset.unwrap()..][..3], "![[");
    }

    #[test]
    fn ignores_empty_and_unterminated_links() {
        assert!(extract_from_body("[[]] et [[   ]]").is_empty());
        assert!(extract_from_body("[[Aria").is_empty());
        assert_eq!(targets(&extract_from_body("[[Aria]] [[Kael")), ["Aria"]);
    }

    #[test]
    fn single_brackets_are_not_wikilinks() {
        assert!(extract_from_body("[Aria](http://x) et [Kael]").is_empty());
    }

    #[test]
    fn extracts_frontmatter_links() {
        let frontmatter: Frontmatter = serde_json::from_str(
            r#"{"pov":"[[Aria]]","locations":["[[Cité de Verre]]","[[Port Franc]]"]}"#,
        )
        .unwrap();
        let occurrences = extract_from_frontmatter(&frontmatter);
        assert_eq!(
            targets(&occurrences),
            ["Aria", "Cité de Verre", "Port Franc"]
        );
        assert_eq!(
            occurrences[1].context,
            LinkContext::Field("locations".into())
        );
        assert_eq!(occurrences[1].context.as_stored(), "field:locations");
        assert_eq!(occurrences[1].context.field(), Some("locations"));
    }

    #[test]
    fn frontmatter_ignores_values_that_are_not_whole_wikilinks() {
        let frontmatter: Frontmatter = serde_json::from_str(
            r#"{"leader":"Aria","empty":"","prose":"voir [[Aria]] et [[Kael]]","n":3}"#,
        )
        .unwrap();
        assert!(extract_from_frontmatter(&frontmatter).is_empty());
    }

    #[test]
    fn frontmatter_walks_nested_mappings_with_a_dotted_path() {
        let frontmatter: Frontmatter =
            serde_json::from_str(r#"{"custom":{"mentor":"[[Orlan]]"}}"#).unwrap();
        let occurrences = extract_from_frontmatter(&frontmatter);
        assert_eq!(
            occurrences[0].context,
            LinkContext::Field("custom.mentor".into())
        );
    }

    #[test]
    fn frontmatter_links_have_no_offset() {
        let frontmatter: Frontmatter = serde_json::from_str(r#"{"pov":"[[Aria]]"}"#).unwrap();
        assert_eq!(extract_from_frontmatter(&frontmatter)[0].offset, None);
    }

    #[test]
    fn link_context_round_trips() {
        for context in [LinkContext::Body, LinkContext::Field("pov".into())] {
            assert_eq!(LinkContext::from_stored(&context.as_stored()), context);
        }
    }

    fn resolver() -> Resolver {
        let mut resolver = Resolver::default();
        resolver.insert("aria-solane", "Aria Solane", &["La Verrière".into()]);
        resolver.insert("kael-vantre", "Kael Vantre", &[]);
        resolver.insert("cite-de-verre", "Cité de Verre", &[]);
        resolver
    }

    #[test]
    fn resolves_by_slug_title_and_alias() {
        let resolver = resolver();
        for target in ["aria-solane", "Aria Solane", "aria solane", "La Verrière"] {
            let resolved = resolver.resolve(target);
            assert_eq!(resolved.resolution, Resolution::Resolved, "{target}");
            assert_eq!(resolved.slug.as_deref(), Some("aria-solane"), "{target}");
        }
    }

    #[test]
    fn resolution_is_case_and_accent_insensitive() {
        let resolved = resolver().resolve("  cite   DE verre ");
        assert_eq!(resolved.slug.as_deref(), Some("cite-de-verre"));
    }

    #[test]
    fn unknown_target_is_a_stub() {
        let resolved = resolver().resolve("Maître Orlan");
        assert_eq!(resolved.resolution, Resolution::Stub);
        assert!(resolved.slug.is_none());
    }

    #[test]
    fn filename_beats_a_homonymous_title() {
        let mut resolver = Resolver::default();
        resolver.insert("aria", "Personne Autre", &[]);
        resolver.insert("aria-solane", "Aria", &[]);
        let resolved = resolver.resolve("Aria");
        assert_eq!(resolved.resolution, Resolution::Resolved);
        assert_eq!(resolved.slug.as_deref(), Some("aria"));
    }

    #[test]
    fn title_beats_an_alias() {
        let mut resolver = Resolver::default();
        resolver.insert("aria-sombre", "Autre", &["Aria".into()]);
        resolver.insert("aria-claire", "Aria", &[]);
        assert_eq!(
            resolver.resolve("Aria").slug.as_deref(),
            Some("aria-claire")
        );
    }

    #[test]
    fn two_identical_titles_are_ambiguous() {
        let mut resolver = Resolver::default();
        resolver.insert("aria-claire", "Aria", &[]);
        resolver.insert("aria-sombre", "Aria", &[]);
        let resolved = resolver.resolve("Aria");
        assert_eq!(resolved.resolution, Resolution::Ambiguous);
        assert!(resolved.slug.is_none());
        assert_eq!(resolved.candidates, ["aria-claire", "aria-sombre"]);
    }

    #[test]
    fn an_alias_equal_to_its_own_title_is_not_ambiguous() {
        let mut resolver = Resolver::default();
        resolver.insert("aria", "Aria", &["Aria".into(), "aria".into()]);
        assert_eq!(resolver.resolve("Aria").resolution, Resolution::Resolved);
    }

    #[test]
    fn body_excerpt_returns_the_trimmed_line() {
        let body = "premiere ligne\n  Aria vit à [[Cité de Verre]].  \nautre\n";
        let offset = body.find("[[").unwrap();
        assert_eq!(
            body_excerpt(body, offset, 200),
            "Aria vit à [[Cité de Verre]]."
        );
    }

    #[test]
    fn body_excerpt_caps_long_lines_on_char_boundaries() {
        let body = "éééééééé [[Aria]]";
        let excerpt = body_excerpt(body, body.find("[[").unwrap(), 4);
        assert_eq!(excerpt, "éééé…");
    }

    #[test]
    fn body_excerpt_handles_a_single_line_body() {
        let body = "[[Aria]] seule";
        assert_eq!(body_excerpt(body, 0, 200), "[[Aria]] seule");
    }
}
