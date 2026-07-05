use crate::animation;
use std::ops::Range;
use std::time::Instant;

use crate::app::{command as command_layer, frame, state::WindowState, text_input};
use crate::geometry::{Rect, area};
use crate::{command, paint, text, ui, widget, window};

pub(crate) struct PaintResult {
    pub scene: paint::Scene,
    pub timings: frame::StageTimings,
    pub layer_updates: Vec<paint::LayerUpdate>,
}

enum ScrollOnlyReplacement {
    LayerRebuild {
        target: ui::Path,
        range: Range<usize>,
        metrics: widget::scroll::Metrics,
        tiles: Vec<ScrollLayerTile>,
        chrome: Vec<paint::Item>,
    },
    RepaintTarget {
        target: ui::Path,
        range: Range<usize>,
        items: Vec<paint::Item>,
        records: ui::ScrollPaintRecords,
        refresh_layer: bool,
    },
}

struct ScrollLayerTile {
    coverage: Rect,
    scene: paint::Scene,
    records: ui::ScrollPaintRecords,
}

impl ScrollOnlyReplacement {
    fn range(&self) -> &Range<usize> {
        match self {
            Self::LayerRebuild { range, .. } | Self::RepaintTarget { range, .. } => range,
        }
    }
}

#[cfg(test)]
pub fn compose(
    window: window::Id,
    tree: &ui::Tree,
    state: &mut WindowState,
    commands: &mut command::Registry,
    text_engine: &mut text::layout::Engine,
    logical_area: area::Logical,
    frame: animation::Frame,
) -> paint::Scene {
    compose_with_timings(
        window,
        tree,
        state,
        commands,
        text_engine,
        logical_area,
        frame,
    )
    .scene
}

pub(crate) fn tree_with_runtime_popups(
    tree: &ui::Tree,
    area: area::Logical,
    menu_presenter: &dyn widget::Presenter,
    floating_surfaces: &[ui::floating::Surface],
    text_engine: &mut text::layout::Engine,
) -> (ui::Tree, Option<widget::menu::Id>, Option<widget::menu::Id>) {
    let mut presentation_tree = tree.clone();
    let menus = presentation_tree.menus();
    let open_menu_surface = floating_surfaces.iter().find(|surface| {
        matches!(surface.kind(), ui::floating::Kind::Menu(menu) if menus.contains_key(menu))
    });
    let open_menu = open_menu_surface.and_then(|surface| surface.menu_id());
    let open_submenu_surface = floating_surfaces.iter().rev().find(|surface| {
        matches!(
            surface.kind(),
            ui::floating::Kind::Submenu(menu) if open_menu.is_some() && menus.contains_key(menu)
        )
    });
    let open_submenu = open_submenu_surface.and_then(|surface| surface.menu_id());
    let context_menu = floating_surfaces
        .iter()
        .rev()
        .find(|surface| matches!(surface.kind(), ui::floating::Kind::ContextMenu { .. }));

    let mut menu_popup_inserted = false;
    if let Some(surface) = open_menu_surface
        && let Some(open_menu) = open_menu
        && let Some(menu) = menus.get(&open_menu)
        && let Some(base_layout) = presentation_tree.layout(area, text_engine)
        && let Some(popup) = widget::menu_popup(
            &presentation_tree,
            &base_layout,
            surface,
            menu,
            menu_presenter,
            text_engine,
        )
    {
        presentation_tree.push_popup(popup);
        menu_popup_inserted = true;
    }

    if menu_popup_inserted
        && let Some(surface) = open_submenu_surface
        && let Some(open_submenu) = open_submenu
        && let Some(menu) = menus.get(&open_submenu)
        && let Some(menu_layout) = presentation_tree.layout(area, text_engine)
        && let Some(popup) = widget::submenu_popup(
            &presentation_tree,
            &menu_layout,
            surface,
            menu,
            menu_presenter,
            text_engine,
        )
    {
        presentation_tree.push_popup(popup);
    }

    if let Some(surface) = context_menu
        && let Some(base_layout) = presentation_tree.layout(area, text_engine)
        && let Some(popup) = widget::text_context_menu_popup(
            surface,
            menu_presenter,
            text_engine,
            base_layout.rect(),
        )
    {
        presentation_tree.push_popup(popup);
    }

    (presentation_tree, open_menu, open_submenu)
}

pub(crate) fn compose_with_timings(
    window: window::Id,
    tree: &ui::Tree,
    state: &mut WindowState,
    commands: &mut command::Registry,
    text_engine: &mut text::layout::Engine,
    logical_area: area::Logical,
    frame: animation::Frame,
) -> PaintResult {
    let mut timings = frame::StageTimings::default();
    text_input::sync_session(state);

    let compose_start = Instant::now();
    let composition =
        compose_tree_with_runtime_state(window, tree, state, commands, text_engine, logical_area);
    timings.compose = compose_start.elapsed();

    let Some(composition) = composition else {
        clear_after_empty_composition(state, text_engine);
        return PaintResult {
            scene: paint::Scene::new(),
            timings,
            layer_updates: Vec::new(),
        };
    };

    install_composition(state, composition);
    reconcile_composed_state(window, state, commands, text_engine, frame, &mut timings);
    let (scene, layer_updates) = paint_full_frame(window, state, text_engine, frame, &mut timings);

    PaintResult {
        scene,
        timings,
        layer_updates,
    }
}

fn compose_tree_with_runtime_state(
    window: window::Id,
    tree: &ui::Tree,
    state: &WindowState,
    commands: &mut command::Registry,
    text_engine: &mut text::layout::Engine,
    logical_area: area::Logical,
) -> Option<ui::Composition> {
    let mut layer = command_layer::Layer::new(commands, window);
    layer.publish_tree_responder_binding_states(tree);
    let (presentation_tree, open_menu, open_submenu) =
        build_runtime_popup_presentation(tree, state, &layer, logical_area, text_engine);

    presentation_tree.compose_with_open_menus(logical_area, open_menu, open_submenu, text_engine)
}

fn build_runtime_popup_presentation(
    tree: &ui::Tree,
    state: &WindowState,
    layer: &command_layer::Layer<'_>,
    logical_area: area::Logical,
    text_engine: &mut text::layout::Engine,
) -> (ui::Tree, Option<widget::menu::Id>, Option<widget::menu::Id>) {
    let base_composition = tree.compose(logical_area, text_engine);
    let menu_presenter = base_composition
        .as_ref()
        .map(|composition| layer.menu_presenter_for_composition(state, composition))
        .unwrap_or_else(|| layer.menu_presenter(state));

    tree_with_runtime_popups(
        tree,
        logical_area,
        &menu_presenter,
        state.floating.surfaces(),
        text_engine,
    )
}

fn install_composition(state: &mut WindowState, composition: ui::Composition) {
    state.open_menu = composition.open_menu();
    state.open_submenu = composition.open_submenu();
    state.composition = Some(composition);
}

fn reconcile_composed_state(
    window: window::Id,
    state: &mut WindowState,
    commands: &mut command::Registry,
    text_engine: &mut text::layout::Engine,
    frame: animation::Frame,
    timings: &mut frame::StageTimings,
) {
    state.sync_menu_focus_scopes();
    state.clear_stale_focus();
    state.clear_stale_command_subject();
    text_input::sync_session(state);
    state.update_command_scope_captures(window);
    publish_composition_bindings(window, state, commands);
    state.sync_text_field_states(text_engine);

    let commit_start = Instant::now();
    state.commit_pending_visual_scroll_offsets(frame);
    state.reconcile_async_scroll_targets(text_engine, frame.now());
    timings.scroll_commit = commit_start.elapsed();

    publish_command_projection(window, state, commands);
    state.clear_stale_focus();
    state.focus_first_floating_row(commands, window);
    sync_command_visual_states(window, state, commands);

    let sync_start = Instant::now();
    state.sync_scroll_projections(text_engine, frame.now());
    timings.scroll_projection_sync = sync_start.elapsed();

    state.refine_idle_scroll_models(text_engine, frame.now());
}

fn publish_composition_bindings(
    window: window::Id,
    state: &WindowState,
    commands: &mut command::Registry,
) {
    let Some(composition) = state.composition.as_ref() else {
        return;
    };

    let mut layer = command_layer::Layer::new(commands, window);
    layer.publish_composition_responder_binding_states(composition);
}

fn publish_command_projection(
    window: window::Id,
    state: &mut WindowState,
    commands: &mut command::Registry,
) {
    text_input::publish_command_states(state, commands, window);
    sync_command_visual_states(window, state, commands);
}

fn sync_command_visual_states(
    window: window::Id,
    state: &mut WindowState,
    commands: &mut command::Registry,
) {
    let layer = command_layer::Layer::new(commands, window);
    layer.sync_visual_states(state);
}

fn paint_full_frame(
    window: window::Id,
    state: &mut WindowState,
    text_engine: &mut text::layout::Engine,
    frame: animation::Frame,
    timings: &mut frame::StageTimings,
) -> (paint::Scene, Vec<paint::LayerUpdate>) {
    let mut scene = paint::Scene::new();
    let paint_start = Instant::now();
    let scroll_ranges =
        paint_current_composition(window, state, text_engine, frame, &mut scene, true);
    timings.paint = paint_start.elapsed();
    let mut layer_updates = state.retain_paint(scene.clone(), scroll_ranges);
    layer_updates.extend(refresh_retained_scroll_layers_after_full_paint(
        window,
        state,
        text_engine,
        frame,
    ));

    (scene, layer_updates)
}

fn clear_after_empty_composition(state: &mut WindowState, text_engine: &mut text::layout::Engine) {
    state.composition = None;
    state.clear_paint_cache();
    text_input::sync_session(state);
    state.sync_text_field_states(text_engine);
    state.clear_focus();
    state.clear_command_subject();
    state.command.scope_captures.clear();
    state.scroll.clear();
    state.clear_async_scroll_targets();
}

pub(crate) fn paint_scroll_only(
    window: window::Id,
    state: &mut WindowState,
    commands: &mut command::Registry,
    text_engine: &mut text::layout::Engine,
    frame: animation::Frame,
) -> Option<PaintResult> {
    state.composition.as_ref()?;
    if state.scroll.is_empty() {
        return None;
    }
    sync_command_visual_states(window, state, commands);

    if let Some(result) = paint_scroll_only_retained(window, state, text_engine, frame) {
        return Some(result);
    }

    let mut scene = paint::Scene::new();
    let mut timings = frame::StageTimings::default();

    let commit_start = Instant::now();
    state.commit_pending_visual_scroll_offsets(frame);
    timings.scroll_commit = commit_start.elapsed();

    let sync_start = Instant::now();
    state.sync_scroll_projections(text_engine, frame.now());
    timings.scroll_projection_sync = sync_start.elapsed();

    let paint_start = Instant::now();
    paint_current_composition(window, state, text_engine, frame, &mut scene, false);
    timings.paint = paint_start.elapsed();

    Some(PaintResult {
        scene,
        timings,
        layer_updates: Vec::new(),
    })
}

