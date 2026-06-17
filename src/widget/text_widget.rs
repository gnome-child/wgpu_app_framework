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

pub fn text_field(id: ui::Id, field: impl Into<text::Field>) -> ui::Node {
    text_field_with_theme(id, field, &theme::Theme::default_dark())
}

pub fn text_field_with_theme(
    id: ui::Id,
    field: impl Into<text::Field>,
    theme: &theme::Theme,
) -> ui::Node {
    let field = field.into();
    let label = text_field_document(&field, theme);
    let mut interactivity = ui::Interactivity::NONE.with_hit_test(true);
    if field.is_selectable() {
        interactivity = interactivity.with_focusable(true);
    }

    let mut node = foundation::control_chrome(
        foundation::content_colors(
            ui::Node::leaf(id)
                .with_text_field(field.clone())
                .with_label(label)
                .with_interactivity(interactivity),
            theme,
        ),
        theme,
    );

    if field.is_editable() {
        node = node
            .with_responder(action::SELECT_ALL)
            .with_responder(action::COPY)
            .with_responder(action::CUT)
            .with_responder(action::PASTE)
            .with_responder(action::UNDO)
            .with_responder(action::REDO)
            .with_responder(action::INSERT_TEXT);
    } else if field.is_read_only() {
        node = node
            .with_responder(action::SELECT_ALL)
            .with_responder(action::COPY);
    }

    node
    .with_padding(layout::Insets {
        left: theme.density().app_padding(),
        top: 0.0,
        right: theme.density().app_padding(),
        bottom: 0.0,
    })
}

fn text_field_document(field: &text::Field, theme: &theme::Theme) -> text::Document {
    if field.buffer().is_empty()
        && let Some(placeholder) = field.placeholder()
    {
        return placeholder.clone().with_color(theme.text().disabled());
    }

    let color = if field.is_disabled() {
        theme.text().disabled()
    } else {
        theme.text().primary()
    };
    let mut block = text::Block::new(text::Align::Start);
    block.push_run(text::Run::new(
        field.presentation_text(),
        text::Style::default()
            .with_size(theme.text().control_size())
            .with_color(color),
    ));

    text::Document::from_block(block)
}
