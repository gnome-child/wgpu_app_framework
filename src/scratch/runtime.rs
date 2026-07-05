use std::{
    any::{Any, TypeId},
    time::{Duration, Instant},
};

use crate::text;

use super::{
    clipboard::Clipboard,
    command::{self, Command},
    composition, context as command_context, diagnostics,
    diagnostics::Diagnostics,
    document,
    error::Error,
    geometry, input, interaction, layout, responder,
    response::{self, AnyResponse, Response},
    scene, session,
    state::{self, Store},
    target::Target,
    task,
    timeline::{self, Timeline},
    view, window,
};

type Started<M> = Box<dyn for<'a> FnMut(&mut Context<'a, M>)>;
type Event<M, E> = Box<dyn for<'a> FnMut(&mut Context<'a, M>, E)>;
type ViewCallback<M, V> = Box<dyn Fn(&M, view::Context) -> V>;
const HISTORY_GROUP_COALESCE_WINDOW: Duration = Duration::from_millis(1000);

struct AnyCommandTransaction {
    response: AnyResponse,
    changed_state: bool,
    effect: response::Effect,
}

struct Services<'a, M: state::State> {
    timeline: &'a mut Timeline<M>,
    session: &'a mut session::Session,
    composition: &'a mut composition::Store,
    diagnostics: &'a mut diagnostics::Store,
    window: Option<window::Id>,
}

struct Gesture<M: state::State> {
    target: interaction::Target,
    initial: M,
    changed_automatic: bool,
}

struct ActiveHistoryGroup {
    group: command::HistoryGroup,
    recorded_at: Instant,
}

#[derive(Clone)]
pub struct Snapshot<M: state::State> {
    state: state::Snapshot<M>,
    session: session::Snapshot,
}

pub trait Persistence<M: state::State> {
    type Error;

    fn save(&mut self, snapshot: &Snapshot<M>) -> Result<(), Self::Error>;

    fn load(&mut self) -> Result<Snapshot<M>, Self::Error>;
}

pub struct Runtime<M: state::State, E: Send + 'static = (), V = ()> {
    store: Store<M>,
    timeline: Timeline<M>,
    session: session::Session,
    composition: composition::Store,
    layout: layout::Engine,
    diagnostics: diagnostics::Store,
    clipboard: Clipboard,
    tasks: task::Queue<E>,
    registry: command::Registry,
    observers: command::Observers<M>,
    responders: responder::Builder<M>,
    gesture: Option<Gesture<M>>,
    history_group: Option<ActiveHistoryGroup>,
    started: Option<Started<M>>,
    event: Option<Event<M, E>>,
    view: Option<ViewCallback<M, V>>,
    started_ran: bool,
}

pub struct Context<'a, M: state::State> {
    store: &'a mut Store<M>,
    timeline: &'a mut Timeline<M>,
    session: &'a mut session::Session,
    composition: &'a mut composition::Store,
    diagnostics: &'a mut diagnostics::Store,
    tasks: task::Sink,
}

pub struct Work {
    presentations: Vec<view::Presentation>,
    requests: Vec<session::Request>,
    pending_tasks: usize,
    task_completions: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Retention {
    changes: usize,
    snapshots: usize,
}

pub struct RenderWork {
    presentations: Vec<scene::Presentation>,
    requests: Vec<session::Request>,
    pending_tasks: usize,
    task_completions: usize,
}

impl Default for Retention {
    fn default() -> Self {
        Self {
            changes: state::DEFAULT_CHANGE_LIMIT,
            snapshots: timeline::DEFAULT_SNAPSHOT_LIMIT,
        }
    }
}

impl Retention {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn changes(mut self, limit: usize) -> Self {
        self.changes = limit;
        self
    }

    pub fn snapshots(mut self, limit: usize) -> Self {
        self.snapshots = limit;
        self
    }

    pub fn change_limit(self) -> usize {
        self.changes
    }

    pub fn snapshot_limit(self) -> usize {
        self.snapshots
    }
}

impl<'a, M: state::State> Services<'a, M> {
    fn new(
        timeline: &'a mut Timeline<M>,
        session: &'a mut session::Session,
        composition: &'a mut composition::Store,
        diagnostics: &'a mut diagnostics::Store,
        window: Option<window::Id>,
    ) -> Self {
        Self {
            timeline,
            session,
            composition,
            diagnostics,
            window,
        }
    }
}

impl<M: state::State> responder::Framework<M> for Services<'_, M> {
    fn state(
        &mut self,
        store: &mut Store<M>,
        command_type: TypeId,
        _command_name: &'static str,
        args: &dyn Any,
        cx: &command_context::Context,
    ) -> std::result::Result<Option<command::State>, Error> {
        if command_type == TypeId::of::<timeline::Undo>() {
            let args = framework_args::<timeline::Undo>(args)?;
            let service = timeline::Service::new(store, &mut *self.timeline);
            return Ok(Some(Target::<timeline::Undo>::state(&service, args, cx)));
        }

        if command_type == TypeId::of::<timeline::Redo>() {
            let args = framework_args::<timeline::Redo>(args)?;
            let service = timeline::Service::new(store, &mut *self.timeline);
            return Ok(Some(Target::<timeline::Redo>::state(&service, args, cx)));
        }

        if command_type == TypeId::of::<session::CloseWindow>() {
            let args = framework_args::<session::CloseWindow>(args)?;
            let service = session::Service::new(
                &mut *self.session,
                &mut *self.composition,
                &mut *self.diagnostics,
                self.window,
            );
            return Ok(Some(Target::<session::CloseWindow>::state(
                &service, args, cx,
            )));
        }

        Ok(None)
    }

    fn invoke(
        &mut self,
        store: &mut Store<M>,
        command_type: TypeId,
        _command_name: &'static str,
        args: Box<dyn Any + Send>,
        cx: &mut command_context::Context,
    ) -> Option<AnyResponse> {
        if command_type == TypeId::of::<timeline::Undo>() {
            let args = match framework_args_box::<timeline::Undo>(args) {
                Ok(args) => args,
                Err(error) => return Some(AnyResponse::failed(error)),
            };
            let mut service = timeline::Service::new(store, &mut *self.timeline);
            return Some(AnyResponse::from_response(
                Target::<timeline::Undo>::invoke(&mut service, args, cx),
            ));
        }

        if command_type == TypeId::of::<timeline::Redo>() {
            let args = match framework_args_box::<timeline::Redo>(args) {
                Ok(args) => args,
                Err(error) => return Some(AnyResponse::failed(error)),
            };
            let mut service = timeline::Service::new(store, &mut *self.timeline);
            return Some(AnyResponse::from_response(
                Target::<timeline::Redo>::invoke(&mut service, args, cx),
            ));
        }

        if command_type == TypeId::of::<session::CloseWindow>() {
            let args = match framework_args_box::<session::CloseWindow>(args) {
                Ok(args) => args,
                Err(error) => return Some(AnyResponse::failed(error)),
            };
            let mut service = session::Service::new(
                &mut *self.session,
                &mut *self.composition,
                &mut *self.diagnostics,
                self.window,
            );
            return Some(AnyResponse::from_response(
                Target::<session::CloseWindow>::invoke(&mut service, args, cx),
            ));
        }

        None
    }
}

