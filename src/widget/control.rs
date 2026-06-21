use crate::{action, icon, layout, text, theme, ui};

use super::foundation;

pub fn panel(id: ui::Id) -> ui::Node {
    panel_with_theme(id, &theme::Theme::default_dark())
}

pub fn panel_with_theme(id: ui::Id, theme: &theme::Theme) -> ui::Node {
    foundation::panel_chrome(ui::Node::container(id, layout::Axis::Vertical), theme)
}

pub fn floating_panel(id: ui::Id) -> ui::Node {
    floating_panel_with_theme(id, &theme::Theme::default_dark())
}

pub fn floating_panel_with_theme(id: ui::Id, theme: &theme::Theme) -> ui::Node {
    let floating = theme.floating_panel();
    let shadow = floating.shadow();

    foundation::content_colors(ui::Node::container(id, layout::Axis::Vertical), theme)
        .with_backdrop(
            ui::Backdrop::glass(floating.backdrop_fill()).with_blur(floating.backdrop_blur()),
        )
        .with_stroke(floating.stroke())
        .with_shadow(
            shadow.brush(),
            shadow.blur(),
            shadow.spread(),
            shadow.offset(),
        )
        .with_rounding(floating.rounding())
        .with_padding(layout::Insets::splat(floating.padding()))
        .with_interactivity(ui::Interactivity::NONE.with_hit_test(true))
}

pub fn button(id: ui::Id, action: action::Id) -> ui::Node {
    button_with_theme(id, action, &theme::Theme::default_dark())
}

pub fn button_with_theme(id: ui::Id, action: action::Id, theme: &theme::Theme) -> ui::Node {
    foundation::control_chrome(
        foundation::actionable(panel_with_theme(id, theme), action),
        theme,
    )
}

pub fn labeled_button(id: ui::Id, action: action::Id, label: impl Into<String>) -> ui::Node {
    labeled_button_with_theme(id, action, label, &theme::Theme::default_dark())
}

pub fn labeled_button_with_theme(
    id: ui::Id,
    action: action::Id,
    label: impl Into<String>,
    theme: &theme::Theme,
) -> ui::Node {
    let mut block = text::document::Block::new(text::document::Align::Center);
    block.push_run(text::document::Run::new(
        label,
        theme
            .text()
            .style(text::document::Role::Control)
            .with_color(theme.text().primary()),
    ));

    button_with_theme(id, action, theme).with_label(text::document::Document::from_block(block))
}

pub fn icon_button(id: ui::Id, action: action::Id, icon: icon::Icon) -> ui::Node {
    icon_button_with_theme(id, action, icon, &theme::Theme::default_dark())
}

pub fn icon_button_with_theme(
    id: ui::Id,
    action: action::Id,
    icon: icon::Icon,
    theme: &theme::Theme,
) -> ui::Node {
    button_with_theme(id, action, theme)
        .with_icon(icon)
        .with_icon_size(theme.text().icon_size())
        .with_size(
            layout::Size::Fill,
            layout::Size::Fixed(theme.density().icon_button_height()),
        )
}
