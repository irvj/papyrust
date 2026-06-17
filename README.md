# papyrust

Build publication-quality EPUB and print-ready PDF from a folder of Markdown.

A CLI tool for writers who want to own their toolchain. Opinionated
trade-press typography, single static binary, no runtime dependencies.

> **Status:** under active development. M1 (project scaffolding +
> validation) is complete. EPUB rendering (M2) and print PDF (M3) are next.
> See [`PLAN.md`](./PLAN.md) for the roadmap.

## Install (from source)

```sh
git clone https://github.com/irvj/papyrust
cd papyrust
cargo install --path crates/papyrust
```

Once published, `cargo install papyrust` will be the standard install path.

## Quick start

```sh
papyrust init my-novel
cd my-novel
# add your cover image as cover.jpg
# edit book.toml, replace the sample chapter
papyrust validate
papyrust build epub      # planned for M2
papyrust build pdf       # planned for M3
```

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

## License

MIT. See [`LICENSE`](./LICENSE).
