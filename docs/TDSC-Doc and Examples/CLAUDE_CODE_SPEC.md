# Typst Text Flow Implementation - Claude Code Specification

## Project Context

**Objective**: Implement native text-flow (wrap around images) and masthead column functionality in the Typst compiler fork to achieve performance targets.

**Performance Goal**: Match or beat Apache FOP's 200ms baseline for document generation with text wrap
- Current Plugin Performance: Meander (500ms), Wrap-it (1200ms)
- Target: <200ms for typical newsletter with wraps
- Why native is needed: Plugins operate in Typst script layer, not native Rust

**Technical Approach**: Use Typst's relayout-based model with region cutout system
- Typst's layout engine is 100% pure (no mutable state)
- `comemo` crate caches results when inputs match
- Relayout approach: multiple passes converge to final layout

**Key References**:
- Past conversations analyzed Typst architecture and contribution policies
- Existing PROJECT_SPEC.md, ARCHITECTURE.md, IMPLEMENTATION_PHASES.md contain detailed design
- Typst repository structure under analysis: `crates/typst-layout/src/` and `crates/typst-library/src/layout/`
- **Laurenz's Blog Post**: https://laurmaedje.github.io/posts/layout-models/ (critical reading)

---

## Architectural Insights from Typst Creator's Blog

### Key Blog Findings (January 2025)

Laurenz Mädje's blog post "Layout Models" provides critical context for this implementation:

#### 1. Cutouts are the Right Approach
> "To accommodate [side-floating elements], the restriction that regions cannot have cutouts would need removal."

This directly validates our region cutout approach. The blog confirms cutouts are the intended mechanism for text flow.

#### 2. Pure Layout Engine
> "Typst's layout engine finally been fixed recently, making layout 100% pure and free of side effects."

**Implications for our code**:
- All layout functions must be pure (deterministic, no side effects)
- Hash implementations must be deterministic for `comemo` memoization
- Cutouts passed as parameters, not stored in mutable state
- The engine is now parallelized - our code must be thread-safe

#### 3. Relayout Model vs TeX
The blog explains two fundamentally different approaches:
- **TeX**: "movability of boxes" - positions determined after layout
- **Typst**: "placement first" - elements can react to their positions

**Key insight**: These approaches are "fundamentally at odds":
> "When things can move after being laid out, they cannot know their own position. When things can react to their exact position during layout, they cannot be moved afterwards."

Typst's relayout model (multiple passes until convergence) allows text to flow around positioned elements by re-running layout with updated cutout information.

#### 4. TeX wrapfig Limitations
The blog shows how TeX's `wrapfig` struggles:
> "extra unoccupied space next to the final lines" when figures break across pages

This happens because TeX can't know vertical line positions beforehand. Our implementation can do better by iterating until stable.

#### 5. Convergence Challenges
The blog acknowledges uncertainty about oscillation:
> "I am not sure whether this [binary search approach] could cause the result to oscillate between two positions."

**Our implementation must**:
- Detect when positions stabilize
- Have maximum iteration limits
- Consider damping or binary search for positioning

#### 6. Current Restrictions We're Changing
The blog notes:
- "All regions in a sequence must currently have the same width" - cutouts change this
- "Regions can currently only be rectangular" - cutouts make them effectively non-rectangular

#### 7. Incremental Compilation Strategy
The blog proposes:
> "Are there at least 4cm left?" rather than "Is there 4.2cm left?"

This suggests width queries could be approximate for better caching. Consider threshold-based queries in Phase 6 optimization.

### Architecture Validation

Our Phase 1 implementation aligns with these insights:

| Design Decision | Blog Validation |
|----------------|-----------------|
| `RegionCutout` types | Directly addresses "no cutouts restriction" |
| `WidthInfo` for queries | Enables "react to position" model |
| Manual `Hash` impl with `to_raw().to_bits()` | Required for pure/parallelized layout |
| Cutouts passed separately (not in Region) | Region is Copy, parallelization-friendly |
| `width_at()` and `width_in_range()` methods | Support relayout convergence |

---

## Development Principles

