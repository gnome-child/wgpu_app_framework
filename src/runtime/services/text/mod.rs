use std::any::{Any, TypeId};

use crate::{
    command, composition, context as command_context, document, responder, session, window,
};
use crate::{target::Target, timeline};

mod focused;

use super::target as service_target;
use focused::FocusedTextBox;

const RESPONDER_NAME: &str = "focused_text";

struct Text<'a> {
    session: &'a mut session::Session,
    composition: &'a composition::Store,
    window: window::Id,
    focus: session::Focus,
}

impl<C> service_target::Provider<C> for Text<'_>
where
    C: command::Command,
    for<'target> FocusedTextBox<'target>: Target<C>,
{
    type Target<'target>
        = FocusedTextBox<'target>
    where
        Self: 'target;

    fn target(&mut self) -> Self::Target<'_> {
        FocusedTextBox::new(self.session, self.composition, self.window, self.focus)
    }
}

pub(super) fn state(
    session: &mut session::Session,
    composition: &composition::Store,
    window: Option<window::Id>,
    command_type: TypeId,
    command_name: &'static str,
    args: &dyn Any,
    cx: &command_context::Context,
) -> crate::error::Result<Option<command::State>> {
    let Some((window, focus)) = base_text_for(session, composition, window) else {
        return Ok(None);
    };
    let mut text = Text {
        session,
        composition,
        window,
        focus,
    };
    let targets = targets();

    service_target::state(
        RESPONDER_NAME,
        &targets,
        &mut text,
        command_type,
        command_name,
        args,
        cx,
    )
}

pub(super) fn claim(
    session: &mut session::Session,
    composition: &composition::Store,
    window: Option<window::Id>,
    command_type: TypeId,
    command_name: &'static str,
    args: &dyn Any,
    cx: &command_context::Context,
) -> crate::error::Result<Option<responder::Claim>> {
    let Some((window, focus)) = base_text_for(session, composition, window) else {
        return Ok(None);
    };
    let mut text = Text {
        session,
        composition,
        window,
        focus,
    };
    let targets = targets();

    Ok(service_target::claim(
        RESPONDER_NAME,
        &targets,
        &mut text,
        command_type,
        command_name,
        args,
        cx,
    )?
    .map(|claim| responder::Claim::service(responder::Kind::Focused, RESPONDER_NAME, claim.state)))
}

pub(super) fn owns_command(
    session: &session::Session,
    composition: &composition::Store,
    window: Option<window::Id>,
    command_type: TypeId,
) -> bool {
    let targets = targets();

    service_target::handles(&targets, command_type)
        && base_text_for(session, composition, window).is_some()
}

pub(super) fn invoke(
    session: &mut session::Session,
    composition: &composition::Store,
    window: Option<window::Id>,
    command_type: TypeId,
    command_name: &'static str,
    args: Box<dyn Any + Send>,
    cx: &mut command_context::Context,
) -> Option<crate::response::AnyResponse> {
    let targets = targets();
    if !service_target::handles(&targets, command_type) {
        return None;
    }

    let Some((window, focus)) = base_text_for(session, composition, window) else {
        return None;
    };

    if !session
        .focused(window)
        .is_some_and(|current| current.same_target(&focus))
    {
        session.focus(window, focus);
    }

    let mut text = Text {
        session,
        composition,
        window,
        focus,
    };

    service_target::invoke(
        RESPONDER_NAME,
        &targets,
        &mut text,
        command_type,
        command_name,
        args,
        cx,
    )
}

fn base_text_for(
    session: &session::Session,
    composition: &composition::Store,
    window: Option<window::Id>,
) -> Option<(window::Id, session::Focus)> {
    let window = window?;
    let focus = session.command_focus(window)?;
    composition.get(window)?.view().text_box_text(focus)?;

    Some((window, focus))
}

fn targets<'a>() -> [service_target::AnyTarget<Text<'a>>; 7] {
    [
        service_target::AnyTarget::for_provider::<document::SelectAll>(),
        service_target::AnyTarget::for_provider::<document::Copy>(),
        service_target::AnyTarget::for_provider::<document::Cut>(),
        service_target::AnyTarget::for_provider::<document::Delete>(),
        service_target::AnyTarget::for_provider::<document::Paste>(),
        service_target::AnyTarget::for_provider::<timeline::Undo>(),
        service_target::AnyTarget::for_provider::<timeline::Redo>(),
    ]
}
