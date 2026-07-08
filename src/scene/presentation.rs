use super::super::{geometry, layout, state, window};
use super::Scene;

#[derive(Clone)]
pub struct Presentation {
    window: window::Id,
    revision: state::Revision,
    layout: layout::Layout,
    scene: Scene,
}

impl Presentation {
    fn new(
        window: window::Id,
        revision: state::Revision,
        layout: layout::Layout,
        scene: Scene,
    ) -> Self {
        Self {
            window,
            revision,
            layout,
            scene,
        }
    }

    pub(crate) fn with_scene(
        window: window::Id,
        revision: state::Revision,
        layout: layout::Layout,
        scene: Scene,
    ) -> Self {
        Self::new(window, revision, layout, scene)
    }

    pub fn window(&self) -> window::Id {
        self.window
    }

    pub fn revision(&self) -> state::Revision {
        self.revision
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
}
