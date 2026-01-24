// Wrap Element Demo
// Compile with: typst compile --no-pdf-tags wrap-demo.typ wrap-demo.pdf

#set page(width: 300pt, height: auto, margin: 20pt)
#set text(size: 10pt)
#set par(justify: true)

= Wrap Element Examples

== Basic Right Wrap

#wrap(right, rect(width: 80pt, height: 60pt, fill: aqua.lighten(50%), stroke: aqua)[
  #align(center + horizon)[Image]
])

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.

#v(1em)

== Basic Left Wrap

#wrap(left, rect(width: 70pt, height: 50pt, fill: orange.lighten(50%), stroke: orange)[
  #align(center + horizon)[Sidebar]
])

Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident.

#v(1em)

== Wrap with Clearance

#wrap(right, clearance: 20pt, rect(width: 60pt, height: 40pt, fill: green.lighten(50%), stroke: green)[
  #align(center + horizon)[20pt gap]
])

Notice the larger gap between this text and the wrapped box. The clearance parameter controls the spacing between wrapped content and flowing text.

#v(1em)

== Multiple Wraps (Both Sides)

#wrap(left, rect(width: 50pt, height: 80pt, fill: blue.lighten(50%), stroke: blue)[
  #align(center + horizon)[Left]
])

#wrap(right, rect(width: 50pt, height: 80pt, fill: red.lighten(50%), stroke: red)[
  #align(center + horizon)[Right]
])

Text flows between two wrapped elements. This creates a narrow column of text in the middle. Useful for creating interesting layouts with content on both sides.

#v(1em)

== Wrap with Figure

#wrap(right, clearance: 12pt)[
  #figure(
    rect(width: 80pt, height: 60pt, fill: purple.lighten(70%))[
      #align(center + horizon)[Chart]
    ],
    caption: [Sales data Q4]
  )
]

Figures can be wrapped too, complete with captions. The caption flows naturally with the figure content and text wraps around the entire figure block.

#pagebreak()

= RTL Text Direction

#set text(dir: rtl)

== Wrap in RTL Mode

#wrap(start, rect(width: 60pt, height: 50pt, fill: teal.lighten(50%), stroke: teal)[
  #align(center + horizon)[بداية]
])

هذا نص عربي يوضح كيفية عمل التفاف النص حول المحتوى في اتجاه من اليمين إلى اليسار. لاحظ أن جانب "البداية" يعني الجانب الأيمن في النص العربي.

#set text(dir: ltr)

#pagebreak()

= Practical Examples

== Pull Quote Design

#let pull-quote(body) = wrap(right, clearance: 1em)[
  #block(
    width: 100pt,
    inset: (x: 10pt, y: 8pt),
    stroke: (left: 3pt + blue),
    fill: blue.lighten(90%),
  )[
    #set text(style: "italic", size: 11pt)
    #body
  ]
]

#pull-quote[
  "Design is not just what it looks like. Design is how it works."
  — Steve Jobs
]

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.

#v(1em)

== Info Box

#wrap(left, clearance: 10pt)[
  #block(
    width: 90pt,
    inset: 8pt,
    radius: 4pt,
    fill: yellow.lighten(70%),
    stroke: yellow.darken(20%),
  )[
    #text(weight: "bold", size: 9pt)[Note]

    #text(size: 8pt)[
      This is an info box with important details that readers should notice.
    ]
  ]
]

Main content continues here with the info box displayed to the left. This pattern is useful for callouts, warnings, tips, or any supplementary information that should stand out but not interrupt the flow.

#v(1em)

== Sequential Wraps

#wrap(right, rect(width: 60pt, height: 40pt, fill: red.lighten(60%))[#align(center + horizon)[1]])

First paragraph wraps around the first box.

#wrap(right, rect(width: 60pt, height: 40pt, fill: green.lighten(60%))[#align(center + horizon)[2]])

Second paragraph wraps around the second box. Each wrap element affects content that follows it.

#wrap(right, rect(width: 60pt, height: 40pt, fill: blue.lighten(60%))[#align(center + horizon)[3]])

Third paragraph with its own wrap. This creates a staggered layout effect.
