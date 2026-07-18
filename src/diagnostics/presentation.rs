use std::{fmt::Write as _, time::Duration};

use super::samples::{SAMPLE_LIMIT, Samples};

pub(crate) const RECEIPT_SCHEMA: &str = "wgpu_l3.presentation_compiler.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrimaryFrameNeed {
    Idle,
    Properties,
    Residency,
    Paint,
    Layout,
    Rebuild,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ChangeSpecies {
    pub(crate) property: bool,
    pub(crate) residency: bool,
    pub(crate) semantic: bool,
    pub(crate) device: bool,
    pub(crate) diagnostic: bool,
}

/// CPU presentation diagnostics for the integrated renderer receipt.
///
/// All cumulative counters saturate and all timing queues retain only the last
/// `SAMPLE_LIMIT` samples. These values are observations: no presentation,
/// layout, scene, or renderer decision may consult them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Presentation {
    pub frames_recorded: usize,
    pub primary_idle: usize,
    pub primary_properties: usize,
    pub primary_residency: usize,
    pub primary_paint: usize,
    pub primary_layout: usize,
    pub primary_rebuild: usize,
    pub property_species_frames: usize,
    pub residency_species_frames: usize,
    pub semantic_species_frames: usize,
    pub device_species_frames: usize,
    pub diagnostic_species_frames: usize,
    pub mixed_property_residency_frames: usize,

    pub materialization_calls: usize,
    pub materialized_lists: usize,
    pub old_interval_start: Option<usize>,
    pub old_interval_end: Option<usize>,
    pub new_interval_start: Option<usize>,
    pub new_interval_end: Option<usize>,
    pub resident_rows_live: usize,
    pub resident_rows_high_water: usize,
    pub entering_rows: usize,
    pub departing_rows: usize,
    pub overlapping_rows: usize,
    pub revised_rows: usize,
    pub moved_rows: usize,
    pub membership_revision_max: u64,
    pub membership_change_events: usize,
    pub provider_binds: usize,
    pub slots_rebound: usize,
    pub view_nodes_cloned: usize,
    pub text_buffers_reused: usize,

    pub composition_reconciliations: usize,
    pub composition_nodes_visited: usize,
    pub composition_nodes_reconstructed: usize,
    pub composition_identities_reused: usize,
    pub composition_nodes_added: usize,
    pub composition_nodes_changed: usize,
    pub composition_nodes_removed: usize,
    pub layout_candidates: usize,
    pub layout_nodes_visited: usize,
    pub layout_nodes_reused: usize,
    pub layout_reused_candidates: usize,
    pub scene_frames_scanned: usize,
    pub scene_frames_painted: usize,
    pub scene_frames_reused: usize,
    pub scene_row_fragments_spliced: usize,
    pub scene_row_fragments_built: usize,
    pub scene_row_roots_visited: usize,
    pub scene_commit_layout_frames_visited: usize,
    pub scene_commit_nodes_registered: usize,
    pub scene_commit_fragments_appended: usize,
    pub scene_commit_draw_ops_lowered: usize,
    pub scene_cache_entries_swept: usize,
    pub scene_semantic_candidate_nodes_visited: usize,
    pub scene_semantic_candidate_draws_visited: usize,
    pub scene_residency_layout_frames_visited: usize,
    pub scene_residency_drawable_nodes_visited: usize,
    pub scene_residency_draw_ops_visited: usize,
    pub scene_residency_snapshot_nodes_built: usize,

    frame_total: Samples,
    materialization: Samples,
    reconciliation: Samples,
}

