//! Region cutout types for variable-width layout.
//!
//! This module provides types for representing rectangular exclusion zones
//! within layout regions. These cutouts enable text to flow around images
//! and other placed content by reducing available width at certain vertical
//! positions.

use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};

use crate::layout::{Abs, Dir};

/// Which side of the region a cutout occupies.
///
/// The side is expressed in logical terms (start/end) which are resolved
/// based on the text direction.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum CutoutSide {
    /// The start side (left in LTR, right in RTL).
    Start,
    /// The end side (right in LTR, left in RTL).
    End,
}

impl CutoutSide {
    /// Returns the opposite side.
    pub const fn opposite(self) -> Self {
        match self {
            CutoutSide::Start => CutoutSide::End,
            CutoutSide::End => CutoutSide::Start,
        }
    }

    /// Converts to physical left/right based on text direction.
    /// Returns true if this side corresponds to the left in the given direction.
    pub const fn is_left(self, dir: Dir) -> bool {
        match (self, dir) {
            (CutoutSide::Start, Dir::LTR) | (CutoutSide::End, Dir::RTL) => true,
            (CutoutSide::End, Dir::LTR) | (CutoutSide::Start, Dir::RTL) => false,
            // Vertical directions: treat start as left
            (CutoutSide::Start, Dir::TTB | Dir::BTT) => true,
            (CutoutSide::End, Dir::TTB | Dir::BTT) => false,
        }
    }
}

/// A rectangular exclusion zone in a region.
///
/// Cutouts represent areas where content should not be placed, typically
/// occupied by images or other floating elements. Text flows around these
/// cutouts by reducing the available line width.
#[derive(Copy, Clone)]
pub struct RegionCutout {
    /// Top of the cutout (region-relative y coordinate).
    pub y_start: Abs,
    /// Bottom of the cutout (region-relative y coordinate).
    pub y_end: Abs,
    /// Which side of the region the cutout occupies.
    pub side: CutoutSide,
    /// Width of the cutout itself.
    pub width: Abs,
    /// Additional spacing between the cutout and flowing text.
    pub clearance: Abs,
}

impl RegionCutout {
    /// Creates a new region cutout.
    ///
    /// # Panics (debug builds only)
    ///
    /// Panics if:
    /// - `y_start > y_end` (invalid range)
    /// - `width < 0` (negative width)
    /// - `clearance < 0` (negative clearance)
    pub fn new(
        y_start: Abs,
        y_end: Abs,
        side: CutoutSide,
        width: Abs,
        clearance: Abs,
    ) -> Self {
        debug_assert!(
            y_start <= y_end,
            "RegionCutout: y_start ({y_start:?}) must be <= y_end ({y_end:?})"
        );
        debug_assert!(
            width >= Abs::zero(),
            "RegionCutout: width ({width:?}) must be non-negative"
        );
        debug_assert!(
            clearance >= Abs::zero(),
            "RegionCutout: clearance ({clearance:?}) must be non-negative"
        );
        Self { y_start, y_end, side, width, clearance }
    }

    /// Returns the total width this cutout reduces from available space.
    ///
    /// This includes both the cutout width and the clearance.
    pub fn total_width(self) -> Abs {
        self.width + self.clearance
    }

    /// Checks if a y position is within this cutout's vertical range.
    pub fn contains_y(self, y: Abs) -> bool {
        y >= self.y_start && y < self.y_end
    }

    /// Checks if this cutout overlaps with a vertical range.
    ///
    /// Returns true if any part of the cutout intersects with [y_start, y_end).
    pub fn overlaps_range(self, y_start: Abs, y_end: Abs) -> bool {
        // Two ranges [a, b) and [c, d) overlap if a < d && c < b
        self.y_start < y_end && y_start < self.y_end
    }

    /// Returns the height of this cutout.
    pub fn height(self) -> Abs {
        self.y_end - self.y_start
    }
}

impl Debug for RegionCutout {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("RegionCutout")
            .field("y_start", &self.y_start)
            .field("y_end", &self.y_end)
            .field("side", &self.side)
            .field("width", &self.width)
            .field("clearance", &self.clearance)
            .finish()
    }
}

