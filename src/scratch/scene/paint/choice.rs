use crate::icon as framework_icon;
use crate::scratch::{geometry, layout, theme::Theme, view};

use super::super::{Brush, Icon, Quad, Scene, Stroke};
use super::super::{Rounding, Style};

pub(super) fn paint(frame: &layout::frame::Frame, scene: &mut Scene, theme: &Theme) {
    let Some(selected) = selected(frame) else {
        return;
    };
    let choice = theme.choice();
    let mark = mark_rect(frame.rect(), choice.mark_inset, choice.mark_size);
    let mark_rounding = match frame.role() {
        view::node::Role::Radio => Rounding::relative(1.0),
        _ => theme.control().rounding,
    };

    scene.push_quad(
        Quad::new(mark, choice.mark)
            .with_rounding(mark_rounding)
            .with_stroke(Stroke::new(Brush::solid(choice.outline), 1.0)),
    );

    if !selected {
        return;
    }

    match frame.role() {
        view::node::Role::Checkbox => {
            scene.push_icon(Icon::new(
                mark,
                framework_icon::Icon::phosphor(framework_icon::Id::new("check"))
                    .with_style(framework_icon::Style::Bold),
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

fn selected(frame: &layout::frame::Frame) -> Option<bool> {
    match frame.role() {
        view::node::Role::Checkbox => Some(frame.checkbox()?.checked()),
        view::node::Role::Radio => Some(frame.radio()?.selected()),
        _ => None,
    }
}

fn mark_rect(rect: geometry::Rect, inset: i32, size: i32) -> geometry::Rect {
    geometry::Rect::new(
        rect.x().saturating_add(inset),
        rect.y()
            .saturating_add((rect.height().saturating_sub(size)) / 2),
        size,
        size,
    )
}

fn inset(rect: geometry::Rect, inset: i32) -> geometry::Rect {
    geometry::Rect::new(
        rect.x().saturating_add(inset),
        rect.y().saturating_add(inset),
        rect.width().saturating_sub(inset.saturating_mul(2)),
        rect.height().saturating_sub(inset.saturating_mul(2)),
    )
}
