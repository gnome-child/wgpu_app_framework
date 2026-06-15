use crate::{action, layout, paint, ui, widget, window};

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

    for brush in resolved_tints(node, &visual) {
        scene.push_tint(paint::Tint { rect, brush });
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

    let clip_rect = if node.clips() {
        Some(widget::scroll::metrics(node, layout).map_or(rect, |metrics| metrics.viewport()))
    } else {
        None
    };

    if let Some(clip_rect) = clip_rect {
        scene.push_clip(paint::Clip { rect: clip_rect });
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

    if node.clips() {
        scene.pop_clip();
    }

    paint_scrollbars(node, layout, interaction, scene);

    if let Some(outline) = resolved_focus_outline(node, rect, visual) {
        overlays.push(outline);
    }
}

fn paint_scrollbars(
    node: &ui::Node,
    layout: &layout::Box,
    interaction: &ui::Interaction,
    scene: &mut paint::Scene,
) {
    let Some(metrics) = widget::scroll::metrics(node, layout) else {
        return;
    };

    let style = metrics.style();

    if let Some(track) = metrics.vertical_track() {
        push_scroll_chrome(scene, track, style.track());
    }

    if let Some(track) = metrics.horizontal_track() {
        push_scroll_chrome(scene, track, style.track());
    }

    if let Some(corner) = metrics.corner() {
        push_scroll_chrome(scene, corner, style.corner());
    }

    if let Some(thumb) = metrics.vertical_thumb() {
        push_scroll_chrome(scene, thumb, style.thumb());
        push_scroll_thumb_tint(
            scene,
            metrics,
            thumb,
            widget::scroll::Part::VerticalThumb,
            interaction.pressed() == Some(layout.path()),
            interaction,
        );
    }

    if let Some(thumb) = metrics.horizontal_thumb() {
        push_scroll_chrome(scene, thumb, style.thumb());
        push_scroll_thumb_tint(
            scene,
            metrics,
            thumb,
            widget::scroll::Part::HorizontalThumb,
            interaction.pressed() == Some(layout.path()),
            interaction,
        );
    }
}

fn push_scroll_chrome(scene: &mut paint::Scene, rect: crate::geometry::Rect, brush: paint::Brush) {
    scene.push_quad(paint::Quad {
        rect,
        style: paint::Style {
            fill: Some(paint::Fill::Brush(brush)),
            stroke: None,
            tint: None,
        },
    });
}

fn push_scroll_thumb_tint(
    scene: &mut paint::Scene,
    metrics: widget::scroll::Metrics,
    thumb: crate::geometry::Rect,
    part: widget::scroll::Part,
    pressed: bool,
    interaction: &ui::Interaction,
) {
    let Some(position) = interaction.pointer_position() else {
        return;
    };
    if metrics.hit_test(position) != Some(part) {
        return;
    }

    let style = metrics.style();
    let brush = if pressed {
        style.thumb_pressed_tint()
    } else {
        style.thumb_hover_tint()
    };

    scene.push_tint(paint::Tint { rect: thumb, brush });
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
    let fill = resolved_background(node, layout, interaction, visual).map(paint::Fill::Brush);
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
) -> Option<paint::Brush> {
    let style = node.style();
    let background = style
        .backdrop()
        .and_then(|backdrop| backdrop.fill())
        .or(style.background());

    if visual.busy {
        return style.busy_background().or(background);
    }

    if !visual.enabled {
        return style
            .disabled_background()
            .or_else(|| background.map(|brush| brush.dimmed(0.45)));
    }

    if visual.focus_visible {
        return style.focus_background().or(background);
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
    let active = action_state.is_active() || menu_intent_active(node.intent(), interaction);
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

fn menu_intent_active(intent: Option<ui::Intent>, interaction: &ui::Interaction) -> bool {
    match intent {
        Some(ui::Intent::OpenMenu(menu)) => interaction.open_menu() == Some(menu),
        Some(ui::Intent::OpenSubmenu(menu)) => interaction.open_submenu() == Some(menu),
        Some(ui::Intent::Action(_) | ui::Intent::CloseSubmenu) | None => false,
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

fn resolved_tints(node: &ui::Node, visual: &VisualState) -> Vec<paint::Brush> {
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

fn push_optional(values: &mut Vec<paint::Brush>, value: Option<paint::Brush>) {
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
        brush: shadow.brush(),
        blur: shadow.blur(),
        spread: shadow.spread(),
        offset: shadow.offset(),
    })
}

fn resolved_backdrop(node: &ui::Node, rect: crate::geometry::Rect) -> Option<paint::Backdrop> {
    let amount = node.style().backdrop()?.blur()?;

    Some(paint::Backdrop {
        rect,
        filter: paint::BackdropFilter::Blur { amount },
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