impl PartialEq for RegionCutout {
    fn eq(&self, other: &Self) -> bool {
        self.y_start == other.y_start
            && self.y_end == other.y_end
            && self.side == other.side
            && self.width == other.width
            && self.clearance == other.clearance
    }
}

impl Eq for RegionCutout {}

// Manual Hash implementation using to_raw() for deterministic hashing.
// This is required for comemo compatibility.
impl Hash for RegionCutout {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.y_start.to_raw().to_bits().hash(state);
        self.y_end.to_raw().to_bits().hash(state);
        self.side.hash(state);
        self.width.to_raw().to_bits().hash(state);
        self.clearance.to_raw().to_bits().hash(state);
    }
}

/// Information about available width at a vertical position.
///
/// When text flows around cutouts, lines may have reduced width and/or
/// need to be offset from the start of the region.
#[derive(Copy, Clone)]
pub struct WidthInfo {
    /// Width available for content at this position.
    pub available: Abs,
    /// Offset from the start edge of the region.
    ///
    /// In LTR text, this is the left offset. In RTL, this represents how
    /// far from the right edge content should start.
    pub start_offset: Abs,
    /// Offset from the end edge of the region.
    ///
    /// This is the space reserved at the end of lines.
    pub end_offset: Abs,
}

impl WidthInfo {
    /// Creates a WidthInfo representing full available width with no offsets.
    pub fn full(width: Abs) -> Self {
        Self {
            available: width,
            start_offset: Abs::zero(),
            end_offset: Abs::zero(),
        }
    }

    /// Creates a new WidthInfo with the specified values.
    pub fn new(available: Abs, start_offset: Abs, end_offset: Abs) -> Self {
        Self { available, start_offset, end_offset }
    }

    /// Checks if a given width fits within the available space.
    pub fn fits(self, width: Abs) -> bool {
        self.available.fits(width)
    }

    /// Returns true if this represents full width with no cutouts.
    pub fn is_full(self, region_width: Abs) -> bool {
        self.start_offset.approx_eq(Abs::zero())
            && self.end_offset.approx_eq(Abs::zero())
            && self.available.approx_eq(region_width)
    }
}

impl Debug for WidthInfo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("WidthInfo")
            .field("available", &self.available)
            .field("start_offset", &self.start_offset)
            .field("end_offset", &self.end_offset)
            .finish()
    }
}

impl PartialEq for WidthInfo {
    fn eq(&self, other: &Self) -> bool {
        self.available == other.available
            && self.start_offset == other.start_offset
            && self.end_offset == other.end_offset
    }
}

impl Eq for WidthInfo {}

// Manual Hash implementation using to_raw() for deterministic hashing.
impl Hash for WidthInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.available.to_raw().to_bits().hash(state);
        self.start_offset.to_raw().to_bits().hash(state);
        self.end_offset.to_raw().to_bits().hash(state);
    }
}

/// Computes width information at a y position given a set of cutouts.
///
/// This is the core function for determining available line width when
/// laying out text that flows around cutouts.
pub fn width_at(
    region_width: Abs,
    y: Abs,
    cutouts: &[RegionCutout],
    dir: Dir,
) -> WidthInfo {
    // Fast path: no cutouts
    if cutouts.is_empty() {
        return WidthInfo::full(region_width);
    }

    let mut start_reduction = Abs::zero();
    let mut end_reduction = Abs::zero();

    for cutout in cutouts {
        if cutout.contains_y(y) {
            let reduction = cutout.total_width();
            match cutout.side {
                CutoutSide::Start => {
                    start_reduction.set_max(reduction);
                }
                CutoutSide::End => {
                    end_reduction.set_max(reduction);
                }
            }
        }
    }

    // Calculate available width, ensuring it doesn't go negative
    let available = (region_width - start_reduction - end_reduction).max(Abs::zero());

    // Swap offsets for RTL direction
    let (start_offset, end_offset) = match dir {
        Dir::LTR | Dir::TTB | Dir::BTT => (start_reduction, end_reduction),
        Dir::RTL => (end_reduction, start_reduction),
    };

    WidthInfo::new(available, start_offset, end_offset)
}

