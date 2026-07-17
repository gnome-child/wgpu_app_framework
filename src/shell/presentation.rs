use crate::{geometry, ime, layout, response, scene, state, window};
use std::sync::Arc;

#[derive(Clone)]
pub struct Presentation {
    window: window::Id,
    revision: state::Revision,
    epoch: window::PresentationEpoch,
    invalidation: response::effect::Invalidation,
    layout: layout::Layout,
    scene: scene::Scene,
    stack: Arc<scene::Stack>,
    ime_projection: ime::Projection,
    property_only: bool,
    residency_urgency: Option<scene::ResidencyUrgency>,
}

impl Presentation {
    pub(super) fn from_scene_presentation(
        presentation: scene::Presentation,
        ime_projection: ime::Projection,
    ) -> Self {
        Self {
            window: presentation.window(),
            revision: presentation.revision(),
            epoch: presentation.epoch(),
            invalidation: presentation.invalidation(),
            layout: presentation.layout().clone(),
            scene: presentation.scene().clone(),
            stack: Arc::clone(presentation.stack()),
            ime_projection,
            property_only: presentation.property_only(),
            residency_urgency: presentation.residency_urgency(),
        }
    }

    pub fn window(&self) -> window::Id {
        self.window
    }

    pub fn revision(&self) -> state::Revision {
        self.revision
    }

    pub(crate) fn epoch(&self) -> window::PresentationEpoch {
        self.epoch
    }

    pub(crate) fn invalidation(&self) -> response::effect::Invalidation {
        self.invalidation
    }

    pub fn size(&self) -> geometry::Size {
        self.layout.size()
    }

    pub(crate) fn layout(&self) -> &layout::Layout {
        &self.layout
    }

    pub fn scene(&self) -> &scene::Scene {
        &self.scene
    }

    #[allow(
        dead_code,
        reason = "the semantic commit remains a distinct presentation fact even when activation uses drawable residency"
    )]
    pub(crate) fn commit(&self) -> &Arc<scene::Commit> {
        self.stack.base().commit()
    }

    pub(crate) fn drawable_commit(&self) -> &Arc<scene::Commit> {
        self.stack.base().drawable_commit()
    }

    pub(crate) fn properties(&self) -> &scene::Properties {
        self.stack.base().properties()
    }

    pub(crate) fn property_only(&self) -> bool {
        self.property_only
    }

    pub(crate) fn residency_urgency(&self) -> Option<scene::ResidencyUrgency> {
        self.residency_urgency
    }

    pub(crate) fn stack(&self) -> &Arc<scene::Stack> {
        &self.stack
    }

    pub(crate) fn ime_projection(&self) -> ime::Projection {
        self.ime_projection
    }

    pub(crate) fn with_active_properties(
        &self,
        properties: scene::Properties,
        supplements: &Self,
    ) -> Self {
        let mut presentation = self.clone();
        presentation.stack = Arc::new(
            self.stack
                .with_base_properties_and_spatial_supplements(properties, supplements.stack()),
        );
        presentation.project_popup_ime_from(supplements);
        presentation.property_only = true;
        presentation
    }

    pub(crate) fn with_activation_properties(
        &self,
        properties: scene::Properties,
        supplements: &Self,
    ) -> Self {
        let mut presentation = self.clone();
        presentation.stack = Arc::new(
            self.stack
                .with_base_properties_and_spatial_supplements(properties, supplements.stack()),
        );
        presentation.project_popup_ime_from(supplements);
        presentation
    }

    pub(crate) fn with_spatial_supplements(&self, supplements: &Self) -> Self {
        let mut presentation = self.clone();
        presentation.stack = Arc::new(self.stack.with_spatial_supplements(supplements.stack()));
        presentation.project_popup_ime_from(supplements);
        presentation
    }

    fn project_popup_ime_from(&mut self, supplements: &Self) {
        if self.ime_projection.targets_popup() || supplements.ime_projection.targets_popup() {
            self.ime_projection = supplements.ime_projection;
        }
    }
}
