//! EPUB 3 navigation document (`nav.xhtml`).
//!
//! Lists user-content pages — front matter (those with a title), all
//! chapters, and back matter (those with a title). Auto-generated cover,
//! title, and copyright pages are not shown in the visible TOC.

use std::fmt::Write as _;

use papyrust_core::ir::Book;

use crate::{escape, pages, paths};

#[must_use]
pub fn build(book: &Book) -> String {
    let mut body = String::from("<nav epub:type=\"toc\" id=\"toc\">\n<h1>Contents</h1>\n<ol>\n");

    for (i, page) in book.front_matter.iter().enumerate() {
        if let Some(title) = &page.title {
            let _ = writeln!(
                body,
                "<li><a href=\"{}\">{}</a></li>",
                paths::front_matter(i),
                escape::text(title)
            );
        }
    }
    for (i, ch) in book.chapters.iter().enumerate() {
        let _ = writeln!(
            body,
            "<li><a href=\"{}\">{}</a></li>",
            paths::chapter(i),
            escape::text(&ch.title)
        );
    }
    for (i, page) in book.back_matter.iter().enumerate() {
        if let Some(title) = &page.title {
            let _ = writeln!(
                body,
                "<li><a href=\"{}\">{}</a></li>",
                paths::back_matter(i),
                escape::text(title)
            );
        }
    }

    body.push_str("</ol>\n</nav>\n");
    pages::document("Contents", None, &body, &book.meta.language)
}

#[cfg(test)]
mod tests {
    use super::*;

    use papyrust_core::config::TrimSize;
    use papyrust_core::ir::{Book, BookMeta, Chapter, Copyright, MatterPage};

    fn book_with(chapters: Vec<&str>, front: Vec<Option<&str>>, back: Vec<Option<&str>>) -> Book {
        Book {
            meta: BookMeta {
                title: "T".into(),
                subtitle: None,
                author: "A".into(),
                language: "en-US".into(),
                copyright: Copyright {
                    year: 2026,
                    holder: "A".into(),
                    isbn_epub: None,
                    isbn_print: None,
                    publisher: "P".into(),
                },
                trim: TrimSize::SixByNine,
            },
            cover: None,
            front_matter: front
                .into_iter()
                .map(|t| MatterPage {
                    title: t.map(String::from),
                    blocks: vec![],
                })
                .collect(),
            chapters: chapters
                .into_iter()
                .map(|t| Chapter {
                    title: t.into(),
                    blocks: vec![],
                })
                .collect(),
            back_matter: back
                .into_iter()
                .map(|t| MatterPage {
                    title: t.map(String::from),
                    blocks: vec![],
                })
                .collect(),
        }
    }

    #[test]
    fn lists_all_chapters_in_order() {
        let book = book_with(vec!["One", "Two", "Three"], vec![], vec![]);
        let nav = build(&book);
        assert!(nav.contains(r#"<a href="chapter-001.xhtml">One</a>"#));
        assert!(nav.contains(r#"<a href="chapter-002.xhtml">Two</a>"#));
        assert!(nav.contains(r#"<a href="chapter-003.xhtml">Three</a>"#));
        let one_pos = nav.find("chapter-001").unwrap();
        let two_pos = nav.find("chapter-002").unwrap();
        assert!(one_pos < two_pos);
    }

    #[test]
    fn front_matter_with_title_is_included() {
        let book = book_with(vec!["One"], vec![Some("Dedication")], vec![]);
        let nav = build(&book);
        assert!(nav.contains(r#"<a href="front-001.xhtml">Dedication</a>"#));
    }

    #[test]
    fn front_matter_without_title_is_skipped() {
        let book = book_with(vec!["One"], vec![None], vec![]);
        let nav = build(&book);
        assert!(!nav.contains("front-001"));
    }

    #[test]
    fn back_matter_with_title_is_included() {
        let book = book_with(vec!["One"], vec![], vec![Some("About")]);
        let nav = build(&book);
        assert!(nav.contains(r#"<a href="back-001.xhtml">About</a>"#));
    }

    #[test]
    fn cover_title_copyright_never_appear() {
        let book = book_with(vec!["One"], vec![], vec![]);
        let nav = build(&book);
        assert!(!nav.contains("cover.xhtml"));
        assert!(!nav.contains("title.xhtml"));
        assert!(!nav.contains("copyright.xhtml"));
    }

    #[test]
    fn nav_is_well_formed_xhtml() {
        let book = book_with(vec!["One"], vec![], vec![]);
        let nav = build(&book);
        assert!(nav.contains("<?xml"));
        assert!(nav.contains(r#"<nav epub:type="toc""#));
        assert!(nav.contains("</nav>"));
    }

    #[test]
    fn chapter_titles_are_escaped() {
        let book = book_with(vec!["a & b"], vec![], vec![]);
        let nav = build(&book);
        assert!(nav.contains("a &amp; b"));
    }
}
