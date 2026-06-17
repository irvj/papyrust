//! Render Book IR blocks/inlines to XHTML body content.
//!
//! Output is a single XHTML body fragment as a string; the caller wraps
//! it in the `<html>` / `<body>` boilerplate appropriate for the page
//! kind (chapter, matter page, auto-generated page).

use crate::ir::{Block, HeadingLevel, Inline, ListItem};

use super::escape;

/// Centered ornament rendered for `Block::SceneBreak`. Three asterisks
/// is the trade-press fiction convention and renders reliably in any
/// font (no dependency on dingbat coverage).
const SCENE_ORNAMENT: &str = "* * *";

/// Render a sequence of blocks to XHTML.
///
/// When `chapter_body` is true, the very first paragraph encountered at
/// this top level gets `class="first-paragraph"` so a stylesheet can
/// apply a drop cap.
#[must_use]
pub fn render_blocks(blocks: &[Block], chapter_body: bool) -> String {
    let mut out = String::new();
    let mut first_paragraph_seen = false;
    for block in blocks {
        let is_first =
            chapter_body && !first_paragraph_seen && matches!(block, Block::Paragraph(_));
        render_block(&mut out, block, is_first);
        if matches!(block, Block::Paragraph(_)) {
            first_paragraph_seen = true;
        }
    }
    out
}

fn render_block(out: &mut String, block: &Block, first_paragraph: bool) {
    match block {
        Block::Paragraph(inlines) => {
            if first_paragraph {
                out.push_str("<p class=\"first-paragraph\">");
            } else {
                out.push_str("<p>");
            }
            render_inlines(out, inlines);
            out.push_str("</p>\n");
        }
        Block::Heading { level, content } => {
            let tag = match level {
                HeadingLevel::H2 => "h2",
                HeadingLevel::H3 => "h3",
                HeadingLevel::H4 => "h4",
            };
            out.push('<');
            out.push_str(tag);
            out.push('>');
            render_inlines(out, content);
            out.push_str("</");
            out.push_str(tag);
            out.push_str(">\n");
        }
        Block::SceneBreak => {
            out.push_str("<div class=\"scene-break\" role=\"separator\">");
            out.push_str(SCENE_ORNAMENT);
            out.push_str("</div>\n");
        }
        Block::BlockQuote(inner) => {
            out.push_str("<blockquote>\n");
            for b in inner {
                render_block(out, b, false);
            }
            out.push_str("</blockquote>\n");
        }
        Block::UnorderedList(items) => {
            out.push_str("<ul>\n");
            for item in items {
                render_list_item(out, item);
            }
            out.push_str("</ul>\n");
        }
        Block::OrderedList(items) => {
            out.push_str("<ol>\n");
            for item in items {
                render_list_item(out, item);
            }
            out.push_str("</ol>\n");
        }
    }
}

fn render_list_item(out: &mut String, item: &ListItem) {
    out.push_str("<li>");
    for b in &item.blocks {
        render_block(out, b, false);
    }
    out.push_str("</li>\n");
}

