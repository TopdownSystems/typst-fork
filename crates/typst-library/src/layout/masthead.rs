//! The masthead element for newsletter-style column layouts.

use crate::foundations::{Cast, Content, StyleChain, elem};
use crate::introspection::{Locatable, Tagged};
use crate::layout::{CutoutSide, Dir, Em, Length, OuterHAlignment, PlacementScope};

use super::wrap::outer_h_alignment_to_cutout_side;

/// How to handle masthead content that exceeds the available region height.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Cast)]
pub enum MastheadOverflow {
    /// Clip content that exceeds the available height (default).
    ///
    /// When masthead content is taller than the available region, it will be
    /// truncated to fit. A warning will be emitted when this occurs.
    #[default]
    Clip,
    /// Allow content to paginate across multiple regions.
    ///
    /// When masthead content is taller than the available region, the layout
    /// will attempt to continue on subsequent pages. Note: This requires
    /// sufficient flowing text content to trigger page breaks.
    Paginate,
}

/// A masthead sidebar for newsletter-style layouts.
///
/// This element creates a vertical sidebar region on one side of the page
/// while allowing text to flow around it. It's commonly used for newsletter
/// mastheads, magazine sidebars, or any layout where a column of content
/// should persist alongside flowing text.
///
/// The masthead is essentially a specialized [`wrap`] element with:
/// - An explicit width parameter (required)
/// - Default alignment to the start side
/// - Larger default clearance for newsletter aesthetics
///
/// # Example
/// ```example
/// #set page(width: 300pt, height: auto)
///
/// #masthead(80pt, clearance: 12pt)[
///   *Newsletter*
///
///   Issue 42
///
///   January 2026
/// ]
///
/// #lorem(60)
/// ```
///
/// # Side Selection
/// The side parameter works identically to [`wrap`]:
/// - `start` (default): The start side (left in LTR, right in RTL)
/// - `end`: The end side (right in LTR, left in RTL)
/// - `left`: Always the left side
/// - `right`: Always the right side
///
/// ```example
/// #set page(width: 300pt, height: auto)
///
/// #masthead(right, 70pt)[
///   *Sidebar*
///
///   Extra info
/// ]
///
/// Main content flows on the left side of the page, wrapping around
/// the masthead column on the right.
///
/// #lorem(40)
/// ```
///
/// # Full-Height Mastheads
/// To create a masthead that extends the full height of the content,
/// you can use a large height value or the `1fr` sizing:
///
/// ```example
/// #set page(width: 300pt, height: 200pt)
///
/// #masthead(60pt)[
///   #block(height: 1fr, fill: aqua.lighten(80%))[
///     *Table of Contents*
///
///     1. Introduction
///     2. Methods
///     3. Results
///   ]
/// ]
///
/// #lorem(30)
/// ```
#[elem(Locatable, Tagged)]
pub struct MastheadElem {
    /// Which side to place the masthead on.
    ///
    /// Can be one of:
    /// - `start` (default): The start side (left in LTR, right in RTL)
    /// - `end`: The end side (right in LTR, left in RTL)
    /// - `left`: Always the left side
    /// - `right`: Always the right side
    ///
    /// ```example
    /// #set page(width: 280pt, height: auto)
    ///
    /// #masthead(left, 60pt)[Left masthead]
    /// Text flows around the left masthead.
    ///
    /// #v(2em)
    ///
    /// #masthead(right, 60pt)[Right masthead]
    /// Text flows around the right masthead.
    /// ```
    #[positional]
    #[default(OuterHAlignment::Start)]
    pub side: OuterHAlignment,

    /// The width of the masthead column.
    ///
    /// This sets the horizontal space reserved for the masthead content.
    /// The body content will be laid out within this width.
    ///
    /// ```example
    /// #set page(width: 300pt, height: auto)
    ///
    /// #masthead(100pt)[
    ///   A wider masthead provides more space for sidebar content.
    /// ]
    ///
    /// #lorem(30)
    /// ```
    #[positional]
    #[required]
    pub width: Length,

    /// The content to display in the masthead.
    ///
    /// This is typically text, images, or a combination. The content
    /// is laid out within the specified width.
    #[required]
    pub body: Content,

    /// The spacing between the masthead and flowing text.
    ///
    /// This creates a buffer zone between the masthead column and the
    /// main text flow. Larger clearance values provide more visual
    /// separation.
    ///
    /// ```example
    /// #set page(width: 300pt, height: auto)
    ///
    /// #masthead(70pt, clearance: 20pt)[
    ///   *Wide gap*
    /// ]
    ///
    /// Notice the larger gap between the masthead and this text.
    ///
    /// #lorem(20)
    /// ```
    #[default(Em::new(1.0).into())]
    pub clearance: Length,

    /// Relative to which containing scope the masthead wraps.
    ///
    /// - `"column"` (default): Masthead only affects the current column
    /// - `"parent"`: Masthead spans across all columns
    ///
    /// ```example
    /// #set page(width: 400pt, height: auto, columns: 2)
    ///
    /// #masthead(80pt, scope: "parent")[
    ///   *Spanning Masthead*
    ///
    ///   This sidebar affects both columns.
    /// ]
    ///
    /// #lorem(80)
    /// ```
    pub scope: PlacementScope,

    /// How to handle content that exceeds the available region height.
    ///
    /// - `"clip"` (default): Truncate content that doesn't fit. A warning is
    ///   emitted when content is clipped.
    /// - `"paginate"`: Attempt to continue on subsequent pages. Requires
    ///   sufficient flowing text content to trigger page breaks.
    ///
    /// ```example
    /// #set page(width: 200pt, height: 150pt)
    ///
    /// #masthead(60pt, overflow: "clip")[
    ///   *Long Content*
    ///
    ///   This sidebar has more content than fits in the region. It will be
    ///   clipped to the available height.
    ///
    ///   More content here...
    ///
    ///   And even more...
    /// ]
    ///
    /// Short text.
    /// ```
    pub overflow: MastheadOverflow,
}

impl MastheadElem {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_masthead_cutout_side_ltr() {
        let dir_ltr = Dir::LTR;

        // Default is Start, which maps to Start in LTR
        assert_eq!(
            outer_h_alignment_to_cutout_side(OuterHAlignment::Start, dir_ltr),
            CutoutSide::Start
        );

        // End maps to End
        assert_eq!(
            outer_h_alignment_to_cutout_side(OuterHAlignment::End, dir_ltr),
            CutoutSide::End
        );
    }

    #[test]
    fn test_masthead_cutout_side_rtl() {
        let dir_rtl = Dir::RTL;

        // Start maps to Start in RTL
        assert_eq!(
            outer_h_alignment_to_cutout_side(OuterHAlignment::Start, dir_rtl),
            CutoutSide::Start
        );

        // Left maps to End in RTL (left is the end side)
        assert_eq!(
            outer_h_alignment_to_cutout_side(OuterHAlignment::Left, dir_rtl),
            CutoutSide::End
        );
    }
}
