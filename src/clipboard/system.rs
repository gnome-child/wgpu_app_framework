pub(super) struct System {
    clipboard: Option<arboard::Clipboard>,
}

impl System {
    pub(super) fn new() -> Self {
        Self { clipboard: None }
    }

    fn clipboard(&mut self) -> Option<&mut arboard::Clipboard> {
        if self.clipboard.is_none() {
            match arboard::Clipboard::new() {
                Ok(clipboard) => {
                    log::debug!("connected to system clipboard");
                    self.clipboard = Some(clipboard);
                }
                Err(error) => {
                    log::warn!("system clipboard unavailable: {error}");
                }
            }
        }

        self.clipboard.as_mut()
    }

    pub(super) fn read_text(&mut self) -> Option<Option<String>> {
        match self.clipboard()?.get_text() {
            Ok(text) => Some((!text.is_empty()).then_some(text)),
            Err(arboard::Error::ContentNotAvailable) => Some(None),
            Err(error) => {
                log::warn!("failed to read text from system clipboard: {error}");
                None
            }
        }
    }

    pub(super) fn write_text(&mut self, text: &str) {
        let Some(clipboard) = self.clipboard() else {
            log::debug!("skipping clipboard write because system clipboard is unavailable");
            return;
        };

        if let Err(error) = clipboard.set_text(text.to_owned()) {
            log::warn!("failed to write text to system clipboard: {error}");
        }
    }
}