fn paint_scroll_only_retained(
    _window: window::Id,
    state: &mut WindowState,
    text_engine: &mut text::layout::Engine,
    frame: animation::Frame,
) -> Option<PaintResult> {
    state.paint_cache()?;

    let mut timings = frame::StageTimings::default();

    let commit_start = Instant::now();
    state.commit_pending_visual_scroll_offsets(frame);
    let targets = state.committed_scroll_targets();
    timings.scroll_commit = commit_start.elapsed();
    if targets.is_empty() {
        return None;
    }

    let paint_start = Instant::now();
    let interaction = current_interaction(state);
    let mut replacements = Vec::new();
    let mut layer_updates = Vec::new();
    let mut skipped_projection_syncs = 0usize;

    for target in targets {
        let Some(metrics) = state.scroll.metrics(&target) else {
            return None;
        };
        let text_area_target = state
            .text_surface(&target)
            .is_some_and(text::Surface::is_area);
        let text_projection_shifted =
            text_area_target && state.scroll.text_area_projection_shifted(&target);
        if !text_area_target {
            match try_layer_retained_scroll_target(state, &target, metrics, interaction.clone()) {
                Ok(()) => {
                    skipped_projection_syncs += 1;
                    continue;
                }
                Err(miss) => {
                    trace_scroll(format_args!(
                        "layer miss target={target:?} offset={:?} miss={miss:?}",
                        metrics.offset()
                    ));
                    state.scroll.record_retained_layer_miss(miss);
                }
            }
        } else {
            trace_scroll(format_args!(
                "text target bypasses generic retained layer target={target:?} offset={:?} shifted_projection={text_projection_shifted}",
                metrics.offset()
            ));
        }

        let range = state.paint_cache()?.scroll_range(&target)?;
        let can_rebuild_layer = !text_area_target
            && state
                .paint_cache()
                .and_then(|cache| cache.scroll_layer_metrics(&target))
                .is_some();
        if let Some((metrics, tiles)) = can_rebuild_layer
            .then(|| {
                paint_scroll_layer_update_for_target(
                    state,
                    text_engine,
                    frame,
                    &target,
                    metrics,
                    interaction.clone(),
                )
            })
            .flatten()
        {
            skipped_projection_syncs += 1;
            let mut chrome = paint::Scene::new();
            widget::scroll::paint_metrics_chrome(&target, metrics, &interaction, &mut chrome);
            replacements.push(ScrollOnlyReplacement::LayerRebuild {
                target,
                range,
                metrics,
                tiles,
                chrome: chrome.items().to_vec(),
            });
            continue;
        }

        if !text_area_target {
            state.scroll.record_retained_repaint_fallback();
        }
        let sync_start = Instant::now();
        state.reconcile_async_scroll_target(&target, text_engine, frame.now());
        timings.scroll_projection_sync += sync_start.elapsed();

        let mut replacement = paint::Scene::new();
        let records = {
            let composition = state.composition.as_ref()?;
            composition.paint_scroll_target_recording_at(
                &target,
                interaction.clone(),
                state.text.states(),
                text_engine,
                frame,
                Some(&state.scroll),
                &mut replacement,
            )?
        };
        replacements.push(ScrollOnlyReplacement::RepaintTarget {
            target,
            range,
            items: replacement.items().to_vec(),
            records,
            refresh_layer: true,
        });
    }

    replacements.sort_by(|left, right| right.range().start.cmp(&left.range().start));
    let mut layer_rebuilds = 0;
    {
        let cache = state.paint_cache_mut()?;
        for replacement in replacements {
            let update = match replacement {
                ScrollOnlyReplacement::LayerRebuild {
                    target,
                    metrics,
                    tiles,
                    chrome,
                    ..
                } => {
                    let updates = cache.update_scroll_layers_from_recorded_scenes(
                        &target,
                        metrics,
                        tiles
                            .iter()
                            .map(|tile| (tile.coverage, &tile.scene, &tile.records)),
                    );
                    if !updates.is_empty() {
                        if cache
                            .replace_scroll_content_with_current_layer(&target, metrics)
                            .is_err()
                        {
                            return None;
                        }
                        if !cache.replace_scroll_chrome(&target, chrome) {
                            return None;
                        }
                    }
                    updates
                }
                ScrollOnlyReplacement::RepaintTarget {
                    target,
                    items,
                    records,
                    refresh_layer,
                    ..
                } => {
                    if !cache.replace_scroll_target(&target, items, records) {
                        return None;
                    }
                    if refresh_layer {
                        let update = cache.layer_update_for_path(&target);
                        update.into_iter().collect()
                    } else {
                        cache.remove_scroll_layers(&target);
                        Vec::new()
                    }
                }
            };
            if !update.is_empty() {
                layer_updates.extend(update);
                layer_rebuilds += 1;
            }
        }
    }
    let retained = state.paint_cache()?.retained_scroll_layers();
    state.scroll.set_retained_layers(retained);
    for _ in 0..layer_rebuilds {
        state.scroll.record_retained_layer_rebuild();
    }
    if skipped_projection_syncs > 0 {
        state
            .scroll
            .record_async_projection_sync_skip(skipped_projection_syncs);
    }
    state.clear_committed_scroll_targets();

    let scene = state.paint_cache()?.scene().clone();
    timings.paint = paint_start.elapsed();

    Some(PaintResult {
        scene,
        timings,
        layer_updates,
    })
}

fn refresh_retained_scroll_layers_after_full_paint(
    _window: window::Id,
    state: &mut WindowState,
    text_engine: &mut text::layout::Engine,
    frame: animation::Frame,
) -> Vec<paint::LayerUpdate> {
    let Some(cache) = state.paint_cache() else {
        return Vec::new();
    };
    let targets = cache
        .scroll_targets()
        .into_iter()
        .filter(|target| cache.scroll_layer_eligible(target))
        .collect::<Vec<_>>();
    let interaction = current_interaction(state);
    let mut updates = Vec::new();

    for target in targets {
        let Some(metrics) = state.scroll.metrics(&target) else {
            continue;
        };
        let Some((metrics, tiles)) = paint_scroll_layer_update_for_target(
            state,
            text_engine,
            frame,
            &target,
            metrics,
            interaction.clone(),
        ) else {
            continue;
        };
        let Some(cache) = state.paint_cache_mut() else {
            continue;
        };
        updates.extend(
            cache.update_scroll_layers_from_recorded_scenes(
                &target,
                metrics,
                tiles
                    .iter()
                    .map(|tile| (tile.coverage, &tile.scene, &tile.records)),
            ),
        );
    }

    if let Some(cache) = state.paint_cache() {
        state
            .scroll
            .set_retained_layers(cache.retained_scroll_layers());
    }

    updates
}

fn try_layer_retained_scroll_target(
    state: &mut WindowState,
    target: &ui::Path,
    metrics: widget::scroll::Metrics,
    interaction: ui::Interaction,
) -> Result<(), crate::app::scroll::RetainedLayerMiss> {
    let mut chrome = paint::Scene::new();
    widget::scroll::paint_metrics_chrome(target, metrics, &interaction, &mut chrome);
    let hit_plan = state.scroll.retained_layer_hit(target, metrics)?;
    trace_scroll(format_args!(
        "layer hit target={target:?} offset={:?} viewport={:?} source={:?}",
        metrics.offset(),
        metrics.viewport(),
        hit_plan.source()
    ));
    let hit = match {
        let Some(cache) = state.paint_cache_mut() else {
            return Err(crate::app::scroll::RetainedLayerMiss::MissingLayer);
        };
        let hit = cache.replace_scroll_content_with_layer(target, metrics, hit_plan);
        if hit.is_ok() {
            if !cache.replace_scroll_chrome(target, chrome.items().to_vec()) {
                return Err(crate::app::scroll::RetainedLayerMiss::MissingLayer);
            }
        }
        hit
    } {
        Ok(hit) => hit,
        Err(_) => return Err(crate::app::scroll::RetainedLayerMiss::MissingLayer),
    };
    if state
        .text_surface(target)
        .is_some_and(text::Surface::is_area)
        && !state.scroll.text_area_projection_shifted(target)
    {
        state.scroll.record_retained_projection_miss();
    }
    state.scroll.record_retained_layer_hit(hit);
    Ok(())
}

fn paint_scroll_layer_update_for_target(
    state: &WindowState,
    text_engine: &mut text::layout::Engine,
    frame: animation::Frame,
    target: &ui::Path,
    metrics: widget::scroll::Metrics,
    interaction: ui::Interaction,
) -> Option<(widget::scroll::Metrics, Vec<ScrollLayerTile>)> {
    let composition = state.composition.as_ref()?;
    let coverages = state.scroll.plan_retained_layer_coverages(target, metrics);
    trace_scroll(format_args!(
        "generic layer rebuild target={target:?} offset={:?} viewport={:?} coverages={coverages:?}",
        metrics.offset(),
        metrics.viewport(),
    ));
    let mut tiles = Vec::new();
    let viewport = metrics.viewport();
    for coverage in coverages {
        let layer_offset = crate::geometry::point::logical(
            metrics.offset().x() + coverage.origin.x() - viewport.origin.x(),
            metrics.offset().y() + coverage.origin.y() - viewport.origin.y(),
        );
        let layer_metrics = metrics.with_layer_viewport(coverage, layer_offset);
        let layer_scroll =
            crate::app::scroll::Driver::from_scroll_metrics(target.clone(), layer_metrics);
        let mut scene = paint::Scene::new();
        let records = composition.paint_scroll_target_recording_at(
            target,
            interaction.clone(),
            state.text.states(),
            text_engine,
            frame,
            Some(&layer_scroll),
            &mut scene,
        )?;
        tiles.push(ScrollLayerTile {
            coverage,
            scene,
            records,
        });
    }

    Some((metrics, tiles))
}

fn trace_scroll(args: std::fmt::Arguments<'_>) {
    if std::env::var_os("WGPU_L3_SCROLL_TRACE").is_some() {
        eprintln!("[wgpu_l3 scroll] {args}");
    }
}

fn paint_current_composition(
    _window: window::Id,
    state: &WindowState,
    text_engine: &mut text::layout::Engine,
    frame: animation::Frame,
    scene: &mut paint::Scene,
    record_scroll_ranges: bool,
) -> ui::ScrollPaintRecords {
    let interaction = current_interaction(state);

    if let Some(composition) = state.composition.as_ref() {
        if record_scroll_ranges {
            return composition.paint_at_recording_scroll_ranges(
                interaction,
                state.text.states(),
                text_engine,
                frame,
                Some(&state.scroll),
                scene,
            );
        } else {
            composition.paint_at(
                interaction,
                state.text.states(),
                text_engine,
                frame,
                Some(&state.scroll),
                scene,
            );
        }
    }

    ui::ScrollPaintRecords::default()
}

fn current_interaction(state: &WindowState) -> ui::Interaction {
    ui::Interaction::new(
        state.hovered.clone(),
        state.focused_path(),
        state.pressed.clone(),
    )
    .with_text_editing_target(text_input::editing_target(state))
    .with_focus_visibility(state.focus_visibility())
    .with_open_menu(state.open_menu)
    .with_open_submenu(state.open_submenu)
    .with_pointer_position(state.pointer.position())
    .with_pointer_capture(state.pointer_capture.clone())
    .with_text_drop_caret(state.text_drop_caret())
    .with_drag_drop_operation(state.drag_drop.resolved_operation())
    .with_cursor_overlay(state.drag_drop.cursor_overlay())
}

