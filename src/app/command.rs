use std::collections::HashMap;

use crate::{command, ui, widget, window};

use super::{floating, state::WindowState, text_input};

#[derive(Debug, Default)]
pub(crate) struct State {
    pub(crate) subject: Option<command::call::Scope>,
    pub(crate) scope_captures: HashMap<ui::Path, command::call::Context>,
}

impl State {
    #[cfg(test)]
    pub(crate) fn with_subject(subject: command::call::Scope) -> Self {
        Self {
            subject: Some(subject),
            ..Self::default()
        }
    }

    #[cfg(test)]
    pub(crate) fn with_scope_captures(
        scope_captures: HashMap<ui::Path, command::call::Context>,
    ) -> Self {
        Self {
            scope_captures,
            ..Self::default()
        }
    }
}

pub(crate) struct Layer<'a> {
    registry: &'a mut command::Registry,
    window: window::Id,
}

impl<'a> Layer<'a> {
    pub(crate) fn new(registry: &'a mut command::Registry, window: window::Id) -> Self {
        Self { registry, window }
    }

    pub(crate) fn menu_presenter<'b>(&'b self, state: &'b WindowState) -> MenuPresenter<'b> {
        MenuPresenter::new(&*self.registry, state, self.window, None)
    }

    pub(crate) fn menu_presenter_for_composition<'b>(
        &'b self,
        state: &'b WindowState,
        composition: &'b ui::Composition,
    ) -> MenuPresenter<'b> {
        MenuPresenter::new(&*self.registry, state, self.window, Some(composition))
    }

    pub(crate) fn publish_tree_responder_binding_states(&mut self, tree: &ui::Tree) -> bool {
        publish_responder_binding_states(&tree.responder_bindings(), self.registry, self.window)
    }

    pub(crate) fn publish_composition_responder_binding_states(
        &mut self,
        composition: &ui::Composition,
    ) -> bool {
        publish_responder_binding_states(
            composition.responder_binding_map(),
            self.registry,
            self.window,
        )
    }

    pub(crate) fn sync_visual_states(&self, state: &mut WindowState) -> bool {
        let Some(visual_states) = projected_visual_states(state, self.registry, self.window) else {
            return false;
        };

        state
            .composition
            .as_mut()
            .is_some_and(|composition| composition.set_visual_states(visual_states))
    }
}

pub(crate) struct MenuPresenter<'a> {
    registry: &'a command::Registry,
    state: &'a WindowState,
    window: window::Id,
    composition: Option<&'a ui::Composition>,
}

impl<'a> MenuPresenter<'a> {
    pub(crate) fn new(
        registry: &'a command::Registry,
        state: &'a WindowState,
        window: window::Id,
        composition: Option<&'a ui::Composition>,
    ) -> Self {
        Self {
            registry,
            state,
            window,
            composition,
        }
    }

    fn context_for(
        &self,
        surface: Option<&ui::floating::Surface>,
    ) -> Option<command::call::Context> {
        surface.and_then(|surface| self.state.floating.command_context(surface).cloned())
    }

    fn context_or_window(&self, surface: &ui::floating::Surface) -> command::call::Context {
        self.context_for(Some(surface))
            .unwrap_or_else(|| command::call::Context::window(self.window))
    }
}

impl widget::Presenter for MenuPresenter<'_> {
    fn item_label(
        &self,
        surface: Option<&ui::floating::Surface>,
        item: &widget::menu::Item,
    ) -> String {
        let command = command::Key::from_action(item.action());
        item.label()
            .map(str::to_owned)
            .or_else(|| {
                self.context_for(surface)
                    .and_then(|context| self.registry.presentation_key(command, context))
                    .map(|presentation| presentation.display().to_owned())
            })
            .or_else(|| {
                self.registry
                    .command_key(command)
                    .map(|command| command.display().to_owned())
            })
            .unwrap_or_else(|| command.as_str().replace('_', " "))
    }

    fn shortcut_label(&self, action: crate::action::Key) -> Option<String> {
        self.registry
            .command_key(command::Key::from_action(action))
            .and_then(|command| command.shortcuts().first())
            .copied()
            .map(command::shortcut::Shortcut::display_label)
    }

    fn menu_can_open(&self, surface: &ui::floating::Surface, menu: &widget::menu::Menu) -> bool {
        menu_can_open_for_context_with_composition(
            self.state,
            menu,
            self.registry,
            self.context_or_window(surface),
            self.composition,
        )
    }
}

fn floating_scope_contexts(
    composition: &ui::Composition,
    floating: &floating::State,
) -> HashMap<ui::Path, command::call::Context> {
    floating
        .surfaces()
        .iter()
        .filter_map(|surface| {
            let context = floating.command_context(surface)?.clone();
            let scope = composition
                .action_scopes()
                .iter()
                .find(|scope| scope.leaf() == Some(surface.root_id()))?
                .clone();

            Some((scope, context))
        })
        .collect()
}

