use std::any::{Any, TypeId};

use crate::{
    command, composition, context as command_context, document, interaction, responder, response,
    session, target::Target, window,
};

use super::target as service_target;

pub(super) const RESPONDER_NAME: &str = "table_selection";

struct Table<'a> {
    session: &'a mut session::Session,
    model: &'a crate::virtual_list::Model,
    window: window::Id,
}

struct SelectionTarget<'a> {
    session: &'a mut session::Session,
    model: &'a crate::virtual_list::Model,
    window: window::Id,
}

impl service_target::Provider<document::SelectAll> for Table<'_> {
    type Target<'target>
        = SelectionTarget<'target>
    where
        Self: 'target;

    fn target(&mut self) -> Self::Target<'_> {
        SelectionTarget {
            session: &mut *self.session,
            model: self.model,
            window: self.window,
        }
    }
}

impl Target<document::SelectAll> for SelectionTarget<'_> {
    fn state(&self, _: &(), _: &command_context::Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(
        &mut self,
        _: (),
        _: &mut command_context::Context,
    ) -> response::Response<document::Outcome> {
        let changed = self
            .session
            .select_all_virtual_rows(self.window, self.model);
        response::Response::output(document::Outcome::from_text_change(false, changed, false))
            .with_effect(
                changed
                    .then_some(response::Effect::Rebuild)
                    .unwrap_or_default(),
            )
    }
}

fn table_model(
    composition: &composition::Store,
    window: Option<window::Id>,
    table: Option<interaction::Id>,
) -> Option<(window::Id, &crate::virtual_list::Model)> {
    let window = window?;
    let table = table?;
    let model = composition.get(window)?.virtual_list_model(table)?;
    Some((window, model))
}

fn table_for<'a>(
    session: &'a mut session::Session,
    composition: &'a composition::Store,
    window: Option<window::Id>,
    table: Option<interaction::Id>,
) -> Option<Table<'a>> {
    let (window, model) = table_model(composition, window, table)?;
    Some(Table {
        session,
        model,
        window,
    })
}

pub(super) fn claim(
    session: &mut session::Session,
    composition: &composition::Store,
    window: Option<window::Id>,
    table: Option<interaction::Id>,
    scope_kind: responder::Kind,
    command_type: TypeId,
    command_name: &'static str,
    args: &dyn Any,
    cx: &command_context::Context,
) -> command::Result<Option<responder::Claim>> {
    let Some(mut service) = table_for(session, composition, window, table) else {
        return Ok(None);
    };
    Ok(service_target::claim(
        RESPONDER_NAME,
        &targets(),
        &mut service,
        command_type,
        command_name,
        args,
        cx,
    )?
    .map(|claim| responder::Claim::service(scope_kind, RESPONDER_NAME, claim.state)))
}

pub(super) fn invoke(
    session: &mut session::Session,
    composition: &composition::Store,
    window: Option<window::Id>,
    table: Option<interaction::Id>,
    command_type: TypeId,
    command_name: &'static str,
    args: Box<dyn Any + Send>,
    cx: &mut command_context::Context,
) -> Option<response::AnyResponse> {
    let mut service = table_for(session, composition, window, table)?;
    service_target::invoke(
        RESPONDER_NAME,
        &targets(),
        &mut service,
        command_type,
        command_name,
        args,
        cx,
    )
}

pub(super) fn contextual_target_types(
    composition: &composition::Store,
    window: Option<window::Id>,
    table: Option<interaction::Id>,
) -> Vec<TypeId> {
    table_model(composition, window, table)
        .map(|_| vec![TypeId::of::<document::SelectAll>()])
        .unwrap_or_default()
}

fn targets<'a>() -> [service_target::AnyTarget<Table<'a>>; 1] {
    [service_target::AnyTarget::for_provider::<document::SelectAll>()]
}
