use std::collections::HashMap;

use crate::animation::Frame as AnimationFrame;
use crate::app::scroll;
use crate::{action, geometry, paint, text, theme, ui, widget, window};
use std::ops::Range;

const CURSOR_OVERLAY_MIN_SIZE: f32 = 1.0;

pub(crate) type ScrollPaintRecords = HashMap<ui::Path, ScrollPaintRecord>;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ScrollPaintRecord {
    pub(crate) target: Range<usize>,
    pub(crate) content: Range<usize>,
    pub(crate) chrome: Range<usize>,
    pub(crate) metrics: widget::scroll::Metrics,
}

pub fn tree<T>(
    root: &ui::Node,
    layout: &ui::Frame,
    actions: &action::Registry<T>,
    window: window::Id,
    interaction: &ui::Interaction,
    text_field_states: &HashMap<ui::Path, text::view::TextViewState>,
    text_engine: &mut text::layout::Engine,
    frame: AnimationFrame,
    scroll_projections: Option<&scroll::Driver>,
    path_states: &HashMap<ui::Path, action::State>,
    scene: &mut paint::Scene,
    scroll_ranges: Option<&mut ScrollPaintRecords>,
) {
    let mut overlays = Vec::new();
    let mut scroll_ranges = scroll_ranges;

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
        scroll_projections,
        path_states,
        scene,
        &mut overlays,
        &mut scroll_ranges,
    );

    for outline in overlays {
        scene.push_outline(outline);
    }
}

pub(crate) fn cursor_overlay(
    layout: &ui::Frame,
    interaction: &ui::Interaction,
    text_engine: &mut text::layout::Engine,
    scene: &mut paint::Scene,
) {
    let Some(pointer) = interaction.pointer_position() else {
        return;
    };

    let Some(overlay) = interaction.cursor_overlay() else {
        return;
    };

    match overlay {
        ui::CursorOverlay::Text(text_overlay) => {
            paint_cursor_overlay_text(layout.rect(), pointer, text_overlay, text_engine, scene);
        }
    }
}

fn paint_cursor_overlay_text(
    root_rect: geometry::Rect,
    pointer: geometry::point::Logical,
    overlay: &ui::CursorOverlayText,
    text_engine: &mut text::layout::Engine,
    scene: &mut paint::Scene,
) {
    if overlay.text().is_empty() {
        return;
    }

    let theme = theme::Theme::default_dark();
    let mut color = theme.text().primary();
    color.a *= overlay.alpha();
    let document = theme.text().document(
        text::document::Role::Control,
        overlay.text().to_owned(),
        color,
    );
    let metrics = text_engine.measure(&document, text::layout::Measure::unbounded());
    let width = metrics
        .width()
        .min(overlay.max_width())
        .max(CURSOR_OVERLAY_MIN_SIZE);
    let height = metrics.height().max(CURSOR_OVERLAY_MIN_SIZE);
    let offset = overlay.offset();
    let origin = geometry::point::logical(
        clamp_overlay_axis(
            pointer.x() + offset.x(),
            root_rect.origin.x(),
            root_rect.area.width(),
            width,
        ),
        clamp_overlay_axis(
            pointer.y() + offset.y(),
            root_rect.origin.y(),
            root_rect.area.height(),
            height,
        ),
    );

    scene.push_text(paint::Text {
        rect: geometry::Rect::new(origin, geometry::area::logical(width, height)),
        document,
        wrap: paint::TextWrap::None,
        vertical_align: paint::TextVerticalAlign::Center,
    });
}

fn clamp_overlay_axis(value: f32, min: f32, available: f32, extent: f32) -> f32 {
    let max = min + (available - extent).max(0.0);
    value.clamp(min, max)
}

