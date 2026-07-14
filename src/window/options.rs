use super::{
    Kind,
    defaults::{DEFAULT_CANVAS_COLOR, DEFAULT_HEIGHT, DEFAULT_TITLE, DEFAULT_WIDTH},
};
use crate::{color, geometry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    title: String,
    inner_size: geometry::Size,
    canvas_color: color::Color,
    kind: Kind,
}

impl Options {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            inner_size: Self::default_inner_size(),
            canvas_color: Self::default_canvas_color(),
            kind: Kind::Application,
        }
    }

    pub fn default_inner_size() -> geometry::Size {
        geometry::Size::new(DEFAULT_WIDTH, DEFAULT_HEIGHT)
    }

    pub const fn default_canvas_color() -> color::Color {
        DEFAULT_CANVAS_COLOR
    }

    pub fn with_inner_size(mut self, size: geometry::Size) -> Self {
        self.inner_size = size;
        self
    }

    pub fn with_canvas_color(mut self, color: color::Color) -> Self {
        self.canvas_color = color;
        self
    }

    pub fn with_kind(mut self, kind: Kind) -> Self {
        self.kind = kind;
        self
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn inner_size(&self) -> geometry::Size {
        self.inner_size
    }

    pub fn canvas_color(&self) -> color::Color {
        self.canvas_color
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub(crate) fn into_parts(self) -> (String, geometry::Size, color::Color, Kind) {
        (self.title, self.inner_size, self.canvas_color, self.kind)
    }
}

impl Default for Options {
    fn default() -> Self {
        Self::new(DEFAULT_TITLE)
    }
}
