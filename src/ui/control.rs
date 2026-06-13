use crate::{action, layout, paint, ui};

pub fn panel(id: ui::Id) -> ui::Node {
    ui::Node::container(id, layout::Axis::Vertical)
        .with_background(paint::Color::rgb(0.18, 0.20, 0.24))
        .with_hover_background(paint::Color::rgb(0.24, 0.27, 0.32))
        .with_focus_background(paint::Color::rgb(0.16, 0.30, 0.58))
        .with_active_background(paint::Color::rgb(0.12, 0.45, 0.26))
        .with_disabled_background(paint::Color::rgb(0.10, 0.10, 0.12))
}

pub fn button(id: ui::Id, action: action::Id) -> ui::Node {
    panel(id)
        .with_action(action)
        .with_interactivity(ui::Interactivity::CONTROL)
        .with_size(layout::Size::Fill, layout::Size::Fixed(160.0))
}
