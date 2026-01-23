// Test the `wrap` element for text flow around content.

--- wrap-basic-right paged ---
// Basic test: text flows around content on the right side.
#set page(width: 200pt, height: auto)

#wrap(right, rect(width: 60pt, height: 60pt, fill: aqua))

#lorem(30)

--- wrap-basic-left paged ---
// Basic test: text flows around content on the left side.
#set page(width: 200pt, height: auto)

#wrap(left, rect(width: 60pt, height: 60pt, fill: aqua))

#lorem(30)

--- wrap-sides-start-end paged ---
// Test logical sides: start and end.
#set page(width: 200pt, height: auto)

#wrap(start, rect(width: 50pt, height: 40pt, fill: red))

Text flows around the start-aligned box.

#v(2em)

#wrap(end, rect(width: 50pt, height: 40pt, fill: blue))

Text flows around the end-aligned box.

--- wrap-clearance paged ---
// Test clearance between wrapped content and text.
#set page(width: 200pt, height: auto)

#wrap(right, clearance: 20pt, rect(width: 50pt, height: 50pt, fill: green))

#lorem(25)

--- wrap-clearance-zero paged ---
// Test zero clearance.
#set page(width: 200pt, height: auto)

#wrap(right, clearance: 0pt, rect(width: 50pt, height: 50pt, fill: orange))

#lorem(25)

--- wrap-multiple-same-side paged ---
// Multiple wraps on the same side, stacked vertically.
#set page(width: 200pt, height: auto)

#wrap(right, rect(width: 50pt, height: 40pt, fill: aqua))

#lorem(10)

#wrap(right, rect(width: 50pt, height: 40pt, fill: conifer))

#lorem(15)

--- wrap-multiple-both-sides paged ---
// Wraps on both sides simultaneously.
#set page(width: 250pt, height: auto)

#wrap(left, rect(width: 50pt, height: 60pt, fill: red))
#wrap(right, rect(width: 50pt, height: 60pt, fill: blue))

#lorem(40)

--- wrap-image paged ---
// Wrap around an image (simulated with rect).
#set page(width: 200pt, height: auto)

#wrap(
  right,
  clearance: 8pt,
  box(
    fill: aqua,
    width: 60pt,
    height: 80pt,
    [Image]
  )
)

#lorem(50)

--- wrap-tall-content paged ---
// Wrap around tall content that spans multiple paragraphs.
#set page(width: 200pt, height: auto)

#wrap(right, rect(width: 50pt, height: 150pt, fill: forest))

First paragraph flows around the tall wrapped content.

Second paragraph continues to flow.

Third paragraph as well.

#lorem(20)

--- wrap-short-content paged ---
// Wrap around short content - text should return to full width.
#set page(width: 200pt, height: auto)

#wrap(right, rect(width: 50pt, height: 30pt, fill: aqua))

Short wrap. After this line, text should use full width.

#lorem(30)

--- wrap-rtl paged ---
// Test wrapping in RTL text direction.
#set page(width: 200pt, height: auto)
#set text(dir: rtl)

#wrap(start, rect(fill: blue, width: 50pt, height: 50pt))

هذا نص عربي يلتف حول الصورة. المحتوى يظهر على الجانب الأيمن لأن النص من اليمين إلى اليسار.

--- wrap-in-columns paged ---
// Test wrap within multi-column layout.
#set page(width: 300pt, height: 150pt, columns: 2)

#wrap(right, rect(width: 40pt, height: 40pt, fill: aqua))

#lorem(40)

--- wrap-scope-parent paged ---
// Test wrap with parent scope spanning columns.
#set page(width: 300pt, height: auto, columns: 2)

#wrap(
  left,
  scope: "parent",
  rect(width: 60pt, height: 80pt, fill: red),
)

#lorem(80)

--- wrap-with-heading paged ---
// Test wrap interaction with headings.
#set page(width: 200pt, height: auto)

= Chapter Title

#wrap(right, rect(width: 50pt, height: 50pt, fill: aqua))

#lorem(30)

--- wrap-narrow-text paged ---
// Test when text area becomes very narrow.
#set page(width: 150pt, height: auto)

#wrap(right, rect(width: 80pt, height: 60pt, fill: aqua))

This is narrow text that flows around a large wrapped element.

--- wrap-nested-containers paged ---
// Test wrap inside a block container.
#set page(width: 200pt, height: auto)

#block(
  fill: luma(240),
  inset: 10pt,
  width: 100%,
)[
  #wrap(right, rect(width: 40pt, height: 40pt, fill: aqua))

  Text inside a container flows around the wrapped content.
]

--- wrap-empty-body paged ---
// Test wrap with minimal body.
#set page(width: 200pt, height: auto)

#wrap(right, rect(width: 30pt, height: 30pt))

Text flows around an empty rectangle.

--- wrap-preserves-paragraph-break paged ---
// Ensure paragraph breaks are preserved.
#set page(width: 200pt, height: auto)

#wrap(right, rect(width: 50pt, height: 100pt, fill: aqua))

First paragraph.

Second paragraph after a break.

Third paragraph continues.

--- wrap-with-list paged ---
// Test wrap with list items.
#set page(width: 200pt, height: auto)

#wrap(right, rect(width: 50pt, height: 80pt, fill: aqua))

- First list item that may wrap
- Second list item
- Third list item
- Fourth list item

--- wrap-sequential paged ---
// Multiple wraps appearing sequentially.
#set page(width: 200pt, height: auto)

#wrap(right, rect(width: 40pt, height: 40pt, fill: red))
Text around first wrap.

#wrap(left, rect(width: 40pt, height: 40pt, fill: blue))
Text around second wrap.

#wrap(right, rect(width: 40pt, height: 40pt, fill: green))
Text around third wrap.

--- wrap-default-side paged ---
// Test default side (should be end).
#set page(width: 200pt, height: auto)

#wrap(rect(width: 50pt, height: 50pt, fill: aqua))

Default side is end (right in LTR).

--- wrap-large-clearance paged ---
// Test with large clearance value.
#set page(width: 200pt, height: auto)

#wrap(right, clearance: 40pt, rect(width: 40pt, height: 40pt, fill: aqua))

Text with large clearance between it and the wrapped content.
