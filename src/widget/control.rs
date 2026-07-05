use crate::command;
use crate::ui::{self, layout};
use crate::{Command, icon, text, theme};

use super::foundation;

pub fn panel() -> ui::Node {
    panel_with_theme(&theme::Theme::default_dark())
}

pub fn panel_with_theme(theme: &theme::Theme) -> ui::Node {
    foundation::panel_chrome(
        ui::Node::container(layout::Axis::Vertical).with_kind("panel"),
        theme,
    )
}

pub fn floating_panel() -> ui::Node {
    floating_panel_with_theme(&theme::Theme::default_dark())
}

pub fn floating_panel_with_theme(theme: &theme::Theme) -> ui::Node {
    let floating = theme.floating_panel();
    let shadow = floating.shadow();

    foundation::content_colors(
        ui::Node::container(layout::Axis::Vertical).with_kind("floating_panel"),
        theme,
    )
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
    .padding(floating.padding())
    .with_interactivity(ui::Interactivity::NONE.with_hit_test(true))
}

pub fn button<C, TTarget>() -> ui::Node
where
    C: Command,
    TTarget: command::Target<C> + 'static,
{
    button_with_theme::<C, TTarget>(&theme::Theme::default_dark())
}

#[cfg(test)]
pub(crate) fn button_key(id: ui::Id, command: command::Key) -> ui::Node {
    button_with_theme_key(command, &theme::Theme::default_dark()).key(id)
}

pub fn button_with_theme<C, TTarget>(theme: &theme::Theme) -> ui::Node
where
    C: Command,
    TTarget: command::Target<C> + 'static,
{
    foundation::control_chrome(
        foundation::actionable::<C, TTarget>(panel_with_theme(theme).with_kind("button")),
        theme,
    )
}

#[cfg(test)]
pub(crate) fn button_with_theme_key(command: command::Key, theme: &theme::Theme) -> ui::Node {
    foundation::control_chrome(
        foundation::actionable_key(panel_with_theme(theme).with_kind("button"), command),
        theme,
    )
}

pub fn labeled_button<C, TTarget>(label: impl Into<String>) -> ui::Node
where
    C: Command,
    TTarget: command::Target<C> + 'static,
{
    labeled_button_with_theme::<C, TTarget>(label, &theme::Theme::default_dark())
}

#[cfg(test)]
pub(crate) fn labeled_button_key(
    id: ui::Id,
    command: command::Key,
    label: impl Into<String>,
) -> ui::Node {
    labeled_button_with_theme_key(command, label, &theme::Theme::default_dark()).key(id)
}

pub fn labeled_button_with_theme<C, TTarget>(
    label: impl Into<String>,
    theme: &theme::Theme,
) -> ui::Node
where
    C: Command,
    TTarget: command::Target<C> + 'static,
{
    let mut block = text::document::Block::new(text::document::Align::Center);
    block.push_run(text::document::Run::new(
        label,
        theme
            .text()
            .style(text::document::Role::Control)
            .with_color(theme.text().primary()),
    ));

    button_with_theme::<C, TTarget>(theme).with_label(text::document::Document::from_block(block))
}

#[cfg(test)]
pub(crate) fn labeled_button_with_theme_key(
    command: command::Key,
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

    button_with_theme_key(command, theme).with_label(text::document::Document::from_block(block))
}

pub fn icon_button<C, TTarget>(icon: icon::Icon) -> ui::Node
where
    C: Command,
    TTarget: command::Target<C> + 'static,
{
    icon_button_with_theme::<C, TTarget>(icon, &theme::Theme::default_dark())
}

#[cfg(test)]
pub(crate) fn icon_button_key(id: ui::Id, command: command::Key, icon: icon::Icon) -> ui::Node {
    icon_button_with_theme_key(command, icon, &theme::Theme::default_dark()).key(id)
}

pub fn icon_button_with_theme<C, TTarget>(icon: icon::Icon, theme: &theme::Theme) -> ui::Node
where
    C: Command,
    TTarget: command::Target<C> + 'static,
{
    button_with_theme::<C, TTarget>(theme)
        .with_icon(icon)
        .with_icon_size(theme.text().icon_size())
        .size(
            layout::Size::fill(),
            layout::Size::fixed(theme.density().icon_button_height()),
        )
}

#[cfg(test)]
pub(crate) fn icon_button_with_theme_key(
    command: command::Key,
    icon: icon::Icon,
    theme: &theme::Theme,
) -> ui::Node {
    button_with_theme_key(command, theme)
        .with_icon(icon)
        .with_icon_size(theme.text().icon_size())
        .size(
            layout::Size::fill(),
            layout::Size::fixed(theme.density().icon_button_height()),
        )
}
