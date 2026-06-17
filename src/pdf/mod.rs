//! Print PDF renderer for papyrust (via embedded Typst).
//!
//! Consumes a [`crate::ir::Book`] and produces a print-ready PDF.

use std::path::Path;

use crate::ir::Book;
use typst::diag::SourceDiagnostic;
use typst::layout::PagedDocument;

mod source;
mod world;

#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("typst compile failed:\n{0}")]
    Compile(String),

    #[error("typst pdf export failed:\n{0}")]
    Export(String),
}

/// Compile the book to a print-ready PDF at `output`.
pub fn render(book: &Book, output: &Path) -> Result<(), PdfError> {
    let typst_source = source::build(book);
    let world = world::PapyrustWorld::new(typst_source);

    let warned = typst::compile::<PagedDocument>(&world);
    let document = warned
        .output
        .map_err(|errs| PdfError::Compile(format_diagnostics(errs.iter())))?;

    let options = typst_pdf::PdfOptions::default();
    let pdf_bytes = typst_pdf::pdf(&document, &options)
        .map_err(|errs| PdfError::Export(format_diagnostics(errs.iter())))?;

    std::fs::write(output, pdf_bytes)?;
    Ok(())
}

fn format_diagnostics<'a>(diags: impl Iterator<Item = &'a SourceDiagnostic>) -> String {
    diags
        .map(|d| d.message.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}
