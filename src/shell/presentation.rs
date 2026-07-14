use crate::{geometry, layout, response, scene, state, window};

#[derive(Clone)]
pub struct Presentation {
    window: window::Id,
    revision: state::Revision,
    epoch: window::PresentationEpoch,
    invalidation: response::effect::Invalidation,
    layout: layout::Layout,
    scene: scene::Scene,
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
}
