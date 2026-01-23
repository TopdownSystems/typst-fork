//! Collection of flow elements into prepared children.
//!
//! This module pre-processes flow elements (paragraphs, blocks, wraps, etc.)
//! into a uniform representation that is easier to lay out. When wrap or masthead
//! elements are present, paragraphs use deferred layout to support variable-width
//! line breaking around cutouts.

use std::cell::{LazyCell, RefCell};
use std::fmt::{self, Debug, Formatter};
use std::hash::Hash;

use bumpalo::Bump;
use bumpalo::boxed::Box as BumpBox;
use comemo::{Track, Tracked, TrackedMut};
use typst_library::World;
use typst_library::diag::{SourceResult, bail, warning};
use typst_library::engine::{Engine, Route, Sink, Traced};
use typst_library::foundations::{Packed, Resolve, Smart, StyleChain};
use typst_library::introspection::{
    Introspector, Location, Locator, LocatorLink, SplitLocator, Tag, TagElem,
};
use typst_library::layout::{
    Abs, AlignElem, Alignment, Axes, BlockElem, ColbreakElem, CutoutSide, FixedAlignment,
    FlushElem, Fr, Fragment, Frame, FrameParent, Inherit, MastheadElem, PagebreakElem,
    PlaceElem, PlacementScope, Ratio, Region, Regions, Rel, Size, Sizing, Spacing, VElem,
    WrapElem,
};
use typst_library::model::ParElem;
use typst_library::routines::{Pair, Routines};
use typst_library::text::TextElem as TextElemModel;
use typst_library::text::TextElem;
use typst_utils::{Protected, SliceExt};

use super::{FlowMode, layout_multi_block, layout_single_block};
use crate::inline::ParSituation;
use crate::modifiers::layout_and_modify;

/// Collects all elements of the flow into prepared children. These are much
/// simpler to handle than the raw elements.
#[typst_macros::time]
#[allow(clippy::too_many_arguments)]
pub fn collect<'a>(
    engine: &mut Engine,
    bump: &'a Bump,
    children: &[Pair<'a>],
    locator: Locator<'a>,
    base: Size,
    expand: bool,
    mode: FlowMode,
) -> SourceResult<Vec<Child<'a>>> {
    // Pre-scan to check if any wrap or masthead elements exist.
    // If so, we use deferred paragraph layout so text can flow around cutouts.
    let has_wraps = children
        .iter()
        .any(|(child, _)| child.is::<WrapElem>() || child.is::<MastheadElem>());

    Collector {
        engine,
        bump,
        children,
        locator: locator.split(),
        base,
        expand,
        output: Vec::with_capacity(children.len()),
        par_situation: ParSituation::First,
        use_deferred_par: has_wraps,
    }
    .run(mode)
}

/// State for collection.
struct Collector<'a, 'x, 'y> {
    engine: &'x mut Engine<'y>,
    bump: &'a Bump,
    children: &'x [Pair<'a>],
    base: Size,
    expand: bool,
    locator: SplitLocator<'a>,
    output: Vec<Child<'a>>,
    par_situation: ParSituation,
    /// Whether to use deferred paragraph layout (when wraps are present).
    use_deferred_par: bool,
}

