//! EPUB 3 renderer for papyrust.
//!
//! Consumes a [`crate::ir::Book`] and produces an EPUB 3 file.
//! Output is plain zipped XHTML + a few XML metadata documents — no
//! reader-specific extensions, no heavyweight framework.

use std::fmt::Write as _;
use std::path::Path;

use crate::ir::{Book, Chapter, MatterPage};

mod archive;
mod escape;
mod nav;
mod opf;
mod pages;
mod paths;
mod xhtml;

const STYLESHEET: &str = include_str!("theme.css");

#[derive(Debug, thiserror::Error)]
pub enum EpubError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("zip: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("formatting time: {0}")]
    Time(#[from] time::error::Format),
}

/// Build a full EPUB 3 file at `output` from the given Book IR.
pub fn render(book: &Book, output: &Path) -> Result<(), EpubError> {
    let pkg = assemble(book)?;
    let file = std::fs::File::create(output)?;
    archive::pack(&pkg, file)
}

fn assemble(book: &Book) -> Result<archive::EpubPackage<'_>, EpubError> {
    let opf = opf::build(book)?;
    let nav = nav::build(book);
    let title_xhtml = pages::title_page(&book.meta);
    let copyright_xhtml = pages::copyright_page(&book.meta);

    let cover = book.cover.as_ref().map(|c| archive::CoverFiles {
        xhtml: pages::cover_page(&book.meta),
        image_bytes: &c.bytes,
    });

    let front_matter_xhtml = book
        .front_matter
        .iter()
        .map(|page| matter_xhtml(page, "frontmatter", &book.meta.language))
        .collect();
    let chapter_xhtml = book
        .chapters
        .iter()
        .map(|ch| chapter_xhtml(ch, &book.meta.language))
        .collect();
    let back_matter_xhtml = book
        .back_matter
        .iter()
        .map(|page| matter_xhtml(page, "backmatter", &book.meta.language))
        .collect();

    Ok(archive::EpubPackage {
        opf,
        nav,
        stylesheet: STYLESHEET,
        title_xhtml,
        copyright_xhtml,
        cover,
        front_matter_xhtml,
        chapter_xhtml,
        back_matter_xhtml,
    })
}

fn chapter_xhtml(ch: &Chapter, lang: &str) -> String {
    let mut body = String::from("<section class=\"chapter\">\n");
    let _ = writeln!(
        body,
        "<h1 class=\"chapter-title\">{}</h1>",
        escape::text(&ch.title)
    );
    body.push_str(&xhtml::render_blocks(&ch.blocks, true));
    body.push_str("</section>\n");
    pages::document(&ch.title, Some("chapter"), &body, lang)
}

fn matter_xhtml(page: &MatterPage, epub_type: &str, lang: &str) -> String {
    let mut body = String::from("<section class=\"matter-page\">\n");
    if let Some(title) = &page.title {
        let _ = writeln!(
            body,
            "<h1 class=\"matter-title\">{}</h1>",
            escape::text(title)
        );
    }
    body.push_str(&xhtml::render_blocks(&page.blocks, false));
    body.push_str("</section>\n");
    let head_title = page.title.as_deref().unwrap_or(epub_type);
    pages::document(head_title, Some(epub_type), &body, lang)
}