impl Presentation {
    pub(crate) fn record_frame(
        &mut self,
        primary: PrimaryFrameNeed,
        species: ChangeSpecies,
        duration: Duration,
    ) {
        self.frames_recorded = self.frames_recorded.saturating_add(1);
        match primary {
            PrimaryFrameNeed::Idle => self.primary_idle = self.primary_idle.saturating_add(1),
            PrimaryFrameNeed::Properties => {
                self.primary_properties = self.primary_properties.saturating_add(1)
            }
            PrimaryFrameNeed::Residency => {
                self.primary_residency = self.primary_residency.saturating_add(1)
            }
            PrimaryFrameNeed::Paint => self.primary_paint = self.primary_paint.saturating_add(1),
            PrimaryFrameNeed::Layout => self.primary_layout = self.primary_layout.saturating_add(1),
            PrimaryFrameNeed::Rebuild => {
                self.primary_rebuild = self.primary_rebuild.saturating_add(1)
            }
        }
        self.property_species_frames = self
            .property_species_frames
            .saturating_add(usize::from(species.property));
        self.residency_species_frames = self
            .residency_species_frames
            .saturating_add(usize::from(species.residency));
        self.semantic_species_frames = self
            .semantic_species_frames
            .saturating_add(usize::from(species.semantic));
        self.device_species_frames = self
            .device_species_frames
            .saturating_add(usize::from(species.device));
        self.diagnostic_species_frames = self
            .diagnostic_species_frames
            .saturating_add(usize::from(species.diagnostic));
        self.mixed_property_residency_frames = self
            .mixed_property_residency_frames
            .saturating_add(usize::from(species.property && species.residency));
        self.frame_total.record(duration.as_micros());
    }

    pub(crate) fn record_materialization(
        &mut self,
        stats: crate::list::MaterializationStats,
        text_buffers_reused: usize,
        duration: Duration,
    ) {
        self.materialization_calls = self.materialization_calls.saturating_add(1);
        self.materialized_lists = self.materialized_lists.saturating_add(stats.lists);
        self.old_interval_start = stats.old_interval_start;
        self.old_interval_end = stats.old_interval_end;
        self.new_interval_start = stats.new_interval_start;
        self.new_interval_end = stats.new_interval_end;
        self.resident_rows_live = stats.resident_rows_after;
        self.resident_rows_high_water =
            self.resident_rows_high_water.max(stats.resident_rows_after);
        self.entering_rows = self.entering_rows.saturating_add(stats.entering_rows);
        self.departing_rows = self.departing_rows.saturating_add(stats.departing_rows);
        self.overlapping_rows = self.overlapping_rows.saturating_add(stats.overlapping_rows);
        self.revised_rows = self.revised_rows.saturating_add(stats.revised_rows);
        self.moved_rows = self.moved_rows.saturating_add(stats.moved_rows);
        self.membership_revision_max = self
            .membership_revision_max
            .max(stats.membership_revision_max);
        self.membership_change_events = self
            .membership_change_events
            .saturating_add(stats.membership_changes);
        self.provider_binds = self.provider_binds.saturating_add(stats.provider_binds);
        self.slots_rebound = self.slots_rebound.saturating_add(stats.slots_rebound);
        self.view_nodes_cloned = self
            .view_nodes_cloned
            .saturating_add(stats.view_nodes_cloned);
        self.text_buffers_reused = self.text_buffers_reused.saturating_add(text_buffers_reused);
        self.materialization.record(duration.as_micros());
    }

    pub(crate) fn record_composition(
        &mut self,
        visited: usize,
        reconstructed: usize,
        identities_reused: usize,
        added: usize,
        changed: usize,
        removed: usize,
        duration: Duration,
    ) {
        self.composition_reconciliations = self.composition_reconciliations.saturating_add(1);
        self.composition_nodes_visited = self.composition_nodes_visited.saturating_add(visited);
        self.composition_nodes_reconstructed = self
            .composition_nodes_reconstructed
            .saturating_add(reconstructed);
        self.composition_identities_reused = self
            .composition_identities_reused
            .saturating_add(identities_reused);
        self.composition_nodes_added = self.composition_nodes_added.saturating_add(added);
        self.composition_nodes_changed = self.composition_nodes_changed.saturating_add(changed);
        self.composition_nodes_removed = self.composition_nodes_removed.saturating_add(removed);
        self.reconciliation.record(duration.as_micros());
    }

    pub(crate) fn record_layout(
        &mut self,
        nodes: usize,
        reused_nodes: usize,
        reused_candidate: bool,
    ) {
        self.layout_candidates = self.layout_candidates.saturating_add(1);
        self.layout_nodes_visited = self.layout_nodes_visited.saturating_add(nodes);
        self.layout_nodes_reused = self.layout_nodes_reused.saturating_add(reused_nodes);
        self.layout_reused_candidates = self
            .layout_reused_candidates
            .saturating_add(usize::from(reused_candidate));
    }

