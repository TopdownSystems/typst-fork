// Masthead Element Demo
// Compile with: typst compile --no-pdf-tags masthead-demo.typ masthead-demo.pdf

#set page(width: 350pt, height: auto, margin: 20pt)
#set text(size: 10pt)
#set par(justify: true)

= Masthead Element Examples

== Basic Left Masthead

#masthead(left, 80pt)[
  #block(fill: blue.lighten(80%), inset: 8pt)[
    *Newsletter*

    Issue #42

    January 2024
  ]
]

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.

Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

#pagebreak()

== Basic Right Masthead

#masthead(right, 70pt)[
  #block(fill: green.lighten(80%), inset: 8pt)[
    *Sidebar*

    Quick links:
    - Home
    - About
    - Contact
  ]
]

Main content flows to the left of the masthead. The masthead extends the full height of the column, creating a sidebar effect that's common in magazine and newsletter layouts.

Additional paragraphs continue to flow alongside the masthead until the content ends or a new page begins.

#pagebreak()

== Masthead with Clearance

#masthead(left, 70pt, clearance: 25pt)[
  #block(fill: orange.lighten(80%), inset: 8pt)[
    *Wide Gap*

    25pt clearance between masthead and text.
  ]
]

Notice the larger gap between the masthead and this text. The clearance parameter controls spacing.

#pagebreak()

== Table of Contents Masthead

#masthead(left, 90pt)[
  #block(fill: purple.lighten(85%), inset: 10pt)[
    #text(weight: "bold", size: 11pt)[Contents]

    #v(0.5em)

    #set text(size: 9pt)
    1. Introduction ... 1
    2. Background ... 3
    3. Methods ... 7
    4. Results ... 12
  ]
]

= Introduction

This document demonstrates how to use the masthead element to create a table of contents sidebar. The masthead runs the full height of the content area, providing a persistent navigation reference.

The main text flows naturally around the masthead, making efficient use of the page width while keeping important reference material visible.

#pagebreak()

= RTL Support

#set text(dir: rtl)

#masthead(start, 80pt)[
  #block(fill: teal.lighten(80%), inset: 8pt)[
    *فهرس*

    المقدمة
    الخلفية
    النتائج
  ]
]

هذا النص العربي يتدفق حول العمود الجانبي. في وضع RTL، يظهر جانب "البداية" على اليمين.

#set text(dir: ltr)

#pagebreak()

= Practical Examples

== Article Info Panel

#masthead(left, 85pt)[
  #block(
    fill: blue.lighten(90%),
    inset: 10pt,
  )[
    #text(weight: "bold", size: 10pt)[Article Info]

    #v(0.5em)

    #set text(size: 8pt)

    *Author*
    Dr. Jane Smith

    *Published*
    January 15, 2024

    *Category*
    Technology
  ]
]

Artificial intelligence is revolutionizing healthcare delivery across the globe. From diagnostic imaging to drug discovery, AI systems are augmenting human expertise and improving patient outcomes.

Machine learning algorithms now assist radiologists in detecting tumors, help pathologists identify disease markers, and enable personalized treatment recommendations based on genetic profiles.

#pagebreak()

== Quote Sidebar

#masthead(right, 100pt)[
  #block(
    fill: yellow.lighten(85%),
    inset: 12pt,
  )[
    #set text(style: "italic")

    "The best way to predict the future is to invent it."

    #v(0.5em)
    #align(right)[— Alan Kay]

    #v(1em)

    "Any sufficiently advanced technology is indistinguishable from magic."

    #v(0.5em)
    #align(right)[— Arthur C. Clarke]
  ]
]

The quotes displayed in the sidebar capture the essence of technological innovation. These visionary statements remind us that the boundaries of possibility are constantly expanding.

When we approach challenges with creativity and determination, we can achieve outcomes that once seemed impossible.
