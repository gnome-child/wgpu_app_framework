use std::collections::HashMap;
use std::sync::Arc;

use super::super::{interaction, layout, notification, overlay, theme::Theme, window};
use super::{Color, Commit, Properties, Residency, Visuals, paint, residency};

#[derive(Default)]
pub(crate) struct Store {
    windows: HashMap<window::Id, paint::Retained>,
    semantics: HashMap<window::Id, Arc<Commit>>,
    drawables: HashMap<window::Id, Arc<Commit>>,
    residencies: HashMap<window::Id, Arc<[Residency]>>,
}

impl Store {
    pub(crate) fn paint(
        &mut self,
        window: window::Id,
        layout: &layout::Layout,
        clear: Color,
        theme: &Theme,
        visuals: &Visuals,
        interaction: Option<&interaction::Interaction>,
    ) -> (
        Arc<Commit>,
        Arc<Commit>,
        Arc<[Residency]>,
        Properties,
        Vec<overlay::Draft>,
        PaintStats,
    ) {
        let retained = self.windows.entry(window).or_default();
        let (candidate, properties, overlays, stats) =
            retained.paint(layout, clear, theme, visuals, interaction);
        let resident_nodes = layout.virtual_resident_node_ids();
        let resident_scrolls = layout.residency_content_scroll_node_ids();
        let previous_semantic = self.semantics.get(&window);
        let commit = Commit::semantic_projection(
            &candidate,
            previous_semantic,
            &resident_nodes,
            &resident_scrolls,
        )
        .expect("painted semantic scene must satisfy the commit contract");
        let semantic_changed =
            previous_semantic.is_none_or(|previous| !Arc::ptr_eq(previous, &commit));
        let drawable = if candidate.revision() == commit.revision() {
            candidate
        } else {
            candidate.with_revision(commit.revision())
        };
        let properties = properties.with_commit_revision(&drawable);
        let previous = self.residencies.get(&window).map_or(&[][..], Arc::as_ref);
        let residencies = residency::Builder::new(&commit, Arc::clone(&drawable), previous)
            .build(layout)
            .expect("painted scene residency must satisfy its layout proof and semantic commit");
        self.semantics.insert(window, Arc::clone(&commit));
        self.drawables.insert(window, Arc::clone(&drawable));
        self.residencies.insert(window, Arc::clone(&residencies));
        let mut stats = PaintStats::from(stats);
        stats.commits_created = usize::from(semantic_changed);
        (commit, drawable, residencies, properties, overlays, stats)
    }

    pub(crate) fn tick_properties(
        &mut self,
        window: window::Id,
        layout: &layout::Layout,
        visuals: &Visuals,
        interaction: Option<&interaction::Interaction>,
    ) -> Option<(
        Arc<Commit>,
        Arc<Commit>,
        Arc<[Residency]>,
        Properties,
        Vec<overlay::Draft>,
        PaintStats,
    )> {
        let retained = self.windows.get_mut(&window)?;
        let (_, properties, overlays) = retained.tick_properties(layout, visuals, interaction)?;
        let commit = Arc::clone(self.semantics.get(&window)?);
        let drawable = Arc::clone(self.drawables.get(&window)?);
        let properties = properties.with_commit_revision(&drawable);
        let residencies = Arc::clone(self.residencies.get(&window)?);
        Some((
            commit,
            drawable,
            residencies,
            properties,
            overlays,
            PaintStats::default(),
        ))
    }

    #[cfg(test)]
    pub(crate) fn residue_count(&self, window: window::Id) -> usize {
        usize::from(self.windows.contains_key(&window))
    }
}

impl notification::Listener<window::Departed> for Store {
    fn notify(&mut self, window: &window::Id) -> notification::Reaction {
        self.windows.remove(window);
        self.semantics.remove(window);
        self.drawables.remove(window);
        self.residencies.remove(window);
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
