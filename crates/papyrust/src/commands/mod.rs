pub mod build;
pub mod init;
pub mod validate;

use papyrust_core::validate::{Report, Severity};

/// Print each issue in a report to stderr in a uniform format.
/// Returns the (errors, warnings) counts for caller-side summaries.
pub fn print_report(report: &Report) -> (usize, usize) {
    for issue in &report.issues {
        let label = match issue.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        eprintln!("{label}: {}", issue.message);
    }
    (report.errors().count(), report.warnings().count())
}

/// Derive a filesystem-safe slug from a book title.
pub fn slug(s: &str) -> String {
    let mut out = String::new();
    let mut last_dash = true;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_end_matches('-').to_owned();
    if trimmed.is_empty() {
        "book".to_owned()
    } else {
        trimmed
    }
}
