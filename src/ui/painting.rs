use crate::{action, layout, paint, ui, window};

pub fn tree<T>(
    root: &ui::Node,
    layout: &layout::Box,
    actions: &action::Registry<T>,
    window: window::Id,
    interaction: ui::Interaction,
    scene: &mut paint::Scene,
) {
    node(root, layout, actions, window, &interaction, scene);
}

fn node<T>(
    node: &ui::Node,
    layout: &layout::Box,
    actions: &action::Registry<T>,
    window: window::Id,
    interaction: &ui::Interaction,
    scene: &mut paint::Scene,
) {
    if let Some(background) = resolved_background(node, layout, actions, window, interaction) {
        scene.push_quad(paint::Quad {
            rect: layout.rect(),
            style: paint::Style {
                fill: Some(paint::Fill::Brush(paint::Brush::Solid(background))),
                stroke: None,
                tint: None,
            },
        });
    }

    if let Some(document) = resolved_label(node, layout, actions, window) {
        scene.push_text(paint::Text {
            rect: layout.rect(),
            document,
        });
    }

    for (child, child_layout) in node.children().iter().zip(layout.children()) {
        self::node(child, child_layout, actions, window, interaction, scene);
    }
}

fn resolved_background<T>(
    node: &ui::Node,
    layout: &layout::Box,
    actions: &action::Registry<T>,
    window: window::Id,
    interaction: &ui::Interaction,
) -> Option<paint::Color> {
    let style = node.style();
    let background = style.background()?;

    if let Some(action) = node.action() {
        let state = actions.state(action, action::Context::path(window, layout.path().clone()));

        if state.is_busy() {
            return Some(style.busy_background().unwrap_or(background));
        }

        if !state.is_enabled() {
            return Some(
                style
                    .disabled_background()
                    .unwrap_or_else(|| dim(background)),
            );
        }

        if state.is_active() {
            return Some(style.active_background().unwrap_or(background));
        }
    }

    if interaction.focused() == Some(layout.path()) {
        return Some(style.focus_background().unwrap_or(background));
    }

    if interaction.hovered() == Some(layout.path()) {
        return Some(style.hover_background().unwrap_or(background));
    }

    Some(background)
}

fn resolved_label<T>(
    node: &ui::Node,
    layout: &layout::Box,
    actions: &action::Registry<T>,
    window: window::Id,
) -> Option<crate::text::Document> {
    let style = node.style();
    let mut document = node.label().cloned()?;

    if let Some(action) = node.action() {
        let state = actions.state(action, action::Context::path(window, layout.path().clone()));

        if state.is_busy() {
            if let Some(color) = style.busy_label_color().or(style.label_color()) {
                document = document.with_color(color);
            }

            return Some(document);
        }

        if !state.is_enabled() {
            if let Some(color) = style.disabled_label_color().or(style.label_color()) {
                document = document.with_color(color);
            }

            return Some(document);
        }
    }

    if let Some(color) = style.label_color() {
        document = document.with_color(color);
    }

    Some(document)
}

fn dim(color: paint::Color) -> paint::Color {
    paint::Color {
        r: color.r * 0.45,
        g: color.g * 0.45,
        b: color.b * 0.45,
        a: color.a,
    }
}