fn node<T>(
    node: &ui::Node,
    layout: &ui::Frame,
    inherited_content: Option<ContentVisualState>,
    actions: &action::Registry<T>,
    window: window::Id,
    interaction: &ui::Interaction,
    text_field_states: &HashMap<ui::Path, text::view::TextViewState>,
    text_engine: &mut text::layout::Engine,
    frame: AnimationFrame,
    scroll_projections: Option<&scroll::Driver>,
    path_states: &HashMap<ui::Path, action::State>,
    scene: &mut paint::Scene,
    overlays: &mut Vec<paint::Outline>,
    scroll_ranges: &mut Option<&mut ScrollPaintRecords>,
) {
    let visual = visual_state(node, layout, actions, window, interaction, path_states);
    let content_visual = content_visual_state(node, &visual, inherited_content);
    let mut text_field_state = text_field_states
        .get(layout.path())
        .cloned()
        .unwrap_or_default();
    if node.text_surface().is_some_and(|surface| surface.is_area())
        && let Some(offset) =
            scroll_projections.and_then(|projections| projections.visual_offset(layout.path()))
    {
        text_field_state = text_field_state.with_scroll(offset.x(), offset.y());
    }
    let scroll_metrics = scroll_metrics_for_node(node, layout, scroll_projections);
    let scroll_range_start = scroll_metrics.map(|_| scene.len());
    let mut scroll_content_range = None::<Range<usize>>;
    let mut scroll_chrome_range = None::<Range<usize>>;

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

    let text_surface = node.text_surface();
    let label = text_surface.map_or_else(
        || resolved_label(node, &content_visual),
        |surface| resolved_text_surface_label(node, surface, &content_visual, &text_field_state),
    );
    let text_style = label
        .as_ref()
        .or_else(|| node.label())
        .and_then(text::document::Document::first_style)
        .unwrap_or_default();
    let text_area_scroll_projection =
        text_surface
            .and_then(text::Surface::as_area)
            .and_then(|area| {
                scroll_projections
                    .and_then(|projections| projections.text_area(layout.path()).cloned())
                    .or_else(|| {
                        scroll_metrics.and_then(|metrics| {
                            text_area_scroll_projection_for_metrics(
                                area,
                                text_style,
                                metrics,
                                &text_field_state,
                                text_engine,
                                frame.now(),
                            )
                        })
                    })
                    .or_else(|| {
                        text_area_scroll_projection(
                            node,
                            layout,
                            area,
                            text_style,
                            &text_field_state,
                            text_engine,
                            frame.now(),
                        )
                    })
            });
    let text_scroll_metrics = text_area_scroll_projection
        .as_ref()
        .map(|projection| projection.metrics());
    let text_rect = text_scroll_metrics
        .map(widget::scroll::Metrics::viewport)
        .unwrap_or_else(|| text_content_rect(node, layout));
    let text_area_layout_ref = text_area_scroll_projection
        .as_ref()
        .map(|projection| projection.layout());
    let text_field_layout_owned = if text_area_layout_ref.is_none() {
        text_surface.map(|surface| {
            let style = label
                .as_ref()
                .or_else(|| node.label())
                .and_then(text::document::Document::first_style)
                .unwrap_or_default();
            text_engine.text_layout_for_surface_at(
                surface,
                style,
                text_rect.area,
                text_field_state.clone(),
                frame.now(),
            )
        })
    } else {
        None
    };
    let text_field_layout = text_area_layout_ref.or(text_field_layout_owned.as_ref());

    let text_content_start = if text_surface.is_some() {
        scene.push_clip(paint::Clip { rect: text_rect });
        Some(scene.len())
    } else {
        None
    };

    paint_text_field_selection(
        node,
        layout,
        text_rect,
        interaction,
        text_field_layout,
        scene,
    );
    paint_text_preedit_selection(layout, text_rect, interaction, text_field_layout, scene);

    if text_surface
        .and_then(text::Surface::as_area)
        .is_some_and(|area| !area.buffer().is_empty() || text_field_state.preedit().is_some())
        && let Some(projection) = text_area_scroll_projection.as_ref()
    {
        scene.push_text_viewport(paint::TextViewport {
            rect: text_rect,
            surfaces: projection
                .render_surfaces()
                .map(|surface| paint::TextSurface {
                    rect: geometry::Rect::new(
                        geometry::point::logical(
                            text_rect.origin.x() + surface.x(),
                            text_rect.origin.y() + surface.y(),
                        ),
                        geometry::area::logical(surface.width(), surface.height()),
                    ),
                    buffer: surface.buffer(),
                    default_color: surface.default_color(),
                })
                .collect(),
        });
    } else if let Some(document) = label {
        let scroll_x = text_field_layout
            .map(text::layout::TextFieldLayout::scroll_x)
            .unwrap_or(0.0);
        let scroll_y = text_field_layout
            .map(text::layout::TextFieldLayout::scroll_y)
            .unwrap_or(0.0);
        scene.push_text(paint::Text {
            rect: if text_surface.is_some() {
                scrolled_text_rect(text_rect, scroll_x, scroll_y)
            } else {
                text_content_rect(node, layout)
            },
            document,
            wrap: text_surface
                .map(text_surface_wrap)
                .unwrap_or(paint::TextWrap::WordOrGlyph),
            vertical_align: text_surface
                .map(text_surface_vertical_align)
                .unwrap_or(paint::TextVerticalAlign::Center),
        });
    }

    paint_text_drop_caret(node, layout, interaction, scene);
    paint_text_preedit_underline(
        node,
        layout,
        text_rect,
        interaction,
        text_field_layout,
        scene,
    );
    paint_text_field_caret(node, text_rect, &visual, text_field_layout, scene);

    if text_surface.is_some() {
        if let (Some(start), Some(_)) = (text_content_start, text_scroll_metrics) {
            scroll_content_range = Some(start..scene.len());
        }
        scene.pop_clip();
    }

    if let Some(metrics) = text_scroll_metrics {
        let start = scene.len();
        widget::scroll::paint_metrics_chrome(layout.path(), metrics, interaction, scene);
        scroll_chrome_range = Some(start..scene.len());
    }

    let clip_rect = if node.clips() {
        Some(
            scroll_metrics_for_node(node, layout, scroll_projections)
                .map_or(rect, |metrics| metrics.viewport()),
        )
    } else {
        None
    };

    let child_content_start = if let Some(clip_rect) = clip_rect {
        scene.push_clip(paint::Clip { rect: clip_rect });
        Some(scene.len())
    } else if text_surface.is_none() && scroll_metrics.is_some() {
        Some(scene.len())
    } else {
        None
    };

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
            scroll_projections,
            path_states,
            scene,
            overlays,
            scroll_ranges,
        );
    }

    if text_surface.is_none()
        && scroll_metrics.is_some()
        && scroll_content_range.is_none()
        && let Some(start) = child_content_start
    {
        scroll_content_range = Some(start..scene.len());
    }

    if node.clips() {
        scene.pop_clip();
    }

    if text_surface.is_none()
        && let Some(metrics) = scroll_metrics
    {
        let start = scene.len();
        widget::scroll::paint_metrics_chrome(layout.path(), metrics, interaction, scene);
        scroll_chrome_range = Some(start..scene.len());
    }

    let recorded_scroll_metrics = text_scroll_metrics.or(scroll_metrics);
    if let (Some(start), Some(metrics), Some(ranges)) = (
        scroll_range_start,
        recorded_scroll_metrics,
        scroll_ranges.as_mut(),
    ) {
        let end = scene.len();
        let target = start..end;
        let content = scroll_content_range.unwrap_or(start..start);
        let chrome = scroll_chrome_range.unwrap_or(end..end);
        ranges.insert(
            layout.path().clone(),
            ScrollPaintRecord {
                target,
                content,
                chrome,
                metrics,
            },
        );
    }

    if let Some(outline) = resolved_focus_outline(node, rect, visual) {
        overlays.push(outline);
    }
}

