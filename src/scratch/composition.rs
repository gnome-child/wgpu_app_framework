use super::{session, view, window};

pub struct Composition {
    window: window::Id,
    view: view::View,
}

#[derive(Default)]
pub(super) struct Store {
    compositions: Vec<Composition>,
}

impl Composition {
    fn new(window: window::Id, view: view::View) -> Self {
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

impl Store {
    pub(super) fn install(&mut self, window: window::Id, view: view::View) -> &Composition {
        if let Some(index) = self.index_of(window) {
            self.compositions[index] = Composition::new(window, view);
            return &self.compositions[index];
        }

        self.compositions.push(Composition::new(window, view));
        self.compositions
            .last()
            .expect("a composition was just installed")
    }

    pub(super) fn get(&self, window: window::Id) -> Option<&Composition> {
        self.compositions
            .iter()
            .find(|composition| composition.window == window)
    }

    pub(super) fn remove_window(&mut self, window: window::Id) -> bool {
        let Some(index) = self.index_of(window) else {
            return false;
        };

        self.compositions.remove(index);
        true
    }

    pub(super) fn clear(&mut self) {
        self.compositions.clear();
    }

    fn index_of(&self, window: window::Id) -> Option<usize> {
        self.compositions
            .iter()
            .position(|composition| composition.window == window)
    }
}
