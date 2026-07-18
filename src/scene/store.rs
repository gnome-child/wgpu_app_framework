use std::collections::HashMap;
use std::sync::Arc;

use super::super::{interaction, layout, notification, overlay, theme::Theme, window};
use super::{Color, Commit, Properties, Residency, Stack, Visuals, paint, residency};

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
        let previous_semantic = self.semantics.get(&window);
        let keyed_residency = layout
            .residency_deltas()
            .iter()
            .any(|delta| !delta.is_reset());
        let (commit, semantic_candidate_nodes_visited, semantic_candidate_draws_visited) =
            if keyed_residency && let Some(previous) = previous_semantic {
                // A typed keyed-residency delta proves that semantic content and
                // order are unchanged. The resident candidate is a drawable
                // coverage snapshot; it must not reconstruct semantic identity.
                (Arc::clone(previous), 0, 0)
            } else {
                let resident_nodes = layout.virtual_resident_node_ids();
                let resident_scrolls = layout.residency_content_scroll_node_ids();
                let node_visits = candidate.nodes().len();
                let draw_visits = candidate.order().map_or(0, <[super::Draw]>::len);
                let commit = Commit::semantic_projection(
                    &candidate,
                    previous_semantic,
                    &resident_nodes,
                    &resident_scrolls,
                )
                .expect("painted semantic scene must satisfy the commit contract");
                (commit, node_visits, draw_visits)
            };
        let semantic_changed =
            previous_semantic.is_none_or(|previous| !Arc::ptr_eq(previous, &commit));
        let previous_drawable = self.drawables.get(&window);
        let drawable = if candidate.revision() == commit.revision() {
            candidate
        } else {
            candidate.with_revision_reusing(commit.revision(), previous_drawable)
        };
        let properties = properties.with_commit_revision(&drawable);
        let previous = self.residencies.get(&window).map_or(&[][..], Arc::as_ref);
        let (residencies, residency_stats) =
            residency::Builder::new(&commit, Arc::clone(&drawable), previous)
                .build_with_stats(layout)
                .expect(
                    "painted scene residency must satisfy its layout proof and semantic commit",
                );
        self.semantics.insert(window, Arc::clone(&commit));
        self.drawables.insert(window, Arc::clone(&drawable));
        self.residencies.insert(window, Arc::clone(&residencies));
        let mut stats = PaintStats::from(stats);
        stats.commits_created = usize::from(semantic_changed);
        stats.semantic_candidate_nodes_visited = semantic_candidate_nodes_visited;
        stats.semantic_candidate_draws_visited = semantic_candidate_draws_visited;
        stats.residency_layout_frames_visited = residency_stats.layout_frames_visited;
        stats.residency_drawable_nodes_visited = residency_stats.drawable_nodes_visited;
        stats.residency_draw_ops_visited = residency_stats.draw_ops_visited;
        stats.residency_snapshot_nodes_built = residency_stats.snapshot_nodes_built;
        (commit, drawable, residencies, properties, overlays, stats)
    }

    pub(crate) fn tick_properties(
        &mut self,
        window: window::Id,
        layout: &layout::Layout,
        visuals: &Visuals,
        interaction: Option<&interaction::Interaction>,
        presented: Option<&Stack>,
    ) -> Option<(
        Arc<Commit>,
        Arc<Commit>,
        Arc<[Residency]>,
        Properties,
        Vec<overlay::Draft>,
        PaintStats,
    )> {
        let retained = self.windows.get_mut(&window)?;
        if let Some(presented) = presented {
            let (properties, overlays) =
                retained.tick_presented_properties(layout, visuals, interaction, presented)?;
            let (commit, drawable, residencies) = presented.base_snapshots();
            return Some((
                commit,
                drawable,
                residencies,
                properties,
                overlays,
                PaintStats::default(),
            ));
        }
        let (_, properties, overlays) = retained.tick_properties(layout, visuals, interaction)?;
        let candidate_drawable = Arc::clone(self.drawables.get(&window)?);
        let properties = properties.with_commit_revision(&candidate_drawable);
        let commit = Arc::clone(self.semantics.get(&window)?);
        let drawable = candidate_drawable;
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
    frames_scanned: usize,
    nodes_added: usize,
    nodes_removed: usize,
    node_paints: usize,
    node_reuses: usize,
    row_fragment_reuses: usize,
    row_fragment_builds: usize,
    row_roots_visited: usize,
    commit_layout_frames_visited: usize,
    commit_nodes_registered: usize,
    commit_fragments_appended: usize,
    commit_draw_ops_lowered: usize,
    cache_entries_swept: usize,
    semantic_candidate_nodes_visited: usize,
    semantic_candidate_draws_visited: usize,
    residency_layout_frames_visited: usize,
    residency_drawable_nodes_visited: usize,
    residency_draw_ops_visited: usize,
    residency_snapshot_nodes_built: usize,
    track_paints: usize,
    chrome_paints: usize,
}

