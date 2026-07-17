use std::sync::Arc;

use super::super::{geometry, layout, response, state, window};
#[cfg(any(test, feature = "renderer-debug"))]
use super::{Commit, Properties};
use super::{Scene, Stack};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ResidencyUrgency {
    Proactive,
    Required,
}

#[derive(Clone)]
pub struct Presentation {
    window: window::Id,
    revision: state::Revision,
    epoch: window::PresentationEpoch,
    invalidation: response::effect::Invalidation,
    layout: layout::Layout,
    scene: Scene,
    stack: Arc<Stack>,
    property_only: bool,
    residency_urgency: Option<ResidencyUrgency>,
}

impl Presentation {
    fn new(
        window: window::Id,
        revision: state::Revision,
        epoch: window::PresentationEpoch,
        invalidation: response::effect::Invalidation,
        layout: layout::Layout,
        scene: Scene,
        stack: Stack,
        property_only: bool,
        residency_urgency: Option<ResidencyUrgency>,
    ) -> Self {
        Self {
            window,
            revision,
            epoch,
            invalidation,
            layout,
            scene,
            stack: Arc::new(stack),
            property_only,
            residency_urgency,
        }
    }

    pub(crate) fn with_scene(
        window: window::Id,
        revision: state::Revision,
        epoch: window::PresentationEpoch,
        invalidation: response::effect::Invalidation,
        layout: layout::Layout,
        scene: Scene,
        stack: Stack,
        property_only: bool,
        residency_urgency: Option<ResidencyUrgency>,
    ) -> Self {
        Self::new(
            window,
            revision,
            epoch,
            invalidation,
            layout,
            scene,
            stack,
            property_only,
            residency_urgency,
        )
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

    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn into_scene(self) -> Scene {
        self.scene
    }

    #[cfg(any(test, feature = "renderer-debug"))]
    pub(crate) fn commit(&self) -> &Arc<Commit> {
        self.stack.base().commit()
    }

    #[cfg(any(test, feature = "renderer-debug"))]
    pub(crate) fn properties(&self) -> &Properties {
        self.stack.base().properties()
    }

    pub(crate) fn property_only(&self) -> bool {
        self.property_only
    }

    pub(crate) fn residency_urgency(&self) -> Option<ResidencyUrgency> {
        self.residency_urgency
    }

    pub(crate) fn stack(&self) -> &Arc<Stack> {
        &self.stack
    }
}
