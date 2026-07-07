use super::super::window;
use super::View;

#[derive(Clone)]
pub struct Presentation {
    window: window::Id,
    view: View,
}

impl Presentation {
    pub(crate) fn new(window: window::Id, view: View) -> Self {
        Self { window, view }
    }

    pub fn window(&self) -> window::Id {
        self.window
    }

    pub fn view(&self) -> &View {
        &self.view
    }

    pub fn into_view(self) -> View {
        self.view
    }
}
