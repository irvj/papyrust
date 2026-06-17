# papyrust — Plan

> Living design document. Update as scope evolves.
> Last updated: 2026-06-17 (after M3 polish, before M4)

## Current state at a glance

- **M1, M2, M3 complete** including M3 print-typography polish.
- **End-to-end pipeline works**: `papyrust init` → `papyrust validate` → `papyrust build epub|pdf|all` produces shippable EPUB 3 and print-ready PDF.
- **M4 (release + distribution) deferred** while the author tests the output on real manuscripts.
- 87 unit tests across the workspace; CI runs fmt + clippy (`-D warnings`) + tests + `epubcheck` on a sample build.

What works today (matches the typography spec below unless flagged "→ note"):

- Trim sizes 5×8, 5.5×8.5, 6×9
- EB Garamond bundled, used for both PDF body text and EPUB CSS preference
- Chapter starts on recto, with blank verso inserted as needed
- Running heads (book title verso, chapter title recto), suppressed on chapter-opening pages
- Arabic page numbers, centered footer, suppressed on chapter-opening pages
- Roman page numbers for user-written front-matter pages (auto title/copyright/TOC stay unnumbered)
- Raised cap on the first alphabetic character of each chapter's first paragraph → note: not a true floating drop cap; Typst lacks text wrap
- Scene break: three centered asterisks with tracking, identical in EPUB and PDF
- EPUB CSS drop cap via `::first-letter` (reader-dependent)
- Cover image embedded in EPUB; cover is omitted gracefully when missing

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
| Language | Rust (edition 2024) | Static binary, no runtime |
| Print PDF | Embedded [Typst](https://typst.app) (`typst` 0.14 + `typst-pdf` 0.14) | Modern typesetting, Rust-native, beautiful output |
| EPUB | Built directly (zipped XHTML + OPF); `zip` + `uuid` (v5) + `time` | EPUB is simple; full control over output |
| Markdown | `pulldown-cmark` 0.13 | Standard, fast, event-based |
| Config | TOML (`serde` + `toml`) | Forgiving for non-technical writers |
| CLI | `clap` 4 (derive macros) | De facto standard |
| Errors | `thiserror` (libs) + `anyhow` (CLI) | Typed library errors, ergonomic CLI errors |
| Fonts | EB Garamond variable (Regular + Italic) bundled in binary (OFL) | Single-binary promise |

### Risks on tech choices

1. **Typst-as-library** — embedded API is less polished than the CLI. So far it has worked cleanly for our use case at 0.14.2. Fallback if it bites later: bundle the `typst` binary via `include_bytes!` + temp extraction.
2. **EPUB validators are picky** — `epubcheck` is wired into CI against a sample build, run via Java in a separate workflow job.
3. **Font licensing** — EB Garamond is OFL; the license is shipped at `crates/papyrust-pdf/fonts/OFL.txt`.

## Architecture

Strict separation of concerns. Each crate has one job and depends only on what it must.

```
crates/
├── papyrust          # binary crate; clap entry; argv → commands → calls into core/epub/pdf
├── papyrust-core     # Book IR, MD parsing, validation, book.toml schema
├── papyrust-epub     # Book IR → .epub (XHTML, OPF, nav, archive)
└── papyrust-pdf      # Book IR → .pdf (via Typst): source generator + World

crates/papyrust-epub/src/theme.css          # embedded EPUB CSS
crates/papyrust-pdf/fonts/                  # bundled EB Garamond + OFL.txt
```

There is no separate top-level `assets/` directory; each renderer carries its own static resources inside its crate so the crate is self-contained.

**Dependency rules:**
- `papyrust-core` depends on no other workspace crate.
- `papyrust-epub` and `papyrust-pdf` depend on `papyrust-core`. They do not know about each other.
- `papyrust` (the binary crate) depends on all three and contains no business logic — only argument parsing, IO orchestration, and user-facing error reporting.

### Rendering pipeline

```
MD files ──► pulldown-cmark events
                ▼
        Book IR (chapters w/ title + structured blocks + cover bytes)
                ▼
    ┌───────────┴───────────┐
    ▼                       ▼
 XHTML renderer        Typst source generator
    │                       │
 EPUB packager          typst::compile (PagedDocument)
    │                       │
 .epub                   typst_pdf::pdf → .pdf
```

The **Book IR** is the contract between parsing and rendering. Both backends consume it; neither modifies it.

## User-facing project layout

A `papyrust` book project on disk:

```
my-novel/
├── book.toml
├── cover.jpg                    # required for EPUB; PDF works without
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

**Chapter ordering:** by filename prefix (`01-`, `02-`, ...). Unprefixed files sort last.
**Chapter title:** extracted from the first `# H1` in the file. Validation fails if absent.

## `book.toml` schema (v1)

```toml
[book]
title = "The Long Road Home"
subtitle = "A Novel"           # optional
author = "Jane Doe"
language = "en-US"             # BCP 47; stripped to ISO 639 for Typst

[copyright]
year = 2026
holder = "Jane Doe"
isbn_epub = "978-0-000-00000-0"   # optional
isbn_print = "978-0-000-00000-1"  # optional
publisher = "Self-Published"

[print]
trim = "6x9"                      # "5x8" | "5.5x8.5" | "6x9"

[ebook]
# reserved for future options
```

Unknown keys are silently ignored (forward-compat). Adding warnings on unknowns is a future polish item.

## CLI

```
papyrust init <path>              # scaffold a new project at <path>
papyrust validate                 # lint book.toml + chapter structure
papyrust build epub               # → build/<slug>.epub
papyrust build pdf                # → build/<slug>-<trim>.pdf
papyrust build all                # both
```

Global `--path <dir>` overrides the project root for `validate` and `build`. `init` ignores it (it always creates at the path argument).

## Opinionated typography

### Print PDF (Typst, as implemented today)
- Body: EB Garamond, 11pt, leading 0.65em (≈ 1.65× line height), justified, hyphenation on
- Page margins: top 0.75in, bottom 0.75in, inside 0.875in, outside 0.75in (suitable for KDP perfect-bind)
- Chapter headings: small caps, centered, ~1.5in down the page, 1.6em size, 0.1em tracking, regular weight
- First-line indent: 1.5em on all paragraphs *except* immediately after a heading or scene break (Typst's `first-line-indent: (amount: 1.5em, all: false)`)
- Raised cap on first alphabetic character of each chapter's first paragraph (2.2em, 0.05em tracking) → note: not a floating drop cap; Typst has no native text wrap
- Running heads: book title verso (small caps, left, 0.85em, 0.1em tracking), chapter title recto (small caps, right, same metrics), suppressed on chapter-opening pages
- Page numbers: arabic in body and back matter, centered in footer, 0.85em, suppressed on chapter-opening pages; roman for user front-matter pages; auto title/copyright/TOC unnumbered
- Chapters start on recto via `pagebreak(weak: true, to: "odd")`
- Scene break: centered `* * *` with 0.5em tracking and ~0.7em vertical breathing room above/below
- Widows/orphans: not explicitly set in Typst — relying on its defaults

### EPUB CSS
- Serif body chain: `"EB Garamond", Garamond, Georgia, serif` (reader chooses)
- Justified text with `text-indent: 1.5em`, suppressed after headings and scene breaks
- Small-caps headings, centered, tracking
- Drop cap via `.first-paragraph::first-letter` (reader-dependent fidelity)
- Centered ornament for scene breaks with `letter-spacing: 0.6em`
- Cover image embedded; title/subtitle/author rendered on the title page

## Milestones

### M1 — Skeleton + IR — **done**
Cargo workspace, four crates, `clap` CLI, `book.toml` parser, directory walker, MD → Book IR via pulldown-cmark, `papyrust validate` and `papyrust init`, CI gates.

### M2 — EPUB — **done**
XHTML renderer, auto cover/title/copyright/TOC pages, `nav.xhtml`, `content.opf` with DC metadata + UUID v5 identifier + `dcterms:modified`, ZIP packaging with `mimetype` stored-and-first, embedded opinionated CSS theme, `papyrust build epub` produces valid EPUB 3, `epubcheck` job in CI.

### M3 — Print PDF via Typst — **done**
EB Garamond bundled, `typst::World` implementation, Book IR → Typst source generator, `papyrust build pdf`, build-all wiring, chapters start on recto, page numbering (roman/arabic) with chapter-opening suppression, running heads, raised cap, three-asterisk scene break.

### M4 — Polish + distribution — **deferred**
Awaiting user testing of M3 output on real manuscripts before opening this milestone. Scope when we get to it:
- Polished error messages, colorized output
- GitHub Actions: cross-compile for macOS (arm64/x64), Linux (x64/arm64), Windows; attach binaries to releases
- Publish to crates.io as `papyrust-cli` (see "Crates.io naming" below)
- Homebrew tap
- README screenshots from a sample book
- **Ship the OFL notice with binary releases** — either bundle `OFL.txt` alongside the binary in the release tarball, or embed it via `include_str!` and expose it via a `papyrust licenses` subcommand. The source distribution already ships the file at `crates/papyrust-pdf/fonts/OFL.txt`; binary releases need their own copy so the notice stays "easily viewable by the user" per OFL §2.

#### Crates.io naming (locked)

The crate name `papyrust` is already taken on crates.io by an unrelated dormant project (a Rust script runner published in 2022, ~2,683 downloads, 2 GitHub stars, last activity 4+ years ago). The two domains don't overlap, so the only friction is the install command. We publish under a different name:

- **Crate name on crates.io:** `papyrust-cli`
- **Install command:** `cargo install papyrust-cli`
- **Binary name on disk:** `papyrust` (unchanged)
- **Repository, README, Homebrew formula, GitHub Releases:** all still `papyrust`

**We do not publish the library crates** (`papyrust-core`, `papyrust-epub`, `papyrust-pdf`) — they're workspace-internal implementation. Publishing them would create an implicit library API commitment we don't want yet. Mark each with `publish = false` in its `Cargo.toml` before the first `cargo publish` of the CLI crate, so accidental publishing is blocked.

When publishing `papyrust-cli`, Cargo bundles the workspace `path` dependencies into the source tarball automatically; users `cargo install papyrust-cli` and the build works without those sub-crates being on crates.io.

## Known gaps from spec (revisit during M4 or v2)

- **True floating drop cap** — Typst has no native text wrap. We ship a raised cap as the substitute. A real floating drop cap would need either the Typst Universe `dropcap` package (requires file resolution in our `World`) or a custom `measure + place + pad` implementation that handles the line-wrap manually.
- **Widows/orphans** in print — relying on Typst defaults. Worth setting explicitly.
- **Leading** — we ship 0.65em (≈ 1.65×); the original spec said 1.35. Easy to tune in `write_preamble`.
- **EPUB drop cap fidelity** — `::first-letter` support varies by reader. Acceptable for v1.
- **Unknown-key warnings in `book.toml`** — currently silently ignored. Spec called for warnings.

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
- `cargo fmt --check` and `cargo clippy -- -D warnings` with workspace lints enabling `clippy::pedantic` (explicit allows only at well-justified sites, never blanket)
- Typed errors (`thiserror`) in library crates; no stringly-typed errors
- No `unwrap()` / `expect()` in library code except in tests or with a justifying comment
- Validate at boundaries (file IO, `book.toml`, MD input). Trust internal data after that.
- No premature abstraction. Three similar lines is fine. Generic helpers earn their existence by being needed twice.

## Open questions

- Should `papyrust init` accept a `--minimal` flag for a single empty chapter vs. the richer sample? *(Defer until a real user asks.)*
- Should `validate` have a `--fix` mode for trivial issues (missing prefixes, etc.)? *(Defer.)*
- For M4, should the GitHub Release flow build binaries for all four targets, or start with macOS-arm64 + Linux-x86_64 and add the rest after first user feedback? *(Decide when starting M4.)*
