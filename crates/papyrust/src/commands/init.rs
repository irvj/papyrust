//! `papyrust init <path>` — scaffold a new book project.

use std::path::Path;
use std::process::ExitCode;

use anyhow::{Context, Result, bail};

const BOOK_TOML: &str = r#"[book]
title = "Untitled Book"
# subtitle = "An Optional Subtitle"
author = "Your Name"
language = "en-US"

[copyright]
year = 2026   # update to current year
holder = "Your Name"
# isbn_epub = "..."
# isbn_print = "..."
publisher = "Self-Published"

[print]
trim = "6x9"   # also supported: "5x8", "5.5x8.5"
"#;

const SAMPLE_CHAPTER: &str = r"# Chapter One

It was the best of times, it was the worst of times. The morning came
slowly, as mornings sometimes do, with a reluctance bordering on apology.

She watched the light spill across the floorboards and thought about
everything she had not yet decided to do.

---

By evening the question had answered itself, as questions sometimes will
when you stop asking them.
";

const SAMPLE_DEDICATION: &str = "For the readers.\n";

const SAMPLE_ABOUT: &str = r"# About the Author

Your Name is the author of *Untitled Book*. They live somewhere and write
things, when there is time.
";

pub fn run(target: &Path) -> Result<ExitCode> {
    if target.exists() {
        bail!("path already exists: {}", target.display());
    }
    std::fs::create_dir_all(target)
        .with_context(|| format!("creating project root {}", target.display()))?;

    write_file(target, "book.toml", BOOK_TOML)?;
    write_file(target, "chapters/01-chapter.md", SAMPLE_CHAPTER)?;
    write_file(target, "front-matter/01-dedication.md", SAMPLE_DEDICATION)?;
    write_file(target, "back-matter/01-about-author.md", SAMPLE_ABOUT)?;

    let display = target.display();
    println!("Created project at {display}");
    println!();
    println!("Next steps:");
    println!("  1. Add a cover image at {display}/cover.jpg");
    println!("  2. Edit {display}/book.toml");
    println!("  3. Replace the sample chapter in {display}/chapters/");
    println!("  4. Run `papyrust --path {display} validate`");
    Ok(ExitCode::SUCCESS)
}

fn write_file(root: &Path, rel: &str, contents: &str) -> Result<()> {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    std::fs::write(&path, contents).with_context(|| format!("writing {}", path.display()))
}