fn framework_args<C: Command>(args: &dyn Any) -> std::result::Result<&C::Args, Error> {
    args.downcast_ref::<C::Args>()
        .ok_or(Error::ArgsMismatch { command: C::NAME })
}

fn text_for_key(
    key: input::Key,
    modifiers: input::Modifiers,
    inserted_text: Option<&str>,
) -> Option<String> {
    if modifiers.control() || modifiers.alt() || modifiers.super_key() {
        return None;
    }

    if let Some(text) =
        inserted_text.filter(|text| text.chars().all(|character| !character.is_control()))
    {
        return Some(text.to_owned());
    }

    match key {
        input::Key::Space => Some(" ".to_owned()),
        input::Key::Character(character) if !character.is_control() => Some(character.to_string()),
        _ => None,
    }
}

fn framework_args_box<C: Command>(
    args: Box<dyn Any + Send>,
) -> std::result::Result<C::Args, Error> {
    args.downcast::<C::Args>()
        .map(|args| *args)
        .map_err(|_| Error::ArgsMismatch { command: C::NAME })
}

impl<M: state::State> Runtime<M> {
    pub fn new(model: M) -> Self {
        let mut registry = command::Registry::default();
        session::register(&mut registry);
        timeline::register(&mut registry);

        Self {
            store: Store::new(model),
            timeline: Timeline::default(),
            session: session::Session::default(),
            composition: composition::Store::default(),
            layout: layout::Engine::default(),
            diagnostics: diagnostics::Store::default(),
            clipboard: Clipboard::default(),
            tasks: task::Queue::default(),
            registry,
            observers: command::Observers::default(),
            responders: responder::Builder::default(),
            gesture: None,
            history_group: None,
            started: None,
            event: None,
            view: None,
            started_ran: false,
        }
    }
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub fn retention(mut self, retention: Retention) -> Self {
        self.store.set_change_limit(retention.change_limit());
        self.timeline.set_snapshot_limit(retention.snapshot_limit());
        self
    }

    pub fn with_clipboard(mut self, clipboard: Clipboard) -> Self {
        self.clipboard = clipboard;
        self
    }

    pub fn snapshot(&self) -> Snapshot<M> {
        Snapshot {
            state: self.store.snapshot(),
            session: self.session.snapshot(),
        }
    }

    pub fn restore(&mut self, snapshot: Snapshot<M>) -> state::Change {
        self.restore_with_reason(snapshot, state::Reason::Restore)
    }

    pub fn save<P>(&mut self, persistence: &mut P) -> Result<state::Revision, P::Error>
    where
        P: Persistence<M>,
    {
        let snapshot = self.snapshot();
        persistence.save(&snapshot)?;
        self.store.mark_saved();
        self.request_all_redraws();
        Ok(self.revision())
    }

    pub fn load<P>(&mut self, persistence: &mut P) -> Result<state::Change, P::Error>
    where
        P: Persistence<M>,
    {
        let snapshot = persistence.load()?;
        Ok(self.restore_with_reason(snapshot, state::Reason::Load))
    }

    fn restore_with_reason(
        &mut self,
        snapshot: Snapshot<M>,
        reason: state::Reason,
    ) -> state::Change {
        let change = self.store.restore(snapshot.state, reason);
        self.store.mark_saved();
        self.session.restore(snapshot.session);
        self.composition.clear();
        self.timeline.clear();
        self.gesture = None;
        self.history_group = None;
        self.tasks.clear();
        self.diagnostics.restore_windows(self.session.windows());
        change
    }