impl PaintStats {
    pub(crate) fn commits_created(self) -> usize {
        self.commits_created
    }

    pub(crate) fn frames_scanned(self) -> usize {
        self.frames_scanned
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

    pub(crate) fn row_fragment_reuses(self) -> usize {
        self.row_fragment_reuses
    }

    pub(crate) fn row_fragment_builds(self) -> usize {
        self.row_fragment_builds
    }

    pub(crate) fn row_roots_visited(self) -> usize {
        self.row_roots_visited
    }

    pub(crate) fn commit_layout_frames_visited(self) -> usize {
        self.commit_layout_frames_visited
    }

    pub(crate) fn commit_nodes_registered(self) -> usize {
        self.commit_nodes_registered
    }

    pub(crate) fn commit_fragments_appended(self) -> usize {
        self.commit_fragments_appended
    }

    pub(crate) fn commit_draw_ops_lowered(self) -> usize {
        self.commit_draw_ops_lowered
    }

    pub(crate) fn cache_entries_swept(self) -> usize {
        self.cache_entries_swept
    }

    pub(crate) fn semantic_candidate_nodes_visited(self) -> usize {
        self.semantic_candidate_nodes_visited
    }

    pub(crate) fn semantic_candidate_draws_visited(self) -> usize {
        self.semantic_candidate_draws_visited
    }

    pub(crate) fn residency_layout_frames_visited(self) -> usize {
        self.residency_layout_frames_visited
    }

    pub(crate) fn residency_drawable_nodes_visited(self) -> usize {
        self.residency_drawable_nodes_visited
    }

    pub(crate) fn residency_draw_ops_visited(self) -> usize {
        self.residency_draw_ops_visited
    }

    pub(crate) fn residency_snapshot_nodes_built(self) -> usize {
        self.residency_snapshot_nodes_built
    }

    pub(crate) fn auxiliary_paints(self) -> usize {
        self.track_paints.saturating_add(self.chrome_paints)
    }
}

impl From<paint::RetainedStats> for PaintStats {
    fn from(stats: paint::RetainedStats) -> Self {
        Self {
            commits_created: stats.commits_created,
            frames_scanned: stats.frames_scanned,
            nodes_added: stats.nodes_added,
            nodes_removed: stats.nodes_removed,
            node_paints: stats.node_paints,
            node_reuses: stats.node_reuses,
            row_fragment_reuses: stats.row_fragment_reuses,
            row_fragment_builds: stats.row_fragment_builds,
            row_roots_visited: stats.row_roots_visited,
            commit_layout_frames_visited: stats.commit_layout_frames_visited,
            commit_nodes_registered: stats.commit_nodes_registered,
            commit_fragments_appended: stats.commit_fragments_appended,
            commit_draw_ops_lowered: stats.commit_draw_ops_lowered,
            cache_entries_swept: stats.cache_entries_swept,
            semantic_candidate_nodes_visited: 0,
            semantic_candidate_draws_visited: 0,
            residency_layout_frames_visited: 0,
            residency_drawable_nodes_visited: 0,
            residency_draw_ops_visited: 0,
            residency_snapshot_nodes_built: 0,
            track_paints: stats.track_paints,
            chrome_paints: stats.chrome_paints,
        }
    }
}
