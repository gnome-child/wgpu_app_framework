use std::collections::HashMap;
use std::sync::Arc;

use super::super::{layout, notification, overlay, theme::Theme, window};
use super::{Color, Commit, Properties, Visuals, paint};

#[derive(Default)]
pub(crate) struct Store {
    windows: HashMap<window::Id, paint::Retained>,
}

impl Store {
    pub(crate) fn paint(
        &mut self,
        window: window::Id,
        layout: &layout::Layout,
        clear: Color,
        theme: &Theme,
        visuals: &Visuals,
    ) -> (Arc<Commit>, Properties, Vec<overlay::Draft>, PaintStats) {
        let retained = self.windows.entry(window).or_default();
        let (commit, properties, overlays, stats) = retained.paint(layout, clear, theme, visuals);
        (commit, properties, overlays, PaintStats::from(stats))
    }

    #[cfg(test)]
    pub(crate) fn residue_count(&self, window: window::Id) -> usize {
        usize::from(self.windows.contains_key(&window))
    }
}

impl notification::Listener<window::Departed> for Store {
    fn notify(&mut self, window: &window::Id) -> notification::Reaction {
        self.windows.remove(window);
        notification::Reaction::ignored()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct PaintStats {
    commits_created: usize,
    nodes_added: usize,
    nodes_removed: usize,
    node_paints: usize,
    node_reuses: usize,
    track_paints: usize,
    chrome_paints: usize,
}

impl PaintStats {
    pub(crate) fn commits_created(self) -> usize {
        self.commits_created
    }

    pub(crate) fn nodes_added(self) -> usize {
        self.nodes_added
    }

    pub(crate) fn nodes_removed(self) -> usize {
        self.nodes_removed
    }

    pub(crate) fn node_paints(self) -> usize {
        self.node_paints
    }

    pub(crate) fn node_reuses(self) -> usize {
        self.node_reuses
    }

    pub(crate) fn auxiliary_paints(self) -> usize {
        self.track_paints.saturating_add(self.chrome_paints)
    }
}

impl From<paint::RetainedStats> for PaintStats {
    fn from(stats: paint::RetainedStats) -> Self {
        Self {
            commits_created: stats.commits_created,
            nodes_added: stats.nodes_added,
            nodes_removed: stats.nodes_removed,
            node_paints: stats.node_paints,
            node_reuses: stats.node_reuses,
            track_paints: stats.track_paints,
            chrome_paints: stats.chrome_paints,
        }
    }
}
