//! In-memory Book intermediate representation.
//!
//! The IR is the contract between parsing and rendering. Renderers
//! (EPUB, PDF) consume a [`Book`] and produce output; they do not
//! consult the on-disk project layout or the raw Markdown.
//!
//! The IR is intentionally concrete and limited to a trade-press book
//! of long-form prose. We add new block kinds only when a renderer
//! actually needs them.

use crate::config::TrimSize;

#[derive(Debug, Clone)]
pub struct Book {
    pub meta: BookMeta,
    pub cover: Option<Cover>,
    pub front_matter: Vec<MatterPage>,
    pub chapters: Vec<Chapter>,
    pub back_matter: Vec<MatterPage>,
}

/// Cover image loaded into memory at project assembly time. Renderers
/// embed `bytes` directly; they do not touch the filesystem for it.
#[derive(Debug, Clone)]
pub struct Cover {
    pub bytes: Vec<u8>,
    pub mime: &'static str,
}

#[derive(Debug, Clone)]
pub struct BookMeta {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: String,
    pub language: String,
    pub copyright: Copyright,
    pub trim: TrimSize,
}

#[derive(Debug, Clone)]
pub struct Copyright {
    pub year: u16,
    pub holder: String,
    pub isbn_epub: Option<String>,
    pub isbn_print: Option<String>,
    pub publisher: String,
}

#[derive(Debug, Clone)]
pub struct Chapter {
    /// Title extracted from the first `# H1` in the chapter file.
    pub title: String,
    pub blocks: Vec<Block>,
}

/// Front-matter or back-matter page. Same shape; pagination rules differ
/// based on whether it appears in [`Book::front_matter`] or
/// [`Book::back_matter`].
#[derive(Debug, Clone)]
pub struct MatterPage {
    /// Optional title from the first `# H1`. Some pages (a one-line
    /// dedication) won't have one.
    pub title: Option<String>,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub enum Block {
    Paragraph(Vec<Inline>),
    Heading {
        level: HeadingLevel,
        content: Vec<Inline>,
    },
    /// A horizontal rule in Markdown. Renders as a centered ornament.
    SceneBreak,
    BlockQuote(Vec<Block>),
    UnorderedList(Vec<ListItem>),
    OrderedList(Vec<ListItem>),
}

/// Subdivisions within a chapter or matter page. The chapter title
/// itself is `# H1` and lives on [`Chapter::title`] / [`MatterPage::title`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingLevel {
    H2,
    H3,
    H4,
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub enum Inline {
    Text(String),
    Emphasis(Vec<Inline>),
    Strong(Vec<Inline>),
    /// Markdown soft break (newline inside paragraph). Most renderers
    /// treat this as a space.
    SoftBreak,
    /// Markdown hard break (two trailing spaces or trailing backslash).
    HardBreak,
}

impl From<&crate::config::BookConfig> for BookMeta {
    fn from(cfg: &crate::config::BookConfig) -> Self {
        Self {
            title: cfg.book.title.clone(),
            subtitle: cfg.book.subtitle.clone(),
            author: cfg.book.author.clone(),
            language: cfg.book.language.clone(),
            copyright: Copyright {
                year: cfg.copyright.year,
                holder: cfg.copyright.holder.clone(),
                isbn_epub: cfg.copyright.isbn_epub.clone(),
                isbn_print: cfg.copyright.isbn_print.clone(),
                publisher: cfg.copyright.publisher.clone(),
            },
            trim: cfg.print.trim,
        }
    }
}
