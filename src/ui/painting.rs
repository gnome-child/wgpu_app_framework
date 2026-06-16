use crate::{action, geometry, paint, text, ui, widget, window};

pub fn tree<T>(
    root: &ui::Node,
    layout: &ui::Frame,
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
    layout: &ui::Frame,
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

    paint_text_field_selection(node, layout, scene);

    if let Some(document) = resolved_label(node, &visual) {
        scene.push_text(paint::Text {
            rect: layout.rect(),
            document,
        });
    }

    paint_text_field_caret(node, layout, &visual, scene);

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

    widget::scroll::paint_chrome(node, layout, interaction, scene);

    if let Some(outline) = resolved_focus_outline(node, rect, visual) {
        overlays.push(outline);
    }
}

fn styled_rect(node: &ui::Node, layout: &ui::Frame) -> crate::geometry::Rect {
    let rect = layout.rect();

    crate::geometry::Rect::rounded(rect.origin, rect.area, node.style().rounding())
}

fn resolved_quad_style(
    node: &ui::Node,
    layout: &ui::Frame,
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
    layout: &ui::Frame,
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
    focused: bool,
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
    layout: &ui::Frame,
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
        focused,
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
    layout: &ui::Frame,
    window: window::Id,
    interaction: &ui::Interaction,
) -> action::Context {
    match node.command_subject() {
        ui::CommandSubject::Origin => action::Context::path(window, layout.path().clone()),
        ui::CommandSubject::Current => interaction
            .command_subject()
            .cloned()
            .unwrap_or_else(|| action::Context::window(window)),
        ui::CommandSubject::Captured => interaction
            .captured_command_subject(layout.path())
            .cloned()
            .unwrap_or_else(|| action::Context::window(window)),
        ui::CommandSubject::Window => action::Context::window(window),
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

fn resolved_icon(node: &ui::Node, layout: &ui::Frame, visual: &VisualState) -> Option<paint::Icon> {
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
    let fallback = crate::text::Style::default().color();

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

fn paint_text_field_selection(node: &ui::Node, layout: &ui::Frame, scene: &mut paint::Scene) {
    let Some(buffer) = node.text_field() else {
        return;
    };
    let Some(range) = buffer.selected_range() else {
        return;
    };

    let rect = text_content_rect(node, layout);
    let start = text_x(buffer.text(), range.start, rect);
    let end = text_x(buffer.text(), range.end, rect).max(start + 1.0);

    scene.push_quad(paint::Quad {
        rect: geometry::Rect::rounded(
            geometry::point::logical(start, rect.origin.y()),
            geometry::area::logical(end - start, rect.area.height()),
            geometry::rect::Rounding::fixed(3.0),
        ),
        style: paint::Style {
            fill: Some(paint::Fill::Brush(
                paint::Color::rgba(0.18, 0.42, 0.86, 0.48).into(),
            )),
            stroke: None,
            tint: None,
        },
    });
}

fn paint_text_field_caret(
    node: &ui::Node,
    layout: &ui::Frame,
    visual: &VisualState,
    scene: &mut paint::Scene,
) {
    let Some(buffer) = node.text_field() else {
        return;
    };

    if !visual.focused {
        return;
    }

    let rect = text_content_rect(node, layout);
    let x = text_x(buffer.text(), buffer.cursor(), rect);

    scene.push_quad(paint::Quad {
        rect: geometry::Rect::new(
            geometry::point::logical(x, rect.origin.y() + 5.0),
            geometry::area::logical(1.0, (rect.area.height() - 10.0).max(6.0)),
        ),
        style: paint::Style {
            fill: Some(paint::Fill::Brush(
                node.style()
                    .label_color()
                    .unwrap_or_else(|| text::Style::default().color())
                    .into(),
            )),
            stroke: None,
            tint: None,
        },
    });
}

fn text_content_rect(node: &ui::Node, layout: &ui::Frame) -> geometry::Rect {
    let rect = layout.rect();
    let padding = node.style().padding();
    let x = rect.origin.x() + padding.left;
    let y = rect.origin.y();
    let width = (rect.area.width() - padding.left - padding.right).max(0.0);

    geometry::Rect::new(
        geometry::point::logical(x, y),
        geometry::area::logical(width, rect.area.height()),
    )
}

fn text_x(text: &str, cursor: usize, rect: geometry::Rect) -> f32 {
    let cursor_chars = text[..cursor.min(text.len())].chars().count() as f32;
    let total_chars = text.chars().count().max(1) as f32;
    rect.origin.x() + rect.area.width() * (cursor_chars / total_chars)
}

fn normalized(value: bool) -> f32 {
    if value { 1.0 } else { 0.0 }
}
