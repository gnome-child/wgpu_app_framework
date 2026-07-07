use super::super::{geometry, layout, theme::Theme, window};
use super::{Color, Scene, Visuals};

#[derive(Clone)]
pub struct Presentation {
    window: window::Id,
    layout: layout::Layout,
    scene: Scene,
}

impl Presentation {
    pub(crate) fn with_canvas_color_theme_and_visuals(
        window: window::Id,
        layout: layout::Layout,
        canvas_color: Color,
        theme: &Theme,
        visuals: &Visuals,
    ) -> Self {
        let scene =
            Scene::paint_with_clear_theme_and_visuals(&layout, canvas_color, theme, visuals);
        Self::new(window, layout, scene)
    }

    fn new(window: window::Id, layout: layout::Layout, scene: Scene) -> Self {
        Self {
            window,
            layout,
            scene,
        }
    }

    pub fn window(&self) -> window::Id {
        self.window
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