#[cfg(test)]
mod tests {
    use crate::geometry::{area, point};
    use crate::widget;
    use crate::widget::menu;
    use crate::{Command, command, paint, text, ui::layout};

    use super::*;

    const ROOT: ui::Id = ui::Id::new("root");
    const CHILD: ui::Id = ui::Id::new("child");
    const OTHER: ui::Id = ui::Id::new("other");
    struct Click;
    struct Toggle;

    impl Command for Click {
        type Args = ();
        type Output = ();

        const NAME: &'static str = "click";
        const DISPLAY: &'static str = "Click";
    }

    impl Command for Toggle {
        type Args = ();
        type Output = ();

        const NAME: &'static str = "toggle";
        const DISPLAY: &'static str = "Toggle";
    }

    const CLICK: command::Key = command::Key::of::<Click>();
    const TOGGLE: command::Key = command::Key::of::<Toggle>();
    const MENU_BAR: ui::Id = ui::Id::new("menu_bar");
    const FILE: menu::Id = menu::Id::new("file");
    const VIEW: menu::Id = menu::Id::new("view");
    const PANELS: menu::Id = menu::Id::new("panels");

    fn top_level_menu_title(index: usize) -> ui::Id {
        ui::Id::structural("menu_title", index)
    }

    fn compose(
        window: window::Id,
        tree: &ui::Tree,
        state: &mut WindowState,
        commands: &mut command::Registry,
        logical_area: area::Logical,
    ) -> paint::Scene {
        let mut text_engine = text::layout::Engine::new();

        super::compose(
            window,
            tree,
            state,
            commands,
            &mut text_engine,
            logical_area,
            crate::animation::Frame::new(std::time::Instant::now(), None),
        )
    }

    fn open_menu_surface(
        state: &mut WindowState,
        window: window::Id,
        menu: menu::Id,
        subject: ui::Path,
    ) {
        state.floating.open_top_menu(
            menu,
            command::call::Context::path(window, subject),
            command::call::Source::Pointer,
            ui::floating::FocusPolicy::PreserveCurrentFocus,
        );
        state.sync_open_menu_mirrors();
    }

    fn open_submenu_surface(
        state: &mut WindowState,
        window: window::Id,
        menu: menu::Id,
        submenu: menu::Id,
        subject: ui::Path,
    ) {
        open_menu_surface(state, window, menu, subject.clone());
        state.floating.show_submenu(
            submenu,
            command::call::Context::path(window, subject),
            command::call::Source::Pointer,
        );
        state.sync_open_menu_mirrors();
    }

