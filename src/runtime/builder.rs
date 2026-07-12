use super::super::{
    clipboard::Clipboard,
    command::{self, Command},
    keymap, responder, state, task, theme, view,
};
use super::{Context, Retention, Runtime};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub fn retention(mut self, retention: Retention) -> Self {
        self.store.set_change_limit(retention.change_limit());
        self.timeline.set_snapshot_limit(retention.snapshot_limit());
        self.session.set_draft_limit(retention.draft_limit());
        self
    }

    pub fn with_clipboard(mut self, clipboard: Clipboard) -> Self {
        self.clipboard = super::ConfiguredClipboard::explicit(clipboard);
        self
    }

    pub(crate) fn with_system_clipboard_default(mut self) -> Self {
        self.clipboard.use_system_default();
        self
    }

    pub fn commands(mut self, configure: impl FnOnce(&mut command::Registry)) -> Self {
        configure(&mut self.registry);
        self
    }

    pub fn keymap(mut self, profile: keymap::Profile) -> Self {
        self.keymap = profile;
        self
    }

    pub fn responders(mut self, configure: impl FnOnce(&mut responder::Builder<M>)) -> Self {
        configure(&mut self.responders);
        self
    }

    pub fn observe<C>(
        mut self,
        callback: impl FnMut(&mut M, &C::Output, &mut command::Observation) + 'static,
    ) -> Self
    where
        C: Command,
    {
        self.observers.observe::<C>(callback);
        self
    }

    pub fn started(mut self, callback: impl for<'a> FnMut(&mut Context<'a, M>) + 'static) -> Self {
        self.started = Some(Box::new(callback));
        self
    }

    pub fn theme(mut self, callback: impl Fn(&M) -> theme::Theme + 'static) -> Self {
        self.theme = Some(Box::new(callback));
        self
    }

    pub fn event<E2: Send + 'static>(
        self,
        callback: impl for<'a> FnMut(&mut Context<'a, M>, E2) + 'static,
    ) -> Runtime<M, E2, V> {
        Runtime {
            store: self.store,
            timeline: self.timeline,
            session: self.session,
            composition: self.composition,
            layout: self.layout,
            diagnostics: self.diagnostics,
            clipboard: self.clipboard,
            tasks: task::Queue::default(),
            registry: self.registry,
            keymap: self.keymap,
            observers: self.observers,
            responders: self.responders,
            gesture: self.gesture,
            history_group: self.history_group,
            started: self.started,
            event: Some(Box::new(callback)),
            theme: self.theme,
            view: self.view,
            started_ran: self.started_ran,
            animation_schedules: self.animation_schedules,
            visual_animations: self.visual_animations,
            overlays: self.overlays,
            overlay_capabilities: self.overlay_capabilities,
            layout_cache: self.layout_cache,
            virtual_materializations: self.virtual_materializations,
            virtual_measurements: self.virtual_measurements,
        }
    }

    pub fn view<V2>(
        self,
        callback: impl Fn(&M, view::Context) -> V2 + 'static,
    ) -> Runtime<M, E, V2> {
        Runtime {
            store: self.store,
            timeline: self.timeline,
            session: self.session,
            composition: self.composition,
            layout: self.layout,
            diagnostics: self.diagnostics,
            clipboard: self.clipboard,
            tasks: self.tasks,
            registry: self.registry,
            keymap: self.keymap,
            observers: self.observers,
            responders: self.responders,
            gesture: self.gesture,
            history_group: self.history_group,
            started: self.started,
            event: self.event,
            theme: self.theme,
            view: Some(Box::new(callback)),
            started_ran: self.started_ran,
            animation_schedules: self.animation_schedules,
            visual_animations: self.visual_animations,
            overlays: self.overlays,
            overlay_capabilities: self.overlay_capabilities,
            layout_cache: self.layout_cache,
            virtual_materializations: self.virtual_materializations,
            virtual_measurements: self.virtual_measurements,
        }
    }
}
