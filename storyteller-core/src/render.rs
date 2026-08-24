//! Markdown → HTML rendering (`docs/api.md` §2, `?render=html`; [ADR
//! 0015](../../docs/adr/0015-markdown-rendering-in-core.md)).
//!
//! [Wikilinks](../../docs/linking.md) and asset embeds (`[[Target]]`,
//! `![[file]]`) are not CommonMark syntax, so they are spliced into literal
//! HTML fragments at their exact byte spans *before* handing the text to
//! `pulldown-cmark` — CommonMark passes inline/block raw HTML straight
//! through, so the parser never needs to know wikilinks exist. This mirrors
//! [`crate::links::rewrite_body_links`]'s shape (find spans via
//! [`extract_from_body`], edit back-to-front so earlier offsets stay valid).
//!
//! Deliberately out of scope (see ADR 0015): the plain `![](assets/...)`
//! markdown image form is left exactly as `pulldown-cmark` renders it — only
//! the `![[file]]` wikilink-style embed gets resolved against `assets/`.

use pulldown_cmark::{html, Options, Parser};

use crate::links::{extract_from_body, LinkOccurrence, Resolution};

/// What a wikilink target resolves to — a narrower view than
/// [`crate::links::OutgoingLink`], independent of how it was computed (an
/// index-backed lookup in the server, a fresh `Resolver` in tests).
#[derive(Debug, Clone)]
pub struct LinkTarget {
    pub resolution: Resolution,
    pub slug: Option<String>,
}

