//! Markdown → Book IR conversion.
//!
//! [`parse_chapter`] requires the file's first block to be an `# H1`
//! and uses it as the chapter title. [`parse_matter_page`] treats the
//! H1 as optional.
//!
//! Unsupported Markdown constructs (code blocks, HTML, images, links,
//! tables, footnotes) are silently dropped in v1. The set of supported
//! constructs grows when a renderer actually needs it.

use std::iter::Peekable;

use pulldown_cmark::{Event, HeadingLevel as MdHeadingLevel, Parser, Tag, TagEnd};

use crate::ir::{Block, Chapter, HeadingLevel, Inline, ListItem, MatterPage};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("chapter is missing a `# Heading` title")]
    MissingChapterTitle,
}

pub fn parse_chapter(source: &str) -> Result<Chapter, ParseError> {
    let mut events = Parser::new(source).peekable();
    let title = take_first_h1_title(&mut events).ok_or(ParseError::MissingChapterTitle)?;
    let blocks = parse_blocks_until(&mut events, Terminator::None);
    Ok(Chapter { title, blocks })
}

pub fn parse_matter_page(source: &str) -> MatterPage {
    let mut events = Parser::new(source).peekable();
    let title = take_first_h1_title(&mut events);
    let blocks = parse_blocks_until(&mut events, Terminator::None);
    MatterPage { title, blocks }
}

/// Match a specific kind of closing tag, ignoring payloads. Replaces
/// per-call closures (which would each be a distinct generic instantiation
/// and hit the recursion limit during monomorphization).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Terminator {
    None,
    Paragraph,
    Heading,
    BlockQuote,
    Item,
    Emphasis,
    Strong,
}

impl Terminator {
    fn matches(self, end: TagEnd) -> bool {
        matches!(
            (self, end),
            (Self::Paragraph, TagEnd::Paragraph)
                | (Self::Heading, TagEnd::Heading(_))
                | (Self::BlockQuote, TagEnd::BlockQuote(_))
                | (Self::Item, TagEnd::Item)
                | (Self::Emphasis, TagEnd::Emphasis)
                | (Self::Strong, TagEnd::Strong)
        )
    }
}

/// If the next block is an H1, consume it and return its plain text.
fn take_first_h1_title<'a, I>(events: &mut Peekable<I>) -> Option<String>
where
    I: Iterator<Item = Event<'a>>,
{
    let is_h1 = matches!(
        events.peek(),
        Some(Event::Start(Tag::Heading {
            level: MdHeadingLevel::H1,
            ..
        }))
    );
    if !is_h1 {
        return None;
    }
    events.next(); // consume Start(Heading H1)
    let inlines = parse_inlines_until(events, Terminator::Heading);
    Some(inlines_to_plain_text(&inlines))
}

fn parse_blocks_until<'a, I>(events: &mut Peekable<I>, terminator: Terminator) -> Vec<Block>
where
    I: Iterator<Item = Event<'a>>,
{
    let mut blocks = Vec::new();
    while let Some(event) = events.peek() {
        if let Event::End(end) = event {
            if terminator.matches(*end) {
                events.next();
                break;
            }
        }
        if let Some(block) = parse_block(events) {
            blocks.push(block);
        }
    }
    blocks
}

fn parse_block<'a, I>(events: &mut Peekable<I>) -> Option<Block>
where
    I: Iterator<Item = Event<'a>>,
{
    match events.next()? {
        Event::Start(Tag::Paragraph) => {
            let inlines = parse_inlines_until(events, Terminator::Paragraph);
            Some(Block::Paragraph(inlines))
        }
        Event::Start(Tag::Heading { level, .. }) => {
            let mapped = map_heading_level(level);
            let content = parse_inlines_until(events, Terminator::Heading);
            Some(Block::Heading {
                level: mapped,
                content,
            })
        }
        Event::Start(Tag::BlockQuote(_)) => {
            let inner = parse_blocks_until(events, Terminator::BlockQuote);
            Some(Block::BlockQuote(inner))
        }
        Event::Start(Tag::List(start)) => {
            let ordered = start.is_some();
            let items = parse_list_items(events);
            Some(if ordered {
                Block::OrderedList(items)
            } else {
                Block::UnorderedList(items)
            })
        }
        Event::Rule => Some(Block::SceneBreak),
        // Unsupported block-level constructs are silently dropped.
        _ => None,
    }
}