    pub(crate) fn record_scene(&mut self, stats: crate::scene::PaintStats) {
        self.scene_frames_scanned = self
            .scene_frames_scanned
            .saturating_add(stats.frames_scanned());
        self.scene_frames_painted = self
            .scene_frames_painted
            .saturating_add(stats.node_paints());
        self.scene_frames_reused = self.scene_frames_reused.saturating_add(stats.node_reuses());
        self.scene_row_fragments_spliced = self
            .scene_row_fragments_spliced
            .saturating_add(stats.row_fragment_reuses());
        self.scene_row_fragments_built = self
            .scene_row_fragments_built
            .saturating_add(stats.row_fragment_builds());
        self.scene_row_roots_visited = self
            .scene_row_roots_visited
            .saturating_add(stats.row_roots_visited());
        self.scene_commit_layout_frames_visited = self
            .scene_commit_layout_frames_visited
            .saturating_add(stats.commit_layout_frames_visited());
        self.scene_commit_nodes_registered = self
            .scene_commit_nodes_registered
            .saturating_add(stats.commit_nodes_registered());
        self.scene_commit_fragments_appended = self
            .scene_commit_fragments_appended
            .saturating_add(stats.commit_fragments_appended());
        self.scene_commit_draw_ops_lowered = self
            .scene_commit_draw_ops_lowered
            .saturating_add(stats.commit_draw_ops_lowered());
        self.scene_cache_entries_swept = self
            .scene_cache_entries_swept
            .saturating_add(stats.cache_entries_swept());
        self.scene_semantic_candidate_nodes_visited = self
            .scene_semantic_candidate_nodes_visited
            .saturating_add(stats.semantic_candidate_nodes_visited());
        self.scene_semantic_candidate_draws_visited = self
            .scene_semantic_candidate_draws_visited
            .saturating_add(stats.semantic_candidate_draws_visited());
        self.scene_residency_layout_frames_visited = self
            .scene_residency_layout_frames_visited
            .saturating_add(stats.residency_layout_frames_visited());
        self.scene_residency_drawable_nodes_visited = self
            .scene_residency_drawable_nodes_visited
            .saturating_add(stats.residency_drawable_nodes_visited());
        self.scene_residency_draw_ops_visited = self
            .scene_residency_draw_ops_visited
            .saturating_add(stats.residency_draw_ops_visited());
        self.scene_residency_snapshot_nodes_built = self
            .scene_residency_snapshot_nodes_built
            .saturating_add(stats.residency_snapshot_nodes_built());
    }

    pub fn frame_total_p95_us(&self) -> u128 {
        self.frame_total.p95()
    }

    pub fn materialization_p95_us(&self) -> u128 {
        self.materialization.p95()
    }

    pub fn reconciliation_p95_us(&self) -> u128 {
        self.reconciliation.p95()
    }

    pub fn receipt_complete(&self, frames_prepared: usize) -> bool {
        self.primary_total() == self.frames_recorded
            && self.frames_recorded == frames_prepared
            && self.frame_total.len() == self.frames_recorded.min(SAMPLE_LIMIT)
            && self.materialization.len() == self.materialization_calls.min(SAMPLE_LIMIT)
            && self.reconciliation.len() == self.composition_reconciliations.min(SAMPLE_LIMIT)
            && self.primary_properties <= self.property_species_frames
            && self.primary_residency <= self.residency_species_frames
    }

