use super::super::{
    context as command_context, geometry, interaction, layout, response, scene, session, state,
    view, window,
};
use super::{CachedLayout, Runtime, services::Services, work};
use crate::{animation, text};
use std::time::Instant;

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
            .filter_map(layout::frame::Frame::text_area_layout)
            .map(|text_area| text_area.render_surfaces().len())
            .sum();
        let text_area_count = layout
            .frames()
            .iter()
            .filter(|frame| frame.text_area_layout().is_some())
            .count();
        let diagnostics = self.diagnostics.get_mut(window);
        diagnostics.text.add(text);
        diagnostics.scroll.projection_count += text_area_count;
        diagnostics.frame.full_redraws += 1;
        diagnostics.frame.layout_recomposes += 1;
        diagnostics.frame.last_scroll_frame.text_surfaces = text_surfaces;
    }

    pub(super) fn record_layout_reuse(&mut self, window: window::Id) {
        self.diagnostics.get_mut(window).frame.layout_reuses += 1;
    }

    pub(super) fn record_view_rebuild(&mut self, window: window::Id) {
        self.diagnostics.get_mut(window).frame.view_rebuilds += 1;
    }

    pub(super) fn apply_layout_feedback(&mut self, window: window::Id, layout: &layout::Layout) {
        for frame in layout.frames() {
            let Some(offset) = frame.resolved_scroll() else {
                continue;
            };
            let Some(target) = frame.target().cloned() else {
                continue;
            };

            if self.session.resolve_scroll(window, target, offset) {
                self.diagnostics.get_mut(window).scroll.frame_scroll_commits += 1;
            }
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
                .map(|interaction| interaction.scroll().offset(&target))
                .unwrap_or_default();
            let selected = (target == interaction::CommandPalette::results_target())
                .then(|| self.session.command_palette_selected(window))
                .flatten();
            let Some(offset) = layout.active_descendant_reveal_offset(
                &target,
                selected,
                theme.viewport().reveal_margin,
            ) else {
                self.session.clear_scroll_reveal(window, &target);
                continue;
            };

            needs_recompose |= current != offset;
            if self.session.resolve_scroll(window, target, offset) {
                self.diagnostics.get_mut(window).scroll.frame_scroll_commits += 1;
            }
        }

        needs_recompose
    }

    pub(super) fn view_context(&self, window: window::Id) -> view::Context {
        view::Context::new(
            window,
            self.diagnostics.get(window).cloned().unwrap_or_default(),
            self.session
                .interaction(window)
                .cloned()
                .unwrap_or_default(),
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
            .map(|(_, schedule)| *schedule)
            .fold(animation::Schedule::Idle, animation::Schedule::merge)
    }

    pub(crate) fn invalidate_due_animation_frames(&mut self, now: Instant) {
        let due = self
            .animation_schedules
            .iter()
            .filter(|(window, schedule)| self.session.contains(**window) && schedule.is_due(now))
            .map(|(window, _)| *window)
            .collect::<Vec<_>>();

        for window in due {
            self.session
                .request_invalidation(window, response::Invalidation::Paint);
        }
    }

    fn frame_at(&self, now: Instant) -> animation::Frame {
        animation::Frame::new(now)
    }

    fn update_animation_schedule(
        &mut self,
        window: window::Id,
        layout: &layout::Layout,
        now: Instant,
        visual_schedule: animation::Schedule,
    ) {
        let schedule = caret_animation_schedule(layout, now).merge(visual_schedule);
        if schedule == animation::Schedule::Idle {
            self.animation_schedules.remove(&window);
        } else {
            self.animation_schedules.insert(window, schedule);
        }
    }
}

