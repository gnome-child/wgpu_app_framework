use super::super::{layout, window};
use super::{Color, Scene};

#[derive(Clone)]
pub struct Presentation {
    window: window::Id,
    layout: layout::Layout,
    scene: Scene,
}

impl Presentation {
    pub(in crate::scratch) fn new(window: window::Id, layout: layout::Layout) -> Self {
        Self::with_canvas_color(window, layout, Scene::default_clear())
    }

    pub(in crate::scratch) fn with_canvas_color(
        window: window::Id,
        layout: layout::Layout,
        canvas_color: Color,
    ) -> Self {
        let scene = Scene::paint_with_clear(&layout, canvas_color);
        Self {
            window,
            layout,
            scene,
        }
    }

    pub fn window(&self) -> window::Id {
        self.window
    }

    pub fn layout(&self) -> &layout::Layout {
        &self.layout
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn into_scene(self) -> Scene {
        self.scene
    }
}