impl<'a> Collector<'a, '_, '_> {
    /// Perform the collection.
    fn run(self, mode: FlowMode) -> SourceResult<Vec<Child<'a>>> {
        match mode {
            FlowMode::Root | FlowMode::Block => self.run_block(),
            FlowMode::Inline => self.run_inline(),
        }
    }

    /// Perform collection for block-level children.
    fn run_block(mut self) -> SourceResult<Vec<Child<'a>>> {
        for &(child, styles) in self.children {
            if let Some(elem) = child.to_packed::<TagElem>() {
                self.output.push(Child::Tag(&elem.tag));
            } else if let Some(elem) = child.to_packed::<VElem>() {
                self.v(elem, styles);
            } else if let Some(elem) = child.to_packed::<ParElem>() {
                self.par(elem, styles)?;
            } else if let Some(elem) = child.to_packed::<BlockElem>() {
                self.block(elem, styles);
            } else if let Some(elem) = child.to_packed::<PlaceElem>() {
                self.place(elem, styles)?;
            } else if let Some(elem) = child.to_packed::<WrapElem>() {
                self.wrap(elem, styles);
            } else if let Some(elem) = child.to_packed::<MastheadElem>() {
                self.masthead(elem, styles);
            } else if child.is::<FlushElem>() {
                self.output.push(Child::Flush);
            } else if let Some(elem) = child.to_packed::<ColbreakElem>() {
                self.output.push(Child::Break(elem.weak.get(styles)));
            } else if child.is::<PagebreakElem>() {
                bail!(
                    child.span(), "pagebreaks are not allowed inside of containers";
                    hint: "try using a `#colbreak()` instead";
                );
            } else {
                self.engine.sink.warn(warning!(
                    child.span(),
                    "{} was ignored during paged export",
                    child.func().name(),
                ));
            }
        }

        Ok(self.output)
    }

    /// Perform collection for inline-level children.
    fn run_inline(mut self) -> SourceResult<Vec<Child<'a>>> {
        // Extract leading and trailing tags.
        let (start, end) = self.children.split_prefix_suffix(|(c, _)| c.is::<TagElem>());
        let inner = &self.children[start..end];

        // Compute the shared styles.
        let styles = StyleChain::trunk_from_pairs(inner).unwrap_or_default();

        // Layout the lines.
        let lines = crate::inline::layout_inline(
            self.engine,
            inner,
            &mut self.locator,
            styles,
            self.base,
            self.expand,
        )?
        .into_frames();

        for (c, _) in &self.children[..start] {
            let elem = c.to_packed::<TagElem>().unwrap();
            self.output.push(Child::Tag(&elem.tag));
        }

        let leading = styles.resolve(ParElem::leading);
        self.lines(lines, leading, styles);

        for (c, _) in &self.children[end..] {
            let elem = c.to_packed::<TagElem>().unwrap();
            self.output.push(Child::Tag(&elem.tag));
        }

        Ok(self.output)
    }

    /// Collect vertical spacing into a relative or fractional child.
    fn v(&mut self, elem: &'a Packed<VElem>, styles: StyleChain<'a>) {
        self.output.push(match elem.amount {
            Spacing::Rel(rel) => {
                Child::Rel(rel.resolve(styles), elem.weak.get(styles) as u8)
            }
            Spacing::Fr(fr) => Child::Fr(fr, elem.weak.get(styles) as u8),
        });
    }

    /// Collect a paragraph into [`LineChild`]ren or a [`ParChild`] for deferred layout.
    ///
    /// When wraps are present (`use_deferred_par` is true), we defer paragraph layout
    /// to distribution time so text can flow around cutouts. Otherwise, we lay out
    /// the paragraph immediately since it's not dependent on the concrete regions.
    fn par(
        &mut self,
        elem: &'a Packed<ParElem>,
        styles: StyleChain<'a>,
    ) -> SourceResult<()> {
        let spacing = elem.spacing.resolve(styles);
        let leading = elem.leading.resolve(styles);

        if self.use_deferred_par {
            // Defer paragraph layout to distribution time when cutouts are known.
            let align = styles.resolve(AlignElem::alignment);
            let costs = styles.get(TextElem::costs);

            let par_child = ParChild {
                elem,
                styles,
                locator: self.locator.next(&elem.span()),
                situation: self.par_situation,
                base: self.base,
                expand: self.expand,
                spacing: spacing.into(),
                leading,
                align,
                costs,
            };

            self.output.push(Child::Par(self.boxed(par_child)));
        } else {
            // Lay out immediately (original behavior for content without wraps).
            let lines = crate::inline::layout_par(
                elem,
                self.engine,
                self.locator.next(&elem.span()),
                styles,
                self.base,
                self.expand,
                self.par_situation,
            )?
            .into_frames();

            self.output.push(Child::Rel(spacing.into(), 4));
            self.lines(lines, leading, styles);
            self.output.push(Child::Rel(spacing.into(), 4));
        }

        self.par_situation = ParSituation::Consecutive;
        Ok(())
    }

    /// Collect laid-out lines.
    fn lines(&mut self, lines: Vec<Frame>, leading: Abs, styles: StyleChain<'a>) {
        let align = styles.resolve(AlignElem::alignment);
        let costs = styles.get(TextElem::costs);

        // Determine whether to prevent widow and orphans.
        let len = lines.len();
        let prevent_orphans =
            costs.orphan() > Ratio::zero() && len >= 2 && !lines[1].is_empty();
        let prevent_widows =
            costs.widow() > Ratio::zero() && len >= 2 && !lines[len - 2].is_empty();
        let prevent_all = len == 3 && prevent_orphans && prevent_widows;

        // Store the heights of lines at the edges because we'll potentially
        // need these later when `lines` is already moved.
        let height_at = |i| lines.get(i).map(Frame::height).unwrap_or_default();
        let front_1 = height_at(0);
        let front_2 = height_at(1);
        let back_2 = height_at(len.saturating_sub(2));
        let back_1 = height_at(len.saturating_sub(1));

        for (i, frame) in lines.into_iter().enumerate() {
            if i > 0 {
                self.output.push(Child::Rel(leading.into(), 5));
            }

            // To prevent widows and orphans, we require enough space for
            // - all lines if it's just three
            // - the first two lines if we're at the first line
            // - the last two lines if we're at the second to last line
            let need = if prevent_all && i == 0 {
                front_1 + leading + front_2 + leading + back_1
            } else if prevent_orphans && i == 0 {
                front_1 + leading + front_2
            } else if prevent_widows && i >= 2 && i + 2 == len {
                back_2 + leading + back_1
            } else {
                frame.height()
            };

            self.output
                .push(Child::Line(self.boxed(LineChild { frame, align, need })));
        }
    }

    /// Collect a block into a [`SingleChild`] or [`MultiChild`] depending on
    /// whether it is breakable.
    fn block(&mut self, elem: &'a Packed<BlockElem>, styles: StyleChain<'a>) {
        let locator = self.locator.next(&elem.span());
        let align = styles.resolve(AlignElem::alignment);
        let alone = self.children.len() == 1;
        let sticky = elem.sticky.get(styles);
        let breakable = elem.breakable.get(styles);
        let fr = match elem.height.get(styles) {
            Sizing::Fr(fr) => Some(fr),
            _ => None,
        };

        let fallback = LazyCell::new(|| styles.resolve(ParElem::spacing));
        let spacing = |amount| match amount {
            Smart::Auto => Child::Rel((*fallback).into(), 4),
            Smart::Custom(Spacing::Rel(rel)) => Child::Rel(rel.resolve(styles), 3),
            Smart::Custom(Spacing::Fr(fr)) => Child::Fr(fr, 2),
        };

        self.output.push(spacing(elem.above.get(styles)));

        if !breakable || fr.is_some() {
            self.output.push(Child::Single(self.boxed(SingleChild {
                align,
                sticky,
                alone,
                fr,
                elem,
                styles,
                locator,
                cell: CachedCell::new(),
            })));
        } else {
            self.output.push(Child::Multi(self.boxed(MultiChild {
                align,
                sticky,
                alone,
                elem,
                styles,
                locator,
                cell: CachedCell::new(),
            })));
        };

        self.output.push(spacing(elem.below.get(styles)));
        self.par_situation = ParSituation::Other;
    }

    /// Collects a placed element into a [`PlacedChild`].
    fn place(
        &mut self,
        elem: &'a Packed<PlaceElem>,
        styles: StyleChain<'a>,
    ) -> SourceResult<()> {
        let alignment = elem.alignment.get(styles);
        let align_x = alignment.map_or(FixedAlignment::Center, |align| {
            align.x().unwrap_or_default().resolve(styles)
        });
        let align_y = alignment.map(|align| align.y().map(|y| y.resolve(styles)));
        let scope = elem.scope.get(styles);
        let float = elem.float.get(styles);

        match (float, align_y) {
            (true, Smart::Custom(None | Some(FixedAlignment::Center))) => bail!(
                elem.span(),
                "vertical floating placement must be `auto`, `top`, or `bottom`"
            ),
            (false, Smart::Auto) => bail!(
                elem.span(),
                "automatic positioning is only available for floating placement";
                hint: "you can enable floating placement with `place(float: true, ..)`";
            ),
            _ => {}
        }

        if !float && scope == PlacementScope::Parent {
            bail!(
                elem.span(),
                "parent-scoped positioning is currently only available for floating placement";
                hint: "you can enable floating placement with `place(float: true, ..)`";
            );
        }

        let locator = self.locator.next(&elem.span());
        let clearance = elem.clearance.resolve(styles);
        let delta = Axes::new(elem.dx.get(styles), elem.dy.get(styles)).resolve(styles);
        self.output.push(Child::Placed(self.boxed(PlacedChild {
            align_x,
            align_y,
            scope,
            float,
            clearance,
            delta,
            elem,
            styles,
            locator,
            alignment,
            cell: CachedCell::new(),
        })));

        Ok(())
    }

    /// Collects a wrap element into a [`WrapChild`].
    fn wrap(&mut self, elem: &'a Packed<WrapElem>, styles: StyleChain<'a>) {
        let locator = self.locator.next(&elem.span());
        let clearance = elem.clearance.resolve(styles);
        let scope = elem.scope.get(styles);

        // Get text direction to resolve logical sides to physical sides.
        let dir = styles.resolve(TextElemModel::dir);
        let side = elem.cutout_side(styles, dir);

        self.output.push(Child::Wrap(self.boxed(WrapChild {
            side,
            scope,
            clearance,
            elem,
            styles,
            locator,
            cell: CachedCell::new(),
        })));

        self.par_situation = ParSituation::Other;
    }

    /// Collects a masthead element into a [`MastheadChild`].
    fn masthead(&mut self, elem: &'a Packed<MastheadElem>, styles: StyleChain<'a>) {
        let locator = self.locator.next(&elem.span());
        let clearance = elem.clearance.resolve(styles);
        let scope = elem.scope.get(styles);
        let width = elem.width.resolve(styles);

        // Get text direction to resolve logical sides to physical sides.
        let dir = styles.resolve(TextElemModel::dir);
        let side = elem.cutout_side(styles, dir);

        self.output.push(Child::Masthead(self.boxed(MastheadChild {
            side,
            scope,
            clearance,
            width,
            elem,
            styles,
            locator,
            cell: CachedCell::new(),
        })));

        self.par_situation = ParSituation::Other;
    }

    /// Wraps a value in a bump-allocated box to reduce its footprint in the
    /// [`Child`] enum.
    fn boxed<T>(&self, value: T) -> BumpBox<'a, T> {
        BumpBox::new_in(value, self.bump)
    }
}