1. **Backward Compatibility**: All changes must not break existing tests
2. **Incremental Development**: Each phase produces a working, testable unit
3. **Test-First**: Write tests before implementation when feasible
4. **Performance Neutral**: No regression for documents without wraps
5. **Code Quality**: Follow Typst's Rust standards (rustfmt, clippy clean)
6. **Purity**: All layout code must be pure (no side effects, deterministic)
7. **Parallelization-Safe**: Code must work with Typst's parallel layout engine

---

## Phase 1: Region Cutout Foundation ✅ COMPLETED

**Status**: Implemented and tested

**Goal**: Extend the region system to support exclusion zones (cutouts) while maintaining backward compatibility

**Branch**: `feature/region-cutouts`
**PR Title**: "Add region cutout support for variable-width layout"

### What Was Delivered

A foundational type system and API for representing rectangular exclusion zones within layout regions.

### Files Created

```
crates/typst-library/src/layout/
├── cutout.rs (NEW)     # Core cutout types - 28 unit tests
└── mod.rs              # Added exports
```

### Files Modified

```
crates/typst-library/src/layout/
└── regions.rs          # Added width_at() and width_in_range() methods
```

### Implementation Summary

#### Core Types (`cutout.rs`)

```rust
/// Rectangular exclusion zone in a region
pub struct RegionCutout {
    pub y_start: Abs,         // Top of cutout (region-relative)
    pub y_end: Abs,           // Bottom of cutout
    pub side: CutoutSide,     // Which side (start/end)
    pub width: Abs,           // Cutout width
    pub clearance: Abs,       // Additional spacing
}

pub enum CutoutSide {
    Start,  // Left in LTR, Right in RTL
    End,    // Right in LTR, Left in RTL
}

/// Width information at a vertical position
pub struct WidthInfo {
    pub available: Abs,        // Width available for content
    pub start_offset: Abs,     // Offset from start edge
    pub end_offset: Abs,       // Offset from end edge
}
```

#### Key Functions

- `width_at(region_width, y, cutouts, dir)` - Query width at single y position
- `width_in_range(region_width, y_start, y_end, cutouts, dir)` - Query minimum width in range
- `cutouts_at_y()` - Helper to find active cutouts at position
- `cutouts_in_range()` - Helper to find cutouts overlapping range

#### Test Coverage (28 tests)

- CutoutSide: opposite, is_left for LTR/RTL
- RegionCutout: new, total_width, contains_y, overlaps_range, height, hash determinism
- WidthInfo: full, new, fits, is_full
- width_at: no cutouts, end cutout, start cutout, both sides, multiple same side, RTL, outside range, never negative
- width_in_range: no cutouts, partial overlap, no overlap, multiple cutouts
- Helpers: cutouts_at_y, cutouts_in_range

### Acceptance Criteria - All Met

- [x] All new types compile without warnings
- [x] All 28 unit tests pass
- [x] Existing `typst-library` tests pass (60 tests, no regressions)
- [x] Code is formatted with `cargo fmt`
- [x] No clippy warnings
- [x] Documentation comments on all public APIs
- [x] Hash implementations use `to_raw().to_bits()` for `comemo` compatibility

### Testing Commands

```bash
# Build library
cargo build -p typst-library

# Run cutout unit tests
cargo test -p typst-library cutout

# Verify no regressions
cargo test -p typst-library

# Format and lint
cargo fmt
cargo clippy -p typst-library
```

---

## Phase 2: Variable-Width Line Breaking Infrastructure ✅ COMPLETED

**Status**: Implemented and tested

**Goal**: Modify paragraph layout to support variable-width lines via width provider abstraction

**Branch**: `feature/variable-width-linebreak`
**PR Title**: "Add width provider abstraction for variable-width line breaking"

### What This Phase Delivers

A width provider trait that abstracts width queries, allowing the Knuth-Plass line breaking algorithm to work with both fixed widths (current behavior) and variable widths (for cutouts).

### Files to Create

```
crates/typst-layout/src/inline/
└── width_provider.rs (NEW)    # Width provider trait and implementations
```

### Files to Modify

```
crates/typst-layout/src/inline/
├── linebreak.rs               # Accept WidthProvider instead of fixed width
├── line.rs                    # Add x_offset field to Line
├── mod.rs                     # Export new types
└── finalize.rs                # Apply line offsets when building frames
```

### Implementation Specification

#### 1. Width Provider Abstraction (`width_provider.rs`)

