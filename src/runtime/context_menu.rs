use std::{any::TypeId, collections::HashSet};

use super::{Runtime, services};
use crate::{
    command, context as command_context, geometry, interaction, responder, response, state, view,
    window,
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
            let traversal = context_traversal(&path, self.session.editing_table_cell(window));
            (path, traversal)
        })
    }

    pub(in crate::runtime) fn context_menu_scope(
        &self,
        window: window::Id,
    ) -> Option<responder::Scope> {
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
    ) -> std::result::Result<crate::input::Outcome, crate::error::Error> {
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
        let traversal = context_traversal(&path, self.session.editing_table_cell(window));
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
        let responder_path = responder::Path::new(scopes, task_frame);
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
                &self.session,
                &self.composition,
                window,
                owner.focus(),
                owner.table(),
            ));
            let binding = owner.binding().map(view::Binding::trigger);
            let candidates = self.registry.local_candidates(binding, targets);
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
            let section = self
                .registry
                .resolve_candidates(candidates, &mut chain, &cx)
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
    editing_cell: Option<crate::table::Cell>,
) -> responder::Traversal {
    editing_cell
        .is_some_and(|cell| path.iter().any(|owner| owner.cell() == Some(cell)))
        .then_some(responder::Traversal::Task)
        .unwrap_or(responder::Traversal::Inspection)
}

fn scope_for(owner: &view::ContextOwner) -> responder::Scope {
    let responder = if owner.is_application() {
        None
    } else {
        owner.responder()
    };
    responder::Scope::contextual_table(responder, owner.focus(), owner.table())
}

impl<M: state::State, E: Send + 'static> Runtime<M, E, view::View> {
    pub(crate) fn open_context_menu_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
    ) -> std::result::Result<crate::input::Outcome, crate::error::Error> {
        self.open_context_menu_on_surface(window, size, point, crate::popup::Surface::Parent)
    }

    pub(crate) fn open_context_menu_on_surface(
        &mut self,
        window: window::Id,
        _size: geometry::Size,
        point: geometry::Point,
        surface: crate::popup::Surface,
    ) -> std::result::Result<crate::input::Outcome, crate::error::Error> {
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

        let traversal = context_traversal(&path, self.session.editing_table_cell(window));
        if self.context_sections(window, &path, traversal).is_empty() {
            if selection_changed {
                return Ok(self.window_outcome(window, false, response::Effect::Rebuild));
            }
            return Ok(crate::input::Outcome::ignored());
        }

        let changed = self
            .session
            .open_menu(window, interaction::Menu::context(node, point));
        let effect = if changed || selection_changed {
            response::Effect::Rebuild
        } else {
            response::Effect::None
        };
        Ok(self.window_outcome(window, false, effect))
    }
}
