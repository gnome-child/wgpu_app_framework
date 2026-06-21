use crate::text;

pub struct SystemClipboard {
    clipboard: Option<arboard::Clipboard>,
}

impl SystemClipboard {
    pub fn new() -> Self {
        Self {
            clipboard: arboard::Clipboard::new().ok(),
        }
    }

    fn clipboard(&mut self) -> text::edit::ClipboardResult<&mut arboard::Clipboard> {
        if self.clipboard.is_none() {
            self.clipboard = arboard::Clipboard::new().ok();
        }

        self.clipboard
            .as_mut()
            .ok_or(text::edit::ClipboardError::Unavailable)
    }
}

impl Default for SystemClipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl text::edit::Clipboard for SystemClipboard {
    fn read_text(&mut self) -> text::edit::ClipboardResult<Option<String>> {
        match self.clipboard()?.get_text() {
            Ok(text) => Ok((!text.is_empty()).then_some(text)),
            Err(arboard::Error::ContentNotAvailable) => Ok(None),
            Err(_) => Err(text::edit::ClipboardError::Unavailable),
        }
    }

    fn write_text(&mut self, text: &str) -> text::edit::ClipboardResult<()> {
        self.clipboard()?
            .set_text(text.to_owned())
            .map_err(|_| text::edit::ClipboardError::Unavailable)
    }
}
