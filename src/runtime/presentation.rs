use super::super::{
    context as command_context, diagnostics, geometry, interaction, layout, response, scene,
    session, state, view, window,
};
use super::{CachedLayout, Runtime, services::Services, work};
use crate::{animation, ime, text};
use std::sync::Arc;
use std::time::Instant;

const MAX_VIRTUAL_REFINEMENT_PASSES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameNeed {
    Idle,
    Properties,
    Invalidated(response::effect::Invalidation),
}

struct PreparedFrame {
    window: window::Id,
    revision: state::Revision,
    epoch: window::PresentationEpoch,
    invalidation: response::effect::Invalidation,
    layout: layout::Layout,
    commit: Arc<scene::Commit>,
    drawable: Arc<scene::Commit>,
    residencies: Arc<[scene::Residency]>,
    properties: scene::Properties,
    layers: Vec<crate::overlay::Layer>,
    capabilities: crate::overlay::Capabilities,
    native_popup_dark: bool,
    overlay_schedule: animation::Schedule,
    property_only: bool,
}

struct RealizedFrame {
    presentation: scene::Presentation,
    popup_presentations: Vec<crate::overlay::PopupPresentation>,
    ime_update: ime::Update,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct VirtualProjectionChange {
    any: bool,
    crossed_guard: bool,
}

impl FrameNeed {
    fn immediate_invalidation(self) -> response::effect::Invalidation {
        match self {
            Self::Idle => response::effect::Invalidation::Paint,
            Self::Properties => response::effect::Invalidation::Paint,
            Self::Invalidated(invalidation) => invalidation,
        }
    }

    fn pending(self) -> Option<Self> {
        match self {
            Self::Idle => None,
            pending => Some(pending),
        }
    }