**Trait Definition**:
```rust
/// Provides width information for line breaking at different heights
pub trait WidthProvider {
    /// Get available width at cumulative height from paragraph start
    fn width_at(&self, cumulative_height: Abs) -> WidthInfo;

    /// Get base width for optimization hints
    fn base_width(&self) -> Abs;
}

/// Fixed-width provider (current behavior, default)
pub struct FixedWidth(pub Abs);

/// Variable-width provider using cutouts
pub struct CutoutWidth<'a> {
    pub region_width: Abs,
    pub cutouts: &'a [RegionCutout],
    pub y_offset: Abs,      // Paragraph start position in region
    pub dir: Dir,            // Text direction for side interpretation
}
```

**Implementation Notes**:
- `FixedWidth` always returns same `WidthInfo` (zero cost abstraction)
- `CutoutWidth` queries cutouts at `y_offset + cumulative_height`
- Both implementations must be efficient (called frequently during line breaking)
- Must be pure - no side effects, deterministic results

#### 2. Update Line Breaking (`linebreak.rs`)

**Modified Signature**:
```rust
// OLD:
fn linebreak_optimized<'a>(
    engine: &Engine,
    p: &'a Preparation<'a>,
    width: Abs,  // Fixed width
) -> Vec<Line<'a>>

// NEW:
fn linebreak_optimized<'a>(
    engine: &Engine,
    p: &'a Preparation<'a>,
    width_provider: &dyn WidthProvider,  // Abstracted width
) -> Vec<Line<'a>>
```

**Key Changes**:
- Add `cumulative_height: Abs` field to `Entry` struct (tracks height for width queries)
- In breakpoint iteration: query `width_provider.width_at(pred.cumulative_height)`
- Use `width_info.available` for Knuth-Plass cost calculations
- Store `width_info.start_offset` for line positioning
- Update `cumulative_height` when adding entries: `pred.height + new_line.height()`

#### 3. Update Line Struct (`line.rs`)

**Add Field**:
```rust
pub struct Line<'a> {
    // ... existing fields ...

    /// Horizontal offset from region start (for cutout avoidance)
    pub x_offset: Abs,
}
```

#### 4. Backward Compatibility

**Default Behavior**:
```rust
// Maintain existing API with default fixed-width behavior
pub fn linebreak<'a>(
    engine: &Engine,
    p: &'a Preparation<'a>,
    width: Abs,  // Keep old signature
) -> Vec<Line<'a>> {
    linebreak_with_provider(engine, p, &FixedWidth(width))
}
```

### Acceptance Criteria

- [x] `WidthProvider` trait and implementations compile
- [x] Line breaking works with fixed width (no regression)
- [x] Line breaking works with cutout-based variable widths
- [x] Line offsets correctly applied in finalization
- [x] All existing inline layout tests pass
- [x] New unit tests pass (12 tests for width_provider)
- [x] Performance neutral for fixed-width case (uses FixedWidth provider)
- [x] Code formatted and clippy clean
- [x] All code is pure (no side effects)

### Implementation Summary

**Files Created:**
- `crates/typst-layout/src/inline/width_provider.rs` - Contains `WidthProvider` trait, `FixedWidth` and `CutoutWidth` implementations with 12 unit tests

**Files Modified:**
- `crates/typst-layout/src/inline/mod.rs` - Added width_provider module
- `crates/typst-layout/src/inline/linebreak.rs` - Added `linebreak_with_provider()` function and modified simple/optimized line breaking to use width provider
- `crates/typst-layout/src/inline/line.rs` - Added `x_offset` field to `Line` struct, updated `line()` and `commit()` functions

**Key Design Decisions:**
- For variable-width (cutouts present), optimized line breaking falls back to simple algorithm
- For constant width (no cutouts), uses full Knuth-Plass optimization
- Line height is estimated at 1.2x font size for cumulative height tracking
- `x_offset` is applied in `commit()` function alongside existing hanging indent logic

### Notes for Claude Code

- The Knuth-Plass algorithm is complex - minimize changes to its core logic
- Focus on passing width provider and tracking cumulative height
- DO NOT expose this to Typst users yet - this is internal infrastructure
- Keep backward compatibility with existing linebreak callers
- Ensure all functions remain pure for parallelization

---

## Phase 3: Wrap Element Definition ✅ COMPLETED

**Status**: Implemented and tested

