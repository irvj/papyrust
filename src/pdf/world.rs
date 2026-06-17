//! Minimal [`typst::World`] for compiling our generated Typst sources.
//!
//! The world is intentionally small: a single in-memory `main.typ`,
//! the bundled EB Garamond fonts, the Typst standard library, and a
//! fixed "today" so builds are deterministic.

use std::sync::OnceLock;

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};

const FONT_REGULAR: &[u8] = include_bytes!("../../fonts/EBGaramond-Variable.ttf");
const FONT_ITALIC: &[u8] = include_bytes!("../../fonts/EBGaramond-Italic-Variable.ttf");

static SHARED_LIBRARY: OnceLock<LazyHash<Library>> = OnceLock::new();
static SHARED_FONTS: OnceLock<(LazyHash<FontBook>, Vec<Font>)> = OnceLock::new();

pub struct PapyrustWorld {
    main: Source,
}

impl PapyrustWorld {
    pub fn new(source_text: String) -> Self {
        let id = FileId::new(None, VirtualPath::new("/main.typ"));
        Self {
            main: Source::new(id, source_text),
        }
    }
}

impl World for PapyrustWorld {
    fn library(&self) -> &LazyHash<Library> {
        SHARED_LIBRARY.get_or_init(|| LazyHash::new(Library::default()))
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &shared_fonts().0
    }

    fn main(&self) -> FileId {
        self.main.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            Ok(self.main.clone())
        } else {
            Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        shared_fonts().1.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        // Fixed for build reproducibility; PDF metadata still gets a
        // creation timestamp from typst-pdf at export time.
        Datetime::from_ymd(2026, 1, 1)
    }
}

fn shared_fonts() -> &'static (LazyHash<FontBook>, Vec<Font>) {
    SHARED_FONTS.get_or_init(|| {
        let fonts: Vec<Font> = Font::iter(Bytes::new(FONT_REGULAR))
            .chain(Font::iter(Bytes::new(FONT_ITALIC)))
            .collect();
        let book = LazyHash::new(FontBook::from_fonts(fonts.iter()));
        (book, fonts)
    })
}
