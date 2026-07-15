use crate::{geometry, layout, response, scene, state, window};
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
    property_only: bool,
}

impl Presentation {
    pub(super) fn from_scene_presentation(presentation: scene::Presentation) -> Self {
        Self {
            window: presentation.window(),
            revision: presentation.revision(),
            epoch: presentation.epoch(),
            invalidation: presentation.invalidation(),
            layout: presentation.layout().clone(),
            scene: presentation.scene().clone(),
            stack: Arc::clone(presentation.stack()),
            property_only: presentation.property_only(),
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

    pub(crate) fn commit(&self) -> &Arc<scene::Commit> {
        self.stack.base().commit()
    }

    pub(crate) fn properties(&self) -> &scene::Properties {
        self.stack.base().properties()
    }

    pub(crate) fn property_only(&self) -> bool {
        self.property_only
    }

    pub(crate) fn stack(&self) -> &Arc<scene::Stack> {
        &self.stack
    }

    pub(crate) fn with_active_properties(&self, properties: scene::Properties) -> Self {
        let mut presentation = self.clone();
        presentation.stack = Arc::new(self.stack.with_base_properties(properties));
        presentation.property_only = true;
        presentation
    }
}