**Goal**: Create the `wrap` element and basic element infrastructure without flow integration

**Branch**: `feature/wrap-element`
**PR Title**: "Add wrap element for text flow layout"

### What This Phase Delivers

The user-facing `wrap` element definition, parameter handling, and style resolution. This phase does NOT implement layout behavior - it only defines the element interface.

### Files to Create

```
crates/typst-library/src/layout/
└── wrap.rs (NEW)              # WrapElem definition
```

### Implementation Specification

#### 1. Wrap Element (`wrap.rs`)

**Element Definition**:
```rust
/// Places content to the side with text flowing around it.
#[elem(scope, Locatable, Tagged)]
pub struct WrapElem {
    /// Which side to place the wrapped content
    #[positional]
    #[default(OuterHAlignment::End)]
    pub side: OuterHAlignment,

    /// The content to wrap text around
    #[required]
    pub body: Content,

    /// Clearance between wrapped content and flowing text
    #[default(Em::new(0.5).into())]
    pub clearance: Length,

    /// Maximum height the wrap should occupy
    pub height: Smart<Length>,

    /// Vertical alignment of content within wrap area
    #[default(Alignment::TOP)]
    pub align: Alignment,

    /// Scope of the wrap placement (column or parent)
    pub scope: PlacementScope,
}
```

### Acceptance Criteria

- [x] `WrapElem` compiles and is accessible in Typst documents
- [x] All parameters have correct types and defaults
- [x] Documentation is complete with examples
- [x] Element shows up in standard library exports
- [x] Code formatted and clippy clean

### Implementation Summary

**Files Created:**
- `crates/typst-library/src/layout/wrap.rs` - Contains `WrapElem` definition with comprehensive documentation and examples

**Files Modified:**
- `crates/typst-library/src/layout/mod.rs` - Added `mod wrap;`, `pub use self::wrap::*;`, and registered `WrapElem` in `define()` function

**WrapElem Fields:**
- `side: OuterHAlignment` (positional, default: `End`) - Which side to place wrapped content
- `body: Content` (required) - The content to wrap text around
- `clearance: Length` (default: `0.5em`) - Spacing between wrapped content and flowing text
- `scope: PlacementScope` - Scope of wrap placement (column or parent)

**Key Functions:**
- `cutout_side(&self, styles: StyleChain, dir: Dir) -> CutoutSide` - Converts alignment to cutout side based on text direction
- `outer_h_alignment_to_cutout_side(side, dir)` - Standalone helper for cleaner testing

**Unit Tests (2 tests):**
- `test_cutout_side_ltr` - Tests alignment conversion for LTR text
- `test_cutout_side_rtl` - Tests alignment conversion for RTL text

**Design Decisions:**
- Used `OuterHAlignment` (Start/End/Left/Right) instead of custom enum to leverage existing Typst alignment system
- Element includes comprehensive doc comments with Typst example code
- Used `#[elem(Locatable, Tagged)]` for proper introspection support
- Kept minimal fields for now (height/align can be added later if needed)

---

## Phase 4: Flow Layout Integration

**Goal**: Integrate wrap elements into the flow layout system with cutout generation and relayout

**Branch**: `feature/wrap-layout-integration`
**PR Title**: "Implement wrap element layout behavior in flow composition"
**Estimated Complexity**: High (5-7 days)

### What This Phase Delivers

Complete wrap layout behavior: collection of wrap elements, cutout generation, integration with paragraph layout via width providers, and relayout mechanism.

### Key Insight from Blog

The relayout mechanism is critical. From the blog:
> "Typst resolves these inherently cyclical dependencies through the introspection loop: The layout phase runs in a loop until the results stabilize."

We leverage this existing mechanism - wrap elements trigger relayout with updated cutout information until positions converge.

### Files to Create

```
crates/typst-layout/src/flow/
└── wrap.rs (NEW)              # Wrap-specific flow logic
```

### Files to Modify

```
crates/typst-layout/src/flow/
├── collect.rs                 # Add WrapChild variant
├── compose.rs                 # Add wrap handling in composer
├── distribute.rs              # Handle wrap in distribution
└── mod.rs                     # Export wrap types
```

### Implementation Specification

#### 1. Collection Phase (`collect.rs`)

