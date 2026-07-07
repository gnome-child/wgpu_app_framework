pub(super) struct System {
    clipboard: Option<arboard::Clipboard>,
}

impl System {
    pub(super) fn new() -> Self {
        Self { clipboard: None }
    }

    fn clipboard(&mut self) -> Option<&mut arboard::Clipboard> {
        if self.clipboard.is_none() {
            self.clipboard = arboard::Clipboard::new().ok();
        }

        self.clipboard.as_mut()
    }

    pub(super) fn read_text(&mut self) -> Option<Option<String>> {
        match self.clipboard()?.get_text() {
            Ok(text) => Some((!text.is_empty()).then_some(text)),
            Err(arboard::Error::ContentNotAvailable) => Some(None),
            Err(_) => None,
        }
    }

    pub(super) fn write_text(&mut self, text: &str) {
        if let Some(clipboard) = self.clipboard() {
            let _ = clipboard.set_text(text.to_owned());
        }
    }
}