fn render_inlines(out: &mut String, inlines: &[Inline]) {
    for inline in inlines {
        match inline {
            Inline::Text(s) => out.push_str(&escape::text(s)),
            Inline::Emphasis(inner) => {
                out.push_str("<em>");
                render_inlines(out, inner);
                out.push_str("</em>");
            }
            Inline::Strong(inner) => {
                out.push_str("<strong>");
                render_inlines(out, inner);
                out.push_str("</strong>");
            }
            Inline::SoftBreak => out.push(' '),
            Inline::HardBreak => out.push_str("<br/>"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_paragraph() {
        let html = render_blocks(
            &[Block::Paragraph(vec![Inline::Text("hello".into())])],
            false,
        );
        assert_eq!(html, "<p>hello</p>\n");
    }

    #[test]
    fn first_paragraph_in_chapter_gets_class() {
        let html = render_blocks(
            &[
                Block::Paragraph(vec![Inline::Text("one".into())]),
                Block::Paragraph(vec![Inline::Text("two".into())]),
            ],
            true,
        );
        assert!(html.contains(r#"<p class="first-paragraph">one</p>"#));
        assert!(html.contains("<p>two</p>"));
    }

    #[test]
    fn matter_pages_get_no_drop_cap_class() {
        let html = render_blocks(&[Block::Paragraph(vec![Inline::Text("hi".into())])], false);
        assert!(!html.contains("first-paragraph"));
    }

    #[test]
    fn emphasis_and_strong() {
        let html = render_blocks(
            &[Block::Paragraph(vec![
                Inline::Emphasis(vec![Inline::Text("i".into())]),
                Inline::Text(" and ".into()),
                Inline::Strong(vec![Inline::Text("b".into())]),
            ])],
            false,
        );
        assert_eq!(html, "<p><em>i</em> and <strong>b</strong></p>\n");
    }

    #[test]
    fn xml_special_chars_escaped() {
        let html = render_blocks(
            &[Block::Paragraph(vec![Inline::Text("a < b & c".into())])],
            false,
        );
        assert_eq!(html, "<p>a &lt; b &amp; c</p>\n");
    }

    #[test]
    fn scene_break_renders_ornament_div() {
        let html = render_blocks(&[Block::SceneBreak], false);
        assert!(html.contains("scene-break"));
        assert!(html.contains("* * *"));
    }

    #[test]
    fn headings_map_to_h2_h3_h4() {
        let html = render_blocks(
            &[
                Block::Heading {
                    level: HeadingLevel::H2,
                    content: vec![Inline::Text("Two".into())],
                },
                Block::Heading {
                    level: HeadingLevel::H3,
                    content: vec![Inline::Text("Three".into())],
                },
                Block::Heading {
                    level: HeadingLevel::H4,
                    content: vec![Inline::Text("Four".into())],
                },
            ],
            false,
        );
        assert!(html.contains("<h2>Two</h2>"));
        assert!(html.contains("<h3>Three</h3>"));
        assert!(html.contains("<h4>Four</h4>"));
    }

    #[test]
    fn blockquote_wraps_inner_blocks() {
        let html = render_blocks(
            &[Block::BlockQuote(vec![Block::Paragraph(vec![
                Inline::Text("quoted".into()),
            ])])],
            false,
        );
        assert!(html.starts_with("<blockquote>"));
        assert!(html.contains("<p>quoted</p>"));
        assert!(html.contains("</blockquote>"));
    }

    #[test]
    fn unordered_list() {
        let html = render_blocks(
            &[Block::UnorderedList(vec![
                ListItem {
                    blocks: vec![Block::Paragraph(vec![Inline::Text("a".into())])],
                },
                ListItem {
                    blocks: vec![Block::Paragraph(vec![Inline::Text("b".into())])],
                },
            ])],
            false,
        );
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li><p>a</p>\n</li>"));
        assert!(html.contains("</ul>"));
    }

    #[test]
    fn hard_break_becomes_br() {
        let html = render_blocks(
            &[Block::Paragraph(vec![
                Inline::Text("line".into()),
                Inline::HardBreak,
                Inline::Text("break".into()),
            ])],
            false,
        );
        assert!(html.contains("<br/>"));
    }

    #[test]
    fn soft_break_becomes_space() {
        let html = render_blocks(
            &[Block::Paragraph(vec![
                Inline::Text("one".into()),
                Inline::SoftBreak,
                Inline::Text("two".into()),
            ])],
            false,
        );
        assert_eq!(html, "<p>one two</p>\n");
    }

    #[test]
    fn paragraphs_inside_blockquote_never_get_drop_cap() {
        let html = render_blocks(
            &[Block::BlockQuote(vec![Block::Paragraph(vec![
                Inline::Text("q".into()),
            ])])],
            true,
        );
        assert!(!html.contains("first-paragraph"));
    }
}