**Add Wrap Child Variant**:
```rust
/// A collected child in flow layout
pub enum Child<'a> {
    // ... existing variants ...
    Wrap(WrapChild<'a>),
}

pub struct WrapChild<'a> {
    pub elem: &'a Packed<WrapElem>,
    pub side: CutoutSide,
    pub clearance: Abs,
    pub max_height: Option<Abs>,
    pub locator: Locator<'a>,
}
```

#### 2. Composer Integration (`compose.rs`)

**Wrap Handling Method**:
```rust
impl Composer {
    pub fn wrap(
        &mut self,
        wrap: &WrapChild,
        regions: &Regions,
        current_y: Abs,
    ) -> FlowResult<()> {
        // Layout wrap content
        let frame = wrap.layout(engine, regions.base())?;

        // Create cutout
        let cutout = RegionCutout::new(
            current_y,
            current_y + frame.height(),
            wrap.side,
            frame.width(),
            wrap.clearance,
        );

        // Add to active cutouts
        self.column_cutouts.push(cutout);

        // Trigger relayout
        Err(Stop::Relayout(PlacementScope::Column))
    }
}
```

#### 3. Convergence Handling

**Important**: Must handle convergence to avoid infinite loops:
```rust
// Track previous cutout positions
// If positions stabilize (within epsilon), stop relayout
// Maximum iteration limit (Typst uses 5 for introspection)
```

### Acceptance Criteria

- [ ] Wrap elements collected during flow layout
- [ ] Text flows around wrap content
- [ ] Multiple wraps work correctly
- [ ] Relayout converges (no infinite loops)
- [ ] All test cases pass
- [ ] No regressions in existing flow tests

---

## Phase 5: Masthead Specialization

**Goal**: Implement masthead element as specialized wrap for newsletter-style layouts

**Branch**: `feature/masthead-element`
**PR Title**: "Add masthead element for newsletter-style column layout"
**Estimated Complexity**: Low-Medium (2-3 days)

### Files to Create

```
crates/typst-library/src/layout/
└── masthead.rs (NEW)          # MastheadElem definition
```

### Implementation Specification

**Element Definition**:
```rust
/// A masthead column for newsletter-style layouts.
#[elem(scope)]
pub struct MastheadElem {
    #[default(OuterHAlignment::Start)]
    pub side: OuterHAlignment,

    #[required]
    pub width: Length,

    #[required]
    pub body: Content,

    #[default(Em::new(1.0).into())]
    pub clearance: Length,

    #[default(true)]
    pub first_page_only: bool,
}
```

### Acceptance Criteria

- [ ] Masthead element compiles and works
- [ ] first_page_only behavior implemented
- [ ] Documentation complete

---

## Phase 6: Performance Optimization

**Goal**: Optimize hot paths to achieve <200ms target for typical documents

**Branch**: `feature/performance-optimization`
**PR Title**: "Optimize text flow performance with caching and indexing"
**Estimated Complexity**: Medium (3-4 days)

### Performance Targets

| Document Type | Target | Acceptable |
|---------------|--------|------------|
| Simple wrap (1 image) | <50ms | <100ms |
| Newsletter (masthead) | <150ms | <200ms |
| Complex (4+ wraps) | <300ms | <400ms |
| No wraps (baseline) | No regression | ±5% |

### Optimization Areas

1. **Cutout Indexing**: Binary search instead of linear scan for y-position queries
2. **Relayout Convergence**: Detect when positions stabilize early
3. **Fast Paths**: Optimize empty-cutout case (zero overhead when no wraps)
4. **Width Query Caching**: Cache repeated queries at same y position
5. **Threshold-Based Queries**: Consider "at least X" queries for better comemo caching (from blog insight)

### Blog-Inspired Optimization

From Laurenz's blog:
> "Are there at least 4cm left?" rather than "Is there 4.2cm left?"

Consider implementing threshold-based width checks that allow more aggressive caching.

### Acceptance Criteria

- [ ] Target performance achieved
- [ ] No regression for non-wrap documents
- [ ] Benchmark suite runs successfully

---

## Phase 7: Documentation and Polish

**Goal**: Complete user documentation, examples, and prepare for upstream contribution

**Branch**: `feature/documentation`
**PR Title**: "Add documentation and examples for wrap and masthead elements"
**Estimated Complexity**: Low-Medium (2-3 days)

### Deliverables