    fn register_text_command<C>(registry: &mut command::Registry, display: &'static str)
    where
        C: text::command::EditCommand,
    {
        registry.commands(|commands| {
            text::command::define::<C>(commands, |command| command.with_display(display));
        });
    }

    fn text_color_for(scene: &paint::Scene, label: &str) -> Option<paint::Color> {
        scene.items().iter().find_map(|item| {
            let paint::Item::Text(text) = item else {
                return None;
            };

            text.document.blocks().iter().find_map(|block| {
                block
                    .runs()
                    .iter()
                    .find_map(|run| (run.text() == label).then_some(run.style().color()))
            })
        })
    }

    fn glyph_paint_items(scene: &paint::Scene) -> usize {
        scene
            .items()
            .iter()
            .filter(|item| {
                matches!(
                    item,
                    paint::Item::Text(_)
                        | paint::Item::TextSurface(_)
                        | paint::Item::TextViewport(_)
                        | paint::Item::Icon(_)
                )
            })
            .count()
    }

    fn text_viewport_count(scene: &paint::Scene) -> usize {
        scene
            .items()
            .iter()
            .filter(|item| matches!(item, paint::Item::TextViewport(_)))
            .count()
    }

    fn selection_quad_count(scene: &paint::Scene) -> usize {
        let fill = Some(paint::Fill::Brush(
            paint::Color::rgba(0.18, 0.42, 0.86, 0.48).into(),
        ));

        scene
            .items()
            .iter()
            .filter(|item| matches!(item, paint::Item::Quad(quad) if quad.style.fill == fill))
            .count()
    }

    fn caret_quad_count(scene: &paint::Scene) -> usize {
        let rasterization = paint::Rasterization {
            snapping: paint::Snapping::FixedWidth { width_px: 2 },
            edge_mode: paint::EdgeMode::Hard,
        };

        scene
            .items()
            .iter()
            .filter(|item| {
                matches!(
                    item,
                    paint::Item::Quad(quad)
                        if (quad.rect.area.width() - 1.0).abs() <= f32::EPSILON
                            && quad.rasterization == rasterization
                )
            })
            .count()
    }

    fn large_text_area_buffer(lines: usize) -> text::Buffer {
        text::Buffer::from_multiline_text(
            (0..lines)
                .map(|line| format!("line {line:03}: scrolling text area content"))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }

    fn selected_large_text_area_buffer(lines: usize) -> text::Buffer {
        let mut buffer = large_text_area_buffer(lines);
        let mut editor = text::edit::Editor::new();
        editor.apply_text_edit(&mut buffer, text::edit::Edit::SelectAll);
        buffer
    }

    fn large_text_area_buffer_with_cursor(lines: usize, line: usize, index: usize) -> text::Buffer {
        let content = (0..lines)
            .map(|line| format!("line {line:03}: scrolling text area content"))
            .collect::<Vec<_>>();
        let cursor_index = content
            .iter()
            .take(line)
            .map(|line| line.len() + 1)
            .sum::<usize>()
            + content
                .get(line)
                .map(|line| index.min(line.len()))
                .unwrap_or_default();
        let mut buffer = text::Buffer::from_multiline_text(content.join("\n"));
        let mut editor = text::edit::Editor::new();
        editor.apply_text_edit(
            &mut buffer,
            text::edit::Edit::set_position(text::TextPosition::new(cursor_index)),
        );
        buffer
    }

    #[test]
    fn compose_updates_state_and_preserves_paint_order() {
        let window = window::Id::new(1);
        let mut state = WindowState::default();
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();

        registry.register(command::definition::Definition::for_command::<
            Click,
            command::TestTarget,
        >());
        tree.set_root(
            widget::panel()
                .key(ROOT)
                .with_background(paint::Color::BLACK)
                .with_child(
                    widget::button_key(CHILD, CLICK)
                        .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
                ),
        );

        let scene = compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(100.0, 100.0),
        );
        let composition = state.composition.as_ref().expect("composition");

        assert!(state.composition.is_some());
        assert_eq!(
            composition
                .action(&ui::Path::new([ROOT, CHILD]))
                .map(|route| route.key()),
            Some(CLICK.action())
        );
        assert_eq!(
            composition.action_subject(&ui::Path::new([ROOT, CHILD])),
            ui::ActionSubject::Origin
        );
        assert!(composition.responder_map().is_empty());
        assert!(composition.interactivity(&ui::Path::from(ROOT)).is_some());
        assert_eq!(composition.focus_order(), &[ui::Path::new([ROOT, CHILD])]);
        assert_eq!(scene.items().len(), 2);
    }

    #[test]
    fn pointer_clicking_menu_title_opens_command_backed_popup() {
        let window = window::Id::new(1);
        let mut state = WindowState::default();
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();

        registry.register(command::definition::Definition::for_command::<
            Click,
            command::TestTarget,
        >());
        tree.set_root(
            widget::panel().key(ROOT).with_child(
                widget::menu_bar(menu::Bar::new().menu(menu::Menu::new("File").section(
                    menu::Section::new().item(menu::Item::invokes::<Click, command::TestTarget>()),
                )))
                .key(MENU_BAR),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(240.0, 120.0),
        );
        let title = ui::Path::new([ROOT, MENU_BAR, top_level_menu_title(0)]);
        let menu_id = match state.intent(&title) {
            Some(ui::Intent::OpenMenu(menu)) => menu,
            intent => panic!("menu title should open a menu, got {intent:?}"),
        };
        assert!(
            state
                .composition
                .as_ref()
                .and_then(|composition| composition.menu(menu_id))
                .is_some()
        );
        let title_rect = state
            .composition
            .as_ref()
            .and_then(|composition| composition.layout().find_path(&title))
            .map(ui::Frame::rect)
            .expect("menu title should be laid out");
        let position = point::logical(title_rect.origin.x() + 2.0, title_rect.origin.y() + 2.0);
        let mut text_engine = text::layout::Engine::new();

        let down = crate::app::input::pointer_pressed(
            &mut state,
            window,
            position,
            crate::pointer::Button::Primary,
            &mut text_engine,
        );
        assert_eq!(down.request, None);
        let up = crate::app::input::pointer_released(
            &registry,
            &mut state,
            window,
            position,
            crate::pointer::Button::Primary,
        );

        assert_eq!(
            up.intent,
            Some(crate::app::input::IntentRequest {
                origin: title,
                intent: ui::Intent::OpenMenu(menu_id),
                source: command::call::Source::Pointer,
            })
        );
        let intent = up.intent.expect("menu click should produce an intent");
        assert!(state.toggle_menu(menu_id, &mut registry, window, intent.source));
        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(240.0, 120.0),
        );

        assert!(
            state
                .composition
                .as_ref()
                .and_then(|composition| {
                    composition
                        .layout()
                        .find_path(&ui::Path::new([ROOT, widget::MENU_POPUP]))
                })
                .is_some()
        );

        let row = ui::Path::new([
            ROOT,
            widget::MENU_POPUP,
            ui::Id::structural("__menu_row", 0),
        ]);
        let row_rect = state
            .composition
            .as_ref()
            .and_then(|composition| composition.layout().find_path(&row))
            .map(ui::Frame::rect)
            .expect("menu command row should be laid out");
        let row_position = point::logical(row_rect.origin.x() + 2.0, row_rect.origin.y() + 2.0);
        let down = crate::app::input::pointer_pressed(
            &mut state,
            window,
            row_position,
            crate::pointer::Button::Primary,
            &mut text_engine,
        );
        assert_eq!(down.request, None);
        let up = crate::app::input::pointer_released(
            &registry,
            &mut state,
            window,
            row_position,
            crate::pointer::Button::Primary,
        );

        assert_eq!(
            up.request,
            Some(
                command::call::Raw::from_route(
                    command::binding::Route::invokes::<Click, command::TestTarget>(),
                    command::call::Source::Pointer,
                    command::call::Context::window(window),
                )
                .with_origin(row)
            )
        );
    }

    #[test]
    fn scroll_only_paint_keeps_text_state_async_until_text_boundary() {
        let window = window::Id::new(1);
        let path = ui::Path::new([ROOT, CHILD]);
        let text = (0..80)
            .map(|line| format!("line {line:02}: scrolling text area content"))
            .collect::<Vec<_>>()
            .join("\n");
        let buffer = text::Buffer::from_multiline_text(text);
        let mut state = WindowState::default();
        state.hovered = Some(path.clone());
        let mut registry = command::Registry::new();
        let mut text_engine = text::layout::Engine::new();
        let mut tree = ui::Tree::new();
        tree.set_root(
            widget::panel().key(ROOT).with_child(
                widget::text_area(text::Area::new(buffer))
                    .key(CHILD)
                    .with_size(layout::Size::Fill, layout::Size::Fill),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(180.0, 90.0),
        );
        assert!(state.scroll.text_area(&path).is_some());
        assert!(state.paint_cache().is_some());
        assert!(
            state
                .paint_cache()
                .and_then(|cache| cache.scroll_range(&path))
                .is_some()
        );
        let cached_scene_len = state
            .paint_cache()
            .expect("full compose should retain paint")
            .scene()
            .items()
            .len();

        let before = state
            .text
            .states()
            .get(&path)
            .map(|state| state.scroll_y())
            .unwrap_or_default();
        assert_eq!(before, 0.0);
        let now = std::time::Instant::now();
        assert!(state.queue_text_area_scroll_by_at(
            &path,
            crate::app::scroll::WheelDelta::lines(point::logical(0.0, -1.0)),
            &mut text_engine,
            now,
        ));
        let target_before_paint = state
            .scroll
            .target_offset(&path)
            .expect("smooth wheel should have a target offset");
        assert!(target_before_paint.y() > 0.0);
        assert!(state.smooth_scroll_active());
        assert_eq!(
            state
                .text
                .states()
                .get(&path)
                .expect("text area state should exist")
                .scroll_y(),
            before
        );

        text_engine.reset_diagnostics();
        let result = paint_scroll_only(
            window,
            &mut state,
            &mut registry,
            &mut text_engine,
            crate::animation::Frame::new(now + std::time::Duration::from_millis(16), Some(now)),
        )
        .expect("cached composition should support scroll-only paint");
        let projected = state
            .scroll
            .metrics(&path)
            .expect("projection metrics should advance during scroll-only paint")
            .offset();
        assert!(projected.y() > 0.0);
        assert!(projected.y() < target_before_paint.y());
        assert!(state.smooth_scroll_active());
        assert_eq!(
            state
                .scroll
                .target_offset(&path)
                .expect("smooth target should survive scroll-only paint"),
            target_before_paint
        );

        assert!(!result.scene.is_empty());
        assert_eq!(
            result.scene.items().len(),
            state
                .paint_cache()
                .expect("scroll-only retained paint should keep cache")
                .scene()
                .items()
                .len()
        );
        assert!(result.scene.items().len() >= cached_scene_len);
        assert!(
            glyph_paint_items(&result.scene) > 0,
            "text scroll content should remain in the normal text paint path"
        );
        assert_eq!(
            state
                .text
                .states()
                .get(&path)
                .expect("text area state should stay layout-scroll stable")
                .scroll_y(),
            before
        );
        assert_eq!(state.scroll.diagnostics().async_scroll_reconciles, 1);
        assert!(text_engine.diagnostics().text_area_paint_layout_calls > 0);
        assert_eq!(
            state
                .scroll
                .diagnostics()
                .async_scroll_projection_sync_skips,
            0
        );
        assert_eq!(state.scroll.diagnostics().retained_scroll_layer_hits, 0);
        assert_eq!(
            state
                .scroll
                .diagnostics()
                .retained_scroll_layer_replaced_items,
            0
        );
        assert_eq!(
            state
                .scroll
                .diagnostics()
                .retained_scroll_layer_text_prepare_skips,
            0
        );
        assert_eq!(
            state.scroll.diagnostics().retained_scroll_chrome_repaints,
            0
        );
        assert_eq!(
            state
                .scroll
                .diagnostics()
                .retained_scroll_target_repaint_fallbacks,
            0
        );
        assert_eq!(state.scroll.diagnostics().retained_scroll_layer_missing, 0);

        assert!(
            state
                .text_field_edit_at(&path, point::logical(12.0, 12.0), &mut text_engine)
                .is_some()
        );
        assert_eq!(
            state
                .scroll
                .metrics(&path)
                .expect("text boundary should keep scroll driver offset")
                .offset(),
            projected
        );
        assert_eq!(
            state
                .text
                .get(&path)
                .expect("text boundary should keep text state")
                .scroll_y(),
            before
        );
        assert_eq!(state.scroll.diagnostics().async_scroll_reconciles, 1);
    }

    #[test]
    fn pointer_leave_full_redraw_preserves_text_area_scroll_driver_offset() {
        let window = window::Id::new(1);
        let path = ui::Path::new([ROOT, CHILD]);
        let text = (0..80)
            .map(|line| format!("line {line:02}: scrolling text area content"))
            .collect::<Vec<_>>()
            .join("\n");
        let buffer = text::Buffer::from_multiline_text(text);
        let mut state = WindowState {
            hovered: Some(path.clone()),
            ..WindowState::default()
        };
        let mut registry = command::Registry::new();
        let mut text_engine = text::layout::Engine::new();
        let mut tree = ui::Tree::new();
        tree.set_root(
            widget::panel().key(ROOT).with_child(
                widget::text_area(text::Area::new(buffer))
                    .key(CHILD)
                    .with_size(layout::Size::Fill, layout::Size::Fill),
            ),
        );

        compose_with_timings(
            window,
            &tree,
            &mut state,
            &mut registry,
            &mut text_engine,
            area::logical(180.0, 90.0),
            crate::animation::Frame::new(std::time::Instant::now(), None),
        );
        assert!(state.queue_text_area_scroll_by(
            &path,
            crate::app::scroll::WheelDelta::pixels(point::logical(0.0, -8.0)),
            &mut text_engine
        ));
        let scrolled = state
            .scroll
            .metrics(&path)
            .expect("scroll metrics should update immediately")
            .offset();
        assert!(scrolled.y() > 0.0);

        paint_scroll_only(
            window,
            &mut state,
            &mut registry,
            &mut text_engine,
            crate::animation::Frame::new(std::time::Instant::now(), None),
        )
        .expect("scroll-only frame should paint");

        state.hovered = None;
        let result = compose_with_timings(
            window,
            &tree,
            &mut state,
            &mut registry,
            &mut text_engine,
            area::logical(180.0, 90.0),
            crate::animation::Frame::new(std::time::Instant::now(), None),
        );

        assert_eq!(
            state
                .scroll
                .metrics(&path)
                .expect("scroll metrics should survive pointer leave")
                .offset(),
            scrolled
        );
        let projection = state
            .scroll
            .text_area(&path)
            .expect("pointer leave should not drop the observed text-area projection");
        assert_eq!(projection.metrics().offset(), scrolled);
        assert!(
            projection
                .render_surfaces()
                .any(|surface| surface.y() < 0.0),
            "full redraw after pointer leave should still paint scrolled text content"
        );
        assert!(
            result
                .scene
                .items()
                .iter()
                .any(|item| matches!(item, paint::Item::TextViewport(_)))
        );
    }

    #[test]
    fn scroll_only_paint_keeps_overlay_text_in_scene() {
        let window = window::Id::new(1);
        let path = ui::Path::new([ROOT, CHILD]);
        let label = ui::Id::new("status");
        let text = (0..80)
            .map(|line| format!("line {line:02}: scrolling text area content"))
            .collect::<Vec<_>>()
            .join("\n");
        let buffer = text::Buffer::from_multiline_text(text);
        let mut state = WindowState::default();
        let mut registry = command::Registry::new();
        let mut text_engine = text::layout::Engine::new();
        let mut tree = ui::Tree::new();
        tree.set_root(
            widget::panel()
                .key(ROOT)
                .with_child(widget::label("Diagnostics").key(label))
                .with_child(
                    widget::text_area(text::Area::new(buffer))
                        .key(CHILD)
                        .with_size(layout::Size::Fill, layout::Size::Fill),
                ),
        );

        let full_scene = compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(180.0, 120.0),
        );
        assert!(glyph_paint_items(&full_scene) > 0);
        assert!(state.queue_text_area_scroll_by(
            &path,
            crate::app::scroll::WheelDelta::pixels(point::logical(0.0, -4.0)),
            &mut text_engine
        ));
        assert!(
            state.scroll.retained_layer_metrics(&path).is_none(),
            "text scroll content should not create a generic retained layer"
        );

        let result = paint_scroll_only(
            window,
            &mut state,
            &mut registry,
            &mut text_engine,
            crate::animation::Frame::new(std::time::Instant::now(), None),
        )
        .expect("cached composition should support scroll-only paint");

        assert!(
            glyph_paint_items(&result.scene) > 0,
            "{:?} {:?}",
            result
                .scene
                .items()
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    matches!(
                        item,
                        paint::Item::Text(_)
                            | paint::Item::TextSurface(_)
                            | paint::Item::TextViewport(_)
                            | paint::Item::Icon(_)
                    )
                })
                .collect::<Vec<_>>(),
            state.scroll.diagnostics()
        );
        assert_eq!(state.scroll.diagnostics().retained_scroll_layer_hits, 0);
        assert_eq!(
            state
                .scroll
                .diagnostics()
                .retained_scroll_target_repaint_fallbacks,
            0
        );
        assert_eq!(state.scroll.diagnostics().retained_scroll_layer_missing, 0);
    }

    #[test]
    fn text_area_scroll_only_paint_preserves_selection_overlay() {
        let window = window::Id::new(1);
        let path = ui::Path::new([ROOT, CHILD]);
        let epoch = std::time::Instant::now();
        let buffer = selected_large_text_area_buffer(120);
        let mut state = WindowState {
            focus: crate::app::state::FocusState::focused(crate::app::state::Focus::new(
                path.clone(),
                ui::focus::Reason::Pointer,
                ui::focus::Visibility::Visible,
            )),
            command: crate::app::command::State::with_subject(command::call::Scope::Path(
                path.clone(),
            )),
            ..WindowState::default()
        };
        state
            .text
            .insert(path.clone(), text::view::TextViewState::new_at(0.0, epoch));
        let mut registry = command::Registry::new();
        let mut text_engine = text::layout::Engine::new();
        let mut tree = ui::Tree::new();
        tree.set_root(
            widget::panel().key(ROOT).with_child(
                widget::text_area(text::Area::new(buffer))
                    .key(CHILD)
                    .with_size(layout::Size::Fill, layout::Size::Fill),
            ),
        );

        let full = super::compose_with_timings(
            window,
            &tree,
            &mut state,
            &mut registry,
            &mut text_engine,
            area::logical(180.0, 90.0),
            crate::animation::Frame::new(epoch, None),
        );

        assert_eq!(state.focused_path(), Some(path.clone()));
        assert!(text_viewport_count(&full.scene) > 0);
        assert!(
            selection_quad_count(&full.scene) > 0,
            "full text-area paint should include selection overlay quads"
        );
        assert!(state.queue_text_area_scroll_by_at(
            &path,
            crate::app::scroll::WheelDelta::pixels(point::logical(0.0, -8.0)),
            &mut text_engine,
            epoch + std::time::Duration::from_millis(1),
        ));
        assert!(
            state.scroll.retained_layer_metrics(&path).is_none(),
            "text areas should stay out of generic retained scroll layers"
        );

        let result = paint_scroll_only(
            window,
            &mut state,
            &mut registry,
            &mut text_engine,
            crate::animation::Frame::new(epoch + std::time::Duration::from_millis(16), Some(epoch)),
        )
        .expect("text-area scroll-only paint should repaint the text target");

        assert!(text_viewport_count(&result.scene) > 0);
        assert!(
            selection_quad_count(&result.scene) > 0,
            "scroll-only text-area paint should preserve selection overlay quads"
        );
        assert_eq!(state.scroll.diagnostics().retained_scroll_layer_hits, 0);
        assert_eq!(
            state
                .scroll
                .diagnostics()
                .retained_scroll_target_repaint_fallbacks,
            0
        );
    }

    #[test]
    fn text_area_scroll_only_paint_preserves_caret_overlay_and_blink_phase() {
        let window = window::Id::new(1);
        let path = ui::Path::new([ROOT, CHILD]);
        let epoch = std::time::Instant::now();
        let buffer = large_text_area_buffer_with_cursor(120, 0, 5);
        let mut state = WindowState {
            focus: crate::app::state::FocusState::focused(crate::app::state::Focus::new(
                path.clone(),
                ui::focus::Reason::Pointer,
                ui::focus::Visibility::Visible,
            )),
            command: crate::app::command::State::with_subject(command::call::Scope::Path(
                path.clone(),
            )),
            ..WindowState::default()
        };
        state
            .text
            .insert(path.clone(), text::view::TextViewState::new_at(0.0, epoch));
        let mut registry = command::Registry::new();
        let mut text_engine = text::layout::Engine::new();
        let mut tree = ui::Tree::new();
        tree.set_root(
            widget::panel().key(ROOT).with_child(
                widget::text_area(text::Area::new(buffer))
                    .key(CHILD)
                    .with_size(layout::Size::Fill, layout::Size::Fill),
            ),
        );

        let full = super::compose_with_timings(
            window,
            &tree,
            &mut state,
            &mut registry,
            &mut text_engine,
            area::logical(180.0, 90.0),
            crate::animation::Frame::new(epoch, None),
        );

        assert_eq!(caret_quad_count(&full.scene), 1);
        assert!(state.queue_text_area_scroll_by_at(
            &path,
            crate::app::scroll::WheelDelta::pixels(point::logical(0.0, -6.0)),
            &mut text_engine,
            epoch + std::time::Duration::from_millis(1),
        ));

        let visible = paint_scroll_only(
            window,
            &mut state,
            &mut registry,
            &mut text_engine,
            crate::animation::Frame::new(epoch + std::time::Duration::from_millis(16), Some(epoch)),
        )
        .expect("text-area scroll-only paint should repaint the text target");

        assert_eq!(caret_quad_count(&visible.scene), 1);
        assert!(state.queue_text_area_scroll_by_at(
            &path,
            crate::app::scroll::WheelDelta::pixels(point::logical(0.0, -6.0)),
            &mut text_engine,
            epoch + std::time::Duration::from_millis(501),
        ));

        let hidden = paint_scroll_only(
            window,
            &mut state,
            &mut registry,
            &mut text_engine,
            crate::animation::Frame::new(
                epoch + std::time::Duration::from_millis(500),
                Some(epoch + std::time::Duration::from_millis(16)),
            ),
        )
        .expect("text-area scroll-only paint should respect caret blink phase");

        assert_eq!(caret_quad_count(&hidden.scene), 0);
        assert_eq!(state.scroll.diagnostics().retained_scroll_layer_hits, 0);
    }

    #[test]
    fn text_area_full_paint_recomputes_caret_blink_from_reused_projection() {
        let window = window::Id::new(1);
        let path = ui::Path::new([ROOT, CHILD]);
        let epoch = std::time::Instant::now();
        let buffer = large_text_area_buffer_with_cursor(80, 0, 5);
        let mut state = WindowState {
            focus: crate::app::state::FocusState::focused(crate::app::state::Focus::new(
                path.clone(),
                ui::focus::Reason::Pointer,
                ui::focus::Visibility::Visible,
            )),
            command: crate::app::command::State::with_subject(command::call::Scope::Path(
                path.clone(),
            )),
            ..WindowState::default()
        };
        state
            .text
            .insert(path.clone(), text::view::TextViewState::new_at(0.0, epoch));
        let mut registry = command::Registry::new();
        let mut text_engine = text::layout::Engine::new();
        let mut tree = ui::Tree::new();
        tree.set_root(
            widget::panel().key(ROOT).with_child(
                widget::text_area(text::Area::new(buffer))
                    .key(CHILD)
                    .with_size(layout::Size::Fill, layout::Size::Fill),
            ),
        );

        let visible = super::compose_with_timings(
            window,
            &tree,
            &mut state,
            &mut registry,
            &mut text_engine,
            area::logical(180.0, 90.0),
            crate::animation::Frame::new(epoch, None),
        );
        let hidden = super::compose_with_timings(
            window,
            &tree,
            &mut state,
            &mut registry,
            &mut text_engine,
            area::logical(180.0, 90.0),
            crate::animation::Frame::new(epoch + std::time::Duration::from_millis(500), None),
        );

        assert_eq!(caret_quad_count(&visible.scene), 1);
        assert_eq!(caret_quad_count(&hidden.scene), 0);
    }

    #[test]
    fn text_area_full_paint_reveals_large_document_caret_overlay() {
        let window = window::Id::new(1);
        let path = ui::Path::new([ROOT, CHILD]);
        let epoch = std::time::Instant::now();
        let buffer = large_text_area_buffer_with_cursor(120, 100, 5);
        let mut state = WindowState {
            focus: crate::app::state::FocusState::focused(crate::app::state::Focus::new(
                path.clone(),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            command: crate::app::command::State::with_subject(command::call::Scope::Path(
                path.clone(),
            )),
            ..WindowState::default()
        };
        state
            .text
            .insert(path.clone(), text::view::TextViewState::new_at(0.0, epoch));
        let mut registry = command::Registry::new();
        let mut text_engine = text::layout::Engine::new();
        let mut tree = ui::Tree::new();
        tree.set_root(
            widget::panel().key(ROOT).with_child(
                widget::text_area(text::Area::new(buffer))
                    .key(CHILD)
                    .with_size(layout::Size::Fill, layout::Size::Fill),
            ),
        );

        super::compose_with_timings(
            window,
            &tree,
            &mut state,
            &mut registry,
            &mut text_engine,
            area::logical(180.0, 90.0),
            crate::animation::Frame::new(epoch, None),
        );
        assert!(state.ensure_text_caret_visible_after_edit(&path, epoch, &mut text_engine, None));
        let result = super::compose_with_timings(
            window,
            &tree,
            &mut state,
            &mut registry,
            &mut text_engine,
            area::logical(180.0, 90.0),
            crate::animation::Frame::new(epoch, None),
        );

        assert!(
            state
                .scroll
                .metrics(&path)
                .expect("large text area should have scroll metrics")
                .offset()
                .y()
                > 0.0,
            "caret reveal should scroll the large text area"
        );
        assert_eq!(caret_quad_count(&result.scene), 1);
    }

    #[test]
    fn full_compose_does_not_refresh_retained_layers_for_text_area() {
        let window = window::Id::new(1);
        let path = ui::Path::new([ROOT, CHILD]);
        let text = (0..80)
            .map(|line| format!("line {line:02}: scrolling text area content"))
            .collect::<Vec<_>>()
            .join("\n");
        let buffer = text::Buffer::from_multiline_text(text);
        let mut state = WindowState::default();
        let mut registry = command::Registry::new();
        let mut text_engine = text::layout::Engine::new();
        let mut tree = ui::Tree::new();
        tree.set_root(
            widget::panel().key(ROOT).with_child(
                widget::text_area(text::Area::new(buffer))
                    .key(CHILD)
                    .with_size(layout::Size::Fill, layout::Size::Fill),
            ),
        );

        text_engine.reset_diagnostics();
        let result = super::compose_with_timings(
            window,
            &tree,
            &mut state,
            &mut registry,
            &mut text_engine,
            area::logical(180.0, 90.0),
            crate::animation::Frame::new(std::time::Instant::now(), None),
        );

        assert!(!result.scene.is_empty());
        assert!(glyph_paint_items(&result.scene) > 0);
        assert!(
            result.layer_updates.is_empty(),
            "text areas should not build hidden retained-layer updates after visible paint"
        );
        assert!(
            !state
                .paint_cache()
                .expect("full compose should retain paint records")
                .scroll_layer_eligible(&path)
        );
        assert!(state.scroll.retained_layer_metrics(&path).is_none());
        assert_eq!(
            text_engine.diagnostics().text_area_paint_layout_calls,
            1,
            "full compose should paint the visible text area once without a hidden layer refresh"
        );
    }

    #[test]
    fn scroll_only_paint_translates_generic_scroll_view_content() {
        let window = window::Id::new(1);
        let path = ui::Path::new([ROOT, CHILD]);
        let mut state = WindowState::default();
        let mut registry = command::Registry::new();
        let mut text_engine = text::layout::Engine::new();
        let mut tree = ui::Tree::new();
        let mut scroll = widget::scroll_view()
            .key(CHILD)
            .with_size(layout::Size::Fixed(120.0), layout::Size::Fixed(60.0))
            .with_gap(4.0);
        for id in [
            ui::Id::new("row_0"),
            ui::Id::new("row_1"),
            ui::Id::new("row_2"),
            ui::Id::new("row_3"),
        ]
        .iter()
        .copied()
        {
            scroll.push_child(
                ui::Node::leaf()
                    .key(id)
                    .with_background(paint::Brush::solid(paint::Color::RED))
                    .with_size(layout::Size::Fill, layout::Size::Fixed(40.0)),
            );
        }
        tree.set_root(widget::panel().key(ROOT).with_child(scroll));

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(180.0, 120.0),
        );
        let metrics = state
            .scroll
            .metrics(&path)
            .expect("generic scroll view should have metrics");
        assert!(metrics.max_offset().y() > 0.0);
        assert!(state.paint_cache().is_some());
        assert!(state.scroll.queue_offset(&path, point::logical(0.0, 18.0)));

        let result = paint_scroll_only(
            window,
            &mut state,
            &mut registry,
            &mut text_engine,
            crate::animation::Frame::new(std::time::Instant::now(), None),
        )
        .expect("generic scroll view should support scroll-only paint");

        assert!(!result.scene.is_empty());
        assert_eq!(state.scroll.diagnostics().retained_scroll_layer_hits, 1);
        assert!(
            state
                .scroll
                .diagnostics()
                .retained_scroll_layer_replaced_items
                > 0
        );
        assert_eq!(
            state
                .scroll
                .diagnostics()
                .retained_scroll_target_repaint_fallbacks,
            0
        );
        assert_eq!(
            state
                .scroll
                .metrics(&path)
                .expect("metrics should remain")
                .offset(),
            point::logical(0.0, 18.0)
        );
    }

    #[test]
    fn scroll_only_paint_falls_back_without_retained_cache() {
        let window = window::Id::new(1);
        let path = ui::Path::new([ROOT, CHILD]);
        let text = (0..80)
            .map(|line| format!("line {line:02}: scrolling text area content"))
            .collect::<Vec<_>>()
            .join("\n");
        let buffer = text::Buffer::from_multiline_text(text);
        let mut state = WindowState::default();
        let mut registry = command::Registry::new();
        let mut text_engine = text::layout::Engine::new();
        let mut tree = ui::Tree::new();
        tree.set_root(
            widget::panel().key(ROOT).with_child(
                widget::text_area(text::Area::new(buffer))
                    .key(CHILD)
                    .with_size(layout::Size::Fill, layout::Size::Fill),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(180.0, 90.0),
        );
        state.clear_paint_cache();
        assert!(state.queue_text_area_scroll_to(
            &path,
            point::logical(0.0, 36.0),
            &mut text_engine
        ));

        let result = paint_scroll_only(
            window,
            &mut state,
            &mut registry,
            &mut text_engine,
            crate::animation::Frame::new(std::time::Instant::now(), None),
        )
        .expect("scroll-only paint should fall back to normal composition paint");

        assert!(!result.scene.is_empty());
        assert!(state.paint_cache().is_none());
        assert_eq!(
            state
                .scroll
                .metrics(&path)
                .expect("scroll metrics should remain available")
                .offset(),
            point::logical(0.0, 36.0)
        );
    }

    #[test]
    fn scroll_only_paint_repaints_text_for_large_text_area_jump() {
        let window = window::Id::new(1);
        let path = ui::Path::new([ROOT, CHILD]);
        let text = (0..240)
            .map(|line| format!("line {line:03}: scrolling text area content"))
            .collect::<Vec<_>>()
            .join("\n");
        let buffer = text::Buffer::from_multiline_text(text);
        let mut state = WindowState::default();
        state.hovered = Some(path.clone());
        let mut registry = command::Registry::new();
        let mut text_engine = text::layout::Engine::new();
        let mut tree = ui::Tree::new();
        tree.set_root(
            widget::panel().key(ROOT).with_child(
                widget::text_area(text::Area::new(buffer))
                    .key(CHILD)
                    .with_size(layout::Size::Fill, layout::Size::Fill),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(180.0, 90.0),
        );
        text_engine.reset_diagnostics();
        assert!(state.queue_text_area_scroll_by(
            &path,
            crate::app::scroll::WheelDelta::pixels(point::logical(0.0, -1200.0)),
            &mut text_engine
        ));

        let result = paint_scroll_only(
            window,
            &mut state,
            &mut registry,
            &mut text_engine,
            crate::animation::Frame::new(std::time::Instant::now(), None),
        )
        .expect("cached composition should support scroll-only paint with fallback");

        assert!(!result.scene.is_empty());
        assert!(
            glyph_paint_items(&result.scene) > 0,
            "text scroll content should repaint through the text renderer"
        );
        assert!(result.layer_updates.is_empty());
        assert_eq!(state.scroll.diagnostics().async_scroll_reconciles, 1);
        let layout_calls = text_engine.diagnostics().text_area_paint_layout_calls;
        assert!(layout_calls > 0);
        assert!(layout_calls <= 3);
        let diagnostics = state.scroll.diagnostics();
        assert_eq!(diagnostics.retained_scroll_layer_hits, 0);
        assert_eq!(diagnostics.retained_scroll_layer_missing, 0);
        assert_eq!(diagnostics.retained_scroll_layer_coverage_misses, 0);
        assert_eq!(diagnostics.retained_scroll_target_repaint_fallbacks, 0);
        assert_eq!(diagnostics.retained_scroll_layer_rebuilds, 0);

        assert!(state.queue_text_area_scroll_by(
            &path,
            crate::app::scroll::WheelDelta::pixels(point::logical(0.0, -72.0)),
            &mut text_engine
        ));

        let result = paint_scroll_only(
            window,
            &mut state,
            &mut registry,
            &mut text_engine,
            crate::animation::Frame::new(std::time::Instant::now(), None),
        )
        .expect("text-area scroll-only repaint should support follow-up scroll");

        assert!(!result.scene.is_empty());
        assert!(glyph_paint_items(&result.scene) > 0);
        let diagnostics = state.scroll.diagnostics();
        assert_eq!(diagnostics.retained_scroll_layer_hits, 0);
        assert_eq!(diagnostics.retained_scroll_layer_missing, 0);
        assert_eq!(diagnostics.retained_scroll_layer_coverage_misses, 0);
        assert_eq!(diagnostics.retained_scroll_target_repaint_fallbacks, 0);
        assert_eq!(diagnostics.retained_scroll_layer_rebuilds, 0);
    }

    #[test]
    fn compose_clears_stale_focused_paths_after_tree_rebuild() {
        let window = window::Id::new(1);
        let mut state = WindowState {
            focus: crate::app::state::FocusState::focused(crate::app::state::Focus::new(
                ui::Path::new([ROOT, CHILD]),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            ..WindowState::default()
        };
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();

        tree.set_root(
            widget::panel().key(ROOT).with_child(
                widget::button_key(OTHER, CLICK)
                    .with_responder_key(CLICK.action())
                    .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(100.0, 100.0),
        );

        assert_eq!(state.focused_path(), None);
    }

    #[test]
    fn compose_clears_stale_command_subject_after_tree_rebuild() {
        let window = window::Id::new(1);
        let mut state = WindowState {
            command: crate::app::command::State::with_subject(command::call::Scope::Path(
                ui::Path::new([ROOT, CHILD]),
            )),
            ..WindowState::default()
        };
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();

        tree.set_root(
            widget::panel().key(ROOT).with_child(
                widget::button_key(OTHER, CLICK)
                    .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(100.0, 100.0),
        );

        assert_eq!(state.command.subject, None);
    }

    #[test]
    fn compose_stores_responder_paths() {
        let window = window::Id::new(1);
        let mut state = WindowState::default();
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();

        tree.set_root(
            widget::panel().key(ROOT).with_child(
                ui::Node::leaf()
                    .key(CHILD)
                    .with_responder_key(
                        command::Key::of::<crate::text::command::SelectAll>().action(),
                    )
                    .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(100.0, 100.0),
        );
        let composition = state.composition.as_ref().expect("composition");

        assert_eq!(
            composition.responders(&ui::Path::new([ROOT, CHILD])),
            Some(&[command::Key::of::<crate::text::command::SelectAll>().action()][..])
        );
    }

    #[test]
    fn compose_publishes_responder_binding_state() {
        let window = window::Id::new(1);
        let mut state = WindowState::default();
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();
        let path = ui::Path::new([ROOT, CHILD]);
        let binding =
            command::binding::Binding::new(command::Key::of::<crate::text::command::SelectAll>())
                .available(false)
                .active(true)
                .running(true);

        register_text_command::<crate::text::command::SelectAll>(&mut registry, "Select All");
        tree.set_root(
            widget::panel().key(ROOT).with_child(
                ui::Node::leaf()
                    .key(CHILD)
                    .with_responder_binding(binding.action())
                    .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(100.0, 100.0),
        );
        let composition = state.composition.as_ref().expect("composition");

        assert_eq!(
            composition.responders(&path),
            Some(&[command::Key::of::<crate::text::command::SelectAll>().action()][..])
        );
        assert_eq!(
            composition.responder_bindings(&path),
            Some(&[binding.action()][..])
        );
        assert_eq!(
            registry.configured_state_key(
                command::Key::of::<crate::text::command::SelectAll>(),
                command::call::Context::path(window, path)
            ),
            binding.state().expect("projected binding state").clone()
        );
    }

    #[test]
    fn compose_disables_text_commands_without_text_editing_focus() {
        let window = window::Id::new(1);
        let path = ui::Path::new([ROOT, CHILD]);
        let mut state = WindowState::default();
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();

        register_text_command::<crate::text::command::SelectAll>(&mut registry, "Select All");
        register_text_command::<crate::text::command::Paste>(&mut registry, "Paste");
        tree.set_root(
            widget::panel()
                .key(ROOT)
                .with_child(widget::text_field(text::Buffer::from_text("hello")).key(CHILD)),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(100.0, 100.0),
        );

        assert!(
            !registry
                .configured_state::<crate::text::command::SelectAll>(command::call::Context::path(
                    window,
                    path.clone(),
                ))
                .is_available()
        );
        assert!(
            !registry
                .configured_state::<crate::text::command::Paste>(command::call::Context::path(
                    window, path
                ))
                .is_available()
        );
    }

    #[test]
    fn compose_enables_text_commands_for_focused_text_field() {
        let window = window::Id::new(1);
        let path = ui::Path::new([ROOT, CHILD]);
        let mut state = WindowState {
            focus: crate::app::state::FocusState::focused(crate::app::state::Focus::new(
                path.clone(),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            ..WindowState::default()
        };
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();

        register_text_command::<crate::text::command::SelectAll>(&mut registry, "Select All");
        register_text_command::<crate::text::command::Paste>(&mut registry, "Paste");
        tree.set_root(
            widget::panel()
                .key(ROOT)
                .with_child(widget::text_field(text::Buffer::from_text("hello")).key(CHILD)),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(100.0, 100.0),
        );

        assert!(
            registry
                .configured_state::<crate::text::command::SelectAll>(command::call::Context::path(
                    window,
                    path.clone(),
                ))
                .is_available()
        );
        assert!(
            registry
                .configured_state::<crate::text::command::Paste>(command::call::Context::path(
                    window, path
                ))
                .is_available()
        );
    }

    #[test]
    fn open_menu_projects_disabled_select_all_when_text_is_fully_selected() {
        let window = window::Id::new(1);
        let field = ui::Path::new([ROOT, CHILD]);
        let mut editor = text::edit::Editor::new();
        let mut buffer = text::Buffer::from_text("hello");
        let mut state = WindowState {
            focus: crate::app::state::FocusState::focused(crate::app::state::Focus::new(
                field.clone(),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            ..WindowState::default()
        };
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();

        editor.apply_text_edit(&mut buffer, text::edit::Edit::SelectAll);
        register_text_command::<crate::text::command::SelectAll>(&mut registry, "Select All");
        tree.set_root(
            widget::panel()
                .key(ROOT)
                .with_child(
                    widget::menu_bar(
                        menu::Bar::new().menu(
                            menu::Menu::new("File").key(FILE).section(
                                menu::Section::new()
                                    .item(menu::Item::text::<crate::text::command::SelectAll>()),
                            ),
                        ),
                    )
                    .key(MENU_BAR),
                )
                .with_child(widget::text_field(buffer).key(CHILD)),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );
        open_menu_surface(&mut state, window, FILE, field.clone());
        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );

        let row = ui::Path::new([
            ROOT,
            widget::MENU_POPUP,
            ui::Id::structural("__menu_row", 0),
        ]);
        let composition = state.composition.as_ref().expect("composition");

        assert_eq!(
            composition.action(&row).map(|route| route.key()),
            Some(command::Key::of::<crate::text::command::SelectAll>().action())
        );
        assert!(
            !registry
                .configured_state::<crate::text::command::SelectAll>(command::call::Context::path(
                    window, field,
                ))
                .is_available()
        );
    }

    #[test]
    fn compose_captures_inherited_command_subject_for_scope() {
        let window = window::Id::new(1);
        let subject = ui::Path::new([ROOT, CHILD]);
        let scope = ui::Path::new([ROOT, OTHER]);
        let mut state = WindowState {
            command: crate::app::command::State::with_subject(command::call::Scope::Path(
                subject.clone(),
            )),
            ..WindowState::default()
        };
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();

        tree.set_root(
            widget::panel()
                .key(ROOT)
                .with_child(
                    ui::Node::leaf()
                        .key(CHILD)
                        .with_responder_key(
                            command::Key::of::<crate::text::command::SelectAll>().action(),
                        )
                        .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
                )
                .with_child(
                    widget::panel()
                        .key(OTHER)
                        .with_action_scope()
                        .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
                ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(100.0, 100.0),
        );

        assert_eq!(
            state.command.scope_captures.get(&scope),
            Some(&command::call::Context::path(window, subject))
        );
    }

    #[test]
    fn compose_clears_stale_scope_captures() {
        let window = window::Id::new(1);
        let scope = ui::Path::new([ROOT, OTHER]);
        let subject = ui::Path::new([ROOT, CHILD]);
        let mut state = WindowState {
            command: crate::app::command::State::with_scope_captures(
                std::collections::HashMap::from([(
                    scope,
                    command::call::Context::path(window, subject),
                )]),
            ),
            ..WindowState::default()
        };
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();

        tree.set_root(widget::panel().key(ROOT));

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(100.0, 100.0),
        );

        assert!(state.command.scope_captures.is_empty());
    }

    #[test]
    fn compose_injects_open_menu_popup_from_menu_bar() {
        let window = window::Id::new(1);
        let subject = ui::Path::new([ROOT, CHILD]);
        let mut state = WindowState {
            command: crate::app::command::State::with_subject(command::call::Scope::Path(
                subject.clone(),
            )),
            ..WindowState::default()
        };
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();

        register_text_command::<crate::text::command::SelectAll>(&mut registry, "Select All");
        registry.set_state_key(
            command::Key::of::<crate::text::command::SelectAll>(),
            command::call::Context::path(window, subject.clone()),
            command::State::available(),
        );
        open_menu_surface(&mut state, window, FILE, subject.clone());
        tree.set_root(
            widget::panel()
                .key(ROOT)
                .with_child(
                    widget::menu_bar(
                        menu::Bar::new().menu(
                            menu::Menu::new("File").key(FILE).section(
                                menu::Section::new()
                                    .item(menu::Item::text::<crate::text::command::SelectAll>()),
                            ),
                        ),
                    )
                    .key(MENU_BAR),
                )
                .with_child(
                    ui::Node::leaf()
                        .key(CHILD)
                        .with_responder_key(
                            command::Key::of::<crate::text::command::SelectAll>().action(),
                        )
                        .with_interactivity(ui::Interactivity::CONTROL),
                ),
        );

        let scene = compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );
        let theme = crate::theme::Theme::default_dark();

        let row = ui::Path::new([
            ROOT,
            widget::MENU_POPUP,
            ui::Id::structural("__menu_row", 0),
        ]);
        let scope = ui::Path::new([ROOT, widget::MENU_POPUP]);
        let composition = state.composition.as_ref().expect("composition");
        let layout = composition.layout();

        assert!(layout.find_path(&scope).is_some());
        let popup_rect = layout
            .find_path(&scope)
            .expect("menu popup root layout")
            .rect();
        let background_hit = layout.hit_test_where(
            point::logical(popup_rect.origin.x() + 1.0, popup_rect.origin.y() + 1.0),
            |path| {
                composition
                    .interactivity(path)
                    .is_some_and(|interactivity| interactivity.hit_test())
            },
        );

        assert_eq!(background_hit, Some(scope.clone()));
        assert!(
            composition
                .interactivity(&scope)
                .is_some_and(|interactivity| interactivity.hit_test()
                    && !interactivity.focusable()
                    && !interactivity.actionable())
        );
        assert_eq!(
            composition.action(&row).map(|route| route.key()),
            Some(command::Key::of::<crate::text::command::SelectAll>().action())
        );
        assert_eq!(
            composition.action_subject(&row),
            ui::ActionSubject::Captured
        );
        assert_eq!(
            state.command.scope_captures.get(&scope),
            Some(&command::call::Context::path(window, subject))
        );

        let backdrop_index = scene
            .items()
            .iter()
            .position(|item| matches!(item, paint::Item::Backdrop(_)))
            .expect("open menu should lower a backdrop item");
        let paint::Item::Backdrop(backdrop) = &scene.items()[backdrop_index] else {
            unreachable!();
        };
        let paint::BackdropFilter::Blur { amount } = backdrop.filter;
        assert_eq!(amount, theme.floating_panel().backdrop_blur());

        let paint::Item::Quad(quad) = &scene.items()[backdrop_index + 1] else {
            panic!("popup material fill should follow popup backdrop");
        };
        assert_eq!(
            quad.style.fill,
            Some(paint::Fill::Brush(theme.floating_panel().backdrop_fill()))
        );
    }

    #[test]
    fn compose_disables_top_level_menu_with_no_available_commands() {
        let window = window::Id::new(1);
        let mut state = WindowState::default();
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();
        let menu_title = ui::Path::new([ROOT, MENU_BAR, top_level_menu_title(0)]);

        tree.set_root(
            widget::panel().key(ROOT).with_child(
                widget::menu_bar(
                    menu::Bar::new().menu(
                        menu::Menu::new("File")
                            .key(FILE)
                            .section(menu::Section::new().command_key(CLICK)),
                    ),
                )
                .key(MENU_BAR),
            ),
        );

        let scene = compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );

        let composition = state.composition.as_ref().expect("composition");
        assert_eq!(
            composition.visual_state(&menu_title),
            Some(ui::VisualState::unavailable())
        );
        assert_eq!(
            composition.interactivity(&menu_title),
            Some(ui::Interactivity::NONE)
        );
        assert!(!composition.focus_order().contains(&menu_title));
        assert_eq!(
            text_color_for(&scene, "File"),
            Some(crate::theme::Theme::default_dark().text().disabled())
        );
        let title_rect = composition
            .layout()
            .find_path(&menu_title)
            .expect("menu title layout")
            .rect();
        let title_center = point::logical(
            title_rect.origin.x() + title_rect.area.width() * 0.5,
            title_rect.origin.y() + title_rect.area.height() * 0.5,
        );

        assert_ne!(state.hit_test(title_center), Some(menu_title));
        assert!(!state.toggle_menu(FILE, &mut registry, window, command::call::Source::Pointer));
    }

    #[test]
    fn compose_enables_top_level_menu_when_descendant_command_can_run() {
        let window = window::Id::new(1);
        let mut state = WindowState::default();
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();
        let menu_title = ui::Path::new([ROOT, MENU_BAR, top_level_menu_title(0)]);

        registry.register(command::definition::Definition::for_command::<
            Click,
            command::TestTarget,
        >());
        tree.set_root(
            widget::panel().key(ROOT).with_child(
                widget::menu_bar(
                    menu::Bar::new().menu(
                        menu::Menu::new("File")
                            .key(FILE)
                            .section(menu::Section::new().command_key(CLICK)),
                    ),
                )
                .key(MENU_BAR),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );

        let composition = state.composition.as_ref().expect("composition");
        assert_eq!(
            composition.visual_state(&menu_title),
            Some(ui::VisualState::available())
        );
        assert!(
            composition
                .interactivity(&menu_title)
                .is_some_and(|interactivity| interactivity.hit_test()
                    && interactivity.focusable()
                    && interactivity.actionable())
        );
        assert!(composition.focus_order().contains(&menu_title));
    }

    #[test]
    fn compose_counts_submenu_descendant_commands_for_top_level_availability() {
        let window = window::Id::new(1);
        let mut state = WindowState::default();
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();
        let menu_title = ui::Path::new([ROOT, MENU_BAR, top_level_menu_title(0)]);

        registry.register(command::definition::Definition::for_command::<
            Click,
            command::TestTarget,
        >());
        tree.set_root(
            widget::panel().key(ROOT).with_child(
                widget::menu_bar(
                    menu::Bar::new().menu(
                        menu::Menu::new("File").key(FILE).section(
                            menu::Section::new().submenu(
                                menu::Menu::new("Panels")
                                    .key(PANELS)
                                    .section(menu::Section::new().command_key(CLICK)),
                            ),
                        ),
                    ),
                )
                .key(MENU_BAR),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );

        let composition = state.composition.as_ref().expect("composition");
        assert_eq!(
            composition.visual_state(&menu_title),
            Some(ui::VisualState::available())
        );
        assert!(
            composition
                .interactivity(&menu_title)
                .is_some_and(|interactivity| interactivity.actionable())
        );
    }

    #[test]
    fn compose_keeps_text_command_menu_unavailable_without_active_text_target() {
        let window = window::Id::new(1);
        let mut state = WindowState::default();
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();
        let menu_title = ui::Path::new([ROOT, MENU_BAR, top_level_menu_title(0)]);

        register_text_command::<crate::text::command::SelectAll>(&mut registry, "Select All");
        tree.set_root(
            widget::panel()
                .key(ROOT)
                .with_child(
                    widget::menu_bar(
                        menu::Bar::new().menu(
                            menu::Menu::new("Edit").key(FILE).section(
                                menu::Section::new()
                                    .item(menu::Item::text::<crate::text::command::SelectAll>()),
                            ),
                        ),
                    )
                    .key(MENU_BAR),
                )
                .with_child(
                    widget::text_area(text::Area::new(text::Buffer::from_text("hello")))
                        .key(CHILD)
                        .with_size(layout::Size::Fill, layout::Size::Fixed(80.0)),
                ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );

        let composition = state.composition.as_ref().expect("composition");
        assert_eq!(
            composition.visual_state(&menu_title),
            Some(ui::VisualState::unavailable())
        );
    }

    #[test]
    fn compose_keeps_global_command_menu_available_with_focused_text_target() {
        let window = window::Id::new(1);
        let text_target = ui::Path::new([ROOT, CHILD]);
        let mut state = WindowState {
            focus: crate::app::state::FocusState::focused(crate::app::state::Focus::new(
                text_target.clone(),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            command: crate::app::command::State::with_subject(command::call::Scope::Path(
                text_target,
            )),
            ..WindowState::default()
        };
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();
        let menu_title = ui::Path::new([ROOT, MENU_BAR, top_level_menu_title(0)]);

        registry.register(command::definition::Definition::for_command::<
            Click,
            command::TestTarget,
        >());
        tree.set_root(
            widget::panel()
                .key(ROOT)
                .with_child(
                    widget::menu_bar(
                        menu::Bar::new().menu(
                            menu::Menu::new("File").key(FILE).section(
                                menu::Section::new()
                                    .item(menu::Item::invokes::<Click, command::TestTarget>()),
                            ),
                        ),
                    )
                    .key(MENU_BAR),
                )
                .with_child(
                    widget::text_area(text::Area::new(text::Buffer::from_text("hello")))
                        .key(CHILD)
                        .with_size(layout::Size::Fill, layout::Size::Fixed(80.0)),
                ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );

        let composition = state.composition.as_ref().expect("composition");
        assert_eq!(
            composition.visual_state(&menu_title),
            Some(ui::VisualState::available())
        );
    }

    #[test]
    fn pointer_opened_menu_preserves_text_field_focus_after_compose() {
        let window = window::Id::new(1);
        let field = ui::Path::new([ROOT, CHILD]);
        let mut state = WindowState {
            focus: crate::app::state::FocusState::focused(crate::app::state::Focus::new(
                field.clone(),
                ui::focus::Reason::Pointer,
                ui::focus::Visibility::Visible,
            )),
            command: crate::app::command::State::with_subject(command::call::Scope::Path(
                field.clone(),
            )),
            ..WindowState::default()
        };
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();

        register_text_command::<crate::text::command::SelectAll>(&mut registry, "Select All");
        registry.set_state_key(
            command::Key::of::<crate::text::command::SelectAll>(),
            command::call::Context::path(window, field.clone()),
            command::State::available(),
        );
        open_menu_surface(&mut state, window, FILE, field.clone());
        tree.set_root(
            widget::panel()
                .key(ROOT)
                .with_child(
                    widget::menu_bar(
                        menu::Bar::new().menu(
                            menu::Menu::new("File").key(FILE).section(
                                menu::Section::new()
                                    .item(menu::Item::text::<crate::text::command::SelectAll>()),
                            ),
                        ),
                    )
                    .key(MENU_BAR),
                )
                .with_child(
                    widget::text_field(text::Buffer::from_text("hello"))
                        .key(CHILD)
                        .with_responder_key(
                            command::Key::of::<crate::text::command::SelectAll>().action(),
                        ),
                ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );

        assert_eq!(state.focused_path(), Some(field));
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Visible);
    }

    #[test]
    fn pointer_opened_menu_preserves_text_area_selection_overlay_after_compose() {
        let window = window::Id::new(1);
        let area = ui::Path::new([ROOT, CHILD]);
        let mut state = WindowState {
            focus: crate::app::state::FocusState::focused(crate::app::state::Focus::new(
                area.clone(),
                ui::focus::Reason::Pointer,
                ui::focus::Visibility::Visible,
            )),
            command: crate::app::command::State::with_subject(command::call::Scope::Path(
                area.clone(),
            )),
            ..WindowState::default()
        };
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();

        register_text_command::<crate::text::command::SelectAll>(&mut registry, "Select All");
        registry.set_state_key(
            command::Key::of::<crate::text::command::SelectAll>(),
            command::call::Context::path(window, area.clone()),
            command::State::available(),
        );
        open_menu_surface(&mut state, window, FILE, area.clone());
        tree.set_root(
            widget::panel()
                .key(ROOT)
                .with_child(
                    widget::menu_bar(
                        menu::Bar::new().menu(
                            menu::Menu::new("File").key(FILE).section(
                                menu::Section::new()
                                    .item(menu::Item::text::<crate::text::command::SelectAll>()),
                            ),
                        ),
                    )
                    .key(MENU_BAR),
                )
                .with_child(
                    widget::text_area(text::Area::new(selected_large_text_area_buffer(3)))
                        .key(CHILD)
                        .with_size(layout::Size::Fill, layout::Size::Fixed(80.0))
                        .with_responder_key(
                            command::Key::of::<crate::text::command::SelectAll>().action(),
                        ),
                ),
        );

        let scene = compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );

        assert_eq!(state.focused_path(), Some(area));
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Visible);
        assert!(
            selection_quad_count(&scene) > 0,
            "menu-title activation that preserves text focus must keep text-area selection visible"
        );
    }

    #[test]
    fn keyboard_opened_menu_focuses_first_enabled_row() {
        let window = window::Id::new(1);
        let field = ui::Path::new([ROOT, CHILD]);
        let row = ui::Path::new([
            ROOT,
            widget::MENU_POPUP,
            ui::Id::structural("__menu_row", 0),
        ]);
        let mut state = WindowState {
            focus: crate::app::state::FocusState::focused(crate::app::state::Focus::new(
                field.clone(),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            command: crate::app::command::State::with_subject(command::call::Scope::Path(
                field.clone(),
            )),
            ..WindowState::default()
        };
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();

        register_text_command::<crate::text::command::SelectAll>(&mut registry, "Select All");
        registry.set_state_key(
            command::Key::of::<crate::text::command::SelectAll>(),
            command::call::Context::path(window, field.clone()),
            command::State::available(),
        );
        tree.set_root(
            widget::panel()
                .key(ROOT)
                .with_child(
                    widget::menu_bar(
                        menu::Bar::new().menu(
                            menu::Menu::new("File").key(FILE).section(
                                menu::Section::new()
                                    .item(menu::Item::text::<crate::text::command::SelectAll>()),
                            ),
                        ),
                    )
                    .key(MENU_BAR),
                )
                .with_child(
                    widget::text_field(text::Buffer::from_text("hello"))
                        .key(CHILD)
                        .with_responder_key(
                            command::Key::of::<crate::text::command::SelectAll>().action(),
                        ),
                ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );
        assert!(state.toggle_menu(FILE, &mut registry, window, command::call::Source::Keyboard));
        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );

        assert_eq!(state.focused_path(), Some(row));
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Visible);
    }

    #[test]
    fn focused_open_menu_row_lowers_focus_background() {
        let window = window::Id::new(1);
        let subject = ui::Path::new([ROOT, CHILD]);
        let row = ui::Path::new([
            ROOT,
            widget::MENU_POPUP,
            ui::Id::structural("__menu_row", 0),
        ]);
        let mut state = WindowState {
            command: crate::app::command::State::with_subject(command::call::Scope::Path(
                subject.clone(),
            )),
            focus: crate::app::state::FocusState::focused(crate::app::state::Focus::new(
                row,
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            ..WindowState::default()
        };
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();

        register_text_command::<crate::text::command::SelectAll>(&mut registry, "Select All");
        registry.set_state_key(
            command::Key::of::<crate::text::command::SelectAll>(),
            command::call::Context::path(window, subject.clone()),
            command::State::available(),
        );
        open_menu_surface(&mut state, window, FILE, subject.clone());
        tree.set_root(
            widget::panel()
                .key(ROOT)
                .with_child(
                    widget::menu_bar(
                        menu::Bar::new().menu(
                            menu::Menu::new("File").key(FILE).section(
                                menu::Section::new()
                                    .item(menu::Item::text::<crate::text::command::SelectAll>()),
                            ),
                        ),
                    )
                    .key(MENU_BAR),
                )
                .with_child(
                    ui::Node::leaf()
                        .key(CHILD)
                        .with_responder_key(
                            command::Key::of::<crate::text::command::SelectAll>().action(),
                        )
                        .with_interactivity(ui::Interactivity::CONTROL),
                ),
        );

        let scene = compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );
        let theme = crate::theme::Theme::default_dark();

        assert!(scene.items().iter().any(|item| {
            matches!(
                item,
                paint::Item::Quad(quad)
                    if quad.style.fill == Some(paint::Fill::Brush(theme.menu().row_hover_tint()))
            )
        }));
    }

    #[test]
    fn active_menu_item_lowers_check_glyph() {
        let window = window::Id::new(1);
        let subject = ui::Path::new([ROOT, CHILD]);
        let mut state = WindowState {
            command: crate::app::command::State::with_subject(command::call::Scope::Path(
                subject.clone(),
            )),
            ..WindowState::default()
        };
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();

        registry.register(
            command::definition::Definition::for_command::<Toggle, command::TestTarget>()
                .with_display("Toggle Preview"),
        );
        registry.set_state_key(
            TOGGLE,
            command::call::Context::path(window, subject.clone()),
            command::State::active(),
        );
        open_menu_surface(&mut state, window, VIEW, subject.clone());
        tree.set_root(
            widget::panel()
                .key(ROOT)
                .with_child(
                    widget::menu_bar(
                        menu::Bar::new().menu(
                            menu::Menu::new("View")
                                .key(VIEW)
                                .section(menu::Section::new().item(menu::Item::key(TOGGLE))),
                        ),
                    )
                    .key(MENU_BAR),
                )
                .with_child(
                    ui::Node::leaf()
                        .key(CHILD)
                        .with_responder_key(TOGGLE.action()),
                ),
        );

        let scene = compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );

        assert!(scene.items().iter().any(|item| {
            matches!(
                item,
                paint::Item::Icon(icon)
                    if icon.icon == crate::Icon::phosphor(crate::icon::Id::new("check"))
            )
        }));
    }

    #[test]
    fn compose_injects_open_submenu_popup() {
        let window = window::Id::new(1);
        let subject = ui::Path::new([ROOT, CHILD]);
        let mut state = WindowState {
            command: crate::app::command::State::with_subject(command::call::Scope::Path(
                subject.clone(),
            )),
            ..WindowState::default()
        };
        let mut registry = command::Registry::new();
        let mut tree = ui::Tree::new();

        registry.register(
            command::definition::Definition::for_command::<Toggle, command::TestTarget>()
                .with_display("Toggle Preview"),
        );
        registry.set_state_key(
            TOGGLE,
            command::call::Context::path(window, subject.clone()),
            command::State::available(),
        );
        open_submenu_surface(&mut state, window, VIEW, PANELS, subject.clone());
        tree.set_root(
            widget::panel()
                .key(ROOT)
                .with_child(
                    widget::menu_bar(
                        menu::Bar::new().menu(
                            menu::Menu::new("View").key(VIEW).section(
                                menu::Section::new().submenu(
                                    menu::Menu::new("Panels").key(PANELS).section(
                                        menu::Section::new().item(menu::Item::key(TOGGLE)),
                                    ),
                                ),
                            ),
                        ),
                    )
                    .key(MENU_BAR),
                )
                .with_child(
                    ui::Node::leaf()
                        .key(CHILD)
                        .with_responder_key(TOGGLE.action()),
                ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(360.0, 220.0),
        );

        let submenu_popup = ui::Path::new([ROOT, widget::MENU_SUBMENU_POPUP]);
        let submenu_row = ui::Path::new([
            ROOT,
            widget::MENU_SUBMENU_POPUP,
            ui::Id::structural("__menu_row", 0),
        ]);
        let top_submenu_row = ui::Path::new([
            ROOT,
            widget::MENU_POPUP,
            ui::Id::structural("__menu_row", 0),
        ]);
        let composition = state.composition.as_ref().expect("composition");

        assert!(composition.layout().find_path(&submenu_popup).is_some());
        assert_eq!(
            composition.action(&submenu_row).map(|route| route.key()),
            Some(TOGGLE.action())
        );
        assert_eq!(
            state.intent(&top_submenu_row),
            Some(ui::Intent::OpenSubmenu(PANELS))
        );
    }
}
