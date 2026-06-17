# papyrust

Build publication-quality EPUB and print-ready PDF from a folder of Markdown.

A CLI tool for writers who want to own their toolchain. Opinionated
trade-press typography, single static binary, no runtime dependencies.

Pre-1.0: expect minor adjustments to output formatting between minor
versions. See [`CHANGELOG.md`](./CHANGELOG.md) for what shipped in each
release.

## Install

```sh
cargo install papyrust-cli
```

The crate is `papyrust-cli`; the installed binary is `papyrust`.

Or from source:

```sh
git clone https://github.com/irvj/papyrust
cd papyrust
cargo install --path .
```

## Quick start

```sh
papyrust init my-novel
cd my-novel
# add your cover image as cover.jpg
# edit book.toml, replace the sample chapter
papyrust validate
papyrust build epub      # outputs build/<slug>.epub
papyrust build pdf       # outputs build/<slug>-<trim>.pdf
papyrust build all       # both
```

## What you get

**EPUB 3** — valid output verified against `epubcheck` in CI. Cover image embedded, auto-generated title page, copyright page, and navigation TOC, plus your custom front-matter and back-matter pages. CSS theme with serif body, small-caps headings, drop cap on chapter openings, centered scene-break ornament.

**Print PDF** — typeset via embedded Typst, EB Garamond bundled in the binary. Selectable trim sizes (5×8, 5.5×8.5, 6×9). Chapters start on the recto. Running heads on body pages: book title on verso, chapter title on recto. Arabic page numbers in the body (roman in user front-matter), suppressed on chapter-opening pages. Raised cap on the first paragraph of each chapter. Scene breaks rendered as a tracked three-asterisk ornament.

## Project layout

```
my-novel/
├── book.toml
├── cover.jpg
├── front-matter/        # optional, custom user pages
│   └── 01-dedication.md
├── chapters/            # required
│   ├── 01-chapter.md
│   └── 02-chapter.md
└── back-matter/         # optional
    └── 01-about-author.md
```

Front-matter pages like the title page, copyright page, and table of
contents are auto-generated from `book.toml`. Custom pages you drop into
`front-matter/` slot in after them.

Chapter titles come from the first `# Heading` in each chapter file.

A fully annotated `book.toml` reference lives at
[`examples/book.toml`](./examples/book.toml). `papyrust init` writes a
shorter scaffolded version into your new project.

## License

MIT. See [`LICENSE`](./LICENSE). The bundled EB Garamond font is OFL;
its license is at [`fonts/OFL.txt`](./fonts/OFL.txt).