    fn property_only(self) -> bool {
        self == Self::Properties
    }
}

impl PreparedFrame {
    fn realize(self) -> RealizedFrame {
        let mut scene = self
            .drawable
            .compatibility_scene(&self.properties)
            .expect("prepared properties must remain compatible with their immutable commit");
        let mut stack = scene::Stack::new(
            Arc::clone(&self.commit),
            Arc::clone(&self.drawable),
            Arc::clone(&self.residencies),
            self.properties.clone(),
        );
        let ime_target = ime_target_for_layers(self.layout.text_caret_rect(), &self.layers);
        let mut popup_presentations = Vec::new();

        for layer in &self.layers {
            log_overlay_layer_application(layer, self.overlay_schedule);
            match layer.kind() {
                crate::overlay::LayerKind::Live | crate::overlay::LayerKind::RetiringPopup => {
                    let in_frame = layer.backend() == crate::overlay::Backend::InFrame;
                    append_or_present_overlay_layer(
                        self.window,
                        &mut scene,
                        layer,
                        self.capabilities,
                        self.native_popup_dark,
                        &mut popup_presentations,
                        self.invalidation == response::effect::Invalidation::Paint,
                    );
                    if in_frame {
                        stack.push(retained_overlay_layer(
                            layer,
                            scene::MaterialProjection::Source,
                        ));
                    }
                }
                crate::overlay::LayerKind::Ghost => {
                    scene.append_ghost_scene_with_opacity(layer.scene(), layer.opacity());
                    stack.push(retained_overlay_layer(
                        layer,
                        scene::MaterialProjection::WithoutBackdropSampling,
                    ));
                }
            }
        }

        RealizedFrame {
            presentation: scene::Presentation::with_scene(
                self.window,
                self.revision,
                self.epoch,
                self.invalidation,
                self.layout,
                scene,
                stack,
                self.property_only,
            ),
            popup_presentations,
            ime_update: ime::Update::new(self.window, ime_target),
        }
    }
}

fn retained_overlay_layer(
    layer: &crate::overlay::Layer,
    material: scene::MaterialProjection,
) -> scene::Layer {
    scene::Layer::projected(
        Arc::clone(layer.commit()),
        Arc::from([]),
        Arc::clone(layer.properties()),
        geometry::Point::new(0, 0),
        layer.bounds(),
        layer.opacity(),
        layer.force_group_at_full_opacity(),
        material,
    )
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub fn render(&self, window: window::Id) -> Option<V> {
        if !self.session.contains(window) {
            return None;
        }

        self.view
            .as_ref()
            .map(|view| view(self.store.model(), self.view_context(window)))
    }

    pub fn render_all(&self) -> Vec<(window::Id, V)> {
        let Some(view) = self.view.as_ref() else {
            return Vec::new();
        };

        self.session
            .windows()
            .iter()
            .map(|window| {
                (
                    window.id(),
                    view(self.store.model(), self.view_context(window.id())),
                )
            })
            .collect()
    }

    pub(super) fn record_layout_diagnostics(
        &mut self,
        window: window::Id,
        layout: &layout::Layout,
    ) {
        let text = self.layout.take_text_diagnostics();
        let text_surfaces = layout
            .frames()
            .iter()
            .filter_map(layout::Frame::text_area_layout)
            .map(|text_area| text_area.render_surfaces().len())
            .sum();
        let text_area_count = layout
            .frames()
            .iter()
            .filter(|frame| frame.text_area_layout().is_some())
            .count();
        let diagnostics = self.diagnostics.get_mut(window);
        diagnostics.text.add(text);
        diagnostics.scroll.text_area_viewports += text_area_count;
        diagnostics.frame.full_redraws += 1;
        diagnostics.frame.layout_recomposes += 1;
        diagnostics.frame.text_area_render_surfaces = text_surfaces;
    }

    pub(super) fn record_layout_reuse(&mut self, window: window::Id) {
        self.diagnostics.get_mut(window).frame.layout_reuses += 1;
    }

    pub(super) fn record_view_rebuild(&mut self, window: window::Id) {
        self.diagnostics.get_mut(window).frame.view_rebuilds += 1;
    }

    pub(super) fn apply_layout_feedback(&mut self, window: window::Id, layout: &layout::Layout) {
        let Some(interaction) = self.session.interaction(window) else {
            return;
        };
        let mut seen = std::collections::HashSet::new();
        let corrections = layout
            .scroll_projections()
            .iter()
            .filter_map(|projection| {
                let target = projection.target();
                if !seen.insert(target.clone()) {
                    return None;
                }
                let desired = interaction.scroll().desired_offset(target);
                let resolved = layout.resolve_scroll_offset(target, desired);
                (resolved != desired).then(|| (target.clone(), resolved))
            })
            .chain(layout.frames().iter().filter_map(|frame| {
                Some((frame.target()?.clone(), frame.resolved_scroll_correction()?))
            }))
            .collect::<std::collections::HashMap<_, _>>();

        for (target, offset) in corrections {
            if self
                .session
                .request_scroll(window, target, interaction::ScrollUpdate::Geometry(offset))
                .is_some()
            {
                self.diagnostics.get_mut(window).scroll.frame_scroll_commits += 1;
            }
        }
    }

    pub(super) fn install_residency_demand(
        &mut self,
        window: window::Id,
        demand: layout::ResidencyDemand,
    ) {
        debug_assert_eq!(
            self.session
                .interaction(window)
                .map(|interaction| interaction.scroll().desired_offset(demand.target())),
            Some(demand.desired()),
            "residency demand must describe the authoritative desired offset"
        );
        for request in demand.virtual_lists() {
            self.install_virtual_materialization(window, request);
        }
    }

    fn install_virtual_materialization(
        &mut self,
        window: window::Id,
        request: &crate::virtual_list::Request,
    ) {
        let mut materializations = self
            .virtual_materializations
            .get(&window)
            .cloned()
            .unwrap_or_default();
        let current = materializations
            .get(&request.id())
            .cloned()
            .unwrap_or_else(|| {
                crate::virtual_list::Materialization::new(request.range(), Vec::new())
            });
        materializations.insert(request.id(), current.with_runway(request.range()));
        self.virtual_materializations
            .insert(window, materializations);

        let mut measurements = self
            .virtual_measurements
            .get(&window)
            .cloned()
            .unwrap_or_default();
        if let Some(next) = request.measurements() {
            measurements.insert(request.id(), next);
        } else {
            measurements.remove(&request.id());
        }
        if measurements.is_empty() {
            self.virtual_measurements.remove(&window);
        } else {
            self.virtual_measurements.insert(window, measurements);
        }
    }

    pub(super) fn apply_active_descendant_reveals(
        &mut self,
        window: window::Id,
        layout: &layout::Layout,
        theme: &crate::theme::Theme,
    ) -> bool {
        let requests = self
            .session
            .interaction(window)
            .map(|interaction| interaction.scroll().active_descendant_targets())
            .unwrap_or_default();
        let mut needs_recompose = false;

        for target in requests {
            let current = self
                .session
                .interaction(window)
                .map(|interaction| interaction.scroll().desired_offset(&target))
                .unwrap_or_default();
            let Some(offset) = active_descendant_reveal_offset(
                layout,
                &target,
                self.session.command_palette_selected(window),
                theme.viewport().reveal_margin,
            ) else {
                self.session.clear_scroll_reveal(window, &target);
                continue;
            };

            let changed = current != offset;
            let residency_demand = changed
                .then(|| layout.residency_demand(&target, offset))
                .flatten();
            needs_recompose |= changed;
            if self
                .session
                .request_scroll(
                    window,
                    target.clone(),
                    interaction::ScrollUpdate::Geometry(offset),
                )
                .is_some()
            {
                if let Some(demand) = residency_demand {
                    self.install_residency_demand(window, demand);
                }
                self.diagnostics.get_mut(window).scroll.frame_scroll_commits += 1;
            }
        }

        needs_recompose
    }

    pub(super) fn view_context(&self, window: window::Id) -> view::Context {
        view::Context::new(
            window,
            self.diagnostics.get(window).cloned().unwrap_or_default(),
        )
    }

    pub(super) fn canvas_color(&self, window: window::Id) -> scene::Color {
        self.session
            .window(window)
            .map(session::Window::canvas_color)
            .unwrap_or_else(window::Options::default_canvas_color)
    }

    pub(crate) fn animation_schedule(&self) -> animation::Schedule {
        self.animation_schedules
            .iter()
            .filter(|(window, _)| self.session.contains(**window))
            .map(|(_, schedules)| schedules.combined())
            .fold(animation::Schedule::Idle, animation::Schedule::merge)
    }

    pub(crate) fn invalidate_due_animation_frames(&mut self, now: Instant) {
        let due = self
            .animation_schedules
            .iter()
            .filter(|(window, _)| self.session.contains(**window))
            .filter_map(|(window, schedules)| {
                let paint = schedules.paint.is_due(now);
                let properties = schedules.properties.is_due(now);
                (paint || properties).then_some((*window, paint, properties))
            })
            .collect::<Vec<_>>();

        for (window, paint_due, properties_due) in due {
            let hover_delay = std::time::Duration::from_millis(
                self.active_theme().auxiliary_panel().hover_delay_ms,
            );
            let hover_tip_due =
                paint_due && self.session.promote_hover_tip(window, now, hover_delay);
            if paint_due {
                log::debug!(
                    "animation frame due for window {window:?}; requesting {}",
                    if hover_tip_due { "rebuild" } else { "paint" }
                );
                self.session.request_invalidation(
                    window,
                    if hover_tip_due {
                        response::effect::Invalidation::Rebuild
                    } else {
                        response::effect::Invalidation::Paint
                    },
                );
            } else if properties_due {
                log::debug!(
                    "property animation frame due for window {window:?}; requesting property tick"
                );
                self.session.request_property_tick(window);
            }
        }
    }

    fn frame_at(&self, now: Instant) -> animation::Frame {
        animation::Frame::new(now)
    }

    fn frame_need(&self, window: window::Id) -> Option<FrameNeed> {
        let revision = self.revision();
        let window_state = self.session.window(window)?;
        let stale = window_state.projected_revision() != Some(revision)
            || self.composition.get(window).is_none();

        if stale {
            Some(FrameNeed::Invalidated(
                response::effect::Invalidation::Rebuild,
            ))
        } else {
            Some(
                window_state
                    .invalidation()
                    .map(FrameNeed::Invalidated)
                    .unwrap_or_else(|| {
                        if window_state.property_tick_requested() {
                            FrameNeed::Properties
                        } else {
                            FrameNeed::Idle
                        }
                    }),
            )
        }
    }

    fn set_animation_schedule(
        &mut self,
        window: window::Id,
        paint: animation::Schedule,
        properties: animation::Schedule,
    ) {
        let schedules = super::AnimationSchedules { paint, properties };
        if schedules.is_idle() {
            self.animation_schedules.remove(&window);
        } else {
            self.animation_schedules.insert(window, schedules);
        }
    }
}

impl<M: state::State, E: Send + 'static> Runtime<M, E, view::View> {
    fn refresh_requested_projection(&mut self, window: window::Id) -> Option<()> {
        let mut interaction = self.session.interaction(window).cloned();
        if let Some(interaction) = interaction.as_mut() {
            interaction.project_requested_scroll();
        }
        let focus = self.session.focused(window);
        let composition = self.composition.get_mut(window)?;
        composition.project_transient_state(interaction.as_ref(), focus);
        Some(())
    }

