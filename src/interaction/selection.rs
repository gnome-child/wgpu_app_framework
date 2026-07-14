use crate::{interaction::Id, selection::Selection, virtual_list};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Selections {
    entries: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    list: Id,
    selection: Selection,
}

impl Selections {
    pub(crate) fn get(&self, list: Id) -> Option<&Selection> {
        self.entries
            .iter()
            .find(|entry| entry.list == list)
            .map(|entry| &entry.selection)
    }

    pub(crate) fn get_mut_or_insert(&mut self, list: Id) -> &mut Selection {
        if let Some(index) = self.entries.iter().position(|entry| entry.list == list) {
            return &mut self.entries[index].selection;
        }
        let index = self.entries.len();
        self.entries.push(Entry {
            list,
            selection: Selection::new(),
        });
        &mut self.entries[index].selection
    }

    pub(crate) fn reconcile(&mut self, models: &[virtual_list::Model]) -> bool {
        let before = self.clone();
        self.entries.retain(|entry| {
            models
                .iter()
                .any(|model| model.id() == entry.list && model.is_selectable())
        });
        for model in models.iter().filter(|model| model.is_selectable()) {
            model.reconcile_selection(self.get_mut_or_insert(model.id()));
        }
        *self != before
    }

    pub(crate) fn snapshot(&self) -> Vec<(Id, Selection)> {
        self.entries
            .iter()
            .map(|entry| (entry.list, entry.selection.clone()))
            .collect()
    }

    pub(crate) fn restore(&mut self, entries: Vec<(Id, Selection)>) {
        self.entries = entries
            .into_iter()
            .map(|(list, selection)| Entry { list, selection })
            .collect();
    }
}
