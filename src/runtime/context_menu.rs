use super::{Runtime, services};
use crate::{
    command, context as command_context, geometry, interaction, responder, response, state, view,
    window,
};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::runtime) fn context_menu_scope(
        &self,
        window: window::Id,
    ) -> Option<responder::Scope> {
        let owner_id = self
            .session
            .interaction(window)?
            .open_menu()?
            .context_owner()?;
        let owner = self
            .composition
            .get(window)?
            .context_owner_for_node(owner_id)?;
        let responder = if owner.is_application() {
            Some(responder::Builder::<M>::app_identity())
        } else {
            owner.responder()
        };
        Some(responder::Scope::contextual(responder, owner.focus()))
    }

    pub(in crate::runtime) fn context_menu_projection(
        &mut self,
        window: window::Id,
    ) -> Option<view::ContextMenu> {
        let menu = self.session.interaction(window)?.open_menu()?.clone();
        let owner_id = menu.context_owner()?;
        let anchor = menu.context_anchor()?;
        let owner = self
            .composition
            .get(window)?
            .context_owner_for_node(owner_id)?;
        let available = self
            .presented_layout(window)
            .and_then(|layout| layout.context_available_for_node(owner_id))?;
        let actions = self.context_actions(window, &owner);
        (!actions.is_empty()).then(|| view::ContextMenu::new(anchor, available, actions))
    }

    pub(in crate::runtime) fn open_context_menu_for_focus(
        &mut self,
        window: window::Id,
    ) -> std::result::Result<crate::input::Outcome, crate::error::Error> {
        let Some(focus) = self.session.focused(window) else {
            return Ok(crate::input::Outcome::ignored());
        };
        let Some((owner, anchor)) = self.presented_layout(window).and_then(|layout| {
            let focused = layout.frame_for_focus(focus)?;
            let owner = self
                .composition
                .get(window)?
                .context_owner_for_node(focused.node_id())?;
            let rect = layout.frame_for_node(owner.node_id())?.rect();
            Some((owner, geometry::Point::new(rect.x(), rect.bottom())))
        }) else {
            return Ok(crate::input::Outcome::ignored());
        };
        if self.context_actions(window, &owner).is_empty() {
            return Ok(crate::input::Outcome::ignored());
        }

        let changed = self
            .session
            .open_menu(window, interaction::Menu::context(owner.node_id(), anchor));
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

    fn context_actions(
        &mut self,
        window: window::Id,
        owner: &view::ContextOwner,
    ) -> Vec<command::ResolvedAction> {
        let responder = if owner.is_application() {
            Some(responder::Builder::<M>::app_identity())
        } else {
            owner.responder()
        };
        let scope = responder::Scope::contextual(responder, owner.focus());
        let mut targets = responder
            .into_iter()
            .flat_map(|identity| {
                self.responders
                    .target_types_for(identity)
                    .into_iter()
                    .map(move |command_type| (command_type, responder::Route::Responder(identity)))
            })
            .collect::<Vec<_>>();
        targets.extend(services::contextual_targets(
            &self.composition,
            window,
            owner.focus(),
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
            .chain_for_scope(&mut self.store, scope)
            .with_service(services);

        self.registry
            .resolve_candidates(candidates, &mut chain, &cx)
            .into_iter()
            .collect()
    }
}

impl<M: state::State, E: Send + 'static> Runtime<M, E, view::View> {
    pub(crate) fn open_context_menu_at(
        &mut self,
        window: window::Id,
        _size: geometry::Size,
        point: geometry::Point,
    ) -> std::result::Result<crate::input::Outcome, crate::error::Error> {
        let Some(node) = self
            .presented_layout(window)
            .and_then(|layout| layout.context_node_at(point))
        else {
            return Ok(crate::input::Outcome::ignored());
        };
        let Some(owner) = self
            .composition
            .get(window)
            .and_then(|composition| composition.context_owner_for_node(node))
        else {
            return Ok(crate::input::Outcome::ignored());
        };
        if self.context_actions(window, &owner).is_empty() {
            return Ok(crate::input::Outcome::ignored());
        }

        let changed = self
            .session
            .open_menu(window, interaction::Menu::context(owner.node_id(), point));
        let effect = if changed {
            response::Effect::Rebuild
        } else {
            response::Effect::None
        };
        Ok(self.window_outcome(window, false, effect))
    }
}
