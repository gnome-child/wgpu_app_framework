use crate::icon as icons;
use crate::{geometry, layout, theme::Theme, view};

use super::super::{Brush, Color, Icon, Quad, Scene, Stroke, Visuals};
use super::super::{Rounding, Style};

pub(super) fn paint(frame: &layout::Frame, scene: &mut Scene, theme: &Theme, visuals: &Visuals) {
    let Some(selected) = selected(frame) else {
        return;
    };
    let choice = theme.choice();
    let mark = layout::choice_mark_rect(frame.rect(), theme);
    let mark_rounding = match frame.role() {
        view::node::Role::Radio => Rounding::relative(1.0),
        _ => theme.control().rounding,
    };
    let mut mark_quad = Quad::new(mark, choice.mark).with_rounding(mark_rounding);
    if choice.outline.channels().3 > 0 {
        mark_quad = mark_quad.with_stroke(Stroke::new(Brush::solid(choice.outline), 1.0));
    }

    if choice.mark.channels().3 > 0 || choice.outline.channels().3 > 0 {
        scene.push_quad(mark_quad);
    }

    if let Some(tint) = choice_tint_for(frame, theme, visuals) {
        scene.push_quad(Quad::new(mark, tint).with_rounding(mark_rounding));
    }

    if !selected {
        return;
    }

    match frame.role() {
        view::node::Role::Checkbox => {
            scene.push_icon(Icon::new(
                mark,
                icons::Icon::phosphor(icons::Id::new("check")).with_style(icons::Style::Bold),
                choice.indicator,
                choice.icon_size,
            ));
        }
        view::node::Role::Radio => {
            scene.push_quad(
                Quad::styled(inset(mark, 4), Style::filled(choice.indicator))
                    .with_rounding(Rounding::relative(1.0)),
            );
        }
        _ => {}
    }
}

fn choice_tint_for(frame: &layout::Frame, theme: &Theme, visuals: &Visuals) -> Option<Color> {
    if !frame.is_enabled() {
        return None;
    }
    let target_visual = frame
        .target()
        .map(|target| visuals.target(target))
        .unwrap_or_default();

    if target_visual.active() || target_visual.pressed() {
        Some(theme.choice().pressed_tint)
    } else if target_visual.hovered() {
        Some(theme.choice().hover_tint)
    } else {
        None
    }
}

fn selected(frame: &layout::Frame) -> Option<bool> {
    match frame.role() {
        view::node::Role::Checkbox => Some(frame.checkbox()?.checked()),
        view::node::Role::Radio => Some(frame.radio()?.selected()),
        _ => None,
    }
}

fn inset(rect: geometry::Rect, inset: i32) -> geometry::Rect {
    geometry::Rect::new(
        rect.x().saturating_add(inset),
        rect.y().saturating_add(inset),
        rect.width().saturating_sub(inset.saturating_mul(2)),
        rect.height().saturating_sub(inset.saturating_mul(2)),
    )
}
