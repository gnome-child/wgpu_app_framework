use std::{
    any::{Any, TypeId},
    result,
};

use crate::scratch::{
    command as framework_command, context as command_context, document, error::Error,
    response::AnyResponse, target::Target, timeline,
};

use super::target::FocusedTextBox;

#[derive(Clone, Copy)]
pub(super) enum Command {
    SelectAll,
    Copy,
    Cut,
    Delete,
    Paste,
    Undo,
    Redo,
}

impl Command {
    pub(super) fn from_args(
        command_type: TypeId,
        args: &dyn Any,
    ) -> result::Result<Option<Self>, Error> {
        let Some(command) = Self::from_type(command_type) else {
            return Ok(None);
        };

        match command {
            Self::SelectAll => {
                text_box_args::<document::SelectAll>(args)?;
            }
            Self::Copy => {
                text_box_args::<document::Copy>(args)?;
            }
            Self::Cut => {
                text_box_args::<document::Cut>(args)?;
            }
            Self::Delete => {
                text_box_args::<document::Delete>(args)?;
            }
            Self::Paste => {
                text_box_args::<document::Paste>(args)?;
            }
            Self::Undo => {
                text_box_args::<timeline::Undo>(args)?;
            }
            Self::Redo => {
                text_box_args::<timeline::Redo>(args)?;
            }
        }

        Ok(Some(command))
    }

    pub(super) fn from_box(
        command_type: TypeId,
        args: Box<dyn Any + Send>,
    ) -> result::Result<Option<Self>, Error> {
        let Some(command) = Self::from_type(command_type) else {
            return Ok(None);
        };

        match command {
            Self::SelectAll => {
                text_box_args_box::<document::SelectAll>(args)?;
            }
            Self::Copy => {
                text_box_args_box::<document::Copy>(args)?;
            }
            Self::Cut => {
                text_box_args_box::<document::Cut>(args)?;
            }
            Self::Delete => {
                text_box_args_box::<document::Delete>(args)?;
            }
            Self::Paste => {
                text_box_args_box::<document::Paste>(args)?;
            }
            Self::Undo => {
                text_box_args_box::<timeline::Undo>(args)?;
            }
            Self::Redo => {
                text_box_args_box::<timeline::Redo>(args)?;
            }
        }

        Ok(Some(command))
    }

    pub(super) fn from_type(command_type: TypeId) -> Option<Self> {
        if command_type == TypeId::of::<document::SelectAll>() {
            return Some(Self::SelectAll);
        }

        if command_type == TypeId::of::<document::Copy>() {
            return Some(Self::Copy);
        }

        if command_type == TypeId::of::<document::Cut>() {
            return Some(Self::Cut);
        }

        if command_type == TypeId::of::<document::Delete>() {
            return Some(Self::Delete);
        }

        if command_type == TypeId::of::<document::Paste>() {
            return Some(Self::Paste);
        }

        if command_type == TypeId::of::<timeline::Undo>() {
            return Some(Self::Undo);
        }

        if command_type == TypeId::of::<timeline::Redo>() {
            return Some(Self::Redo);
        }

        None
    }

    pub(super) fn invoke(
        self,
        target: &mut FocusedTextBox<'_>,
        cx: &mut command_context::Context,
    ) -> AnyResponse {
        match self {
            Self::SelectAll => {
                let response = Target::<document::SelectAll>::invoke(target, (), cx);
                AnyResponse::from_response(response)
            }
            Self::Copy => {
                let response = Target::<document::Copy>::invoke(target, (), cx);
                AnyResponse::from_response(response)
            }
            Self::Cut => {
                let response = Target::<document::Cut>::invoke(target, (), cx);
                AnyResponse::from_response(response)
            }
            Self::Delete => {
                let response = Target::<document::Delete>::invoke(target, (), cx);
                AnyResponse::from_response(response)
            }
            Self::Paste => {
                let response = Target::<document::Paste>::invoke(target, (), cx);
                AnyResponse::from_response(response)
            }
            Self::Undo => {
                let response = Target::<timeline::Undo>::invoke(target, (), cx);
                AnyResponse::from_response(response)
            }
            Self::Redo => {
                let response = Target::<timeline::Redo>::invoke(target, (), cx);
                AnyResponse::from_response(response)
            }
        }
    }
}

fn text_box_args<C: framework_command::Command>(args: &dyn Any) -> result::Result<&C::Args, Error> {
    args.downcast_ref::<C::Args>()
        .ok_or(Error::ArgsMismatch { command: C::NAME })
}

fn text_box_args_box<C: framework_command::Command>(
    args: Box<dyn Any + Send>,
) -> result::Result<C::Args, Error> {
    args.downcast::<C::Args>()
        .map(|args| *args)
        .map_err(|_| Error::ArgsMismatch { command: C::NAME })
}
