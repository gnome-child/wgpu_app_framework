use super::Composition;
use crate::scratch::{view, window};

#[derive(Default)]
pub(in crate::scratch) struct Store {
    compositions: Vec<Composition>,
}

impl Store {
    pub(in crate::scratch) fn install(
        &mut self,
        window: window::Id,
        view: view::View,
    ) -> &Composition {
        if let Some(index) = self.index_of(window) {
            self.compositions[index] = Composition::new(window, view);
            return &self.compositions[index];
        }

        self.compositions.push(Composition::new(window, view));
        self.compositions
            .last()
            .expect("a composition was just installed")
    }

    pub(in crate::scratch) fn get(&self, window: window::Id) -> Option<&Composition> {
        self.compositions
            .iter()
            .find(|composition| composition.window() == window)
    }

    pub(in crate::scratch) fn remove_window(&mut self, window: window::Id) -> bool {
        let Some(index) = self.index_of(window) else {
            return false;
        };

        self.compositions.remove(index);
        true
    }

    pub(in crate::scratch) fn clear(&mut self) {
        self.compositions.clear();
    }

    fn index_of(&self, window: window::Id) -> Option<usize> {
        self.compositions
            .iter()
            .position(|composition| composition.window() == window)
    }
}