    fn interaction_projected_for_layout(
        &self,
        window: window::Id,
        layout: &layout::Layout,
    ) -> Option<interaction::Interaction> {
        let mut interaction = self.session.interaction(window)?.clone();
        let hovered = interaction
            .pointer()
            .location()
            .and_then(|location| layout.hit_test_on_surface(location.point(), location.surface()))
            .and_then(|hit| hit.target().cloned());
        interaction.project_pointer_hover(hovered, false);
        Some(interaction)
    }

    fn update_virtual_projections(
        &mut self,
        window: window::Id,
        layout: &layout::Layout,
    ) -> VirtualProjectionChange {
        let mut next_materializations = self
            .virtual_materializations
            .get(&window)
            .cloned()
            .unwrap_or_default();
        let mut next_measurements = self
            .virtual_measurements
            .get(&window)
            .cloned()
            .unwrap_or_default();
        let mut seen = std::collections::HashSet::new();
        let mut crossed_guard = false;

        for request in layout.virtual_list_requests() {
            seen.insert(request.id());
            crossed_guard |= next_materializations
                .get(&request.id())
                .is_some_and(|current| current.range() != request.range());
            let current = next_materializations
                .get(&request.id())
                .cloned()
                .unwrap_or_else(|| {
                    crate::virtual_list::Materialization::new(request.range(), Vec::new())
                });
            let materialization = if current.preserves(&request.range()) {
                current
            } else {
                current.with_range(request.range())
            };
            next_materializations.insert(request.id(), materialization);
            if let Some(measurements) = request.measurements() {
                next_measurements.insert(request.id(), measurements);
            } else {
                next_measurements.remove(&request.id());
            }
        }
        next_materializations.retain(|id, _| seen.contains(id));
        next_measurements.retain(|id, _| seen.contains(id));

        let materializations_changed = self
            .virtual_materializations
            .get(&window)
            .map_or(!next_materializations.is_empty(), |previous| {
                previous != &next_materializations
            });
        let measurements_changed = self
            .virtual_measurements
            .get(&window)
            .map_or(!next_measurements.is_empty(), |previous| {
                previous != &next_measurements
            });
        if materializations_changed {
            if next_materializations.is_empty() {
                self.virtual_materializations.remove(&window);
            } else {
                self.virtual_materializations
                    .insert(window, next_materializations);
            }
        }
        if measurements_changed {
            if next_measurements.is_empty() {
                self.virtual_measurements.remove(&window);
            } else {
                self.virtual_measurements.insert(window, next_measurements);
            }
        }
        VirtualProjectionChange {
            any: materializations_changed || measurements_changed,
            crossed_guard,
        }
    }

    fn compose_layout_for_scene(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        theme: &crate::theme::Theme,
        frame: animation::Frame,
        popup_surfaces: layout::PopupSurfaces,
    ) -> Option<layout::Layout> {
        self.refresh_requested_projection(window)?;
        let mut layout = {
            let composition = self.composition.get(window)?;
            layout::Layout::compose_composition_with_theme_at(
                composition,
                size,
                &mut self.layout,
                theme,
                frame,
                self.keymap,
                popup_surfaces,
            )
        };

        let mut refinement_passes = 0;
        loop {
            let replenishment_started_at = Instant::now();
            let change = self.update_virtual_projections(window, &layout);
            if change.any {
                if refinement_passes == MAX_VIRTUAL_REFINEMENT_PASSES {
                    log::warn!(
                        "virtual geometry did not converge after {MAX_VIRTUAL_REFINEMENT_PASSES} refinement passes"
                    );
                    break;
                }
                refinement_passes += 1;
                self.present(window)?;
                self.refresh_requested_projection(window)?;
                let composition = self.composition.get(window)?;
                layout = layout::Layout::compose_composition_with_theme_at(
                    composition,
                    size,
                    &mut self.layout,
                    theme,
                    frame,
                    self.keymap,
                    popup_surfaces,
                );
                if change.crossed_guard {
                    self.diagnostics
                        .get_mut(window)
                        .render
                        .record_replenishment_commit(replenishment_started_at.elapsed());
                }
                continue;
            }

            if self.apply_active_descendant_reveals(window, &layout, theme) {
                if refinement_passes == MAX_VIRTUAL_REFINEMENT_PASSES {
                    log::warn!(
                        "virtual reveal did not converge after {MAX_VIRTUAL_REFINEMENT_PASSES} refinement passes"
                    );
                    break;
                }
                refinement_passes += 1;
                self.present(window)?;
                self.refresh_requested_projection(window)?;
                let composition = self.composition.get(window)?;
                layout = layout::Layout::compose_composition_with_theme_at(
                    composition,
                    size,
                    &mut self.layout,
                    theme,
                    frame,
                    self.keymap,
                    popup_surfaces,
                );
                continue;
            }

            break;
        }

        if !layout.scene_residency_is_complete() {
            log::warn!(
                "rejecting scene preparation because virtual residency is incomplete after refinement"
            );
            self.session
                .request_invalidation(window, response::effect::Invalidation::Layout);
            return None;
        }

        Some(layout)
    }