pub(crate) fn scroll_subtree_recording<T>(
    root: &ui::Node,
    layout: &ui::Frame,
    actions: &action::Registry<T>,
    window: window::Id,
    interaction: &ui::Interaction,
    text_field_states: &HashMap<ui::Path, text::view::TextViewState>,
    text_engine: &mut text::layout::Engine,
    frame: AnimationFrame,
    scroll_projections: Option<&scroll::Driver>,
    path_states: &HashMap<ui::Path, action::State>,
    scene: &mut paint::Scene,
) -> ScrollPaintRecords {
    let mut overlays = Vec::new();
    let mut scroll_ranges = ScrollPaintRecords::default();
    let mut scroll_ranges_option = Some(&mut scroll_ranges);

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
        scroll_projections,
        path_states,
        scene,
        &mut overlays,
        &mut scroll_ranges_option,
    );

    scroll_ranges
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
    path_states: &HashMap<ui::Path, action::State>,
) -> VisualState {
    let action_state = path_states
        .get(layout.path())
        .copied()
        .or_else(|| {
            node.action().map(|action| {
                actions.state(action, action_context(node, layout, window, interaction))
            })
        })
        .unwrap_or_default();
    let enabled = action_state.is_enabled();
    let busy = action_state.is_busy();
    let interactive = enabled && !busy;
    let active =
        enabled && (action_state.is_active() || menu_intent_active(node.intent(), interaction));
    let hovered = interaction.hovered() == Some(layout.path());
    let pressed = interaction.pressed() == Some(layout.path());
    let focused = interaction.focused() == Some(layout.path());
    let focus_visible = enabled && focused && interaction.focus_visibility().is_visible();

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
) -> Option<crate::text::document::Document> {
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

fn resolved_text_surface_label(
    node: &ui::Node,
    surface: &text::Surface,
    content_visual: &ContentVisualState,
    state: &text::view::TextViewState,
) -> Option<crate::text::document::Document> {
    let style = node.style();
    let default_color = text::document::Style::default().color();
    let disabled_color = style
        .disabled_label_color()
        .or(style.label_color())
        .unwrap_or(default_color);
    let normal_color = if content_visual.busy {
        style
            .busy_label_color()
            .or(style.label_color())
            .unwrap_or(default_color)
    } else if !content_visual.enabled || surface.is_disabled() {
        disabled_color
    } else {
        style.label_color().unwrap_or(default_color)
    };

    if state.preedit().is_some() {
        let text = surface.presentation_text_for_state(state);
        if text.is_empty() {
            return None;
        }

        let preedit_style = node
            .label()
            .and_then(text::document::Document::first_style)
            .unwrap_or_default()
            .with_color(normal_color);
        let mut block = text::document::Block::new(text::document::Align::Start);
        block.push_run(text::document::Run::new(text, preedit_style));
        return Some(text::document::Document::from_block(block));
    }

    if surface.buffer().is_empty() {
        if let Some(placeholder) = surface.placeholder() {
            let placeholder_style = node
                .label()
                .and_then(text::document::Document::first_style)
                .unwrap_or_default()
                .with_color(disabled_color);
            let mut block = text::document::Block::new(text::document::Align::Start);
            block.push_run(text::document::Run::new(placeholder, placeholder_style));
            return Some(text::document::Document::from_block(block));
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
    let fallback = crate::text::document::Style::default().color();

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
    rect: geometry::Rect,
    interaction: &ui::Interaction,
    text_field_layout: Option<&text::layout::TextFieldLayout>,
    scene: &mut paint::Scene,
) {
    let Some(surface) = node.text_surface() else {
        return;
    };

    if !surface.is_selectable() || !surface.buffer().has_selection() {
        return;
    }

    if interaction.text_editing_target() != Some(layout.path()) {
        return;
    }

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

fn paint_text_preedit_selection(
    layout: &ui::Frame,
    rect: geometry::Rect,
    interaction: &ui::Interaction,
    text_field_layout: Option<&text::layout::TextFieldLayout>,
    scene: &mut paint::Scene,
) {
    if interaction.text_editing_target() != Some(layout.path()) {
        return;
    }

    let Some(text_field_layout) = text_field_layout else {
        return;
    };

    for span in text_field_layout.preedit_selection_spans() {
        scene.push_quad(paint::Quad {
            rect: geometry::Rect::rounded(
                geometry::point::logical(rect.origin.x() + span.x(), rect.origin.y() + span.y()),
                geometry::area::logical(span.width().max(1.0), span.height()),
                geometry::rect::Rounding::fixed(2.0),
            ),
            rasterization: paint::Rasterization::default(),
            style: paint::Style {
                fill: Some(paint::Fill::Brush(
                    paint::Color::rgba(0.18, 0.42, 0.86, 0.32).into(),
                )),
                stroke: None,
                tint: None,
            },
        });
    }
}

fn paint_text_preedit_underline(
    node: &ui::Node,
    layout: &ui::Frame,
    rect: geometry::Rect,
    interaction: &ui::Interaction,
    text_field_layout: Option<&text::layout::TextFieldLayout>,
    scene: &mut paint::Scene,
) {
    if node.text_surface().is_none() || interaction.text_editing_target() != Some(layout.path()) {
        return;
    }

    let Some(text_field_layout) = text_field_layout else {
        return;
    };

    let color = node
        .style()
        .label_color()
        .unwrap_or_else(|| text::document::Style::default().color());

    for span in text_field_layout.preedit_underline_spans() {
        let y = rect.origin.y() + span.y() + span.height().max(1.0) - 2.0;
        scene.push_quad(paint::Quad {
            rect: geometry::Rect::new(
                geometry::point::logical(rect.origin.x() + span.x(), y),
                geometry::area::logical(span.width().max(1.0), 1.0),
            ),
            rasterization: paint::Rasterization {
                snapping: paint::Snapping::FixedWidth { width_px: 1 },
                edge_mode: paint::EdgeMode::Hard,
            },
            style: paint::Style {
                fill: Some(paint::Fill::Brush(color.into())),
                stroke: None,
                tint: None,
            },
        });
    }
}

fn paint_text_field_caret(
    node: &ui::Node,
    rect: geometry::Rect,
    visual: &VisualState,
    text_field_layout: Option<&text::layout::TextFieldLayout>,
    scene: &mut paint::Scene,
) {
    if node.text_surface().is_none() {
        return;
    }

    if !visual.focused {
        return;
    }

    let Some(caret) = text_field_layout.and_then(text::layout::TextFieldLayout::caret) else {
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
                    .unwrap_or_else(|| text::document::Style::default().color())
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
    if node.text_surface().is_none() {
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
                    .unwrap_or_else(|| text::document::Style::default().color())
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
    let y = rect.origin.y() + padding.top;
    let width = (rect.area.width() - padding.left - padding.right).max(0.0);
    let height = (rect.area.height() - padding.top - padding.bottom).max(0.0);

    geometry::Rect::new(
        geometry::point::logical(x, y),
        geometry::area::logical(width, height),
    )
}

fn scrolled_text_rect(rect: geometry::Rect, scroll_x: f32, scroll_y: f32) -> geometry::Rect {
    let scroll_x = scroll_x.max(0.0);
    let scroll_y = scroll_y.max(0.0);

    geometry::Rect::new(
        geometry::point::logical(rect.origin.x() - scroll_x, rect.origin.y() - scroll_y),
        geometry::area::logical(rect.area.width() + scroll_x, rect.area.height() + scroll_y),
    )
}

fn text_area_scroll_projection(
    node: &ui::Node,
    layout: &ui::Frame,
    area_model: &text::Area,
    style: text::document::Style,
    state: &text::view::TextViewState,
    text_engine: &mut text::layout::Engine,
    now: std::time::Instant,
) -> Option<scroll::TextAreaProjection> {
    let scroll = node.scroll()?;
    if !scroll.axes().is_enabled() {
        return None;
    }

    let viewport_base = text_content_rect(node, layout);
    let (_, base_content) =
        text::layout::text_area_scroll_base_content_area(area_model, style, viewport_base.area);
    let mut content_area = text::layout::stable_text_area_content_area(
        area_model.wrap(),
        base_content,
        None,
        geometry::area::logical(0.0, 0.0),
        viewport_base.area,
    );
    let mut axes =
        scroll
            .bars()
            .active_axes(scroll.axes(), viewport_base, scroll.style(), content_area);
    for _ in 0..3 {
        let viewport = widget::scroll::viewport_rect_for_axes(viewport_base, scroll.style(), axes);
        let candidate = text_engine.text_area_metrics_layout_for_area_at(
            area_model,
            style,
            viewport.area,
            state.clone(),
            now,
        );
        let next_content = text::layout::stable_text_area_content_area(
            area_model.wrap(),
            base_content,
            Some(content_area),
            candidate.content_area(),
            viewport.area,
        );
        let next =
            scroll
                .bars()
                .active_axes(scroll.axes(), viewport_base, scroll.style(), next_content);
        content_area = next_content;

        if next == axes {
            break;
        }

        axes = next;
    }

    if state.caret_visibility_pending() {
        let viewport = widget::scroll::viewport_rect_for_axes(viewport_base, scroll.style(), axes);
        let width = match area_model.wrap() {
            text::AreaWrap::None => content_area
                .width()
                .max(state.scroll_x() + viewport.area.width()),
            text::AreaWrap::WordOrGlyph => viewport.area.width().max(0.0),
        };
        content_area = geometry::area::logical(
            width,
            content_area
                .height()
                .max(state.scroll_y() + viewport.area.height()),
        );
    }

    let metrics = widget::scroll::Metrics::resolve(
        layout.rect(),
        viewport_base,
        content_area,
        geometry::point::logical(state.scroll_x(), state.scroll_y()),
        scroll.axes(),
        scroll.bars(),
        scroll.style(),
    );

    text_area_scroll_projection_for_metrics(area_model, style, metrics, state, text_engine, now)
}

fn text_area_scroll_projection_for_metrics(
    area_model: &text::Area,
    style: text::document::Style,
    metrics: widget::scroll::Metrics,
    state: &text::view::TextViewState,
    text_engine: &mut text::layout::Engine,
    now: std::time::Instant,
) -> Option<scroll::TextAreaProjection> {
    let resolved_state = state
        .clone()
        .with_scroll(metrics.offset().x(), metrics.offset().y());
    let paint_layout = text_engine.text_area_render_layout_for_area_at(
        area_model,
        style,
        metrics.viewport().area,
        resolved_state,
        now,
        metrics.content_size(),
    );
    let (layout, surfaces, render_surfaces) = paint_layout.into_projection_parts();

    Some(scroll::TextAreaProjection::with_render_surfaces(
        metrics,
        layout,
        surfaces,
        render_surfaces,
    ))
}

fn scroll_metrics_for_node(
    node: &ui::Node,
    layout: &ui::Frame,
    scroll_projections: Option<&scroll::Driver>,
) -> Option<widget::scroll::Metrics> {
    scroll_projections
        .and_then(|projections| projections.metrics(layout.path()))
        .or_else(|| widget::scroll::metrics(node, layout))
}

fn text_surface_wrap(surface: &text::Surface) -> paint::TextWrap {
    match surface {
        text::Surface::Field(_) => paint::TextWrap::None,
        text::Surface::Area(area) => match area.wrap() {
            text::AreaWrap::None => paint::TextWrap::None,
            text::AreaWrap::WordOrGlyph => paint::TextWrap::WordOrGlyph,
        },
    }
}

fn text_surface_vertical_align(surface: &text::Surface) -> paint::TextVerticalAlign {
    match surface {
        text::Surface::Field(_) => paint::TextVerticalAlign::Center,
        text::Surface::Area(_) => paint::TextVerticalAlign::Start,
    }
}

fn normalized(value: bool) -> f32 {
    if value { 1.0 } else { 0.0 }
}
