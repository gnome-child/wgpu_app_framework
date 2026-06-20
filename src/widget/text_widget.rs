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
    let style = theme.text().style(text::Role::Label);
    foundation::content_colors(
        ui::Node::leaf(id)
            .with_label(
                label
                    .into()
                    .with_size(style.size())
                    .with_color(style.color()),
            )
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
    let style = theme.text().style(text::Role::Body);
    foundation::content_colors(
        ui::Node::leaf(id)
            .with_label(
                label
                    .into()
                    .with_size(style.size())
                    .with_color(style.color()),
            )
            .with_size(layout::Size::Fill, layout::Size::Fit),
        theme,
    )
}

pub fn text_field(id: ui::Id, field: impl Into<text::Field>) -> ui::Node {
    text_field_with_theme(id, field, &theme::Theme::default_dark())
}

pub fn text_area(id: ui::Id, area: impl Into<text::Area>) -> ui::Node {
    text_area_with_theme(id, area, &theme::Theme::default_dark())
}

pub fn text_field_with_theme(
    id: ui::Id,
    field: impl Into<text::Field>,
    theme: &theme::Theme,
) -> ui::Node {
    let field = field.into();
    let label = text_field_document(&field, theme);
    let outline = theme.control().focus_outline();
    let mut interactivity = ui::Interactivity::NONE.with_hit_test(true);
    let mut cursor = ui::Cursor::Default;
    if field.is_selectable() {
        interactivity = interactivity.with_focusable(true);
        cursor = ui::Cursor::Text;
    }

    let node = foundation::control_chrome(
        foundation::content_colors(
            ui::Node::leaf(id)
                .with_text_field(field.clone())
                .with_label(label)
                .with_interactivity(interactivity)
                .with_cursor(cursor),
            theme,
        ),
        theme,
    )
    .with_focus_outline(outline.brush(), outline.width(), outline.offset());

    bind_text_surface_responders(
        node.with_padding(layout::Insets {
            left: theme.density().app_padding(),
            top: 0.0,
            right: theme.density().app_padding(),
            bottom: 0.0,
        }),
        &text::Surface::Field(field),
    )
}

pub fn text_area_with_theme(
    id: ui::Id,
    area: impl Into<text::Area>,
    theme: &theme::Theme,
) -> ui::Node {
    let area = area.into();
    let label = text_area_document(&area, theme);
    let outline = theme.control().focus_outline();
    let mut interactivity = ui::Interactivity::NONE.with_hit_test(true);
    let mut cursor = ui::Cursor::Default;
    if area.is_selectable() {
        interactivity = interactivity.with_focusable(true);
        cursor = ui::Cursor::Text;
    }

    let surface = text::Surface::Area(area.clone());
    let scroll = theme.scroll();
    let text_scroll = super::Scroll::new()
        .with_bars(super::scroll::Bars::both())
        .with_style(super::scroll::Style::new(
            scroll.thickness(),
            scroll.min_thumb_length(),
            scroll.track(),
            scroll.thumb(),
            scroll.thumb_hover_tint(),
            scroll.thumb_pressed_tint(),
            scroll.corner(),
        ));
    let node = foundation::control_chrome(
        foundation::content_colors(
            ui::Node::leaf(id)
                .with_text_area(area)
                .with_text_scroll(text_scroll)
                .with_label(label)
                .with_interactivity(interactivity)
                .with_cursor(cursor),
            theme,
        ),
        theme,
    )
    .with_focus_outline(outline.brush(), outline.width(), outline.offset())
    .with_size(
        layout::Size::Fill,
        layout::Size::Fixed(theme.density().control_height() * 6.0),
    )
    .with_padding(layout::Insets {
        left: theme.density().app_padding(),
        top: theme.density().app_padding(),
        right: theme.density().app_padding(),
        bottom: theme.density().app_padding(),
    });

    bind_text_surface_responders(node, &surface)
}

fn text_field_document(field: &text::Field, theme: &theme::Theme) -> text::Document {
    if field.buffer().is_empty()
        && let Some(placeholder) = field.placeholder()
    {
        return theme.text().document(
            text::Role::Placeholder,
            placeholder,
            theme.text().disabled(),
        );
    }

    let color = if field.is_disabled() {
        theme.text().disabled()
    } else {
        theme.text().primary()
    };
    theme
        .text()
        .document(text::Role::Control, field.presentation_text(), color)
}

fn text_area_document(area: &text::Area, theme: &theme::Theme) -> text::Document {
    if area.buffer().is_empty()
        && let Some(placeholder) = area.placeholder()
    {
        return theme.text().document(
            text::Role::Placeholder,
            placeholder,
            theme.text().disabled(),
        );
    }

    let color = if area.is_disabled() {
        theme.text().disabled()
    } else {
        theme.text().primary()
    };
    theme.text().document(text::Role::Control, "", color)
}

fn bind_text_surface_responders(mut node: ui::Node, surface: &text::Surface) -> ui::Node {
    if surface.is_editable() {
        node = node
            .with_responder(action::SELECT_ALL)
            .with_responder(action::COPY)
            .with_responder(action::CUT)
            .with_responder(action::PASTE)
            .with_responder(action::UNDO)
            .with_responder(action::REDO)
            .with_responder(action::INSERT_TEXT);
    } else if surface.is_read_only() {
        node = node
            .with_responder(action::SELECT_ALL)
            .with_responder(action::COPY);
    }

    node
}
