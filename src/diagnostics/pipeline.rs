use std::time::Duration;

use super::samples::Samples;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pipeline {
    pub events_received: usize,
    pub frames_prepared: usize,
    pub routing_layouts: usize,
    native_translation: Samples,
    event_handling: Samples,
    native_event_pass: Samples,
    view_rebuild: Samples,
    composition_reconciliation: Samples,
    routing_layout: Samples,
    presentation_layout: Samples,
    scene_assembly: Samples,
}

impl Pipeline {
    pub(crate) fn record_native_translation(&mut self, duration: Duration) {
        self.native_translation.record(duration.as_micros());
    }

    pub(crate) fn record_event_handling(&mut self, duration: Duration) {
        self.events_received += 1;
        self.event_handling.record(duration.as_micros());
    }

    pub(crate) fn record_native_event_pass(&mut self, duration: Duration) {
        self.native_event_pass.record(duration.as_micros());
    }

    pub(crate) fn record_view_rebuild(&mut self, duration: Duration) {
        self.view_rebuild.record(duration.as_micros());
    }

    #[cfg(test)]
    pub(crate) fn record_routing_layout(&mut self, duration: Duration) {
        self.routing_layouts += 1;
        self.routing_layout.record(duration.as_micros());
    }

    pub(crate) fn record_composition_reconciliation(&mut self, duration: Duration) {
        self.composition_reconciliation.record(duration.as_micros());
    }

    pub(crate) fn record_presentation_layout(&mut self, duration: Duration) {
        self.presentation_layout.record(duration.as_micros());
    }

    pub(crate) fn record_scene_assembly(&mut self, duration: Duration) {
        self.scene_assembly.record(duration.as_micros());
    }

    pub(crate) fn record_frame_prepared(&mut self) {
        self.frames_prepared += 1;
    }

    pub fn event_handling_p95_us(&self) -> u128 {
        self.event_handling.p95()
    }

    pub fn native_translation_p95_us(&self) -> u128 {
        self.native_translation.p95()
    }

    pub fn native_event_pass_p95_us(&self) -> u128 {
        self.native_event_pass.p95()
    }

    pub fn view_rebuild_p95_us(&self) -> u128 {
        self.view_rebuild.p95()
    }

    pub fn routing_layout_p95_us(&self) -> u128 {
        self.routing_layout.p95()
    }

    pub fn composition_reconciliation_p95_us(&self) -> u128 {
        self.composition_reconciliation.p95()
    }

    pub fn presentation_layout_p95_us(&self) -> u128 {
        self.presentation_layout.p95()
    }

    pub fn scene_assembly_p95_us(&self) -> u128 {
        self.scene_assembly.p95()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Pipeline;

    #[test]
    fn pipeline_keeps_phase_counts_and_samples_separate() {
        let mut pipeline = Pipeline::default();
        pipeline.record_event_handling(Duration::from_micros(10));
        pipeline.record_routing_layout(Duration::from_micros(20));
        pipeline.record_presentation_layout(Duration::from_micros(30));
        pipeline.record_frame_prepared();

        assert_eq!(pipeline.events_received, 1);
        assert_eq!(pipeline.routing_layouts, 1);
        assert_eq!(pipeline.frames_prepared, 1);
        assert_eq!(pipeline.event_handling_p95_us(), 10);
        assert_eq!(pipeline.routing_layout_p95_us(), 20);
        assert_eq!(pipeline.presentation_layout_p95_us(), 30);
    }
}
