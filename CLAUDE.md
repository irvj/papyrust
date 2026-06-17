# CLAUDE.md

Instructions for Claude Code when working in this repo. Read `PLAN.md` for the full design.

## What this project is

`papyrust` — a Rust CLI that builds publication-quality EPUB and print-ready PDF from a folder of Markdown. Open source, single static binary, opinionated typography. See `PLAN.md`.

Do not name competitor or "replaced" software in any committed file (README, PLAN.md, CLAUDE.md, commit messages). Describe the project on its own terms.

## Workflow

- **Start every session by skimming `PLAN.md`.** Locked decisions live there; do not relitigate them without the user asking.
- **Update `PLAN.md` when scope changes.** If a decision changes during a session, update the relevant section in the same commit that implements the change.
- **Milestones are tracked in `PLAN.md`.** Check progress against M1–M4 there.

## Rust quality bar (non-negotiable)

These apply to every line of code in this repo:

1. **Simplicity over cleverness.** Prefer the obvious approach. No premature abstraction. Three similar lines beats a generic helper that earns its keep only once.
2. **Strict separation of concerns:**
   - `papyrust-core` knows nothing about EPUB or PDF.
   - `papyrust-epub` and `papyrust-pdf` depend on `papyrust-core` but never on each other.
   - `papyrust` (the binary crate at `crates/papyrust`) is a thin shell: argument parsing, IO orchestration, error formatting. No business logic.
3. **Safety:** `#![deny(unsafe_code)]` on every crate unless a documented reason exists.
4. **Typed errors:** `thiserror` in library crates, `anyhow` in the CLI. No `String` errors.
5. **No `unwrap()`/`expect()` in library code** except in tests or with a justifying comment.
6. **Validate at boundaries** (file IO, `book.toml`, Markdown). Trust internal data after that.
7. **CI gates:** `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` must pass.

## Things not to relitigate

These are locked. Don't re-propose alternatives unless the user opens the question:

- Language: Rust
- Print PDF engine: Typst (embedded as a library)
- EPUB: built directly, no heavy framework
- Config format: TOML
- Manuscript format: one Markdown file per chapter, ordered by numeric prefix
- Project layout: `front-matter/`, `chapters/`, `back-matter/`, `cover.jpg`, `book.toml`
- Chapter title source: the file's first `# H1`
- Front matter strategy: hybrid — title/copyright/TOC auto-generated, user MD slots in after
- Trim sizes v1: 5x8, 5.5x8.5, 6x9
- Body font: EB Garamond (bundled, OFL)
- v1 is opinionated only — no user-facing templating system
- No images in chapters in v1
- No print cover generation in v1
- Binary name: `papyrust`

## Things the user cares about

- Output that looks like a real trade-press book
- A workflow that's fast to use from the terminal
- Code that other contributors can read; this will be open-sourced
- KDP as the primary upload target

## Things to avoid

- Day/hour timeline estimates in plans (the user finds them unhelpful)
- Suggesting heavy frameworks "just in case"
- Adding configuration knobs for hypothetical future users
- Drive-by refactors that aren't part of the current task
