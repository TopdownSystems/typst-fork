// Sidebar example using native masthead element
// Compile with: typst compile --no-pdf-tags sidebar0.typ sidebar0.pdf

#set page(
  margin: (top: 1in, bottom: 1in, left: 0.5in, right: 0.5in),
  header-ascent: 0in,  // Header starts right at the top margin boundary
  header: box(
    width: 100%,
    height: 0.5in,     // Fixed 0.5in header height
    fill: aqua,
    clip: true,
    inset: 12pt,
  )[
    #place(top + left)[
      #set par(leading: 0.65em)
      My Document Header \
      Test
    ]
  ],
  footer-descent: 0in,
  footer: box(
    width: 100%,
    height: 0.5in,
    fill: orange,
    clip: true,
    inset: 5pt,
  )[
    #place(top + left)[
      #set par(leading: 0.65em)
      #grid(
        columns: (1fr, 1fr),
        align(left)[Author Name],
        align(right)[
          Page #context counter(page).display()
        ],
      )
      #v(-0.5em)
      Second line
    ]
  ],
)

#set par(justify: true)

// Use native masthead for the sidebar - positioned on the right
#masthead(right, 150pt, clearance: 10pt)[
  #block(
    fill: luma(250),
    inset: 10pt,
  )[
    #set text(size: 9pt)
    *Sidebar*

    Notes here.

    More sidebar content. More sidebar content. More sidebar content. More sidebar content. More sidebar content. More sidebar content. More sidebar content. More sidebar content.
  ]
]

= Main Content

This text will wrap around the sidebar on the first page,
then automatically continue at full width on subsequent pages.\
#lorem(1000)

= Another Section

#lorem(350)

#pagebreak()

#set page(
  paper: "us-legal",  // 8.5" × 14"
  margin: 2in,
  fill: green,
)

= Last Section

#lorem(100)

#let custom-item(label, body, spacing: 1em) = {
  grid(
    columns: (auto, 1fr),
    column-gutter: spacing,
    [#label],
    body
  )
}

#custom-item("1)", [First item text here])
#custom-item("2)", [Second item text here])
#custom-item("1.1.1.", [Nested item], spacing: 2em)
