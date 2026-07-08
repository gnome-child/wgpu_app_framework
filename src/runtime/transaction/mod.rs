mod activation;
mod command;
pub(in crate::runtime) mod gesture;
pub(in crate::runtime) mod history;
mod notification;
mod observer;
mod outcome;

use std::any::TypeId;

use crate::{command as command_api, context, session, window};

pub(in crate::runtime) struct AnyInvocation {
    pub(in crate::runtime) focus: Option<session::Focus>,
    pub(in crate::runtime) window: Option<window::Id>,
    pub(in crate::runtime) command_type: TypeId,
    pub(in crate::runtime) command_name: &'static str,
    pub(in crate::runtime) history_group: Option<command_api::HistoryGroup>,
    pub(in crate::runtime) source: context::Source,
}