    fn compose_presentation_layout(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        theme: &crate::theme::Theme,
        frame: animation::Frame,
        popup_surfaces: layout::PopupSurfaces,
    ) -> Option<layout::Layout> {
        let started_at = Instant::now();
        let layout = self.compose_layout_for_scene(window, size, theme, frame, popup_surfaces)?;
        self.diagnostics
            .get_mut(window)
            .pipeline
            .record_presentation_layout(started_at.elapsed());
        Some(layout)
    }

    fn cached_layout(
        &self,
        window: window::Id,
        size: geometry::Size,
        theme: &crate::theme::Theme,
        popup_surfaces: layout::PopupSurfaces,
    ) -> Option<layout::Layout> {
        let cached = self.layout_cache.get(&window)?;
        (cached.size == size && cached.theme == *theme && cached.popup_surfaces == popup_surfaces)
            .then(|| cached.layout.clone())
    }

    fn cache_layout(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        theme: &crate::theme::Theme,
        popup_surfaces: layout::PopupSurfaces,
        layout: &layout::Layout,
    ) {
        self.layout_cache.insert(
            window,
            CachedLayout {
                size,
                theme: theme.clone(),
                popup_surfaces,
                layout: layout.clone(),
            },
        );
    }

    fn layout_for_scene(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        theme: &crate::theme::Theme,
        frame: animation::Frame,
        invalidation: response::effect::Invalidation,
        popup_surfaces: layout::PopupSurfaces,
    ) -> Option<layout::Layout> {
        if invalidation == response::effect::Invalidation::Paint
            && let Some(layout) = self.cached_layout(window, size, theme, popup_surfaces)
        {
            if self.apply_active_descendant_reveals(window, &layout, theme) {
                log::debug!(
                    "paint-only scene render for window {:?} promoted to layout by reveal feedback",
                    window
                );
                let layout =
                    self.compose_presentation_layout(window, size, theme, frame, popup_surfaces)?;
                self.apply_layout_feedback(window, &layout);
                self.record_layout_diagnostics(window, &layout);
                self.cache_layout(window, size, theme, popup_surfaces, &layout);
                return Some(layout);
            }

            self.record_layout_reuse(window);
            return Some(layout);
        }

        let layout =
            self.compose_presentation_layout(window, size, theme, frame, popup_surfaces)?;
        self.apply_layout_feedback(window, &layout);
        self.record_layout_diagnostics(window, &layout);
        self.cache_layout(window, size, theme, popup_surfaces, &layout);
        Some(layout)
    }

    #[cfg(test)]
    pub(crate) fn drain(&mut self) -> work::Work {
        work::Work::new(
            self.present_pending(),
            self.requests(),
            self.session.take_cursor_updates(),
            self.pending_tasks(),
            self.pending_task_completions(),
            self.animation_schedule(),
        )
    }

    pub(crate) fn drain_scenes(
        &mut self,
        size_for: impl FnMut(window::Id) -> geometry::Size,
    ) -> work::RenderWork {
        let (presentations, popup_presentations, ime_updates) = self.render_pending(size_for);
        work::RenderWork::new(
            presentations,
            popup_presentations,
            ime_updates,
            self.requests(),
            self.session.take_cursor_updates(),
            self.pending_tasks(),
            self.pending_task_completions(),
            self.animation_schedule(),
            self.pending_redraw_windows(),
        )
    }

    pub(crate) fn drain_immediate(&mut self) -> work::ImmediateWork {
        work::ImmediateWork::new(
            self.requests(),
            self.session.take_cursor_updates(),
            self.pending_tasks(),
            self.pending_task_completions(),
            self.animation_schedule(),
            self.pending_redraw_windows(),
        )
    }

    pub(crate) fn drain_window_scene(
        &mut self,
        window: window::Id,
        size: geometry::Size,
    ) -> work::RenderWork {
        let mut presentations = Vec::new();
        let mut popup_presentations = Vec::new();
        let mut ime_updates = Vec::new();

        if let Some(need) = self.frame_need(window) {
            let theme = self.active_theme();
            if let Some(prepared) =
                self.prepare_pending_frame(window, size, need, &theme, Instant::now())
            {
                let realized = prepared.realize();
                presentations.push(realized.presentation);
                popup_presentations.extend(realized.popup_presentations);
                ime_updates.push(realized.ime_update);
            }
        }

        let popup_presentations = (!presentations.is_empty()).then_some(popup_presentations);
        work::RenderWork::new(
            presentations,
            popup_presentations,
            ime_updates,
            self.requests(),
            self.session.take_cursor_updates(),
            self.pending_tasks(),
            self.pending_task_completions(),
            self.animation_schedule(),
            self.pending_redraw_windows(),
        )
    }

    fn pending_redraw_windows(&self) -> Vec<window::Id> {
        self.session
            .windows()
            .iter()
            .filter(|window| window.redraw_requested())
            .map(session::Window::id)
            .collect()
    }

    pub(crate) fn present(&mut self, window: window::Id) -> Option<view::View> {
        self.present_with_virtual_pin(window, None)
    }

