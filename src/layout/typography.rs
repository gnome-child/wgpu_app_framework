use super::super::{text, theme};

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
