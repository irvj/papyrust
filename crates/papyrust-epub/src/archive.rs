//! ZIP packaging of an EPUB.
//!
//! EPUB has two rigid layout requirements that other ZIP files don't:
//!
//! 1. The first entry must be `mimetype`, stored (no compression),
//!    with no extra fields, so a reader can identify the file by
//!    peeking at the raw bytes.
//! 2. The OPF and content documents live under a sub-directory
//!    (we use `OEBPS/`) and are referenced from `META-INF/container.xml`.

use std::io::{Seek, Write};

use zip::CompressionMethod;
use zip::write::{SimpleFileOptions, ZipWriter};

use crate::{EpubError, paths};

const MIMETYPE: &[u8] = b"application/epub+zip";

const CONTAINER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>
"#;

/// All the file contents that go into an EPUB. Built by the renderer
/// orchestrator, consumed by [`pack`].
#[derive(Debug)]
pub struct EpubPackage<'a> {
    pub opf: String,
    pub nav: String,
    pub stylesheet: &'a str,
    pub title_xhtml: String,
    pub copyright_xhtml: String,
    /// Set together with `cover_image` or not at all.
    pub cover: Option<CoverFiles<'a>>,
    pub front_matter_xhtml: Vec<String>,
    pub chapter_xhtml: Vec<String>,
    pub back_matter_xhtml: Vec<String>,
}

#[derive(Debug)]
pub struct CoverFiles<'a> {
    pub xhtml: String,
    pub image_bytes: &'a [u8],
}

/// Write a complete EPUB archive to `writer`.
pub fn pack<W: Write + Seek>(pkg: &EpubPackage<'_>, writer: W) -> Result<(), EpubError> {
    let mut zip = ZipWriter::new(writer);
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("mimetype", stored)?;
    zip.write_all(MIMETYPE)?;

    zip.start_file("META-INF/container.xml", deflated)?;
    zip.write_all(CONTAINER_XML.as_bytes())?;

    zip.start_file("OEBPS/content.opf", deflated)?;
    zip.write_all(pkg.opf.as_bytes())?;

    zip.start_file(oebps(paths::STYLESHEET), deflated)?;
    zip.write_all(pkg.stylesheet.as_bytes())?;

    zip.start_file(oebps(paths::NAV_XHTML), deflated)?;
    zip.write_all(pkg.nav.as_bytes())?;

    if let Some(cover) = &pkg.cover {
        zip.start_file(oebps(paths::COVER_IMAGE), stored)?;
        zip.write_all(cover.image_bytes)?;
        zip.start_file(oebps(paths::COVER_XHTML), deflated)?;
        zip.write_all(cover.xhtml.as_bytes())?;
    }

    zip.start_file(oebps(paths::TITLE_XHTML), deflated)?;
    zip.write_all(pkg.title_xhtml.as_bytes())?;

    zip.start_file(oebps(paths::COPYRIGHT_XHTML), deflated)?;
    zip.write_all(pkg.copyright_xhtml.as_bytes())?;

    for (i, xhtml) in pkg.front_matter_xhtml.iter().enumerate() {
        zip.start_file(oebps(&paths::front_matter(i)), deflated)?;
        zip.write_all(xhtml.as_bytes())?;
    }
    for (i, xhtml) in pkg.chapter_xhtml.iter().enumerate() {
        zip.start_file(oebps(&paths::chapter(i)), deflated)?;
        zip.write_all(xhtml.as_bytes())?;
    }
    for (i, xhtml) in pkg.back_matter_xhtml.iter().enumerate() {
        zip.start_file(oebps(&paths::back_matter(i)), deflated)?;
        zip.write_all(xhtml.as_bytes())?;
    }

    zip.finish()?;
    Ok(())
}

fn oebps(name: &str) -> String {
    format!("OEBPS/{name}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn minimal_pkg() -> EpubPackage<'static> {
        EpubPackage {
            opf: "<package/>".into(),
            nav: "<nav/>".into(),
            stylesheet: "body{}",
            title_xhtml: "<html/>".into(),
            copyright_xhtml: "<html/>".into(),
            cover: None,
            front_matter_xhtml: vec![],
            chapter_xhtml: vec!["<html>chapter 1</html>".into()],
            back_matter_xhtml: vec![],
        }
    }

    fn read_archive(bytes: Vec<u8>) -> zip::ZipArchive<Cursor<Vec<u8>>> {
        zip::ZipArchive::new(Cursor::new(bytes)).expect("valid zip")
    }

    #[test]
    fn writes_a_valid_zip() {
        let mut buf = Cursor::new(Vec::<u8>::new());
        pack(&minimal_pkg(), &mut buf).unwrap();
        let bytes = buf.into_inner();
        assert!(bytes.starts_with(&[0x50, 0x4B])); // PK signature
        let _ = read_archive(bytes);
    }

    #[test]
    fn mimetype_is_first_entry_and_stored() {
        let mut buf = Cursor::new(Vec::<u8>::new());
        pack(&minimal_pkg(), &mut buf).unwrap();
        let bytes = buf.into_inner();

        // First entry: local file header has signature PK\x03\x04 at offset 0.
        // At offset 26 begins file name length (2 bytes LE), 28 extra length (2 bytes LE),
        // 30 is filename.
        let name_len = u16::from_le_bytes([bytes[26], bytes[27]]) as usize;
        let name = &bytes[30..30 + name_len];
        assert_eq!(name, b"mimetype");

        // Compression method: bytes 8..10 of local file header → 0 = stored.
        let compression = u16::from_le_bytes([bytes[8], bytes[9]]);
        assert_eq!(compression, 0, "mimetype must be stored");

        // The literal "application/epub+zip" should appear in the raw bytes.
        let needle = b"application/epub+zip";
        assert!(bytes.windows(needle.len()).any(|w| w == needle));
    }

    #[test]
    fn contains_required_layout_entries() {
        let mut buf = Cursor::new(Vec::<u8>::new());
        pack(&minimal_pkg(), &mut buf).unwrap();
        let archive = read_archive(buf.into_inner());
        let names: Vec<&str> = archive.file_names().collect();
        for required in [
            "mimetype",
            "META-INF/container.xml",
            "OEBPS/content.opf",
            "OEBPS/nav.xhtml",
            "OEBPS/theme.css",
            "OEBPS/title.xhtml",
            "OEBPS/copyright.xhtml",
            "OEBPS/chapter-001.xhtml",
        ] {
            assert!(names.contains(&required), "missing {required} in {names:?}");
        }
    }

    #[test]
    fn includes_cover_when_present() {
        let mut pkg = minimal_pkg();
        pkg.cover = Some(CoverFiles {
            xhtml: "<html>cover</html>".into(),
            image_bytes: &[0xff, 0xd8, 0xff],
        });
        let mut buf = Cursor::new(Vec::<u8>::new());
        pack(&pkg, &mut buf).unwrap();
        let archive = read_archive(buf.into_inner());
        let names: Vec<&str> = archive.file_names().collect();
        assert!(names.contains(&"OEBPS/cover.xhtml"));
        assert!(names.contains(&"OEBPS/cover.jpg"));
    }

    #[test]
    fn omits_cover_when_absent() {
        let mut buf = Cursor::new(Vec::<u8>::new());
        pack(&minimal_pkg(), &mut buf).unwrap();
        let archive = read_archive(buf.into_inner());
        let names: Vec<&str> = archive.file_names().collect();
        assert!(!names.contains(&"OEBPS/cover.xhtml"));
        assert!(!names.contains(&"OEBPS/cover.jpg"));
    }
}
