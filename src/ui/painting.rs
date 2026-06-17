use std::collections::HashMap;

use crate::animation::Frame as AnimationFrame;
use crate::{action, geometry, paint, text, ui, widget, window};

pub fn tree<T>(
    root: &ui::Node,
    layout: &ui::Frame,
    actions: &action::Registry<T>,
    window: window::Id,
    interaction: &ui::Interaction,
    text_field_states: &HashMap<ui::Path, text::TextFieldState>,
    text_engine: &mut text::Engine,
    frame: AnimationFrame,
    scene: &mut paint::Scene,
) {
    let mut overlays = Vec::new();

    node(
        root,
        layout,
        None,
        actions,
        window,
        interaction,
        text_field_states,
        text_engine,
        frame,
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
    inherited_content: Option<ContentVisualState>,
    actions: &action::Registry<T>,
    window: window::Id,
    interaction: &ui::Interaction,
    text_field_states: &HashMap<ui::Path, text::TextFieldState>,
    text_engine: &mut text::Engine,
    frame: AnimationFrame,
    scene: &mut paint::Scene,
    overlays: &mut Vec<paint::Outline>,
) {
    let visual = visual_state(node, layout, actions, window, interaction);
    let content_visual = content_visual_state(node, &visual, inherited_content);
    let text_field_state = text_field_states
        .get(layout.path())
        .cloned()
        .unwrap_or_default();

    let rect = styled_rect(node, layout);

    if let Some(shadow) = resolved_shadow(node, rect) {
        scene.push_shadow(shadow);
    }

    if let Some(backdrop) = resolved_backdrop(node, rect) {
        scene.push_backdrop(backdrop);
    }

    if let Some(style) = resolved_quad_style(node, layout, interaction, &visual) {
        scene.push_quad(paint::Quad {
            rect,
            style,
            rasterization: paint::Rasterization::default(),
        });
    }

    for brush in resolved_tints(node, &visual) {
        scene.push_tint(paint::Tint { rect, brush });
    }

    if let Some(icon) = resolved_icon(node, layout, &content_visual) {
        scene.push_icon(icon);
    }

    let label = node.text_field().map_or_else(
        || resolved_label(node, &content_visual),
        |field| resolved_text_field_label(node, field, &content_visual, &text_field_state),
    );
    let text_field_layout = node.text_field().map(|field| {
        let style = label
            .as_ref()
            .or_else(|| node.label())
            .and_then(text::Document::first_style)
            .unwrap_or_default();
        text_engine.text_field_layout_for_field_at(
            field,
            style,
            text_content_rect(node, layout).area,
            text_field_state.clone(),
            frame.now(),
        )
    });

    if node.text_field().is_some() {
        scene.push_clip(paint::Clip {
            rect: text_content_rect(node, layout),
        });
    }

    paint_text_field_selection(node, layout, interaction, text_field_layout.as_ref(), scene);

    if let Some(document) = label {
        let scroll_x = text_field_layout
            .as_ref()
            .map(text::TextFieldLayout::scroll_x)
            .unwrap_or(0.0);
        scene.push_text(paint::Text {
            rect: if node.text_field().is_some() {
                scrolled_text_rect(node, layout, scroll_x)
            } else {
                layout.rect()
            },
            document,
            wrap: if node.text_field().is_some() {
                paint::TextWrap::None
            } else {
                paint::TextWrap::WordOrGlyph
            },
        });
    }

    paint_text_drop_caret(node, layout, interaction, scene);
    paint_text_field_caret(node, layout, &visual, text_field_layout.as_ref(), scene);

    if node.text_field().is_some() {
        scene.pop_clip();
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
            Some(content_visual),
            actions,
            window,
            interaction,
            text_field_states,
            text_engine,
            frame,
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct ContentVisualState {
    enabled: bool,
    busy: bool,
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

fn content_visual_state(
    node: &ui::Node,
    visual: &VisualState,
    inherited: Option<ContentVisualState>,
) -> ContentVisualState {
    if node_owns_content_visual_state(node) {
        return ContentVisualState {
            enabled: visual.enabled,
            busy: visual.busy,
        };
    }

    inherited.unwrap_or(ContentVisualState {
        enabled: visual.enabled,
        busy: visual.busy,
    })
}

fn node_owns_content_visual_state(node: &ui::Node) -> bool {
    node.action().is_some()
        || matches!(
            node.intent(),
            Some(ui::Intent::Action(_) | ui::Intent::OpenMenu(_) | ui::Intent::OpenSubmenu(_))
        )
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

fn resolved_label(
    node: &ui::Node,
    content_visual: &ContentVisualState,
) -> Option<crate::text::Document> {
    let style = node.style();
    let mut document = node.label().cloned()?;

    if content_visual.busy {
        if let Some(color) = style.busy_label_color().or(style.label_color()) {
            document = document.with_color(color);
        }

        return Some(document);
    }

    if !content_visual.enabled {
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

fn resolved_text_field_label(
    node: &ui::Node,
    field: &text::Field,
    content_visual: &ContentVisualState,
    state: &text::TextFieldState,
) -> Option<crate::text::Document> {
    let style = node.style();
    let default_color = text::Style::default().color();
    let disabled_color = style
        .disabled_label_color()
        .or(style.label_color())
        .unwrap_or(default_color);
    let normal_color = if content_visual.busy {
        style
            .busy_label_color()
            .or(style.label_color())
            .unwrap_or(default_color)
    } else if !content_visual.enabled || field.is_disabled() {
        disabled_color
    } else {
        style.label_color().unwrap_or(default_color)
    };

    if field.buffer().is_empty() {
        if state.preedit().is_some() {
            return None;
        }

        if let Some(placeholder) = field.placeholder() {
            let placeholder_style = node
                .label()
                .and_then(text::Document::first_style)
                .unwrap_or_default()
                .with_color(disabled_color);
            let mut block = text::Block::new(text::Align::Start);
            block.push_run(text::Run::new(placeholder, placeholder_style));
            return Some(text::Document::from_block(block));
        }
    }

    let source = node.label().cloned()?;
    Some(source.with_color(normal_color))
}

fn resolved_icon(
    node: &ui::Node,
    layout: &ui::Frame,
    content_visual: &ContentVisualState,
) -> Option<paint::Icon> {
    let icon = node.icon()?;

    Some(paint::Icon {
        rect: layout.rect(),
        icon,
        color: resolved_icon_color(node, content_visual),
        size: node.icon_size().unwrap_or(24.0),
    })
}

fn resolved_icon_color(node: &ui::Node, content_visual: &ContentVisualState) -> paint::Color {
    let style = node.style();
    let fallback = crate::text::Style::default().color();

    if content_visual.busy {
        return style
            .busy_label_color()
            .or(style.label_color())
            .unwrap_or(fallback);
    }

    if !content_visual.enabled {
        return style
            .disabled_label_color()
            .or(style.label_color())
            .unwrap_or(fallback);
    }

    style.label_color().unwrap_or(fallback)
}

fn paint_text_field_selection(
    node: &ui::Node,
    layout: &ui::Frame,
    interaction: &ui::Interaction,
    text_field_layout: Option<&text::TextFieldLayout>,
    scene: &mut paint::Scene,
) {
    let Some(field) = node.text_field() else {
        return;
    };

    if !field.is_selectable() || !field.buffer().has_selection() {
        return;
    }

    if interaction.text_editing_target() != Some(layout.path()) {
        return;
    }

    let rect = text_content_rect(node, layout);
    let Some(text_field_layout) = text_field_layout else {
        return;
    };

    for span in text_field_layout.selection_spans() {
        scene.push_quad(paint::Quad {
            rect: geometry::Rect::rounded(
                geometry::point::logical(rect.origin.x() + span.x(), rect.origin.y() + span.y()),
                geometry::area::logical(span.width().max(1.0), span.height()),
                geometry::rect::Rounding::fixed(3.0),
            ),
            rasterization: paint::Rasterization::default(),
            style: paint::Style {
                fill: Some(paint::Fill::Brush(
                    paint::Color::rgba(0.18, 0.42, 0.86, 0.48).into(),
                )),
                stroke: None,
                tint: None,
            },
        });
    }
}

fn paint_text_field_caret(
    node: &ui::Node,
    layout: &ui::Frame,
    visual: &VisualState,
    text_field_layout: Option<&text::TextFieldLayout>,
    scene: &mut paint::Scene,
) {
    if node.text_field().is_none() {
        return;
    }

    if !visual.focused {
        return;
    }

    let rect = text_content_rect(node, layout);
    let Some(caret) = text_field_layout.and_then(text::TextFieldLayout::caret) else {
        return;
    };

    scene.push_quad(paint::Quad {
        rect: geometry::Rect::new(
            geometry::point::logical(rect.origin.x() + caret.x(), rect.origin.y() + caret.y()),
            geometry::area::logical(1.0, caret.height().max(6.0)),
        ),
        rasterization: paint::Rasterization {
            snapping: paint::Snapping::FixedWidth { width_px: 2 },
            edge_mode: paint::EdgeMode::Hard,
        },
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

fn paint_text_drop_caret(
    node: &ui::Node,
    layout: &ui::Frame,
    interaction: &ui::Interaction,
    scene: &mut paint::Scene,
) {
    if node.text_field().is_none() {
        return;
    }

    let Some((path, rect)) = interaction.text_drop_caret() else {
        return;
    };

    if path != layout.path() {
        return;
    }

    scene.push_quad(paint::Quad {
        rect: geometry::Rect::new(
            rect.origin,
            geometry::area::logical(1.0, rect.area.height().max(6.0)),
        ),
        rasterization: paint::Rasterization {
            snapping: paint::Snapping::FixedWidth { width_px: 2 },
            edge_mode: paint::EdgeMode::Hard,
        },
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

fn scrolled_text_rect(node: &ui::Node, layout: &ui::Frame, scroll_x: f32) -> geometry::Rect {
    let rect = text_content_rect(node, layout);
    let scroll_x = scroll_x.max(0.0);

    geometry::Rect::new(
        geometry::point::logical(rect.origin.x() - scroll_x, rect.origin.y()),
        geometry::area::logical(rect.area.width() + scroll_x, rect.area.height()),
    )
}

fn normalized(value: bool) -> f32 {
    if value { 1.0 } else { 0.0 }
}
