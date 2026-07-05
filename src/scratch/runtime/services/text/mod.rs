use std::{
    any::{Any, TypeId},
    result,
};

use crate::scratch::{
    clipboard, command as framework_command, composition, context as command_context, draft,
    error::Error, interaction, session, window,
};

mod command;
mod target;

use command::Command as EditCommand;
use target::FocusedTextBox;

pub(super) fn state(
    session: &session::Session,
    composition: &composition::Store,
    window: Option<window::Id>,
    command_type: TypeId,
    args: &dyn Any,
    cx: &command_context::Context,
) -> result::Result<Option<framework_command::State>, Error> {
    Ok(EditCommand::from_args(command_type, args)?
        .and_then(|command| state_for_command(command, session, composition, window, cx)))
}

pub(in crate::scratch::runtime) fn handles(
    session: &session::Session,
    composition: &composition::Store,
    window: Option<window::Id>,
    command_type: TypeId,
) -> bool {
    EditCommand::from_type(command_type).is_some()
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
    let Some((window, focus, _)) = base_text_for(session, composition, window) else {
        return None;
    };

    if !session
        .focused(window)
        .is_some_and(|current| current.same_target(&focus))
    {
        session.focus(window, focus);
    }

    let command = match EditCommand::from_box(command_type, args) {
        Ok(Some(command)) => command,
        Ok(None) => return None,
        Err(error) => return Some(crate::scratch::response::AnyResponse::failed(error)),
    };

    let mut target = FocusedTextBox::new(session, composition, window, focus);

    Some(command.invoke(&mut target, cx))
}

fn state_for_command(
    command: EditCommand,
    session: &session::Session,
    composition: &composition::Store,
    window: Option<window::Id>,
    cx: &command_context::Context,
) -> Option<framework_command::State> {
    match command {
        EditCommand::SelectAll | EditCommand::Delete => draft_for(session, composition, window)
            .map(|draft| {
                if draft.text().is_empty() {
                    framework_command::State::disabled()
                } else {
                    framework_command::State::enabled()
                }
            }),
        EditCommand::Copy | EditCommand::Cut => {
            draft_for(session, composition, window).map(selection_state)
        }
        EditCommand::Paste => base_text_for(session, composition, window).map(|_| {
            if cx
                .clipboard()
                .is_some_and(|clipboard| clipboard.contains::<clipboard::Text>())
            {
                framework_command::State::enabled()
            } else {
                framework_command::State::disabled()
            }
        }),
        EditCommand::Undo => draft_for(session, composition, window).map(|draft| {
            if draft.can_undo() {
                framework_command::State::enabled()
            } else {
                framework_command::State::disabled()
            }
        }),
        EditCommand::Redo => draft_for(session, composition, window).map(|draft| {
            if draft.can_redo() {
                framework_command::State::enabled()
            } else {
                framework_command::State::disabled()
            }
        }),
    }
}

fn draft_for(
    session: &session::Session,
    composition: &composition::Store,
    window: Option<window::Id>,
) -> Option<draft::State> {
    let (window, focus, base) = base_text_for(session, composition, window)?;
    let target = interaction::Target::text_area(focus);

    Some(
        session
            .interaction(window)
            .and_then(|interaction| interaction.text_input().draft_for(&target).cloned())
            .unwrap_or_else(|| draft::State::new(base)),
    )
}

fn base_text_for(
    session: &session::Session,
    composition: &composition::Store,
    window: Option<window::Id>,
) -> Option<(window::Id, session::Focus, String)> {
    let window = window?;
    let focus = session.command_focus(window)?;
    let base = composition
        .get(window)?
        .view()
        .text_box_text(focus)?
        .to_owned();

    Some((window, focus, base))
}

fn selection_state(draft: draft::State) -> framework_command::State {
    if draft.selected_text().is_some() {
        framework_command::State::enabled()
    } else {
        framework_command::State::disabled()
    }
}
