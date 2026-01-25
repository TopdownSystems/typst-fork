// Multi-Page Article Example
// Demonstrates masthead and wrap elements across page boundaries

#set page(paper: "a4", margin: 2cm)
#set text(font: "Linux Libertine", size: 11pt)
#set par(justify: true)
// Note: Heading numbering disabled because numbered headings use a hanging
// indent that conflicts with left-side mastheads. The numbers would extend
// into the cutout region. For documents with left mastheads, either use
// unnumbered headings or place the masthead on the right side.
// #set heading(numbering: "1.")

// Title block
#align(center)[
  #text(size: 24pt, weight: "bold")[The Complete Guide to Text Flow]
  #v(0.5em)
  #text(size: 14pt, fill: gray)[A Multi-Page Document Demonstration]
  #v(0.3em)
  #text(size: 11pt)[By Example Author | January 2024]
]

#v(1.5em)

// Masthead sidebar for the first page area
// Note: Using right side to avoid conflicts with heading alignment
// (left-side mastheads can overlap with heading text that extends to the margin)
#masthead(right, 3cm, clearance: 1em)[
  #set text(size: 9pt)

  #box(
    fill: blue.lighten(90%),
    inset: 10pt,
    width: 100%,
    radius: 4pt,
  )[
    #text(weight: "bold", size: 10pt)[Contents]

    #v(0.5em)

    1. Introduction

    2. Basic Concepts

    3. Advanced Usage

    4. Best Practices

    5. Conclusion

    #v(1em)

    #line(length: 100%, stroke: 0.5pt + gray)

    #v(0.5em)

    _Reading time: 5 min_
  ]
]

= Introduction

This document demonstrates how wrap and masthead elements work correctly in multi-page documents. The text flows naturally around sidebar content on the first page, and then continues at full width on subsequent pages.

// Note: Using left side for this wrap to avoid conflict with the right-side masthead
#wrap(left, clearance: 1em)[
  #box(
    width: 5cm,
    stroke: 2pt + blue,
    inset: 12pt,
    radius: 4pt,
    fill: blue.lighten(95%),
  )[
    #text(weight: "bold")[Key Insight]

    Text flow adapts dynamically to the available space, creating professional magazine-style layouts.
  ]
]

The wrap element allows content to float alongside text, similar to how images work in word processors and desktop publishing software. However, unlike simple floating, Typst's wrap element fully participates in the layout algorithm.

When text encounters a wrap element, it calculates the available width for each line and adjusts accordingly. This means text truly flows around the wrapped content rather than simply being pushed aside.

= Basic Concepts

Understanding the fundamentals of text flow is essential for creating professional documents. Let's explore the key concepts.

== Width Calculation

#wrap(left, clearance: 1em)[
  #box(
    width: 4cm,
    fill: green.lighten(90%),
    inset: 10pt,
  )[
    #text(weight: "bold")[Formula]

    `available = total - cutout - clearance`
  ]
]

Each line of text calculates its available width by subtracting any cutout regions (from wrap or masthead elements) from the total column width. The clearance provides spacing between the cutout and the flowing text.

This calculation happens independently for each line, allowing text to flow smoothly around irregular shapes and multiple cutout regions.

== Vertical Positioning

Wrap elements are positioned vertically at the current flow position when they are encountered. Masthead elements, by contrast, span the full height of their containing region.

This distinction makes wraps ideal for images and pull quotes, while mastheads work best for sidebars and navigation panels.

= Advanced Usage

Once you understand the basics, you can create sophisticated layouts combining multiple elements.

// Note: Page break ensures the Multiple Wraps section has room for both wraps
// This avoids a known issue where wraps at page boundaries may overlap with text
#pagebreak()

== Multiple Wraps

#wrap(right, clearance: 0.8em)[
  #box(width: 3.5cm, height: 2.5cm, fill: red.lighten(80%), inset: 8pt)[
    #align(center + horizon)[Image A]
  ]
]

You can use multiple wrap elements on the same page. Each wrap creates its own cutout region, and text flows around all of them simultaneously.

#wrap(left, clearance: 0.8em)[
  #box(width: 3.5cm, height: 2.5cm, fill: purple.lighten(80%), inset: 8pt)[
    #align(center + horizon)[Image B]
  ]
]

When wraps appear on opposite sides, text flows between them in the middle. This creates a visually interesting layout that draws the reader's attention to the wrapped content.

The layout engine handles all the complexity of calculating available widths when multiple cutouts overlap vertically.

== Combining Wrap and Masthead

For complex layouts, you might combine a masthead with wrap elements. The masthead provides consistent sidebar content throughout a section, while wraps highlight specific content at particular points.

#wrap(right, clearance: 1em)[
  #box(
    width: 4.5cm,
    stroke: 1pt + orange,
    inset: 10pt,
    radius: 4pt,
  )[
    #text(weight: "bold")[Pro Tip]

    Use mastheads for persistent navigation and wraps for contextual callouts.
  ]
]

This combination is particularly effective in technical documentation, where readers benefit from both persistent reference material and inline highlights.

= Best Practices

Follow these guidelines for optimal results:

== Content Width

Avoid making wrap or masthead content so wide that the remaining text becomes difficult to read. As a rule of thumb, leave at least 60% of the column width for flowing text.

#wrap(left, clearance: 1em)[
  #box(
    width: 4cm,
    fill: yellow.lighten(80%),
    inset: 10pt,
  )[
    #text(weight: "bold")[Rule of Thumb]

    Keep cutouts under 40% of column width for readable text.
  ]
]

This ensures that readers can comfortably read the main content without their eyes having to jump across large gaps. When planning your layout, consider the reading experience first. A well-balanced layout keeps readers engaged with the content.

The visual balance between wrapped elements and flowing text is crucial for professional documents. Too narrow a text column forces excessive hyphenation and creates an uncomfortable reading rhythm. Conversely, wrapped elements that are too small may not provide sufficient visual weight to justify their presence. Finding the right proportion requires consideration of your content type, audience, and the overall document design.

== Clearance Settings

Appropriate clearance improves readability:
- 1em: Good default for most content
- 0.5em: Tight layouts, minimal spacing
- 1.5em+: Emphasis, important callouts

== Page Breaks

The layout engine handles page breaks automatically. When content with cutouts spans multiple pages, the text flow adapts correctly to each page's available space.

#lorem(150)

= Conclusion

Text flow with wrap and masthead elements enables professional document layouts in Typst. The system handles complex scenarios including:

- Multiple concurrent cutouts
- Page boundary crossings
- Variable width calculations
- Widow and orphan prevention

#wrap(right, clearance: 1em)[
  #box(
    width: 5cm,
    fill: blue.lighten(90%),
    stroke: 2pt + blue,
    inset: 12pt,
    radius: 4pt,
  )[
    #text(weight: "bold", size: 12pt)[Summary]

    #v(0.5em)

    - Wrap: floating content
    - Masthead: full-height sidebars
    - Both support RTL
    - Multi-page works
  ]
]

By understanding these elements and following best practices, you can create documents that rival professional desktop publishing software while maintaining the simplicity and reproducibility that Typst provides.

#lorem(200)

#v(2em)

#align(center)[
  #line(length: 30%, stroke: 1pt + gray)
  #v(1em)
  #text(style: "italic")[End of Document]
]
