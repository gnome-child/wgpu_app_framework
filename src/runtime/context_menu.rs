use std::{any::TypeId, collections::HashSet};

use super::{Runtime, services};
use crate::{
    command, context as command_context, geometry, interaction, responder, response, session,
    state, view, window,
};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    fn captured_context_path(
        &self,
        window: window::Id,
    ) -> Option<(Vec<view::ContextOwner>, responder::Traversal)> {
        let owner = self
            .session
            .interaction(window)?
            .open_menu()?
            .context_owner()?;
        let path = self.composition.get(window)?.context_path_for_node(owner);
        (!path.is_empty()).then(|| {
            let traversal = context_traversal(&path, &self.session, window);
            (path, traversal)
        })
    }

    pub(in crate::runtime) fn context_menu_scope(
        &self,
        window: window::Id,
    ) -> Option<session::CommandScope> {
        let (path, _) = self.captured_context_path(window)?;
        let owner = path.last()?;
        Some(scope_for(owner))
    }

    pub(in crate::runtime) fn context_menu_projection(
        &mut self,
        window: window::Id,
    ) -> Option<view::ContextMenu> {
        let menu = self.session.interaction(window)?.open_menu()?.clone();
        let owner_id = menu.context_owner()?;
        let anchor = menu.context_anchor()?;
        let available = self
            .presented_layout(window)
            .and_then(|layout| layout.context_available_for_node(owner_id))?;
        let (path, traversal) = self.captured_context_path(window)?;
        let sections = self.context_sections(window, &path, traversal);
        (!sections.is_empty())
            .then(|| view::ContextMenu::new(owner_id, anchor, available, sections))
    }

    pub(in crate::runtime) fn open_context_menu_for_focus(
        &mut self,
        window: window::Id,
    ) -> std::result::Result<crate::input::Outcome, crate::command::Error> {
        let Some(focus) = self.session.focused(window) else {
            return Ok(crate::input::Outcome::ignored());
        };
        let Some((node, anchor)) = self.presented_layout(window).and_then(|layout| {
            let focused = layout.frame_for_focus(focus)?;
            let rect = focused.rect();
            Some((
                focused.node_id(),
                geometry::Point::new(rect.x(), rect.bottom()),
            ))
        }) else {
            return Ok(crate::input::Outcome::ignored());
        };
        let path = self
            .composition
            .get(window)
            .map(|composition| composition.context_path_for_node(node))
            .unwrap_or_default();
        if path.is_empty() {
            return Ok(crate::input::Outcome::ignored());
        }
        let traversal = context_traversal(&path, &self.session, window);
        if self.context_sections(window, &path, traversal).is_empty() {
            return Ok(crate::input::Outcome::ignored());
        }

        let changed = self
            .session
            .open_menu(window, interaction::Menu::context(node, anchor));
        Ok(self.window_outcome(
            window,
            false,
            if changed {
                response::Effect::Rebuild
            } else {
                response::Effect::None
            },
        ))
    }

    fn context_sections(
        &mut self,
        window: window::Id,
        path: &[view::ContextOwner],
        traversal: responder::Traversal,
    ) -> Vec<Vec<command::ResolvedAction>> {
        let scopes = path.iter().map(scope_for).collect::<Vec<_>>();
        let task_frame = scopes.len().saturating_sub(1);
        let responder_path = responder::Path::new(
            scopes.iter().copied().map(session::CommandScope::routing),
            task_frame,
        );
        let mut consumed = HashSet::<TypeId>::new();
        let mut sections = Vec::new();

        for ordinal in responder_path.ordinals(traversal) {
            let owner = &path[ordinal];
            let scope = scope_for(owner);
            let responder = if owner.is_application() {
                Some(responder::Builder::<M>::app_identity())
            } else {
                owner.responder()
            };
            let mut targets = responder
                .into_iter()
                .flat_map(|identity| {
                    self.responders.target_types_for(identity).into_iter().map(
                        move |command_type| (command_type, responder::Route::Responder(identity)),
                    )
                })
                .collect::<Vec<_>>();
            targets.extend(services::contextual_targets(
                &self.composition,
                window,
                owner.service(),
                owner.focus(),
                owner.table(),
            ));
            let binding = owner.binding().map(view::Binding::trigger);
            let population = self.registry.population();
            let candidates = population.context_candidates(binding, targets);
            let cx = command_context::Context::with_clipboard_source(
                &mut self.clipboard,
                command_context::Source::Menu,
            );
            let services = services::Services::new(
                &mut self.timeline,
                &mut self.session,
                &mut self.composition,
                Some(window),
                scope,
            );
            let mut chain = self
                .responders
                .chain_for_path(&mut self.store, &responder_path, traversal)
                .with_service(services);
            let section = population
                .resolve_claimed(candidates, &mut chain, &cx)
                .into_iter()
                .filter(|action| consumed.insert(action.command_type()))
                .collect::<Vec<_>>();
            if !section.is_empty() {
                sections.push(section);
            }
        }
        sections
    }
}

