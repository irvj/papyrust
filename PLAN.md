# papyrust — Plan

> Living design document. Update as scope evolves.
> Last updated: 2026-06-17 (planning session, pre-code)

## What this is

`papyrust` is a Rust CLI that turns a folder of Markdown into a publication-quality EPUB and a print-ready PDF, with opinionated trade-press typography. Single static binary, no runtime dependencies.

Built for writers who want to own their toolchain (no subscription risk) and prefer a CLI workflow over a GUI.

## Goals

- Beautiful default output, indistinguishable from a real trade-press book
- One static binary, installable via `cargo install`, Homebrew, or GitHub releases
- Zero runtime configuration for the common case — opinionated defaults
- Outputs that upload cleanly to KDP (primary), with reasonable compatibility elsewhere
- Idiomatic, simple, well-separated Rust that other contributors can read and extend

## Non-goals (v1)

- Flexible templating / per-book theme system
- Images inside chapters
- Hardcover trim sizes
- Print cover PDF generation (KDP wants a separate cover anyway)
- Input formats other than Markdown
- GUI

## Tech stack (locked)

| Concern | Choice | Rationale |
|---|---|---|
| Language | Rust | Static binary, no runtime |
| Print PDF | Embedded [Typst](https://typst.app) (`typst` crate) | Modern typesetting, Rust-native, beautiful output |
| EPUB | Built directly (zipped XHTML + OPF); `epub-builder` as fallback | EPUB is simple; full control over output |
| Markdown | `pulldown-cmark` | Standard, fast, event-based |
| Config | TOML (`serde` + `toml`) | Forgiving for non-technical writers |
| CLI | `clap` (derive macros) | De facto standard |
| Errors | `thiserror` (libs) + `anyhow` (CLI) | Typed library errors, ergonomic CLI errors |
| Fonts | EB Garamond bundled in binary (OFL) | Single-binary promise |

### Risks on tech choices

1. **Typst-as-library** — embedded API is less polished than the CLI. Fallback: bundle the `typst` binary via `include_bytes!` + temp extraction, or shell out. Either still preserves the static-binary feel.
2. **EPUB validators are picky** — wire `epubcheck` into CI early so we catch malformed output before shipping.
3. **Font licensing** — EB Garamond is OFL; document the license inclusion clearly.

## Architecture

Strict separation of concerns. Each crate has one job and depends only on what it must.

```
crates/
├── papyrust          # binary crate; clap entry; argv → commands → calls into core/epub/pdf
├── papyrust-core     # Book IR, MD parsing, validation, book.toml schema
├── papyrust-epub     # Book IR → .epub
└── papyrust-pdf      # Book IR → .pdf (via Typst)

assets/
├── fonts/EBGaramond/      # bundled (OFL)
├── ornaments/             # SVG/glyph for scene breaks
└── templates/             # Typst + EPUB CSS templates

examples/sample-book/      # fixture used by tests + demos
```

**Dependency rules:**
- `papyrust-core` depends on no other workspace crate.
- `papyrust-epub` and `papyrust-pdf` depend on `papyrust-core`. They do not know about each other.
- `papyrust` (the binary crate) depends on all three and contains no business logic — only argument parsing, IO orchestration, and user-facing error reporting.

### Rendering pipeline

```
MD files ──► pulldown-cmark events
                ▼
        Book IR (chapters w/ title + structured blocks)
                ▼
    ┌───────────┴───────────┐
    ▼                       ▼
 XHTML renderer        Typst renderer
    │                       │
 EPUB packager          typst::compile
    │                       │
 .epub                   .pdf
```

The **Book IR** is the contract between parsing and rendering. Both backends consume it; neither modifies it.

## User-facing project layout

A `papyrust` book project on disk:

```
my-novel/
├── book.toml
├── cover.jpg                    # required for EPUB
├── front-matter/                # optional; custom user-written
│   ├── 01-dedication.md
│   └── 02-epigraph.md
├── chapters/                    # required; at least one
│   ├── 01-chapter.md
│   ├── 02-chapter.md
│   └── ...
└── back-matter/                 # optional
    └── 01-about-author.md
```

**Auto-generated pages** (never user files): title page, copyright page, table of contents. They are injected from `book.toml`. User-written front-matter files slot in *after* the auto-generated pages.

**Chapter ordering:** by filename prefix (`01-`, `02-`, ...).
**Chapter title:** extracted from the first `# H1` in the file. Validation fails if absent.

## `book.toml` schema (v1)

```toml
[book]
title = "The Long Road Home"
subtitle = "A Novel"
author = "Jane Doe"
language = "en-US"

[copyright]
year = 2026
holder = "Jane Doe"
isbn_epub = "978-0-000-00000-0"
isbn_print = "978-0-000-00000-1"
publisher = "Self-Published"

[print]
trim = "6x9"                      # "5x8" | "5.5x8.5" | "6x9"

[ebook]
# reserved for future options
```

Unknown keys → validation warning (not error), to allow forward compatibility.

## CLI

```
papyrust init <name>              # scaffold a new project with sample chapter
papyrust validate                 # lint book.toml + chapter structure
papyrust build epub               # → build/<slug>.epub
papyrust build pdf                # → build/<slug>-<trim>.pdf
papyrust build all                # both
```

All commands run from the book project root unless `--path` is given.

## Opinionated typography

### Print PDF (Typst)
- Body: EB Garamond 11pt, leading 1.35
- Chapter headings: EB Garamond small caps, centered, ~⅓ down the page
- Drop cap on first paragraph of each chapter (3 lines)
- No indent on first paragraph after a heading or scene break; indent elsewhere
- Running heads: book title verso (left), chapter title recto (right)
- Page numbers: roman in front matter, arabic in body, suppressed on chapter-opening and blank pages
- Chapters start on recto; blank verso inserted if needed
- Scene break: centered ornament (✦) with vertical breathing room
- Widows/orphans control set to 2

### EPUB
- Serif body (EB Garamond with fallback chain)
- Small-caps headings, drop cap (with reflow-safe fallback)
- Centered ornament for scene breaks
- Cover image embedded

## Milestones

### M1 — Skeleton + IR
- Cargo workspace with the four crates
- `clap` CLI; `papyrust init <name>` scaffolds a working sample project
- `book.toml` parsing with `serde`
- Directory walker → ordered chapter list
- MD → Book IR via `pulldown-cmark`
- `papyrust validate` catches: missing H1, malformed TOML, missing cover, missing chapters
- CI: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`

### M2 — EPUB
- XHTML renderer per chapter (consumes Book IR)
- Auto-generated title / copyright / nav pages
- Cover embedding
- Opinionated CSS theme
- `papyrust build epub` produces a valid EPUB 3
- `epubcheck` wired into CI against the sample book

### M3 — Print PDF via Typst
- Embed `typst` crate; implement `World` trait with bundled fonts
- Typst template realizing the trade-press look
- Trim size selection wired from `book.toml`
- Auto front matter + TOC, drop caps, running heads, page-numbering rules, ornament
- `papyrust build pdf`

### M4 — Polish + distribution
- `papyrust build all`
- Polished error messages, colorized output
- GitHub Actions: cross-compile for macOS (arm64/x64), Linux (x64/arm64), Windows; attach binaries to releases
- Publish to crates.io
- Homebrew tap
- README with screenshots from the sample book

## Deferred (v2+)

- Configurable themes / custom CSS / custom Typst templates
- Image support in chapters
- Hardcover trim sizes
- Print cover PDF generation
- Additional input formats (DOCX, plain text)
- Per-distributor profiles (Kobo, Apple Books quirks)
- Live preview server
- Word/chapter statistics command

## Quality bar

Hard rules (CI-enforced where possible):

- `#![deny(unsafe_code)]` on every crate unless a specific reason is documented at the attribute
- `cargo fmt --check` and `cargo clippy -- -D warnings -W clippy::pedantic` (with explicit allows where pedantic is wrong, never blanket)
- Typed errors (`thiserror`) in library crates; no stringly-typed errors
- No `unwrap()` / `expect()` in library code except in tests or with a `// SAFETY:`-style comment justifying invariant
- Validate at boundaries (file IO, `book.toml`, MD input). Trust internal data after that.
- No premature abstraction. Three similar lines is fine. Generic helpers earn their existence by being needed twice.

## Open questions

- Should `papyrust init` accept a `--minimal` flag for a single empty chapter vs. a richer sample? *(Probably yes after M1 ships.)*
- Should `validate` have a `--fix` mode for trivial issues (missing prefixes, etc.)? *(Defer.)*
- Do we ship the EPUB CSS as one bundled file or split per-concern? *(Decide when writing M2.)*
