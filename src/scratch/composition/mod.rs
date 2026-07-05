mod store;

pub(in crate::scratch) use store::Store;

use super::{session, view, window};

pub struct Composition {
    window: window::Id,
    view: view::View,
}

impl Composition {
    pub(super) fn new(window: window::Id, view: view::View) -> Self {
        Self { window, view }
    }

    pub fn window(&self) -> window::Id {
        self.window
    }

    pub fn view(&self) -> &view::View {
        &self.view
    }

    pub fn contains_focus(&self, focus: session::Focus) -> bool {
        self.view.contains_focus(focus)
    }
}
