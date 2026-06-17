// Body + back-matter page layout: arabic page numbers, running heads
// (book title verso, chapter title recto), all suppressed on
// chapter-opening pages. Pulled in by `source.rs` via `include_str!`.
//
// Expects a `book-title` string binding to be defined beforehand (emitted
// from Rust so the title is escaped); everything else here is static.
#set page(
  numbering: "1",
  header: context {
    let page-num = counter(page).get().first()
    let chapter-here = query(heading.where(level: 1)).filter(c => c.location().page() == page-num)
    if chapter-here.len() > 0 { return [] }
    let chapters-before = query(heading.where(level: 1).before(here()))
    if chapters-before.len() == 0 { return [] }
    let chapter-title = chapters-before.last().body
    if calc.even(page-num) {
      align(left, text(size: 0.85em, tracking: 0.1em, smallcaps(book-title)))
    } else {
      align(right, text(size: 0.85em, tracking: 0.1em, smallcaps(chapter-title)))
    }
  },
  footer: context {
    let page-num = counter(page).get().first()
    let chapter-here = query(heading.where(level: 1)).filter(c => c.location().page() == page-num)
    if chapter-here.len() > 0 { return [] }
    align(center, text(size: 0.85em, numbering("1", page-num)))
  },
)
#counter(page).update(1)
