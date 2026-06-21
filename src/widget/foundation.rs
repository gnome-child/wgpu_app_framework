use crate::{action, layout, paint, text, theme, ui};

pub fn document(
    label: impl Into<String>,
    align: text::document::Align,
    size: f32,
    color: paint::Color,
) -> text::document::Document {
    let mut block = text::document::Block::new(align);
    block.push_run(text::document::Run::new(
        label,
        text::document::Style::default()
            .with_size(size)
            .with_color(color),
    ));

    text::document::Document::from_block(block)
}

pub fn content_colors(node: ui::Node, theme: &theme::Theme) -> ui::Node {
    node.with_label_color(theme.text().primary())
        .with_busy_label_color(theme.text().busy())
        .with_disabled_label_color(theme.text().disabled())
}

pub fn panel_chrome(node: ui::Node, theme: &theme::Theme) -> ui::Node {
    let control = theme.control();
    let focus_outline = control.focus_outline();

    content_colors(node, theme)
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
        .with_rounding(theme.roundings().panel())
}

pub fn control_chrome(node: ui::Node, theme: &theme::Theme) -> ui::Node {
    node.with_background(theme.control().background())
        .with_stroke(theme.control().stroke())
        .with_rounding(theme.roundings().control())
        .with_size(
            layout::Size::Fill,
            layout::Size::Fixed(theme.density().control_height()),
        )
}

pub fn actionable(node: ui::Node, action: action::Id) -> ui::Node {
    node.with_action(action)
        .with_interactivity(ui::Interactivity::CONTROL)
}