fn context_traversal(
    path: &[view::ContextOwner],
    session: &crate::session::Session,
    window: window::Id,
) -> responder::Traversal {
    let active = session
        .interaction(window)
        .and_then(|interaction| interaction.text_input().target());
    path.last()
        .and_then(view::ContextOwner::focus)
        .and_then(crate::session::Focus::text_target)
        .as_ref()
        .is_some_and(|target| Some(target) == active)
        .then_some(responder::Traversal::Task)
        .unwrap_or(responder::Traversal::Inspection)
}

fn scope_for(owner: &view::ContextOwner) -> session::CommandScope {
    let responder = if owner.is_application() {
        None
    } else {
        owner.responder()
    };
    session::CommandScope::contextual(responder, owner.focus(), owner.table())
}

impl<M: state::State, E: Send + 'static> Runtime<M, E, view::View> {
    pub(crate) fn open_context_menu_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
    ) -> std::result::Result<crate::input::Outcome, crate::command::Error> {
        self.open_context_menu_on_surface(window, size, point, crate::popup::Surface::Parent)
    }

    pub(crate) fn open_context_menu_on_surface(
        &mut self,
        window: window::Id,
        _size: geometry::Size,
        point: geometry::Point,
        surface: crate::popup::Surface,
    ) -> std::result::Result<crate::input::Outcome, crate::command::Error> {
        let Some(node) = self
            .presented_layout(window)
            .and_then(|layout| layout.context_node_at_surface(point, surface))
        else {
            return Ok(crate::input::Outcome::ignored());
        };
        let path = self
            .composition
            .get(window)
            .map(|composition| composition.context_path_for_node(node))
            .unwrap_or_default();
        if path.is_empty() {
            return Ok(crate::input::Outcome::ignored());
        }

        let mut departure = None;
        let mut selection_changed = false;
        if let Some(row) = path.iter().find_map(view::ContextOwner::row) {
            let model = self
                .composition
                .get(window)
                .and_then(|composition| composition.virtual_list_model(row.table()))
                .cloned();
            if let Some(model) = model
                && !self
                    .session
                    .selection(window, row.table())
                    .is_some_and(|selection| selection.contains(row.key()))
            {
                let transition = match self.commit_and_deactivate_focused_text_box(window)? {
                    Some(transition) if !transition.is_accepted() => {
                        return Ok(transition.into_outcome());
                    }
                    transition => transition,
                };
                departure = transition;
                selection_changed |= self.session.select_virtual_row(
                    window,
                    &model,
                    row.key(),
                    row.index(),
                    false,
                    false,
                );
            }
        }

        let traversal = context_traversal(&path, &self.session, window);
        if self.context_sections(window, &path, traversal).is_empty() {
            let outcome = if selection_changed {
                self.window_outcome(window, false, response::Effect::Rebuild)
            } else {
                crate::input::Outcome::ignored()
            };
            return Ok(match departure {
                Some(transition) => transition.then(outcome),
                None => outcome,
            });
        }

        let changed = self
            .session
            .open_menu(window, interaction::Menu::context(node, point));
        let effect = if changed || selection_changed {
            response::Effect::Rebuild
        } else {
            response::Effect::None
        };
        let outcome = self.window_outcome(window, false, effect);
        Ok(match departure {
            Some(transition) => transition.then(outcome),
            None => outcome,
        })
    }
}
