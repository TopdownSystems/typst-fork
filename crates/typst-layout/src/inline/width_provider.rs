//! Width provider abstraction for variable-width line breaking.
//!
//! This module provides a trait and implementations for querying available
//! line widths during paragraph layout. This enables text to flow around
//! cutouts (images, floats) by providing different widths at different
//! vertical positions within a paragraph.

use typst_library::layout::{Abs, Dir, RegionCutout, WidthInfo};

/// Provides width information for line breaking at different vertical positions.
///
/// This trait abstracts how available line width is determined, allowing the
/// Knuth-Plass algorithm to work with both fixed widths (current behavior) and
/// variable widths (for cutouts/text flow).
///
/// # Purity
///
/// Implementations must be pure (deterministic, no side effects) to work with
/// Typst's parallelized layout engine and comemo memoization.
pub trait WidthProvider {
    /// Get available width information at the given cumulative height.
    ///
    /// The `cumulative_height` is measured from the start of the paragraph,
    /// representing the total height of all lines laid out so far.
    fn width_at(&self, cumulative_height: Abs) -> WidthInfo;

    /// Get the base (maximum possible) width for optimization hints.
    ///
    /// This is used for quick checks and as a fallback when no cutouts
    /// are active. For fixed-width providers, this equals the constant width.
    fn base_width(&self) -> Abs;

    /// Check if the width is constant (no cutouts affecting this paragraph).
    ///
    /// This enables fast-path optimizations when width doesn't vary.
    fn is_constant(&self) -> bool {
        true
    }
}

/// Fixed-width provider representing current (non-cutout) behavior.
///
/// This is a zero-cost abstraction that always returns the same width,
/// maintaining backward compatibility with existing paragraph layout.
#[derive(Debug, Clone, Copy)]
pub struct FixedWidth {
    /// The constant available width.
    width: Abs,
}

impl FixedWidth {
    /// Creates a new fixed-width provider.
    pub fn new(width: Abs) -> Self {
        Self { width }
    }
}

impl WidthProvider for FixedWidth {
    #[inline]
    fn width_at(&self, _cumulative_height: Abs) -> WidthInfo {
        WidthInfo::full(self.width)
    }

    #[inline]
    fn base_width(&self) -> Abs {
        self.width
    }

    #[inline]
    fn is_constant(&self) -> bool {
        true
    }
}

/// Variable-width provider using region cutouts.
///
/// This provider queries cutout information to determine available width
/// at each vertical position within a paragraph.
///
/// Note: This type will be used in Phase 4 (Flow Layout Integration) when
/// wrap elements are integrated with paragraph layout.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CutoutWidth<'a> {
    /// The full width of the region (without cutouts).
    region_width: Abs,
    /// The cutouts that may affect this paragraph.
    cutouts: &'a [RegionCutout],
    /// The paragraph's starting y position in the region.
    ///
    /// Used to translate cumulative paragraph heights to region-relative
    /// y coordinates for cutout queries.
    y_offset: Abs,
    /// Text direction for interpreting start/end sides.
    dir: Dir,
}

#[allow(dead_code)]
impl<'a> CutoutWidth<'a> {
    /// Creates a new cutout-based width provider.
    ///
    /// # Arguments
    ///
    /// * `region_width` - The full width of the region
    /// * `cutouts` - Slice of cutouts that may affect width
    /// * `y_offset` - The paragraph's starting y position in the region
    /// * `dir` - Text direction for side interpretation
    pub fn new(
        region_width: Abs,
        cutouts: &'a [RegionCutout],
        y_offset: Abs,
        dir: Dir,
    ) -> Self {
        Self { region_width, cutouts, y_offset, dir }
    }

    /// Returns true if this provider has no cutouts (constant width).
    pub fn is_empty(&self) -> bool {
        self.cutouts.is_empty()
    }
}

impl WidthProvider for CutoutWidth<'_> {
    fn width_at(&self, cumulative_height: Abs) -> WidthInfo {
        // Fast path: no cutouts means full width
        if self.cutouts.is_empty() {
            return WidthInfo::full(self.region_width);
        }

        // Translate paragraph-relative height to region-relative y coordinate
        let region_y = self.y_offset + cumulative_height;

        // Query cutouts at this y position
        typst_library::layout::width_at(
            self.region_width,
            region_y,
            self.cutouts,
            self.dir,
        )
    }

    #[inline]
    fn base_width(&self) -> Abs {
        self.region_width
    }

    #[inline]
    fn is_constant(&self) -> bool {
        self.cutouts.is_empty()
    }
}

