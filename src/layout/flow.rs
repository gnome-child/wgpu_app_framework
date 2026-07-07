use super::super::{
    geometry::{Rect, Size},
    view,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Constraints {
    min: Size,
    max: Size,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SizeHint {
    min: Size,
    preferred: Size,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Item {
    hint: SizeHint,
    grow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Row {
    items: Vec<Item>,
    gap: i32,
    padding: view::style::Padding,
    align_items: view::style::Align,
    justify_content: view::style::Align,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Column {
    items: Vec<Item>,
    gap: i32,
    padding: view::style::Padding,
    align_items: view::style::Align,
    justify_content: view::style::Align,
}

impl Constraints {
    pub(crate) fn new(min: Size, max: Size) -> Self {
        let min = min.sanitized();
        let max = Size::new(max.width().max(min.width()), max.height().max(min.height()));

        Self { min, max }
    }

    pub(crate) fn max(self) -> Size {
        self.max
    }

    pub(crate) fn constrain(self, size: Size) -> Size {
        Size::new(
            size.width().clamp(self.min.width(), self.max.width()),
            size.height().clamp(self.min.height(), self.max.height()),
        )
    }
}

impl SizeHint {
    pub(crate) fn new(min: Size, preferred: Size) -> Self {
        let min = min.sanitized();
        let preferred = Size::new(
            preferred.width().max(min.width()),
            preferred.height().max(min.height()),
        );

        Self { min, preferred }
    }

    pub(crate) fn fixed(size: Size) -> Self {
        Self::new(size, size)
    }

    pub(crate) fn min(self) -> Size {
        self.min
    }

    pub(crate) fn preferred(self) -> Size {
        self.preferred
    }

    pub(crate) fn constrained(self, constraints: Constraints) -> Self {
        Self::new(
            constraints.constrain(self.min),
            constraints.constrain(self.preferred),
        )
    }
}

impl Item {
    pub(crate) fn fixed(hint: SizeHint) -> Self {
        Self { hint, grow: false }
    }

    pub(crate) fn grow(hint: SizeHint) -> Self {
        Self { hint, grow: true }
    }
}

impl Row {
    pub(crate) fn new() -> Self {
        Self {
            items: Vec::new(),
            gap: 0,
            padding: view::style::Padding::zero(),
            align_items: view::style::Align::Stretch,
            justify_content: view::style::Align::Start,
        }
    }

    pub(crate) fn gap(mut self, gap: i32) -> Self {
        self.gap = gap.max(0);
        self
    }

    pub(crate) fn padding(mut self, padding: view::style::Padding) -> Self {
        self.padding = padding;
        self
    }

    pub(crate) fn align_items(mut self, align: view::style::Align) -> Self {
        self.align_items = align;
        self
    }

    pub(crate) fn justify_content(mut self, align: view::style::Align) -> Self {
        self.justify_content = align;
        self
    }

    pub(crate) fn item(mut self, item: Item) -> Self {
        self.items.push(item);
        self
    }

    pub(crate) fn size_hint(&self) -> SizeHint {
        let min_width = self
            .items
            .iter()
            .map(|item| item.hint.min().width())
            .sum::<i32>()
            .saturating_add(gap_total(self.gap, self.items.len()))
            .saturating_add(self.padding.horizontal());
        let preferred_width = self
            .items
            .iter()
            .map(|item| item.hint.preferred().width())
            .sum::<i32>()
            .saturating_add(gap_total(self.gap, self.items.len()))
            .saturating_add(self.padding.horizontal());
        let min_height = self
            .items
            .iter()
            .map(|item| item.hint.min().height())
            .max()
            .unwrap_or_default()
            .saturating_add(self.padding.vertical());
        let preferred_height = self
            .items
            .iter()
            .map(|item| item.hint.preferred().height())
            .max()
            .unwrap_or_default()
            .saturating_add(self.padding.vertical());

        SizeHint::new(
            Size::new(min_width, min_height),
            Size::new(preferred_width, preferred_height),
        )
    }

    pub(crate) fn layout(&self, rect: Rect) -> Vec<Rect> {
        let content = inset_rect(rect, self.padding);
        let widths = allocate_main(content.width(), self.gap, &self.items, MainAxis::Width);
        let content_width = widths
            .iter()
            .copied()
            .sum::<i32>()
            .saturating_add(gap_total(self.gap, widths.len()));
        let mut x = content.x().saturating_add(axis_offset(
            self.justify_content,
            content.width(),
            content_width,
        ));

        self.items
            .iter()
            .zip(widths)
            .map(|(item, width)| {
                let height = cross_axis_extent(
                    self.align_items,
                    content.height(),
                    item.hint.preferred().height(),
                );
                let y = content.y().saturating_add(axis_offset(
                    self.align_items,
                    content.height(),
                    height,
                ));
                let child = Rect::new(x, y, width, height);
                x = x.saturating_add(width).saturating_add(self.gap);
                child
            })
            .collect()
    }
}

impl Column {
    pub(crate) fn new() -> Self {
        Self {
            items: Vec::new(),
            gap: 0,
            padding: view::style::Padding::zero(),
            align_items: view::style::Align::Stretch,
            justify_content: view::style::Align::Start,
        }
    }

    pub(crate) fn gap(mut self, gap: i32) -> Self {
        self.gap = gap.max(0);
        self
    }

    pub(crate) fn padding(mut self, padding: view::style::Padding) -> Self {
        self.padding = padding;
        self
    }

    pub(crate) fn align_items(mut self, align: view::style::Align) -> Self {
        self.align_items = align;
        self
    }

    pub(crate) fn justify_content(mut self, align: view::style::Align) -> Self {
        self.justify_content = align;
        self
    }

    pub(crate) fn item(mut self, item: Item) -> Self {
        self.items.push(item);
        self
    }

    pub(crate) fn size_hint(&self) -> SizeHint {
        let min_width = self
            .items
            .iter()
            .map(|item| item.hint.min().width())
            .max()
            .unwrap_or_default()
            .saturating_add(self.padding.horizontal());
        let preferred_width = self
            .items
            .iter()
            .map(|item| item.hint.preferred().width())
            .max()
            .unwrap_or_default()
            .saturating_add(self.padding.horizontal());
        let min_height = self
            .items
            .iter()
            .map(|item| item.hint.min().height())
            .sum::<i32>()
            .saturating_add(gap_total(self.gap, self.items.len()))
            .saturating_add(self.padding.vertical());
        let preferred_height = self
            .items
            .iter()
            .map(|item| item.hint.preferred().height())
            .sum::<i32>()
            .saturating_add(gap_total(self.gap, self.items.len()))
            .saturating_add(self.padding.vertical());

        SizeHint::new(
            Size::new(min_width, min_height),
            Size::new(preferred_width, preferred_height),
        )
    }

    pub(crate) fn layout(&self, rect: Rect) -> Vec<Rect> {
        let content = inset_rect(rect, self.padding);
        let heights = allocate_main(content.height(), self.gap, &self.items, MainAxis::Height);
        let content_height = heights
            .iter()
            .copied()
            .sum::<i32>()
            .saturating_add(gap_total(self.gap, heights.len()));
        let mut y = content.y().saturating_add(axis_offset(
            self.justify_content,
            content.height(),
            content_height,
        ));

        self.items
            .iter()
            .zip(heights)
            .map(|(item, height)| {
                let width = cross_axis_extent(
                    self.align_items,
                    content.width(),
                    item.hint.preferred().width(),
                );
                let x = content.x().saturating_add(axis_offset(
                    self.align_items,
                    content.width(),
                    width,
                ));
                let child = Rect::new(x, y, width, height);
                y = y.saturating_add(height).saturating_add(self.gap);
                child
            })
            .collect()
    }
}

#[derive(Clone, Copy)]
enum MainAxis {
    Width,
    Height,
}

fn allocate_main(available: i32, gap: i32, items: &[Item], axis: MainAxis) -> Vec<i32> {
    if items.is_empty() {
        return Vec::new();
    }

    let available = available.max(0).saturating_sub(gap_total(gap, items.len()));
    let min = items
        .iter()
        .map(|item| axis_size(item.hint.min(), axis))
        .collect::<Vec<_>>();
    let preferred = items
        .iter()
        .map(|item| axis_size(item.hint.preferred(), axis))
        .collect::<Vec<_>>();
    let fixed_preferred = items
        .iter()
        .zip(&preferred)
        .filter_map(|(item, width)| (!item.grow).then_some(*width))
        .sum::<i32>();
    let grow_min = items
        .iter()
        .zip(&min)
        .filter_map(|(item, width)| item.grow.then_some(*width))
        .sum::<i32>();
    let grow_count = items.iter().filter(|item| item.grow).count();

    if grow_count > 0 && available >= fixed_preferred.saturating_add(grow_min) {
        let mut remaining = available
            .saturating_sub(fixed_preferred)
            .saturating_sub(grow_min);
        let base = remaining / grow_count as i32;
        remaining %= grow_count as i32;

        return items
            .iter()
            .zip(min)
            .zip(preferred)
            .map(|((item, min), preferred)| {
                if item.grow {
                    let extra = base + i32::from(remaining > 0);
                    remaining = remaining.saturating_sub(1);
                    min.saturating_add(extra)
                } else {
                    preferred
                }
            })
            .collect();
    }

    let preferred_total = preferred.iter().copied().sum::<i32>();
    if available >= preferred_total {
        return preferred;
    }

    shrink_to_available(preferred, min, available)
}

fn shrink_to_available(mut sizes: Vec<i32>, min: Vec<i32>, available: i32) -> Vec<i32> {
    let mut shortage = sizes.iter().copied().sum::<i32>().saturating_sub(available);

    for index in (0..sizes.len()).rev() {
        if shortage == 0 {
            break;
        }

        let shrink = sizes[index].saturating_sub(min[index]).min(shortage);
        sizes[index] = sizes[index].saturating_sub(shrink);
        shortage = shortage.saturating_sub(shrink);
    }

    for index in (0..sizes.len()).rev() {
        if shortage == 0 {
            break;
        }

        let shrink = sizes[index].min(shortage);
        sizes[index] = sizes[index].saturating_sub(shrink);
        shortage = shortage.saturating_sub(shrink);
    }

    sizes
}

fn axis_size(size: Size, axis: MainAxis) -> i32 {
    match axis {
        MainAxis::Width => size.width(),
        MainAxis::Height => size.height(),
    }
}

fn cross_axis_extent(align: view::style::Align, available: i32, preferred: i32) -> i32 {
    match align {
        view::style::Align::Stretch => available.max(0),
        view::style::Align::Start | view::style::Align::Center | view::style::Align::End => {
            preferred.clamp(0, available.max(0))
        }
    }
}

fn axis_offset(align: view::style::Align, available: i32, content: i32) -> i32 {
    let slack = available.saturating_sub(content);
    match align {
        view::style::Align::Start | view::style::Align::Stretch => 0,
        view::style::Align::Center => slack / 2,
        view::style::Align::End => slack,
    }
}

fn inset_rect(rect: Rect, padding: view::style::Padding) -> Rect {
    Rect::new(
        rect.x().saturating_add(padding.left()),
        rect.y().saturating_add(padding.top()),
        rect.width().saturating_sub(padding.horizontal()),
        rect.height().saturating_sub(padding.vertical()),
    )
}

fn gap_total(gap: i32, child_count: usize) -> i32 {
    gap.max(0)
        .saturating_mul(child_count.saturating_sub(1) as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_preferred_width_sums_children_gaps_and_padding() {
        let row = Row::new()
            .gap(4)
            .padding(view::style::Padding::symmetric(3, 2))
            .item(Item::fixed(SizeHint::fixed(Size::new(10, 5))))
            .item(Item::fixed(SizeHint::fixed(Size::new(20, 7))));

        assert_eq!(row.size_hint().preferred(), Size::new(40, 11));
    }

    #[test]
    fn column_preferred_height_sums_children_gaps_and_padding() {
        let column = Column::new()
            .gap(4)
            .padding(view::style::Padding::symmetric(3, 2))
            .item(Item::fixed(SizeHint::fixed(Size::new(10, 5))))
            .item(Item::fixed(SizeHint::fixed(Size::new(20, 7))));

        assert_eq!(column.size_hint().preferred(), Size::new(26, 20));
    }

    #[test]
    fn row_allocates_fixed_and_grow_items() {
        let row = Row::new()
            .gap(5)
            .item(Item::fixed(SizeHint::fixed(Size::new(20, 10))))
            .item(Item::grow(SizeHint::new(
                Size::new(10, 10),
                Size::new(10, 10),
            )))
            .item(Item::fixed(SizeHint::fixed(Size::new(15, 10))));

        let rects = row.layout(Rect::new(0, 0, 100, 10));

        assert_eq!(rects[0], Rect::new(0, 0, 20, 10));
        assert_eq!(rects[1], Rect::new(25, 0, 55, 10));
        assert_eq!(rects[2], Rect::new(85, 0, 15, 10));
    }

    #[test]
    fn column_allocates_fixed_and_grow_items() {
        let column = Column::new()
            .gap(5)
            .item(Item::fixed(SizeHint::fixed(Size::new(10, 20))))
            .item(Item::grow(SizeHint::new(
                Size::new(10, 10),
                Size::new(10, 10),
            )))
            .item(Item::fixed(SizeHint::fixed(Size::new(10, 15))));

        let rects = column.layout(Rect::new(0, 0, 10, 100));

        assert_eq!(rects[0], Rect::new(0, 0, 10, 20));
        assert_eq!(rects[1], Rect::new(0, 25, 10, 55));
        assert_eq!(rects[2], Rect::new(0, 85, 10, 15));
    }

    #[test]
    fn constraints_clamp_size_hints() {
        let constraints = Constraints::new(Size::new(10, 5), Size::new(30, 15));
        let hint = SizeHint::new(Size::new(4, 4), Size::new(40, 20)).constrained(constraints);

        assert_eq!(hint.min(), Size::new(10, 5));
        assert_eq!(hint.preferred(), Size::new(30, 15));
    }
}
