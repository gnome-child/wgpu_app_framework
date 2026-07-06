use std::any::{Any, TypeId};

use crate::scratch::{
    command as framework_command, composition, context as command_context, document, session,
    window,
};
use crate::scratch::{response::AnyResponse, target::Target, timeline};

mod target;

use super::target as service_target;
use target::FocusedTextBox;

const RESPONDER_NAME: &str = "focused_text";

struct Text<'a> {
    session: &'a mut session::Session,
    composition: &'a composition::Store,
    window: window::Id,
    focus: session::Focus,
}

impl Text<'_> {
    fn target(&mut self) -> FocusedTextBox<'_> {
        FocusedTextBox::new(self.session, self.composition, self.window, self.focus)
    }
}

pub(super) fn state(
    session: &mut session::Session,
    composition: &composition::Store,
    window: Option<window::Id>,
    command_type: TypeId,
    args: &dyn Any,
    cx: &command_context::Context,
) -> crate::scratch::error::Result<Option<framework_command::State>> {
    let Some((window, focus)) = base_text_for(session, composition, window) else {
        return Ok(None);
    };
    let mut text = Text {
        session,
        composition,
        window,
        focus,
    };

    service_target::state(
        RESPONDER_NAME,
        targets(),
        &mut text,
        command_type,
        command_name(command_type),
        args,
        cx,
    )
}

pub(in crate::scratch::runtime) fn has_target(
    session: &session::Session,
    composition: &composition::Store,
    window: Option<window::Id>,
    command_type: TypeId,
) -> bool {
    targets()
        .iter()
        .any(|target| target.handles_type(command_type))
        && base_text_for(session, composition, window).is_some()
}

pub(super) fn invoke(
    session: &mut session::Session,
    composition: &composition::Store,
    window: Option<window::Id>,
    command_type: TypeId,
    args: Box<dyn Any + Send>,
    cx: &mut command_context::Context,
) -> Option<crate::scratch::response::AnyResponse> {
    let Some((window, focus)) = base_text_for(session, composition, window) else {
        return None;
    };

    if !session
        .focused(window)
        .is_some_and(|current| current.same_target(&focus))
    {
        session.focus(window, focus);
    }

    let targets = targets();
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
        command_name(command_type),
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
        service_target::AnyTarget::new::<document::SelectAll>(select_all_state, select_all_invoke),
        service_target::AnyTarget::new::<document::Copy>(copy_state, copy_invoke),
        service_target::AnyTarget::new::<document::Cut>(cut_state, cut_invoke),
        service_target::AnyTarget::new::<document::Delete>(delete_state, delete_invoke),
        service_target::AnyTarget::new::<document::Paste>(paste_state, paste_invoke),
        service_target::AnyTarget::new::<timeline::Undo>(undo_state, undo_invoke),
        service_target::AnyTarget::new::<timeline::Redo>(redo_state, redo_invoke),
    ]
}

macro_rules! text_target {
    ($state:ident, $invoke:ident, $command:ty) => {
        fn $state(
            text: &mut Text<'_>,
            args: &dyn Any,
            cx: &command_context::Context,
        ) -> crate::scratch::error::Result<framework_command::State> {
            let args = service_target::args::<$command>(args)?;
            let target = text.target();

            Ok(Target::<$command>::state(&target, args, cx))
        }

        fn $invoke(
            text: &mut Text<'_>,
            args: Box<dyn Any + Send>,
            cx: &mut command_context::Context,
        ) -> AnyResponse {
            let args = match service_target::args_box::<$command>(args) {
                Ok(args) => args,
                Err(error) => return AnyResponse::failed(error),
            };
            let mut target = text.target();

            AnyResponse::from_response(Target::<$command>::invoke(&mut target, args, cx))
        }
    };
}

text_target!(select_all_state, select_all_invoke, document::SelectAll);
text_target!(copy_state, copy_invoke, document::Copy);
text_target!(cut_state, cut_invoke, document::Cut);
text_target!(delete_state, delete_invoke, document::Delete);
text_target!(paste_state, paste_invoke, document::Paste);
text_target!(undo_state, undo_invoke, timeline::Undo);
text_target!(redo_state, redo_invoke, timeline::Redo);

fn command_name(command_type: TypeId) -> &'static str {
    if command_type == TypeId::of::<document::SelectAll>() {
        return <document::SelectAll as framework_command::Command>::NAME;
    }
    if command_type == TypeId::of::<document::Copy>() {
        return <document::Copy as framework_command::Command>::NAME;
    }
    if command_type == TypeId::of::<document::Cut>() {
        return <document::Cut as framework_command::Command>::NAME;
    }
    if command_type == TypeId::of::<document::Delete>() {
        return <document::Delete as framework_command::Command>::NAME;
    }
    if command_type == TypeId::of::<document::Paste>() {
        return <document::Paste as framework_command::Command>::NAME;
    }
    if command_type == TypeId::of::<timeline::Undo>() {
        return <timeline::Undo as framework_command::Command>::NAME;
    }
    if command_type == TypeId::of::<timeline::Redo>() {
        return <timeline::Redo as framework_command::Command>::NAME;
    }

    "unknown"
}
