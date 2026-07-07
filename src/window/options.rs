use super::{DEFAULT_CANVAS_COLOR, DEFAULT_HEIGHT, DEFAULT_TITLE, DEFAULT_WIDTH};
use crate::{geometry, scene};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    title: String,
    inner_size: geometry::Size,
    canvas_color: scene::Color,
}

impl Options {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            inner_size: Self::default_inner_size(),
            canvas_color: Self::default_canvas_color(),
        }
    }

    pub fn default_inner_size() -> geometry::Size {
        geometry::Size::new(DEFAULT_WIDTH, DEFAULT_HEIGHT)
    }

    pub const fn default_canvas_color() -> scene::Color {
        DEFAULT_CANVAS_COLOR
    }

    pub fn with_inner_size(mut self, size: geometry::Size) -> Self {
        self.inner_size = size;
        self
    }

    pub fn with_canvas_color(mut self, color: scene::Color) -> Self {
        self.canvas_color = color;
        self
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn inner_size(&self) -> geometry::Size {
        self.inner_size
    }

    pub fn canvas_color(&self) -> scene::Color {
        self.canvas_color
    }

    pub(crate) fn into_parts(self) -> (String, geometry::Size, scene::Color) {
        (self.title, self.inner_size, self.canvas_color)
    }
}

impl Default for Options {
    fn default() -> Self {
        Self::new(DEFAULT_TITLE)
    }
}
