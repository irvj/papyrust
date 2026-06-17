use std::path::Path;
use std::process::ExitCode;

use crate::validate::load_project;

use crate::commands::print_report;

pub fn run(root: &Path) -> ExitCode {
    let (_book, report) = load_project(root);
    let (errors, warnings) = print_report(&report);

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
