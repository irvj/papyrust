//! Auto-generated front-matter pages and XHTML document boilerplate.
//!
//! The cover page, title page, and copyright page are generated from
//! `book.toml` so the user never has to write them by hand. They are
//! inserted at the front of the book before any user-written
//! front-matter pages.

use std::fmt::Write as _;

use crate::ir::BookMeta;

use super::escape;

/// Wrap a body fragment in the full EPUB 3 XHTML document boilerplate.
pub fn document(title: &str, epub_type: Option<&str>, body: &str, lang: &str) -> String {
    let title_e = escape::text(title);
    let lang_e = escape::attribute(lang);
    let type_attr = match epub_type {
        Some(t) => format!(r#" epub:type="{}""#, escape::attribute(t)),
        None => String::new(),
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="{lang_e}" xml:lang="{lang_e}">
<head>
<meta charset="utf-8"/>
<title>{title_e}</title>
<link rel="stylesheet" type="text/css" href="theme.css"/>
</head>
<body{type_attr}>
{body}</body>
</html>
"#
    )
}

pub fn cover_page(meta: &BookMeta) -> String {
    let body = "<div class=\"cover\"><img src=\"cover.jpg\" alt=\"Cover\"/></div>\n";
    document("Cover", Some("cover"), body, &meta.language)
}

pub fn title_page(meta: &BookMeta) -> String {
    let mut body = String::from("<div class=\"title-page\">\n");
    let _ = writeln!(
        body,
        "<h1 class=\"book-title\">{}</h1>",
        escape::text(&meta.title)
    );
    if let Some(subtitle) = &meta.subtitle {
        let _ = writeln!(
            body,
            "<p class=\"book-subtitle\">{}</p>",
            escape::text(subtitle)
        );
    }
    let _ = writeln!(
        body,
        "<p class=\"book-author\">{}</p>",
        escape::text(&meta.author)
    );
    body.push_str("</div>\n");
    document(&meta.title, Some("titlepage"), &body, &meta.language)
}

pub fn copyright_page(meta: &BookMeta) -> String {
    let cp = &meta.copyright;
    let mut body = String::from("<div class=\"copyright\">\n");
    let _ = writeln!(
        body,
        "<p>Copyright \u{00A9} {} {}</p>",
        cp.year,
        escape::text(&cp.holder)
    );
    body.push_str("<p>All rights reserved.</p>\n");
    let _ = writeln!(body, "<p>Published by {}</p>", escape::text(&cp.publisher));
    if let Some(isbn) = &cp.isbn_epub {
        let _ = writeln!(body, "<p>ISBN: {}</p>", escape::text(isbn));
    }
    body.push_str("</div>\n");
    document("Copyright", Some("copyright-page"), &body, &meta.language)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::TrimSize;
    use crate::ir::Copyright;

    fn meta() -> BookMeta {
        BookMeta {
            title: "The Long Road".into(),
            subtitle: Some("A Novel".into()),
            author: "Jane Doe".into(),
            language: "en-US".into(),
            copyright: Copyright {
                year: 2026,
                holder: "Jane Doe".into(),
                isbn_epub: Some("978-0-000-00000-0".into()),
                isbn_print: None,
                publisher: "Self-Published".into(),
            },
            trim: TrimSize::SixByNine,
        }
    }

    #[test]
    fn document_well_formed_shell() {
        let doc = document("Hi", Some("chapter"), "<p>body</p>\n", "en-US");
        assert!(doc.starts_with("<?xml"));
        assert!(doc.contains("<!DOCTYPE html>"));
        assert!(doc.contains(r#"lang="en-US""#));
        assert!(doc.contains(r#"epub:type="chapter""#));
        assert!(doc.contains("<title>Hi</title>"));
        assert!(doc.contains("<p>body</p>"));
    }

    #[test]
    fn document_escapes_title_and_lang() {
        let doc = document("a & b", None, "", r#""en""#);
        assert!(doc.contains("a &amp; b"));
        assert!(doc.contains(r#"lang="&quot;en&quot;""#));
    }

    #[test]
    fn cover_page_references_cover_jpg() {
        let html = cover_page(&meta());
        assert!(html.contains(r#"img src="cover.jpg""#));
        assert!(html.contains(r#"epub:type="cover""#));
    }

    #[test]
    fn title_page_contains_title_subtitle_author() {
        let html = title_page(&meta());
        assert!(html.contains("The Long Road"));
        assert!(html.contains("A Novel"));
        assert!(html.contains("Jane Doe"));
        assert!(html.contains(r#"epub:type="titlepage""#));
    }

    #[test]
    fn title_page_omits_subtitle_when_none() {
        let mut m = meta();
        m.subtitle = None;
        let html = title_page(&m);
        assert!(!html.contains("book-subtitle"));
    }

    #[test]
    fn copyright_page_includes_year_holder_publisher_isbn() {
        let html = copyright_page(&meta());
        assert!(html.contains("2026"));
        assert!(html.contains("Jane Doe"));
        assert!(html.contains("Self-Published"));
        assert!(html.contains("978-0-000-00000-0"));
        assert!(html.contains("\u{00A9}")); // copyright symbol
        assert!(html.contains(r#"epub:type="copyright-page""#));
    }

    #[test]
    fn copyright_page_omits_isbn_when_none() {
        let mut m = meta();
        m.copyright.isbn_epub = None;
        let html = copyright_page(&m);
        assert!(!html.contains("ISBN:"));
    }

    #[test]
    fn title_page_escapes_special_chars() {
        let mut m = meta();
        m.title = "A & B".into();
        let html = title_page(&m);
        assert!(html.contains("A &amp; B"));
    }
}
