//! Convert a Book IR into a self-contained Typst source string.
//!
//! All user-provided text flows through [`escape_str`] and is emitted
//! as `"..."` strings — never raw markup — so Typst markup
//! metacharacters in prose (`#`, `*`, `_`, `[`, etc.) can never trigger
//! unintended formatting.

use std::fmt::Write as _;

use papyrust_core::ir::{
    Block, Book, BookMeta, Chapter, HeadingLevel, Inline, ListItem, MatterPage,
};

pub fn build(book: &Book) -> String {
    let mut s = String::new();
    write_preamble(&mut s, book);
    write_title_page(&mut s, &book.meta);
    write_copyright_page(&mut s, &book.meta);
    write_toc(&mut s);

    // Front matter (if any): roman numerals.
    if !book.front_matter.is_empty() {
        s.push_str("\n#set page(numbering: \"i\")\n");
        s.push_str("#counter(page).update(1)\n");
        write_matter_pages(&mut s, &book.front_matter);
    }

    // Body (chapters + back matter): arabic numerals from 1, running
    // heads (book title verso, chapter title recto), all suppressed on
    // chapter-opening pages.
    write_body_layout(&mut s, &book.meta);
    write_chapters(&mut s, &book.chapters);
    write_matter_pages(&mut s, &book.back_matter);
    s
}

fn write_body_layout(s: &mut String, meta: &BookMeta) {
    s.push_str("\n#set page(\n");
    s.push_str("  numbering: \"1\",\n");
    // Running head: book title verso, chapter title recto.
    s.push_str("  header: context {\n");
    s.push_str("    let page-num = counter(page).get().first()\n");
    s.push_str("    let chapter-here = query(heading.where(level: 1)).filter(c => c.location().page() == page-num)\n");
    s.push_str("    if chapter-here.len() > 0 { return [] }\n");
    s.push_str("    let chapters-before = query(heading.where(level: 1).before(here()))\n");
    s.push_str("    if chapters-before.len() == 0 { return [] }\n");
    s.push_str("    let chapter-title = chapters-before.last().body\n");
    s.push_str("    if calc.even(page-num) {\n");
    let _ = writeln!(
        s,
        "      align(left, text(size: 0.85em, tracking: 0.1em, smallcaps(\"{}\")))",
        escape_str(&meta.title)
    );
    s.push_str("    } else {\n");
    s.push_str("      align(right, text(size: 0.85em, tracking: 0.1em, smallcaps(chapter-title)))\n");
    s.push_str("    }\n");
    s.push_str("  },\n");
    // Footer: centered page number, suppressed on chapter-opening pages.
    s.push_str("  footer: context {\n");
    s.push_str("    let page-num = counter(page).get().first()\n");
    s.push_str("    let chapter-here = query(heading.where(level: 1)).filter(c => c.location().page() == page-num)\n");
    s.push_str("    if chapter-here.len() > 0 { return [] }\n");
    s.push_str("    align(center, text(size: 0.85em, numbering(\"1\", page-num)))\n");
    s.push_str("  },\n");
    s.push_str(")\n");
    s.push_str("#counter(page).update(1)\n");
}

fn write_preamble(s: &mut String, book: &Book) {
    let (w, h) = book.meta.trim.dimensions_inches();
    let _ = writeln!(
        s,
        "#set document(title: \"{}\", author: \"{}\")",
        escape_str(&book.meta.title),
        escape_str(&book.meta.author)
    );
    let _ = writeln!(s, "#set page(");
    let _ = writeln!(s, "  width: {w}in, height: {h}in,");
    s.push_str("  margin: (top: 0.75in, bottom: 0.75in, inside: 0.875in, outside: 0.75in),\n");
    s.push_str("  numbering: none,\n");
    s.push_str(")\n");
    // Typst expects a bare ISO 639 language code (e.g. "en"), not a
    // BCP 47 tag (e.g. "en-US"). Strip the region subtag.
    let lang_iso = book.meta.language.split(['-', '_']).next().unwrap_or("en");
    let _ = writeln!(
        s,
        "#set text(font: \"EB Garamond\", size: 11pt, lang: \"{}\", hyphenate: true)",
        escape_str(lang_iso)
    );
    s.push_str(
        "#set par(leading: 0.65em, justify: true, first-line-indent: (amount: 1.5em, all: false))\n",
    );
    // Scene break helper: three asterisks with generous tracking is
    // the trade-press fiction convention and works in any font.
    s.push_str(
        "#let scene-break = {\n  v(0.7em)\n  align(center, text(tracking: 0.5em, \"* * *\"))\n  v(0.7em)\n}\n",
    );
    // Raised cap: the first character of each chapter's first paragraph
    // is set larger and slightly tracked, in the Penguin Classics style.
    // Typst lacks native text-wrap so a true floating drop cap isn't
    // available; this stays within the first line for a clean result.
    s.push_str(
        "#let raise-cap(letter) = text(size: 2.2em, weight: \"regular\", tracking: 0.05em, letter)\n",
    );
    // Chapter heading: each level-1 heading starts a new recto (odd page),
    // with centered small-caps display and breathing room.
    s.push_str("#show heading.where(level: 1): it => {\n");
    s.push_str("  pagebreak(weak: true, to: \"odd\")\n");
    s.push_str("  v(1.5in)\n");
    s.push_str("  align(center, text(size: 1.6em, tracking: 0.1em, weight: \"regular\", smallcaps(it.body)))\n");
    s.push_str("  v(2em)\n");
    s.push_str("}\n");
    s.push_str("#show heading.where(level: 2): it => align(center, text(size: 1.2em, smallcaps(it.body)))\n");
}

