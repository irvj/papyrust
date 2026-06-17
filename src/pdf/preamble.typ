// Static typography rules and helpers, shared by every build. Pulled in
// verbatim by `source.rs` via `include_str!` after the dynamic document /
// page / text settings (which interpolate per-book values) are emitted.
//
// Keep this pure Typst: no per-book values belong here. Anything that
// varies by book is written from Rust, where user text is escaped.

// Body paragraphs: justified, first-line indent on all but the first
// paragraph after a heading or scene break.
#set par(leading: 0.65em, justify: true, first-line-indent: (amount: 1.5em, all: false))

// Scene break: three asterisks with generous tracking is the trade-press
// fiction convention and works in any font (no dingbat coverage needed).
#let scene-break = {
  v(0.7em)
  align(center, text(tracking: 0.5em, "* * *"))
  v(0.7em)
}

// Raised cap: the first character of each chapter's first paragraph is set
// larger and slightly tracked, in the Penguin Classics style. Typst lacks
// native text-wrap so a true floating drop cap isn't available; this stays
// within the first line for a clean result.
#let raise-cap(letter) = text(size: 2.2em, weight: "regular", tracking: 0.05em, letter)

// Chapter heading: each level-1 heading starts a new recto (odd page),
// with centered small-caps display and breathing room.
#show heading.where(level: 1): it => {
  pagebreak(weak: true, to: "odd")
  v(1.5in)
  align(center, text(size: 1.6em, tracking: 0.1em, weight: "regular", smallcaps(it.body)))
  v(2em)
}

#show heading.where(level: 2): it => align(center, text(size: 1.2em, smallcaps(it.body)))
