use crate::geometry::rect;
use crate::{layout, menu, paint, text, ui};

pub const MENU_POPUP: ui::Id = ui::Id::new("__menu_popup");

pub fn label(id: ui::Id, label: impl Into<String>) -> ui::Node {
    ui::Node::leaf(id)
        .with_label(document(label))
        .with_label_color(paint::Color::rgb(0.88, 0.90, 0.94))
        .with_size(layout::Size::Fill, layout::Size::Fixed(24.0))
}

pub fn separator(id: ui::Id) -> ui::Node {
    ui::Node::leaf(id)
        .with_background(paint::Color::rgba(1.0, 1.0, 1.0, 0.12))
        .with_size(layout::Size::Fill, layout::Size::Fixed(1.0))
}

pub fn menu_bar(id: ui::Id, bar: menu::Bar) -> ui::Node {
    let mut node = ui::Node::container(id, layout::Axis::Horizontal)
        .with_menu_bar(bar.clone())
        .with_background(paint::Color::rgba(0.08, 0.09, 0.11, 0.92))
        .with_stroke(paint::Stroke {
            brush: paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.10)),
            width: 1.0,
        })
        .with_size(layout::Size::Fill, layout::Size::Fixed(30.0));

    for menu in bar.menus() {
        node.push_child(menu_title(menu));
    }

    node
}

pub fn document(label: impl Into<String>) -> text::Document {
    let mut block = text::Block::new(text::Align::Center);
    block.push_run(text::Run::new(label, text::Style::default()));

    text::Document::from_block(block)
}

fn menu_title(menu: &menu::Menu) -> ui::Node {
    ui::Node::leaf(ui::Id::new(menu.id().as_str()))
        .with_intent(ui::Intent::OpenMenu(menu.id()))
        .with_interactivity(ui::Interactivity::CONTROL)
        .with_label(document(menu.label()))
        .with_background(paint::Color::rgba(1.0, 1.0, 1.0, 0.00))
        .with_hover_tint(paint::Color::rgba(1.0, 1.0, 1.0, 0.10))
        .with_pressed_tint(paint::Color::rgba(0.0, 0.0, 0.0, 0.18))
        .with_active_tint(paint::Color::rgba(0.26, 0.52, 1.0, 0.24))
        .with_focus_outline(paint::Color::rgba(0.32, 0.56, 1.0, 0.95), 1.0, 1.0)
        .with_label_color(paint::Color::rgb(0.90, 0.92, 0.96))
        .with_radius(rect::Radius::splat(0.22))
        .with_size(
            layout::Size::Fixed(menu_title_width(menu.label())),
            layout::Size::Fill,
        )
}

fn menu_title_width(label: &str) -> f32 {
    (label.chars().count() as f32 * 8.0 + 28.0).max(56.0)
}