/// Computes the minimum width information across a vertical range.
///
/// This returns the most restrictive width info (smallest available width)
/// within the specified range, which is needed when laying out content
/// that spans multiple lines.
pub fn width_in_range(
    region_width: Abs,
    y_start: Abs,
    y_end: Abs,
    cutouts: &[RegionCutout],
    dir: Dir,
) -> WidthInfo {
    // Fast path: no cutouts
    if cutouts.is_empty() {
        return WidthInfo::full(region_width);
    }

    let mut start_reduction = Abs::zero();
    let mut end_reduction = Abs::zero();

    for cutout in cutouts {
        if cutout.overlaps_range(y_start, y_end) {
            let reduction = cutout.total_width();
            match cutout.side {
                CutoutSide::Start => {
                    start_reduction.set_max(reduction);
                }
                CutoutSide::End => {
                    end_reduction.set_max(reduction);
                }
            }
        }
    }

    // Calculate available width, ensuring it doesn't go negative
    let available = (region_width - start_reduction - end_reduction).max(Abs::zero());

    // Swap offsets for RTL direction
    let (start_offset, end_offset) = match dir {
        Dir::LTR | Dir::TTB | Dir::BTT => (start_reduction, end_reduction),
        Dir::RTL => (end_reduction, start_reduction),
    };

    WidthInfo::new(available, start_offset, end_offset)
}

/// Returns an iterator over cutouts that affect a given y position.
///
/// This returns an iterator rather than a collected Vec to avoid
/// unnecessary allocations when the caller only needs to iterate once.
pub fn cutouts_at_y(
    cutouts: &[RegionCutout],
    y: Abs,
) -> impl Iterator<Item = &RegionCutout> {
    cutouts.iter().filter(move |c| c.contains_y(y))
}

