use super::super::{context::Source, text, theme, view};

pub(crate) fn section_header_text(label: &str) -> String {
    label.chars().flat_map(char::to_uppercase).collect()
}

pub(crate) fn section_header_style(theme: &theme::Theme) -> theme::TypeStyle {
    let caption = theme.typography().caption();
    theme::TypeStyle::new(caption.size(), text::document::Weight::Bold)
}

pub(crate) fn shortcut_text_style(theme: &theme::Theme) -> theme::TypeStyle {
    interface_text_style(theme)
}

pub(crate) fn shortcut_run_gap(_theme: &theme::Theme) -> i32 {
    2
}

pub(crate) fn interface_text_style(theme: &theme::Theme) -> theme::TypeStyle {
    theme.typography().interface()
}

pub(crate) fn label_style(node: &view::Node, theme: &theme::Theme) -> theme::TypeStyle {
    if matches!(
        node.participation(),
        Some(view::Participation::Table(_) | view::Participation::AuxiliaryText)
    ) {
        return interface_text_style(theme);
    }

    label_style_for(
        node.role(),
        node.binding().map(view::Binding::source),
        theme,
    )
}

pub(crate) fn label_style_for(
    role: view::Role,
    source: Option<Source>,
    theme: &theme::Theme,
) -> theme::TypeStyle {
    match role {
        view::Role::Menu
        | view::Role::Binding
        | view::Role::Button
        | view::Role::Checkbox
        | view::Role::Radio
        | view::Role::Slider
        | view::Role::TextBox => interface_text_style(theme),
        view::Role::Label if source == Some(Source::Palette) => interface_text_style(theme),
        view::Role::SectionHeader => section_header_style(theme),
        _ => theme.typography().body(),
    }
}
