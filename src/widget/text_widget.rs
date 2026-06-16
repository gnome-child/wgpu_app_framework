use crate::{layout, text, theme, ui};

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