fn publish_responder_binding_states(
    bindings: &HashMap<ui::Path, Vec<ui::ActionBinding>>,
    registry: &mut command::Registry,
    window: window::Id,
) -> bool {
    let mut changed = false;

    for (path, bindings) in bindings {
        for binding in bindings
            .iter()
            .cloned()
            .map(command::binding::Binding::from_action)
        {
            let Some(state) = binding.state() else {
                continue;
            };

            changed |= registry.set_state_key(
                binding.command(),
                command::call::Context::path(window, path.clone()),
                state.clone(),
            );
        }
    }

    changed
}

fn projected_visual_states(
    state: &WindowState,
    registry: &command::Registry,
    window: window::Id,
) -> Option<HashMap<ui::Path, ui::VisualState>> {
    let composition = state.composition.as_ref()?;
    let mut visual_states = HashMap::new();

    for (path, route) in composition.action_map() {
        let route = command::binding::Route::from_action(*route);
        let context = context_for_path(state, window, path);
        if let Some(state) = projected_action_state(registry, route.command(), &context) {
            visual_states.insert(path.clone(), visual_state(state));
        }
    }

    for (path, intent) in composition.intents() {
        let ui::Intent::OpenMenu(menu) = intent else {
            continue;
        };
        let Some(menu) = composition.menu(*menu) else {
            continue;
        };
        let state = if menu_can_open(state, menu, registry, window) {
            command::State::available()
        } else {
            command::State::unavailable()
        };

        visual_states.insert(path.clone(), visual_state(state));
    }

    Some(visual_states)
}

fn projected_action_state(
    registry: &command::Registry,
    command: command::Key,
    context: &command::call::Context,
) -> Option<command::State> {
    let running = registry.state_key(command, context.clone()).is_running();

    match (
        registry.configured_state_override_key(command, context),
        running,
    ) {
        (Some(state), running) => Some(state.with_running(running)),
        (None, true) => Some(command::State::running()),
        (None, false) => None,
    }
}

fn visual_state(state: command::State) -> ui::VisualState {
    ui::VisualState::available_if(state.is_available())
        .with_active(state.is_active())
        .with_running(state.is_running())
}

pub fn context(state: &WindowState, window: window::Id) -> command::call::Context {
    if let Some(target) = text_input::editing_target(state)
        && state.has_responder(&target)
    {
        return command::call::Context::path(window, target);
    }

    if let Some(scope) = state.command.subject.clone() {
        return command::call::Context::with_scope(window, scope);
    }

    state
        .focused_path()
        .and_then(|path| subject_for_path(state, &path))
        .map(|path| command::call::Context::path(window, path))
        .unwrap_or_else(|| command::call::Context::window(window))
}

pub fn context_for_path(
    state: &WindowState,
    window: window::Id,
    path: &ui::Path,
) -> command::call::Context {
    match state.action_subject(path) {
        ui::ActionSubject::Origin => command::call::Context::path(window, path.clone()),
        ui::ActionSubject::Current => context(state, window),
        ui::ActionSubject::Captured => captured_context_for_path(state, window, path),
        ui::ActionSubject::Window => command::call::Context::window(window),
    }
}

pub fn set_subject(state: &mut WindowState, context: command::call::Context) -> bool {
    let scope = Some(context.scope().clone());
    if state.command.subject == scope {
        return false;
    }

    state.command.subject = scope;
    true
}

pub fn set_subject_from_path(state: &mut WindowState, path: &ui::Path) -> bool {
    let Some(path) = subject_for_path(state, path) else {
        return false;
    };
    let subject = Some(command::call::Scope::Path(path));

    if state.command.subject == subject {
        return false;
    }

    state.command.subject = subject;
    true
}

pub fn clear_subject(state: &mut WindowState) -> bool {
    let changed = state.command.subject.is_some();
    state.command.subject = None;
    changed
}

pub fn clear_stale_subject(state: &mut WindowState) -> bool {
    let Some(command::call::Scope::Path(path)) = state.command.subject.as_ref() else {
        return false;
    };

    if state.has_responder(path) {
        return false;
    }

    clear_subject(state)
}

