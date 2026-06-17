# CLAUDE.md

Instructions for Claude Code when working in this repo. Read `PLAN.md` for the full design and current status.

## What this project is

`papyrust` — a Rust CLI that builds publication-quality EPUB and print-ready PDF from a folder of Markdown. Open source, single static binary, opinionated typography. See `PLAN.md`.

Do not name competitor or "replaced" software in any committed file (README, PLAN.md, CLAUDE.md, commit messages). Describe the project on its own terms.

## Current status (read first)

- **M1 + M2 + M3 (including print-typography polish) are complete and committed on `main`.**
- **The crate is now a single flat package** (`papyrust-cli`) at the repo root — the original four-crate workspace was flattened so we can publish one crate to crates.io.
- The end-to-end pipeline works: `papyrust init <path>` → `papyrust validate` → `papyrust build epub|pdf|all` produces shippable EPUB 3 and print-ready PDF.
- **M4 (releases + distribution) is in progress** — versioning policy and `v0.1.0` tag in place; crate-name decision locked (`papyrust-cli` on crates.io, binary stays `papyrust`); flat layout ready for publishing.
- 90 unit tests; CI gates are fmt + clippy `-D warnings` + tests + `epubcheck` on a sample EPUB.

If asked to resume work, the most likely real tasks are:
1. **Adjustments based on visual feedback** on the PDF or EPUB (typography tweaks, ornament changes, page layout).
2. **Items from `PLAN.md` § "Known gaps from spec"** — e.g. true floating drop cap, widows/orphans in Typst, leading tweak.
3. **Continuing M4 work** — actual `cargo publish`, GitHub Releases workflow, Homebrew tap.

Confirm scope before starting any of these.

## Workflow

- **Start every session by skimming `PLAN.md`.** Locked decisions and current status live there; do not relitigate them without the user asking.
- **Update `PLAN.md` when scope changes.** If a decision changes during a session, update the relevant section in the same commit that implements the change.
- **Milestones and status in `PLAN.md`.** Check progress and the "Known gaps" section before assuming something was or wasn't done.

## Rust quality bar (non-negotiable)

These apply to every line of code in this repo:

1. **Simplicity over cleverness.** Prefer the obvious approach. No premature abstraction. Three similar lines beats a generic helper that earns its keep only once.
2. **Cohesive modules.** The `epub` and `pdf` modules consume `crate::ir`, `crate::config`, etc. but should never depend on each other. The `commands` module is a thin shell over those: argument parsing, IO orchestration, error formatting. Treat module boundaries with care even though the compiler no longer enforces them at the crate level.
3. **Safety:** `#![deny(unsafe_code)]` at the crate root unless a documented reason exists.
4. **Typed errors:** `thiserror` for module-level error enums (`EpubError`, `PdfError`, etc.); `anyhow` at the CLI surface. No stringly-typed errors.
5. **No `unwrap()`/`expect()` in non-test code** except with a justifying comment.
6. **Validate at boundaries** (file IO, `book.toml`, Markdown). Trust internal data after that.
7. **CI gates:** `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` must pass.

## Things not to relitigate

These are locked. Don't re-propose alternatives unless the user opens the question:

- Language: Rust (edition 2024)
- Print PDF engine: Typst 0.14, embedded as a library
- EPUB: built directly (zipped XHTML + OPF), no heavy framework
- Config format: TOML
- Manuscript format: one Markdown file per chapter, ordered by numeric prefix
- Project layout: `front-matter/`, `chapters/`, `back-matter/`, `cover.jpg`, `book.toml`
- Chapter title source: the file's first `# H1`
- Front matter strategy: hybrid — title/copyright/TOC auto-generated, user MD slots in after
- Trim sizes v1: 5x8, 5.5x8.5, 6x9
- Body font: EB Garamond (bundled, OFL, variable Regular + Italic)
- Scene break ornament: `* * *` (three asterisks with tracking), same in EPUB and PDF
- Drop cap in PDF is a "raised cap" (large first letter), not a floating drop cap (Typst lacks text wrap)
- Page-numbering scheme: auto pages unnumbered, user front matter roman, body/back arabic, suppressed on chapter-opening pages
- Running heads: book title verso, chapter title recto, suppressed on chapter-opening pages
- Chapters start on recto via `pagebreak(weak: true, to: "odd")`
- v1 is opinionated only — no user-facing templating system
- No images in chapters in v1
- No print cover generation in v1
- Binary name: `papyrust`
- **Crates.io name: `papyrust-cli`** (the bare `papyrust` is taken by an unrelated dormant 2022 script runner; the binary is still installed as `papyrust`)
- **Single-crate layout.** The original four-crate workspace (`papyrust`, `papyrust-core`, `papyrust-epub`, `papyrust-pdf`) was flattened into one crate at the repo root before first publish. Don't re-suggest a workspace.

## Where things live (orientation)

- `src/main.rs` — CLI entry, clap setup, dispatch
- `src/commands/` — `init.rs`, `validate.rs`, `build.rs`, `mod.rs` (shared helpers)
- `src/config.rs`, `src/ir.rs`, `src/parse.rs`, `src/project.rs`, `src/validate.rs` — core types + loaders (the former `papyrust-core`)
- `src/epub/` — EPUB renderer: `mod.rs` (public `render` + `EpubError`), `archive`, `escape`, `nav`, `opf`, `pages`, `paths`, `xhtml`, `theme.css`
- `src/pdf/` — PDF renderer: `mod.rs` (public `render` + `PdfError`), `world.rs` (typst::World impl), `source.rs` (Book IR → Typst source generator)
- `fonts/` — EB Garamond variable TTFs + `OFL.txt`
- `examples/book.toml` — annotated reference for the `book.toml` schema
- `.github/workflows/ci.yml` — fmt + clippy + test job, plus a separate `epubcheck` job
- `rust-toolchain.toml` — pins to 1.95.0 to match CI

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
- Re-proposing the four-crate workspace structure — we deliberately flattened it
- Starting M4 work without explicit confirmation

## Commit conventions

- Terse, lowercase commit messages (e.g., `m3 polish: recto starts, page numbers, running heads, raised cap`).
- No `Co-Authored-By: Claude ...` trailer.
- Stage specific paths (not `-A`/`.`), but using a small set of top-level dirs (e.g., `git add src .github README.md`) is fine.

## Versioning

Single source of truth: `[package].version` in the root `Cargo.toml`. `papyrust --version` displays it automatically via `CARGO_PKG_VERSION`.

When changes warrant a bump (see `PLAN.md` § Versioning for the policy):

1. Edit `version` in `[package]`.
2. Add a new dated section at the top of `CHANGELOG.md` (above any older releases). Move accumulated `## [Unreleased]` entries into it.
3. Commit with message `release: vX.Y.Z`.
4. `git tag vX.Y.Z && git push --tags`.

When making any user-visible change between releases, append a bullet under `## [Unreleased]` in `CHANGELOG.md` in the same commit. That way, when we bump, the changelog entry is already drafted.
