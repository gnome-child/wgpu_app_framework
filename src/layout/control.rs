use super::super::{
    context::Source,
    geometry::{Rect, Size},
    theme, view,
};
use super::flow::{Item, Row, SizeHint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MenuRowParts {
    pub glyph: Rect,
    pub label: Rect,
    pub shortcut: Rect,
    pub trailing: Rect,
    pub separator: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PaletteRowParts {
    pub label: Rect,
    pub shortcut: Rect,
}

pub(crate) fn padded_control_extent(content_extent: i32, theme: &theme::Theme) -> i32 {
    content_extent
        .max(0)
        .saturating_add(theme.control().padding.max(0).saturating_mul(2))
}

pub(crate) fn control_content_extent(control_extent: i32, theme: &theme::Theme) -> i32 {
    control_extent
        .max(0)
        .saturating_sub(theme.control().padding.max(0).saturating_mul(2))
}

pub(crate) fn menu_row_parts(
    rect: Rect,
    shortcut_width: i32,
    theme: &theme::Theme,
) -> MenuRowParts {
    let parts = menu_row(0, shortcut_width, theme).layout(rect);
    let glyph = rect_part(&parts, rect, 0);
    let label = rect_part(&parts, rect, 1);
    let shortcut = rect_part(&parts, rect, 2);
    let trailing = rect_part(&parts, rect, 3);
    let separator = Rect::new(
        rect.x(),
        rect.y().saturating_add(
            (rect
                .height()
                .saturating_sub(theme.menu().separator_line_height))
                / 2,
        ),
        rect.width(),
        theme.menu().separator_line_height,
    );

    MenuRowParts {
        glyph,
        label,
        shortcut,
        trailing,
        separator,
    }
}

pub(crate) fn menu_row_width(label_width: i32, shortcut_width: i32, theme: &theme::Theme) -> i32 {
    menu_row(label_width, shortcut_width, theme)
        .size_hint()
        .preferred()
        .width()
}

pub(crate) fn is_menu_panel(node: &view::Node) -> bool {
    !node.children().is_empty() && node.children().iter().all(is_menu_panel_row)
}

pub(crate) fn is_menu_panel_row(node: &view::Node) -> bool {
    node.role() == view::Role::Separator
        || node
            .binding()
            .is_some_and(|binding| binding.source() == Source::Menu)
}

pub(crate) fn palette_row_parts(
    rect: Rect,
    shortcut_width: i32,
    theme: &theme::Theme,
) -> PaletteRowParts {
    let parts = palette_row(0, shortcut_width, theme).layout(rect);

    PaletteRowParts {
        label: rect_part(&parts, rect, 0),
        shortcut: rect_part(&parts, rect, 1),
    }
}

pub(crate) fn palette_row_width(
    label_width: i32,
    shortcut_width: i32,
    theme: &theme::Theme,
) -> i32 {
    palette_row(label_width, shortcut_width, theme)
        .size_hint()
        .preferred()
        .width()
}

pub(crate) fn choice_row_width(label_width: i32, theme: &theme::Theme) -> i32 {
    choice_row(label_width, theme)
        .size_hint()
        .preferred()
        .width()
}

pub(crate) fn choice_mark_rect(rect: Rect, theme: &theme::Theme) -> Rect {
    let choice = theme.choice();
    centered_rect(
        row_part(choice_row(0, theme), rect, 0),
        choice.mark_size,
        choice.mark_size,
    )
}

pub(crate) fn choice_label_rect(rect: Rect, theme: &theme::Theme) -> Rect {
    row_part(choice_row(0, theme), rect, 1)
}

pub(crate) fn slider_label_rect(
    rect: Rect,
    measured_label_width: i32,
    theme: &theme::Theme,
) -> Rect {
    row_part(slider_row(measured_label_width, theme), rect, 0)
}

pub(crate) fn slider_track_rect(
    rect: Rect,
    measured_label_width: i32,
    theme: &theme::Theme,
) -> Rect {
    let area = row_part(slider_row(measured_label_width, theme), rect, 1);
    centered_rect(area, area.width(), theme.slider().track_height)
}

pub(crate) fn slider_row_width(label_width: i32, theme: &theme::Theme) -> i32 {
    slider_row(label_width, theme)
        .size_hint()
        .preferred()
        .width()
}

pub(crate) fn slider_thumb_rect(
    rect: Rect,
    slider: &view::Slider,
    measured_label_width: i32,
    theme: &theme::Theme,
) -> Rect {
    let slider_theme = theme.slider();
    let track = slider_track_rect(rect, measured_label_width, theme);
    let filled_width = ((track.width() as f64) * slider_fraction(slider)).round() as i32;
    let thumb_center = track.x().saturating_add(filled_width);

    Rect::new(
        thumb_center.saturating_sub(slider_theme.thumb_width / 2),
        rect.y()
            .saturating_add((rect.height().saturating_sub(slider_theme.thumb_height)) / 2),
        slider_theme.thumb_width,
        slider_theme.thumb_height,
    )
}

pub(crate) fn slider_active_rect(
    rect: Rect,
    slider: &view::Slider,
    measured_label_width: i32,
    theme: &theme::Theme,
) -> Rect {
    union(
        slider_track_rect(rect, measured_label_width, theme),
        slider_thumb_rect(rect, slider, measured_label_width, theme),
    )
}

fn slider_fraction(slider: &view::Slider) -> f64 {
    let span = slider.end() - slider.start();
    if span.abs() <= f64::EPSILON {
        return 0.0;
    }

    ((slider.value() - slider.start()) / span).clamp(0.0, 1.0)
}

fn palette_row(label_width: i32, shortcut_width: i32, theme: &theme::Theme) -> Row {
    let row_height = theme.control().height;

    Row::new()
        .gap(theme.menu().padding)
        .padding(view::Padding::symmetric(theme.menu().padding, 0))
        .item(Item::grow(SizeHint::new(
            Size::new(0, row_height),
            Size::new(label_width.max(0), row_height),
        )))
        .item(Item::fixed(SizeHint::new(
            Size::new(0, row_height),
            Size::new(shortcut_width.max(0), row_height),
        )))
}

fn menu_row(label_width: i32, shortcut_width: i32, theme: &theme::Theme) -> Row {
    let row_height = theme.menu().row_height;
    let side_width = padded_control_extent(control_content_extent(row_height, theme), theme);

    Row::new()
        .item(Item::fixed(SizeHint::fixed(Size::new(
            side_width, row_height,
        ))))
        .item(Item::grow(SizeHint::new(
            Size::new(0, row_height),
            Size::new(label_width.max(0), row_height),
        )))
        .item(Item::fixed(SizeHint::new(
            Size::new(0, row_height),
            Size::new(shortcut_width.max(0), row_height),
        )))
        .item(Item::fixed(SizeHint::fixed(Size::new(
            side_width, row_height,
        ))))
}

fn choice_row(label_width: i32, theme: &theme::Theme) -> Row {
    let choice = theme.choice();
    let row_height = theme.control().height;

    Row::new()
        .gap(choice.label_gap)
        .padding(view::Padding::edges(
            0,
            theme.control().padding,
            0,
            choice.mark_inset,
        ))
        .item(Item::fixed(SizeHint::fixed(Size::new(
            choice.mark_size,
            row_height,
        ))))
        .item(Item::grow(SizeHint::new(
            Size::new(0, row_height),
            Size::new(label_width.max(0), row_height),
        )))
}

fn slider_row(measured_label_width: i32, theme: &theme::Theme) -> Row {
    let slider = theme.slider();
    let row_height = theme.control().height;
    let label_width = slider.label_width.max(measured_label_width).max(0);

    Row::new()
        .gap(slider.gap)
        .padding(view::Padding::symmetric(slider.inset, 0))
        .item(Item::fixed(SizeHint::new(
            Size::new(0, row_height),
            Size::new(label_width, row_height),
        )))
        .item(Item::grow(SizeHint::new(
            Size::new(0, row_height),
            Size::new(160, row_height),
        )))
}

fn row_part(row: Row, rect: Rect, index: usize) -> Rect {
    row.layout(rect)
        .get(index)
        .copied()
        .unwrap_or_else(|| Rect::new(rect.x(), rect.y(), 0, rect.height()))
}

fn rect_part(parts: &[Rect], fallback: Rect, index: usize) -> Rect {
    parts
        .get(index)
        .copied()
        .unwrap_or_else(|| Rect::new(fallback.x(), fallback.y(), 0, fallback.height()))
}

fn centered_rect(area: Rect, width: i32, height: i32) -> Rect {
    let width = width.max(0).min(area.width());
    let height = height.max(0).min(area.height());
    Rect::new(
        area.x()
            .saturating_add((area.width().saturating_sub(width)) / 2),
        area.y()
            .saturating_add((area.height().saturating_sub(height)) / 2),
        width,
        height,
    )
}

fn union(a: Rect, b: Rect) -> Rect {
    let x = a.x().min(b.x());
    let y = a.y().min(b.y());
    let right = a.right().max(b.right());
    let bottom = a.bottom().max(b.bottom());

    Rect::new(x, y, right.saturating_sub(x), bottom.saturating_sub(y))
}
