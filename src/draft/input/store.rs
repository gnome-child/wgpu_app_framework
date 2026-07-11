use std::collections::{HashMap, VecDeque};

use crate::{draft::State, interaction::Target};

pub(crate) const DEFAULT_DRAFT_LIMIT: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Store {
    drafts: HashMap<Target, State>,
    order: VecDeque<Target>,
    limit: usize,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            drafts: HashMap::default(),
            order: VecDeque::default(),
            limit: DEFAULT_DRAFT_LIMIT,
        }
    }
}

impl Store {
    pub(super) fn is_empty(&self) -> bool {
        self.drafts.is_empty()
    }

    pub(super) fn get(&self, target: &Target) -> Option<&State> {
        self.drafts.get(target)
    }

    pub(super) fn get_mut(&mut self, target: &Target) -> Option<&mut State> {
        self.drafts.get_mut(target)
    }

    pub(super) fn contains(&self, target: &Target) -> bool {
        self.drafts.contains_key(target)
    }

    pub(super) fn insert(&mut self, target: Target, draft: State) {
        self.drafts.insert(target, draft);
    }

    pub(super) fn get_or_insert_with(
        &mut self,
        target: Target,
        draft: impl FnOnce() -> State,
    ) -> &mut State {
        self.drafts.entry(target).or_insert_with(draft)
    }

    pub(super) fn remove(&mut self, target: &Target) -> bool {
        let draft_changed = self.drafts.remove(target).is_some();
        let order_len = self.order.len();
        self.order.retain(|existing| existing != target);
        let order_changed = self.order.len() != order_len;

        draft_changed || order_changed
    }

    pub(super) fn clear(&mut self) {
        self.drafts.clear();
        self.order.clear();
    }

    pub(super) fn set_limit(&mut self, limit: usize, protected: Option<&Target>) {
        self.limit = limit;
        self.prune(protected);
    }

    pub(super) fn prune_removed(
        &mut self,
        removed_nodes: &[crate::composition::NodeId],
        removed_elements: &[crate::interaction::Id],
        removed_table_cells: &[crate::table::Cell],
    ) -> bool {
        let stale = self
            .order
            .iter()
            .filter(|target| {
                target.matches_removed_identity(
                    removed_nodes,
                    removed_elements,
                    removed_table_cells,
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        let changed = !stale.is_empty();
        for target in stale {
            self.drafts.remove(&target);
        }
        self.order.retain(|target| {
            !target.matches_removed_identity(removed_nodes, removed_elements, removed_table_cells)
        });
        changed
    }

    pub(super) fn touch(&mut self, target: &Target, protected: Option<&Target>) {
        self.order.retain(|existing| existing != target);
        self.order.push_back(target.clone());
        self.prune(protected);
    }

    fn prune(&mut self, protected: Option<&Target>) {
        let effective_limit = self.limit.max(usize::from(protected.is_some()));

        while self.order.len() > effective_limit {
            let Some(stale) = self.order.pop_front() else {
                break;
            };
            if protected == Some(&stale) {
                self.order.push_back(stale);
                continue;
            }

            self.drafts.remove(&stale);
        }
    }
}