/// A prepared child in flow layout.
///
/// The larger variants are bump-boxed to keep the enum size down.
#[derive(Debug)]
pub enum Child<'a> {
    /// An introspection tag.
    Tag(&'a Tag),
    /// Relative spacing with a specific weakness level.
    Rel(Rel<Abs>, u8),
    /// Fractional spacing with a specific weakness level.
    Fr(Fr, u8),
    /// An already layouted line of a paragraph.
    Line(BumpBox<'a, LineChild>),
    /// A paragraph that needs layout with cutout awareness.
    ///
    /// Unlike `Line`, this variant stores the paragraph element for deferred
    /// layout during distribution, when the actual cutout positions are known.
    /// This is used when wrap elements affect the paragraph's layout.
    Par(BumpBox<'a, ParChild<'a>>),
    /// An unbreakable block.
    Single(BumpBox<'a, SingleChild<'a>>),
    /// A breakable block.
    Multi(BumpBox<'a, MultiChild<'a>>),
    /// An absolutely or floatingly placed element.
    Placed(BumpBox<'a, PlacedChild<'a>>),
    /// A wrap element that text flows around.
    Wrap(BumpBox<'a, WrapChild<'a>>),
    /// A masthead element that creates a fixed-width column for text to flow around.
    Masthead(BumpBox<'a, MastheadChild<'a>>),
    /// A place flush.
    Flush,
    /// An explicit column break.
    Break(bool),
}

/// A child that encapsulates a layouted line of a paragraph.
#[derive(Debug)]
pub struct LineChild {
    pub frame: Frame,
    pub align: Axes<FixedAlignment>,
    pub need: Abs,
}

/// A child that encapsulates a paragraph for deferred layout.
///
/// This is used when wrap elements may affect the paragraph's layout.
/// The paragraph is laid out during distribution when the cutout
/// positions are known, rather than during collection.
#[derive(Debug)]
pub struct ParChild<'a> {
    /// The paragraph element to layout.
    pub elem: &'a Packed<ParElem>,
    /// The styles to use for layout.
    pub styles: StyleChain<'a>,
    /// The locator for introspection.
    pub locator: Locator<'a>,
    /// The paragraph situation (first, consecutive, other).
    pub situation: crate::inline::ParSituation,
    /// The base size for layout.
    pub base: Size,
    /// Whether to expand horizontally.
    pub expand: bool,
    /// Spacing before and after the paragraph.
    pub spacing: Rel<Abs>,
    /// Leading between lines.
    pub leading: Abs,
    /// Text alignment.
    pub align: Axes<FixedAlignment>,
    /// Costs for widow/orphan prevention.
    pub costs: typst_library::text::Costs,
}

