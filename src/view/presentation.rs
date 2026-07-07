use super::super::window;
use super::View;

#[derive(Clone)]
pub(crate) struct Presentation {
    window: window::Id,
    view: View,
}

impl Presentation {
    pub(crate) fn new(window: window::Id, view: View) -> Self {
        Self { window, view }
    }

    pub(crate) fn window(&self) -> window::Id {
        self.window
    }

    pub(crate) fn view(&self) -> &View {
        &self.view
    }
}
