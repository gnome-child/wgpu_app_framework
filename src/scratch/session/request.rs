use super::super::window as app_window;
use super::Session;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileDialog {
    Open,
    SaveAs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Request {
    window: app_window::Id,
    kind: RequestKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequestKind {
    FileDialog(FileDialog),
}

impl Request {
    pub fn file_dialog(window: app_window::Id, dialog: FileDialog) -> Self {
        Self {
            window,
            kind: RequestKind::FileDialog(dialog),
        }
    }

    pub fn window(self) -> app_window::Id {
        self.window
    }

    pub fn kind(self) -> RequestKind {
        self.kind
    }
}

impl Session {
    pub fn request_file_dialog(&mut self, id: app_window::Id, dialog: FileDialog) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = window.file_dialog != Some(dialog);
        window.file_dialog = Some(dialog);
        changed
    }

    pub fn take_file_dialog(&mut self, id: app_window::Id) -> Option<FileDialog> {
        self.window_mut(id)?.file_dialog.take()
    }

    pub fn file_dialog(&self, id: app_window::Id) -> Option<FileDialog> {
        self.window(id).and_then(|window| window.file_dialog)
    }

    pub fn requests(&self) -> Vec<Request> {
        self.windows
            .iter()
            .filter_map(|window| {
                window
                    .file_dialog
                    .map(|dialog| Request::file_dialog(window.id, dialog))
            })
            .collect()
    }
}
