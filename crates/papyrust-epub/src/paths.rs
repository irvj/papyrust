//! Filenames of every file written into the EPUB archive.
//!
//! Centralised so that modules that emit references (nav, opf, render
//! orchestrator) agree with the modules that emit the files themselves.

pub const COVER_XHTML: &str = "cover.xhtml";
pub const TITLE_XHTML: &str = "title.xhtml";
pub const COPYRIGHT_XHTML: &str = "copyright.xhtml";
pub const NAV_XHTML: &str = "nav.xhtml";
pub const COVER_IMAGE: &str = "cover.jpg";
pub const STYLESHEET: &str = "theme.css";

#[must_use]
pub fn chapter(idx: usize) -> String {
    format!("chapter-{:03}.xhtml", idx + 1)
}

#[must_use]
pub fn front_matter(idx: usize) -> String {
    format!("front-{:03}.xhtml", idx + 1)
}

#[must_use]
pub fn back_matter(idx: usize) -> String {
    format!("back-{:03}.xhtml", idx + 1)
}
