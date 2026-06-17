//! EPUB 3 OPF package document (`content.opf`).
//!
//! Holds the Dublin Core metadata, the manifest (every file in the
//! archive with media type and properties), and the spine (reading order).

use std::fmt::Write as _;

use crate::ir::{Book, BookMeta};
use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;
use uuid::Uuid;

use super::{EpubError, escape, paths};

const DCTERMS_FMT: &[FormatItem<'_>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");

const XHTML_MIME: &str = "application/xhtml+xml";

pub fn build(book: &Book) -> Result<String, EpubError> {
    let modified = OffsetDateTime::now_utc().format(DCTERMS_FMT)?;
    let lang = escape::attribute(&book.meta.language);

    let mut out = String::new();
    let _ = writeln!(out, r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    let _ = writeln!(
        out,
        r#"<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="pub-id" xml:lang="{lang}">"#
    );
    write_metadata(&mut out, book, &lang, &modified);
    write_manifest(&mut out, book);
    write_spine(&mut out, book);
    out.push_str("</package>\n");
    Ok(out)
}

fn write_metadata(out: &mut String, book: &Book, lang: &str, modified: &str) {
    let identifier = book_identifier(&book.meta);
    out.push_str(r#"  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">"#);
    out.push('\n');
    let _ = writeln!(
        out,
        r#"    <dc:identifier id="pub-id">{}</dc:identifier>"#,
        escape::text(&identifier)
    );
    let _ = writeln!(
        out,
        "    <dc:title>{}</dc:title>",
        escape::text(&book.meta.title)
    );
    let _ = writeln!(
        out,
        "    <dc:creator>{}</dc:creator>",
        escape::text(&book.meta.author)
    );
    let _ = writeln!(out, "    <dc:language>{lang}</dc:language>");
    let _ = writeln!(out, "    <dc:date>{}</dc:date>", book.meta.copyright.year);
    let _ = writeln!(
        out,
        "    <dc:publisher>{}</dc:publisher>",
        escape::text(&book.meta.copyright.publisher)
    );
    let _ = writeln!(
        out,
        "    <dc:rights>Copyright \u{00A9} {} {}</dc:rights>",
        book.meta.copyright.year,
        escape::text(&book.meta.copyright.holder)
    );
    let _ = writeln!(
        out,
        r#"    <meta property="dcterms:modified">{modified}</meta>"#
    );
    if book.cover.is_some() {
        out.push_str("    <meta name=\"cover\" content=\"cover-image\"/>\n");
    }
    out.push_str("  </metadata>\n");
}

fn write_manifest(out: &mut String, book: &Book) {
    out.push_str("  <manifest>\n");
    if let Some(cover) = &book.cover {
        let _ = writeln!(
            out,
            r#"    <item id="cover-image" href="{}" media-type="{}" properties="cover-image"/>"#,
            paths::COVER_IMAGE,
            cover.mime
        );
    }
    let _ = writeln!(
        out,
        r#"    <item id="theme-css" href="{}" media-type="text/css"/>"#,
        paths::STYLESHEET
    );
    let _ = writeln!(
        out,
        r#"    <item id="nav" href="{}" media-type="{XHTML_MIME}" properties="nav"/>"#,
        paths::NAV_XHTML
    );
    if book.cover.is_some() {
        let _ = writeln!(
            out,
            r#"    <item id="cover" href="{}" media-type="{XHTML_MIME}"/>"#,
            paths::COVER_XHTML
        );
    }
    let _ = writeln!(
        out,
        r#"    <item id="title" href="{}" media-type="{XHTML_MIME}"/>"#,
        paths::TITLE_XHTML
    );
    let _ = writeln!(
        out,
        r#"    <item id="copyright" href="{}" media-type="{XHTML_MIME}"/>"#,
        paths::COPYRIGHT_XHTML
    );
    for i in 0..book.front_matter.len() {
        let _ = writeln!(
            out,
            r#"    <item id="front-{:03}" href="{}" media-type="{XHTML_MIME}"/>"#,
            i + 1,
            paths::front_matter(i)
        );
    }
    for i in 0..book.chapters.len() {
        let _ = writeln!(
            out,
            r#"    <item id="chapter-{:03}" href="{}" media-type="{XHTML_MIME}"/>"#,
            i + 1,
            paths::chapter(i)
        );
    }
    for i in 0..book.back_matter.len() {
        let _ = writeln!(
            out,
            r#"    <item id="back-{:03}" href="{}" media-type="{XHTML_MIME}"/>"#,
            i + 1,
            paths::back_matter(i)
        );
    }
    out.push_str("  </manifest>\n");
}

fn write_spine(out: &mut String, book: &Book) {
    out.push_str("  <spine>\n");
    if book.cover.is_some() {
        out.push_str("    <itemref idref=\"cover\"/>\n");
    }
    out.push_str("    <itemref idref=\"title\"/>\n");
    out.push_str("    <itemref idref=\"copyright\"/>\n");
    out.push_str("    <itemref idref=\"nav\"/>\n");
    for i in 0..book.front_matter.len() {
        let _ = writeln!(out, "    <itemref idref=\"front-{:03}\"/>", i + 1);
    }
    for i in 0..book.chapters.len() {
        let _ = writeln!(out, "    <itemref idref=\"chapter-{:03}\"/>", i + 1);
    }
    for i in 0..book.back_matter.len() {
        let _ = writeln!(out, "    <itemref idref=\"back-{:03}\"/>", i + 1);
    }
    out.push_str("  </spine>\n");
}

fn book_identifier(meta: &BookMeta) -> String {
    if let Some(isbn) = &meta.copyright.isbn_epub {
        return format!("urn:isbn:{isbn}");
    }
    let seed = format!("{}|{}|{}", meta.title, meta.author, meta.copyright.year);
    let uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, seed.as_bytes());
    format!("urn:uuid:{uuid}")
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::TrimSize;
    use crate::ir::{Chapter, Copyright, Cover, MatterPage};

    fn book(with_cover: bool, isbn: Option<&str>) -> Book {
        Book {
            meta: BookMeta {
                title: "Test & Co".into(),
                subtitle: None,
                author: "A".into(),
                language: "en-US".into(),
                copyright: Copyright {
                    year: 2026,
                    holder: "A".into(),
                    isbn_epub: isbn.map(String::from),
                    isbn_print: None,
                    publisher: "P".into(),
                },
                trim: TrimSize::SixByNine,
            },
            cover: with_cover.then(|| Cover {
                bytes: vec![0xff, 0xd8, 0xff],
                mime: "image/jpeg",
            }),
            front_matter: vec![MatterPage {
                title: Some("F".into()),
                blocks: vec![],
            }],
            chapters: vec![
                Chapter {
                    title: "One".into(),
                    blocks: vec![],
                },
                Chapter {
                    title: "Two".into(),
                    blocks: vec![],
                },
            ],
            back_matter: vec![MatterPage {
                title: Some("B".into()),
                blocks: vec![],
            }],
        }
    }

    #[test]
    fn includes_required_metadata() {
        let opf = build(&book(true, None)).unwrap();
        assert!(opf.contains("<dc:identifier"));
        assert!(opf.contains("<dc:title>Test &amp; Co</dc:title>"));
        assert!(opf.contains("<dc:language>en-US</dc:language>"));
        assert!(opf.contains("<dc:date>2026</dc:date>"));
        assert!(opf.contains(r#"property="dcterms:modified""#));
    }

    #[test]
    fn uses_isbn_when_provided() {
        let opf = build(&book(false, Some("978-0-000-00000-0"))).unwrap();
        assert!(opf.contains("urn:isbn:978-0-000-00000-0"));
    }

    #[test]
    fn falls_back_to_uuid_when_no_isbn() {
        let opf = build(&book(false, None)).unwrap();
        assert!(opf.contains("urn:uuid:"));
    }

    #[test]
    fn uuid_is_deterministic_for_same_metadata() {
        let a = build(&book(false, None)).unwrap();
        let b = build(&book(false, None)).unwrap();
        let extract_id = |s: &str| {
            let start = s.find("urn:uuid:").unwrap();
            let end = s[start..].find('<').unwrap() + start;
            s[start..end].to_string()
        };
        assert_eq!(extract_id(&a), extract_id(&b));
    }

    #[test]
    fn manifest_lists_all_pieces() {
        let opf = build(&book(true, None)).unwrap();
        assert!(opf.contains(r#"id="cover-image""#));
        assert!(opf.contains(r#"id="theme-css""#));
        assert!(opf.contains(r#"id="nav""#));
        assert!(opf.contains(r#"id="cover""#));
        assert!(opf.contains(r#"id="title""#));
        assert!(opf.contains(r#"id="copyright""#));
        assert!(opf.contains(r#"id="front-001""#));
        assert!(opf.contains(r#"id="chapter-001""#));
        assert!(opf.contains(r#"id="chapter-002""#));
        assert!(opf.contains(r#"id="back-001""#));
    }

    #[test]
    fn cover_omitted_when_no_cover() {
        let opf = build(&book(false, None)).unwrap();
        assert!(!opf.contains("cover-image"));
        assert!(!opf.contains(r#"id="cover""#));
        assert!(!opf.contains(r#"name="cover""#));
    }

    #[test]
    fn spine_reading_order_is_cover_title_copyright_nav_front_chapters_back() {
        let opf = build(&book(true, None)).unwrap();
        let spine = &opf[opf.find("<spine>").unwrap()..opf.find("</spine>").unwrap()];
        let positions: Vec<usize> = [
            "cover",
            "title",
            "copyright",
            "nav",
            "front-001",
            "chapter-001",
            "back-001",
        ]
        .iter()
        .map(|id| spine.find(&format!(r#"idref="{id}""#)).unwrap())
        .collect();
        assert!(positions.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn dcterms_modified_format_matches_spec() {
        let opf = build(&book(false, None)).unwrap();
        let start = opf.find(r#"dcterms:modified">"#).unwrap() + 18;
        let end = start + opf[start..].find('<').unwrap();
        let stamp = &opf[start..end];
        assert_eq!(
            stamp.len(),
            20,
            "expected YYYY-MM-DDThh:mm:ssZ, got {stamp}"
        );
        assert!(stamp.ends_with('Z'));
        assert!(!stamp.contains('.'));
    }
}
