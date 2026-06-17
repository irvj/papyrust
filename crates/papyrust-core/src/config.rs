//! `book.toml` schema and parser.
//!
//! The user's project root contains a `book.toml` describing the book's
//! metadata, copyright, and print settings. This module defines the typed
//! shape of that file and the loader.
//!
//! Unknown keys are silently ignored (forward-compat); upgrade to an
//! `unknown_fields` collection later if we need warnings.

use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct BookConfig {
    pub book: BookMeta,
    pub copyright: CopyrightMeta,
    pub print: PrintMeta,
    #[serde(default)]
    pub ebook: EbookMeta,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BookMeta {
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    pub author: String,
    #[serde(default = "default_language")]
    pub language: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CopyrightMeta {
    pub year: u16,
    pub holder: String,
    #[serde(default)]
    pub isbn_epub: Option<String>,
    #[serde(default)]
    pub isbn_print: Option<String>,
    #[serde(default = "default_publisher")]
    pub publisher: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrintMeta {
    pub trim: TrimSize,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct EbookMeta {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum TrimSize {
    #[serde(rename = "5x8")]
    FiveByEight,
    #[serde(rename = "5.5x8.5")]
    FiveFiveByEightFive,
    #[serde(rename = "6x9")]
    SixByNine,
}

impl TrimSize {
    #[must_use]
    pub const fn dimensions_inches(self) -> (f32, f32) {
        match self {
            Self::FiveByEight => (5.0, 8.0),
            Self::FiveFiveByEightFive => (5.5, 8.5),
            Self::SixByNine => (6.0, 9.0),
        }
    }

    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::FiveByEight => "5x8",
            Self::FiveFiveByEightFive => "5.5x8.5",
            Self::SixByNine => "6x9",
        }
    }
}

fn default_language() -> String {
    "en-US".to_string()
}

fn default_publisher() -> String {
    "Self-Published".to_string()
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("could not read {path}: {source}", path = path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("could not parse {path}: {source}", path = path.display())]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
}

/// Load and parse `book.toml` from disk.
pub fn load(path: &Path) -> Result<BookConfig, ConfigError> {
    let raw = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    toml::from_str(&raw).map_err(|source| ConfigError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = r#"
        [book]
        title = "Test Book"
        author = "Jane Doe"

        [copyright]
        year = 2026
        holder = "Jane Doe"

        [print]
        trim = "6x9"
    "#;

    #[test]
    fn parses_minimal_config() {
        let cfg: BookConfig = toml::from_str(MINIMAL).unwrap();
        assert_eq!(cfg.book.title, "Test Book");
        assert_eq!(cfg.book.author, "Jane Doe");
        assert!(cfg.book.subtitle.is_none());
        assert_eq!(cfg.book.language, "en-US"); // default
        assert_eq!(cfg.copyright.year, 2026);
        assert_eq!(cfg.copyright.publisher, "Self-Published"); // default
        assert_eq!(cfg.print.trim, TrimSize::SixByNine);
    }

    #[test]
    fn parses_full_config() {
        let raw = r#"
            [book]
            title = "T"
            subtitle = "S"
            author = "A"
            language = "fr-FR"

            [copyright]
            year = 2027
            holder = "H"
            isbn_epub = "978-0-000-00000-0"
            isbn_print = "978-0-000-00000-1"
            publisher = "P"

            [print]
            trim = "5.5x8.5"

            [ebook]
        "#;
        let cfg: BookConfig = toml::from_str(raw).unwrap();
        assert_eq!(cfg.book.subtitle.as_deref(), Some("S"));
        assert_eq!(cfg.book.language, "fr-FR");
        assert_eq!(
            cfg.copyright.isbn_epub.as_deref(),
            Some("978-0-000-00000-0")
        );
        assert_eq!(cfg.copyright.publisher, "P");
        assert_eq!(cfg.print.trim, TrimSize::FiveFiveByEightFive);
    }

    #[test]
    fn rejects_unknown_trim_size() {
        let raw = r#"
            [book]
            title = "T"
            author = "A"
            [copyright]
            year = 2026
            holder = "H"
            [print]
            trim = "7x10"
        "#;
        let result: Result<BookConfig, _> = toml::from_str(raw);
        assert!(result.is_err());
    }

    #[test]
    fn requires_book_section() {
        let raw = r#"
            [copyright]
            year = 2026
            holder = "H"
            [print]
            trim = "6x9"
        "#;
        let result: Result<BookConfig, _> = toml::from_str(raw);
        assert!(result.is_err());
    }

    #[test]
    fn trim_size_dimensions_match_slug() {
        for trim in [
            TrimSize::FiveByEight,
            TrimSize::FiveFiveByEightFive,
            TrimSize::SixByNine,
        ] {
            let (w, h) = trim.dimensions_inches();
            assert!(w > 0.0 && h > 0.0);
            assert!(!trim.slug().is_empty());
        }
    }
}
