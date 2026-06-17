//! `papyrust build <format>` — produces output files from the project.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::ir::Book;
use crate::validate::{Report, load_project};
use anyhow::{Context, Result};

use crate::commands::{print_report, slug};

pub fn epub(root: &Path) -> Result<ExitCode> {
    let book = load_validated(root)?;
    let output_path = output_path(root, &book, "epub", None)?;
    crate::epub::render(&book, &output_path)
        .with_context(|| format!("rendering {}", output_path.display()))?;
    println!("wrote {}", output_path.display());
    Ok(ExitCode::SUCCESS)
}

pub fn pdf(root: &Path) -> Result<ExitCode> {
    let book = load_validated(root)?;
    let trim_slug = book.meta.trim.slug();
    let output_path = output_path(root, &book, "pdf", Some(trim_slug))?;
    crate::pdf::render(&book, &output_path)
        .with_context(|| format!("rendering {}", output_path.display()))?;
    println!("wrote {}", output_path.display());
    Ok(ExitCode::SUCCESS)
}

pub fn all(root: &Path) -> Result<ExitCode> {
    let book = load_validated(root)?;
    let epub_path = output_path(root, &book, "epub", None)?;
    crate::epub::render(&book, &epub_path)
        .with_context(|| format!("rendering {}", epub_path.display()))?;
    println!("wrote {}", epub_path.display());

    let trim_slug = book.meta.trim.slug();
    let pdf_path = output_path(root, &book, "pdf", Some(trim_slug))?;
    crate::pdf::render(&book, &pdf_path)
        .with_context(|| format!("rendering {}", pdf_path.display()))?;
    println!("wrote {}", pdf_path.display());

    Ok(ExitCode::SUCCESS)
}

fn load_validated(root: &Path) -> Result<Book> {
    let (book, report): (Option<Book>, Report) = load_project(root);
    print_report(&report);
    book.ok_or_else(|| anyhow::anyhow!("project has validation errors; cannot build"))
}

fn output_path(root: &Path, book: &Book, ext: &str, suffix: Option<&str>) -> Result<PathBuf> {
    let dir = root.join("build");
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let stem = match suffix {
        Some(s) => format!("{}-{s}", slug(&book.meta.title)),
        None => slug(&book.meta.title),
    };
    Ok(dir.join(format!("{stem}.{ext}")))
}
