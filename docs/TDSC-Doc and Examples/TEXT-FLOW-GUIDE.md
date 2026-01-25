# Text Flow Guide: Wrap and Masthead Elements

This guide explains how to use Typst's text flow features to create professional PDF documents with text wrapping around images and sidebar content.

## Table of Contents

- [Overview](#overview)
- [The `wrap` Element](#the-wrap-element)
- [The `masthead` Element](#the-masthead-element)
- [RTL (Right-to-Left) Language Support](#rtl-right-to-left-language-support)
- [Multi-Page Documents](#multi-page-documents)
- [First Page Only Masthead (Workaround)](#first-page-only-masthead-workaround)
- [PDF Export: Avoiding Tag Errors](#pdf-export-avoiding-tag-errors)
- [Performance Characteristics](#performance-characteristics)
- [Complete Examples](#complete-examples)

---

## Overview

Two new elements enable magazine-style text flow layouts:

| Element | Purpose | Width | Height |
|---------|---------|-------|--------|
| `wrap` | Float content (images, sidebars) with text flowing around | Determined by content | Determined by content |
| `masthead` | Full-height column sidebars | Explicit (required) | Full column height |

Both elements support:
- Logical positioning (`start`/`end`) and physical positioning (`left`/`right`)
- RTL text direction
- Configurable clearance (gap between wrapped content and text)
- Column or parent scope in multi-column layouts

---

## The `wrap` Element

Use `wrap` to position content with text flowing around it.

### Basic Usage

```typst
// Image on the right with text wrapping around it
#wrap(right)[
  #image("photo.jpg", width: 4cm)
]

Lorem ipsum dolor sit amet, consectetur adipiscing elit.
Text will flow around the image on the left side.
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `side` | `start`, `end`, `left`, `right` | `end` | Which side to place the wrapped content |
| `body` | content | (required) | The content to wrap around |
| `clearance` | length | `1em` | Gap between wrapped content and flowing text |
| `scope` | `column`, `parent` | `column` | Whether wrap affects just the column or spans columns |

### Side Positioning

```typst
// Logical sides (respect text direction)
#wrap(start)[...]  // Left in LTR, right in RTL
#wrap(end)[...]    // Right in LTR, left in RTL

// Physical sides (fixed regardless of text direction)
#wrap(left)[...]   // Always left
#wrap(right)[...]  // Always right
```

### Clearance

```typst
// Large gap between image and text
#wrap(right, clearance: 2em)[
  #image("photo.jpg", width: 4cm)
]

// No gap (text touches the image)
#wrap(right, clearance: 0pt)[
  #image("photo.jpg", width: 4cm)
]
```

### Multiple Wraps

```typst
// Wraps on both sides
#wrap(left)[#box(width: 3cm, height: 4cm, fill: blue)]
#wrap(right)[#box(width: 3cm, height: 4cm, fill: red)]

Text flows between the two wrapped elements in the middle.
```

### Wrap with Images

```typst
#wrap(right, clearance: 12pt)[
  #figure(
    image("diagram.png", width: 5cm),
    caption: [System architecture]
  )
]

The system architecture shown in the figure demonstrates...
```

---

## The `masthead` Element

Use `masthead` to create full-height column sidebars, commonly used for:
- Magazine article sidebars
- Pull quotes that span the column height
- Navigation or reference panels

### Basic Usage

```typst
// Sidebar on the left, 3cm wide
#masthead(left, 3cm)[
  *Quick Facts*
  - Founded: 1995
  - Employees: 500
  - Revenue: \$50M
]

Main article text flows in the remaining space to the right
of the masthead sidebar.
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `side` | `start`, `end`, `left`, `right` | `start` | Which side to place the masthead |
| `width` | length | (required) | Width of the masthead (positional argument) |
| `body` | content | (required) | The content inside the masthead |
| `clearance` | length | `1em` | Gap between masthead and flowing text |
| `scope` | `column`, `parent` | `column` | Whether masthead affects just the column or spans columns |

### Width is Required

Unlike `wrap`, masthead requires an explicit width:

```typst
// Correct: width specified
#masthead(3cm)[Sidebar content]

// This would be an error - width is required
// #masthead()[Sidebar content]
```

### Full-Height Behavior

The masthead automatically extends to the full height of the column:

```typst
#masthead(right, 4cm)[
  #align(top)[*Top content*]
  #v(1fr)
  #align(bottom)[*Bottom content*]
]

Text in the main column flows alongside the masthead
for the entire page/column height.
```

---

## RTL (Right-to-Left) Language Support

The wrap and masthead elements fully support right-to-left languages like Arabic, Hebrew, Persian, and Urdu. The key concept is understanding **logical** vs **physical** positioning.

### Logical vs Physical Sides

| Side | In LTR Text | In RTL Text |
|------|-------------|-------------|
| `start` | Left | Right |
| `end` | Right | Left |
| `left` | Left | Left |
| `right` | Right | Right |

**Use logical sides (`start`/`end`)** when you want content to follow text direction:
- `start` = "where the reader begins" (left in English, right in Arabic)
- `end` = "where the reader finishes" (right in English, left in Arabic)

**Use physical sides (`left`/`right`)** when position should be fixed regardless of language.

### Arabic Document Example

```typst
#set text(lang: "ar", dir: rtl)
#set page(paper: "a4", margin: 2cm)
#set par(justify: true)

// Masthead on the "start" side - appears on RIGHT for RTL
#masthead(start, 3cm)[
  #text(weight: "bold")[محتويات المقال]

  - المقدمة
  - الفصل الأول
  - الفصل الثاني
  - الخاتمة
]

= عنوان المقال الرئيسي

هذا النص العربي يتدفق إلى يسار الشريط الجانبي.
النص يحترم اتجاه القراءة من اليمين إلى اليسار.
```

### Hebrew Newsletter with Wrap

```typst
#set text(lang: "he", dir: rtl)
#set page(paper: "a4", margin: 2cm)

= כותרת העיתון

#wrap(end, clearance: 1em)[
  #box(
    width: 4cm,
    stroke: 1pt + blue,
    inset: 10pt,
  )[
    *נקודות מפתח*

    - נקודה ראשונה
    - נקודה שנייה
    - נקודה שלישית
  ]
]

// Text flows to the RIGHT of the wrap (because "end" = left in RTL)
זהו טקסט בעברית שזורם סביב התיבה.
הטקסט מופיע בצד ימין של התיבה כי אנחנו בכיוון ימין לשמאל.
```

### Mixed LTR/RTL Document

```typst
// English section (LTR)
#set text(lang: "en", dir: ltr)

= English Section

#wrap(start)[
  #box(width: 3cm, fill: blue.lighten(80%), inset: 8pt)[
    *Info Box*

    On the left in English.
  ]
]

#lorem(50)

// Arabic section (RTL)
#set text(lang: "ar", dir: rtl)

= القسم العربي

#wrap(start)[
  #box(width: 3cm, fill: green.lighten(80%), inset: 8pt)[
    *معلومات*

    على اليمين بالعربية.
  ]
]

هذا النص العربي يظهر على يسار المربع لأن "start" يعني اليمين في النصوص العربية.
```

### Best Practices for RTL

1. **Use logical sides by default** - `start` and `end` adapt to text direction automatically
2. **Use physical sides for fixed layouts** - when position must stay constant regardless of language
3. **Test with actual RTL content** - placeholder text may not reveal all layout issues
4. **Consider mixed documents** - sections may have different text directions

---

## Multi-Page Documents

Wrap and masthead elements work correctly in documents that span multiple pages. Content automatically flows across page boundaries.

### Long Document with Masthead

```typst
#set page(paper: "a4", margin: 2cm)
#set par(justify: true)

#masthead(left, 3cm)[
  *Reference Panel*

  This sidebar content appears alongside the main text.

  #v(1fr)

  _See page 1 for index_
]

// This content will flow across multiple pages
// The masthead appears on the first page only by default
#lorem(500)

// For very long content (1000+ words), layout works efficiently
#lorem(2000)
```

### Multi-Page Newsletter

```typst
#set page(paper: "letter", margin: 1.5cm)
#set text(size: 10pt)
#set par(justify: true)

= Company Newsletter - January 2024

#wrap(right, clearance: 1em)[
  #figure(
    rect(width: 5cm, height: 3cm, fill: luma(220))[
      #align(center + horizon)[Photo]
    ],
    caption: [CEO at annual meeting]
  )
]

#lorem(100)

== Department Updates

#wrap(left, clearance: 1em)[
  #box(
    width: 4cm,
    fill: blue.lighten(90%),
    inset: 10pt,
  )[
    *Quick Stats*
    - Revenue: +15%
    - Employees: 450
    - Projects: 23
  ]
]

#lorem(200)

== Looking Ahead

#lorem(300)

// Document continues across pages naturally
```

### Performance with Large Documents

The text-flow feature has been optimized for large documents:

| Document Size | Compilation Time | Notes |
|--------------|------------------|-------|
| 100 words + masthead | ~0.15s | Instant |
| 1,000 words + masthead | ~0.18s | Very fast |
| 3,500 words + masthead | ~0.21s | Fast |
| 50,000 words + masthead | ~0.37s | Still fast (72 pages) |

Content flows efficiently across page boundaries with linear scaling.

---

## First Page Only Masthead (Workaround)

Typst's `masthead` element affects all content that follows it. To have a masthead appear only on the first page, use this workaround with page state tracking:

### Method 1: Counter-Based Approach

```typst
#let first-page-masthead(width, content, body) = {
  // Track if we've shown the masthead
  let shown = state("masthead-shown", false)

  context {
    if not shown.get() {
      shown.update(true)
      masthead(left, width)[#content]
    }
  }
  body
}

// Usage
#first-page-masthead(3cm)[
  *Article Info*
  - Author: Jane Doe
  - Date: 2024
][
  Your main article content goes here...

  This text will have the masthead on page 1 only.
  Subsequent pages will use full width.
]
```

### Method 2: Manual Page Break Approach

For more control, manually structure your document:

```typst
// Page 1: With masthead
#page[
  #masthead(left, 3cm)[
    *Sidebar Content*

    Navigation or reference material here.
  ]

  First page content with the masthead sidebar.
  Add enough content to fill this page.
]

// Page 2 onwards: No masthead
#page[
  Remaining content without masthead.
  Full page width available.
]
```

### Method 3: Using `place` for Positioned Sidebar

If you need a sidebar that doesn't affect text flow (overlay style):

```typst
#set page(margin: (left: 5cm, rest: 2cm))

#place(
  left + top,
  dx: -3.5cm,
  block(
    width: 3cm,
    height: 100%,
    fill: luma(240),
    inset: 8pt,
  )[
    *Sidebar*

    This is positioned absolutely and doesn't
    participate in text flow.
  ]
)

Main content here with enlarged left margin
to make room for the positioned sidebar.
```

---

## PDF Export: Avoiding Tag Errors

When exporting to PDF with accessibility tagging enabled, you may encounter errors like:

```
Error [pdf]: internal error: parent group
```

This is a known issue with PDF tag generation for the new wrap/masthead elements.

### Workaround: Disable PDF Tagging

Export PDFs with the `--no-pdf-tags` flag to disable accessibility tagging:

```bash
# Command line - this works!
typst compile --no-pdf-tags document.typ output.pdf
```

This produces valid PDFs without the tagging error. The PDFs will render correctly but won't have accessibility structure tags (used by screen readers).

### Understanding the Issue

The PDF tagging system (`typst-pdf/src/tags/`) hasn't been updated to handle the new wrap and masthead elements. The error occurs because:

1. Wrap/masthead create "cutout" regions in the page
2. The PDF tag tree builder expects a specific parent-child relationship
3. Content that flows around cutouts doesn't fit the expected structure

### When This Will Be Fixed

This requires updates to the PDF export code to:
- Recognize wrap/masthead elements
- Generate appropriate tag structures for floating content
- Handle the parent-child relationships correctly

For production use where tagged PDFs are required, consider:
- Using the `place` element instead (has better PDF tag support)
- Waiting for PDF tagging updates
- Post-processing PDFs to add tags

---

## Performance Characteristics

The text-flow feature is designed for production use with efficient performance characteristics.

### Typical Performance

| Scenario | Time | Notes |
|----------|------|-------|
| No wraps/mastheads | 0ms overhead | Original memoized path used |
| 1-3 cutouts per page | Minimal overhead | Typical use case |
| Single-page document | ~0.15s | Instant feedback |
| Multi-page (10 pages) | ~0.2s | Linear scaling |
| Large document (72 pages) | ~0.4s | Still interactive |

### Memory Usage

- Cutouts stored per-column in small vectors (typically 1-3 items)
- Cutouts cleared at column boundaries (no accumulation)
- Deferred paragraph layout only when cutouts exist

### When Performance May Degrade

Performance remains good for typical documents. Consider these scenarios:

1. **Many concurrent cutouts** (10+) - Linear search becomes noticeable
2. **Very narrow remaining width** - More line breaks needed
3. **Complex widow/orphan settings** - Additional layout passes

### Optimization Tips

```typst
// Good: Single masthead with content
#masthead(left, 3cm)[
  All sidebar content in one masthead
]

// Avoid: Many small wraps stacked vertically
// This creates many cutouts to process
#wrap[Item 1]
#wrap[Item 2]
#wrap[Item 3]
// ... many more
```

For documents with 10+ cutouts per column, performance is still acceptable but may become noticeable on older hardware.

---

## Complete Examples

### Magazine Article Layout

```typst
#set page(paper: "a4", margin: 2cm)
#set text(font: "Linux Libertine", size: 11pt)
#set par(justify: true)

#align(center)[
  #text(size: 24pt, weight: "bold")[The Future of Technology]
  #v(0.5em)
  #text(size: 12pt, fill: gray)[By Jane Smith | January 2024]
]

#v(1em)

#wrap(left, clearance: 1em)[
  #box(
    width: 5cm,
    stroke: 1pt + gray,
    inset: 10pt,
    radius: 4pt,
  )[
    #text(weight: "bold")[Key Takeaways]

    - AI advances rapidly
    - Privacy concerns grow
    - Regulation lags behind
    - Innovation continues
  ]
]

#lorem(100)

#wrap(right, clearance: 12pt)[
  #figure(
    rect(width: 6cm, height: 4cm, fill: luma(220))[
      #align(center + horizon)[Image placeholder]
    ],
    caption: [Technology adoption rates 2020-2024]
  )
]

#lorem(150)
```

### Two-Column Newsletter with Masthead

```typst
#set page(paper: "letter", margin: 1.5cm)
#set text(size: 10pt)

#show: columns.with(2, gutter: 1cm)

#masthead(left, 2.5cm)[
  #set text(size: 8pt)

  *IN THIS ISSUE*

  #v(0.5em)

  Technology ... 1

  Business ... 3

  Science ... 5

  #v(1fr)

  _Volume 12, Issue 3_
]

= Technology News

#lorem(80)

= Business Update

#lorem(80)

= Science Corner

#lorem(80)
```

### Pull Quote Design

```typst
#let pull-quote(body) = wrap(right, clearance: 1.5em)[
  #block(
    width: 6cm,
    inset: (x: 1em, y: 0.5em),
    stroke: (left: 3pt + blue),
    fill: blue.lighten(90%),
  )[
    #set text(style: "italic", size: 12pt)
    #body
  ]
]

#lorem(30)

#pull-quote[
  "The future belongs to those who believe in the beauty
  of their dreams." — Eleanor Roosevelt
]

#lorem(100)
```

### Image Gallery with Text

```typst
#let gallery-image(path, caption, side: right) = wrap(
  side,
  clearance: 10pt,
)[
  #figure(
    image(path, width: 4.5cm),
    caption: caption,
  )
]

= Photo Essay: Urban Architecture

#gallery-image("building1.jpg", [Downtown skyline], side: left)

#lorem(40)

#gallery-image("building2.jpg", [Historic district])

#lorem(40)

#gallery-image("building3.jpg", [Modern design], side: left)

#lorem(40)
```

---

## Quick Reference

### Wrap Syntax
```typst
#wrap(
  end,              // positional: start | end | left | right
  clearance: 1em,   // gap between content and text
  scope: column,    // column | parent
)[content]
```

### Masthead Syntax
```typst
#masthead(
  start,            // positional: start | end | left | right
  3cm,              // positional: width (required)
  clearance: 1em,   // gap between masthead and text
  scope: column,    // column | parent
)[content]
```

### Common Patterns

| Use Case | Element | Side |
|----------|---------|------|
| Right-aligned image | `wrap` | `right` or `end` |
| Left sidebar | `masthead` | `left` or `start` |
| Pull quote | `wrap` | `right` or `end` |
| Navigation panel | `masthead` | `left` or `start` |
| Figure with caption | `wrap` | varies |

---

## Troubleshooting

### Text not wrapping around content
- Ensure the wrap/masthead is placed *before* the text that should flow around it
- Check that content width leaves room for text

### Masthead not full height
- Masthead automatically fills column height; if it appears shorter, check for page breaks

### Content overlapping
- Increase `clearance` value
- Reduce content width
- Check `scope` setting in multi-column layouts

### PDF export errors
- See [PDF Export: Avoiding Tag Errors](#pdf-export-avoiding-tag-errors)
- Use `--no-pdf-tags` flag when compiling
- This is a known issue that will be addressed in a future update
