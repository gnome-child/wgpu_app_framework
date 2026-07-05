use std::path::PathBuf;

use super::super::Runtime;
use crate::scratch::{
    command, context as command_context, document, error::Error, input, session, state, window,
};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::scratch::runtime::input) fn handle_file_path_selected(
        &mut self,
        window: window::Id,
        path: Option<PathBuf>,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(dialog) = self.session.take_file_dialog(window) else {
            return Ok(input::Outcome::ignored());
        };

        match (dialog, path) {
            (session::FileDialog::Open, Some(path)) => self.invoke_dialog_command(
                window,
                command::Trigger::<document::OpenPath>::command(path),
            ),
            (session::FileDialog::Open, None) => self.invoke_dialog_command(
                window,
                command::Trigger::<document::OpenCanceled>::command(()),
            ),
            (session::FileDialog::SaveAs, Some(path)) => self.invoke_dialog_command(
                window,
                command::Trigger::<document::SaveToPath>::command(path),
            ),
            (session::FileDialog::SaveAs, None) => self.invoke_dialog_command(
                window,
                command::Trigger::<document::SaveCanceled>::command(()),
            ),
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
}