impl<'a> ParChild<'a> {
    /// Layout the paragraph with optional cutout information.
    ///
    /// If cutouts are provided and affect this paragraph's position,
    /// the lines will be broken with variable widths to flow around them.
    pub fn layout(
        &self,
        engine: &mut Engine,
        cutouts: &[typst_library::layout::RegionCutout],
        y_offset: Abs,
    ) -> SourceResult<Vec<Frame>> {
        use crate::inline::{InlineContext, layout_par_with_context};

        let context = if cutouts.is_empty() {
            None
        } else {
            Some(InlineContext::new(cutouts, y_offset))
        };

        layout_par_with_context(
            self.elem,
            engine,
            self.locator.track(),
            self.styles,
            self.base,
            self.expand,
            self.situation,
            context.as_ref(),
        )
        .map(|fragment| fragment.into_frames())
    }
}

/// A child that encapsulates a prepared unbreakable block.
#[derive(Debug)]
pub struct SingleChild<'a> {
    pub align: Axes<FixedAlignment>,
    pub sticky: bool,
    pub alone: bool,
    pub fr: Option<Fr>,
    elem: &'a Packed<BlockElem>,
    styles: StyleChain<'a>,
    locator: Locator<'a>,
    cell: CachedCell<SourceResult<Frame>>,
}

