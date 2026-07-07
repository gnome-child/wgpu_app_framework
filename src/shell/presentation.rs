use crate::{geometry, layout, scene, window};

#[derive(Clone)]
pub struct Presentation {
    window: window::Id,
    layout: layout::Layout,
    scene: scene::Scene,
}

impl Presentation {
    pub(super) fn from_scene_presentation(presentation: scene::Presentation) -> Self {
        Self {
            window: presentation.window(),
            layout: presentation.layout().clone(),
            scene: presentation.scene().clone(),
        }
    }

    pub fn window(&self) -> window::Id {
        self.window
    }

    pub fn size(&self) -> geometry::Size {
        self.layout.size()
    }

    #[cfg(test)]
    pub(crate) fn layout(&self) -> &layout::Layout {
        &self.layout
    }

    pub fn scene(&self) -> &scene::Scene {
        &self.scene
    }
}