    pub(in crate::runtime) fn present_with_virtual_pin(
        &mut self,
        window: window::Id,
        virtual_pin: Option<(crate::interaction::Id, crate::virtual_list::Key)>,
    ) -> Option<view::View> {
        if !self.session.contains(window) {
            log::debug!("skipping present for unknown window {window:?}");
            return None;
        }

        log::debug!("rebuilding view projection for window {window:?}");
        let rebuild_started_at = Instant::now();
        self.layout_cache.remove(&window);
        self.record_view_rebuild(window);
        self.refresh_virtual_pins(window);
        if let Some((list, key)) = virtual_pin {
            let materializations = self.virtual_materializations.get(&window)?.clone();
            let materialization = materializations.get(&list)?.with_pin(key);
            let mut next = materializations;
            next.insert(list, materialization);
            self.virtual_materializations.insert(window, next);
        }
        let view = self.view.as_ref()?;
        let mut view = view(self.store.model(), self.view_context(window));
        let interaction = self.session.interaction(window).cloned();
        if let Some(interaction) = interaction.as_ref() {
            view.project_table_widths(interaction.tables());
        }
        if let Some(palette) = self.command_palette_projection(window) {
            view.project_command_palette(palette);
        }
        if let Some(menu) = self.context_menu_projection(window) {
            view.project_context_menu(menu);
        }
        let selectable_virtual_lists = view.selectable_virtual_lists();
        self.session
            .reconcile_virtual_selections(window, &selectable_virtual_lists);
        let virtual_materializations = self
            .virtual_materializations
            .get(&window)
            .cloned()
            .unwrap_or_default();
        let virtual_measurements = self
            .virtual_measurements
            .get(&window)
            .cloned()
            .unwrap_or_default();
        view.materialize_virtual_lists(&virtual_materializations, &virtual_measurements);
        let virtual_selections = self.session.virtual_selection_snapshot(window);
        view.project_virtual_selections(&virtual_selections);
        if let Some(interaction) = interaction.as_ref() {
            view.project_active_table_cells(interaction, &virtual_selections);
            view.project_input_feedback(interaction);
        }
        let window_feedback = self
            .session
            .window(window)
            .and_then(session::Window::feedback)
            .map(|(severity, text)| (severity, text.to_owned()));
        view.project_feedback(window_feedback);
        let mut focus = self.session.focused(window);
        if focus.is_some_and(|focus| focus.target_id().is_some() && !view.contains_focus(focus)) {
            log::debug!("clearing stale focus before command resolution for window {window:?}");
            self.clear_focus(window);
            focus = None;
        }

        let command_focus = self.session.command_focus(window);
        let command_scope = self
            .context_menu_scope(window)
            .or_else(|| self.session.command_palette_captured_scope(window))
            .unwrap_or_else(|| session::CommandScope::focused(command_focus));
        let cx = command_context::Context::with_clipboard(&mut self.clipboard);
        {
            let services = Services::new(
                &mut self.timeline,
                &mut self.session,
                &mut self.composition,
                Some(window),
                command_scope,
            );
            let mut chain = self
                .responders
                .chain_for_scope(&mut self.store, command_scope.routing())
                .with_service(services);

            view.resolve_commands(&self.registry, &mut chain, &cx);
        }
        if view.has_standard_menu_bar() {
            let live_scope = session::CommandScope::focused(command_focus);
            let services = Services::new(
                &mut self.timeline,
                &mut self.session,
                &mut self.composition,
                Some(window),
                live_scope,
            );
            let mut chain = self
                .responders
                .chain_for_scope(&mut self.store, live_scope.routing())
                .with_service(services);
            view.resolve_standard_menu_extensions(&self.registry, &mut chain, &cx);
            let population = self.registry.population();
            let bar = population.standard_bar(self.keymap.platform(), &mut chain, &cx);
            view.project_standard_menu_bar(&bar);
        }
        if let Some(interaction) = interaction.as_ref() {
            view.project_surfaces(interaction);
        }
        let reconciliation_started_at = Instant::now();
        let (mut tree, mut changes) = self.composition.prepare(window, &view);
        let hover_tip = self
            .session
            .interaction(window)
            .filter(|interaction| interaction.pointer().hover_tip_visible())
            .and_then(|interaction| {
                interaction
                    .pointer()
                    .hovered()
                    .cloned()
                    .zip(interaction.pointer().hover_tip_anchor())
            });
        if let Some((target, pointer_anchor)) = hover_tip
            && view.project_hover_tip(
                &tree,
                &target,
                pointer_anchor,
                self.presented_geometry
                    .get(&window)
                    .and_then(|presented| presented.layout.overflow_tip_for_target(&target))
                    .map(str::to_owned),
            )
        {
            (tree, changes) = self.composition.prepare(window, &view);
        }
        if !changes.is_empty() {
            log::debug!(
                "composition changed for window {:?}: added={}, changed={}, removed={}, removed_elements={}",
                window,
                changes.added().len(),
                changes.changed().len(),
                changes.removed().len(),
                changes.removed_elements().len()
            );
        }
        let pruned = if changes.is_empty() {
            crate::interaction::Pruned::default()
        } else {
            self.session.prune_removed_interaction(
                window,
                changes.removed(),
                changes.removed_elements(),
                changes.removed_table_cells(),
            )
        };
        if pruned.capture_removed() {
            self.cancel_pointer_gesture(window);
        }
        let interaction = if pruned.changed() {
            self.session.interaction(window).cloned()
        } else {
            interaction
        };
        if pruned.changed() {
            log::debug!(
                "pruned interaction state for removed composition nodes in window {window:?}"
            );
        }
        if let Some(interaction) = interaction.as_ref() {
            view.project_layout_interaction_retained(interaction, &tree);
        }
        if focus.is_some_and(|focus| !view.contains_enabled_focus_retained(&tree, focus)) {
            log::debug!(
                "clearing stale focus after composition reconciliation for window {window:?}"
            );
            self.clear_focus(window);
            focus = None;
        }
        view.project_focus_retained(focus, &tree);

        let presented = self
            .composition
            .install_prepared(window, view, tree, changes)
            .view()
            .clone();
        self.diagnostics
            .get_mut(window)
            .pipeline
            .record_composition_reconciliation(reconciliation_started_at.elapsed());
        self.session.mark_projected(window, self.revision());
        self.diagnostics
            .get_mut(window)
            .pipeline
            .record_view_rebuild(rebuild_started_at.elapsed());

        Some(presented)
    }

    #[cfg(test)]
    pub(crate) fn focus_virtual_row(
        &mut self,
        window: window::Id,
        list: crate::interaction::Id,
        key: crate::virtual_list::Key,
        focus: session::Focus,
    ) -> bool {
        let Some(view) = self.present_with_virtual_pin(window, Some((list, key))) else {
            return false;
        };
        if !view.contains_focus(focus) {
            return false;
        }

        self.focus(window, focus)
    }