fn write_title_page(s: &mut String, meta: &BookMeta) {
    s.push_str("\n#align(center + horizon)[\n");
    let _ = writeln!(s, "  #text(size: 2.5em, \"{}\")", escape_str(&meta.title));
    if let Some(sub) = &meta.subtitle {
        s.push_str("  #v(0.5em)\n");
        let _ = writeln!(
            s,
            "  #text(size: 1.3em, style: \"italic\", \"{}\")",
            escape_str(sub)
        );
    }
    s.push_str("  #v(3em)\n");
    let _ = writeln!(
        s,
        "  #text(size: 1.2em, tracking: 0.08em, smallcaps(\"{}\"))",
        escape_str(&meta.author)
    );
    s.push_str("]\n");
    s.push_str("#pagebreak()\n");
}

fn write_copyright_page(s: &mut String, meta: &BookMeta) {
    let cp = &meta.copyright;
    s.push_str("\n#align(center + bottom)[\n");
    s.push_str("  #set text(size: 0.85em)\n");
    let _ = writeln!(
        s,
        "  #text(\"Copyright \u{00A9} {} {}\") \\",
        cp.year,
        escape_str(&cp.holder)
    );
    s.push_str("  All rights reserved. \\\n");
    let _ = writeln!(s, "  #text(\"Published by {}\")", escape_str(&cp.publisher));
    if let Some(isbn) = &cp.isbn_print.as_ref().or(cp.isbn_epub.as_ref()) {
        s.push_str("  \\\n");
        let _ = writeln!(s, "  #text(\"ISBN: {}\")", escape_str(isbn));
    }
    s.push_str("  #v(1in)\n");
    s.push_str("]\n");
    s.push_str("#pagebreak()\n");
}

fn write_toc(s: &mut String) {
    s.push_str("\n#align(center)[\n");
    s.push_str("  #v(1.5in)\n");
    s.push_str("  #text(size: 1.6em, tracking: 0.1em, smallcaps(\"Contents\"))\n");
    s.push_str("]\n");
    s.push_str("#v(1em)\n");
    s.push_str("#outline(title: none, depth: 1)\n");
    s.push_str("#pagebreak()\n");
}

fn write_chapters(s: &mut String, chapters: &[Chapter]) {
    for ch in chapters {
        s.push('\n');
        // The level-1 heading triggers the show rule (new page + display).
        let _ = writeln!(s, "= {}", escape_str(&ch.title));
        write_chapter_body(s, &ch.blocks);
    }
}

/// Render a chapter's blocks, applying a raised cap to the first
/// character of the first paragraph (when that first block is a
/// paragraph beginning with a letter).
fn write_chapter_body(s: &mut String, blocks: &[Block]) {
    let mut applied_cap = false;
    for block in blocks {
        s.push('\n');
        if !applied_cap {
            applied_cap = true;
            if let Block::Paragraph(inlines) = block {
                if write_paragraph_with_cap(s, inlines) {
                    continue;
                }
            }
        }
        write_block(s, block);
    }
}

/// If `inlines` starts with a `Text` whose first char is alphabetic,
/// emit the paragraph with that character wrapped in `#raise-cap(...)`
/// and return true. Otherwise emit nothing and return false.
fn write_paragraph_with_cap(s: &mut String, inlines: &[Inline]) -> bool {
    let Some(Inline::Text(text)) = inlines.first() else {
        return false;
    };
    let Some(first_char) = text.chars().next() else {
        return false;
    };
    if !first_char.is_alphabetic() {
        return false;
    }
    let rest = &text[first_char.len_utf8()..];
    let _ = write!(
        s,
        "#raise-cap(\"{}\")",
        escape_str(&first_char.to_string())
    );
    if !rest.is_empty() {
        let _ = write!(s, "#text(\"{}\")", escape_str(rest));
    }
    for inline in &inlines[1..] {
        write_inline(s, inline);
    }
    s.push('\n');
    true
}

