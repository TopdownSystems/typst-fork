# Proposal: Native Text Flow Implementation for Typst

## Executive Summary

I'd like to propose implementing native text-flow support (wrap text around images/content) in Typst's layout engine. This addresses issue #5181 and related discussions, providing the functionality that plugins like `wrap-it` and `meander` currently attempt to solve, but with significantly better performance and tighter integration with Typst's layout system.

**Key Points:**
- Performance-critical: Current plugins are 2.5-6x slower than alternatives due to operating in the script layer
- Aligns with Typst's roadmap: Leverages the relayout-based architecture described in recent blog posts
- Phased implementation: Backward-compatible changes that can be reviewed incrementally
- Willing to contribute: We have detailed specifications and are prepared to implement

## The Performance Problem

We've been evaluating Typst to replace Apache FOP in our customer communications platform (INTOUCH). While Typst dramatically outperforms FOP for standard text layout (<50ms vs 200ms), text wrapping reveals a critical gap:

**Performance Benchmarks (Newsletter-style document with masthead):**
- Apache FOP: ~200ms
- Typst (pure text, no wrap): <50ms (FASTER)
- Meander plugin: ~500ms (SLOWER)
- Wrap-it plugin: ~1200ms (SLOWER)

**Why Plugins Are Slow:**

Current plugins operate in Typst's script layer and must:
1. Repeatedly invoke Typst's layout engine to measure text
2. Create trial layouts to determine text flow
3. Work through the public API with associated overhead
4. Cannot access internal optimizations like the layout cache (`comemo`)

This is analogous to implementing custom layout in CSS/JavaScript versus having browser-native flexbox. The functionality works, but performance suffers because the implementation can't access the layout engine's internals.

## Proposed Technical Approach

### Foundation: Region Cutout System

Extend Typst's region model to support rectangular exclusion zones (cutouts) that reduce available width at specific vertical positions. This aligns with discussions in Laurenz Mädje's blog post about lifting the "rectangular-only restriction on regions."

```rust
// Conceptual types (pseudo-code)
pub struct RegionCutout {
    y_start: Abs,      // Vertical range where cutout applies
    y_end: Abs,
    side: CutoutSide,  // Start (left in LTR) or End (right in LTR)
    width: Abs,        // How much to reduce available width
    clearance: Abs,    // Spacing around the cutout
}

impl Region {
    pub fn width_at(&self, y: Abs, cutouts: &[RegionCutout]) -> WidthInfo {
        // Query available width at vertical position
        // considering active cutouts
    }
}
```

**Backward Compatibility:** Empty cutouts array returns full width (current behavior).

### Variable-Width Line Breaking

Modify the Knuth-Plass line breaking algorithm to accept a width provider instead of fixed width:

```rust
// Current: Fixed width for all lines
fn linebreak(p: &Preparation, width: Abs) -> Vec<Line>

// Proposed: Width function that varies by vertical position
fn linebreak(p: &Preparation, width_provider: &dyn WidthProvider) -> Vec<Line>
```

The width provider queries cutouts based on cumulative paragraph height, allowing different line widths at different vertical positions. This maintains the elegant Knuth-Plass optimization while supporting variable widths.

### User-Facing API

Two new elements following Typst's design patterns:

**1. Wrap Element** (general-purpose):
```typst
#wrap(
  side: right,           // or left, start, end
  clearance: 0.5em,
)[
  #image("photo.jpg", width: 80pt)
]

#lorem(100)  // Text flows around the image
```

**2. Masthead Element** (newsletter-style):
```typst
#masthead(
  width: 80pt,
  side: left,
)[
  *Newsletter*
  Issue 42
  January 2026
]

= Article Title
#lorem(200)  // Flows around masthead column
```

### Relayout Mechanism

Leverages Typst's existing relayout infrastructure:

1. **Initial Pass:** Layout wrap content to determine size
2. **Create Cutout:** Generate `RegionCutout` from positioned wrap
3. **Relayout:** Trigger `Stop::Relayout` so paragraphs can use cutouts
4. **Convergence:** Typically 2-3 iterations until positions stabilize

This aligns with Typst's pure, functional layout engine where "regions require relayout" for great typesetting.

## Implementation Plan

We propose a phased approach with independent, reviewable PRs:

**Phase 1: Region Cutout Foundation**
- Add `RegionCutout`, `CutoutSide`, `WidthInfo` types
- Extend `Region` with width query methods
- Comprehensive unit tests
- Zero functional changes to existing layouts

