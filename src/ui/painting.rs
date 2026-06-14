use crate::{action, layout, paint, ui, window};

pub(super) fn tree(
    root: &ui::Node,
    layout: &layout::Box,
    actions: &action::Registry,
    window: window::Id,
    interaction: ui::Interaction,
    scene: &mut paint::Scene,
) {
    node(root, layout, actions, window, interaction, scene);
}

fn node(
    node: &ui::Node,
    layout: &layout::Box,
    actions: &action::Registry,
    window: window::Id,
    interaction: ui::Interaction,
    scene: &mut paint::Scene,
) {
    if let Some(background) = resolved_background(node, actions, window, interaction) {
        scene.push_quad(paint::Quad {
            rect: layout.rect,
            style: paint::Style {
                fill: Some(paint::Fill::Brush(paint::Brush::Solid(background))),
                stroke: None,
                tint: None,
            },
        });
    }

    if let Some(document) = resolved_label(node, actions, window) {
        scene.push_text(paint::Text {
            rect: layout.rect,
            document,
        });
    }

    for (child, child_layout) in node.children.iter().zip(&layout.children) {
        self::node(child, child_layout, actions, window, interaction, scene);
    }
}

fn resolved_background(
    node: &ui::Node,
    actions: &action::Registry,
    window: window::Id,
    interaction: ui::Interaction,
) -> Option<paint::Color> {
    let background = node.style.background?;

    if let Some(action) = node.action {
        let state = actions.state(
            action,
            action::Context {
                window,
                target: Some(node.id),
            },
        );

        if !state.enabled {
            return Some(
                node.style
                    .disabled_background
                    .unwrap_or_else(|| dim(background)),
            );
        }

        if state.active {
            return Some(node.style.active_background.unwrap_or(background));
        }
    }

    if interaction.focused == Some(node.id) {
        return Some(node.style.focus_background.unwrap_or(background));
    }

    if interaction.hovered == Some(node.id) {
        return Some(node.style.hover_background.unwrap_or(background));
    }

    Some(background)
}

fn resolved_label(
    node: &ui::Node,
    actions: &action::Registry,
    window: window::Id,
) -> Option<crate::text::Document> {
    let mut document = node.label.clone()?;

    if let Some(action) = node.action {
        let state = actions.state(
            action,
            action::Context {
                window,
                target: Some(node.id),
            },
        );

        if !state.enabled {
            if let Some(color) = node.style.disabled_label_color.or(node.style.label_color) {
                document = document.with_color(color);
            }

            return Some(document);
        }
    }

    if let Some(color) = node.style.label_color {
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