    pub(crate) fn receipt_text(&self, frames_prepared: usize) -> String {
        let mut receipt = String::new();
        let _ = writeln!(receipt, "presentation_receipt_schema={RECEIPT_SCHEMA}");
        let _ = writeln!(
            receipt,
            "presentation_receipt_complete={}",
            self.receipt_complete(frames_prepared)
        );
        for (name, value) in [
            ("presentation_frames_recorded", self.frames_recorded),
            ("primary_idle_frames", self.primary_idle),
            ("primary_property_frames", self.primary_properties),
            ("primary_residency_frames", self.primary_residency),
            ("primary_paint_frames", self.primary_paint),
            ("primary_layout_frames", self.primary_layout),
            ("primary_rebuild_frames", self.primary_rebuild),
            ("property_species_frames", self.property_species_frames),
            ("residency_species_frames", self.residency_species_frames),
            ("semantic_species_frames", self.semantic_species_frames),
            ("device_species_frames", self.device_species_frames),
            ("diagnostic_species_frames", self.diagnostic_species_frames),
            (
                "mixed_property_residency_frames",
                self.mixed_property_residency_frames,
            ),
            ("materialization_calls", self.materialization_calls),
            ("materialized_lists", self.materialized_lists),
            ("resident_rows_live", self.resident_rows_live),
            ("resident_rows_high_water", self.resident_rows_high_water),
            ("entering_rows", self.entering_rows),
            ("departing_rows", self.departing_rows),
            ("overlapping_rows", self.overlapping_rows),
            ("revised_rows", self.revised_rows),
            ("moved_rows", self.moved_rows),
            ("membership_change_events", self.membership_change_events),
            ("provider_binds", self.provider_binds),
            ("slots_rebound", self.slots_rebound),
            ("view_nodes_cloned", self.view_nodes_cloned),
            ("text_buffers_reused", self.text_buffers_reused),
            (
                "composition_reconciliations",
                self.composition_reconciliations,
            ),
            ("composition_nodes_visited", self.composition_nodes_visited),
            (
                "composition_nodes_reconstructed",
                self.composition_nodes_reconstructed,
            ),
            (
                "composition_identities_reused",
                self.composition_identities_reused,
            ),
            ("composition_nodes_added", self.composition_nodes_added),
            ("composition_nodes_changed", self.composition_nodes_changed),
            ("composition_nodes_removed", self.composition_nodes_removed),
            ("layout_candidates", self.layout_candidates),
            ("layout_nodes_visited", self.layout_nodes_visited),
            ("layout_nodes_reused", self.layout_nodes_reused),
            ("layout_reused_candidates", self.layout_reused_candidates),
            ("scene_frames_scanned", self.scene_frames_scanned),
            ("scene_frames_painted", self.scene_frames_painted),
            ("scene_frames_reused", self.scene_frames_reused),
            (
                "scene_row_fragments_spliced",
                self.scene_row_fragments_spliced,
            ),
            ("scene_row_fragments_built", self.scene_row_fragments_built),
            ("scene_row_roots_visited", self.scene_row_roots_visited),
            (
                "scene_commit_layout_frames_visited",
                self.scene_commit_layout_frames_visited,
            ),
            (
                "scene_commit_nodes_registered",
                self.scene_commit_nodes_registered,
            ),
            (
                "scene_commit_fragments_appended",
                self.scene_commit_fragments_appended,
            ),
            (
                "scene_commit_draw_ops_lowered",
                self.scene_commit_draw_ops_lowered,
            ),
            ("scene_cache_entries_swept", self.scene_cache_entries_swept),
            (
                "scene_semantic_candidate_nodes_visited",
                self.scene_semantic_candidate_nodes_visited,
            ),
            (
                "scene_semantic_candidate_draws_visited",
                self.scene_semantic_candidate_draws_visited,
            ),
            (
                "scene_residency_layout_frames_visited",
                self.scene_residency_layout_frames_visited,
            ),
            (
                "scene_residency_drawable_nodes_visited",
                self.scene_residency_drawable_nodes_visited,
            ),
            (
                "scene_residency_draw_ops_visited",
                self.scene_residency_draw_ops_visited,
            ),
            (
                "scene_residency_snapshot_nodes_built",
                self.scene_residency_snapshot_nodes_built,
            ),
        ] {
            let _ = writeln!(receipt, "{name}={value}");
        }
        let _ = writeln!(
            receipt,
            "old_resident_interval={}..{}",
            optional_usize(self.old_interval_start),
            optional_usize(self.old_interval_end)
        );
        let _ = writeln!(
            receipt,
            "new_resident_interval={}..{}",
            optional_usize(self.new_interval_start),
            optional_usize(self.new_interval_end)
        );
        let _ = writeln!(
            receipt,
            "membership_revision_max={}",
            self.membership_revision_max
        );
        let _ = writeln!(
            receipt,
            "presentation_frame_total_p95_us={}",
            self.frame_total_p95_us()
        );
        let _ = writeln!(
            receipt,
            "presentation_materialization_p95_us={}",
            self.materialization_p95_us()
        );
        let _ = writeln!(
            receipt,
            "presentation_reconciliation_p95_us={}",
            self.reconciliation_p95_us()
        );
        receipt
    }

