use super::{Error, Result};

pub(super) struct System {
    clipboard: Option<arboard::Clipboard>,
    #[cfg(test)]
    forced_unavailable: bool,
}

impl System {
    pub(super) fn new() -> Self {
        Self {
            clipboard: None,
            #[cfg(test)]
            forced_unavailable: false,
        }
    }

    #[cfg(test)]
    pub(super) fn unavailable() -> Self {
        Self {
            clipboard: None,
            forced_unavailable: true,
        }
    }

    fn clipboard(&mut self) -> Result<&mut arboard::Clipboard> {
        #[cfg(test)]
        if self.forced_unavailable {
            return Err(Error::Unavailable);
        }

        if self.clipboard.is_none() {
            match arboard::Clipboard::new() {
                Ok(clipboard) => {
                    log::debug!("connected to system clipboard");
                    self.clipboard = Some(clipboard);
                }
                Err(error) => {
                    log::warn!("system clipboard unavailable: {error}");
                    return Err(Error::Unavailable);
                }
            }
        }

        self.clipboard.as_mut().ok_or(Error::Unavailable)
    }

    pub(super) fn read_text(&mut self) -> Result<Option<String>> {
        match self.clipboard()?.get_text() {
            Ok(text) => Ok((!text.is_empty()).then_some(text)),
            Err(arboard::Error::ContentNotAvailable) => Ok(None),
            Err(error) => {
                log::warn!("failed to read text from system clipboard: {error}");
                Err(Error::Unavailable)
            }
        }
    }

    pub(super) fn write_text(&mut self, text: &str) -> Result<()> {
        self.clipboard()?
            .set_text(text.to_owned())
            .map_err(|error| {
                log::warn!("failed to write text to system clipboard: {error}");
                Error::Unavailable
            })
    }
}