- [ ] Reference documentation for wrap and masthead
- [ ] User guide with practical examples
- [ ] Example gallery (4+ working examples)
- [ ] Contributing documentation
- [ ] Updated README and CHANGELOG
- [ ] Visual integration tests in `tests/suite/layout/`

---

## Crate Organization

Understanding which code goes where:

| Crate | Purpose | Our Changes |
|-------|---------|-------------|
| `typst-library` | Foundational types (`Abs`, `Region`, etc.) | `cutout.rs`, `wrap.rs`, `masthead.rs` |
| `typst-layout` | Layout algorithms | `width_provider.rs`, flow integration |

**Why cutout types are in typst-library**: They are foundational data structures like `Region`, `Size`, `Abs`. The layout *algorithms* that use them go in `typst-layout`.

---

## Security Considerations

This implementation must address several security concerns to prevent malicious documents from causing DoS, memory exhaustion, or incorrect rendering.

### 1. Denial of Service - Relayout Infinite Loops

**Threat:** Malicious documents could create cutout configurations that never converge, causing infinite relayout loops.

**Mitigation:**
```rust
// In compose.rs
const MAX_RELAYOUT_ITERATIONS: usize = 5;

impl Composer {
    fn wrap(&mut self, ...) -> FlowResult<()> {
        // Track iteration count
        if self.relayout_count >= MAX_RELAYOUT_ITERATIONS {
            // Force convergence - stop relayout
            return Ok(());
        }
        
        // ... normal wrap logic ...
        
        // Increment counter before triggering relayout
        self.relayout_count += 1;
        Err(Stop::Relayout(PlacementScope::Column))
    }
}
```

**Testing:**
```rust
#[test]
fn test_relayout_iteration_limit() {
    // Create document that would relayout infinitely
    // Verify it stops at MAX_RELAYOUT_ITERATIONS
    // Should not hang or panic
}
```

### 2. Memory Exhaustion

**Threat:** Documents with thousands of wrap elements could exhaust memory.

**Mitigation:**
```rust
// Reasonable limits
const MAX_WRAPS_PER_COLUMN: usize = 100;
const MAX_CUTOUTS_PER_REGION: usize = 100;

impl Composer {
    fn wrap(&mut self, ...) -> FlowResult<()> {
        // Check limits before adding
        if self.wrap_frames.len() >= MAX_WRAPS_PER_COLUMN {
            return Err(Stop::Error("Too many wrap elements in column"));
        }
        
        if self.column_cutouts.len() >= MAX_CUTOUTS_PER_REGION {
            return Err(Stop::Error("Too many cutouts in region"));
        }
        
        // ... proceed with wrap ...
    }
}
```

**Testing:**
```rust
#[test]
fn test_wrap_count_limits() {
    // Create document with 150 wrap elements
    // Verify graceful error, not memory exhaustion
}
```

### 3. Integer Overflow & Numeric Edge Cases

**Threat:** Extreme coordinate values could cause overflow or negative widths.

**Mitigation:**
```rust
impl RegionCutout {
    pub fn new(
        y_start: Abs,
        y_end: Abs,
        side: CutoutSide,
        width: Abs,
        clearance: Abs,
    ) -> Self {
        // Validate ranges
        debug_assert!(y_start <= y_end, "Invalid cutout range");
        debug_assert!(width >= Abs::zero(), "Negative width");
        debug_assert!(clearance >= Abs::zero(), "Negative clearance");
        
        Self { y_start, y_end, side, width, clearance }
    }
    
    pub fn total_width(&self) -> Abs {
        // Use saturating add to prevent overflow
        self.width.saturating_add(self.clearance)
    }
}

impl Region {
    pub fn width_at(&self, y: Abs, cutouts: &[RegionCutout], dir: Dir) -> WidthInfo {
        let mut start_reduction = Abs::zero();
        let mut end_reduction = Abs::zero();
        
        for cutout in cutouts {
            if cutout.contains_y(y) {
                match cutout.side {
                    CutoutSide::Start => {
                        start_reduction = start_reduction.max(cutout.total_width());
                    }
                    CutoutSide::End => {
                        end_reduction = end_reduction.max(cutout.total_width());
                    }
                }
            }
        }
        
        // CRITICAL: Clamp to zero to prevent negative widths
        let available = (self.size.x - start_reduction - end_reduction)
            .max(Abs::zero());
        
        WidthInfo {
            available,
            start_offset: start_reduction,
            end_offset: end_reduction,
        }
    }
}
```

