//! Match-key normalization for link resolution (`docs/linking.md` §3.1).
//!
//! Normalization exists **only** for comparison. Original texts (filename,
//! `title`, `aliases`, raw link target) are always preserved as written.

use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

/// Computes the normalized match key of a textual target.
///
/// Steps, in the order fixed by the spec:
/// 1. Unicode NFC;
/// 2. trim + collapse runs of internal whitespace into a single space;
/// 3. case folding (lowercase);
/// 4. diacritic folding (`é` → `e`, `ç` → `c`).
///
/// ```
/// use storyteller_core::normalize::normalize;
/// assert_eq!(normalize("  Cité   de   Verre "), normalize("cite de verre"));
/// ```
pub fn normalize(s: &str) -> String {
    // 1. NFC, then 2. whitespace collapsing on the composed form.
    let composed: String = s.nfc().collect();
    let collapsed = collapse_whitespace(&composed);

    // 3. case folding, then 4. diacritic folding. Folding needs the decomposed
    //    form so that combining marks can be dropped; we recompose after.
    collapsed
        .to_lowercase()
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .nfc()
        .collect()
}

fn collapse_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut pending_space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            // Never emit leading whitespace; defer the rest until a real char
            // shows up, which also drops trailing whitespace for free.
            pending_space = !out.is_empty();
            continue;
        }
        if pending_space {
            out.push(' ');
            pending_space = false;
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn folds_case_and_accents() {
        assert_eq!(normalize("Élodie"), "elodie");
        assert_eq!(normalize("ÇA"), "ca");
        assert_eq!(normalize("Cité de Verre"), "cite de verre");
    }

    #[test]
    fn collapses_and_trims_whitespace() {
        assert_eq!(normalize("  Glass  City "), "glass city");
        assert_eq!(normalize("Glass\tCity"), "glass city");
        assert_eq!(normalize("Glass\n City"), "glass city");
        assert_eq!(normalize("   "), "");
    }

    #[test]
    fn precomposed_and_decomposed_forms_match() {
        // "é" as U+00E9 vs "e" + U+0301.
        assert_eq!(normalize("\u{00e9}pe\u{0301}e"), normalize("e\u{0301}pee"));
    }

    #[test]
    fn keeps_non_latin_scripts_usable() {
        // No transliteration: we fold marks, we do not romanize.
        assert_eq!(normalize("Аria"), normalize("аria"));
    }

    #[test]
    fn is_idempotent() {
        for s in ["Élodie", "  Glass  City ", "ÇA VA"] {
            let once = normalize(s);
            assert_eq!(normalize(&once), once);
        }
    }
}