/// Helper to create a width provider from a fixed width (for backward compatibility).
///
/// This allows existing code to continue using `Abs` values where a `WidthProvider`
/// is now expected.
impl From<Abs> for FixedWidth {
    fn from(width: Abs) -> Self {
        Self::new(width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typst_library::layout::CutoutSide;

    fn pt(val: f64) -> Abs {
        Abs::pt(val)
    }

    mod fixed_width_tests {
        use super::*;

        #[test]
        fn test_fixed_width_returns_constant() {
            let provider = FixedWidth::new(pt(500.0));

            // Width should be constant at any height
            let info_at_0 = provider.width_at(pt(0.0));
            let info_at_100 = provider.width_at(pt(100.0));
            let info_at_1000 = provider.width_at(pt(1000.0));

            assert_eq!(info_at_0.available, pt(500.0));
            assert_eq!(info_at_100.available, pt(500.0));
            assert_eq!(info_at_1000.available, pt(500.0));
        }

        #[test]
        fn test_fixed_width_no_offsets() {
            let provider = FixedWidth::new(pt(300.0));
            let info = provider.width_at(pt(50.0));

            assert_eq!(info.start_offset, pt(0.0));
            assert_eq!(info.end_offset, pt(0.0));
        }

        #[test]
        fn test_fixed_width_is_constant() {
            let provider = FixedWidth::new(pt(200.0));
            assert!(provider.is_constant());
        }

        #[test]
        fn test_fixed_width_base_width() {
            let provider = FixedWidth::new(pt(400.0));
            assert_eq!(provider.base_width(), pt(400.0));
        }

        #[test]
        fn test_fixed_width_from_abs() {
            let provider: FixedWidth = pt(250.0).into();
            assert_eq!(provider.base_width(), pt(250.0));
        }
    }

    mod cutout_width_tests {
        use super::*;

        #[test]
        fn test_cutout_width_empty_cutouts() {
            let provider = CutoutWidth::new(pt(500.0), &[], pt(0.0), Dir::LTR);

            let info = provider.width_at(pt(50.0));
            assert_eq!(info.available, pt(500.0));
            assert!(provider.is_constant());
            assert!(provider.is_empty());
        }

        #[test]
        fn test_cutout_width_with_cutout() {
            // Cutout from y=20 to y=80 on the end side
            let cutout = RegionCutout::new(
                pt(20.0),
                pt(80.0),
                CutoutSide::End,
                pt(100.0),
                pt(10.0),
            );
            let cutouts = [cutout];
            let provider = CutoutWidth::new(pt(500.0), &cutouts, pt(0.0), Dir::LTR);

            // Before cutout
            let info_before = provider.width_at(pt(10.0));
            assert_eq!(info_before.available, pt(500.0));

            // Inside cutout
            let info_inside = provider.width_at(pt(50.0));
            assert_eq!(info_inside.available, pt(390.0)); // 500 - 100 - 10
            assert_eq!(info_inside.end_offset, pt(110.0)); // 100 + 10

            // After cutout
            let info_after = provider.width_at(pt(100.0));
            assert_eq!(info_after.available, pt(500.0));
        }

        #[test]
        fn test_cutout_width_with_y_offset() {
            // Cutout from y=50 to y=100 in region coordinates
            let cutout = RegionCutout::new(
                pt(50.0),
                pt(100.0),
                CutoutSide::Start,
                pt(80.0),
                pt(20.0),
            );
            let cutouts = [cutout];

            // Paragraph starts at y=30 in the region
            let provider = CutoutWidth::new(pt(500.0), &cutouts, pt(30.0), Dir::LTR);

            // At paragraph height 0 (region y=30), no cutout
            let info_0 = provider.width_at(pt(0.0));
            assert_eq!(info_0.available, pt(500.0));

            // At paragraph height 25 (region y=55), inside cutout
            let info_25 = provider.width_at(pt(25.0));
            assert_eq!(info_25.available, pt(400.0)); // 500 - 80 - 20
            assert_eq!(info_25.start_offset, pt(100.0)); // 80 + 20

            // At paragraph height 80 (region y=110), after cutout
            let info_80 = provider.width_at(pt(80.0));
            assert_eq!(info_80.available, pt(500.0));
        }

        #[test]
        fn test_cutout_width_is_not_constant() {
            let cutout = RegionCutout::new(
                pt(0.0),
                pt(50.0),
                CutoutSide::End,
                pt(100.0),
                pt(10.0),
            );
            let cutouts = [cutout];
            let provider = CutoutWidth::new(pt(500.0), &cutouts, pt(0.0), Dir::LTR);

            assert!(!provider.is_constant());
            assert!(!provider.is_empty());
        }

        #[test]
        fn test_cutout_width_base_width() {
            let cutout = RegionCutout::new(
                pt(0.0),
                pt(50.0),
                CutoutSide::End,
                pt(100.0),
                pt(10.0),
            );
            let cutouts = [cutout];
            let provider = CutoutWidth::new(pt(500.0), &cutouts, pt(0.0), Dir::LTR);

            // Base width is always the full region width
            assert_eq!(provider.base_width(), pt(500.0));
        }

        #[test]
        fn test_cutout_width_rtl() {
            // Cutout on start side (left in LTR, right in RTL)
            let cutout = RegionCutout::new(
                pt(0.0),
                pt(100.0),
                CutoutSide::Start,
                pt(80.0),
                pt(20.0),
            );
            let cutouts = [cutout];
            let provider = CutoutWidth::new(pt(500.0), &cutouts, pt(0.0), Dir::RTL);

            let info = provider.width_at(pt(50.0));
            assert_eq!(info.available, pt(400.0)); // 500 - 80 - 20
            // In RTL, start_offset should be swapped (was end_reduction, now 0)
            // and end_offset should be the start_reduction
            assert_eq!(info.start_offset, pt(0.0));
            assert_eq!(info.end_offset, pt(100.0)); // 80 + 20
        }

        #[test]
        fn test_cutout_width_multiple_cutouts() {
            let cutout1 = RegionCutout::new(
                pt(0.0),
                pt(100.0),
                CutoutSide::Start,
                pt(50.0),
                pt(10.0),
            );
            let cutout2 =
                RegionCutout::new(pt(0.0), pt(100.0), CutoutSide::End, pt(80.0), pt(5.0));
            let cutouts = [cutout1, cutout2];
            let provider = CutoutWidth::new(pt(500.0), &cutouts, pt(0.0), Dir::LTR);

            let info = provider.width_at(pt(50.0));
            // Both cutouts active: 500 - (50+10) - (80+5) = 355
            assert_eq!(info.available, pt(355.0));
            assert_eq!(info.start_offset, pt(60.0)); // 50 + 10
            assert_eq!(info.end_offset, pt(85.0)); // 80 + 5
        }
    }
}