    fn refresh_virtual_pins(&mut self, window: window::Id) {
        let focus = self.session.focused(window);
        let Some(interaction) = self.session.interaction(window) else {
            return;
        };
        let targets = interaction
            .pointer()
            .capture()
            .map(|capture| capture.target().clone())
            .into_iter()
            .chain(interaction.text_input().target().cloned())
            .collect::<Vec<_>>();
        let mut pins = self
            .composition
            .get(window)
            .map(|composition| composition.virtual_list_pins(focus, &targets))
            .unwrap_or_default();
        if let Some(owner) = interaction
            .open_menu()
            .and_then(crate::interaction::Menu::context_owner)
            && let Some(row) = self
                .composition
                .get(window)
                .and_then(|composition| composition.provided_row_for_node(owner))
        {
            pins.entry(row.list()).or_default().push(row.key());
        }
        for target in interaction.scroll().active_descendant_targets() {
            let Some(list) = target.element_id() else {
                continue;
            };
            if let Some(active) = self
                .session
                .selection(window, list)
                .and_then(crate::selection::Selection::active)
            {
                pins.entry(list).or_default().push(active);
            }
        }
        let Some(materializations) = self.virtual_materializations.get(&window).cloned() else {
            return;
        };
        let next = materializations
            .into_iter()
            .map(|(id, materialization)| {
                let keys = pins.get(&id).cloned().unwrap_or_default();
                (id, materialization.with_pins(keys))
            })
            .collect();
        self.virtual_materializations.insert(window, next);
    }

    #[cfg(test)]
    pub(crate) fn present_pending(&mut self) -> Vec<view::Presentation> {
        let revision = self.revision();
        let windows = self
            .session
            .windows()
            .iter()
            .filter(|window| {
                window.redraw_requested() || window.projected_revision() != Some(revision)
            })
            .map(session::Window::id)
            .collect::<Vec<_>>();

        windows
            .into_iter()
            .filter_map(|window| {
                let view = self.present(window)?;
                self.session.clear_redraw_request(window);
                Some(view::Presentation::new(window, view))
            })
            .collect()
    }

    fn prepare_frame(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        need: FrameNeed,
        theme: &crate::theme::Theme,
        now: Instant,
        capabilities: crate::overlay::Capabilities,
    ) -> Option<PreparedFrame> {
        let frame_started_at = Instant::now();
        let candidate_work_before =
            diagnostics::CandidateWork::snapshot(self.diagnostics.get(window));
        let invalidation = need.immediate_invalidation();
        let property_only = need.property_only();
        let revision = self.revision();
        let rebuilt = invalidation == response::effect::Invalidation::Rebuild;
        let rebuild_started_at = Instant::now();
        if invalidation == response::effect::Invalidation::Rebuild {
            self.present(window)?;
        }
        let rebuild_elapsed = rebuild_started_at.elapsed();
        self.session.clear_redraw_request(window);

        let frame = self.frame_at(now);
        let layout_started_at = Instant::now();
        let popup_surfaces = if capabilities.native_popups_supported() {
            layout::PopupSurfaces::Native
        } else {
            layout::PopupSurfaces::InFrame
        };
        let layout =
            self.layout_for_scene(window, size, theme, frame, invalidation, popup_surfaces)?;
        let layout_elapsed = layout_started_at.elapsed();
        let epoch = self.session.window(window)?.requested_presentation_epoch();
        let assembly_started_at = Instant::now();
        let interaction = self.interaction_projected_for_layout(window, &layout);
        let canvas_color = self.canvas_color(window);
        let (commit, drawable, residencies, properties, entries, scene_stats, visual_schedule) =
            if property_only {
                let visual_update = self.visual_animations.update_window(
                    window,
                    &layout,
                    interaction.as_ref(),
                    theme,
                    now,
                );
                let Some((commit, drawable, residencies, properties, entries, scene_stats)) =
                    self.scene.tick_properties(
                        window,
                        &layout,
                        visual_update.visuals(),
                        interaction.as_ref(),
                    )
                else {
                    return self.prepare_frame(
                        window,
                        size,
                        FrameNeed::Invalidated(response::effect::Invalidation::Layout),
                        theme,
                        now,
                        capabilities,
                    );
                };
                (
                    commit,
                    drawable,
                    residencies,
                    properties,
                    entries,
                    scene_stats,
                    visual_update.schedule(),
                )
            } else {
                let visual_update = self.visual_animations.update_window(
                    window,
                    &layout,
                    interaction.as_ref(),
                    theme,
                    now,
                );
                let (commit, drawable, residencies, properties, entries, scene_stats) =
                    self.scene.paint(
                        window,
                        &layout,
                        canvas_color,
                        theme,
                        visual_update.visuals(),
                        interaction.as_ref(),
                    );
                (
                    commit,
                    drawable,
                    residencies,
                    properties,
                    entries,
                    scene_stats,
                    visual_update.schedule(),
                )
            };
        let overlay_update =
            self.overlays
                .update_window(window, entries, theme.overlay(), capabilities, now);
        let (layers, overlay_schedule) = overlay_update.into_parts();
        let property_schedule = caret_animation_schedule(&layout, now);
        let paint_schedule = visual_schedule.merge(overlay_schedule).merge(
            self.session
                .hover_tip_deadline(
                    window,
                    std::time::Duration::from_millis(theme.auxiliary_panel().hover_delay_ms),
                )
                .map(animation::Schedule::At)
                .unwrap_or(animation::Schedule::Idle),
        );
        self.set_animation_schedule(window, paint_schedule, property_schedule);
        let assembly_elapsed = assembly_started_at.elapsed();
        for layer in layers
            .iter()
            .filter(|layer| layer.backend() == crate::overlay::Backend::NativePopup)
        {
            log::debug!(
                target: "wgpu_l3::native_popup",
                "first-present stage=runtime-prepared popup={:?} parent={:?} elapsed_us={} frame_us={} rebuild_us={} rebuilt={} layout_us={} scene_us={}",
                layer.id(),
                window,
                layer.lifecycle_epoch().elapsed().as_micros(),
                frame_started_at.elapsed().as_micros(),
                rebuild_elapsed.as_micros(),
                rebuilt,
                layout_elapsed.as_micros(),
                assembly_elapsed.as_micros()
            );
        }
        let candidate_work = diagnostics::CandidateWork::since(
            candidate_work_before,
            self.diagnostics.get(window),
            scene_stats,
        );
        let diagnostics = self.diagnostics.get_mut(window);
        diagnostics.pipeline.record_scene_assembly(assembly_elapsed);
        diagnostics.pipeline.record_frame_prepared();
        diagnostics.render.record_scene_projection(scene_stats);
        diagnostics.scroll.record_candidate_constructed(
            epoch,
            properties.serial().value(),
            Instant::now(),
            candidate_work,
        );

        Some(PreparedFrame {
            window,
            revision,
            epoch,
            invalidation,
            layout,
            commit,
            drawable,
            residencies,
            properties,
            layers,
            capabilities,
            native_popup_dark: theme.variant() == crate::theme::Variant::Dark,
            overlay_schedule,
            property_only,
        })
    }

