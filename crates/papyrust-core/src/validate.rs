//! Validation and project assembly.
//!
//! [`load_project`] is the single entry point for turning an on-disk
//! project into a Book IR. It collects all issues (errors and warnings)
//! into a [`Report`] rather than failing on the first problem, so users
//! see a complete picture.
//!
//! If the report has any errors, the returned `Book` is `None` and no
//! rendering can proceed. Warnings are non-fatal.

use std::path::{Path, PathBuf};

use crate::config;
use crate::ir::{Book, BookMeta, Chapter, MatterPage};
use crate::parse;
use crate::project::ProjectLayout;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Issue {
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Default, Clone)]
pub struct Report {
    pub issues: Vec<Issue>,
}

impl Report {
    fn push(&mut self, severity: Severity, message: impl Into<String>) {
        self.issues.push(Issue {
            severity,
            message: message.into(),
        });
    }

    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|i| i.severity == Severity::Error)
    }

    pub fn errors(&self) -> impl Iterator<Item = &Issue> {
        self.issues.iter().filter(|i| i.severity == Severity::Error)
    }

    pub fn warnings(&self) -> impl Iterator<Item = &Issue> {
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
    }
}

/// Load and validate a project, returning a Book IR if there are no errors.
///
/// Warnings (like a missing cover) are non-fatal and the Book is still
/// returned. Errors (missing `book.toml`, no chapters, chapter without
/// a heading) prevent the Book from being returned.
pub fn load_project(root: &Path) -> (Option<Book>, Report) {
    let mut report = Report::default();

    let layout = match ProjectLayout::discover(root) {
        Ok(layout) => layout,
        Err(e) => {
            report.push(Severity::Error, e.to_string());
            return (None, report);
        }
    };

    let cfg = match config::load(&layout.book_toml) {
        Ok(c) => c,
        Err(e) => {
            report.push(Severity::Error, e.to_string());
            return (None, report);
        }
    };

    if layout.cover.is_none() {
        report.push(
            Severity::Warning,
            "cover.jpg not found at project root (required for EPUB build)",
        );
    }

    if layout.chapters.is_empty() {
        report.push(Severity::Error, "no chapters found in chapters/");
    }

    let chapters = read_chapters(&layout.chapters, &mut report);
    let front_matter = read_matter(&layout.front_matter, &mut report);
    let back_matter = read_matter(&layout.back_matter, &mut report);

    if report.has_errors() {
        return (None, report);
    }

    let book = Book {
        meta: BookMeta::from(&cfg),
        front_matter,
        chapters,
        back_matter,
    };
    (Some(book), report)
}

fn read_chapters(paths: &[PathBuf], report: &mut Report) -> Vec<Chapter> {
    let mut chapters = Vec::with_capacity(paths.len());
    for path in paths {
        match std::fs::read_to_string(path) {
            Ok(src) => match parse::parse_chapter(&src) {
                Ok(chapter) => chapters.push(chapter),
                Err(e) => report.push(Severity::Error, format!("{}: {e}", path.display())),
            },
            Err(e) => report.push(
                Severity::Error,
                format!("could not read {}: {e}", path.display()),
            ),
        }
    }
    chapters
}

fn read_matter(paths: &[PathBuf], report: &mut Report) -> Vec<MatterPage> {
    let mut pages = Vec::with_capacity(paths.len());
    for path in paths {
        match std::fs::read_to_string(path) {
            Ok(src) => pages.push(parse::parse_matter_page(&src)),
            Err(e) => report.push(
                Severity::Warning,
                format!("could not read {}: {e}", path.display()),
            ),
        }
    }
    pages
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use tempfile::TempDir;

    const GOOD_TOML: &str = r#"
        [book]
        title = "Test"
        author = "A"
        [copyright]
        year = 2026
        holder = "A"
        [print]
        trim = "6x9"
    "#;

    fn write(root: &Path, rel: &str, contents: &str) {
        let p = root.join(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, contents).unwrap();
    }

    #[test]
    fn full_valid_project_loads_clean() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        write(root, "book.toml", GOOD_TOML);
        write(root, "cover.jpg", "");
        write(root, "chapters/01-a.md", "# One\n\nProse.\n");
        write(root, "chapters/02-b.md", "# Two\n\nMore prose.\n");
        write(root, "front-matter/01-ded.md", "For you.\n");
        write(root, "back-matter/01-about.md", "About me.\n");

        let (book, report) = load_project(root);
        assert!(!report.has_errors(), "issues: {:?}", report.issues);
        assert!(report.warnings().count() == 0);
        let book = book.unwrap();
        assert_eq!(book.chapters.len(), 2);
        assert_eq!(book.front_matter.len(), 1);
        assert_eq!(book.back_matter.len(), 1);
    }

    #[test]
    fn missing_book_toml_is_error() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        write(root, "chapters/01-a.md", "# One\n");

        let (book, report) = load_project(root);
        assert!(book.is_none());
        assert!(report.has_errors());
    }

    #[test]
    fn malformed_book_toml_is_error() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        write(root, "book.toml", "not = valid =");
        write(root, "chapters/01-a.md", "# One\n");

        let (book, report) = load_project(root);
        assert!(book.is_none());
        assert!(report.has_errors());
    }

    #[test]
    fn missing_cover_is_warning_not_error() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        write(root, "book.toml", GOOD_TOML);
        write(root, "chapters/01-a.md", "# One\n");

        let (book, report) = load_project(root);
        assert!(book.is_some(), "should still load without cover");
        assert!(!report.has_errors());
        assert_eq!(report.warnings().count(), 1);
    }

    #[test]
    fn empty_chapters_is_error() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        write(root, "book.toml", GOOD_TOML);
        write(root, "cover.jpg", "");

        let (book, report) = load_project(root);
        assert!(book.is_none());
        assert!(report.has_errors());
    }

    #[test]
    fn chapter_without_h1_is_error() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        write(root, "book.toml", GOOD_TOML);
        write(root, "cover.jpg", "");
        write(root, "chapters/01-a.md", "No heading here.\n");

        let (book, report) = load_project(root);
        assert!(book.is_none());
        assert!(report.has_errors());
    }

    #[test]
    fn root_not_a_directory_is_error() {
        let tmp = TempDir::new().unwrap();
        let bogus = tmp.path().join("does-not-exist");
        let (book, report) = load_project(&bogus);
        assert!(book.is_none());
        assert!(report.has_errors());
    }
}
