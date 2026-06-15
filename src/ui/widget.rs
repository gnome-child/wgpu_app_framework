use crate::geometry::point;
use crate::{layout, menu, text, theme, ui};

pub const MENU_POPUP: ui::Id = ui::Id::new("__menu_popup");
pub const MENU_SUBMENU_POPUP: ui::Id = ui::Id::new("__menu_submenu_popup");

pub fn label(id: ui::Id, label: impl Into<String>) -> ui::Node {
    label_with_theme(id, label, &theme::Theme::default_dark())
}

pub fn label_with_theme(id: ui::Id, label: impl Into<String>, theme: &theme::Theme) -> ui::Node {
    ui::Node::leaf(id)
        .with_label(document_with_theme(label, theme, text::Align::Center))
        .with_label_color(theme.text().secondary())
        .with_size(
            layout::Size::Fill,
            layout::Size::Fixed(theme.density().label_height()),
        )
}

pub fn separator(id: ui::Id) -> ui::Node {
    separator_with_theme(id, &theme::Theme::default_dark())
}

pub fn separator_with_theme(id: ui::Id, theme: &theme::Theme) -> ui::Node {
    ui::Node::leaf(id)
        .with_background(theme.surfaces().separator())
        .with_size(layout::Size::Fill, layout::Size::Fixed(1.0))
}

pub fn scroll_view(id: ui::Id) -> ui::Node {
    ui::Node::container(id, layout::Axis::Vertical)
        .clipped()
        .with_scroll_offset(point::logical(0.0, 0.0))
        .with_size(layout::Size::Fill, layout::Size::Fill)
}

pub fn menu_bar(id: ui::Id, bar: menu::Bar) -> ui::Node {
    menu_bar_with_theme(id, bar, &theme::Theme::default_dark())
}

pub fn menu_bar_with_theme(id: ui::Id, bar: menu::Bar, theme: &theme::Theme) -> ui::Node {
    let mut node = ui::Node::container(id, layout::Axis::Horizontal)
        .with_menu_bar(bar.clone())
        .with_background(theme.menu().bar_background())
        .with_stroke(theme.menu().bar_stroke())
        .with_size(
            layout::Size::Fill,
            layout::Size::Fixed(theme.density().menu_bar_height()),
        );

    for menu in bar.menus() {
        node.push_child(menu_title(menu, theme));
    }

    node
}

pub fn document(label: impl Into<String>) -> text::Document {
    let mut block = text::Block::new(text::Align::Center);
    block.push_run(text::Run::new(label, text::Style::default()));

    text::Document::from_block(block)
}

fn document_with_theme(
    label: impl Into<String>,
    theme: &theme::Theme,
    align: text::Align,
) -> text::Document {
    let mut block = text::Block::new(align);
    block.push_run(text::Run::new(
        label,
        text::Style::default()
            .with_size(theme.text().menu_size())
            .with_color(theme.text().primary()),
    ));

    text::Document::from_block(block)
}

fn menu_title(menu: &menu::Menu, theme: &theme::Theme) -> ui::Node {
    let outline = theme.menu().title_focus_outline();

    ui::Node::leaf(ui::Id::new(menu.id().as_str()))
        .with_intent(ui::Intent::OpenMenu(menu.id()))
        .with_interactivity(ui::Interactivity::CONTROL)
        .with_label(document_with_theme(
            menu.label(),
            theme,
            text::Align::Center,
        ))
        .with_background(theme.menu().title_background())
        .with_hover_tint(theme.menu().title_hover_tint())
        .with_pressed_tint(theme.menu().title_pressed_tint())
        .with_active_tint(theme.menu().title_active_tint())
        .with_focus_outline(outline.brush(), outline.width(), outline.offset())
        .with_label_color(theme.text().primary())
        .with_radius(theme.radii().menu_title())
        .with_size(
            layout::Size::Fixed(menu_title_width(menu.label(), theme)),
            layout::Size::Fill,
        )
}

fn menu_title_width(label: &str, theme: &theme::Theme) -> f32 {
    let density = theme.density();

    (label.chars().count() as f32 * density.menu_title_char_width()
        + density.menu_title_horizontal_padding())
    .max(density.menu_title_min_width())
}
