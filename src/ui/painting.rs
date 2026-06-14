use crate::{action, layout, paint, ui, window};

pub fn tree<T>(
    root: &ui::Node,
    layout: &layout::Box,
    actions: &action::Registry<T>,
    window: window::Id,
    interaction: &ui::Interaction,
    scene: &mut paint::Scene,
) {
    let mut overlays = Vec::new();

    node(
        root,
        layout,
        actions,
        window,
        interaction,
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
    let visual = visual_state(node, layout, actions, window, interaction);

    let rect = styled_rect(node, layout);

    if let Some(shadow) = resolved_shadow(node, rect) {
        scene.push_shadow(shadow);
    }

    if let Some(backdrop) = resolved_backdrop(node, rect) {
        scene.push_backdrop(backdrop);
    }

    if let Some(style) = resolved_quad_style(node, layout, interaction, &visual) {
        scene.push_quad(paint::Quad { rect, style });
    }

    for tint in resolved_tints(node, &visual) {
        scene.push_tint(paint::Tint { rect, color: tint });
    }

    if let Some(icon) = resolved_icon(node, layout, &visual) {
        scene.push_icon(icon);
    }

    if let Some(document) = resolved_label(node, &visual) {
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

    if let Some(outline) = resolved_focus_outline(node, rect, visual) {
        overlays.push(outline);
    }
}

fn styled_rect(node: &ui::Node, layout: &layout::Box) -> crate::geometry::Rect {
    let rect = layout.rect();

    crate::geometry::Rect::rounded(rect.origin, rect.area, node.style().radius())
}

fn resolved_quad_style(
    node: &ui::Node,
    layout: &layout::Box,
    interaction: &ui::Interaction,
    visual: &VisualState,
) -> Option<paint::Style> {
    let style = node.style();
    let fill = resolved_background(node, layout, interaction, visual)
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

fn resolved_background(
    node: &ui::Node,
    layout: &layout::Box,
    interaction: &ui::Interaction,
    visual: &VisualState,
) -> Option<paint::Color> {
    let style = node.style();
    let background = style.background();

    if visual.busy {
        return style.busy_background().or(background);
    }

    if !visual.enabled {
        return style.disabled_background().or_else(|| background.map(dim));
    }

    if visual.active {
        return style.active_background().or(background);
    }

    if interaction.hovered() == Some(layout.path()) {
        return style.hover_background().or(background);
    }

    background
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct VisualState {
    enabled: bool,
    busy: bool,
    active: bool,
    focus_visible: bool,
    position: StatePosition,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StatePosition {
    active: f32,
    hover: f32,
    press: f32,
}

fn visual_state<T>(
    node: &ui::Node,
    layout: &layout::Box,
    actions: &action::Registry<T>,
    window: window::Id,
    interaction: &ui::Interaction,
) -> VisualState {
    let action_state = node
        .action()
        .map(|action| actions.state(action, action_context(node, layout, window, interaction)))
        .unwrap_or_default();
    let enabled = action_state.is_enabled();
    let busy = action_state.is_busy();
    let interactive = enabled && !busy;
    let active = action_state.is_active();
    let hovered = interaction.hovered() == Some(layout.path());
    let pressed = interaction.pressed() == Some(layout.path());
    let focused = interaction.focused() == Some(layout.path());
    let focus_visible = focused && interaction.focus_visibility().is_visible();

    VisualState {
        enabled,
        busy,
        active,
        focus_visible,
        position: StatePosition {
            active: normalized(active),
            hover: normalized(interactive && hovered),
            press: normalized(interactive && pressed),
        },
    }
}

fn action_context(
    node: &ui::Node,
    layout: &layout::Box,
    window: window::Id,
    interaction: &ui::Interaction,
) -> action::Context {
    match node.action_target() {
        ui::ActionTarget::Origin => action::Context::path(window, layout.path().clone()),
        ui::ActionTarget::Command => interaction
            .command_target()
            .cloned()
            .unwrap_or_else(|| action::Context::window(window)),
        ui::ActionTarget::Captured => interaction
            .captured_command_target(layout.path())
            .cloned()
            .unwrap_or_else(|| action::Context::window(window)),
        ui::ActionTarget::Window => action::Context::window(window),
    }
}

fn resolved_tints(node: &ui::Node, visual: &VisualState) -> Vec<paint::Color> {
    let style = node.style();
    let mut tints = Vec::new();

    if visual.position.active > 0.0 {
        push_optional(&mut tints, style.active_tint());
    }

    if visual.busy {
        push_optional(&mut tints, style.busy_tint());
        return tints;
    }

    if !visual.enabled {
        push_optional(&mut tints, style.disabled_tint());
        return tints;
    }

    if visual.position.press > 0.0 {
        push_optional(&mut tints, style.pressed_tint());
    } else if visual.position.hover > 0.0 {
        push_optional(&mut tints, style.hover_tint());
    }

    tints
}

fn push_optional(values: &mut Vec<paint::Color>, value: Option<paint::Color>) {
    if let Some(value) = value {
        values.push(value);
    }
}

fn resolved_focus_outline(
    node: &ui::Node,
    rect: crate::geometry::Rect,
    visual: VisualState,
) -> Option<paint::Outline> {
    if !visual.focus_visible {
        return None;
    }

    let outline = node.style().focus_outline()?;

    Some(paint::Outline {
        rect,
        brush: outline.brush(),
        width: outline.width(),
        offset: outline.offset(),
    })
}

fn resolved_shadow(node: &ui::Node, rect: crate::geometry::Rect) -> Option<paint::Shadow> {
    let shadow = node.style().shadow()?;

    Some(paint::Shadow {
        rect,
        color: shadow.color(),
        blur: shadow.blur(),
        spread: shadow.spread(),
        offset: shadow.offset(),
    })
}

fn resolved_backdrop(node: &ui::Node, rect: crate::geometry::Rect) -> Option<paint::Backdrop> {
    Some(paint::Backdrop {
        rect,
        filter: node.style().backdrop_filter()?,
    })
}

fn resolved_label(node: &ui::Node, visual: &VisualState) -> Option<crate::text::Document> {
    let style = node.style();
    let mut document = node.label().cloned()?;

    if visual.busy {
        if let Some(color) = style.busy_label_color().or(style.label_color()) {
            document = document.with_color(color);
        }

        return Some(document);
    }

    if !visual.enabled {
        if let Some(color) = style.disabled_label_color().or(style.label_color()) {
            document = document.with_color(color);
        }

        return Some(document);
    }

    if let Some(color) = style.label_color() {
        document = document.with_color(color);
    }

    Some(document)
}

fn resolved_icon(
    node: &ui::Node,
    layout: &layout::Box,
    visual: &VisualState,
) -> Option<paint::Icon> {
    let icon = node.icon()?;

    Some(paint::Icon {
        rect: layout.rect(),
        icon,
        color: resolved_icon_color(node, visual),
        size: node.icon_size().unwrap_or(24.0),
    })
}

fn resolved_icon_color(node: &ui::Node, visual: &VisualState) -> paint::Color {
    let style = node.style();
    let fallback = crate::text::Style::default().color;

    if visual.busy {
        return style
            .busy_label_color()
            .or(style.label_color())
            .unwrap_or(fallback);
    }

    if !visual.enabled {
        return style
            .disabled_label_color()
            .or(style.label_color())
            .unwrap_or(fallback);
    }

    style.label_color().unwrap_or(fallback)
}

fn normalized(value: bool) -> f32 {
    if value { 1.0 } else { 0.0 }
}

fn dim(color: paint::Color) -> paint::Color {
    paint::Color {
        r: color.r * 0.45,
        g: color.g * 0.45,
        b: color.b * 0.45,
        a: color.a,
    }
}