pub fn update_scope_captures(state: &mut WindowState, window: window::Id) {
    let Some(composition) = state.composition.as_ref() else {
        state.command.scope_captures.clear();
        return;
    };
    let scopes = composition.action_scopes().to_vec();
    let responders = composition.responder_map().clone();
    let action_targets = composition.action_target_map().clone();
    let explicit_contexts = floating_scope_contexts(composition, &state.floating);

    state.command.scope_captures.retain(|scope, context| {
        scopes.contains(scope)
            && (explicit_contexts.contains_key(scope)
                || match context.scope() {
                    command::call::Scope::Path(path) => {
                        responders
                            .get(path)
                            .is_some_and(|actions| !actions.is_empty())
                            || action_targets
                                .get(path)
                                .is_some_and(|targets| !targets.is_empty())
                    }
                    command::call::Scope::Window
                    | command::call::Scope::Current
                    | command::call::Scope::Focused
                    | command::call::Scope::Captured => true,
                })
    });

    for scope in scopes {
        if let Some(context) = explicit_contexts.get(&scope) {
            state.command.scope_captures.insert(scope, context.clone());
        } else if let Some(context) = context_outside_scope(state, window, &scope) {
            state.command.scope_captures.insert(scope, context);
        } else {
            state
                .command
                .scope_captures
                .entry(scope)
                .or_insert_with(|| command::call::Context::window(window));
        }
    }
}

pub(crate) fn request_for_path(
    state: &WindowState,
    window: window::Id,
    origin: ui::Path,
    source: command::call::Source,
) -> Option<command::call::Raw> {
    let route = match state.intent(&origin) {
        Some(ui::Intent::Action(route)) => command::binding::Route::from_action(route),
        Some(ui::Intent::OpenMenu(_) | ui::Intent::OpenSubmenu(_) | ui::Intent::CloseSubmenu) => {
            return None;
        }
        None => state
            .composition
            .as_ref()
            .and_then(|composition| composition.action(&origin))
            .map(command::binding::Route::from_action)?,
    };
    let context = context_for_path(state, window, &origin);

    Some(command::call::Raw::from_route(route, source, context).with_origin(origin))
}

pub(crate) fn can_focus(
    state: &WindowState,
    registry: &command::Registry,
    window: window::Id,
    path: &ui::Path,
) -> bool {
    if !state.is_focusable(path) {
        return false;
    }

    let Some(route) = state
        .composition
        .as_ref()
        .and_then(|composition| composition.action(path))
    else {
        return true;
    };
    let route = command::binding::Route::from_action(route);

    let request = command::call::Raw::from_route(
        route,
        command::call::Source::Keyboard,
        context_for_path(state, window, path),
    )
    .with_origin(path.clone());

    can_execute_request(state, registry, &request)
}

pub(crate) fn menu_can_open(
    state: &WindowState,
    menu: &widget::menu::Menu,
    registry: &command::Registry,
    window: window::Id,
) -> bool {
    if state.composition.is_none() {
        return false;
    }

    menu_can_open_for_context(state, menu, registry, context(state, window))
}

fn menu_can_open_for_context(
    state: &WindowState,
    menu: &widget::menu::Menu,
    registry: &command::Registry,
    context: command::call::Context,
) -> bool {
    menu_can_open_for_context_with_composition(state, menu, registry, context, None)
}

fn menu_can_open_for_context_with_composition(
    state: &WindowState,
    menu: &widget::menu::Menu,
    registry: &command::Registry,
    context: command::call::Context,
    composition: Option<&ui::Composition>,
) -> bool {
    menu.actions().any(|route| {
        let route = command::binding::Route::from_action(route);
        let request =
            command::call::Raw::from_route(route, command::call::Source::Pointer, context.clone());
        let Some(request) =
            resolve_executable_request_with_composition(state, registry, request, composition)
        else {
            return false;
        };

        can_execute_menu_command(registry, &request)
    })
}

pub(crate) fn can_execute_menu_command(
    registry: &command::Registry,
    request: &command::call::Raw,
) -> bool {
    can_execute_resolved_request(registry, request)
}

pub(crate) fn can_execute_request(
    state: &WindowState,
    registry: &command::Registry,
    request: &command::call::Raw,
) -> bool {
    let Some(request) = resolve_executable_request(state, registry, request.clone()) else {
        return false;
    };

    can_execute_resolved_request(registry, &request)
}

fn can_execute_resolved_request(
    registry: &command::Registry,
    request: &command::call::Raw,
) -> bool {
    if !registry.can_execute(request) {
        return false;
    }

    request.target().is_command(request.command())
        || registry.can_invoke_key(request.command(), request.context().clone())
}

#[cfg(test)]
pub(crate) fn resolve_request(
    state: &WindowState,
    request: command::call::Raw,
) -> Option<command::call::Raw> {
    resolve_request_with_composition(state, request, None)
}

pub(crate) fn resolve_executable_request(
    state: &WindowState,
    registry: &command::Registry,
    request: command::call::Raw,
) -> Option<command::call::Raw> {
    resolve_executable_request_with_composition(state, registry, request, None)
}

