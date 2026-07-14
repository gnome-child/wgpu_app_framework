use super::{Composition, Tree, tree::Changes};
use crate::{notification, view, window};

#[derive(Default)]
pub(crate) struct Store {
    compositions: Vec<Composition>,
    next_node_id: u64,
}

impl Store {
    #[cfg(test)]
    pub(crate) fn install(&mut self, window: window::Id, view: view::View) -> &Composition {
        let (tree, changes) = self.prepare(window, &view);
        self.install_prepared(window, view, tree, changes)
    }

    pub(crate) fn prepare(&mut self, window: window::Id, view: &view::View) -> (Tree, Changes) {
        if self.next_node_id == 0 {
            self.next_node_id = 1;
        }

        if let Some(index) = self.index_of(window) {
            return self.compositions[index]
                .tree()
                .reconcile(view, &mut self.next_node_id);
        }

        Tree::new(view, &mut self.next_node_id)
    }

    pub(crate) fn install_prepared(
        &mut self,
        window: window::Id,
        view: view::View,
        tree: Tree,
        changes: Changes,
    ) -> &Composition {
        if let Some(index) = self.index_of(window) {
            self.compositions[index].install_prepared(view, tree, changes);
            return &self.compositions[index];
        }

        self.compositions
            .push(Composition::new_prepared(window, view, tree, changes));
        self.compositions
            .last()
            .expect("a composition was just installed")
    }

    pub(crate) fn get(&self, window: window::Id) -> Option<&Composition> {
        self.compositions
            .iter()
            .find(|composition| composition.window() == window)
    }

    pub(crate) fn get_mut(&mut self, window: window::Id) -> Option<&mut Composition> {
        self.compositions
            .iter_mut()
            .find(|composition| composition.window() == window)
    }

    pub(crate) fn remove_window(&mut self, window: window::Id) -> bool {
        let Some(index) = self.index_of(window) else {
            return false;
        };

        self.compositions.remove(index);
        true
    }

    pub(crate) fn clear(&mut self) {
        self.compositions.clear();
    }

    fn index_of(&self, window: window::Id) -> Option<usize> {
        self.compositions
            .iter()
            .position(|composition| composition.window() == window)
    }
}

impl notification::Listener<window::Departed> for Store {
    fn notify(&mut self, window: &window::Id) -> notification::Reaction {
        self.remove_window(*window);
        notification::Reaction::ignored()
    }
}