**Phase 2: Variable-Width Line Breaking**
- Add `WidthProvider` trait abstraction
- Modify `linebreak_optimized` to track cumulative height
- Update `Line` struct with horizontal offset
- Maintain backward compatibility with wrapper function

**Phase 3: Wrap Element Definition**
- Create `WrapElem` with parameters (side, clearance, height, etc.)
- Define element interface (no layout behavior yet)
- Documentation and examples

**Phase 4: Flow Layout Integration**
- Collect wrap elements in flow layout
- Generate cutouts from wrap frames
- Connect to paragraph layout via width providers
- Implement relayout mechanism
- End-to-end functionality

**Phase 5: Masthead Specialization**
- Add `MastheadElem` as specialized wrap
- Implement first-page-only behavior
- Newsletter-specific ergonomics

**Phase 6: Performance Optimization**
- Cutout indexing (binary search vs linear)
- Relayout convergence detection
- Fast-path optimization for empty cutouts
- Benchmark suite

**Phase 7: Documentation**
- User guide with examples
- API documentation
- Contributing notes for future developers

## Why This Approach

**1. Aligns with Typst's Architecture**
- Uses the relayout-based model described in the blog post "The four architectures of layout"
- Leverages `comemo` for incremental compilation
- Respects the pure, functional layout engine design

**2. Performance-First**
- Native implementation removes script-layer overhead
- Direct access to layout engine internals
- Achieves target <200ms (vs current 500-1200ms)

**3. Incremental & Reviewable**
- Each phase delivers testable functionality
- Backward compatibility maintained throughout
- Can be reviewed/merged independently

**4. Extensible Foundation**
- Region cutouts enable future irregular shape support
- Width provider abstraction allows other use cases
- Clean API for users to build upon

## Research & References

We've conducted extensive research on Typst's architecture and this proposal:

**Reviewed:**
- Typst layout engine source code (`crates/typst-layout/src/`)
- Region and flow composition implementation
- Knuth-Plass line breaking algorithm in `linebreak.rs`
- Relayout mechanism and `Stop::Relayout` usage
- Blog post: "The four architectures of layout" (relayout approach)

**Documented:**
- Detailed architecture design (40+ pages)
- Phase-by-phase implementation guide
- Performance benchmarking methodology
- Test strategy and acceptance criteria

**Available:** Full specifications at https://github.com/[org]/typst-text-flow-project

## Community Impact

Issue #5181 has 67 positive reactions and 12 heart reactions, demonstrating strong community demand. Native support would:

- Enable newsletter, magazine, and book layouts
- Eliminate need for manual text splitting workarounds
- Remove dependency on slow third-party plugins
- Provide foundation for future irregular shape support

## Security Considerations

We're mindful of security implications in this implementation:

**Denial of Service Prevention:**
- Hard limits on relayout iterations (max 3-5) to prevent infinite loops
- Reasonable bounds on wrap element count per column (max 100)
- Convergence detection to stop unnecessary relayout cycles

**Input Validation:**
- Validate clearance, width, and height parameters for reasonable ranges
- Reject extreme coordinate values that could cause overflow
- Limit cutout width to 90% of region width maximum

**Numeric Safety:**
- Saturating arithmetic for all Abs operations
- Width calculations always clamped to zero minimum (never negative)
- Proper handling of overlapping cutouts

**Cache Correctness:**
- Consistent Hash implementation using `to_raw()` for Abs values
- PartialEq/Eq consistency with Hash for comemo cache integrity

**Inherited Protections:**
- Rust memory safety (no buffer overflows, use-after-free)
- Typst's existing document compilation timeouts
- Resource limits already enforced by Typst

Implementation will follow Typst's existing security patterns and include comprehensive edge-case testing with extreme values, pathological documents, and DOS scenarios.

## Request for Feedback

Before proceeding with implementation, we'd appreciate feedback on:

1. **Technical Approach:** Does the region cutout system align with planned architecture?
2. **API Design:** Are the proposed `wrap` and `masthead` elements appropriate?
3. **Implementation Scope:** Should we target all phases or subset for initial PR?
4. **Contribution Process:** Any specific requirements beyond CONTRIBUTING.md?

We're committed to implementing this if the approach aligns with Typst's vision and maintainers are supportive of the contribution.

## About Us

We're the team behind INTOUCH, a cloud-native customer communications platform serving financial services and insurance clients. We're migrating from Apache FOP to Typst for document generation and are eager to contribute back to the project.

---

**Questions or concerns?** Happy to discuss technical details, provide additional benchmarks, or clarify any aspect of the proposal.
