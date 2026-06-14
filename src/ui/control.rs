use crate::{action, layout, paint, text, ui};

pub fn panel(id: ui::Id) -> ui::Node {
    ui::Node::container(id, layout::Axis::Vertical)
        .with_background(paint::Color::rgb(0.18, 0.20, 0.24))
        .with_hover_background(paint::Color::rgb(0.24, 0.27, 0.32))
        .with_focus_background(paint::Color::rgb(0.16, 0.30, 0.58))
        .with_active_background(paint::Color::rgb(0.12, 0.45, 0.26))
        .with_disabled_background(paint::Color::rgb(0.10, 0.10, 0.12))
        .with_label_color(paint::Color::rgb(0.92, 0.94, 0.98))
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
