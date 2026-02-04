# Text-Flow Feature Testing Guide

This guide is for developers testing the text-flow feature (wrap and masthead elements) in a production-like environment.

## Repository

**URL:** https://github.com/TopdownSystems/typst-fork

**Branch:** `feature/text-flow`

## Setup

```bash
git clone https://github.com/TopdownSystems/typst-fork.git
cd typst-fork
git checkout feature/text-flow
cargo build --release
```

## Running Tests

### Unit Tests

```bash
# Masthead tests (19 tests)
cargo test -p typst-tests -- masthead --stages render

# Wrap tests (24 tests)
cargo test -p typst-tests -- wrap --stages render
```

### Compile Example Documents

```bash
# Compile all examples
for f in docs/TDSC-Doc\ and\ Examples/examples/typst/*.typ; do
  ./target/release/typst compile --no-pdf-tags "$f" "/tmp/$(basename "$f" .typ).pdf"
done

# Or compile individually
./target/release/typst compile --no-pdf-tags "docs/TDSC-Doc and Examples/examples/typst/sidebar0.typ" output.pdf
./target/release/typst compile --no-pdf-tags "docs/TDSC-Doc and Examples/examples/typst/masthead-demo.typ" output.pdf
./target/release/typst compile --no-pdf-tags "docs/TDSC-Doc and Examples/examples/typst/wrap-demo.typ" output.pdf
```

### Performance Benchmark

```bash
bash "docs/TDSC-Doc and Examples/examples/scripts/benchmark-typst.sh"

# Expected results: ~275-295ms for sidebar0.typ
```

## Example Documents

| File | Description |
|------|-------------|
| `sidebar0.typ` | Basic sidebar with masthead |
| `masthead-demo.typ` | Masthead feature showcase |
| `wrap-demo.typ` | Wrap feature showcase |
| `multipage-article.typ` | Multi-page document with wrap/masthead |
| `rtl-arabic-newsletter.typ` | RTL language support test |
| `test-50000.typ` | Large document (50,000 words) stress test |
| `perf-test-3500.typ` | Performance test document |

## Testing Checklist

### Basic Functionality

- [ ] `#wrap(right)[content]` places content on right with text flowing left
- [ ] `#wrap(left)[content]` places content on left with text flowing right
- [ ] `#masthead(right, 150pt)[content]` creates full-height sidebar
- [ ] `#masthead(left, 150pt)[content]` creates full-height sidebar on left
- [ ] `clearance` parameter controls gap between content and text

### Multi-page Behavior

- [ ] Paragraphs spanning pages render at full width on pages without cutouts
- [ ] Masthead only appears on its starting page (not subsequent pages)
- [ ] `overflow: "paginate"` allows masthead content to continue on next page
- [ ] `overflow: "clip"` (default) truncates masthead content that doesn't fit

### RTL Support

- [ ] `#set text(dir: rtl)` correctly mirrors wrap/masthead positioning
- [ ] `start`/`end` sides respect text direction
- [ ] `left`/`right` sides remain fixed regardless of text direction

### Column Layouts

- [ ] Wrap/masthead work correctly inside `#columns(2)[...]`
- [ ] `scope: "column"` (default) affects only the current column
- [ ] `scope: "parent"` spans across columns

### Edge Cases

- [ ] Empty wrap body doesn't crash
- [ ] Very long masthead content clips or paginates correctly
- [ ] Multiple wraps on same page work together
- [ ] Wrap inside nested containers

## Documentation

| Document | Location |
|----------|----------|
| User Guide | `docs/TDSC-Doc and Examples/TEXT-FLOW-GUIDE.md` |
| Known Issues | `docs/TDSC-Doc and Examples/README.md` |
| Development Log | `docs/TDSC-Doc and Examples/DEVELOPMENT_LOG.md` |
| Code Reviews | `docs/TDSC-Doc and Examples/CODE_REVIEW*.md` |

## Important Notes

- Always use `--no-pdf-tags` flag when compiling to avoid PDF tagging errors
- The `sandbox` tenant should be used for any integration testing
- Report issues with specific .typ file and steps to reproduce

## Contact

Report issues or questions to the repository maintainer.