impl SingleChild<'_> {
    /// Build the child's frame given the region's base size.
    pub fn layout(&self, engine: &mut Engine, region: Region) -> SourceResult<Frame> {
        self.cell.get_or_init(region, |mut region| {
            // Vertical expansion is only kept if this block is the only child.
            region.expand.y &= self.alone;
            layout_single_impl(
                engine.routines,
                engine.world,
                engine.introspector.into_raw(),
                engine.traced,
                TrackedMut::reborrow_mut(&mut engine.sink),
                engine.route.track(),
                self.elem,
                self.locator.track(),
                self.styles,
                region,
            )
        })
    }
}

/// The cached, internal implementation of [`SingleChild::layout`].
#[comemo::memoize]
#[allow(clippy::too_many_arguments)]
fn layout_single_impl(
    routines: &Routines,
    world: Tracked<dyn World + '_>,
    introspector: Tracked<Introspector>,
    traced: Tracked<Traced>,
    sink: TrackedMut<Sink>,
    route: Tracked<Route>,
    elem: &Packed<BlockElem>,
    locator: Tracked<Locator>,
    styles: StyleChain,
    region: Region,
) -> SourceResult<Frame> {
    let introspector = Protected::from_raw(introspector);
    let link = LocatorLink::new(locator);
    let locator = Locator::link(&link);
    let mut engine = Engine {
        routines,
        world,
        introspector,
        traced,
        sink,
        route: Route::extend(route),
    };

    layout_and_modify(styles, |styles| {
        layout_single_block(elem, &mut engine, locator, styles, region)
    })
}

