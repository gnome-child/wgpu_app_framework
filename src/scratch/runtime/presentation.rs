use super::super::{
    context as command_context, geometry, layout, scene, session, state, view, window,
};
use super::{Runtime, services::Services, work};

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
        diagnostics.frame.last_scroll_frame.text_surfaces = text_surfaces;
    }

    pub(super) fn apply_layout_feedback(&mut self, window: window::Id, layout: &layout::Layout) {
        for frame in layout.frames() {
            let Some(offset) = frame
                .text_area_layout()
                .and_then(layout::text::TextAreaLayout::resolved_scroll)
            else {
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
}

impl<M: state::State, E: Send + 'static> Runtime<M, E, view::View> {
    pub fn drain(&mut self) -> work::Work {
        work::Work::new(
            self.present_pending(),
            self.requests(),
            self.pending_tasks(),
            self.pending_task_completions(),
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
        )
    }

    pub fn present(&mut self, window: window::Id) -> Option<view::View> {
        if !self.session.contains(window) {
            return None;
        }

        let view = self.view.as_ref()?;
        let mut view = view(self.store.model(), self.view_context(window));
        if self
            .session
            .focused(window)
            .is_some_and(|focus| !view.contains_focus(focus))
        {
            self.session.clear_focus(window);
        }

        let cx = command_context::Context::with_clipboard(&mut self.clipboard);
        let focus = self.session.focused(window);
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
                .chain_for(&mut self.store, focus)
                .with_framework(services);

            view.resolve_commands(&self.registry, &mut chain, &cx);
        }
        if let Some(interaction) = self.session.interaction(window) {
            view.project_interaction(interaction);
        }
        view.project_focus(focus);

        let presented = self.composition.install(window, view).view().clone();
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
        let view = self.present(window)?;
        self.session.clear_redraw_request(window);
        let layout = layout::Layout::compose(&view, size, &mut self.layout);
        self.record_layout_diagnostics(window, &layout);
        self.apply_layout_feedback(window, &layout);
        let canvas_color = self.canvas_color(window);

        Some(scene::Presentation::with_canvas_color(
            window,
            layout,
            canvas_color,
        ))
    }

    pub fn render_pending(
        &mut self,
        mut size_for: impl FnMut(window::Id) -> geometry::Size,
    ) -> Vec<scene::Presentation> {
        let presentations = self.present_pending();
        let mut rendered = Vec::with_capacity(presentations.len());

        for presentation in presentations {
            let window = presentation.window();
            let layout =
                layout::Layout::compose(presentation.view(), size_for(window), &mut self.layout);
            self.record_layout_diagnostics(window, &layout);
            self.apply_layout_feedback(window, &layout);
            rendered.push(scene::Presentation::with_canvas_color(
                window,
                layout,
                self.canvas_color(window),
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
        let composition = self.composition.get(window)?;
        layout::Layout::compose(composition.view(), size, &mut self.layout).hit_test(point)
    }
}
