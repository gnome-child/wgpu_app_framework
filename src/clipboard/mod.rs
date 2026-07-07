use std::{cell::RefCell, fmt, rc::Rc};

mod payload;
mod representations;
mod system;
mod text;

pub use payload::Payload;
pub use representations::Representations;
pub use text::Text;

use system::System;

#[derive(Default, Clone)]
pub struct Clipboard {
    inner: Rc<RefCell<Inner>>,
}

#[derive(Default)]
struct Inner {
    representations: Representations,
    system: Option<System>,
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
            Some(Some(text)) => self.representations.set_text(text),
            Some(None) => self.representations.clear_text(),
            None => {}
        }
    }

    fn write_system_text(&mut self) {
        let Some(system) = self.system.as_mut() else {
            return;
        };
        let Some(text) = self.representations.text() else {
            return;
        };

        system.write_text(text);
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
