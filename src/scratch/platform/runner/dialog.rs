use std::path::PathBuf;

use winit::event_loop::ActiveEventLoop;

use super::super::{Error, Native, NativeContext, NativeError};
use super::Runner;
use crate::scratch::{host, session, state::State};

impl<M: State, E: Send + 'static> Runner<M, E, Native> {
    pub(in crate::scratch::platform::runner) fn handle_native_requests(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), Error<NativeError>> {
        let requests = self.platform.backend_mut().take_requests();

        for request in requests {
            let path = native_file_dialog(request.kind());
            let event = file_dialog_selected(request, path);
            let mut context = NativeContext::new(event_loop);
            self.platform.handle_event_with(&mut context, event)?;
            self.sync_native_event_state();
        }

        Ok(())
    }
}

pub(in crate::scratch) fn file_dialog_selected(
    request: session::Request,
    path: Option<PathBuf>,
) -> host::Event {
    match request.kind() {
        session::RequestKind::FileDialog(_) => host::Event::FilePathSelected {
            window: request.window(),
            path,
        },
    }
}

fn native_file_dialog(kind: session::RequestKind) -> Option<PathBuf> {
    match kind {
        session::RequestKind::FileDialog(session::FileDialog::Open) => {
            rfd::FileDialog::new().pick_file()
        }
        session::RequestKind::FileDialog(session::FileDialog::SaveAs) => {
            rfd::FileDialog::new().save_file()
        }
    }
}
