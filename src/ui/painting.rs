use crate::{action, layout, paint, ui, window};

pub fn tree<T>(
    root: &ui::Node,
    layout: &layout::Box,
    actions: &action::Registry<T>,
    window: window::Id,
    interaction: ui::Interaction,
    scene: &mut paint::Scene,
) {
    let mut overlays = Vec::new();

    node(
        root,
        layout,
        actions,
        window,
        &interaction,
        scene,
        &mut overlays,
    );

    for outline in overlays {
        scene.push_outline(outline);
    }
}

fn node<T>(
    node: &ui::Node,
    layout: &layout::Box,
    actions: &action::Registry<T>,
    window: window::Id,
    interaction: &ui::Interaction,
    scene: &mut paint::Scene,
    overlays: &mut Vec<paint::Outline>,
) {
    if let Some(style) = resolved_quad_style(node, layout, actions, window, interaction) {
        scene.push_quad(paint::Quad {
            rect: layout.rect(),
            style,
        });
    }

    if let Some(tint) = resolved_tint(node, layout, actions, window, interaction) {
        scene.push_tint(paint::Tint {
            rect: layout.rect(),
            color: tint,
        });
    }

    if let Some(document) = resolved_label(node, layout, actions, window) {
        scene.push_text(paint::Text {
            rect: layout.rect(),
            document,
        });
    }

    for (child, child_layout) in node.children().iter().zip(layout.children()) {
        self::node(
            child,
            child_layout,
            actions,
            window,
            interaction,
            scene,
            overlays,
        );
    }

    if let Some(outline) = resolved_focus_outline(node, layout, interaction) {
        overlays.push(outline);
    }
}

fn resolved_quad_style<T>(
    node: &ui::Node,
    layout: &layout::Box,
    actions: &action::Registry<T>,
    window: window::Id,
    interaction: &ui::Interaction,
) -> Option<paint::Style> {
    let style = node.style();
    let fill = resolved_background(node, layout, actions, window, interaction)
        .map(|color| paint::Fill::Brush(paint::Brush::Solid(color)));
    let stroke = style.stroke();

    if fill.is_none() && stroke.is_none() {
        return None;
    }

    Some(paint::Style {
        fill,
        stroke,
        tint: None,
    })
}

fn resolved_background<T>(
    node: &ui::Node,
    layout: &layout::Box,
    actions: &action::Registry<T>,
    window: window::Id,
    interaction: &ui::Interaction,
) -> Option<paint::Color> {
    let style = node.style();
    let background = style.background();

    if let Some(action) = node.action() {
        let state = actions.state(action, action::Context::path(window, layout.path().clone()));

        if state.is_busy() {
            return style.busy_background().or(background);
        }

        if !state.is_enabled() {
            return style.disabled_background().or_else(|| background.map(dim));
        }

        if state.is_active() {
            return style.active_background().or(background);
        }
    }

    if interaction.hovered() == Some(layout.path()) {
        return style.hover_background().or(background);
    }

    background
}

fn resolved_tint<T>(
    node: &ui::Node,
    layout: &layout::Box,
    actions: &action::Registry<T>,
    window: window::Id,
    interaction: &ui::Interaction,
) -> Option<paint::Color> {
    let style = node.style();
    let pressed = interaction.pressed() == Some(layout.path());
    let hovered = interaction.hovered() == Some(layout.path());

    if let Some(action) = node.action() {
        let state = actions.state(action, action::Context::path(window, layout.path().clone()));

        if state.is_busy() {
            return style.busy_tint();
        }

        if !state.is_enabled() {
            return style.disabled_tint();
        }

        if state.is_active() {
            return style.active_tint();
        }
    }

    if pressed {
        return style.pressed_tint();
    }

    if hovered {
        return style.hover_tint();
    }

    None
}

fn resolved_focus_outline(
    node: &ui::Node,
    layout: &layout::Box,
    interaction: &ui::Interaction,
) -> Option<paint::Outline> {
    if interaction.focused() != Some(layout.path()) {
        return None;
    }

    let outline = node.style().focus_outline()?;

    Some(paint::Outline {
        rect: layout.rect(),
        brush: outline.brush(),
        width: outline.width(),
        offset: outline.offset(),
    })
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