/// Returns an iterator over cutouts that affect a vertical range.
///
/// This returns an iterator rather than a collected Vec to avoid
/// unnecessary allocations when the caller only needs to iterate once.
pub fn cutouts_in_range(
    cutouts: &[RegionCutout],
    y_start: Abs,
    y_end: Abs,
) -> impl Iterator<Item = &RegionCutout> {
    cutouts.iter().filter(move |c| c.overlaps_range(y_start, y_end))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create absolute lengths
    fn pt(val: f64) -> Abs {
        Abs::pt(val)
    }

    mod cutout_side_tests {
        use super::*;

        #[test]
        fn test_opposite() {
            assert_eq!(CutoutSide::Start.opposite(), CutoutSide::End);
            assert_eq!(CutoutSide::End.opposite(), CutoutSide::Start);
        }

        #[test]
        fn test_is_left_ltr() {
            assert!(CutoutSide::Start.is_left(Dir::LTR));
            assert!(!CutoutSide::End.is_left(Dir::LTR));
        }

        #[test]
        fn test_is_left_rtl() {
            assert!(!CutoutSide::Start.is_left(Dir::RTL));
            assert!(CutoutSide::End.is_left(Dir::RTL));
        }
    }

    mod cutout_tests {
        use super::*;

        #[test]
        fn test_cutout_new() {
            let cutout = RegionCutout::new(
                pt(10.0),
                pt(100.0),
                CutoutSide::End,
                pt(50.0),
                pt(5.0),
            );
            assert_eq!(cutout.y_start, pt(10.0));
            assert_eq!(cutout.y_end, pt(100.0));
            assert_eq!(cutout.side, CutoutSide::End);
            assert_eq!(cutout.width, pt(50.0));
            assert_eq!(cutout.clearance, pt(5.0));
        }

        #[test]
        fn test_total_width() {
            let cutout = RegionCutout::new(
                pt(0.0),
                pt(100.0),
                CutoutSide::End,
                pt(50.0),
                pt(10.0),
            );
            assert_eq!(cutout.total_width(), pt(60.0));
        }

        #[test]
        fn test_contains_y() {
            let cutout = RegionCutout::new(
                pt(20.0),
                pt(80.0),
                CutoutSide::Start,
                pt(50.0),
                pt(5.0),
            );

            // Before cutout
            assert!(!cutout.contains_y(pt(10.0)));
            // At start (inclusive)
            assert!(cutout.contains_y(pt(20.0)));
            // Inside cutout
            assert!(cutout.contains_y(pt(50.0)));
            // At end (exclusive)
            assert!(!cutout.contains_y(pt(80.0)));
            // After cutout
            assert!(!cutout.contains_y(pt(100.0)));
        }

        #[test]
        fn test_overlaps_range_no_overlap() {
            let cutout = RegionCutout::new(
                pt(50.0),
                pt(100.0),
                CutoutSide::End,
                pt(30.0),
                pt(5.0),
            );

            // Range entirely before
            assert!(!cutout.overlaps_range(pt(0.0), pt(50.0)));
            // Range entirely after
            assert!(!cutout.overlaps_range(pt(100.0), pt(150.0)));
        }

        #[test]
        fn test_overlaps_range_overlap() {
            let cutout = RegionCutout::new(
                pt(50.0),
                pt(100.0),
                CutoutSide::End,
                pt(30.0),
                pt(5.0),
            );

            // Range overlaps start
            assert!(cutout.overlaps_range(pt(40.0), pt(60.0)));
            // Range overlaps end
            assert!(cutout.overlaps_range(pt(90.0), pt(110.0)));
            // Range inside cutout
            assert!(cutout.overlaps_range(pt(60.0), pt(80.0)));
            // Range contains cutout
            assert!(cutout.overlaps_range(pt(0.0), pt(200.0)));
            // Range equals cutout
            assert!(cutout.overlaps_range(pt(50.0), pt(100.0)));
        }

        #[test]
        fn test_height() {
            let cutout = RegionCutout::new(
                pt(25.0),
                pt(75.0),
                CutoutSide::Start,
                pt(40.0),
                pt(10.0),
            );
            assert_eq!(cutout.height(), pt(50.0));
        }

        #[test]
        fn test_hash_determinism() {
            use std::collections::hash_map::DefaultHasher;

            let cutout1 = RegionCutout::new(
                pt(10.0),
                pt(100.0),
                CutoutSide::End,
                pt(50.0),
                pt(5.0),
            );
            let cutout2 = RegionCutout::new(
                pt(10.0),
                pt(100.0),
                CutoutSide::End,
                pt(50.0),
                pt(5.0),
            );

            let mut hasher1 = DefaultHasher::new();
            let mut hasher2 = DefaultHasher::new();
            cutout1.hash(&mut hasher1);
            cutout2.hash(&mut hasher2);

            assert_eq!(hasher1.finish(), hasher2.finish());
        }
    }

    mod width_info_tests {
        use super::*;

        #[test]
        fn test_full() {
            let info = WidthInfo::full(pt(500.0));
            assert_eq!(info.available, pt(500.0));
            assert_eq!(info.start_offset, pt(0.0));
            assert_eq!(info.end_offset, pt(0.0));
        }

        #[test]
        fn test_new() {
            let info = WidthInfo::new(pt(400.0), pt(50.0), pt(50.0));
            assert_eq!(info.available, pt(400.0));
            assert_eq!(info.start_offset, pt(50.0));
            assert_eq!(info.end_offset, pt(50.0));
        }

        #[test]
        fn test_fits() {
            let info = WidthInfo::new(pt(100.0), pt(0.0), pt(0.0));
            assert!(info.fits(pt(50.0)));
            assert!(info.fits(pt(100.0)));
            assert!(!info.fits(pt(150.0)));
        }

        #[test]
        fn test_is_full() {
            let full = WidthInfo::full(pt(500.0));
            assert!(full.is_full(pt(500.0)));

            let with_offset = WidthInfo::new(pt(400.0), pt(50.0), pt(50.0));
            assert!(!with_offset.is_full(pt(500.0)));
        }
    }

    mod width_at_tests {
        use super::*;

        #[test]
        fn test_width_no_cutouts() {
            let info = width_at(pt(500.0), pt(50.0), &[], Dir::LTR);
            assert_eq!(info.available, pt(500.0));
            assert_eq!(info.start_offset, pt(0.0));
            assert_eq!(info.end_offset, pt(0.0));
        }

        #[test]
        fn test_width_with_end_cutout() {
            let cutout = RegionCutout::new(
                pt(0.0),
                pt(100.0),
                CutoutSide::End,
                pt(100.0),
                pt(10.0),
            );
            let cutouts = [cutout];

            let info = width_at(pt(500.0), pt(50.0), &cutouts, Dir::LTR);
            assert_eq!(info.available, pt(390.0)); // 500 - 100 - 10
            assert_eq!(info.start_offset, pt(0.0));
            assert_eq!(info.end_offset, pt(110.0)); // 100 + 10
        }

        #[test]
        fn test_width_with_start_cutout() {
            let cutout = RegionCutout::new(
                pt(0.0),
                pt(100.0),
                CutoutSide::Start,
                pt(80.0),
                pt(20.0),
            );
            let cutouts = [cutout];

            let info = width_at(pt(500.0), pt(50.0), &cutouts, Dir::LTR);
            assert_eq!(info.available, pt(400.0)); // 500 - 80 - 20
            assert_eq!(info.start_offset, pt(100.0)); // 80 + 20
            assert_eq!(info.end_offset, pt(0.0));
        }

        #[test]
        fn test_width_with_both_sides() {
            let start_cutout = RegionCutout::new(
                pt(0.0),
                pt(100.0),
                CutoutSide::Start,
                pt(50.0),
                pt(10.0),
            );
            let end_cutout =
                RegionCutout::new(pt(0.0), pt(100.0), CutoutSide::End, pt(80.0), pt(5.0));
            let cutouts = [start_cutout, end_cutout];

            let info = width_at(pt(500.0), pt(50.0), &cutouts, Dir::LTR);
            assert_eq!(info.available, pt(355.0)); // 500 - 60 - 85
            assert_eq!(info.start_offset, pt(60.0)); // 50 + 10
            assert_eq!(info.end_offset, pt(85.0)); // 80 + 5
        }

        #[test]
        fn test_multiple_cutouts_same_side() {
            // Two cutouts on same side - should use maximum
            let cutout1 =
                RegionCutout::new(pt(0.0), pt(100.0), CutoutSide::End, pt(50.0), pt(5.0));
            let cutout2 = RegionCutout::new(
                pt(0.0),
                pt(100.0),
                CutoutSide::End,
                pt(80.0),
                pt(10.0),
            );
            let cutouts = [cutout1, cutout2];

            let info = width_at(pt(500.0), pt(50.0), &cutouts, Dir::LTR);
            // Should use max of (50+5=55) and (80+10=90) = 90
            assert_eq!(info.available, pt(410.0)); // 500 - 90
            assert_eq!(info.end_offset, pt(90.0));
        }

        #[test]
        fn test_width_rtl_direction() {
            let cutout = RegionCutout::new(
                pt(0.0),
                pt(100.0),
                CutoutSide::Start,
                pt(80.0),
                pt(20.0),
            );
            let cutouts = [cutout];

            // In RTL, Start means right side, so offsets are swapped
            let info = width_at(pt(500.0), pt(50.0), &cutouts, Dir::RTL);
            assert_eq!(info.available, pt(400.0)); // 500 - 80 - 20
            // In RTL, the start_offset should be end_reduction (swapped)
            assert_eq!(info.start_offset, pt(0.0));
            assert_eq!(info.end_offset, pt(100.0)); // 80 + 20
        }

        #[test]
        fn test_width_outside_cutout_range() {
            let cutout = RegionCutout::new(
                pt(50.0),
                pt(100.0),
                CutoutSide::End,
                pt(100.0),
                pt(10.0),
            );
            let cutouts = [cutout];

            // Y position before cutout
            let info_before = width_at(pt(500.0), pt(25.0), &cutouts, Dir::LTR);
            assert_eq!(info_before.available, pt(500.0));

            // Y position after cutout
            let info_after = width_at(pt(500.0), pt(150.0), &cutouts, Dir::LTR);
            assert_eq!(info_after.available, pt(500.0));
        }

        #[test]
        fn test_width_never_negative() {
            // Cutouts wider than the region
            let cutout1 = RegionCutout::new(
                pt(0.0),
                pt(100.0),
                CutoutSide::Start,
                pt(300.0),
                pt(10.0),
            );
            let cutout2 = RegionCutout::new(
                pt(0.0),
                pt(100.0),
                CutoutSide::End,
                pt(300.0),
                pt(10.0),
            );
            let cutouts = [cutout1, cutout2];

            let info = width_at(pt(500.0), pt(50.0), &cutouts, Dir::LTR);
            assert_eq!(info.available, pt(0.0)); // Should be 0, not negative
        }
    }

    mod width_in_range_tests {
        use super::*;

        #[test]
        fn test_width_in_range_no_cutouts() {
            let info = width_in_range(pt(500.0), pt(0.0), pt(100.0), &[], Dir::LTR);
            assert_eq!(info.available, pt(500.0));
        }

        #[test]
        fn test_width_in_range_partial_overlap() {
            // Cutout from 50-150, query range 0-100
            let cutout = RegionCutout::new(
                pt(50.0),
                pt(150.0),
                CutoutSide::End,
                pt(100.0),
                pt(10.0),
            );
            let cutouts = [cutout];

            let info = width_in_range(pt(500.0), pt(0.0), pt(100.0), &cutouts, Dir::LTR);
            // Cutout overlaps with range, so width is reduced
            assert_eq!(info.available, pt(390.0)); // 500 - 100 - 10
        }

        #[test]
        fn test_width_in_range_no_overlap() {
            // Cutout from 100-200, query range 0-50
            let cutout = RegionCutout::new(
                pt(100.0),
                pt(200.0),
                CutoutSide::End,
                pt(100.0),
                pt(10.0),
            );
            let cutouts = [cutout];

            let info = width_in_range(pt(500.0), pt(0.0), pt(50.0), &cutouts, Dir::LTR);
            // No overlap, full width available
            assert_eq!(info.available, pt(500.0));
        }

        #[test]
        fn test_width_in_range_multiple_cutouts() {
            // Multiple cutouts at different heights, both overlapping the range
            let cutout1 =
                RegionCutout::new(pt(0.0), pt(50.0), CutoutSide::End, pt(80.0), pt(10.0));
            let cutout2 = RegionCutout::new(
                pt(30.0),
                pt(100.0),
                CutoutSide::End,
                pt(60.0),
                pt(5.0),
            );
            let cutouts = [cutout1, cutout2];

            // Query range overlaps both
            let info = width_in_range(pt(500.0), pt(0.0), pt(60.0), &cutouts, Dir::LTR);
            // Should use maximum reduction: max(90, 65) = 90
            assert_eq!(info.available, pt(410.0)); // 500 - 90
        }
    }

    mod helper_tests {
        use super::*;

        #[test]
        fn test_cutouts_at_y() {
            let cutout1 = RegionCutout::new(
                pt(0.0),
                pt(50.0),
                CutoutSide::Start,
                pt(40.0),
                pt(5.0),
            );
            let cutout2 = RegionCutout::new(
                pt(30.0),
                pt(100.0),
                CutoutSide::End,
                pt(60.0),
                pt(10.0),
            );
            let cutouts = [cutout1, cutout2];

            // At y=25, only cutout1 is active
            let active: Vec<_> = cutouts_at_y(&cutouts, pt(25.0)).collect();
            assert_eq!(active.len(), 1);
            assert_eq!(active[0].side, CutoutSide::Start);

            // At y=40, both cutouts are active
            let active: Vec<_> = cutouts_at_y(&cutouts, pt(40.0)).collect();
            assert_eq!(active.len(), 2);

            // At y=75, only cutout2 is active
            let active: Vec<_> = cutouts_at_y(&cutouts, pt(75.0)).collect();
            assert_eq!(active.len(), 1);
            assert_eq!(active[0].side, CutoutSide::End);
        }

        #[test]
        fn test_cutouts_in_range() {
            let cutout1 = RegionCutout::new(
                pt(0.0),
                pt(50.0),
                CutoutSide::Start,
                pt(40.0),
                pt(5.0),
            );
            let cutout2 = RegionCutout::new(
                pt(100.0),
                pt(150.0),
                CutoutSide::End,
                pt(60.0),
                pt(10.0),
            );
            let cutouts = [cutout1, cutout2];

            // Range overlapping only cutout1
            let active: Vec<_> = cutouts_in_range(&cutouts, pt(20.0), pt(60.0)).collect();
            assert_eq!(active.len(), 1);

            // Range overlapping both
            let active: Vec<_> = cutouts_in_range(&cutouts, pt(0.0), pt(200.0)).collect();
            assert_eq!(active.len(), 2);

            // Range between cutouts
            let active: Vec<_> = cutouts_in_range(&cutouts, pt(60.0), pt(90.0)).collect();
            assert_eq!(active.len(), 0);
        }
    }
}