**Testing:**
```rust
#[test]
fn test_extreme_coordinate_values() {
    // Test with very large Abs values
    let cutout = RegionCutout::new(
        Abs::pt(1_000_000.0),
        Abs::pt(2_000_000.0),
        CutoutSide::Start,
        Abs::pt(500_000.0),
        Abs::pt(100_000.0),
    );
    // Should not panic or overflow
}

#[test]
fn test_overlapping_cutouts_no_negative_width() {
    let region = Region::new(Size::new(Abs::pt(100.0), Abs::pt(100.0)));
    
    // Create cutouts that would reduce width below zero
    let cutouts = vec![
        RegionCutout::new(Abs::zero(), Abs::pt(50.0), CutoutSide::Start, Abs::pt(80.0), Abs::zero()),
        RegionCutout::new(Abs::zero(), Abs::pt(50.0), CutoutSide::End, Abs::pt(80.0), Abs::zero()),
    ];
    
    let width_info = region.width_at(Abs::pt(25.0), &cutouts, Dir::LTR);
    
    // Width should be clamped to zero, not negative
    assert_eq!(width_info.available, Abs::zero());
}
```

### 4. Hash Implementation Correctness

**Threat:** Inconsistent hashing could poison the `comemo` cache, causing incorrect layout or memory leaks.

**Mitigation:**
```rust
impl Hash for RegionCutout {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // CRITICAL: Use to_raw() for consistent hashing of Abs values
        // Float equality is unreliable, but raw bits are deterministic
        self.y_start.to_raw().hash(state);
        self.y_end.to_raw().hash(state);
        self.side.hash(state);
        self.width.to_raw().hash(state);
        self.clearance.to_raw().hash(state);
    }
}

impl PartialEq for RegionCutout {
    fn eq(&self, other: &Self) -> bool {
        // Must be consistent with Hash
        self.y_start.to_raw() == other.y_start.to_raw()
            && self.y_end.to_raw() == other.y_end.to_raw()
            && self.side == other.side
            && self.width.to_raw() == other.width.to_raw()
            && self.clearance.to_raw() == other.clearance.to_raw()
    }
}

impl Eq for RegionCutout {}
```

**Testing:**
```rust
#[test]
fn test_hash_consistency() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let cutout1 = RegionCutout::new(
        Abs::pt(10.0), Abs::pt(50.0),
        CutoutSide::Start, Abs::pt(30.0), Abs::pt(5.0)
    );
    
    let cutout2 = RegionCutout::new(
        Abs::pt(10.0), Abs::pt(50.0),
        CutoutSide::Start, Abs::pt(30.0), Abs::pt(5.0)
    );
    
    let mut hasher1 = DefaultHasher::new();
    cutout1.hash(&mut hasher1);
    let hash1 = hasher1.finish();
    
    let mut hasher2 = DefaultHasher::new();
    cutout2.hash(&mut hasher2);
    let hash2 = hasher2.finish();
    
    assert_eq!(hash1, hash2, "Equal cutouts must hash to same value");
    assert_eq!(cutout1, cutout2, "Equal cutouts must be equal");
}
```

### 5. Untrusted Document Input Validation

**Threat:** Attackers could craft documents with extreme parameter values.

**Mitigation:**
```rust
// In wrap element realization/layout
const MAX_REASONABLE_CLEARANCE: Abs = Abs::pt(100.0);
const MAX_REASONABLE_WIDTH: Abs = Abs::pt(1000.0);
const MAX_REASONABLE_HEIGHT: Abs = Abs::pt(2000.0);

impl WrapChild {
    pub fn layout(&self, engine: &mut Engine, base: Size) -> SourceResult<Frame> {
        // Validate parameters before layout
        if self.clearance > MAX_REASONABLE_CLEARANCE {
            return Err("Wrap clearance too large".into());
        }
        
        let height = self.max_height.unwrap_or(base.y);
        if height > MAX_REASONABLE_HEIGHT {
            return Err("Wrap height too large".into());
        }
        
        // ... proceed with layout ...
    }
}
```

### 6. Performance-Based DoS

**Threat:** Complex cutout patterns with long paragraphs could cause exponential time in line breaking.