/// A child that encapsulates a prepared breakable block.
#[derive(Debug)]
pub struct MultiChild<'a> {
    pub align: Axes<FixedAlignment>,
    pub sticky: bool,
    alone: bool,
    elem: &'a Packed<BlockElem>,
    styles: StyleChain<'a>,
    locator: Locator<'a>,
    cell: CachedCell<SourceResult<Fragment>>,
}

impl<'a> MultiChild<'a> {
    /// Build the child's frames given regions.
    pub fn layout<'b>(
        &'b self,
        engine: &mut Engine,
        regions: Regions,
    ) -> SourceResult<(Frame, Option<MultiSpill<'a, 'b>>)> {
        let fragment = self.layout_full(engine, regions)?;
        let exist_non_empty_frame = fragment.iter().any(|f| !f.is_empty());

        // Extract the first frame.
        let mut frames = fragment.into_iter();
        let frame = frames.next().unwrap();

        // If there's more, return a `spill`.
        let mut spill = None;
        if frames.next().is_some() {
            spill = Some(MultiSpill {
                exist_non_empty_frame,
                multi: self,
                full: regions.full,
                first: regions.size.y,
                backlog: vec![],
                min_backlog_len: regions.backlog.len(),
            });
        }

        Ok((frame, spill))
    }

    /// The shared internal implementation of [`Self::layout`] and
    /// [`MultiSpill::layout`].
    fn layout_full(
        &self,
        engine: &mut Engine,
        regions: Regions,
    ) -> SourceResult<Fragment> {
        self.cell.get_or_init(regions, |mut regions| {
            // Vertical expansion is only kept if this block is the only child.
            regions.expand.y &= self.alone;
            layout_multi_impl(
                engine.routines,
                engine.world,
                engine.introspector.into_raw(),
                engine.traced,
                TrackedMut::reborrow_mut(&mut engine.sink),
                engine.route.track(),
                self.elem,
                self.locator.track(),
                self.styles,
                regions,
            )
        })
    }
}

/// The cached, internal implementation of [`MultiChild::layout_full`].
#[comemo::memoize]
#[allow(clippy::too_many_arguments)]
fn layout_multi_impl(
    routines: &Routines,
    world: Tracked<dyn World + '_>,
    introspector: Tracked<Introspector>,
    traced: Tracked<Traced>,
    sink: TrackedMut<Sink>,
    route: Tracked<Route>,
    elem: &Packed<BlockElem>,
    locator: Tracked<Locator>,
    styles: StyleChain,
    regions: Regions,
) -> SourceResult<Fragment> {
    let introspector = Protected::from_raw(introspector);
    let link = LocatorLink::new(locator);
    let locator = Locator::link(&link);
    let mut engine = Engine {
        routines,
        world,
        introspector,
        traced,
        sink,
        route: Route::extend(route),
    };

    layout_and_modify(styles, |styles| {
        layout_multi_block(elem, &mut engine, locator, styles, regions)
    })
}

/// The spilled remains of a `MultiChild` that broke across two regions.
#[derive(Debug, Clone)]
pub struct MultiSpill<'a, 'b> {
    pub(super) exist_non_empty_frame: bool,
    multi: &'b MultiChild<'a>,
    first: Abs,
    full: Abs,
    backlog: Vec<Abs>,
    min_backlog_len: usize,
}

