use crate::{action, layout, paint, text, ui};

pub fn panel(id: ui::Id) -> ui::Node {
    ui::Node::container(id, layout::Axis::Vertical)
        .with_background(paint::Color::rgb(0.18, 0.20, 0.24))
        .with_stroke(paint::Stroke {
            brush: paint::Brush::Solid(paint::Color::rgb(0.30, 0.33, 0.38)),
            width: 1.0,
        })
        .with_hover_tint(paint::Color::rgba(1.0, 1.0, 1.0, 0.08))
        .with_pressed_tint(paint::Color::rgba(0.0, 0.0, 0.0, 0.16))
        .with_active_tint(paint::Color::rgba(0.00, 0.80, 0.28, 0.36))
        .with_busy_tint(paint::Color::rgba(1.00, 0.68, 0.05, 0.46))
        .with_disabled_tint(paint::Color::rgba(0.0, 0.0, 0.0, 0.44))
        .with_focus_outline(paint::Color::rgb(0.24, 0.48, 0.96), 2.0, 2.0)
        .with_label_color(paint::Color::rgb(0.92, 0.94, 0.98))
        .with_busy_label_color(paint::Color::rgb(1.00, 0.94, 0.78))
        .with_disabled_label_color(paint::Color::rgb(0.42, 0.44, 0.48))
}

pub fn button(id: ui::Id, action: action::Id) -> ui::Node {
    panel(id)
        .with_action(action)
        .with_interactivity(ui::Interactivity::CONTROL)
        .with_size(layout::Size::Fill, layout::Size::Fixed(160.0))
}

pub fn labeled_button(id: ui::Id, action: action::Id, label: impl Into<String>) -> ui::Node {
    let mut block = text::Block::new(text::Align::Center);
    block.push_run(text::Run::new(label, text::Style::default()));

    button(id, action).with_label(text::Document::from_block(block))
}
