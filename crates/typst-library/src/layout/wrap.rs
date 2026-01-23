//! The wrap element for text flow layout.

use crate::foundations::{Content, StyleChain, elem};
use crate::introspection::{Locatable, Tagged};
use crate::layout::{CutoutSide, Dir, Em, Length, OuterHAlignment, PlacementScope};

/// Places content to the side with text flowing around it.
///
/// This element positions content (like images or sidebars) to one side of
/// the page or column while allowing text to flow around it. This is commonly
/// used for magazine-style layouts, figure placement, or pull quotes.
///
/// The wrap element participates in flow layout, creating a cutout region
/// that text avoids. Unlike [`place`] with floating, wrap content does not
/// displace other content but rather reduces the available width for text
/// at the wrapped content's vertical position.
///
/// # Example
/// ```example
/// #set page(width: 200pt, height: auto)
///
/// #wrap(
///   right,
///   rect(width: 60pt, height: 80pt, fill: aqua),
/// )
///
/// #lorem(50)
/// ```
///
/// # Side Selection
/// The side parameter determines where the wrapped content appears:
/// - `start` / `end`: Logical sides based on text direction
/// - `left` / `right`: Physical sides regardless of text direction
///
/// For left-to-right text, `start` equals `left` and `end` equals `right`.
/// For right-to-left text, these are reversed.
///
/// ```example
/// #set page(width: 200pt, height: auto)
/// #set text(dir: rtl)
///
/// // In RTL, 'start' means right side
/// #wrap(start, rect(fill: blue, width: 40pt, height: 40pt))
///
/// هذا نص عربي يلتف حول الصورة.
/// ```
///
/// # Clearance
/// The clearance parameter controls the space between the wrapped content
/// and the flowing text:
///
/// ```example
/// #set page(width: 200pt, height: auto)
///
/// #wrap(
///   right,
///   clearance: 12pt,
///   rect(width: 50pt, height: 50pt, fill: green),
/// )
///
/// #lorem(30)
/// ```
///
/// # Scope
/// By default, wrap content only affects the current column. Use
/// `scope: "parent"` to make the wrap span across all columns:
///
/// ```example
/// #set page(width: 300pt, height: auto, columns: 2)
///
/// #wrap(
///   left,
///   scope: "parent",
///   rect(width: 80pt, height: 100pt, fill: red),
/// )
///
/// #lorem(80)
/// ```
#[elem(Locatable, Tagged)]
pub struct WrapElem {
    /// Which side to place the wrapped content on.
    ///
    /// Can be one of:
    /// - `start`: The start side (left in LTR, right in RTL)
    /// - `end`: The end side (right in LTR, left in RTL)
    /// - `left`: Always the left side
    /// - `right`: Always the right side
    ///
    /// ```example
    /// #set page(width: 180pt, height: auto)
    ///
    /// #wrap(left, rect(fill: red, width: 40pt, height: 40pt))
    /// Left-wrapped content appears here.
    ///
    /// #v(1em)
    ///
    /// #wrap(right, rect(fill: blue, width: 40pt, height: 40pt))
    /// Right-wrapped content appears here.
    /// ```
    #[positional]
    #[default(OuterHAlignment::End)]
    pub side: OuterHAlignment,

    /// The content to wrap text around.
    ///
    /// This can be any content, but is typically an image, rectangle,
    /// or other fixed-size element.
    #[required]
    pub body: Content,

    /// The spacing between the wrapped content and flowing text.
    ///
    /// This creates a buffer zone around the wrapped content that text
    /// will not enter. Larger clearance values provide more visual
    /// separation.
    ///
    /// ```example
    /// #set page(width: 200pt, height: auto)
    ///
    /// #wrap(
    ///   right,
    ///   clearance: 20pt,
    ///   rect(fill: orange, width: 50pt, height: 50pt),
    /// )
    ///
    /// #lorem(25)
    /// ```
    #[default(Em::new(0.5).into())]
    pub clearance: Length,

    /// Relative to which containing scope the content wraps.
    ///
    /// - `"column"` (default): Wrap only affects the current column
    /// - `"parent"`: Wrap spans across all columns
    ///
    /// ```example
    /// #set page(width: 300pt, height: auto, columns: 2)
    ///
    /// #wrap(
    ///   left,
    ///   scope: "parent",
    ///   rect(fill: purple, width: 80pt, height: 60pt),
    /// )
    ///
    /// #lorem(60)
    /// ```
    pub scope: PlacementScope,
}

impl WrapElem {
    /// Converts the side alignment to a logical cutout side based on text direction.
    ///
    /// This method resolves the `OuterHAlignment` to a `CutoutSide` taking into
    /// account whether the alignment is logical (start/end) or physical (left/right)
    /// and the text direction.
    pub fn cutout_side(&self, styles: StyleChain, dir: Dir) -> CutoutSide {
        let side = self.side.get(styles);
        outer_h_alignment_to_cutout_side(side, dir)
    }
}

/// Converts an OuterHAlignment to a CutoutSide based on text direction.
///
/// - `Start` and `End` map directly to their logical equivalents
/// - `Left` and `Right` are physical and depend on text direction:
///   - In LTR: Left -> Start, Right -> End
///   - In RTL: Left -> End, Right -> Start
pub fn outer_h_alignment_to_cutout_side(side: OuterHAlignment, dir: Dir) -> CutoutSide {
    match side {
        OuterHAlignment::Start => CutoutSide::Start,
        OuterHAlignment::End => CutoutSide::End,
        OuterHAlignment::Left => {
            if dir.is_positive() {
                CutoutSide::Start
            } else {
                CutoutSide::End
            }
        }
        OuterHAlignment::Right => {
            if dir.is_positive() {
                CutoutSide::End
            } else {
                CutoutSide::Start
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cutout_side_ltr() {
        let dir_ltr = Dir::LTR;

        // Start -> Start in any direction
        assert_eq!(
            outer_h_alignment_to_cutout_side(OuterHAlignment::Start, dir_ltr),
            CutoutSide::Start
        );

        // End -> End in any direction
        assert_eq!(
            outer_h_alignment_to_cutout_side(OuterHAlignment::End, dir_ltr),
            CutoutSide::End
        );

        // Left -> Start in LTR
        assert_eq!(
            outer_h_alignment_to_cutout_side(OuterHAlignment::Left, dir_ltr),
            CutoutSide::Start
        );

        // Right -> End in LTR
        assert_eq!(
            outer_h_alignment_to_cutout_side(OuterHAlignment::Right, dir_ltr),
            CutoutSide::End
        );
    }

    #[test]
    fn test_cutout_side_rtl() {
        let dir_rtl = Dir::RTL;

        // Start -> Start in any direction
        assert_eq!(
            outer_h_alignment_to_cutout_side(OuterHAlignment::Start, dir_rtl),
            CutoutSide::Start
        );

        // End -> End in any direction
        assert_eq!(
            outer_h_alignment_to_cutout_side(OuterHAlignment::End, dir_rtl),
            CutoutSide::End
        );

        // Left -> End in RTL (left is end side in RTL)
        assert_eq!(
            outer_h_alignment_to_cutout_side(OuterHAlignment::Left, dir_rtl),
            CutoutSide::End
        );

        // Right -> Start in RTL (right is start side in RTL)
        assert_eq!(
            outer_h_alignment_to_cutout_side(OuterHAlignment::Right, dir_rtl),
            CutoutSide::Start
        );
    }
}
