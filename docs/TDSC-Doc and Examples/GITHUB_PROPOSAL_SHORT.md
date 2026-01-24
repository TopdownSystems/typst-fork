# Proposal: Native Text Flow Implementation

Hi\! I'd like to propose implementing native text-flow support in Typst to address this issue. We've done extensive research and are prepared to contribute the implementation.  We believe our approach conforms to the ideas presented in Laurenz's blog on layout models. https://laurmaedje.github.io/posts/layout-models/.

## The Performance Problem

We're evaluating Typst to replace Apache FOP in our customer communications platform. While Typst dramatically outperforms FOP for standard layouts (\<50ms vs 200ms), the current plugin-based text wrapping solutions have a critical performance gap which will prevent us from using Typst:

**Benchmark: Newsletter with masthead column**

- Apache FOP: \~200ms  
- Typst (no wrap): \<50ms (FASTER)  
- Meander plugin: \~500ms (SLOWER)  
- Wrap-it plugin: \~1200ms (SLOWER)

**Root Cause:** Plugins operate in the script layer and repeatedly invoke Typst's layout engine for measurements. They can't access internal optimizations like `comemo` caching.

## Proposed Solution

Implement native text flow using Typst's relayout-based architecture:

### 1\. Region Cutout System

Extend regions to support rectangular exclusion zones that reduce available width at specific vertical positions:

```rust
pub struct RegionCutout {
    y_start: Abs, y_end: Abs,  // Vertical range
    side: CutoutSide,           // Start/End (LTR-aware)
    width: Abs,                 // Width reduction
    clearance: Abs,             // Spacing
}
```

### 2\. Variable-Width Line Breaking

Modify Knuth-Plass to accept a width function instead of fixed width:

```rust
fn linebreak(p: &Preparation, width_provider: &dyn WidthProvider) -> Vec<Line>
```

This allows different line widths at different vertical positions while maintaining Knuth-Plass optimization.

### 3\. User-Facing API

```
// General-purpose wrap
#wrap(side: right)[
  #image("photo.jpg", width: 80pt)
]
#lorem(100)

// Newsletter masthead
#masthead(width: 80pt, side: left)[
  *Newsletter* Issue 42
]
= Article Title
#lorem(200)
```

### 4\. Relayout Integration

Uses existing `Stop::Relayout` mechanism \- layout wrap content, create cutouts, trigger relayout for paragraphs to use cutouts. Converges in 2-3 iterations.

## Implementation Plan

Phased approach with independent PRs:

1. **Region Cutout Foundation** \- Types and APIs, zero functional changes  
2. **Variable-Width Line Breaking** \- Knuth-Plass enhancement, backward compatible  
3. **Wrap Element** \- Element definition  
4. **Flow Integration** \- End-to-end functionality  
5. **Masthead Specialization** \- Newsletter-specific element  
6. **Performance Optimization** \- Hit \<200ms target  
7. **Documentation** \- User guide and examples

## This Approach:

- **Aligns with architecture:** Uses relayout model from "four architectures" blog post  
- **Performance-first:** Native implementation removes script overhead  
- **Backward compatible:** All existing tests continue passing  
- **Incremental:** Each phase independently reviewable  
- **Extensible:** Foundation for future irregular shape support

## Research Done:

- Reviewed layout engine source (`typst-layout`, `typst-library`)  
- Analyzed Knuth-Plass implementation  
- Studied relayout mechanism and `comemo` integration  
- Created detailed specs (architecture, implementation guide, test strategy)  
- Benchmarked current solutions

## Security Considerations

We're mindful of security implications including:

- Relayout iteration limits to prevent infinite loops  
- Input validation for extreme parameter values  
- Memory bounds on cutout collections  
- Overflow-safe arithmetic in width calculations

Implementation will follow Typst's existing security patterns and include comprehensive edge-case testing.

## Feedback

Before implementing, would appreciate maintainer input on:

1. Does region cutout approach align with planned architecture?  
2. Are proposed `wrap`/`masthead` APIs appropriate?  
3. Any specific contribution requirements beyond CONTRIBUTING.md?

We're committed to implementing this if the approach fits Typst's vision. The issue has 67 positive reactions showing strong community demand.

**About us:** Team behind INTOUCH, a CCM platform migrating from Apache FOP to Typst. Happy to contribute back\!

---

Full technical specification available if helpful. Happy to discuss details, provide benchmarks, or clarify anything\!
