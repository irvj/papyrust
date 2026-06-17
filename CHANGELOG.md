# Changelog

All notable changes to papyrust are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning uses the cargo `0.x` convention while pre-1.0: the middle digit
bumps for breaking changes (incompatible `book.toml` / project layout /
CLI surface), the trailing digit bumps for everything else.

## [0.1.2] — 2026-06-17

### Changed
- Moved the static Typst (paragraph settings, scene-break and raised-cap
  helpers, heading show rules, running-head/page-number layout) out of the
  Rust string builder into `src/pdf/preamble.typ` and `src/pdf/body_layout.typ`,
  pulled in via `include_str!`. Internal refactor only — PDF output is
  unchanged.

## [0.1.1] — 2026-06-17

### Changed
- Flattened the four-crate workspace (`papyrust`, `papyrust-core`,
  `papyrust-epub`, `papyrust-pdf`) into a single crate at the repo
  root for publication as `papyrust-cli` on crates.io. No user-visible
  behavior change — output is byte-identical to v0.1.0.

## [0.1.0] — 2026-06-17

First tagged version. End-to-end pipeline works: scaffold a project,
validate it, build a valid EPUB 3 and a print-ready PDF.

### Added
- Cargo workspace with four crates: `papyrust` (CLI binary),
  `papyrust-core` (Book IR, parsing, validation), `papyrust-epub`
  (EPUB renderer), `papyrust-pdf` (Typst-based PDF renderer).
- `papyrust init <path>` — scaffold a new book project with a sample
  chapter, a dedication, an about-the-author page, and a `book.toml`.
- `papyrust validate` — lint `book.toml`, chapter structure, missing
  cover, etc. Returns a structured report with errors and warnings.
- `papyrust build epub` — produces a valid EPUB 3 with auto-generated
  cover/title/copyright/TOC pages, embedded cover image, and an
  opinionated CSS theme (serif body, small-caps headings, drop cap
  via `::first-letter`, centered scene-break ornament).
- `papyrust build pdf` — produces a print-ready PDF via embedded
  [Typst](https://typst.app). EB Garamond bundled in the binary.
  Trade-press typography: chapters start on the recto, running heads
  (book title verso, chapter title recto), arabic page numbers
  (suppressed on chapter-opening pages), raised cap on the first
  paragraph of each chapter, three-asterisk scene break.
- `papyrust build all` — both formats in one command.
- Trim sizes: `5x8`, `5.5x8.5`, `6x9`.
- Annotated reference `examples/book.toml` documenting every field.

### Quality
- 87 unit tests across the workspace.
- CI: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`,
  plus a separate `epubcheck` job that validates a sample build.
- Rust toolchain pinned at `1.95.0` via `rust-toolchain.toml` so
  rustfmt/clippy output matches between local and CI.

[0.1.2]: https://github.com/irvj/papyrust/releases/tag/v0.1.2
[0.1.1]: https://github.com/irvj/papyrust/releases/tag/v0.1.1
[0.1.0]: https://github.com/irvj/papyrust/releases/tag/v0.1.0
