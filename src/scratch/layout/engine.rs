use crate::text;

use super::super::{diagnostics, geometry, view};
use super::text::{TextAreaLayout, TextHitMap, TextService};

pub struct Engine {
    pub(super) text: TextService,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            text: TextService::new(),
        }
    }

    pub(in crate::scratch) fn text_service(&self) -> TextService {
        self.text.clone()
    }

    pub(super) fn label_width(&self, label: &str) -> i32 {
        self.text.label_width(label)
    }

    pub(super) fn text_area_layout(
        &self,
        text_area: &view::control::TextArea,
        rect: geometry::Rect,
    ) -> TextAreaLayout {
        self.text.text_area_layout(text_area, rect)
    }

    pub(super) fn text_area_position_at(
        &self,
        text_area: &view::control::TextArea,
        layout: &TextAreaLayout,
        rect: geometry::Rect,
        position: geometry::Point,
    ) -> Option<text::buffer::Position> {
        self.text
            .text_area_position_at(text_area, layout, rect, position)
    }

    pub(super) fn text_hit_map(&self, text: &str) -> TextHitMap {
        TextHitMap::new(text, &self.text)
    }

    pub(in crate::scratch) fn take_text_diagnostics(&mut self) -> diagnostics::Text {
        self.text.take_diagnostics()
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
