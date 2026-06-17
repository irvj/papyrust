//! `papyrust validate` — load the project and print issues.

use std::path::Path;
use std::process::ExitCode;

use papyrust_core::validate::{Severity, load_project};

pub fn run(root: &Path) -> ExitCode {
    let (_book, report) = load_project(root);

    for issue in &report.issues {
        let label = match issue.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        eprintln!("{label}: {}", issue.message);
    }

    let errors = report.errors().count();
    let warnings = report.warnings().count();

    if report.has_errors() {
        eprintln!();
        eprintln!("validation failed: {errors} error(s), {warnings} warning(s)");
        return ExitCode::FAILURE;
    }

    if warnings > 0 {
        eprintln!();
        eprintln!("validation passed with {warnings} warning(s)");
    } else {
        println!("validation passed");
    }
    ExitCode::SUCCESS
}
