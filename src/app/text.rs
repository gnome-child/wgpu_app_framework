use std::collections::HashMap;
use std::collections::hash_map::{Entry, ValuesMut};

use crate::{text, ui};

#[derive(Debug, Default)]
pub(crate) struct Driver {
    states: HashMap<ui::Path, text::view::TextViewState>,
}

impl Driver {
    pub(crate) fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    pub(crate) fn clear(&mut self) {
        self.states.clear();
    }

    pub(crate) fn contains(&self, path: &ui::Path) -> bool {
        self.states.contains_key(path)
    }

    pub(crate) fn states(&self) -> &HashMap<ui::Path, text::view::TextViewState> {
        &self.states
    }

    pub(crate) fn states_mut(&mut self) -> &mut HashMap<ui::Path, text::view::TextViewState> {
        &mut self.states
    }

    pub(crate) fn get(&self, path: &ui::Path) -> Option<&text::view::TextViewState> {
        self.states.get(path)
    }

    pub(crate) fn get_mut(&mut self, path: &ui::Path) -> Option<&mut text::view::TextViewState> {
        self.states.get_mut(path)
    }

    pub(crate) fn get_cloned_or_default(&self, path: &ui::Path) -> text::view::TextViewState {
        self.states.get(path).cloned().unwrap_or_default()
    }

    pub(crate) fn insert(
        &mut self,
        path: ui::Path,
        state: text::view::TextViewState,
    ) -> Option<text::view::TextViewState> {
        self.states.insert(path, state)
    }

    pub(crate) fn entry(
        &mut self,
        path: ui::Path,
    ) -> Entry<'_, ui::Path, text::view::TextViewState> {
        self.states.entry(path)
    }

    pub(crate) fn values_mut(&mut self) -> ValuesMut<'_, ui::Path, text::view::TextViewState> {
        self.states.values_mut()
    }
}

impl From<HashMap<ui::Path, text::view::TextViewState>> for Driver {
    fn from(states: HashMap<ui::Path, text::view::TextViewState>) -> Self {
        Self { states }
    }
}