fn parse_list_items<'a, I>(events: &mut Peekable<I>) -> Vec<ListItem>
where
    I: Iterator<Item = Event<'a>>,
{
    let mut items = Vec::new();
    while let Some(event) = events.peek() {
        match event {
            Event::Start(Tag::Item) => {
                events.next();
                let blocks = parse_blocks_until(events, Terminator::Item);
                items.push(ListItem { blocks });
            }
            Event::End(TagEnd::List(_)) => {
                events.next();
                break;
            }
            _ => {
                events.next();
            }
        }
    }
    items
}

fn parse_inlines_until<'a, I>(events: &mut Peekable<I>, terminator: Terminator) -> Vec<Inline>
where
    I: Iterator<Item = Event<'a>>,
{
    let mut inlines = Vec::new();
    while let Some(event) = events.peek() {
        if let Event::End(end) = event {
            if terminator.matches(*end) {
                events.next();
                break;
            }
        }
        let Some(event) = events.next() else { break };
        match event {
            Event::Text(text) | Event::Code(text) => {
                inlines.push(Inline::Text(text.to_string()));
            }
            Event::SoftBreak => inlines.push(Inline::SoftBreak),
            Event::HardBreak => inlines.push(Inline::HardBreak),
            Event::Start(Tag::Emphasis) => {
                let inner = parse_inlines_until(events, Terminator::Emphasis);
                inlines.push(Inline::Emphasis(inner));
            }
            Event::Start(Tag::Strong) => {
                let inner = parse_inlines_until(events, Terminator::Strong);
                inlines.push(Inline::Strong(inner));
            }
            // Unsupported inlines are silently dropped.
            _ => {}
        }
    }
    inlines
}

fn map_heading_level(level: MdHeadingLevel) -> HeadingLevel {
    // H1 past the title is treated as H2; H5/H6 collapse to H4.
    match level {
        MdHeadingLevel::H1 | MdHeadingLevel::H2 => HeadingLevel::H2,
        MdHeadingLevel::H3 => HeadingLevel::H3,
        MdHeadingLevel::H4 | MdHeadingLevel::H5 | MdHeadingLevel::H6 => HeadingLevel::H4,
    }
}

