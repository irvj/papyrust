use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod commands;
mod config;
mod epub;
mod ir;
mod parse;
mod pdf;
mod project;
mod validate;

#[derive(Parser, Debug)]
#[command(
    name = "papyrust",
    version,
    about = "Build publication-quality EPUB and print-ready PDF from a folder of Markdown."
)]
struct Cli {
    /// Project root for `validate` and `build` (defaults to the current directory).
    /// Ignored by `init`.
    #[arg(long, global = true)]
    path: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Scaffold a new book project at the given path.
    Init {
        /// Directory to create. Must not already exist.
        path: PathBuf,
    },
    /// Check the project for problems.
    Validate,
    /// Build output(s) from the project.
    Build {
        #[command(subcommand)]
        format: BuildFormat,
    },
}

#[derive(Subcommand, Debug)]
enum BuildFormat {
    /// Build the EPUB (ebook).
    Epub,
    /// Build the print-ready PDF.
    Pdf,
    /// Build both EPUB and PDF.
    All,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = run(cli);
    match result {
        Ok(code) => code,
        Err(err) => {
            eprintln!("papyrust: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> anyhow::Result<ExitCode> {
    match cli.command {
        Command::Init { path } => commands::init::run(&path),
        Command::Validate => Ok(commands::validate::run(&project_root(cli.path)?)),
        Command::Build {
            format: BuildFormat::Epub,
        } => commands::build::epub(&project_root(cli.path)?),
        Command::Build {
            format: BuildFormat::Pdf,
        } => commands::build::pdf(&project_root(cli.path)?),
        Command::Build {
            format: BuildFormat::All,
        } => commands::build::all(&project_root(cli.path)?),
    }
}

fn project_root(explicit: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    if let Some(p) = explicit {
        return Ok(p);
    }
    std::env::current_dir().map_err(|e| anyhow::anyhow!("could not determine current dir: {e}"))
}
