//! XML escape helpers for XHTML output.
//!
//! EPUB content documents are XHTML — strict, well-formed XML. Any user
//! text that flows into element content or attribute values must be
//! escaped to avoid producing broken (or unsafe) XML.

/// Escape characters that have a special meaning in XML element content.
#[must_use]
pub fn text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            _ => out.push(c),
        }
    }
    out
}

/// Escape characters for use inside a double-quoted attribute value.
#[must_use]
pub fn attribute(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_escapes_lt_gt_amp() {
        assert_eq!(text("a < b & c > d"), "a &lt; b &amp; c &gt; d");
    }

    #[test]
    fn text_leaves_quotes_alone() {
        assert_eq!(text(r#"she said "hi" 'world'"#), r#"she said "hi" 'world'"#);
    }

    #[test]
    fn attribute_escapes_double_quotes() {
        assert_eq!(attribute(r#"he said "yes""#), "he said &quot;yes&quot;");
    }

    #[test]
    fn round_trip_safe_unicode() {
        assert_eq!(text("café — résumé"), "café — résumé");
    }
}
