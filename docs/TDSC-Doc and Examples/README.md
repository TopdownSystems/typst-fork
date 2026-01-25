# TDSC Text Flow Feature Documentation

This folder contains all development documentation, code reviews, and examples for the Typst text-flow feature (wrap and masthead elements).

## Contents

| File | Description |
|------|-------------|
| `DEVELOPMENT_LOG.md` | Complete development history and timeline |
| `TEXT-FLOW-GUIDE.md` | User guide for wrap and masthead elements |
| `CODE_REVIEW*.md` | Code review documentation (v1-v4) |
| `CLAUDE_CODE_SPEC.md` | Original feature specification |
| `GITHUB_PROPOSAL*.md` | Upstream contribution proposals |
| `examples/typst/` | Source .typ files demonstrating features |
| `examples/pdfResults/` | Compiled PDF outputs |

## Quick Start

```typst
// Simple wrap example - image with text flowing around it
#wrap(right, clearance: 1em)[
  #image("photo.jpg", width: 4cm)
]
Lorem ipsum dolor sit amet...

// Simple masthead example - full-height sidebar
#masthead(left, 3cm)[
  *Sidebar Content*

  Navigation or reference material here.
]
Main article text flows to the right of the masthead.
```

See `TEXT-FLOW-GUIDE.md` for comprehensive usage documentation.

## Building Examples

Compile the example files using:

```bash
cd examples/typst
typst compile --no-pdf-tags sidebar0.typ ../pdfResults/sidebar0.pdf
```

**Note:** The `--no-pdf-tags` flag is required due to a known issue with PDF tagging (see below).

---

## Known Issues

### 1. PDF Tagging Error (Workaround Available)

**Status:** Known issue, workaround available

**Symptom:** When exporting to PDF without the `--no-pdf-tags` flag, you may encounter:
```
Error [pdf]: internal error: parent group
```

**Cause:** The PDF tagging system hasn't been updated to handle wrap and masthead elements. The tag tree builder expects a specific parent-child relationship that content flowing around cutouts doesn't satisfy.

**Workaround:** Use the `--no-pdf-tags` flag when compiling:
```bash
typst compile --no-pdf-tags document.typ output.pdf
```

PDFs will render correctly but won't have accessibility structure tags.

---

### 2. ~~Masthead Overflow on Empty/Short Pages~~ (FIXED)

**Status:** FIXED in January 2026

**Original Issue:** When a masthead was placed on a page with little or no text content, the masthead content would overflow or cause an infinite loop.

**Solution:** Added `overflow` parameter to masthead with two modes:
- `clip` (default): Truncates content that exceeds available height and emits a warning
- `paginate`: Attempts to continue on subsequent pages (requires sufficient flowing text)

```typst
#masthead(60pt, overflow: "clip")[
  Long content that will be truncated if it doesn't fit...
]
```

---

### 3. Wraps at Page Boundaries

**Status:** Known limitation

**Symptom:** When wrap elements don't fit on a page and spill to the next page, text that was already laid out after the wrap may overlap with the wrap content on the new page.

**Cause:** When wraps are queued for the next page, text that was laid out in the same paragraph on the original page also spills over. However, this spilled text was already formatted without knowledge of the cutouts that will appear on the new page.

**Workaround:** Ensure wraps have enough space on their starting page:
```typst
// Add a page break before wrap-heavy sections if needed
#pagebreak()

#wrap(right)[...]
Text that should flow around the wrap...
```

Alternatively, place wraps earlier in the content to ensure they fit on their original page, or use mastheads (which have better page boundary handling).

**Note:** This is a complex layout interaction that would require significant changes to the flow engine to resolve fully. The issue is most visible when multiple wraps appear near page boundaries.

---

### 4. Masthead Cutout Persists in Spilled Paragraphs

**Status:** Known limitation

**Symptom:** When a paragraph starts on a page with a masthead and continues to the next page (paragraph spill), the text on the second page continues to flow in a narrow column as if the masthead were still present, even though the masthead only appears on page 1.

**Cause:** When paragraphs are laid out with mastheads, they use variable-width line breaking that accounts for the masthead cutout. If the paragraph spans multiple pages, the already-formatted lines are saved and reused on subsequent pages. These pre-formatted lines retain their narrow widths from page 1.

**Workaround:** For documents with mastheads and very long paragraphs:
```typst
// Option 1: Break long paragraphs into smaller ones
#masthead(right, 100pt)[Sidebar]
First paragraph with moderate length.

Second paragraph continues with moderate length.

// Option 2: Use a page break before masthead section
#pagebreak()
#masthead(right, 100pt)[Sidebar]
Content that fits on this page...
```

**Note:** This is a fundamental limitation of the paragraph spill mechanism. Fixing it would require re-laying out spilled paragraphs with current page cutouts, which has performance implications.

---

### 5. Left-Side Mastheads with Headings

**Status:** Known limitation

**Symptom:** When using a left-side masthead, heading text may overlap with the masthead content. This is especially noticeable with numbered headings (e.g., "1. Introduction") where the number extends into the masthead's cutout region.

**Cause:** Typst headings are positioned at the left edge of the available text area. When a left-side masthead creates a cutout, the heading text starts at the cutout boundary, but the heading's visual rendering (especially with numbered headings using hanging indent style) may extend into the cutout region.

**Workarounds:**
1. Use a right-side masthead instead of left-side
2. Disable heading numbering when using left-side mastheads
3. Ensure wraps on the same side as the masthead are avoided

```typst
// Prefer right-side mastheads to avoid heading conflicts
#masthead(right, 3cm)[
  Sidebar content...
]

// If using left masthead, disable heading numbering
// #set heading(numbering: "1.")  // Commented out
```

---

## Future Features

### 1. `first_page_only` Parameter for Masthead

Allow mastheads to appear only on the first page of a document or section:
```typst
#masthead(left, 3cm, first_page_only: true)[
  *Article Info*
  - Author: Jane Doe
  - Published: 2024
]
```

**Challenge:** Requires integration with Typst's introspection system to track page context.

### 3. PDF Tag Support

Update the PDF export code to properly handle wrap and masthead elements:
- Recognize wrap/masthead in the tag tree
- Generate appropriate tag structures for floating content
- Handle parent-child relationships correctly for flowed content

### 4. Vertical Text Direction Support

Document and test behavior with vertical text directions (e.g., traditional Chinese/Japanese layouts). Currently untested but the logical positioning system should support it.

### 5. Column-Spanning Wraps

Allow wrap elements to span multiple columns in multi-column layouts:
```typst
#show: columns.with(2)
#wrap(side: right, scope: parent)[
  // This image spans both columns
  #image("wide-photo.jpg", width: 100%)
]
```

---

## Development History

See `DEVELOPMENT_LOG.md` for the complete development timeline, including:
- Phase 1-7 implementation details
- Bug fixes (RTL positioning, page break infinite loop)
- Performance optimization work
- Code review findings and remediations

## Contributing

This feature is maintained in the TopdownSystems/typst-main repository on the `feature/text-flow` branch. For upstream contribution to typst/typst, see `GITHUB_PROPOSAL.md`.
