use crate::geometry::{Rect, area, point};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Fit,
    Fill,
    Fixed(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    min: area::Logical,
    max: area::Logical,
}

pub type Constraints = Limits;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Insets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gap {
    main: f32,
    cross: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Intrinsic {
    min: area::Logical,
    natural: area::Logical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxModel {
    width: Size,
    height: Size,
    min: area::Logical,
    max: area::Logical,
    padding: Insets,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Layout {
    Overlay,
    Stack(Stack),
    Grid(Grid),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stack {
    axis: Axis,
    gap: Gap,
    align: Align,
    cross_align: Align,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Track {
    Fixed(f32),
    Fit,
    Fr(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    columns: Vec<Track>,
    rows: Vec<Track>,
    gap: Gap,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item<K> {
    key: K,
    box_model: BoxModel,
    layout: Layout,
    children: Vec<Item<K>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame<K> {
    key: K,
    rect: Rect,
    content_rect: Rect,
    children: Vec<Frame<K>>,
}

pub trait Measurer<K> {
    fn measure(&mut self, key: &K, limits: Limits) -> Intrinsic;
}

#[derive(Debug, Default)]
pub struct Engine;

#[derive(Debug)]
struct MeasureCache<K> {
    entries: Vec<(K, Limits, Intrinsic)>,
}

impl<K> Default for MeasureCache<K> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<K: Clone + PartialEq> MeasureCache<K> {
    fn measure<M>(&mut self, key: &K, limits: Limits, measurer: &mut M) -> Intrinsic
    where
        M: Measurer<K>,
    {
        if let Some((_, _, intrinsic)) = self
            .entries
            .iter()
            .find(|(entry_key, entry_limits, _)| entry_key == key && *entry_limits == limits)
        {
            return *intrinsic;
        }

        let intrinsic = measurer.measure(key, limits);
        self.entries.push((key.clone(), limits, intrinsic));
        intrinsic
    }
}

impl Limits {
    pub fn new(min: area::Logical, max: area::Logical) -> Self {
        Self {
            min: sanitize_area(min),
            max: sanitize_area(max),
        }
        .normalize()
    }

    pub fn loose(max: area::Logical) -> Self {
        Self::new(area::logical(0.0, 0.0), max)
    }

    pub fn tight(area: area::Logical) -> Self {
        Self::new(area, area)
    }

    pub fn min(self) -> area::Logical {
        self.min
    }

    pub fn max(self) -> area::Logical {
        self.max
    }

    pub fn constrain(self, area: area::Logical) -> area::Logical {
        area::logical(
            area.width().max(self.min.width()).min(self.max.width()),
            area.height().max(self.min.height()).min(self.max.height()),
        )
    }

    fn inset(self, insets: Insets) -> Self {
        Self::new(
            area::logical(
                (self.min.width() - insets.horizontal()).max(0.0),
                (self.min.height() - insets.vertical()).max(0.0),
            ),
            area::logical(
                (self.max.width() - insets.horizontal()).max(0.0),
                (self.max.height() - insets.vertical()).max(0.0),
            ),
        )
    }

    fn normalize(self) -> Self {
        Self {
            min: area::logical(
                self.min.width().min(self.max.width()),
                self.min.height().min(self.max.height()),
            ),
            max: self.max,
        }
    }
}

impl Insets {
    pub const ZERO: Self = Self {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
    };

    pub const fn splat(value: f32) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }

    pub fn horizontal(self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(self) -> f32 {
        self.top + self.bottom
    }

    fn sanitize(self) -> Self {
        Self {
            left: self.left.max(0.0),
            top: self.top.max(0.0),
            right: self.right.max(0.0),
            bottom: self.bottom.max(0.0),
        }
    }
}

impl Gap {
    pub const ZERO: Self = Self {
        main: 0.0,
        cross: 0.0,
    };

    pub fn new(main: f32, cross: f32) -> Self {
        Self {
            main: main.max(0.0),
            cross: cross.max(0.0),
        }
    }

    pub fn uniform(value: f32) -> Self {
        Self::new(value, value)
    }

    pub fn main(self) -> f32 {
        self.main
    }

    pub fn cross(self) -> f32 {
        self.cross
    }
}

impl Intrinsic {
    pub fn new(min: area::Logical, natural: area::Logical) -> Self {
        let min = sanitize_area(min);
        let natural = sanitize_area(natural);

        Self {
            min,
            natural: area::logical(
                natural.width().max(min.width()),
                natural.height().max(min.height()),
            ),
        }
    }

    pub fn zero() -> Self {
        Self::fixed(area::logical(0.0, 0.0))
    }

    pub fn fixed(area: area::Logical) -> Self {
        let area = sanitize_area(area);

        Self {
            min: area,
            natural: area,
        }
    }

    pub fn min(self) -> area::Logical {
        self.min
    }

    pub fn natural(self) -> area::Logical {
        self.natural
    }
}

impl BoxModel {
    pub fn new(width: Size, height: Size) -> Self {
        Self {
            width,
            height,
            min: area::logical(0.0, 0.0),
            max: area::logical(f32::INFINITY, f32::INFINITY),
            padding: Insets::ZERO,
        }
    }

    pub fn fill() -> Self {
        Self::new(Size::Fill, Size::Fill)
    }

    pub fn fit() -> Self {
        Self::new(Size::Fit, Size::Fit)
    }

    pub fn width(self) -> Size {
        self.width
    }

    pub fn height(self) -> Size {
        self.height
    }

    pub fn min(self) -> area::Logical {
        self.min
    }

    pub fn max(self) -> area::Logical {
        self.max
    }

    pub fn padding(self) -> Insets {
        self.padding
    }

    pub fn with_size(mut self, width: Size, height: Size) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_min(mut self, min: area::Logical) -> Self {
        self.min = sanitize_area(min);
        self
    }

    pub fn with_max(mut self, max: area::Logical) -> Self {
        self.max = sanitize_area(max);
        self
    }

    pub fn with_padding(mut self, padding: Insets) -> Self {
        self.padding = padding.sanitize();
        self
    }

    fn limits(self, parent: Limits) -> Limits {
        let min = area::logical(
            parent.min().width().max(self.min.width()),
            parent.min().height().max(self.min.height()),
        );
        let max = area::logical(
            parent.max().width().min(self.max.width()),
            parent.max().height().min(self.max.height()),
        );

        Limits::new(min, max)
    }
}

impl Layout {
    pub fn overlay() -> Self {
        Self::Overlay
    }

    pub fn row() -> Self {
        Self::Stack(Stack::row())
    }

    pub fn column() -> Self {
        Self::Stack(Stack::column())
    }

    pub fn grid(grid: Grid) -> Self {
        Self::Grid(grid)
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::overlay()
    }
}

impl Stack {
    pub fn new(axis: Axis) -> Self {
        Self {
            axis,
            gap: Gap::ZERO,
            align: Align::Start,
            cross_align: Align::Stretch,
        }
    }

    pub fn row() -> Self {
        Self::new(Axis::Horizontal)
    }

    pub fn column() -> Self {
        Self::new(Axis::Vertical)
    }

    pub fn axis(self) -> Axis {
        self.axis
    }

    pub fn gap(self) -> Gap {
        self.gap
    }

    pub fn align(self) -> Align {
        self.align
    }

    pub fn cross_align(self) -> Align {
        self.cross_align
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = Gap::uniform(gap);
        self
    }

    pub fn with_align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn with_cross_align(mut self, align: Align) -> Self {
        self.cross_align = align;
        self
    }
}

impl Grid {
    pub fn new(columns: Vec<Track>, rows: Vec<Track>) -> Self {
        Self {
            columns,
            rows,
            gap: Gap::ZERO,
        }
    }

    pub fn columns(&self) -> &[Track] {
        &self.columns
    }

    pub fn rows(&self) -> &[Track] {
        &self.rows
    }

    pub fn gap(&self) -> Gap {
        self.gap
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = Gap::uniform(gap);
        self
    }
}

impl Track {
    fn fixed_size(self) -> f32 {
        match self {
            Self::Fixed(value) => value.max(0.0),
            Self::Fit | Self::Fr(_) => 0.0,
        }
    }

    fn fr(self) -> f32 {
        match self {
            Self::Fr(value) => value.max(0.0),
            Self::Fixed(_) | Self::Fit => 0.0,
        }
    }
}

impl<K> Item<K> {
    pub fn new(key: K) -> Self {
        Self {
            key,
            box_model: BoxModel::fill(),
            layout: Layout::overlay(),
            children: Vec::new(),
        }
    }

    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn box_model(&self) -> BoxModel {
        self.box_model
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    pub fn children(&self) -> &[Item<K>] {
        &self.children
    }

    pub fn with_box_model(mut self, box_model: BoxModel) -> Self {
        self.box_model = box_model;
        self
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_children(mut self, children: Vec<Item<K>>) -> Self {
        self.children = children;
        self
    }

    pub fn push_child(&mut self, child: Item<K>) {
        self.children.push(child);
    }
}

impl<K> Frame<K> {
    pub fn new(key: K, rect: Rect, children: Vec<Frame<K>>) -> Self {
        let content_rect = rect;

        Self {
            key,
            rect,
            content_rect,
            children,
        }
    }

    pub fn with_path(key: K, rect: Rect, children: Vec<Frame<K>>) -> Self {
        Self::new(key, rect, children)
    }

    pub fn with_content_rect(
        key: K,
        rect: Rect,
        content_rect: Rect,
        children: Vec<Frame<K>>,
    ) -> Self {
        Self {
            key,
            rect,
            content_rect,
            children,
        }
    }

    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn path(&self) -> &K {
        &self.key
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn content_rect(&self) -> Rect {
        self.content_rect
    }

    pub fn children(&self) -> &[Frame<K>] {
        &self.children
    }

    pub fn with_children(mut self, children: Vec<Frame<K>>) -> Self {
        self.children = children;
        self
    }

    pub fn map_key<T>(self, map: impl Copy + Fn(K) -> T) -> Frame<T> {
        Frame {
            key: map(self.key),
            rect: self.rect,
            content_rect: self.content_rect,
            children: self
                .children
                .into_iter()
                .map(|child| child.map_key(map))
                .collect(),
        }
    }
}

impl<K: Clone + PartialEq> Frame<K> {
    pub fn hit_test(&self, position: point::Logical) -> Option<K> {
        self.hit_test_where(position, |_| true)
    }

    pub fn hit_test_where(
        &self,
        position: point::Logical,
        accepts: impl Copy + Fn(&K) -> bool,
    ) -> Option<K> {
        if !contains(self.rect, position) {
            return None;
        }

        for child in self.children.iter().rev() {
            if let Some(key) = child.hit_test_where(position, accepts) {
                return Some(key);
            }
        }

        accepts(&self.key).then_some(self.key.clone())
    }

    pub fn find_path(&self, key: &K) -> Option<&Frame<K>> {
        if &self.key == key {
            return Some(self);
        }

        self.children.iter().find_map(|child| child.find_path(key))
    }
}

impl Engine {
    pub fn new() -> Self {
        Self
    }

    pub fn layout<K, M>(
        &mut self,
        root: &Item<K>,
        area: area::Logical,
        measurer: &mut M,
    ) -> Frame<K>
    where
        K: Clone + PartialEq,
        M: Measurer<K>,
    {
        let mut cache = MeasureCache::default();
        let limits = Limits::loose(area);
        let measured = self.measure_item(root, limits, measurer, &mut cache);
        let root_area = area::logical(
            resolve_root_axis(root.box_model().width(), measured.width(), area.width()),
            resolve_root_axis(root.box_model().height(), measured.height(), area.height()),
        );
        let rect = Rect::new(point::logical(0.0, 0.0), root_area);

        self.arrange_item(root, rect, measurer, &mut cache)
    }

    fn measure_item<K, M>(
        &mut self,
        item: &Item<K>,
        limits: Limits,
        measurer: &mut M,
        cache: &mut MeasureCache<K>,
    ) -> area::Logical
    where
        K: Clone + PartialEq,
        M: Measurer<K>,
    {
        let box_model = item.box_model();
        let limits = box_model.limits(limits);
        let padding = box_model.padding();
        let content_limits = limits.inset(padding);
        let content = match item.layout() {
            Layout::Overlay => self.measure_overlay(item, content_limits, measurer, cache),
            Layout::Stack(stack) => {
                self.measure_stack(item, content_limits, *stack, measurer, cache)
            }
            Layout::Grid(grid) => self.measure_grid(item, content_limits, grid, measurer, cache),
        };
        let own = cache
            .measure(item.key(), content_limits, measurer)
            .natural();
        let desired = area::logical(
            content.width().max(own.width()) + padding.horizontal(),
            content.height().max(own.height()) + padding.vertical(),
        );

        area::logical(
            resolve_measured_axis(
                box_model.width(),
                desired.width(),
                limits.min().width(),
                limits.max().width(),
            ),
            resolve_measured_axis(
                box_model.height(),
                desired.height(),
                limits.min().height(),
                limits.max().height(),
            ),
        )
    }

    fn measure_overlay<K, M>(
        &mut self,
        item: &Item<K>,
        limits: Limits,
        measurer: &mut M,
        cache: &mut MeasureCache<K>,
    ) -> area::Logical
    where
        K: Clone + PartialEq,
        M: Measurer<K>,
    {
        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;

        for child in item.children() {
            let measured = self.measure_item(child, limits, measurer, cache);
            width = width.max(measured.width());
            height = height.max(measured.height());
        }

        limits.constrain(area::logical(width, height))
    }

    fn measure_stack<K, M>(
        &mut self,
        item: &Item<K>,
        limits: Limits,
        stack: Stack,
        measurer: &mut M,
        cache: &mut MeasureCache<K>,
    ) -> area::Logical
    where
        K: Clone + PartialEq,
        M: Measurer<K>,
    {
        let gap = gap_total(stack.gap().main(), item.children().len());
        let mut main: f32 = gap;
        let mut cross: f32 = 0.0;

        for child in item.children() {
            let measured = self.measure_item(child, limits, measurer, cache);
            match stack.axis() {
                Axis::Vertical => {
                    main += measured.height();
                    cross = cross.max(measured.width());
                }
                Axis::Horizontal => {
                    main += measured.width();
                    cross = cross.max(measured.height());
                }
            }
        }

        let area = match stack.axis() {
            Axis::Vertical => area::logical(cross, main),
            Axis::Horizontal => area::logical(main, cross),
        };

        limits.constrain(area)
    }

    fn measure_grid<K, M>(
        &mut self,
        item: &Item<K>,
        limits: Limits,
        grid: &Grid,
        measurer: &mut M,
        cache: &mut MeasureCache<K>,
    ) -> area::Logical
    where
        K: Clone + PartialEq,
        M: Measurer<K>,
    {
        let columns = grid.columns().len().max(1);
        let rows = grid.rows().len().max(1);
        let child_sizes: Vec<_> = item
            .children()
            .iter()
            .map(|child| self.measure_item(child, limits, measurer, cache))
            .collect();
        let column_sizes = measure_tracks(grid.columns(), columns, &child_sizes, Axis::Horizontal);
        let row_sizes = measure_tracks(grid.rows(), rows, &child_sizes, Axis::Vertical);
        let width = column_sizes.iter().sum::<f32>() + gap_total(grid.gap().main(), columns);
        let height = row_sizes.iter().sum::<f32>() + gap_total(grid.gap().cross(), rows);

        limits.constrain(area::logical(width, height))
    }

    fn arrange_item<K, M>(
        &mut self,
        item: &Item<K>,
        rect: Rect,
        measurer: &mut M,
        cache: &mut MeasureCache<K>,
    ) -> Frame<K>
    where
        K: Clone + PartialEq,
        M: Measurer<K>,
    {
        let padding = item.box_model().padding();
        let content_rect = inset_rect(rect, padding);
        let children = match item.layout() {
            Layout::Overlay => self.arrange_overlay(item, content_rect, measurer, cache),
            Layout::Stack(stack) => self.arrange_stack(item, content_rect, *stack, measurer, cache),
            Layout::Grid(grid) => self.arrange_grid(item, content_rect, grid, measurer, cache),
        };

        Frame::with_content_rect(item.key().clone(), rect, content_rect, children)
    }

    fn arrange_overlay<K, M>(
        &mut self,
        item: &Item<K>,
        content_rect: Rect,
        measurer: &mut M,
        cache: &mut MeasureCache<K>,
    ) -> Vec<Frame<K>>
    where
        K: Clone + PartialEq,
        M: Measurer<K>,
    {
        let available = content_rect.area;
        let limits = Limits::loose(available);

        item.children()
            .iter()
            .map(|child| {
                let measured = self.measure_item(child, limits, measurer, cache);
                let width = resolve_overlay_axis(
                    child.box_model().width(),
                    measured.width(),
                    available.width(),
                );
                let height = resolve_overlay_axis(
                    child.box_model().height(),
                    measured.height(),
                    available.height(),
                );
                let x =
                    content_rect.origin.x() + align_offset(Align::Start, available.width(), width);
                let y = content_rect.origin.y()
                    + align_offset(Align::Start, available.height(), height);
                let rect = Rect::new(point::logical(x, y), area::logical(width, height));

                self.arrange_item(child, rect, measurer, cache)
            })
            .collect()
    }

    fn arrange_stack<K, M>(
        &mut self,
        item: &Item<K>,
        content_rect: Rect,
        stack: Stack,
        measurer: &mut M,
        cache: &mut MeasureCache<K>,
    ) -> Vec<Frame<K>>
    where
        K: Clone + PartialEq,
        M: Measurer<K>,
    {
        let available = content_rect.area;
        let measured = self.measure_children(item, available, measurer, cache);
        let gap = stack.gap().main();
        let total_gap = gap_total(gap, item.children().len());
        let fixed_fit_main = item
            .children()
            .iter()
            .zip(&measured)
            .map(|(child, measured)| match stack.axis() {
                Axis::Vertical => match child.box_model().height() {
                    Size::Fixed(value) => resolve_fixed_axis(value, available.height()),
                    Size::Fit => measured.height(),
                    Size::Fill => 0.0,
                },
                Axis::Horizontal => match child.box_model().width() {
                    Size::Fixed(value) => resolve_fixed_axis(value, available.width()),
                    Size::Fit => measured.width(),
                    Size::Fill => 0.0,
                },
            })
            .sum::<f32>();
        let fill_count = item
            .children()
            .iter()
            .filter(|child| match stack.axis() {
                Axis::Vertical => matches!(child.box_model().height(), Size::Fill),
                Axis::Horizontal => matches!(child.box_model().width(), Size::Fill),
            })
            .count();
        let fill = fill_size(
            main_size(available, stack.axis()),
            fixed_fit_main,
            total_gap,
            fill_count,
        );
        let total_main = fixed_fit_main + fill * fill_count as f32 + total_gap;
        let mut cursor = main_origin(content_rect, stack.axis())
            + align_offset(
                stack.align(),
                main_size(available, stack.axis()),
                total_main,
            );
        let mut children = Vec::with_capacity(item.children().len());

        for (child, measured) in item.children().iter().zip(measured) {
            let (width, height) = match stack.axis() {
                Axis::Vertical => {
                    let height = resolve_stack_main_axis(
                        child.box_model().height(),
                        measured.height(),
                        available.height(),
                        fill,
                    );
                    let width = resolve_stack_cross_axis(
                        child.box_model().width(),
                        measured.width(),
                        available.width(),
                        stack.cross_align(),
                    );

                    (width, height)
                }
                Axis::Horizontal => {
                    let width = resolve_stack_main_axis(
                        child.box_model().width(),
                        measured.width(),
                        available.width(),
                        fill,
                    );
                    let height = resolve_stack_cross_axis(
                        child.box_model().height(),
                        measured.height(),
                        available.height(),
                        stack.cross_align(),
                    );

                    (width, height)
                }
            };
            let origin = match stack.axis() {
                Axis::Vertical => point::logical(
                    content_rect.origin.x()
                        + align_offset(stack.cross_align(), available.width(), width),
                    cursor,
                ),
                Axis::Horizontal => point::logical(
                    cursor,
                    content_rect.origin.y()
                        + align_offset(stack.cross_align(), available.height(), height),
                ),
            };
            let rect = Rect::new(origin, area::logical(width, height));

            children.push(self.arrange_item(child, rect, measurer, cache));
            cursor += main_size(rect.area, stack.axis()) + gap;
        }

        children
    }

    fn arrange_grid<K, M>(
        &mut self,
        item: &Item<K>,
        content_rect: Rect,
        grid: &Grid,
        measurer: &mut M,
        cache: &mut MeasureCache<K>,
    ) -> Vec<Frame<K>>
    where
        K: Clone + PartialEq,
        M: Measurer<K>,
    {
        let columns = grid.columns().len().max(1);
        let rows = grid.rows().len().max(1);
        let limits = Limits::loose(content_rect.area);
        let child_sizes: Vec<_> = item
            .children()
            .iter()
            .map(|child| self.measure_item(child, limits, measurer, cache))
            .collect();
        let column_sizes = arrange_tracks(
            grid.columns(),
            columns,
            content_rect.area.width(),
            grid.gap().main(),
            &child_sizes,
            Axis::Horizontal,
        );
        let row_sizes = arrange_tracks(
            grid.rows(),
            rows,
            content_rect.area.height(),
            grid.gap().cross(),
            &child_sizes,
            Axis::Vertical,
        );
        let mut children = Vec::with_capacity(item.children().len());

        for (index, child) in item.children().iter().enumerate() {
            let column = index % columns;
            let row = index / columns;
            if row >= rows {
                break;
            }

            let x = content_rect.origin.x()
                + column_sizes[..column].iter().sum::<f32>()
                + grid.gap().main() * column as f32;
            let y = content_rect.origin.y()
                + row_sizes[..row].iter().sum::<f32>()
                + grid.gap().cross() * row as f32;
            let rect = Rect::new(
                point::logical(x, y),
                area::logical(column_sizes[column], row_sizes[row]),
            );

            children.push(self.arrange_item(child, rect, measurer, cache));
        }

        children
    }

    fn measure_children<K, M>(
        &mut self,
        item: &Item<K>,
        available: area::Logical,
        measurer: &mut M,
        cache: &mut MeasureCache<K>,
    ) -> Vec<area::Logical>
    where
        K: Clone + PartialEq,
        M: Measurer<K>,
    {
        let limits = Limits::loose(available);

        item.children()
            .iter()
            .map(|child| self.measure_item(child, limits, measurer, cache))
            .collect()
    }
}

fn measure_tracks(
    tracks: &[Track],
    count: usize,
    child_sizes: &[area::Logical],
    axis: Axis,
) -> Vec<f32> {
    let mut sizes = vec![0.0; count];

    for (index, track) in tracks.iter().copied().enumerate().take(count) {
        sizes[index] = track.fixed_size();
    }

    for (index, child) in child_sizes.iter().copied().enumerate() {
        let track_index = match axis {
            Axis::Horizontal => index % count,
            Axis::Vertical => index / count,
        };

        if track_index >= count {
            continue;
        }

        if matches!(tracks.get(track_index), Some(Track::Fit) | None) {
            sizes[track_index] = sizes[track_index].max(main_size(child, axis));
        }
    }

    sizes
}

fn arrange_tracks(
    tracks: &[Track],
    count: usize,
    available: f32,
    gap: f32,
    child_sizes: &[area::Logical],
    axis: Axis,
) -> Vec<f32> {
    let mut sizes = measure_tracks(tracks, count, child_sizes, axis);
    let fixed_fit = sizes.iter().sum::<f32>();
    let total_gap = gap_total(gap, count);
    let remaining = (available - fixed_fit - total_gap).max(0.0);
    let total_fr = tracks.iter().copied().map(Track::fr).sum::<f32>();

    if total_fr > 0.0 {
        for (index, track) in tracks.iter().copied().enumerate().take(count) {
            if let Track::Fr(value) = track {
                sizes[index] = remaining * value.max(0.0) / total_fr;
            }
        }
    }

    sizes
}

fn resolve_root_axis(size: Size, measured: f32, available: f32) -> f32 {
    match size {
        Size::Fixed(value) => resolve_fixed_axis(value, available),
        Size::Fill => available.max(0.0),
        Size::Fit => measured.min(available.max(0.0)),
    }
}

fn resolve_measured_axis(size: Size, desired: f32, min: f32, max: f32) -> f32 {
    let max = max.max(0.0);
    let value = match size {
        Size::Fixed(value) => value.max(0.0),
        Size::Fill | Size::Fit => desired.max(0.0),
    };

    value.max(min.max(0.0)).min(max)
}

fn resolve_stack_main_axis(size: Size, measured: f32, available: f32, fill: f32) -> f32 {
    match size {
        Size::Fixed(value) => resolve_fixed_axis(value, available),
        Size::Fill => fill,
        Size::Fit => measured.min(available.max(0.0)),
    }
}

fn resolve_stack_cross_axis(size: Size, measured: f32, available: f32, align: Align) -> f32 {
    match size {
        Size::Fixed(value) => resolve_fixed_axis(value, available),
        Size::Fill => available.max(0.0),
        Size::Fit if align == Align::Stretch => available.max(0.0),
        Size::Fit => measured.min(available.max(0.0)),
    }
}

fn resolve_overlay_axis(size: Size, measured: f32, available: f32) -> f32 {
    match size {
        Size::Fixed(value) => resolve_fixed_axis(value, available),
        Size::Fill => available.max(0.0),
        Size::Fit => measured.min(available.max(0.0)),
    }
}

fn resolve_fixed_axis(value: f32, max: f32) -> f32 {
    value.max(0.0).min(max.max(0.0))
}

fn fill_size(available: f32, fixed_fit: f32, gap: f32, fill_count: usize) -> f32 {
    if fill_count == 0 {
        return 0.0;
    }

    ((available - fixed_fit - gap).max(0.0)) / fill_count as f32
}

fn align_offset(align: Align, available: f32, size: f32) -> f32 {
    let extra = (available - size).max(0.0);

    match align {
        Align::Start | Align::Stretch => 0.0,
        Align::Center => extra / 2.0,
        Align::End => extra,
    }
}

fn main_size(area: area::Logical, axis: Axis) -> f32 {
    match axis {
        Axis::Horizontal => area.width(),
        Axis::Vertical => area.height(),
    }
}

fn main_origin(rect: Rect, axis: Axis) -> f32 {
    match axis {
        Axis::Horizontal => rect.origin.x(),
        Axis::Vertical => rect.origin.y(),
    }
}

fn gap_total(gap: f32, count: usize) -> f32 {
    gap.max(0.0) * count.saturating_sub(1) as f32
}

fn inset_rect(rect: Rect, insets: Insets) -> Rect {
    Rect::rounded(
        point::logical(rect.origin.x() + insets.left, rect.origin.y() + insets.top),
        area::logical(
            (rect.area.width() - insets.horizontal()).max(0.0),
            (rect.area.height() - insets.vertical()).max(0.0),
        ),
        rect.rounding,
    )
}

fn sanitize_area(area: area::Logical) -> area::Logical {
    area::logical(area.width().max(0.0), area.height().max(0.0))
}

fn contains(rect: Rect, position: point::Logical) -> bool {
    let x = position.x();
    let y = position.y();
    let left = rect.origin.x();
    let top = rect.origin.y();
    let right = left + rect.area.width();
    let bottom = top + rect.area.height();

    x >= left && x < right && y >= top && y < bottom
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[derive(Default)]
    struct TestMeasurer {
        sizes: HashMap<&'static str, area::Logical>,
        calls: HashMap<&'static str, usize>,
    }

    impl Measurer<&'static str> for TestMeasurer {
        fn measure(&mut self, key: &&'static str, _limits: Limits) -> Intrinsic {
            *self.calls.entry(*key).or_default() += 1;
            Intrinsic::fixed(
                self.sizes
                    .get(*key)
                    .copied()
                    .unwrap_or_else(|| area::logical(0.0, 0.0)),
            )
        }
    }

    fn fixed(key: &'static str, width: f32, height: f32) -> Item<&'static str> {
        Item::new(key).with_box_model(BoxModel::new(Size::Fixed(width), Size::Fixed(height)))
    }

    fn run(root: &Item<&'static str>, area: area::Logical) -> Frame<&'static str> {
        Engine::new().layout(root, area, &mut TestMeasurer::default())
    }

    #[test]
    fn fixed_fit_fill_and_clamping_resolve_expected_sizes() {
        let root = Item::new("root")
            .with_layout(Layout::row())
            .with_box_model(BoxModel::new(Size::Fill, Size::Fixed(40.0)))
            .with_children(vec![
                fixed("fixed", -20.0, 20.0),
                fixed("wide", 300.0, 20.0),
                Item::new("fill").with_box_model(BoxModel::new(Size::Fill, Size::Fixed(20.0))),
            ]);

        let frame = run(&root, area::logical(100.0, 40.0));
        let children = frame.children();

        assert_eq!(children[0].rect().area.width(), 0.0);
        assert_eq!(children[1].rect().area.width(), 100.0);
        assert_eq!(children[2].rect().area.width(), 0.0);
    }

    #[test]
    fn fit_parent_sizes_to_fixed_children_padding_and_gaps() {
        let root = Item::new("root")
            .with_layout(Layout::column())
            .with_box_model(
                BoxModel::fit()
                    .with_padding(Insets::splat(5.0))
                    .with_size(Size::Fit, Size::Fit),
            )
            .with_children(vec![fixed("a", 30.0, 10.0), fixed("b", 20.0, 12.0)]);
        let root = root.with_layout(Layout::Stack(Stack::column().with_gap(4.0)));

        let frame = run(&root, area::logical(200.0, 100.0));

        assert_eq!(frame.rect().area.width(), 40.0);
        assert_eq!(frame.rect().area.height(), 36.0);
    }

    #[test]
    fn row_and_column_gaps_alignment_and_stretch_are_deterministic() {
        let root = Item::new("root")
            .with_layout(Layout::Stack(
                Stack::column()
                    .with_gap(5.0)
                    .with_align(Align::Center)
                    .with_cross_align(Align::Stretch),
            ))
            .with_box_model(BoxModel::new(Size::Fixed(100.0), Size::Fixed(80.0)))
            .with_children(vec![
                Item::new("a").with_box_model(BoxModel::new(Size::Fit, Size::Fixed(10.0))),
                Item::new("b").with_box_model(BoxModel::new(Size::Fit, Size::Fixed(15.0))),
            ]);

        let frame = run(&root, area::logical(100.0, 80.0));
        let a = &frame.children()[0];
        let b = &frame.children()[1];

        assert_eq!(a.rect().origin.y(), 25.0);
        assert_eq!(b.rect().origin.y(), 40.0);
        assert_eq!(a.rect().area.width(), 100.0);
        assert_eq!(b.rect().area.width(), 100.0);
    }

    #[test]
    fn grid_places_fixed_fit_and_fr_tracks() {
        let root = Item::new("root")
            .with_layout(Layout::grid(Grid::new(
                vec![Track::Fixed(20.0), Track::Fr(1.0), Track::Fr(2.0)],
                vec![Track::Fixed(10.0), Track::Fr(1.0)],
            )))
            .with_box_model(BoxModel::new(Size::Fixed(150.0), Size::Fixed(70.0)))
            .with_children(vec![
                fixed("a", 1.0, 1.0),
                fixed("b", 1.0, 1.0),
                fixed("c", 1.0, 1.0),
                fixed("d", 1.0, 1.0),
            ]);

        let frame = run(&root, area::logical(150.0, 70.0));
        let b = &frame.children()[1];
        let c = &frame.children()[2];
        let d = &frame.children()[3];

        assert_eq!(b.rect().origin.x(), 20.0);
        assert_eq!(b.rect().area.width(), 130.0 / 3.0);
        assert_eq!(c.rect().area.width(), 260.0 / 3.0);
        assert_eq!(d.rect().origin.y(), 10.0);
    }

    #[test]
    fn overflow_remains_deterministic() {
        let root = Item::new("root")
            .with_layout(Layout::row())
            .with_box_model(BoxModel::new(Size::Fixed(30.0), Size::Fixed(20.0)))
            .with_children(vec![fixed("a", 80.0, 10.0), fixed("b", 80.0, 10.0)]);

        let frame = run(&root, area::logical(30.0, 20.0));

        assert_eq!(frame.children()[0].rect().origin.x(), 0.0);
        assert_eq!(frame.children()[1].rect().origin.x(), 30.0);
    }

    #[test]
    fn child_is_measured_once_per_normal_pass() {
        let root = Item::new("root")
            .with_layout(Layout::row())
            .with_children(vec![fixed("a", 20.0, 10.0)]);
        let mut measurer = TestMeasurer::default();

        let _ = Engine::new().layout(&root, area::logical(100.0, 100.0), &mut measurer);

        assert_eq!(measurer.calls.get("a").copied(), Some(1));
    }

    #[test]
    fn hit_testing_returns_deepest_matching_frame() {
        let layout = Frame::new(
            "root",
            Rect::new(point::logical(0.0, 0.0), area::logical(100.0, 100.0)),
            vec![Frame::new(
                "child",
                Rect::new(point::logical(10.0, 10.0), area::logical(20.0, 20.0)),
                Vec::new(),
            )],
        );

        assert_eq!(layout.hit_test(point::logical(15.0, 15.0)), Some("child"));
        assert_eq!(layout.hit_test(point::logical(90.0, 90.0)), Some("root"));
        assert_eq!(layout.hit_test(point::logical(110.0, 90.0)), None);
    }
}