    fn primary_total(&self) -> usize {
        self.primary_idle
            .saturating_add(self.primary_properties)
            .saturating_add(self.primary_residency)
            .saturating_add(self.primary_paint)
            .saturating_add(self.primary_layout)
            .saturating_add(self.primary_rebuild)
    }
}

impl Default for Presentation {
    fn default() -> Self {
        Self {
            frames_recorded: 0,
            primary_idle: 0,
            primary_properties: 0,
            primary_residency: 0,
            primary_paint: 0,
            primary_layout: 0,
            primary_rebuild: 0,
            property_species_frames: 0,
            residency_species_frames: 0,
            semantic_species_frames: 0,
            device_species_frames: 0,
            diagnostic_species_frames: 0,
            mixed_property_residency_frames: 0,
            materialization_calls: 0,
            materialized_lists: 0,
            old_interval_start: None,
            old_interval_end: None,
            new_interval_start: None,
            new_interval_end: None,
            resident_rows_live: 0,
            resident_rows_high_water: 0,
            entering_rows: 0,
            departing_rows: 0,
            overlapping_rows: 0,
            revised_rows: 0,
            moved_rows: 0,
            membership_revision_max: 0,
            membership_change_events: 0,
            provider_binds: 0,
            slots_rebound: 0,
            view_nodes_cloned: 0,
            text_buffers_reused: 0,
            composition_reconciliations: 0,
            composition_nodes_visited: 0,
            composition_nodes_reconstructed: 0,
            composition_identities_reused: 0,
            composition_nodes_added: 0,
            composition_nodes_changed: 0,
            composition_nodes_removed: 0,
            layout_candidates: 0,
            layout_nodes_visited: 0,
            layout_nodes_reused: 0,
            layout_reused_candidates: 0,
            scene_frames_scanned: 0,
            scene_frames_painted: 0,
            scene_frames_reused: 0,
            scene_row_fragments_spliced: 0,
            scene_row_fragments_built: 0,
            scene_row_roots_visited: 0,
            scene_commit_layout_frames_visited: 0,
            scene_commit_nodes_registered: 0,
            scene_commit_fragments_appended: 0,
            scene_commit_draw_ops_lowered: 0,
            scene_cache_entries_swept: 0,
            scene_semantic_candidate_nodes_visited: 0,
            scene_semantic_candidate_draws_visited: 0,
            scene_residency_layout_frames_visited: 0,
            scene_residency_drawable_nodes_visited: 0,
            scene_residency_draw_ops_visited: 0,
            scene_residency_snapshot_nodes_built: 0,
            frame_total: Samples::default(),
            materialization: Samples::default(),
            reconciliation: Samples::default(),
        }
    }
}

fn optional_usize(value: Option<usize>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::{ChangeSpecies, Presentation, PrimaryFrameNeed};
    use std::time::Duration;

    #[test]
    fn primary_need_and_change_species_are_independent() {
        let mut diagnostics = Presentation::default();
        diagnostics.record_frame(
            PrimaryFrameNeed::Residency,
            ChangeSpecies {
                property: true,
                residency: true,
                ..ChangeSpecies::default()
            },
            Duration::from_micros(50),
        );

        assert_eq!(diagnostics.primary_residency, 1);
        assert_eq!(diagnostics.property_species_frames, 1);
        assert_eq!(diagnostics.residency_species_frames, 1);
        assert_eq!(diagnostics.mixed_property_residency_frames, 1);
        assert!(diagnostics.receipt_complete(1));
    }

    #[test]
    fn completeness_rejects_erased_concurrent_species() {
        let mut diagnostics = Presentation::default();
        diagnostics.record_frame(
            PrimaryFrameNeed::Residency,
            ChangeSpecies::default(),
            Duration::ZERO,
        );

        assert!(!diagnostics.receipt_complete(1));
    }

    #[test]
    fn completeness_rejects_misclassified_primary_count() {
        let mut diagnostics = Presentation::default();
        diagnostics.record_frame(
            PrimaryFrameNeed::Properties,
            ChangeSpecies {
                property: true,
                ..ChangeSpecies::default()
            },
            Duration::ZERO,
        );
        diagnostics.primary_layout = 1;

        assert!(!diagnostics.receipt_complete(1));
    }
}