    fn prepare_pending_frame(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        need: FrameNeed,
        theme: &crate::theme::Theme,
        now: Instant,
    ) -> Option<PreparedFrame> {
        self.prepare_frame(window, size, need, theme, now, self.overlay_capabilities)
    }

    pub fn render_scene(
        &mut self,
        window: window::Id,
        size: geometry::Size,
    ) -> Option<scene::Presentation> {
        self.render_scene_at(window, size, Instant::now())
    }

    #[cfg(test)]
    pub(crate) fn show_scene(
        &mut self,
        window: window::Id,
        size: geometry::Size,
    ) -> Option<scene::Presentation> {
        let presentation = self.render_scene(window, size)?;
        self.finish_render_report(
            window,
            presentation.epoch(),
            presentation.invalidation(),
            presentation.layout(),
            presentation.stack(),
            presentation.property_only(),
            crate::diagnostics::RenderReport::new(
                std::time::Duration::ZERO,
                std::time::Duration::ZERO,
                Instant::now(),
            ),
        );
        Some(presentation)
    }

    #[cfg(test)]
    pub(crate) fn render_scene_after_overlay_fade(
        &mut self,
        window: window::Id,
        size: geometry::Size,
    ) -> Option<scene::Presentation> {
        let settled_at = Instant::now();
        let enter =
            std::time::Duration::from_millis(self.active_theme().overlay().enter_fade_ms + 1);
        let started_at = settled_at.checked_sub(enter).unwrap_or(settled_at);

        self.render_scene_at(window, size, started_at)?;
        self.render_scene_at(window, size, settled_at)
    }

    #[cfg(test)]
    pub(crate) fn show_scene_after_overlay_fade(
        &mut self,
        window: window::Id,
        size: geometry::Size,
    ) -> Option<scene::Presentation> {
        let presentation = self.render_scene_after_overlay_fade(window, size)?;
        self.finish_render_report(
            window,
            presentation.epoch(),
            presentation.invalidation(),
            presentation.layout(),
            presentation.stack(),
            presentation.property_only(),
            crate::diagnostics::RenderReport::new(
                std::time::Duration::ZERO,
                std::time::Duration::ZERO,
                Instant::now(),
            ),
        );
        Some(presentation)
    }

    pub(crate) fn render_scene_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        now: Instant,
    ) -> Option<scene::Presentation> {
        let need = self.frame_need(window)?;
        let theme = self.active_theme();
        let prepared = self.prepare_frame(
            window,
            size,
            need,
            &theme,
            now,
            crate::overlay::Capabilities::in_frame_only(),
        )?;

        Some(prepared.realize().presentation)
    }

    #[cfg(test)]
    pub(crate) fn show_scene_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        now: Instant,
    ) -> Option<scene::Presentation> {
        let presentation = self.render_scene_at(window, size, now)?;
        self.finish_render_report(
            window,
            presentation.epoch(),
            presentation.invalidation(),
            presentation.layout(),
            presentation.stack(),
            presentation.property_only(),
            crate::diagnostics::RenderReport::new(
                std::time::Duration::ZERO,
                std::time::Duration::ZERO,
                Instant::now(),
            ),
        );
        Some(presentation)
    }

    pub(crate) fn render_pending(
        &mut self,
        mut size_for: impl FnMut(window::Id) -> geometry::Size,
    ) -> (
        Vec<scene::Presentation>,
        Option<Vec<crate::overlay::PopupPresentation>>,
        Vec<ime::Update>,
    ) {
        let windows = self
            .session
            .windows()
            .iter()
            .filter_map(|window| {
                let need = self.frame_need(window.id())?.pending()?;
                Some((window.id(), need))
            })
            .collect::<Vec<_>>();
        let mut rendered = Vec::with_capacity(windows.len());
        let mut popup_presentations = Vec::new();
        let mut ime_updates = Vec::with_capacity(windows.len());
        let theme = self.active_theme();
        let now = Instant::now();

        for (window, need) in windows {
            let Some(prepared) =
                self.prepare_pending_frame(window, size_for(window), need, &theme, now)
            else {
                continue;
            };
            let realized = prepared.realize();
            rendered.push(realized.presentation);
            popup_presentations.extend(realized.popup_presentations);
            ime_updates.push(realized.ime_update);
        }

        let popup_presentations = (!rendered.is_empty()).then_some(popup_presentations);

        (rendered, popup_presentations, ime_updates)
    }

    #[cfg(test)]
    pub(crate) fn hit_test(
        &mut self,
        window: window::Id,
        _size: geometry::Size,
        point: geometry::Point,
    ) -> Option<layout::Hit> {
        self.hit_test_on_surface(window, _size, point, crate::popup::Surface::Parent)
    }

    pub(crate) fn hit_test_on_surface(
        &mut self,
        window: window::Id,
        _size: geometry::Size,
        point: geometry::Point,
        surface: crate::popup::Surface,
    ) -> Option<layout::Hit> {
        self.presented_geometry
            .get(&window)?
            .hit_test_on_surface(point, surface)
    }
}

