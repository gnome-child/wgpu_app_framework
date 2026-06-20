pub use super::{SelectionSpan, TextAreaPaintLayout, TextAreaSurface, TextFieldLayout};

use super::{
    Caret,
    view::{Viewport, Visibility},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaretLayout {
    caret: Caret,
}

impl CaretLayout {
    pub fn new(caret: Caret) -> Self {
        Self { caret }
    }

    pub fn caret(self) -> Caret {
        self.caret
    }

    pub fn visibility_in(self, viewport: Viewport, margin: f32) -> Visibility {
        viewport.visibility_of_local_caret(self.caret, margin)
    }
}
