use crate::{action, ui, window};

use super::state::WindowState;

pub fn context(state: &WindowState, window: window::Id) -> action::Context {
    if let Some(scope) = state.command_subject.clone() {
        return action::Context::with_scope(window, scope);
    }

    state
        .focused_path()
        .and_then(|path| subject_for_path(state, &path))
        .map(|path| action::Context::path(window, path))
        .unwrap_or_else(|| action::Context::window(window))
}

pub fn context_for_path(
    state: &WindowState,
    window: window::Id,
    path: &ui::Path,
) -> action::Context {
    match state.action_target(path) {
        ui::ActionTarget::Origin => action::Context::path(window, path.clone()),
        ui::ActionTarget::Command => context(state, window),
        ui::ActionTarget::Captured => captured_context_for_path(state, window, path),
        ui::ActionTarget::Window => action::Context::window(window),
    }
}

pub fn set_subject(state: &mut WindowState, context: action::Context) -> bool {
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
    let subject = Some(action::Scope::Path(path));

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
    let Some(action::Scope::Path(path)) = state.command_subject.as_ref() else {
        return false;
    };

    if state.has_responder(path) {
        return false;
    }

    clear_subject(state)
}

pub fn update_scope_captures(state: &mut WindowState, window: window::Id) {
    let scopes = state.command_scopes.clone();
    let responders = &state.responders;
    state.command_scope_captures.retain(|scope, context| {
        scopes.contains(scope)
            && match context.scope() {
                action::Scope::Path(path) => responders
                    .get(path)
                    .is_some_and(|actions| !actions.is_empty()),
                action::Scope::Window => true,
            }
    });

    for scope in scopes {
        if let Some(context) = context_outside_scope(state, window, &scope) {
            state.command_scope_captures.insert(scope, context);
        } else if !state.command_scope_captures.contains_key(&scope) {
            state
                .command_scope_captures
                .insert(scope, action::Context::window(window));
        }
    }
}

pub fn resolve_request(state: &WindowState, request: action::Request) -> action::Request {
    let action = request.action();
    let target = request.target().clone();
    let window = target.window_id();
    let resolved = match target.scope() {
        action::Scope::Path(path) => handler_for_path(state, action, path)
            .map(|path| action::Context::path(window, path))
            .unwrap_or_else(|| action::Context::window(window)),
        action::Scope::Window => action::Context::window(window),
    };

    request.with_target(resolved)
}

fn subject_for_path(state: &WindowState, path: &ui::Path) -> Option<ui::Path> {
    nearest_path(path, |path| state.has_responder(path))
}

fn captured_context_for_path(
    state: &WindowState,
    window: window::Id,
    path: &ui::Path,
) -> action::Context {
    nearest_path(path, |path| state.command_scope_captures.contains_key(path))
        .and_then(|scope| state.command_scope_captures.get(&scope).cloned())
        .unwrap_or_else(|| action::Context::window(window))
}

fn context_outside_scope(
    state: &WindowState,
    window: window::Id,
    scope: &ui::Path,
) -> Option<action::Context> {
    if let Some(action::Scope::Path(path)) = state.command_subject.as_ref()
        && !path.is_descendant_of(scope)
    {
        return Some(action::Context::path(window, path.clone()));
    }

    state
        .focused_path()
        .filter(|path| !path.is_descendant_of(scope))
        .and_then(|path| subject_for_path(state, &path))
        .map(|path| action::Context::path(window, path))
}

fn handler_for_path(state: &WindowState, action: action::Id, path: &ui::Path) -> Option<ui::Path> {
    nearest_path(path, |path| {
        state
            .responders
            .get(path)
            .is_some_and(|actions| actions.contains(&action))
            || (state.actions.get(path) == Some(&action)
                && state.action_target(path) == ui::ActionTarget::Origin)
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