/// Renders a body to HTML.
///
/// `resolve` maps a wikilink's raw target text to its resolution; `asset_path`
/// maps an embed's raw target text (a bare filename, per `docs/linking.md`
/// "Images and assets") to the matching asset's project-relative path, when
/// one exists.
pub fn render_body(
    body: &str,
    resolve: &dyn Fn(&str) -> LinkTarget,
    asset_path: &dyn Fn(&str) -> Option<String>,
) -> String {
    let spliced = splice_wikilinks(body, resolve, asset_path);
    let parser = Parser::new_ext(
        &spliced,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS,
    );
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

fn splice_wikilinks(
    body: &str,
    resolve: &dyn Fn(&str) -> LinkTarget,
    asset_path: &dyn Fn(&str) -> Option<String>,
) -> String {
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    for occurrence in extract_from_body(body) {
        let Some(start) = occurrence.offset else {
            continue;
        };
        let Some(rel_end) = body[start..].find("]]") else {
            continue;
        };
        let end = start + rel_end + 2;
        let display = occurrence
            .display
            .clone()
            .unwrap_or_else(|| occurrence.target_raw.clone());
        let fragment = if occurrence.is_embed {
            render_embed(&occurrence.target_raw, &display, asset_path)
        } else {
            render_link(&occurrence, &display, resolve)
        };
        edits.push((start, end, fragment));
    }

    let mut out = body.to_string();
    for (start, end, replacement) in edits.into_iter().rev() {
        out.replace_range(start..end, &replacement);
    }
    out
}

/// `<a>`/`<span>` for a wikilink, classed by resolution — same vocabulary the
/// frontend already uses for the stopgap raw-text highlighter
/// (`BodySection.svelte`), so it drops in as a replacement.
///
/// The `/entry/{slug}` href bakes in the frontend's own route: accepted by
/// ADR 0015 since there is one target-independent SvelteKit frontend.
fn render_link(occurrence: &LinkOccurrence, display: &str, resolve: &dyn Fn(&str) -> LinkTarget) -> String {
    let target = resolve(&occurrence.target_raw);
    let display = escape_html(display);
    match target.resolution {
        Resolution::Resolved => {
            let slug = target.slug.unwrap_or_default();
            format!(
                r#"<a class="wikilink resolved" href="/entry/{}">{display}</a>"#,
                escape_attr(&slug)
            )
        }
        Resolution::Stub => format!(
            r#"<span class="wikilink stub" data-target="{}">{display}</span>"#,
            escape_attr(&occurrence.target_raw)
        ),
        Resolution::Ambiguous => format!(
            r#"<span class="wikilink ambiguous" data-target="{}">{display}</span>"#,
            escape_attr(&occurrence.target_raw)
        ),
    }
}

/// `<img>` for a resolved embed, or a visible "not found" block — never a
/// broken `<img>` tag (`docs/ui/components.md` §3, "embed d'asset non trouvé").
fn render_embed(target_raw: &str, display: &str, asset_path: &dyn Fn(&str) -> Option<String>) -> String {
    match asset_path(target_raw) {
        Some(path) => format!(
            r#"<img class="asset-embed" src="/api/v1/assets/{}" alt="{}">"#,
            escape_attr(&path),
            escape_attr(display)
        ),
        None => format!(
            r#"<div class="asset-embed missing">Embed <code>![[{}]]</code> — asset not found</div>"#,
            escape_html(target_raw)
        ),
    }
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn escape_attr(text: &str) -> String {
    escape_html(text).replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolved(slug: &str) -> LinkTarget {
        LinkTarget {
            resolution: Resolution::Resolved,
            slug: Some(slug.to_string()),
        }
    }

    #[test]
    fn renders_plain_markdown_untouched_by_wikilinks() {
        let html = render_body("# Title\n\nSome *emphasis* and **bold**.", &|_| stub(), &|_| None);
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<em>emphasis</em>"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    fn stub() -> LinkTarget {
        LinkTarget {
            resolution: Resolution::Stub,
            slug: None,
        }
    }

    #[test]
    fn resolved_wikilink_becomes_a_link_to_the_slug() {
        let html = render_body("Voir [[Aria Solane]].", &|t| {
            assert_eq!(t, "Aria Solane");
            resolved("aria-solane")
        }, &|_| None);
        assert_eq!(
            html.trim(),
            r#"<p>Voir <a class="wikilink resolved" href="/entry/aria-solane">Aria Solane</a>.</p>"#
        );
    }

    #[test]
    fn a_piped_display_is_kept_as_the_link_text() {
        let html = render_body("[[aria-solane|la capitaine]]", &|_| resolved("aria-solane"), &|_| None);
        assert!(html.contains(">la capitaine</a>"));
    }

    #[test]
    fn a_stub_becomes_a_span_carrying_the_raw_target() {
        let html = render_body("[[Maître Orlan]]", &|_| stub(), &|_| None);
        assert_eq!(
            html.trim(),
            r#"<p><span class="wikilink stub" data-target="Maître Orlan">Maître Orlan</span></p>"#
        );
    }

    #[test]
    fn an_ambiguous_target_becomes_a_span_too() {
        let html = render_body(
            "[[Solane]]",
            &|_| LinkTarget {
                resolution: Resolution::Ambiguous,
                slug: None,
            },
            &|_| None,
        );
        assert!(html.contains(r#"class="wikilink ambiguous""#));
    }

    #[test]
    fn a_resolved_embed_becomes_an_img_tag() {
        let html = render_body("![[aria.jpg]]", &|_| stub(), &|target| {
            assert_eq!(target, "aria.jpg");
            Some("assets/images/aria.jpg".to_string())
        });
        // A raw HTML tag alone on its own line is a CommonMark HTML block
        // (spec type 7): no surrounding `<p>`, unlike an inline `<span>`.
        assert_eq!(
            html.trim(),
            r#"<img class="asset-embed" src="/api/v1/assets/assets/images/aria.jpg" alt="aria.jpg">"#
        );
    }

    #[test]
    fn a_missing_embed_is_a_visible_block_not_a_broken_img() {
        let html = render_body("![[nope.jpg]]", &|_| stub(), &|_| None);
        assert!(html.contains("asset-embed missing"));
        assert!(!html.contains("<img"));
    }

    #[test]
    fn html_in_the_display_text_is_escaped() {
        let html = render_body("[[<script>alert(1)</script>]]", &|_| stub(), &|_| None);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn plain_markdown_image_is_left_to_pulldown_cmark_as_is() {
        let html = render_body("![alt](assets/maps/city.png)", &|_| stub(), &|_| None);
        assert!(html.contains(r#"<img src="assets/maps/city.png" alt="alt""#));
    }

    #[test]
    fn multiple_occurrences_do_not_corrupt_each_others_offsets() {
        let html = render_body(
            "[[Aria]] rencontre [[Kael]] près de [[Cité de Verre]].",
            &|t| resolved(&t.to_lowercase().replace(' ', "-")),
            &|_| None,
        );
        assert!(html.contains(r#"href="/entry/aria">Aria</a>"#));
        assert!(html.contains(r#"href="/entry/kael">Kael</a>"#));
        assert!(html.contains(r#"href="/entry/cité-de-verre">Cité de Verre</a>"#));
    }
}
