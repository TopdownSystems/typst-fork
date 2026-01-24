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
#wrap(side: right, clearance: 1em)[
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

### 2. Masthead Overflow on Empty/Short Pages

**Status:** Known issue, fix planned

**Symptom:** When a masthead is placed on a page with little or no text content, the masthead content may overflow beyond the page boundary instead of being truncated or paginated.

**Cause:** Mastheads are designed to extend the full height of the column/region. When the masthead content exceeds the available region height and there's no text to trigger natural page breaks, the masthead content overflows.

**Workaround:** Ensure pages with mastheads have sufficient text content, or manually constrain masthead content height:
```typst
#masthead(left, 3cm)[
  #block(height: 100%, clip: true)[
    // Masthead content that may be long
  ]
]
```

**Planned Fix:** Masthead content should be truncated (clipped) when it exceeds the available region height, rather than overflowing.

---

## Future Features

### 1. Masthead Truncation/Pagination

Implement proper handling when masthead content exceeds region height:
- Option A: Truncate (clip) content that doesn't fit
- Option B: Allow masthead content to paginate across pages
- May require a new parameter like `overflow: clip | paginate`

### 2. `first_page_only` Parameter for Masthead

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
