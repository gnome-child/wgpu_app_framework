use std::{cell::RefCell, fmt, rc::Rc};

use crate::text;

#[derive(Default, Clone)]
pub struct Clipboard {
    inner: Rc<RefCell<Inner>>,
}

pub trait Payload: Sized + 'static {
    fn write(&self, out: &mut Representations);
    fn read(source: &Representations) -> Option<Self>;

    fn contains(source: &Representations) -> bool {
        Self::read(source).is_some()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Representations {
    text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text(String);

#[derive(Default)]
struct Inner {
    representations: Representations,
    system: Option<System>,
}

struct System {
    clipboard: Option<arboard::Clipboard>,
}

impl Clipboard {
    pub fn system() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner {
                representations: Representations::default(),
                system: Some(System::new()),
            })),
        }
    }

    pub fn put<T: Payload>(&self, payload: &T) {
        let mut inner = self.inner.borrow_mut();
        payload.write(&mut inner.representations);
        inner.write_system_text();
    }

    pub fn get<T: Payload>(&self) -> Option<T> {
        let mut inner = self.inner.borrow_mut();
        inner.read_system_text();
        T::read(&inner.representations)
    }

    pub fn contains<T: Payload>(&self) -> bool {
        let mut inner = self.inner.borrow_mut();
        inner.read_system_text();
        T::contains(&inner.representations)
    }

    pub fn text(&self) -> Option<String> {
        self.get::<Text>().map(Text::into_string)
    }

    pub fn has_text(&self) -> bool {
        self.contains::<Text>()
    }

    pub fn is_system_enabled(&self) -> bool {
        self.inner.borrow().system.is_some()
    }
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl Payload for Text {
    fn write(&self, out: &mut Representations) {
        out.text = Some(self.0.clone());
    }

    fn read(source: &Representations) -> Option<Self> {
        source
            .text
            .as_ref()
            .filter(|text| !text.is_empty())
            .cloned()
            .map(Self)
    }
}

impl text::edit::Clipboard for Clipboard {
    fn read_text(&mut self) -> text::edit::ClipboardResult<Option<String>> {
        Ok(self.text())
    }

    fn write_text(&mut self, text: &str) -> text::edit::ClipboardResult<()> {
        self.put(&Text::new(text));
        Ok(())
    }
}

impl From<String> for Text {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for Text {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl PartialEq for Clipboard {
    fn eq(&self, other: &Self) -> bool {
        self.inner.borrow().representations == other.inner.borrow().representations
    }
}

impl Eq for Clipboard {}

impl Inner {
    fn read_system_text(&mut self) {
        let Some(system) = self.system.as_mut() else {
            return;
        };

        match system.read_text() {
            Some(Some(text)) => self.representations.text = Some(text),
            Some(None) => self.representations.text = None,
            None => {}
        }
    }

    fn write_system_text(&mut self) {
        let Some(system) = self.system.as_mut() else {
            return;
        };
        let Some(text) = self.representations.text.as_deref() else {
            return;
        };

        system.write_text(text);
    }
}

impl System {
    fn new() -> Self {
        Self { clipboard: None }
    }

    fn clipboard(&mut self) -> Option<&mut arboard::Clipboard> {
        if self.clipboard.is_none() {
            self.clipboard = arboard::Clipboard::new().ok();
        }

        self.clipboard.as_mut()
    }

    fn read_text(&mut self) -> Option<Option<String>> {
        match self.clipboard()?.get_text() {
            Ok(text) => Some((!text.is_empty()).then_some(text)),
            Err(arboard::Error::ContentNotAvailable) => Some(None),
            Err(_) => None,
        }
    }

    fn write_text(&mut self, text: &str) {
        if let Some(clipboard) = self.clipboard() {
            let _ = clipboard.set_text(text.to_owned());
        }
    }
}

impl fmt::Debug for Clipboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.inner.borrow();
        f.debug_struct("Clipboard")
            .field("representations", &inner.representations)
            .field("system_enabled", &inner.system.is_some())
            .finish()
    }
}