impl MultiSpill<'_, '_> {
    /// Build the spill's frames given regions.
    pub fn layout(
        mut self,
        engine: &mut Engine,
        regions: Regions,
    ) -> SourceResult<(Frame, Option<Self>)> {
        // The first region becomes unchangeable and committed to our backlog.
        self.backlog.push(regions.size.y);

        // The remaining regions are ephemeral and may be replaced.
        let mut backlog: Vec<_> =
            self.backlog.iter().chain(regions.backlog).copied().collect();

        // Remove unnecessary backlog items to prevent it from growing
        // unnecessarily, changing the region's hash.
        while backlog.len() > self.min_backlog_len
            && backlog.last().copied() == regions.last
        {
            backlog.pop();
        }

        // Build the pod with the merged regions.
        let pod = Regions {
            size: Size::new(regions.size.x, self.first),
            expand: regions.expand,
            full: self.full,
            backlog: &backlog,
            last: regions.last,
        };

        // Extract the not-yet-processed frames.
        let mut frames = self
            .multi
            .layout_full(engine, pod)?
            .into_iter()
            .skip(self.backlog.len());

        // Ensure that the backlog never shrinks, so that unwrapping below is at
        // least fairly safe. Note that the whole region juggling here is
        // fundamentally not ideal: It is a compatibility layer between the old
        // (all regions provided upfront) & new (each region provided on-demand,
        // like an iterator) layout model. This approach is not 100% correct, as
        // in the old model later regions could have an effect on earlier
        // frames, but it's the best we can do for now, until the multi
        // layouters are refactored to the new model.
        self.min_backlog_len = self.min_backlog_len.max(backlog.len());

        // Save the first frame.
        let frame = frames.next().unwrap();

        // If there's more, return a `spill`.
        let mut spill = None;
        if frames.next().is_some() {
            spill = Some(self);
        }

        Ok((frame, spill))
    }

    /// The alignment of the breakable block.
    pub fn align(&self) -> Axes<FixedAlignment> {
        self.multi.align
    }
}

/// A child that encapsulates a prepared placed element.
#[derive(Debug)]
pub struct PlacedChild<'a> {
    pub align_x: FixedAlignment,
    pub align_y: Smart<Option<FixedAlignment>>,
    pub scope: PlacementScope,
    pub float: bool,
    pub clearance: Abs,
    pub delta: Axes<Rel<Abs>>,
    elem: &'a Packed<PlaceElem>,
    styles: StyleChain<'a>,
    locator: Locator<'a>,
    alignment: Smart<Alignment>,
    cell: CachedCell<SourceResult<Frame>>,
}

impl PlacedChild<'_> {
    /// Build the child's frame given the region's base size.
    pub fn layout(&self, engine: &mut Engine, base: Size) -> SourceResult<Frame> {
        self.cell.get_or_init(base, |base| {
            let align = self.alignment.unwrap_or_else(|| Alignment::CENTER);
            let aligned = AlignElem::alignment.set(align).wrap();
            let styles = self.styles.chain(&aligned);

            let mut frame = layout_and_modify(styles, |styles| {
                crate::layout_frame(
                    engine,
                    &self.elem.body,
                    self.locator.relayout(),
                    styles,
                    Region::new(base, Axes::splat(false)),
                )
            })?;

            if self.float {
                frame.set_parent(FrameParent::new(
                    self.elem.location().unwrap(),
                    Inherit::Yes,
                ));
            }

            Ok(frame)
        })
    }

    /// The element's location.
    pub fn location(&self) -> Location {
        self.elem.location().unwrap()
    }
}

/// A child that encapsulates a prepared wrap element.
///
/// Wrap elements create cutout regions that text flows around.
#[derive(Debug)]
pub struct WrapChild<'a> {
    /// Which side the wrap content appears on (logical Start/End).
    pub side: CutoutSide,
    /// The scope of the wrap (column or parent).
    pub scope: PlacementScope,
    /// The clearance between wrap content and flowing text.
    pub clearance: Abs,
    /// The wrap element itself.
    elem: &'a Packed<WrapElem>,
    /// The styles applicable to this wrap.
    styles: StyleChain<'a>,
    /// The locator for this wrap element.
    locator: Locator<'a>,
    /// Cached layout result.
    cell: CachedCell<SourceResult<Frame>>,
}

