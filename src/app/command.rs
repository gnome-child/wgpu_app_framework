use crate::{command, ui, window};

use super::{state::WindowState, text_input};

pub fn context(state: &WindowState, window: window::Id) -> command::call::Context {
    if let Some(target) = text_input::editing_target(state)
        && state.has_responder(&target)
    {
        return command::call::Context::path(window, target);
    }

    if let Some(scope) = state.command_subject.clone() {
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
    match state.command_subject(path) {
        ui::CommandSubject::Origin => command::call::Context::path(window, path.clone()),
        ui::CommandSubject::Current => context(state, window),
        ui::CommandSubject::Captured => captured_context_for_path(state, window, path),
        ui::CommandSubject::Window => command::call::Context::window(window),
    }
}

pub fn set_subject(state: &mut WindowState, context: command::call::Context) -> bool {
    let scope = Some(context.scope().clone());
    if state.command_subject == scope {
        return false;
    }

    state.command_subject = scope;
    true
}

pub fn set_subject_from_path(state: &mut WindowState, path: &ui::Path) -> bool {
    let Some(path) = subject_for_path(state, path) else {
        return false;
    };
    let subject = Some(command::call::Scope::Path(path));

    if state.command_subject == subject {
        return false;
    }

    state.command_subject = subject;
    true
}

pub fn clear_subject(state: &mut WindowState) -> bool {
    let changed = state.command_subject.is_some();
    state.command_subject = None;
    changed
}

pub fn clear_stale_subject(state: &mut WindowState) -> bool {
    let Some(command::call::Scope::Path(path)) = state.command_subject.as_ref() else {
        return false;
    };

    if state.has_responder(path) {
        return false;
    }

    clear_subject(state)
}

pub fn update_scope_captures(state: &mut WindowState, window: window::Id) {
    let Some(composition) = state.composition.as_ref() else {
        state.command_scope_captures.clear();
        return;
    };
    let scopes = composition.command_scopes().to_vec();
    let responders = composition.responder_map().clone();
    let command_targets = composition.command_target_map().clone();
    let explicit_contexts = composition.command_scope_contexts().clone();

    state.command_scope_captures.retain(|scope, context| {
        scopes.contains(scope)
            && (explicit_contexts.contains_key(scope)
                || match context.scope() {
                    command::call::Scope::Path(path) => {
                        responders
                            .get(path)
                            .is_some_and(|commands| !commands.is_empty())
                            || command_targets
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
            state.command_scope_captures.insert(scope, context.clone());
        } else if let Some(context) = context_outside_scope(state, window, &scope) {
            state.command_scope_captures.insert(scope, context);
        } else if !state.command_scope_captures.contains_key(&scope) {
            state
                .command_scope_captures
                .insert(scope, command::call::Context::window(window));
        }
    }
}

pub fn resolve_request(
    state: &WindowState,
    _registry: &command::Registry,
    request: command::call::Raw,
) -> command::call::Raw {
    let command = request.command();
    let target = request.target();
    let requested_context = request.context().clone();
    let window = requested_context.window_id();
    let resolved = match requested_context.scope() {
        command::call::Scope::Path(path) => command_target_for_path(state, command, target, path)
            .map(|path| command::call::Context::path(window, path))
            .unwrap_or_else(|| command::call::Context::window(window)),
        command::call::Scope::Window => command::call::Context::window(window),
        command::call::Scope::Current | command::call::Scope::Captured => context(state, window),
        command::call::Scope::Focused => state
            .focused_path()
            .and_then(|path| subject_for_path(state, &path))
            .map(|path| command::call::Context::path(window, path))
            .unwrap_or_else(|| command::call::Context::window(window)),
    };

    request.with_context(resolved)
}

fn subject_for_path(state: &WindowState, path: &ui::Path) -> Option<ui::Path> {
    nearest_path(path, |path| state.has_responder(path))
}

fn captured_context_for_path(
    state: &WindowState,
    window: window::Id,
    path: &ui::Path,
) -> command::call::Context {
    nearest_path(path, |path| state.command_scope_captures.contains_key(path))
        .and_then(|scope| state.command_scope_captures.get(&scope).cloned())
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

    if let Some(command::call::Scope::Path(path)) = state.command_subject.as_ref()
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

fn command_target_for_path(
    state: &WindowState,
    command: command::Key,
    target: command::target::Kind,
    path: &ui::Path,
) -> Option<ui::Path> {
    nearest_path(path, |path| {
        let Some(composition) = state.composition.as_ref() else {
            return false;
        };

        composition
            .responders(path)
            .is_some_and(|commands| commands.contains(&command))
            || composition
                .command_targets(path)
                .is_some_and(|targets| targets.contains(&target))
            || (composition
                .command(path)
                .is_some_and(|route| route.command() == command && route.target() == target)
                && state.command_subject(path) == ui::CommandSubject::Origin)
    })
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