fn ime_target_for_layers(
    parent_caret: Option<geometry::Rect>,
    layers: &[crate::overlay::Layer],
) -> Option<ime::Target> {
    layers
        .iter()
        .rev()
        .find_map(|layer| {
            (layer.kind() == crate::overlay::LayerKind::Live
                && layer.backend() == crate::overlay::Backend::NativePopup)
                .then(|| {
                    let area = layer.text_caret_rect()?;
                    Some(ime::Target::popup(layer.id(), area, layer.bounds()))
                })
                .flatten()
        })
        .or_else(|| parent_caret.map(ime::Target::parent))
}

fn active_descendant_reveal_offset(
    layout: &layout::Layout,
    target: &interaction::Target,
    selected_palette_index: Option<usize>,
    margin: i32,
) -> Option<interaction::ScrollOffset> {
    let palette_results_target = interaction::CommandPalette::results_target();
    let table_scroll = layout.is_table_scroll_target(target);
    let mut palette_row = 0_usize;

    layout.reveal_offset_for_descendant(target, margin, |frame| {
        if target == &palette_results_target {
            if !frame.is_palette_row() {
                return false;
            }

            let selected = selected_palette_index == Some(palette_row);
            palette_row = palette_row.saturating_add(1);
            return selected;
        }

        if table_scroll {
            return frame.table_cell().is_some() && frame.is_active_item();
        }

        if frame.provided_row().is_some() {
            frame.is_active_item()
        } else {
            frame.is_selected()
        }
    })
}

fn caret_animation_schedule(layout: &layout::Layout, now: Instant) -> animation::Schedule {
    layout
        .frames()
        .iter()
        .filter_map(|frame| focused_text_caret_deadline(frame, now))
        .map(animation::Schedule::At)
        .fold(animation::Schedule::Idle, animation::Schedule::merge)
}

fn append_overlay_layer(scene: &mut scene::Scene, layer: &crate::overlay::Layer) {
    let resolved = layer
        .scene()
        .resolve_material(scene::MaterialRenderer::InFrame, &[]);
    if layer.force_group_at_full_opacity() {
        scene.append_scene_with_forced_group(resolved.scene(), layer.opacity());
    } else {
        scene.append_scene_with_opacity(resolved.scene(), layer.opacity());
    }
}

fn append_or_present_overlay_layer(
    window: window::Id,
    scene: &mut scene::Scene,
    layer: &crate::overlay::Layer,
    capabilities: crate::overlay::Capabilities,
    native_popup_dark: bool,
    popup_presentations: &mut Vec<crate::overlay::PopupPresentation>,
    paint_only: bool,
) {
    match layer.backend() {
        crate::overlay::Backend::InFrame => append_overlay_layer(scene, layer),
        crate::overlay::Backend::NativePopup if capabilities.native_popups_supported() => {
            log::debug!(
                target: "wgpu_l3::native_popup",
                "first-present stage=runtime-realized popup={:?} parent={:?} elapsed_us={}",
                layer.id(),
                window,
                layer.lifecycle_epoch().elapsed().as_micros()
            );
            let local = layer.scene().native_popup_request(layer.bounds());
            let popup_scene = local.scene().clone();
            popup_presentations.push(crate::overlay::PopupPresentation::new(
                window,
                layer.id(),
                layer.bounds(),
                layer.placement(),
                Arc::clone(layer.commit()),
                Arc::clone(layer.properties()),
                popup_scene,
                layer.opacity(),
                layer.fade(),
                crate::overlay::PopupMaterial::NativeWindow {
                    dark: native_popup_dark,
                    tint: local.accent_tint(),
                    preference: layer.popup_material_preference(),
                },
                layer.popup_border(),
                layer.lifecycle_epoch(),
                paint_only,
                layer.kind(),
                layer.context_fingerprint(),
                layer.accepts_input(),
            ));
        }
        crate::overlay::Backend::NativePopup => {}
    }
}

fn log_overlay_layer_application(layer: &crate::overlay::Layer, schedule: animation::Schedule) {
    if layer.kind() != crate::overlay::LayerKind::Live {
        return;
    }

    let Some(state) = layer.state() else {
        return;
    };
    let elapsed_ms = layer.elapsed().as_millis();
    log::debug!(
        target: "wgpu_l3::overlay::fade",
        "overlay fade frame={} elapsed_ms={} sampled_alpha={:.6} applied_alpha={:.6} state={:?} schedule={:?} force_group={} demotion={}",
        layer.frame_number(),
        elapsed_ms,
        layer.opacity(),
        layer.opacity(),
        state,
        schedule,
        layer.force_group_at_full_opacity(),
        layer.demotion_marker(),
    );
    if layer.demotion_marker() {
        log::debug!(
            target: "wgpu_l3::overlay::fade",
            "overlay fade demotion frame={} elapsed_ms={} alpha={:.6}",
            layer.frame_number(),
            elapsed_ms,
            layer.opacity(),
        );
    }
}

fn focused_text_caret_deadline(frame: &layout::Frame, now: Instant) -> Option<Instant> {
    if !frame.is_focused() {
        return None;
    }

    if let Some(text_area) = frame.text_area() {
        let area = text_area.area_model();
        if !area.paints_caret()
            || text_area
                .buffer()
                .has_selection_for_state(text_area.state())
        {
            return None;
        }

        let epoch = text_area.caret_epoch().unwrap_or(now);
        return Some(text::view::ViewState::new_at(0.0, epoch).next_caret_deadline(now));
    }

    if let Some(text_box) = frame.text_box() {
        if text_box.cursor().is_none() || has_non_empty_text_box_selection(text_box) {
            return None;
        }

        let epoch = text_box.caret_epoch().unwrap_or(now);
        return Some(text::view::ViewState::new_at(0.0, epoch).next_caret_deadline(now));
    }

    None
}

fn has_non_empty_text_box_selection(text_box: &view::TextBox) -> bool {
    text_box
        .selection()
        .is_some_and(|selection| selection.start != selection.end)
}