fn write_matter_pages(s: &mut String, pages: &[MatterPage]) {
    for page in pages {
        s.push_str("\n#pagebreak(weak: true)\n");
        if let Some(title) = &page.title {
            // Matter page titles use a level-2 heading so they don't
            // trigger the chapter show rule but still appear in print.
            let _ = writeln!(s, "== {}", escape_str(title));
        }
        write_blocks(s, &page.blocks, false);
    }
}

fn write_blocks(s: &mut String, blocks: &[Block], _is_chapter_body: bool) {
    for block in blocks {
        s.push('\n');
        write_block(s, block);
    }
}

fn write_block(s: &mut String, block: &Block) {
    match block {
        Block::Paragraph(inlines) => {
            write_inlines(s, inlines);
            s.push('\n');
        }
        Block::Heading { level, content } => {
            let prefix = match level {
                HeadingLevel::H2 => "==",
                HeadingLevel::H3 => "===",
                HeadingLevel::H4 => "====",
            };
            s.push_str(prefix);
            s.push(' ');
            write_inlines(s, content);
            s.push('\n');
        }
        Block::SceneBreak => {
            s.push_str("#scene-break\n");
        }
        Block::BlockQuote(inner) => {
            s.push_str("#quote(block: true)[\n");
            for b in inner {
                write_block(s, b);
            }
            s.push_str("]\n");
        }
        Block::UnorderedList(items) => {
            for item in items {
                write_list_item(s, item, "-");
            }
        }
        Block::OrderedList(items) => {
            for item in items {
                write_list_item(s, item, "+");
            }
        }
    }
}

fn write_list_item(s: &mut String, item: &ListItem, marker: &str) {
    s.push_str(marker);
    s.push(' ');
    // Item blocks are usually a single paragraph; render inlines without
    // a wrapping `#par`.
    for block in &item.blocks {
        match block {
            Block::Paragraph(inlines) => write_inlines(s, inlines),
            other => write_block(s, other),
        }
    }
    s.push('\n');
}

fn write_inlines(s: &mut String, inlines: &[Inline]) {
    for inline in inlines {
        write_inline(s, inline);
    }
}

fn write_inline(s: &mut String, inline: &Inline) {
    match inline {
        Inline::Text(t) => {
            let _ = write!(s, "#text(\"{}\")", escape_str(t));
        }
        Inline::Emphasis(inner) => {
            s.push_str("#emph[");
            write_inlines(s, inner);
            s.push(']');
        }
        Inline::Strong(inner) => {
            s.push_str("#strong[");
            write_inlines(s, inner);
            s.push(']');
        }
        Inline::SoftBreak => s.push(' '),
        Inline::HardBreak => s.push_str(" \\\n"),
    }
}

