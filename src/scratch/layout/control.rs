use super::super::{
    geometry::{Point, Rect},
    theme,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::scratch) struct MenuRowSlots {
    pub glyph: Rect,
    pub label: Rect,
    pub shortcut: Rect,
    pub trailing: Rect,
    pub separator: Rect,
}

pub(in crate::scratch) fn padded_control_extent(content_extent: i32, theme: &theme::Theme) -> i32 {
    content_extent
        .max(0)
        .saturating_add(theme.control().padding.max(0).saturating_mul(2))
}

pub(in crate::scratch) fn control_content_extent(control_extent: i32, theme: &theme::Theme) -> i32 {
    control_extent
        .max(0)
        .saturating_sub(theme.control().padding.max(0).saturating_mul(2))
}

pub(in crate::scratch) fn menu_row_slots(
    rect: Rect,
    shortcut_width: i32,
    theme: &theme::Theme,
) -> MenuRowSlots {
    let side_slot_width = padded_control_extent(
        control_content_extent(theme.menu().row_height, theme),
        theme,
    );
    let glyph_width = side_slot_width.min(rect.width());
    let trailing_width = side_slot_width.min(rect.width().saturating_sub(glyph_width));
    let content_width = rect.width().saturating_sub(glyph_width + trailing_width);
    let shortcut_width = shortcut_width.max(0).min(content_width);
    let glyph = Rect::new(rect.x(), rect.y(), glyph_width, rect.height());
    let trailing = Rect::new(
        rect.right().saturating_sub(trailing_width),
        rect.y(),
        trailing_width,
        rect.height(),
    );
    let shortcut = Rect::new(
        trailing.x().saturating_sub(shortcut_width),
        rect.y(),
        shortcut_width,
        rect.height(),
    );
    let label = Rect::new(
        glyph.right(),
        rect.y(),
        shortcut.x().saturating_sub(glyph.right()),
        rect.height(),
    );
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

    MenuRowSlots {
        glyph,
        label,
        shortcut,
        trailing,
        separator,
    }
}

pub(in crate::scratch) fn slider_track_rect(rect: Rect, theme: &theme::Theme) -> Rect {
    let slider = theme.slider();
    let track_left = rect
        .x()
        .saturating_add(slider.inset)
        .saturating_add(slider.label_width)
        .saturating_add(slider.gap);
    let track_right = rect.right().saturating_sub(slider.inset);
    let track_width = track_right.saturating_sub(track_left);
    let track_y = rect
        .y()
        .saturating_add((rect.height().saturating_sub(slider.track_height)) / 2);

    Rect::new(track_left, track_y, track_width, slider.track_height)
}

pub(in crate::scratch) fn slider_fraction_at(
    rect: Rect,
    theme: &theme::Theme,
    point: Point,
) -> f64 {
    let track = slider_track_rect(rect, theme);
    let width = track.width().max(1) as f64;
    let offset = point.x().saturating_sub(track.x()) as f64;

    offset / width
}
