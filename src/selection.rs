use std::collections::BTreeSet;

use crate::list::{Key, Model};

/// Window-local keyed selection state for a provided container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    membership: Membership,
    selected_count: usize,
    anchor: Option<Endpoint>,
    active: Option<Endpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Membership {
    Explicit(BTreeSet<Key>),
    AllExcept(BTreeSet<Key>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Endpoint {
    key: Key,
    index: usize,
}

impl Endpoint {
    fn new(key: Key, index: usize) -> Self {
        Self { key, index }
    }

    fn resolve(self, provider: &dyn Model) -> Option<Self> {
        provider
            .index_of(self.key)
            .map(|index| Self::new(self.key, index))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Move {
    Previous,
    Next,
    First,
    Last,
    PagePrevious(usize),
    PageNext(usize),
}

impl Selection {
    pub fn new() -> Self {
        Self {
            membership: Membership::Explicit(BTreeSet::new()),
            selected_count: 0,
            anchor: None,
            active: None,
        }
    }

    pub fn len(&self) -> usize {
        self.selected_count
    }

    pub fn is_empty(&self) -> bool {
        self.selected_count == 0
    }

    pub fn contains(&self, key: Key) -> bool {
        match &self.membership {
            Membership::Explicit(keys) => keys.contains(&key),
            Membership::AllExcept(excluded) => !excluded.contains(&key),
        }
    }

    pub fn is_all(&self) -> bool {
        matches!(&self.membership, Membership::AllExcept(excluded) if excluded.is_empty())
    }

    pub fn anchor(&self) -> Option<Key> {
        self.anchor.map(|endpoint| endpoint.key)
    }

    pub fn active(&self) -> Option<Key> {
        self.active.map(|endpoint| endpoint.key)
    }

    pub(crate) fn clear(&mut self) -> bool {
        let changed = !self.is_empty() || self.anchor.is_some() || self.active.is_some();
        *self = Self::new();
        changed
    }

    pub(crate) fn click(
        &mut self,
        provider: &dyn Model,
        key: Key,
        index: usize,
        extend: bool,
        toggle: bool,
    ) -> bool {
        let before = self.clone();
        if extend {
            self.extend_to(provider, key, index);
        } else if toggle {
            self.toggle(provider.len(), key, index);
        } else {
            self.select_only(key, index);
        }
        *self != before
    }

    pub(crate) fn select_all(&mut self, provider: &dyn Model) -> bool {
        let before = self.clone();
        let len = provider.len();
        self.membership = Membership::AllExcept(BTreeSet::new());
        self.selected_count = len;
        if len == 0 {
            self.anchor = None;
            self.active = None;
        } else if self
            .active
            .and_then(|endpoint| endpoint.resolve(provider))
            .is_none()
        {
            let key = provider.key(0);
            let endpoint = Endpoint::new(key, 0);
            self.anchor = Some(endpoint);
            self.active = Some(endpoint);
        } else {
            self.active = self.active.and_then(|endpoint| endpoint.resolve(provider));
            if self
                .anchor
                .and_then(|endpoint| provider.index_of(endpoint.key))
                .is_none()
            {
                self.anchor = self.active;
            }
        }
        *self != before
    }

    pub(crate) fn move_active(
        &mut self,
        provider: &dyn Model,
        movement: Move,
        extend: bool,
    ) -> bool {
        let len = provider.len();
        if len == 0 {
            return self.clear();
        }

        let current = self
            .active
            .and_then(|endpoint| provider.index_of(endpoint.key))
            .or(self.active.map(|endpoint| endpoint.index))
            .unwrap_or(0)
            .min(len - 1);
        let index = match movement {
            Move::Previous => current.saturating_sub(1),
            Move::Next => current.saturating_add(1).min(len - 1),
            Move::First => 0,
            Move::Last => len - 1,
            Move::PagePrevious(page) => current.saturating_sub(page.max(1)),
            Move::PageNext(page) => current.saturating_add(page.max(1)).min(len - 1),
        };
        let key = provider.key(index);
        let before = self.clone();
        if extend {
            if self.anchor.is_none() {
                let anchor = provider.key(current);
                self.anchor = Some(Endpoint::new(anchor, current));
            }
            self.extend_to(provider, key, index);
        } else {
            self.select_only(key, index);
        }
        *self != before
    }

    pub(crate) fn reconcile(&mut self, provider: &dyn Model) -> bool {
        let before = self.clone();
        let len = provider.len();
        match &mut self.membership {
            Membership::Explicit(keys) => {
                keys.retain(|key| provider.index_of(*key).is_some());
                self.selected_count = keys.len();
            }
            Membership::AllExcept(excluded) => {
                excluded.retain(|key| provider.index_of(*key).is_some());
                self.selected_count = len.saturating_sub(excluded.len());
            }
        }

        let old_active_index = self.active.map_or(0, |endpoint| endpoint.index);
        self.active = self.active.and_then(|endpoint| endpoint.resolve(provider));
        if self.active.is_none() {
            self.active = self
                .nearest_selected(provider, old_active_index)
                .map(|(key, index)| Endpoint::new(key, index));
        }

        self.anchor = self.anchor.and_then(|endpoint| endpoint.resolve(provider));
        if self.anchor.is_none() {
            self.anchor = self.active;
        }
        *self != before
    }

    fn select_only(&mut self, key: Key, index: usize) {
        self.membership = Membership::Explicit(BTreeSet::from([key]));
        self.selected_count = 1;
        let endpoint = Endpoint::new(key, index);
        self.anchor = Some(endpoint);
        self.active = Some(endpoint);
    }

    fn toggle(&mut self, provider_len: usize, key: Key, index: usize) {
        match &mut self.membership {
            Membership::Explicit(keys) => {
                if !keys.insert(key) {
                    keys.remove(&key);
                }
                self.selected_count = keys.len();
            }
            Membership::AllExcept(excluded) => {
                if !excluded.insert(key) {
                    excluded.remove(&key);
                }
                self.selected_count = provider_len.saturating_sub(excluded.len());
            }
        }
        let endpoint = Endpoint::new(key, index);
        self.anchor = Some(endpoint);
        self.active = Some(endpoint);
    }

    fn extend_to(&mut self, provider: &dyn Model, key: Key, index: usize) {
        let anchor_index = self
            .anchor
            .and_then(|endpoint| provider.index_of(endpoint.key))
            .or(self.anchor.map(|endpoint| endpoint.index))
            .unwrap_or(index)
            .min(provider.len().saturating_sub(1));
        let anchor = provider.key(anchor_index);
        let (start, end) = if anchor_index <= index {
            (anchor_index, index)
        } else {
            (index, anchor_index)
        };
        let keys = (start..=end)
            .map(|row| provider.key(row))
            .collect::<BTreeSet<_>>();
        self.selected_count = keys.len();
        self.membership = Membership::Explicit(keys);
        self.anchor = Some(Endpoint::new(anchor, anchor_index));
        self.active = Some(Endpoint::new(key, index));
    }

    fn nearest_selected(&self, provider: &dyn Model, desired: usize) -> Option<(Key, usize)> {
        if provider.len() == 0 || self.is_empty() {
            return None;
        }
        match &self.membership {
            Membership::Explicit(keys) => keys
                .iter()
                .filter_map(|key| provider.index_of(*key).map(|index| (*key, index)))
                .min_by_key(|(_, index)| index.abs_diff(desired)),
            Membership::AllExcept(excluded) => {
                let start = desired.min(provider.len() - 1);
                (0..provider.len()).find_map(|distance| {
                    [
                        start.saturating_sub(distance),
                        start.saturating_add(distance),
                    ]
                    .into_iter()
                    .filter(|index| *index < provider.len())
                    .map(|index| (provider.key(index), index))
                    .find(|(key, _)| !excluded.contains(key))
                })
            }
        }
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    #[derive(Clone)]
    struct Keys(Rc<RefCell<Vec<u64>>>);

    impl Model for Keys {
        fn len(&self) -> usize {
            self.0.borrow().len()
        }

        fn key(&self, index: usize) -> Key {
            Key::new(self.0.borrow()[index])
        }

        fn index_of(&self, key: Key) -> Option<usize> {
            self.0
                .borrow()
                .iter()
                .position(|value| *value == key.value())
        }

        fn membership_revision(&self) -> u64 {
            0
        }

        fn changes_since(&self, _revision: u64) -> Vec<crate::list::Change> {
            Vec::new()
        }

        fn item_revision(&self, _index: usize) -> u64 {
            0
        }
    }

    fn keys(values: impl IntoIterator<Item = u64>) -> Keys {
        Keys(Rc::new(RefCell::new(values.into_iter().collect())))
    }

    #[test]
    fn click_toggle_and_range_use_stable_keys() {
        let provider = keys(10..20);
        let mut selection = Selection::new();

        assert!(selection.click(&provider, Key::new(12), 2, false, false));
        assert!(selection.click(&provider, Key::new(14), 4, false, true));
        assert!(selection.contains(Key::new(12)));
        assert!(selection.contains(Key::new(14)));
        assert!(selection.click(&provider, Key::new(16), 6, true, false));
        assert_eq!(selection.anchor(), Some(Key::new(14)));
        assert_eq!(selection.active(), Some(Key::new(16)));
        assert_eq!(selection.len(), 3);
        assert!(!selection.contains(Key::new(12)));
        assert!(selection.contains(Key::new(15)));
    }

    #[test]
    fn select_all_is_constant_state_and_toggle_records_one_exception() {
        let provider = keys(0..1_000_000);
        let mut selection = Selection::new();

        assert!(selection.select_all(&provider));
        assert!(selection.is_all());
        assert_eq!(selection.len(), 1_000_000);
        assert!(selection.click(&provider, Key::new(500_000), 500_000, false, true));
        assert!(!selection.is_all());
        assert_eq!(selection.len(), 999_999);
        assert!(!selection.contains(Key::new(500_000)));
        assert!(selection.contains(Key::new(999_999)));
    }

    #[test]
    fn reorder_and_deletion_reconcile_anchor_active_and_membership_by_key() {
        let provider = keys([10, 11, 12, 13, 14]);
        let mut selection = Selection::new();
        selection.click(&provider, Key::new(11), 1, false, false);
        selection.click(&provider, Key::new(13), 3, true, false);

        provider.0.borrow_mut().reverse();
        assert!(selection.reconcile(&provider));
        assert_eq!(selection.anchor(), Some(Key::new(11)));
        assert_eq!(selection.active(), Some(Key::new(13)));
        assert!(selection.contains(Key::new(12)));

        provider
            .0
            .borrow_mut()
            .retain(|key| *key != 13 && *key != 12);
        assert!(selection.reconcile(&provider));
        assert_eq!(selection.len(), 1);
        assert_eq!(selection.active(), Some(Key::new(11)));
        assert_eq!(selection.anchor(), Some(Key::new(11)));
    }

    #[test]
    fn navigation_clamps_and_shift_extends_from_original_anchor() {
        let provider = keys(0..10);
        let mut selection = Selection::new();
        selection.click(&provider, Key::new(4), 4, false, false);

        assert!(selection.move_active(&provider, Move::Next, true));
        assert!(selection.move_active(&provider, Move::PageNext(3), true));
        assert_eq!(selection.anchor(), Some(Key::new(4)));
        assert_eq!(selection.active(), Some(Key::new(8)));
        assert_eq!(selection.len(), 5);
        assert!(selection.move_active(&provider, Move::Last, false));
        assert_eq!(selection.len(), 1);
        assert_eq!(selection.active(), Some(Key::new(9)));
    }
}
