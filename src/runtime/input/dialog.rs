use std::path::PathBuf;

use super::super::Runtime;
use crate::{
    command::{self, Error},
    context as command_context, document, input, notification, session, state, window,
};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::runtime::input) fn handle_file_path_selected(
        &mut self,
        window: window::Id,
        path: Option<PathBuf>,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(dialog) = self.session.take_file_dialog(window) else {
            log::warn!(
                "received file dialog result for window {:?} with no pending dialog",
                window
            );
            return Ok(input::Outcome::ignored());
        };

        match (dialog, path) {
            (session::FileDialog::Open, Some(path)) => {
                log::debug!(
                    "open dialog selected path for window {:?}: {:?}",
                    window,
                    path
                );
                self.invoke_dialog_command(
                    window,
                    command::Trigger::<document::OpenPath>::command(path),
                )
            }
            (session::FileDialog::Open, None) => {
                log::debug!("open dialog canceled for window {:?}", window);
                self.notify_dialog::<document::OpenDialogCanceled>(window, ())
            }
            (session::FileDialog::SaveAs, Some(path)) => {
                log::debug!(
                    "save-as dialog selected path for window {:?}: {:?}",
                    window,
                    path
                );
                self.invoke_dialog_command(
                    window,
                    command::Trigger::<document::SaveToPath>::command(path),
                )
            }
            (session::FileDialog::SaveAs, None) => {
                log::debug!("save-as dialog canceled for window {:?}", window);
                self.notify_dialog::<document::SaveDialogCanceled>(window, ())
            }
        }
    }

    fn invoke_dialog_command<C: command::Command>(
        &mut self,
        window: window::Id,
        trigger: command::Trigger<C>,
    ) -> std::result::Result<input::Outcome, Error> {
        let response =
            self.invoke_focused_with_source(window, trigger, command_context::Source::Input);
        let changed = response.changed_state();
        let effect = response.effect.clone();

        response
            .output
            .map(|_| input::Outcome::handled(changed, effect))
    }

    fn notify_dialog<N: notification::Notification>(
        &mut self,
        window: window::Id,
        payload: N::Payload,
    ) -> std::result::Result<input::Outcome, Error> {
        let reaction = self.notify_focused::<N>(window, payload, command_context::Source::Input);

        Ok(self.window_outcome(window, reaction.changed_state(), reaction.effect().clone()))
    }
}
