use crate::{action, layout, text, theme, ui};

use super::foundation;

pub fn text(id: ui::Id, label: impl Into<text::Document>) -> ui::Node {
    text_with_theme(id, label, &theme::Theme::default_dark())
}

pub fn text_with_theme(
    id: ui::Id,
    label: impl Into<text::Document>,
    theme: &theme::Theme,
) -> ui::Node {
    foundation::content_colors(
        ui::Node::leaf(id)
            .with_label(label.into().with_color(theme.text().primary()))
            .with_size(
                layout::Size::Fit,
                layout::Size::Fixed(theme.density().label_height()),
            ),
        theme,
    )
}

pub fn paragraph(id: ui::Id, label: impl Into<text::Document>) -> ui::Node {
    paragraph_with_theme(id, label, &theme::Theme::default_dark())
}

pub fn paragraph_with_theme(
    id: ui::Id,
    label: impl Into<text::Document>,
    theme: &theme::Theme,
) -> ui::Node {
    foundation::content_colors(
        ui::Node::leaf(id)
            .with_label(label.into().with_color(theme.text().primary()))
            .with_size(layout::Size::Fill, layout::Size::Fit),
        theme,
    )
}

pub fn text_field(id: ui::Id, buffer: impl Into<text::Buffer>) -> ui::Node {
    text_field_with_theme(id, buffer, &theme::Theme::default_dark())
}

pub fn text_field_with_theme(
    id: ui::Id,
    buffer: impl Into<text::Buffer>,
    theme: &theme::Theme,
) -> ui::Node {
    let buffer = buffer.into();
    let mut block = text::Block::new(text::Align::Start);
    block.push_run(text::Run::new(
        buffer.text().to_owned(),
        text::Style::default()
            .with_size(theme.text().control_size())
            .with_color(theme.text().primary()),
    ));

    foundation::control_chrome(
        foundation::content_colors(
            ui::Node::leaf(id)
                .with_text_field(buffer.clone())
                .with_label(text::Document::from_block(block))
                .with_interactivity(
                    ui::Interactivity::NONE
                        .with_hit_test(true)
                        .with_focusable(true),
                )
                .with_responder_binding(
                    action::Binding::new(action::SELECT_ALL).enabled(!buffer.is_empty()),
                )
                .with_responder_binding(action::Binding::new(action::COPY).enabled(false))
                .with_responder_binding(action::Binding::new(action::CUT).enabled(false))
                .with_responder_binding(action::Binding::new(action::PASTE).enabled(false))
                .with_responder_binding(action::Binding::new(action::INSERT_TEXT).enabled(true)),
            theme,
        ),
        theme,
    )
    .with_padding(layout::Insets {
        left: theme.density().app_padding(),
        top: 0.0,
        right: theme.density().app_padding(),
        bottom: 0.0,
    })
}