impl<M: state::State, E: Send + 'static> Runtime<M, E, view::View> {
    fn refresh_presented_projection(&mut self, window: window::Id) -> Option<()> {
        let interaction = self.session.interaction(window).cloned();
        let focus = self.session.focused(window);
        let composition = self.composition.get_mut(window)?;
        composition.project_transient_state(interaction.as_ref(), focus);
        Some(())
    }

    fn compose_layout_for_scene(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        theme: &crate::theme::Theme,
        frame: animation::Frame,
    ) -> Option<layout::Layout> {
        self.refresh_presented_projection(window)?;
        let mut layout = {
            let composition = self.composition.get(window)?;
            layout::Layout::compose_composition_with_theme_at(
                composition,
                size,
                &mut self.layout,
                theme,
                frame,
                self.keymap,
            )
        };

        if self.apply_active_descendant_reveals(window, &layout, theme) {
            self.refresh_presented_projection(window)?;
            let composition = self.composition.get(window)?;
            layout = layout::Layout::compose_composition_with_theme_at(
                composition,
                size,
                &mut self.layout,
                theme,
                frame,
                self.keymap,
            );
        }

        Some(layout)
    }

    fn cached_layout(
        &self,
        window: window::Id,
        size: geometry::Size,
        theme: &crate::theme::Theme,
    ) -> Option<layout::Layout> {
        let cached = self.layout_cache.get(&window)?;
        (cached.size == size && cached.theme == *theme).then(|| cached.layout.clone())
    }

    fn cache_layout(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        theme: &crate::theme::Theme,
        layout: &layout::Layout,
    ) {
        self.layout_cache.insert(
            window,
            CachedLayout {
                size,
                theme: theme.clone(),
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
        invalidation: response::Invalidation,
    ) -> Option<layout::Layout> {
        if invalidation == response::Invalidation::Paint
            && let Some(layout) = self.cached_layout(window, size, theme)
        {
            if self.apply_active_descendant_reveals(window, &layout, theme) {
                let layout = self.compose_layout_for_scene(window, size, theme, frame)?;
                self.record_layout_diagnostics(window, &layout);
                self.cache_layout(window, size, theme, &layout);
                return Some(layout);
            }

            self.record_layout_reuse(window);
            return Some(layout);
        }

        let layout = self.compose_layout_for_scene(window, size, theme, frame)?;
        self.record_layout_diagnostics(window, &layout);
        self.cache_layout(window, size, theme, &layout);
        Some(layout)
    }

    pub fn drain(&mut self) -> work::Work {
        work::Work::new(
            self.present_pending(),
            self.requests(),
            self.pending_tasks(),
            self.pending_task_completions(),
            self.animation_schedule(),
        )
    }

    pub fn drain_scenes(
        &mut self,
        size_for: impl FnMut(window::Id) -> geometry::Size,
    ) -> work::RenderWork {
        work::RenderWork::new(
            self.render_pending(size_for),
            self.requests(),
            self.pending_tasks(),
            self.pending_task_completions(),
            self.animation_schedule(),
        )
    }

    pub fn present(&mut self, window: window::Id) -> Option<view::View> {
        if !self.session.contains(window) {
            return None;
        }

        self.layout_cache.remove(&window);
        self.record_view_rebuild(window);
        let view = self.view.as_ref()?;
        let mut view = view(self.store.model(), self.view_context(window));
        if let Some(palette) = self.command_palette_projection(window) {
            view.project_command_palette(palette);
        }
        let mut focus = self.session.focused(window);
        if focus.is_some_and(|focus| focus.target_id().is_some() && !view.contains_focus(focus)) {
            self.clear_focus(window);
            focus = None;
        }

        let command_focus = self.session.command_focus(window);
        let cx = command_context::Context::with_clipboard(&mut self.clipboard);
        {
            let services = Services::new(
                &mut self.timeline,
                &mut self.session,
                &mut self.composition,
                &mut self.diagnostics,
                Some(window),
            );
            let mut chain = self
                .responders
                .chain_for(&mut self.store, command_focus)
                .with_service(services);

            view.resolve_commands(&self.registry, &mut chain, &cx);
        }
        let interaction = self.session.interaction(window).cloned();
        if let Some(interaction) = interaction.as_ref() {
            view.project_surfaces(interaction);
        }
        let (tree, changes) = self.composition.prepare(window, &view);
        let interaction_pruned = !changes.is_empty()
            && self.session.prune_removed_interaction(
                window,
                changes.removed(),
                changes.removed_elements(),
            );
        let interaction = if interaction_pruned {
            self.session.interaction(window).cloned()
        } else {
            interaction
        };
        if let Some(interaction) = interaction.as_ref() {
            view.project_interaction_retained(interaction, &tree);
        }
        if focus.is_some_and(|focus| !view.contains_enabled_focus_retained(&tree, focus)) {
            self.clear_focus(window);
            focus = None;
        }
        view.project_focus_retained(focus, &tree);

        let presented = self
            .composition
            .install_prepared(window, view, tree, changes)
            .view()
            .clone();
        self.session.mark_presented(window, self.revision());

        Some(presented)
    }

    pub fn present_pending(&mut self) -> Vec<view::Presentation> {
        let revision = self.revision();
        let windows = self
            .session
            .windows()
            .iter()
            .filter(|window| {
                window.redraw_requested() || window.presented_revision() != Some(revision)
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

    pub fn render_scene(
        &mut self,
        window: window::Id,
        size: geometry::Size,
    ) -> Option<scene::Presentation> {
        self.render_scene_at(window, size, Instant::now())
    }

    pub(crate) fn render_scene_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        now: Instant,
    ) -> Option<scene::Presentation> {
        let revision = self.revision();
        let window_state = self.session.window(window)?;
        let stale = window_state.presented_revision() != Some(revision)
            || self.composition.get(window).is_none();
        let invalidation = if stale {
            response::Invalidation::Rebuild
        } else {
            window_state
                .invalidation()
                .unwrap_or(response::Invalidation::Paint)
        };
        if invalidation == response::Invalidation::Rebuild {
            self.present(window)?;
        }
        self.session.clear_redraw_request(window);
        let theme = self.active_theme();
        let frame = self.frame_at(now);
        let layout = self.layout_for_scene(window, size, &theme, frame, invalidation)?;
        self.apply_layout_feedback(window, &layout);
        let interaction = self.session.interaction(window).cloned();
        let visual_update = self.visual_animations.update_window(
            window,
            &layout,
            interaction.as_ref(),
            &theme,
            now,
        );
        self.update_animation_schedule(window, &layout, now, visual_update.schedule());
        let canvas_color = self.canvas_color(window);

        Some(scene::Presentation::with_canvas_color_theme_and_visuals(
            window,
            layout,
            canvas_color,
            &theme,
            visual_update.visuals(),
        ))
    }

    pub fn render_pending(
        &mut self,
        mut size_for: impl FnMut(window::Id) -> geometry::Size,
    ) -> Vec<scene::Presentation> {
        let revision = self.revision();
        let windows = self
            .session
            .windows()
            .iter()
            .filter_map(|window| {
                let stale = window.presented_revision() != Some(revision)
                    || self.composition.get(window.id()).is_none();
                let invalidation = if stale {
                    Some(response::Invalidation::Rebuild)
                } else {
                    window.invalidation()
                }?;
                Some((window.id(), invalidation))
            })
            .collect::<Vec<_>>();
        let mut rendered = Vec::with_capacity(windows.len());
        let theme = self.active_theme();
        let now = Instant::now();

        for (window, invalidation) in windows {
            if invalidation == response::Invalidation::Rebuild && self.present(window).is_none() {
                continue;
            }
            self.session.clear_redraw_request(window);
            let frame = self.frame_at(now);
            let Some(layout) =
                self.layout_for_scene(window, size_for(window), &theme, frame, invalidation)
            else {
                continue;
            };
            self.apply_layout_feedback(window, &layout);
            let interaction = self.session.interaction(window).cloned();
            let visual_update = self.visual_animations.update_window(
                window,
                &layout,
                interaction.as_ref(),
                &theme,
                now,
            );
            self.update_animation_schedule(window, &layout, now, visual_update.schedule());
            rendered.push(scene::Presentation::with_canvas_color_theme_and_visuals(
                window,
                layout,
                self.canvas_color(window),
                &theme,
                visual_update.visuals(),
            ));
        }

        rendered
    }

    pub fn hit_test(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
    ) -> Option<layout::hit::Hit> {
        let theme = self.active_theme();
        let frame = animation::Frame::new(Instant::now());
        if let Some(layout) = self.cached_layout(window, size, &theme) {
            self.record_layout_reuse(window);
            return layout.hit_test(point);
        }

        let layout = self.compose_layout_for_scene(window, size, &theme, frame)?;
        self.record_layout_diagnostics(window, &layout);
        self.cache_layout(window, size, &theme, &layout);
        layout.hit_test(point)
    }
}

fn caret_animation_schedule(layout: &layout::Layout, now: Instant) -> animation::Schedule {
    layout
        .frames()
        .iter()
        .filter_map(|frame| focused_text_caret_deadline(frame, now))
        .map(animation::Schedule::At)
        .fold(animation::Schedule::Idle, animation::Schedule::merge)
}

fn focused_text_caret_deadline(frame: &layout::frame::Frame, now: Instant) -> Option<Instant> {
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
        return Some(text::edit::ViewState::new_at(0.0, epoch).next_caret_deadline(now));
    }

    if let Some(text_box) = frame.text_box() {
        if text_box.cursor().is_none() || has_non_empty_text_box_selection(text_box) {
            return None;
        }

        let epoch = text_box.caret_epoch().unwrap_or(now);
        return Some(text::edit::ViewState::new_at(0.0, epoch).next_caret_deadline(now));
    }

    None
}

fn has_non_empty_text_box_selection(text_box: &view::control::TextBox) -> bool {
    text_box
        .selection()
        .is_some_and(|selection| selection.start != selection.end)
}
