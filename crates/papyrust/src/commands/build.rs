//! `papyrust build <format>` — produces output files from the project.

use std::path::Path;
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use papyrust_core::validate::load_project;

use crate::commands::{print_report, slug};

pub fn epub(root: &Path) -> Result<ExitCode> {
    let (book, report) = load_project(root);
    print_report(&report);
    let Some(book) = book else {
        bail!("project has validation errors; cannot build");
    };

    let output_dir = root.join("build");
    std::fs::create_dir_all(&output_dir)
        .with_context(|| format!("creating {}", output_dir.display()))?;
    let output_path = output_dir.join(format!("{}.epub", slug(&book.meta.title)));

    papyrust_epub::render(&book, &output_path)
        .with_context(|| format!("rendering {}", output_path.display()))?;
    println!("wrote {}", output_path.display());
    Ok(ExitCode::SUCCESS)
}

pub fn pdf(_root: &Path) -> Result<ExitCode> {
    bail!("`build pdf` is not implemented yet (planned for M3)");
}

pub fn all(_root: &Path) -> Result<ExitCode> {
    bail!("`build all` is not implemented yet (planned for M2/M3)");
}
