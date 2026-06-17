//! `papyrust build <format>` — stubs until renderers land in M2/M3.

use std::path::Path;
use std::process::ExitCode;

use anyhow::{Result, bail};

pub fn epub(_root: &Path) -> Result<ExitCode> {
    bail!("`build epub` is not implemented yet (planned for M2)");
}

pub fn pdf(_root: &Path) -> Result<ExitCode> {
    bail!("`build pdf` is not implemented yet (planned for M3)");
}

pub fn all(_root: &Path) -> Result<ExitCode> {
    bail!("`build all` is not implemented yet (planned for M2/M3)");
}