fn inlines_to_plain_text(inlines: &[Inline]) -> String {
    let mut out = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(s) => out.push_str(s),
            Inline::Emphasis(inner) | Inline::Strong(inner) => {
                out.push_str(&inlines_to_plain_text(inner));
            }
            Inline::SoftBreak | Inline::HardBreak => out.push(' '),
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_chapter_with_title_and_paragraphs() {
        let md = "# The Beginning\n\nIt was a dark night.\n\nThen morning came.\n";
        let ch = parse_chapter(md).unwrap();
        assert_eq!(ch.title, "The Beginning");
        assert_eq!(ch.blocks.len(), 2);
        assert!(matches!(ch.blocks[0], Block::Paragraph(_)));
    }

    #[test]
    fn chapter_without_h1_errors() {
        let md = "Just some prose with no heading.\n";
        assert!(matches!(
            parse_chapter(md),
            Err(ParseError::MissingChapterTitle)
        ));
    }

    #[test]
    fn title_collapses_inline_emphasis() {
        let md = "# Chapter 1: *The Beginning*\n";
        let ch = parse_chapter(md).unwrap();
        assert_eq!(ch.title, "Chapter 1: The Beginning");
    }

    #[test]
    fn horizontal_rule_becomes_scene_break() {
        let md = "# X\n\nFirst.\n\n---\n\nSecond.\n";
        let ch = parse_chapter(md).unwrap();
        let kinds: Vec<&'static str> = ch
            .blocks
            .iter()
            .map(|b| match b {
                Block::Paragraph(_) => "p",
                Block::SceneBreak => "br",
                _ => "?",
            })
            .collect();
        assert_eq!(kinds, vec!["p", "br", "p"]);
    }

    #[test]
    fn supports_h2_h3() {
        let md = "# X\n\n## Subhead\n\n### Smaller\n";
        let ch = parse_chapter(md).unwrap();
        assert_eq!(ch.blocks.len(), 2);
        assert!(matches!(
            ch.blocks[0],
            Block::Heading {
                level: HeadingLevel::H2,
                ..
            }
        ));
        assert!(matches!(
            ch.blocks[1],
            Block::Heading {
                level: HeadingLevel::H3,
                ..
            }
        ));
    }

    #[test]
    fn supports_blockquote() {
        let md = "# X\n\n> Quoted line.\n> Continued.\n";
        let ch = parse_chapter(md).unwrap();
        assert_eq!(ch.blocks.len(), 1);
        let Block::BlockQuote(inner) = &ch.blocks[0] else {
            panic!("expected blockquote");
        };
        assert_eq!(inner.len(), 1);
        assert!(matches!(inner[0], Block::Paragraph(_)));
    }

    #[test]
    fn supports_unordered_list() {
        let md = "# X\n\n- one\n- two\n- three\n";
        let ch = parse_chapter(md).unwrap();
        let Block::UnorderedList(items) = &ch.blocks[0] else {
            panic!("expected unordered list");
        };
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn supports_ordered_list() {
        let md = "# X\n\n1. one\n2. two\n";
        let ch = parse_chapter(md).unwrap();
        assert!(matches!(ch.blocks[0], Block::OrderedList(_)));
    }

    #[test]
    fn emphasis_and_strong_in_paragraph() {
        let md = "# X\n\nThis is *italic* and **bold**.\n";
        let ch = parse_chapter(md).unwrap();
        let Block::Paragraph(inlines) = &ch.blocks[0] else {
            panic!();
        };
        let kinds: Vec<&'static str> = inlines
            .iter()
            .map(|i| match i {
                Inline::Text(_) => "t",
                Inline::Emphasis(_) => "em",
                Inline::Strong(_) => "st",
                Inline::SoftBreak => "sb",
                Inline::HardBreak => "hb",
            })
            .collect();
        assert!(kinds.contains(&"em"));
        assert!(kinds.contains(&"st"));
    }

    #[test]
    fn matter_page_h1_optional() {
        let with = parse_matter_page("# Dedication\n\nFor you.\n");
        assert_eq!(with.title.as_deref(), Some("Dedication"));

        let without = parse_matter_page("For you.\n");
        assert!(without.title.is_none());
        assert_eq!(without.blocks.len(), 1);
    }

    #[test]
    fn empty_input_parses_as_empty_matter_page() {
        let p = parse_matter_page("");
        assert!(p.title.is_none());
        assert!(p.blocks.is_empty());
    }

    #[test]
    fn stray_h1_after_title_downgraded_to_h2() {
        let md = "# First\n\n# Second\n";
        let ch = parse_chapter(md).unwrap();
        assert_eq!(ch.title, "First");
        let Block::Heading { level, .. } = &ch.blocks[0] else {
            panic!("expected heading block");
        };
        assert_eq!(*level, HeadingLevel::H2);
    }

    #[test]
    fn unsupported_constructs_do_not_panic() {
        let md = "# X\n\n```\nlet x = 1;\n```\n\n![alt](img.png)\n\n[link](u)\n";
        let ch = parse_chapter(md).unwrap();
        assert!(ch.blocks.iter().all(|b| !matches!(b, Block::SceneBreak)));
    }
}
