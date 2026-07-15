use std::sync::Arc;

use super::super::{geometry, layout, response, state, window};
use super::{Commit, Properties, Scene};

#[derive(Clone)]
pub struct Presentation {
    window: window::Id,
    revision: state::Revision,
    epoch: window::PresentationEpoch,
    invalidation: response::effect::Invalidation,
    layout: layout::Layout,
    scene: Scene,
    commit: Arc<Commit>,
    properties: Properties,
    overlays: Scene,
    property_only: bool,
}

impl Presentation {
    fn new(
        window: window::Id,
        revision: state::Revision,
        epoch: window::PresentationEpoch,
        invalidation: response::effect::Invalidation,
        layout: layout::Layout,
        scene: Scene,
        commit: Arc<Commit>,
        properties: Properties,
        overlays: Scene,
        property_only: bool,
    ) -> Self {
        Self {
            window,
            revision,
            epoch,
            invalidation,
            layout,
            scene,
            commit,
            properties,
            overlays,
            property_only,
        }
    }

    pub(crate) fn with_scene(
        window: window::Id,
        revision: state::Revision,
        epoch: window::PresentationEpoch,
        invalidation: response::effect::Invalidation,
        layout: layout::Layout,
        scene: Scene,
        commit: Arc<Commit>,
        properties: Properties,
        overlays: Scene,
        property_only: bool,
    ) -> Self {
        Self::new(
            window,
            revision,
            epoch,
            invalidation,
            layout,
            scene,
            commit,
            properties,
            overlays,
            property_only,
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

    pub(crate) fn commit(&self) -> &Arc<Commit> {
        &self.commit
    }

    pub(crate) fn properties(&self) -> &Properties {
        &self.properties
    }

    pub(crate) fn property_only(&self) -> bool {
        self.property_only
    }

    pub(crate) fn overlays(&self) -> &Scene {
        &self.overlays
    }
}
