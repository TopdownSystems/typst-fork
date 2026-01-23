// Test the `masthead` element for newsletter-style column layouts.

--- masthead-basic-left paged ---
// Basic test: masthead on the left side with explicit width.
#set page(width: 300pt, height: auto)

#masthead(left, 80pt)[
  *Newsletter*

  Issue 42
]

#lorem(30)

--- masthead-basic-right paged ---
// Basic test: masthead on the right side.
#set page(width: 300pt, height: auto)

#masthead(right, 70pt)[
  *Sidebar*

  Extra info
]

#lorem(30)

--- masthead-sides-start-end paged ---
// Test logical sides: start and end.
#set page(width: 300pt, height: auto)

#masthead(start, 60pt)[
  Start side
]

Text flows around the start-aligned masthead.

#v(2em)

#masthead(end, 60pt)[
  End side
]

Text flows around the end-aligned masthead.

--- masthead-clearance paged ---
// Test clearance between masthead and text.
#set page(width: 300pt, height: auto)

#masthead(left, 70pt, clearance: 20pt)[
  Large clearance
]

#lorem(25)

--- masthead-clearance-zero paged ---
// Test zero clearance.
#set page(width: 300pt, height: auto)

#masthead(left, 60pt, clearance: 0pt)[
  No gap
]

#lorem(25)

--- masthead-tall-content paged ---
// Masthead with tall content spanning multiple paragraphs.
#set page(width: 300pt, height: auto)

#masthead(left, 80pt)[
  *Table of Contents*

  1. Introduction
  2. Methods
  3. Results
  4. Discussion
  5. Conclusion
]

First paragraph flows around the masthead column.

Second paragraph continues to flow.

Third paragraph as well.

#lorem(20)

--- masthead-short-content paged ---
// Masthead with short content - text returns to full width.
#set page(width: 300pt, height: auto)

#masthead(left, 60pt)[
  Short
]

After this line, text should use full width.

#lorem(30)

--- masthead-rtl paged ---
// Test masthead in RTL text direction.
#set page(width: 300pt, height: auto)
#set text(dir: rtl)

#masthead(start, 70pt)[
  عنوان
]

هذا نص عربي يلتف حول العمود الجانبي. المحتوى يظهر على الجانب الصحيح.

--- masthead-in-columns paged ---
// Test masthead within multi-column layout.
#set page(width: 400pt, height: 150pt, columns: 2)

#masthead(left, 50pt)[
  Col 1
]

#lorem(40)

--- masthead-scope-parent paged ---
// Test masthead with parent scope spanning columns.
#set page(width: 400pt, height: auto, columns: 2)

#masthead(left, 80pt, scope: "parent")[
  *Spanning*

  This masthead spans both columns.
]

#lorem(80)

--- masthead-with-heading paged ---
// Test masthead interaction with headings.
#set page(width: 300pt, height: auto)

= Chapter Title

#masthead(left, 60pt)[
  Chapter notes
]

#lorem(30)

--- masthead-narrow-text paged ---
// Test when text area becomes very narrow.
#set page(width: 200pt, height: auto)

#masthead(left, 100pt)[
  Wide masthead
]

This is narrow text that flows around a large masthead column.

--- masthead-with-list paged ---
// Test masthead with list items.
#set page(width: 300pt, height: auto)

#masthead(left, 70pt)[
  *Index*

  A, B, C
]

- First list item that may wrap
- Second list item
- Third list item
- Fourth list item

--- masthead-default-side paged ---
// Test default side (should be start).
#set page(width: 300pt, height: auto)

#masthead(60pt)[
  Default
]

Default side is start (left in LTR).

--- masthead-large-clearance paged ---
// Test with large clearance value.
#set page(width: 300pt, height: auto)

#masthead(left, 60pt, clearance: 40pt)[
  Gap
]

Text with large clearance between it and the masthead column.

--- masthead-multiple paged ---
// Multiple mastheads on same page.
#set page(width: 400pt, height: auto)

#masthead(left, 60pt)[
  Left side
]

#masthead(right, 60pt)[
  Right side
]

#lorem(50)
