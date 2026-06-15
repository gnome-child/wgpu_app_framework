use crate::{action, icon, layout, paint, text, theme, ui};

pub fn panel(id: ui::Id) -> ui::Node {
    panel_with_theme(id, &theme::Theme::default_dark())
}

pub fn panel_with_theme(id: ui::Id, theme: &theme::Theme) -> ui::Node {
    let control = theme.control();
    let focus_outline = control.focus_outline();

    ui::Node::container(id, layout::Axis::Vertical)
        .with_background(theme.surfaces().panel())
        .with_stroke(paint::Stroke {
            brush: theme.surfaces().panel_stroke(),
            width: 1.0,
        })
        .with_hover_tint(control.hover_tint())
        .with_pressed_tint(control.pressed_tint())
        .with_active_tint(control.active_tint())
        .with_busy_tint(control.busy_tint())
        .with_disabled_tint(control.disabled_tint())
        .with_focus_outline(
            focus_outline.brush(),
            focus_outline.width(),
            focus_outline.offset(),
        )
        .with_label_color(theme.text().primary())
        .with_busy_label_color(theme.text().busy())
        .with_disabled_label_color(theme.text().disabled())
        .with_radius(theme.radii().panel())
}

pub fn floating_panel(id: ui::Id) -> ui::Node {
    floating_panel_with_theme(id, &theme::Theme::default_dark())
}

pub fn floating_panel_with_theme(id: ui::Id, theme: &theme::Theme) -> ui::Node {
    let floating = theme.floating_panel();
    let shadow = floating.shadow();

    ui::Node::container(id, layout::Axis::Vertical)
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
        .with_radius(floating.radius())
        .with_padding(layout::Insets::splat(floating.padding()))
        .with_label_color(theme.text().primary())
        .with_busy_label_color(theme.text().busy())
        .with_disabled_label_color(theme.text().disabled())
        .with_interactivity(ui::Interactivity::NONE.with_hit_test(true))
}

pub fn button(id: ui::Id, action: action::Id) -> ui::Node {
    button_with_theme(id, action, &theme::Theme::default_dark())
}

pub fn button_with_theme(id: ui::Id, action: action::Id, theme: &theme::Theme) -> ui::Node {
    panel_with_theme(id, theme)
        .with_action(action)
        .with_interactivity(ui::Interactivity::CONTROL)
        .with_background(theme.control().background())
        .with_stroke(theme.control().stroke())
        .with_radius(theme.radii().control())
        .with_size(
            layout::Size::Fill,
            layout::Size::Fixed(theme.density().control_height()),
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
    let mut block = text::Block::new(text::Align::Center);
    block.push_run(text::Run::new(
        label,
        text::Style::default()
            .with_size(theme.text().control_size())
            .with_color(theme.text().primary()),
    ));

    button_with_theme(id, action, theme).with_label(text::Document::from_block(block))
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