/// Escape a string for use inside a Typst `"..."` literal.
/// Only backslash and double-quote need escaping in that context.
fn escape_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    use papyrust_core::config::TrimSize;
    use papyrust_core::ir::{Chapter, Copyright};

    fn meta() -> BookMeta {
        BookMeta {
            title: "Title".into(),
            subtitle: None,
            author: "Author".into(),
            language: "en-US".into(),
            copyright: Copyright {
                year: 2026,
                holder: "Author".into(),
                isbn_epub: None,
                isbn_print: None,
                publisher: "Self-Published".into(),
            },
            trim: TrimSize::SixByNine,
        }
    }

    fn small_book() -> Book {
        Book {
            meta: meta(),
            cover: None,
            front_matter: vec![],
            chapters: vec![Chapter {
                title: "One".into(),
                blocks: vec![Block::Paragraph(vec![Inline::Text("Hello.".into())])],
            }],
            back_matter: vec![],
        }
    }

    #[test]
    fn escape_str_handles_backslash_and_quote() {
        assert_eq!(escape_str(r#"a \ b "quoted""#), r#"a \\ b \"quoted\""#);
    }

    #[test]
    fn escape_str_leaves_typst_markup_chars_alone() {
        // Inside a "..." string, these are literal; no Typst escaping needed.
        assert_eq!(escape_str("a # b * c _ d"), "a # b * c _ d");
    }

    #[test]
    fn preamble_uses_trim_dimensions() {
        let mut s = String::new();
        write_preamble(&mut s, &small_book());
        assert!(s.contains("width: 6in"));
        assert!(s.contains("height: 9in"));
    }

    #[test]
    fn preamble_sets_font_and_strips_region_from_language() {
        let mut s = String::new();
        write_preamble(&mut s, &small_book());
        assert!(s.contains(r#"font: "EB Garamond""#));
        // Typst wants ISO 639 only — "en", not "en-US".
        assert!(s.contains(r#"lang: "en""#));
        assert!(!s.contains(r#"lang: "en-US""#));
    }

    #[test]
    fn build_includes_required_sections() {
        let src = build(&small_book());
        assert!(src.contains("#set document"));
        assert!(src.contains("#set page"));
        assert!(src.contains("#set text"));
        assert!(src.contains("#align(center + horizon)")); // title page
        assert!(src.contains("Copyright")); // copyright page
        assert!(src.contains("#outline")); // TOC
        assert!(src.contains("= One")); // chapter heading
    }

    #[test]
    fn user_text_is_safely_quoted() {
        let mut book = small_book();
        // Two paragraphs: the first has a raised cap applied to "H",
        // the second is rendered as a single escaped `#text(...)`.
        book.chapters[0].blocks = vec![
            Block::Paragraph(vec![Inline::Text("First paragraph.".into())]),
            Block::Paragraph(vec![Inline::Text(
                r#"He said "hi" \ and # also"#.into(),
            )]),
        ];
        let src = build(&book);
        assert!(src.contains(r#"#text("He said \"hi\" \\ and # also")"#));
    }

    #[test]
    fn emphasis_and_strong_use_typst_functions() {
        let mut book = small_book();
        book.chapters[0].blocks = vec![Block::Paragraph(vec![
            Inline::Emphasis(vec![Inline::Text("em".into())]),
            Inline::Strong(vec![Inline::Text("st".into())]),
        ])];
        let src = build(&book);
        assert!(src.contains(r#"#emph[#text("em")]"#));
        assert!(src.contains(r#"#strong[#text("st")]"#));
    }

    #[test]
    fn scene_break_emits_helper() {
        let mut book = small_book();
        book.chapters[0].blocks = vec![Block::SceneBreak];
        let src = build(&book);
        assert!(src.contains("#scene-break"));
    }

    #[test]
    fn subtitle_omitted_when_none() {
        let src = build(&small_book());
        assert!(!src.contains("style: \"italic\""));
    }

    #[test]
    fn subtitle_included_when_some() {
        let mut book = small_book();
        book.meta.subtitle = Some("A Subtitle".into());
        let src = build(&book);
        assert!(src.contains(r#"style: "italic""#));
        assert!(src.contains("A Subtitle"));
    }

    #[test]
    fn first_chapter_paragraph_gets_raised_cap() {
        let mut book = small_book();
        book.chapters[0].blocks = vec![
            Block::Paragraph(vec![Inline::Text("It was a dark night.".into())]),
            Block::Paragraph(vec![Inline::Text("Second paragraph.".into())]),
        ];
        let src = build(&book);
        // First char "I" wrapped in raise-cap, rest of first paragraph text follows.
        assert!(src.contains(r#"#raise-cap("I")"#));
        assert!(src.contains(r#"#text("t was a dark night.")"#));
        // Second paragraph: no raise-cap applied.
        assert!(src.contains(r#"#text("Second paragraph.")"#));
    }

    #[test]
    fn raised_cap_skipped_when_first_char_not_alphabetic() {
        let mut book = small_book();
        book.chapters[0].blocks = vec![Block::Paragraph(vec![Inline::Text(
            "1999 was a strange year.".into(),
        )])];
        let src = build(&book);
        assert!(!src.contains("#raise-cap"));
        assert!(src.contains(r#"#text("1999 was a strange year.")"#));
    }

    #[test]
    fn raised_cap_skipped_when_first_inline_is_emphasis() {
        let mut book = small_book();
        book.chapters[0].blocks = vec![Block::Paragraph(vec![
            Inline::Emphasis(vec![Inline::Text("Italic".into())]),
            Inline::Text(" start.".into()),
        ])];
        let src = build(&book);
        assert!(!src.contains("#raise-cap"));
    }

    #[test]
    fn isbn_print_prefers_over_isbn_epub() {
        let mut book = small_book();
        book.meta.copyright.isbn_print = Some("PRINT".into());
        book.meta.copyright.isbn_epub = Some("EPUB".into());
        let src = build(&book);
        assert!(src.contains("ISBN: PRINT"));
        assert!(!src.contains("ISBN: EPUB"));
    }
}