**Mitigation:**
```rust
// Typst already has paragraph layout timeouts
// We add additional safeguards:

const MAX_CUTOUTS_FOR_OPTIMIZATION: usize = 10;

fn linebreak_optimized(
    engine: &Engine,
    p: &Preparation,
    width_provider: &dyn WidthProvider,
) -> Vec<Line> {
    // If too many cutouts, fall back to simpler algorithm
    if is_complex_cutout_pattern(width_provider) {
        return linebreak_simple(engine, p, width_provider);
    }
    
    // ... normal Knuth-Plass optimization ...
}

fn is_complex_cutout_pattern(provider: &dyn WidthProvider) -> bool {
    // Check if cutout pattern is pathologically complex
    // e.g., many narrow cutouts, frequent width changes
    false // Implementation depends on actual complexity metrics
}
```

### 7. Cutout Width Validation

**Threat:** Cutouts that take up entire region width could make layout impossible.

**Mitigation:**
```rust
const MAX_CUTOUT_WIDTH_RATIO: f64 = 0.90; // 90% of region width

impl Composer {
    fn wrap(&mut self, wrap: &WrapChild, regions: &Regions, current_y: Abs) -> FlowResult<()> {
        let frame = wrap.layout(self.engine, regions.base())?;
        
        // Validate cutout doesn't take up too much width
        let max_allowed_width = regions.base().x * MAX_CUTOUT_WIDTH_RATIO;
        if frame.width() > max_allowed_width {
            return Err(Stop::Error("Wrap content too wide for region"));
        }
        
        // ... proceed ...
    }
}
```

### Security Testing Checklist

**Phase 1 (Region Cutouts):**
- [ ] Test with Abs::MAX and extreme values
- [ ] Verify width calculations never go negative
- [ ] Test overlapping cutouts
- [ ] Verify Hash/PartialEq consistency
- [ ] Test with zero-width regions

**Phase 2 (Line Breaking):**
- [ ] Test with pathological cutout patterns
- [ ] Verify performance with many cutouts
- [ ] Test with zero-width available space
- [ ] Ensure no panics on edge cases

**Phase 4 (Flow Integration):**
- [ ] Test relayout iteration limit enforcement
- [ ] Test with maximum wrap count
- [ ] Test recursive/nested wrap scenarios
- [ ] Memory profiling with many wraps
- [ ] Test convergence detection

**Phase 6 (Performance):**
- [ ] Benchmark worst-case documents
- [ ] DOS testing with malicious inputs
- [ ] Memory leak testing
- [ ] Timeout testing

### Inherited Typst Security Features

Good news - Typst already provides:
- **Rust memory safety:** No buffer overflows, use-after-free, null pointers
- **Document compilation timeouts:** Prevents infinite loops at document level
- **Resource limits:** Memory and processing limits enforced
- **Pure functional layout:** No mutable global state to corrupt
- **Sandboxed execution:** Script execution is isolated

### Security Review Process

1. **Self-review:** Check all validation and limits before PR
2. **Unit tests:** Comprehensive edge case testing
3. **Fuzzing consideration:** Suggest fuzzing infrastructure for upstream
4. **Documentation:** Document all limits and validation rules
5. **Maintainer review:** Highlight security-critical code sections

---

## Quick Reference

### Build Commands
```bash
cargo build                    # Build project
cargo test                     # Run tests
cargo fmt                      # Format code
cargo clippy                   # Lint code
cargo bench                    # Run benchmarks
```

### Phase 1 Verification
```bash
cargo build -p typst-library
cargo test -p typst-library cutout  # 28 tests
cargo test -p typst-library         # 60 tests total
```

### Performance Goal
**Primary Target**: <200ms for newsletter with masthead (vs Apache FOP 200ms, Meander 500ms, Wrap-it 1200ms)

---

## Contribution Guidelines Summary

For upstream contribution to Typst:

1. **Open GitHub issue first** - Discuss design before implementation
2. **Follow Typst patterns** - Pure functions, manual Hash impls for comemo
3. **Add visual integration tests** - In `tests/suite/` with reference images
4. **Documentation required** - Doc comments on all public APIs
5. **Minimize interface changes** - Prefer internal changes over API changes

See: https://github.com/typst/typst/blob/main/CONTRIBUTING.md
