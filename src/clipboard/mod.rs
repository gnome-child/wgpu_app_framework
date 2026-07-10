use std::{cell::RefCell, fmt, rc::Rc};

mod error;
mod payload;
mod representations;
mod system;
mod text;

pub use error::{Error, Result};
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

    pub fn put<T: Payload>(&self, payload: &T) -> Result<()> {
        let mut inner = self.inner.borrow_mut();
        let mut representations = inner.representations.clone();
        payload.write(&mut representations);
        inner.write_system_text(&representations)?;
        inner.representations = representations;
        Ok(())
    }

    pub fn get<T: Payload>(&self) -> Result<Option<T>> {
        let mut inner = self.inner.borrow_mut();
        inner.read_system_text()?;
        Ok(T::read(&inner.representations))
    }

    pub fn contains<T: Payload>(&self) -> Result<bool> {
        let mut inner = self.inner.borrow_mut();
        inner.read_system_text()?;
        Ok(T::contains(&inner.representations))
    }

    pub fn text(&self) -> Result<Option<String>> {
        Ok(self.get::<Text>()?.map(Text::into_string))
    }

    pub fn has_text(&self) -> Result<bool> {
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
    fn read_system_text(&mut self) -> Result<()> {
        let Some(system) = self.system.as_mut() else {
            return Ok(());
        };

        match system.read_text()? {
            Some(text) => self.representations.set_text(text),
            None => self.representations.clear_text(),
        }
        Ok(())
    }

    fn write_system_text(&mut self, representations: &Representations) -> Result<()> {
        let Some(system) = self.system.as_mut() else {
            return Ok(());
        };
        let Some(text) = representations.text() else {
            return Ok(());
        };

        system.write_text(text)
    }
}

#[cfg(test)]
impl Clipboard {
    pub(crate) fn unavailable_system() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner {
                representations: Representations::default(),
                system: Some(System::unavailable()),
            })),
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