    pub fn commands(mut self, configure: impl FnOnce(&mut command::Registry)) -> Self {
        configure(&mut self.registry);
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
            observers: self.observers,
            responders: self.responders,
            gesture: self.gesture,
            history_group: self.history_group,
            started: self.started,
            event: Some(Box::new(callback)),
            view: self.view,
            started_ran: self.started_ran,
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
            observers: self.observers,
            responders: self.responders,
            gesture: self.gesture,
            history_group: self.history_group,
            started: self.started,
            event: self.event,
            view: Some(Box::new(callback)),
            started_ran: self.started_ran,
        }
    }

    pub fn state(&self) -> &M {
        self.store.model()
    }

    pub fn store(&self) -> &Store<M> {
        &self.store
    }

    pub fn timeline(&self) -> &Timeline<M> {
        &self.timeline
    }

    pub fn session(&self) -> &session::Session {
        &self.session
    }

    pub fn composition(&self, window: window::Id) -> Option<&composition::Composition> {
        self.composition.get(window)
    }

    pub fn requests(&self) -> Vec<session::Request> {
        self.session.requests()
    }

    pub fn request_redraw(&mut self, window: window::Id) -> bool {
        self.session.request_redraw(window)
    }

    pub fn clear_redraw_request(&mut self, window: window::Id) -> bool {
        self.session.clear_redraw_request(window)
    }

    pub fn clipboard(&self) -> &Clipboard {
        &self.clipboard
    }

    pub fn pending_tasks(&self) -> usize {
        self.tasks.len()
    }

    pub fn pending_task_completions(&self) -> usize {
        self.tasks.completions_len()
    }

    pub fn task_status(&self, id: task::Id) -> Option<task::Status> {
        self.tasks.status(id)
    }

    pub fn cancel_task(&mut self, id: task::Id) -> bool {
        self.tasks.cancel(id)
    }

    pub fn complete_next_task(&mut self) -> Option<task::Id> {
        self.tasks.run_next()
    }

    pub fn dispatch_next_task_completion(&mut self) -> Option<task::Outcome> {
        let (id, event) = self.tasks.pop_completion()?;
        let before = self.revision();
        self.emit(event);
        Some(task::Outcome::completed(id, self.revision() != before))
    }

    pub fn run_next_task(&mut self) -> Option<task::Outcome> {
        if self.pending_task_completions() == 0 {
            self.complete_next_task()?;
        }

        self.dispatch_next_task_completion()
    }

    pub fn revision(&self) -> state::Revision {
        self.store.revision()
    }

    pub fn is_dirty(&self) -> bool {
        self.store.is_dirty()
    }

    pub fn mark_saved(&mut self) {
        self.store.mark_saved();
        self.request_all_redraws();
    }

    pub fn change(&mut self, reason: state::Reason, mutate: impl FnOnce(&mut M)) -> state::Change {
        let before = self.store.prepare_snapshot();
        mutate(self.store.model_mut());
        self.timeline.record(before.into_model());
        let change = self.store.commit_retaining_current(reason);
        self.request_all_redraws();
        change
    }

    pub fn undo(&mut self) -> bool {
        let trigger = self.trigger::<timeline::Undo>(());
        self.invoke(trigger).is_ok()
    }

    pub fn redo(&mut self) -> bool {
        let trigger = self.trigger::<timeline::Redo>(());
        self.invoke(trigger).is_ok()
    }

    pub fn trigger<C: Command>(&self, args: C::Args) -> command::Trigger<C> {
        command::Trigger::command(args)
    }

    pub fn start(&mut self) {
        if self.started_ran {
            return;
        }

        self.started_ran = true;

        let Some(started) = self.started.as_mut() else {
            return;
        };
        let task_sink = self.tasks.sink();
        let mut cx = Context::new(
            &mut self.store,
            &mut self.timeline,
            &mut self.session,
            &mut self.composition,
            &mut self.diagnostics,
            task_sink,
        );

        started(&mut cx);
    }

    pub fn emit(&mut self, event: E) {
        let before = self.revision();
        let Some(handler) = self.event.as_mut() else {
            return;
        };
        let task_sink = self.tasks.sink();
        let mut cx = Context::new(
            &mut self.store,
            &mut self.timeline,
            &mut self.session,
            &mut self.composition,
            &mut self.diagnostics,
            task_sink,
        );

        handler(&mut cx, event);

        if self.revision() != before {
            self.request_all_redraws();
        }
    }

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

    pub fn diagnostics(&self, window: window::Id) -> Option<&Diagnostics> {
        if !self.session.contains(window) {
            return None;
        }

        self.diagnostics.get(window)
    }

    pub fn diagnostics_mut(&mut self, window: window::Id) -> Option<&mut Diagnostics> {
        if !self.session.contains(window) {
            return None;
        }

        Some(self.diagnostics.get_mut(window))
    }

    pub fn state_for<C: Command>(&mut self, trigger: &command::Trigger<C>) -> command::State {
        let cx = command_context::Context::with_clipboard_source(
            &mut self.clipboard,
            command_context::Source::Programmatic,
        );
        let services = Services::new(
            &mut self.timeline,
            &mut self.session,
            &mut self.composition,
            &mut self.diagnostics,
            None,
        );
        let mut chain = self
            .responders
            .chain(&mut self.store)
            .with_framework(services);

        self.registry.state::<C>(&mut chain, trigger.args(), &cx)
    }

    pub fn state_for_focused<C: Command>(
        &mut self,
        window: window::Id,
        trigger: &command::Trigger<C>,
    ) -> command::State {
        if !self.session.contains(window) {
            return self
                .registry
                .apply_spec::<C>(command::State::disabled().with_tooltip("window is not open"));
        }

        let focus = self.session.focused(window);
        let cx = command_context::Context::with_clipboard_source(
            &mut self.clipboard,
            command_context::Source::Programmatic,
        );
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

        self.registry.state::<C>(&mut chain, trigger.args(), &cx)
    }

    pub fn invoke<C: Command>(&mut self, trigger: command::Trigger<C>) -> Response<C::Output> {
        self.transact_command::<C>(
            None,
            None,
            trigger.into_args(),
            command_context::Source::Programmatic,
            true,
        )
    }

    pub fn activate(
        &mut self,
        command: &view::Binding,
    ) -> std::result::Result<response::Effect, Error> {
        self.activate_with_focus(None, None, command)
    }

    pub fn activate_in(
        &mut self,
        window: window::Id,
        command: &view::Binding,
    ) -> std::result::Result<response::Effect, Error> {
        if !self.session.contains(window) {
            return Err(Error::MissingTarget {
                command: command.command_name(),
            });
        }

        let before = self.revision();
        let result = self.activate_with_focus(self.session.focused(window), Some(window), command);
        if let Ok(effect) = &result {
            self.apply_window_update(window, self.revision() != before, effect);
            self.close_menu_after_command(window, command);
        }

        result
    }

    pub fn handle_view(
        &mut self,
        window: window::Id,
        action: view::Action,
    ) -> std::result::Result<input::Outcome, Error> {
        match action {
            view::Action::Sequence(actions) => {
                let mut handled = false;
                let mut changed_state = false;
                let mut effect = response::Effect::None;

                for action in actions {
                    let outcome = self.handle_view(window, action)?;
                    handled |= outcome.is_handled();
                    changed_state |= outcome.changed_state();
                    effect = effect.then(outcome.effect().clone());
                }

                if handled {
                    Ok(input::Outcome::handled(changed_state, effect))
                } else {
                    Ok(input::Outcome::ignored())
                }
            }
            view::Action::Command(command) => {
                let before = self.revision();
                let effect = self.activate_in(window, &command)?;

                Ok(input::Outcome::handled(before != self.revision(), effect))
            }
            view::Action::Focus(focus) => self.handle_input(window, input::Input::focus(focus)),
            view::Action::PointerMove(target) => {
                self.handle_input(window, input::Input::pointer_move(target))
            }
            view::Action::PointerDown(target) => {
                self.handle_input(window, input::Input::pointer_down(target))
            }
            view::Action::PointerDrag {
                hovered,
                target,
                action,
            } => {
                let captured = self
                    .session
                    .interaction(window)
                    .and_then(|interaction| interaction.pointer().capture())
                    .map(|capture| capture.target())
                    == Some(&target);
                let pointer = self.handle_input(window, input::Input::pointer_drag(hovered))?;

                if !captured {
                    return Ok(pointer);
                }

                let Some(action) = action else {
                    return Ok(pointer);
                };

                let dragged = self.handle_view(window, *action)?;
                let effect = pointer.effect().clone().then(dragged.effect().clone());

                Ok(input::Outcome::handled(
                    pointer.changed_state() || dragged.changed_state(),
                    effect,
                ))
            }
            view::Action::PointerUp { target, action } => {
                let activate = self
                    .session
                    .interaction(window)
                    .and_then(|interaction| interaction.pointer().pressed())
                    == target.as_ref();
                let pointer = self.handle_pointer_up_input(window, target.clone(), false)?;

                if !activate {
                    self.finish_pointer_gesture();
                    return Ok(pointer);
                }

                let Some(action) = action else {
                    self.finish_pointer_gesture();
                    return Ok(pointer);
                };

                let activated = self.handle_view(window, *action);
                self.finish_pointer_gesture();
                let activated = activated?;
                let effect = pointer.effect().clone().then(activated.effect().clone());

                Ok(input::Outcome::handled(
                    pointer.changed_state() || activated.changed_state(),
                    effect,
                ))
            }
            view::Action::PointerLeft => self.handle_input(window, input::Input::pointer_left()),
            view::Action::Scroll { target, delta } => {
                self.handle_input(window, input::Input::scroll(target, delta))
            }
            view::Action::ToggleMenu(menu) => {
                self.handle_input(window, input::Input::toggle_menu(menu))
            }
            view::Action::TextEdit(edit) => {
                self.handle_input(window, input::Input::text_edit(edit))
            }
            view::Action::TextDrop(drop) => self.handle_input(window, input::Input::TextDrop(drop)),
        }
    }

    pub fn focus(&mut self, window: window::Id, focus: session::Focus) -> bool {
        self.session.focus(window, focus)
    }

    pub fn clear_focus(&mut self, window: window::Id) -> bool {
        self.session.clear_focus(window)
    }

    pub fn invoke_focused<C: Command>(
        &mut self,
        window: window::Id,
        trigger: command::Trigger<C>,
    ) -> Response<C::Output> {
        self.invoke_focused_with_source(window, trigger, command_context::Source::Programmatic)
    }

    fn invoke_focused_with_source<C: Command>(
        &mut self,
        window: window::Id,
        trigger: command::Trigger<C>,
        source: command_context::Source,
    ) -> Response<C::Output> {
        if !self.session.contains(window) {
            return Response::failed(Error::MissingTarget { command: C::NAME });
        }

        let response =
            self.invoke_with_focus(self.session.focused(window), Some(window), trigger, source);
        if response.is_ok() {
            self.apply_window_update(window, response.changed_state(), &response.effect);
        }

        response
    }

    pub fn handle_input(
        &mut self,
        window: window::Id,
        input: input::Input,
    ) -> std::result::Result<input::Outcome, Error> {
        if !self.session.contains(window) {
            return Ok(input::Outcome::ignored());
        }

        match input {
            input::Input::Cancel => {
                if self.session.close_menu(window) {
                    return Ok(self.window_outcome(window, false, response::Effect::Repaint));
                }

                if self.session.clear_text_input(window) {
                    return Ok(self.window_outcome(window, false, response::Effect::Repaint));
                }

                if self.session.cancel_pointer(window) {
                    self.finish_pointer_gesture();
                    return Ok(self.window_outcome(window, false, response::Effect::Repaint));
                }

                if self.clear_focus(window) {
                    return Ok(self.window_outcome(window, false, response::Effect::Repaint));
                }

                Ok(input::Outcome::ignored())
            }
            input::Input::Focus(focus) => {
                let effect = if self.focus(window, focus) {
                    response::Effect::Repaint
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::PointerMove(target) => {
                let effect = if self.session.pointer_move(window, target) {
                    response::Effect::Repaint
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::PointerDown(target) => {
                self.begin_pointer_gesture(&target);
                let effect = if self.session.pointer_down(window, target) {
                    response::Effect::Repaint
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::PointerDrag(hovered) => {
                let effect = if self.session.pointer_move(window, hovered) {
                    response::Effect::Repaint
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::PointerUp(target) => self.handle_pointer_up_input(window, target, true),
            input::Input::PointerLeft => {
                let effect = if self.session.pointer_left(window) {
                    response::Effect::Repaint
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::Scroll { target, delta } => {
                let scrolled = self.session.scroll_by(window, target, delta);
                let effect = if scrolled {
                    response::Effect::Repaint
                } else {
                    response::Effect::None
                };
                self.record_scroll_input(window, scrolled, &effect);

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::ToggleMenu(menu) => {
                let effect = if self.session.toggle_menu(window, menu) {
                    response::Effect::Repaint
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::FilePathSelected(path) => {
                let Some(dialog) = self.session.take_file_dialog(window) else {
                    return Ok(input::Outcome::ignored());
                };

                match (dialog, path) {
                    (session::FileDialog::Open, Some(path)) => {
                        let response = self.invoke_focused_with_source(
                            window,
                            command::Trigger::<document::OpenPath>::command(path),
                            command_context::Source::Input,
                        );
                        let changed = response.changed_state();
                        let effect = response.effect.clone();

                        response
                            .output
                            .map(|_| input::Outcome::handled(changed, effect))
                    }
                    (session::FileDialog::Open, None) => {
                        let response = self.invoke_focused_with_source(
                            window,
                            command::Trigger::<document::OpenCanceled>::command(()),
                            command_context::Source::Input,
                        );
                        let changed = response.changed_state();
                        let effect = response.effect.clone();

                        response
                            .output
                            .map(|_| input::Outcome::handled(changed, effect))
                    }
                    (session::FileDialog::SaveAs, Some(path)) => {
                        let response = self.invoke_focused_with_source(
                            window,
                            command::Trigger::<document::SaveToPath>::command(path),
                            command_context::Source::Input,
                        );
                        let changed = response.changed_state();
                        let effect = response.effect.clone();

                        response
                            .output
                            .map(|_| input::Outcome::handled(changed, effect))
                    }
                    (session::FileDialog::SaveAs, None) => {
                        let response = self.invoke_focused_with_source(
                            window,
                            command::Trigger::<document::SaveCanceled>::command(()),
                            command_context::Source::Input,
                        );
                        let changed = response.changed_state();
                        let effect = response.effect.clone();

                        response
                            .output
                            .map(|_| input::Outcome::handled(changed, effect))
                    }
                }
            }
            input::Input::Shortcut(shortcut) => self.handle_shortcut(window, shortcut),
            input::Input::KeyDown {
                key,
                modifiers,
                text,
            } => self.handle_key_down(window, key, modifiers, text),
            input::Input::TextEdit(edit) => {
                self.handle_text_edit(window, edit, command_context::Source::Keyboard)
            }
            input::Input::TextCommit(text) => self.handle_text_commit(window, text),
            input::Input::TextPreedit(preedit) => {
                let Some(focus) = self.session.focused(window) else {
                    return Ok(input::Outcome::ignored());
                };
                let target = self.text_input_target(window, focus);
                let changed = self.session.set_text_preedit_for(window, target, preedit);
                let effect = if changed {
                    response::Effect::Repaint
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::TextDrop(drop) => self.handle_text_drop(window, drop),
        }
    }

    fn handle_key_down(
        &mut self,
        window: window::Id,
        key: input::Key,
        modifiers: input::Modifiers,
        text: Option<String>,
    ) -> std::result::Result<input::Outcome, Error> {
        if key == input::Key::Escape {
            return self.handle_input(window, input::Input::cancel());
        }

        if key == input::Key::Tab
            && !modifiers.control()
            && !modifiers.alt()
            && !modifiers.super_key()
        {
            return Ok(self.handle_tab_focus(window, modifiers.shift()));
        }

        if let Some(shortcut) = self.registry.shortcut_for_key(key, modifiers) {
            let outcome = self.handle_shortcut(window, shortcut)?;
            if outcome.is_handled() {
                return Ok(outcome);
            }
        }

        if let Some(text) = text_for_key(key, modifiers, text.as_deref()) {
            return self.handle_text_commit(window, text);
        }

        let Some(edit) = input::edit_for_key(key, modifiers) else {
            return Ok(input::Outcome::ignored());
        };

        self.handle_text_edit(window, edit, command_context::Source::Keyboard)
    }

    fn handle_tab_focus(&mut self, window: window::Id, reverse: bool) -> input::Outcome {
        let direction = if reverse {
            view::FocusDirection::Backward
        } else {
            view::FocusDirection::Forward
        };
        let Some(next) = self.composition.get(window).and_then(|composition| {
            composition
                .view()
                .next_focus(self.session.focused(window), direction)
        }) else {
            return input::Outcome::ignored();
        };

        let effect = if self.focus(window, next) {
            response::Effect::Repaint
        } else {
            response::Effect::None
        };

        input::Outcome::handled(false, effect)
    }

    fn handle_text_commit(
        &mut self,
        window: window::Id,
        text: String,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(focus) = self.session.focused(window) else {
            return Ok(input::Outcome::ignored());
        };

        if self.text_box_base_text(window, focus).is_some() {
            return self.handle_text_box_edit(window, focus, text::edit::Edit::ime_commit(text));
        }

        if text.is_empty() {
            return if self.session.clear_text_input(window) {
                Ok(self.window_outcome(window, false, response::Effect::Repaint))
            } else {
                Ok(input::Outcome::ignored())
            };
        }

        self.handle_text_edit(
            window,
            text::edit::Edit::ime_commit(text),
            command_context::Source::Input,
        )
    }

    fn text_box_base_text(&self, window: window::Id, focus: session::Focus) -> Option<String> {
        self.composition
            .get(window)?
            .view()
            .text_box_text(focus)
            .map(str::to_owned)
    }

    fn text_input_target(&self, window: window::Id, focus: session::Focus) -> interaction::Target {
        self.composition
            .get(window)
            .and_then(|composition| composition.view().text_input_target(focus))
            .unwrap_or_else(|| interaction::Target::text_area(focus))
    }

    fn handle_text_box_edit(
        &mut self,
        window: window::Id,
        focus: session::Focus,
        edit: text::edit::Edit,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(base) = self.text_box_base_text(window, focus) else {
            return Ok(input::Outcome::ignored());
        };
        let Some(change) = self.session.edit_text_draft(window, focus, base, edit) else {
            return Ok(input::Outcome::ignored());
        };

        let mut handled = change.changed() || change.submit();
        let mut changed_state = false;
        let mut effect = if change.changed() {
            response::Effect::Repaint
        } else {
            response::Effect::None
        };

        if change.text_changed() || change.submit() {
            let action = self.composition.get(window).and_then(|composition| {
                composition
                    .view()
                    .text_commit_action(focus, change.text().to_owned())
            });

            if let Some(action) = action {
                let outcome = self.handle_view(window, action)?;
                handled |= outcome.is_handled();
                changed_state |= outcome.changed_state();
                effect = effect.then(outcome.effect().clone());
            }
        }

        if handled {
            Ok(self.window_outcome(window, changed_state, effect))
        } else {
            Ok(input::Outcome::ignored())
        }
    }

    fn handle_pointer_up_input(
        &mut self,
        window: window::Id,
        target: Option<interaction::Target>,
        finish_gesture: bool,
    ) -> std::result::Result<input::Outcome, Error> {
        let effect = if self.session.pointer_up(window, target) {
            response::Effect::Repaint
        } else {
            response::Effect::None
        };

        if finish_gesture {
            self.finish_pointer_gesture();
        }

        Ok(self.window_outcome(window, false, effect))
    }

    fn begin_pointer_gesture(&mut self, target: &interaction::Target) {
        if !Self::coalesces_pointer_gesture(target) || self.gesture.is_some() {
            return;
        }

        self.gesture = Some(Gesture {
            target: target.clone(),
            initial: self.store.model().clone(),
            changed_automatic: false,
        });
    }

    fn finish_pointer_gesture(&mut self) {
        let Some(gesture) = self.gesture.take() else {
            return;
        };

        if gesture.changed_automatic {
            self.timeline.record(gesture.initial);
        }
    }

    fn active_automatic_gesture(&self) -> bool {
        self.gesture.is_some()
    }

    fn mark_automatic_gesture_changed(&mut self) {
        if let Some(gesture) = &mut self.gesture {
            gesture.changed_automatic = true;
        }
    }

    fn coalesces_pointer_gesture(target: &interaction::Target) -> bool {
        target.captures() && target.kind() == interaction::Kind::Command
    }

    fn coalesces_history_group(&mut self, group: Option<command::HistoryGroup>) -> bool {
        let Some(group) = group else {
            self.clear_history_group();
            return false;
        };
        let now = Instant::now();
        let coalesces = self.history_group.as_ref().is_some_and(|active| {
            active.group == group
                && now.saturating_duration_since(active.recorded_at)
                    <= HISTORY_GROUP_COALESCE_WINDOW
        });
        self.history_group = Some(ActiveHistoryGroup {
            group,
            recorded_at: now,
        });
        coalesces
    }

    fn clear_history_group(&mut self) {
        self.history_group = None;
    }

    fn handle_text_edit(
        &mut self,
        window: window::Id,
        edit: text::edit::Edit,
        source: command_context::Source,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(focus) = self.session.focused(window) else {
            return Ok(input::Outcome::ignored());
        };

        if self.text_box_base_text(window, focus).is_some() {
            return self.handle_text_box_edit(window, focus, edit);
        }

        let reveal_target = self.text_input_target(window, focus);
        let cleared_preedit = self.session.clear_text_input(window);
        let response = self.invoke_focused_with_source(
            window,
            command::Trigger::<document::ApplyEdit>::command(edit),
            source,
        );
        let changed = response.changed_state();
        let reveal = response
            .output_ref()
            .is_some_and(|outcome| outcome.buffer_changed());
        let mut effect = response.effect.clone();
        if reveal && self.session.reveal_scroll(window, reveal_target) {
            effect = effect.then(response::Effect::Repaint);
        }
        if cleared_preedit {
            effect = effect.then(response::Effect::Repaint);
            self.apply_window_update(window, false, &response::Effect::Repaint);
        }

        response
            .output
            .map(|_| input::Outcome::handled(changed, effect))
    }

    fn handle_shortcut(
        &mut self,
        window: window::Id,
        shortcut: command::KeyChord,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(command) = self.registry.shortcut_command(shortcut)? else {
            return Ok(input::Outcome::ignored());
        };
        let command_type = command.command_type();
        let command_name = command.command_name();

        let source = command_context::Source::Shortcut;
        let Some(transaction) = self.transact_any_command(
            self.session.focused(window),
            Some(window),
            command_type,
            command_name,
            source,
            |registry, chain, cx| registry.invoke_shortcut(shortcut, chain, cx),
        )?
        else {
            return Ok(input::Outcome::ignored());
        };

        let changed = transaction.changed_state;
        let effect = transaction.effect;
        transaction
            .response
            .into_result()
            .map(|_| self.window_outcome(window, changed, effect))
    }

    fn handle_text_drop(
        &mut self,
        window: window::Id,
        text_drop: input::TextDrop,
    ) -> std::result::Result<input::Outcome, Error> {
        let before = self.store.prepare_snapshot();
        let focus = self.session.focused(window);
        let reveal_target = focus.map(|focus| self.text_input_target(window, focus));
        let (edit, source_cleanup) = text_drop.into_edits();
        let task_sink = self.tasks.sink();
        let mut cx = command_context::Context::with_services_source(
            &mut self.clipboard,
            task_sink,
            command_context::Source::Input,
        )
        .with_text_service(self.layout.text_service());
        let mut chain = self.responders.chain_for(&mut self.store, focus);

        let response = self
            .registry
            .invoke::<document::ApplyEdit>(&mut chain, edit, &mut cx);
        let mut changed = response.changed_state();
        let mut effect = response.effect.clone();

        if let Err(error) = response.output {
            drop(chain);
            if changed {
                self.store.discard_retained_snapshot();
            } else {
                self.store.restore_prepared_snapshot(before);
            }
            return Err(error);
        }

        if changed && let Some(source_cleanup) = source_cleanup {
            let cleanup_response =
                self.registry
                    .invoke::<document::ApplyEdit>(&mut chain, source_cleanup, &mut cx);

            changed |= cleanup_response.changed_state();
            effect = effect.then(cleanup_response.effect.clone());

            if let Err(error) = cleanup_response.output {
                drop(chain);
                if changed {
                    self.store.discard_retained_snapshot();
                } else {
                    self.store.restore_prepared_snapshot(before);
                }
                return Err(error);
            }
        }

        drop(chain);

        if changed {
            if let Some(target) = reveal_target
                && self.session.reveal_scroll(window, target)
            {
                effect = effect.then(response::Effect::Repaint);
            }
            self.timeline.record(before.into_model());
            self.store
                .commit_retaining_current(state::Reason::event("text_drop"));
        } else {
            self.store.restore_prepared_snapshot(before);
        }

        Ok(self.window_outcome(window, changed, effect))
    }

    fn invoke_with_focus<C: Command>(
        &mut self,
        focus: Option<session::Focus>,
        window: Option<window::Id>,
        trigger: command::Trigger<C>,
        source: command_context::Source,
    ) -> Response<C::Output> {
        self.transact_command::<C>(focus, window, trigger.into_args(), source, false)
    }

    fn transact_command<C: Command>(
        &mut self,
        focus: Option<session::Focus>,
        window: Option<window::Id>,
        args: C::Args,
        source: command_context::Source,
        request_all_redraws: bool,
    ) -> Response<C::Output> {
        let history = C::HISTORY;
        let history_group = C::history_group(&args);
        let before = self.snapshot_before_transaction(history);
        let revision_before = self.revision();
        let task_sink = self.tasks.sink();
        let mut cx =
            command_context::Context::with_services_source(&mut self.clipboard, task_sink, source)
                .with_text_service(self.layout.text_service());
        let services = Services::new(
            &mut self.timeline,
            &mut self.session,
            &mut self.composition,
            &mut self.diagnostics,
            window,
        );
        let mut chain = self
            .responders
            .chain_for(&mut self.store, focus)
            .with_framework(services);
        let mut response = self.registry.invoke::<C>(&mut chain, args, &mut cx);
        let command_changed = response.is_ok() && response.changed_state();

        drop(chain);
        drop(cx);

        let observer_changed = match self.observe_response::<C>(&response, source) {
            Ok(changed) => changed,
            Err(error) => {
                self.finish_transaction(
                    before,
                    history,
                    history_group,
                    revision_before,
                    state::Reason::command::<C>(),
                    false,
                );
                return Response::failed(error);
            }
        };
        if observer_changed {
            response.mark_changed();
        }
        let changed = response.is_ok() && (command_changed || observer_changed);

        self.finish_transaction(
            before,
            history,
            history_group,
            revision_before,
            state::Reason::command::<C>(),
            changed,
        );
        if changed && request_all_redraws {
            self.request_all_redraws();
        }

        response
    }

    fn transact_any_command(
        &mut self,
        focus: Option<session::Focus>,
        window: Option<window::Id>,
        command_type: TypeId,
        command_name: &'static str,
        source: command_context::Source,
        invoke: impl FnOnce(
            &command::Registry,
            &mut responder::Chain<'_, M>,
            &mut command_context::Context,
        ) -> std::result::Result<Option<AnyResponse>, Error>,
    ) -> std::result::Result<Option<AnyCommandTransaction>, Error> {
        let history = self
            .registry
            .history_for(command_type)
            .unwrap_or(command::History::Automatic);
        let before = self.snapshot_before_transaction(history);
        let revision_before = self.revision();
        let task_sink = self.tasks.sink();
        let mut cx =
            command_context::Context::with_services_source(&mut self.clipboard, task_sink, source)
                .with_text_service(self.layout.text_service());
        let services = Services::new(
            &mut self.timeline,
            &mut self.session,
            &mut self.composition,
            &mut self.diagnostics,
            window,
        );
        let mut chain = self
            .responders
            .chain_for(&mut self.store, focus)
            .with_framework(services);
        let mut response = match invoke(&self.registry, &mut chain, &mut cx) {
            Ok(Some(response)) => response,
            Ok(None) => {
                drop(chain);
                drop(cx);
                self.finish_transaction(
                    before,
                    history,
                    None,
                    revision_before,
                    state::Reason::Command(command_name),
                    false,
                );
                return Ok(None);
            }
            Err(error) => {
                drop(chain);
                drop(cx);
                self.finish_transaction(
                    before,
                    history,
                    None,
                    revision_before,
                    state::Reason::Command(command_name),
                    false,
                );
                return Err(error);
            }
        };
        let command_changed = response.is_ok() && response.changed_state();

        drop(chain);
        drop(cx);

        let observer_changed = match self.observe_any_response(command_type, &response, source) {
            Ok(changed) => changed,
            Err(error) => {
                self.finish_transaction(
                    before,
                    history,
                    None,
                    revision_before,
                    state::Reason::Command(command_name),
                    false,
                );
                return Err(error);
            }
        };
        if observer_changed {
            response.mark_changed();
        }
        let changed = response.is_ok() && (command_changed || observer_changed);

        self.finish_transaction(
            before,
            history,
            None,
            revision_before,
            state::Reason::Command(command_name),
            changed,
        );

        let effect = response.effect();

        Ok(Some(AnyCommandTransaction {
            response,
            changed_state: changed,
            effect,
        }))
    }

    fn activate_with_focus(
        &mut self,
        focus: Option<session::Focus>,
        window: Option<window::Id>,
        command: &view::Binding,
    ) -> std::result::Result<response::Effect, Error> {
        let source = command.source();
        let transaction = self
            .transact_any_command(
                focus,
                window,
                command.command_type(),
                command.command_name(),
                source,
                |registry, chain, cx| Ok(Some(command.invoke(registry, chain, cx))),
            )?
            .expect("view command activation always invokes a command");

        transaction
            .response
            .into_result()
            .map(|_| transaction.effect)
    }

    fn snapshot_before_transaction(
        &mut self,
        history: command::History,
    ) -> Option<state::PendingSnapshot<M>> {
        match history {
            command::History::Automatic => Some(self.store.prepare_snapshot()),
            command::History::Committed | command::History::Ignored => None,
        }
    }

    fn finish_transaction(
        &mut self,
        before: Option<state::PendingSnapshot<M>>,
        history: command::History,
        history_group: Option<command::HistoryGroup>,
        revision_before: state::Revision,
        reason: state::Reason,
        changed: bool,
    ) {
        if !changed {
            if let Some(before) = before {
                self.store.restore_prepared_snapshot(before);
            }
            return;
        }

        match history {
            command::History::Automatic => {
                let before = before.expect("automatic history snapshots before dispatch");
                if self.active_automatic_gesture() {
                    self.mark_automatic_gesture_changed();
                    self.clear_history_group();
                    drop(before);
                } else if self.coalesces_history_group(history_group) {
                    drop(before);
                } else {
                    self.timeline.record(before.into_model());
                }
                self.store.commit_retaining_current(reason);
            }
            command::History::Committed | command::History::Ignored => {
                self.clear_history_group();
                if self.revision() == revision_before {
                    self.store.commit(reason);
                } else {
                    self.store.discard_retained_snapshot();
                }
            }
        }
    }

    fn observe_response<C: Command>(
        &mut self,
        response: &Response<C::Output>,
        source: command_context::Source,
    ) -> std::result::Result<bool, Error> {
        if !response.is_ok() {
            return Ok(false);
        }

        let observers = &mut self.observers;
        let model = self.store.model_mut();
        observers.observe_response::<C>(model, response, source)
    }

    fn observe_any_response(
        &mut self,
        command_type: TypeId,
        response: &response::AnyResponse,
        source: command_context::Source,
    ) -> std::result::Result<bool, Error> {
        let observers = &mut self.observers;
        let model = self.store.model_mut();
        observers.observe_any(command_type, model, response, source)
    }

    fn window_outcome(
        &mut self,
        window: window::Id,
        changed_state: bool,
        effect: response::Effect,
    ) -> input::Outcome {
        self.apply_window_update(window, changed_state, &effect);
        input::Outcome::handled(changed_state, effect)
    }

    fn record_scroll_input(
        &mut self,
        window: window::Id,
        offset_changed: bool,
        effect: &response::Effect,
    ) {
        let diagnostics = self.diagnostics.get_mut(window);
        diagnostics.scroll.wheel_events += 1;
        if offset_changed {
            diagnostics.scroll.scroll_offset_changes += 1;
        }
        if effect.contains(&response::Effect::Repaint) {
            diagnostics.scroll.scroll_redraw_requests += 1;
        }
    }

    fn record_layout_diagnostics(&mut self, window: window::Id, layout: &layout::Layout) {
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
        diagnostics.scroll.projection_count += text_area_count;
        diagnostics.frame.full_redraws += 1;
        diagnostics.frame.last_scroll_frame.text_surfaces = text_surfaces;
    }

    fn apply_layout_feedback(&mut self, window: window::Id, layout: &layout::Layout) {
        for frame in layout.frames() {
            let Some(offset) = frame
                .text_area_layout()
                .and_then(layout::TextAreaLayout::resolved_scroll)
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

    fn apply_window_update(
        &mut self,
        window: window::Id,
        changed_state: bool,
        effect: &response::Effect,
    ) {
        if changed_state {
            self.session.request_redraw(window);
        }

        self.apply_window_effect(window, effect);
    }

    fn apply_window_effect(&mut self, window: window::Id, effect: &response::Effect) {
        match effect {
            response::Effect::OpenFileDialog => {
                self.session
                    .request_file_dialog(window, session::FileDialog::Open);
            }
            response::Effect::SaveFileDialog => {
                self.session
                    .request_file_dialog(window, session::FileDialog::SaveAs);
            }
            response::Effect::Repaint => {
                self.session.request_redraw(window);
            }
            response::Effect::ClosePopup => {
                if self.session.close_menu(window) {
                    self.session.request_redraw(window);
                }
            }
            response::Effect::Batch(effects) => {
                for effect in effects {
                    self.apply_window_effect(window, effect);
                }
            }
            response::Effect::None => {}
        }
    }

    fn close_menu_after_command(&mut self, window: window::Id, command: &view::Binding) {
        if command.source() == command_context::Source::Menu && self.session.close_menu(window) {
            self.session.request_redraw(window);
        }
    }

    fn request_all_redraws(&mut self) {
        let windows = self
            .session
            .windows()
            .iter()
            .map(session::Window::id)
            .collect::<Vec<_>>();

        for window in windows {
            self.session.request_redraw(window);
        }
    }

    fn view_context(&self, window: window::Id) -> view::Context {
        view::Context::new(
            window,
            self.diagnostics.get(window).cloned().unwrap_or_default(),
            self.session
                .interaction(window)
                .cloned()
                .unwrap_or_default(),
        )
    }

    fn canvas_color(&self, window: window::Id) -> scene::Color {
        self.session
            .window(window)
            .map(session::Window::canvas_color)
            .unwrap_or_else(window::Options::default_canvas_color)
    }
}

impl<M: state::State, E: Send + 'static> Runtime<M, E, view::View> {
    pub fn drain(&mut self) -> Work {
        Work::new(
            self.present_pending(),
            self.requests(),
            self.pending_tasks(),
            self.pending_task_completions(),
        )
    }

    pub fn drain_scenes(
        &mut self,
        size_for: impl FnMut(window::Id) -> geometry::Size,
    ) -> RenderWork {
        RenderWork::new(
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
    ) -> Option<layout::Hit> {
        let composition = self.composition.get(window)?;
        layout::Layout::compose(composition.view(), size, &mut self.layout).hit_test(point)
    }

    pub fn pointer_move_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
    ) -> std::result::Result<input::Outcome, Error> {
        if self
            .session
            .interaction(window)
            .and_then(|interaction| interaction.pointer().pressed())
            .is_some()
        {
            return self.pointer_drag_at(window, size, point);
        }

        let target = self
            .hit_test(window, size, point)
            .and_then(|hit| hit.target().cloned());

        self.handle_view(window, view::Action::pointer_move(target))
    }

    pub fn pointer_down_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(hit) = self.hit_test(window, size, point) else {
            return Ok(input::Outcome::ignored());
        };
        let Some(target) = hit.target().cloned() else {
            return Ok(input::Outcome::ignored());
        };

        let action = if matches!(
            hit.frame().role(),
            view::Role::TextArea | view::Role::TextBox
        ) {
            hit.action_at_with_engine(point, &mut self.layout)
                .map(|action| {
                    view::Action::sequence([view::Action::pointer_down(target.clone()), action])
                })
                .unwrap_or_else(|| view::Action::pointer_down(target))
        } else if hit.frame().role() == view::Role::Slider {
            hit.action_at_with_engine(point, &mut self.layout)
                .map(|action| {
                    view::Action::sequence([view::Action::pointer_down(target.clone()), action])
                })
                .unwrap_or_else(|| view::Action::pointer_down(target))
        } else {
            view::Action::pointer_down(target)
        };

        self.handle_view(window, action)
    }

    pub fn pointer_up_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
    ) -> std::result::Result<input::Outcome, Error> {
        let hit = self.hit_test(window, size, point);
        let target = hit.as_ref().and_then(|hit| hit.target().cloned());
        let action = hit.as_ref().and_then(|hit| hit.action_at(point));

        self.handle_view(window, view::Action::pointer_up(target, action))
    }

    pub fn pointer_drag_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(composition) = self.composition.get(window) else {
            return Ok(input::Outcome::ignored());
        };

        let layout = layout::Layout::compose(composition.view(), size, &mut self.layout);
        let hit = layout.hit_test(point);
        let hovered = hit.as_ref().and_then(|hit| hit.target().cloned());
        let active = self.session.interaction(window).and_then(|interaction| {
            interaction
                .pointer()
                .capture()
                .map(|capture| capture.target().clone())
                .or_else(|| interaction.pointer().pressed().cloned())
        });

        let Some(target) = active else {
            return self.handle_view(window, view::Action::pointer_move(hovered));
        };

        let action = layout
            .frames()
            .iter()
            .find(|frame| frame.target() == Some(&target))
            .and_then(|frame| frame.drag_action_at_with_engine(point, &mut self.layout));

        self.handle_view(window, view::Action::pointer_drag(hovered, target, action))
    }

    pub fn scroll_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
        delta: interaction::ScrollDelta,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(target) = self
            .hit_test(window, size, point)
            .and_then(|hit| hit.target().cloned())
        else {
            return Ok(input::Outcome::ignored());
        };

        self.handle_view(window, view::Action::scroll(target, delta))
    }
}

impl Work {
    fn new(
        presentations: Vec<view::Presentation>,
        requests: Vec<session::Request>,
        pending_tasks: usize,
        task_completions: usize,
    ) -> Self {
        Self {
            presentations,
            requests,
            pending_tasks,
            task_completions,
        }
    }

    pub fn presentations(&self) -> &[view::Presentation] {
        &self.presentations
    }

    pub fn requests(&self) -> &[session::Request] {
        &self.requests
    }

    pub fn pending_tasks(&self) -> usize {
        self.pending_tasks
    }

    pub fn task_completions(&self) -> usize {
        self.task_completions
    }

    pub fn is_empty(&self) -> bool {
        self.presentations.is_empty()
            && self.requests.is_empty()
            && self.pending_tasks == 0
            && self.task_completions == 0
    }
}

impl RenderWork {
    fn new(
        presentations: Vec<scene::Presentation>,
        requests: Vec<session::Request>,
        pending_tasks: usize,
        task_completions: usize,
    ) -> Self {
        Self {
            presentations,
            requests,
            pending_tasks,
            task_completions,
        }
    }

    pub fn presentations(&self) -> &[scene::Presentation] {
        &self.presentations
    }

    pub fn requests(&self) -> &[session::Request] {
        &self.requests
    }

    pub fn pending_tasks(&self) -> usize {
        self.pending_tasks
    }

    pub fn task_completions(&self) -> usize {
        self.task_completions
    }

    pub fn is_empty(&self) -> bool {
        self.presentations.is_empty()
            && self.requests.is_empty()
            && self.pending_tasks == 0
            && self.task_completions == 0
    }
}

impl<M: state::State> Snapshot<M> {
    pub fn new(state: state::Snapshot<M>, session: session::Snapshot) -> Self {
        Self { state, session }
    }

    pub fn state(&self) -> &state::Snapshot<M> {
        &self.state
    }

    pub fn session(&self) -> &session::Snapshot {
        &self.session
    }
}

impl<'a, M: state::State> Context<'a, M> {
    fn new(
        store: &'a mut Store<M>,
        timeline: &'a mut Timeline<M>,
        session: &'a mut session::Session,
        composition: &'a mut composition::Store,
        diagnostics: &'a mut diagnostics::Store,
        tasks: task::Sink,
    ) -> Self {
        Self {
            store,
            timeline,
            session,
            composition,
            diagnostics,
            tasks,
        }
    }

    pub fn state(&self) -> &M {
        self.store.model()
    }

    pub fn revision(&self) -> state::Revision {
        self.store.revision()
    }

    pub fn is_dirty(&self) -> bool {
        self.store.is_dirty()
    }

    pub fn mark_saved(&mut self) {
        self.store.mark_saved();
        self.request_all_redraws();
    }

    pub fn change(&mut self, reason: state::Reason, mutate: impl FnOnce(&mut M)) -> state::Change {
        let before = self.store.prepare_snapshot();
        mutate(self.store.model_mut());
        self.timeline.record(before.into_model());
        let change = self.store.commit_retaining_current(reason);
        self.request_all_redraws();
        change
    }

    pub fn open_window(&mut self, options: window::Options) -> window::Id {
        let window = self.session.open_window(options);
        self.diagnostics.insert_window(window);
        window
    }

    pub fn close_window(&mut self, id: window::Id) -> bool {
        if !self.session.close_window(id) {
            return false;
        }

        self.composition.remove_window(id);
        self.diagnostics.remove_window(id);
        true
    }

    pub fn diagnostics(&self, id: window::Id) -> Option<&Diagnostics> {
        self.session
            .contains(id)
            .then(|| self.diagnostics.get(id))
            .flatten()
    }

    pub fn diagnostics_mut(&mut self, id: window::Id) -> Option<&mut Diagnostics> {
        if !self.session.contains(id) {
            return None;
        }

        Some(self.diagnostics.get_mut(id))
    }

    pub fn request_redraw(&mut self, id: window::Id) -> bool {
        self.session.request_redraw(id)
    }

    pub fn spawn<E: Send + 'static>(&mut self, task: task::Task<E>) -> Option<task::Id> {
        self.tasks.spawn(task.into_any())
    }

    pub fn windows(&self) -> &[session::Window] {
        self.session.windows()
    }

    fn request_all_redraws(&mut self) {
        let windows = self
            .session
            .windows()
            .iter()
            .map(session::Window::id)
            .collect::<Vec<_>>();

        for window in windows {
            self.session.request_redraw(window);
        }
    }
}