impl WrapChild<'_> {
    /// Build the child's frame given the region's base size.
    pub fn layout(&self, engine: &mut Engine, base: Size) -> SourceResult<Frame> {
        self.cell.get_or_init(base, |base| {
            let mut frame = layout_and_modify(self.styles, |styles| {
                crate::layout_frame(
                    engine,
                    &self.elem.body,
                    self.locator.relayout(),
                    styles,
                    Region::new(base, Axes::splat(false)),
                )
            })?;

            // Set parent for introspection linking.
            frame.set_parent(FrameParent::new(
                self.elem.location().unwrap(),
                Inherit::Yes,
            ));

            Ok(frame)
        })
    }

    /// The element's location.
    pub fn location(&self) -> Location {
        self.elem.location().unwrap()
    }
}

/// A child that encapsulates a prepared masthead element.
///
/// Masthead elements create fixed-width cutout regions that text flows around.
/// Unlike wrap elements which derive width from their body content, mastheads
/// have an explicit width parameter.
#[derive(Debug)]
pub struct MastheadChild<'a> {
    /// Which side the masthead content appears on (logical Start/End).
    pub side: CutoutSide,
    /// The scope of the masthead (column or parent).
    pub scope: PlacementScope,
    /// The clearance between masthead content and flowing text.
    pub clearance: Abs,
    /// The explicit width of the masthead column.
    pub width: Abs,
    /// The masthead element itself.
    elem: &'a Packed<MastheadElem>,
    /// The styles applicable to this masthead.
    styles: StyleChain<'a>,
    /// The locator for this masthead element.
    locator: Locator<'a>,
    /// Cached layout result.
    cell: CachedCell<SourceResult<Frame>>,
}

impl MastheadChild<'_> {
    /// Build the child's frame given the region's base size.
    ///
    /// Unlike WrapChild, the masthead uses its explicit width parameter
    /// to constrain the body content.
    pub fn layout(&self, engine: &mut Engine, base: Size) -> SourceResult<Frame> {
        self.cell.get_or_init(base, |base| {
            // Use the explicit width for the masthead region
            let masthead_base = Size::new(self.width, base.y);

            let mut frame = layout_and_modify(self.styles, |styles| {
                crate::layout_frame(
                    engine,
                    &self.elem.body,
                    self.locator.relayout(),
                    styles,
                    Region::new(masthead_base, Axes::splat(false)),
                )
            })?;

            // Set parent for introspection linking.
            frame.set_parent(FrameParent::new(
                self.elem.location().unwrap(),
                Inherit::Yes,
            ));

            Ok(frame)
        })
    }

    /// The element's location.
    pub fn location(&self) -> Location {
        self.elem.location().unwrap()
    }
}

/// Wraps a parameterized computation and caches its latest output.
///
/// - When the computation is performed multiple times consecutively with the
///   same argument, reuses the cache.
/// - When the argument changes, the new output is cached.
#[derive(Clone)]
struct CachedCell<T>(RefCell<Option<(u128, T)>>);

impl<T> CachedCell<T> {
    /// Create an empty cached cell.
    fn new() -> Self {
        Self(RefCell::new(None))
    }

    /// Perform the computation `f` with caching.
    fn get_or_init<F, I>(&self, input: I, f: F) -> T
    where
        I: Hash,
        T: Clone,
        F: FnOnce(I) -> T,
    {
        let input_hash = typst_utils::hash128(&input);

        let mut slot = self.0.borrow_mut();
        if let Some((hash, output)) = &*slot
            && *hash == input_hash
        {
            return output.clone();
        }

        let output = f(input);
        *slot = Some((input_hash, output.clone()));
        output
    }
}

impl<T> Default for CachedCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Debug for CachedCell<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad("CachedCell(..)")
    }
}