fn resolve_executable_request_with_composition(
    state: &WindowState,
    registry: &command::Registry,
    request: command::call::Raw,
    composition: Option<&ui::Composition>,
) -> Option<command::call::Raw> {
    resolve_request_with_composition(state, request.clone(), composition)
        .or_else(|| resolve_window_command_request(registry, request))
}

fn resolve_window_command_request(
    registry: &command::Registry,
    request: command::call::Raw,
) -> Option<command::call::Raw> {
    if !request.target().is_command(request.command()) {
        return None;
    }

    let window = request.context().window_id();
    let window_request = request.with_context(command::call::Context::window(window));
    registry
        .can_execute(&window_request)
        .then_some(window_request)
}

fn resolve_request_with_composition(
    state: &WindowState,
    request: command::call::Raw,
    composition: Option<&ui::Composition>,
) -> Option<command::call::Raw> {
    let command = request.command();
    let target = request.target();
    let requested_context = request.context().clone();
    let window = requested_context.window_id();
    let resolved = match requested_context.scope() {
        command::call::Scope::Path(path) => {
            command_target_for_path_with_composition(state, command, target, path, composition)
                .map(|path| command::call::Context::path(window, path))?
        }
        command::call::Scope::Window => command::call::Context::window(window),
        command::call::Scope::Current | command::call::Scope::Captured => context(state, window),
        command::call::Scope::Focused => state
            .focused_path()
            .and_then(|path| subject_for_path_with_composition(state, &path, composition))
            .map(|path| command::call::Context::path(window, path))?,
    };

    Some(request.with_context(resolved))
}

fn subject_for_path(state: &WindowState, path: &ui::Path) -> Option<ui::Path> {
    subject_for_path_with_composition(state, path, None)
}

fn subject_for_path_with_composition(
    state: &WindowState,
    path: &ui::Path,
    composition: Option<&ui::Composition>,
) -> Option<ui::Path> {
    nearest_path(path, |path| has_responder(state, composition, path))
}

fn captured_context_for_path(
    state: &WindowState,
    window: window::Id,
    path: &ui::Path,
) -> command::call::Context {
    nearest_path(path, |path| state.command.scope_captures.contains_key(path))
        .and_then(|scope| state.command.scope_captures.get(&scope).cloned())
        .unwrap_or_else(|| command::call::Context::window(window))
}

fn context_outside_scope(
    state: &WindowState,
    window: window::Id,
    scope: &ui::Path,
) -> Option<command::call::Context> {
    if let Some(target) = text_input::editing_target(state)
        && !target.is_descendant_of(scope)
        && state.has_responder(&target)
    {
        return Some(command::call::Context::path(window, target));
    }

    if let Some(command::call::Scope::Path(path)) = state.command.subject.as_ref()
        && !path.is_descendant_of(scope)
    {
        return Some(command::call::Context::path(window, path.clone()));
    }

    state
        .focused_path()
        .filter(|path| !path.is_descendant_of(scope))
        .and_then(|path| subject_for_path(state, &path))
        .map(|path| command::call::Context::path(window, path))
}

fn command_target_for_path_with_composition(
    state: &WindowState,
    command: command::Key,
    target: command::target::Kind,
    path: &ui::Path,
    composition: Option<&ui::Composition>,
) -> Option<ui::Path> {
    let action = command.action();
    let target = target.action();
    let composition = composition.or_else(|| state.composition.as_ref())?;

    nearest_path(path, |path| {
        composition
            .responders(path)
            .is_some_and(|actions| actions.contains(&action))
            || composition
                .action_targets(path)
                .is_some_and(|targets| targets.contains(&target))
            || (composition
                .action(path)
                .is_some_and(|route| route.key() == action && route.target() == target)
                && action_subject(state, Some(composition), path) == ui::ActionSubject::Origin)
    })
}

fn has_responder(
    state: &WindowState,
    composition: Option<&ui::Composition>,
    path: &ui::Path,
) -> bool {
    composition.map_or_else(
        || state.has_responder(path),
        |composition| composition.has_responder(path),
    )
}

fn action_subject(
    state: &WindowState,
    composition: Option<&ui::Composition>,
    path: &ui::Path,
) -> ui::ActionSubject {
    composition.map_or_else(
        || state.action_subject(path),
        |composition| composition.action_subject(path),
    )
}

fn nearest_path(path: &ui::Path, matches: impl Fn(&ui::Path) -> bool) -> Option<ui::Path> {
    for length in (1..=path.ids().len()).rev() {
        let candidate = ui::Path::new(path.ids()[..length].to_vec());

        if matches(&candidate) {
            return Some(candidate);
        }
    }

    None
}
