use crate::text as text_engine;

use super::super::{diagnostics, geometry, view};
use super::text;

pub struct Engine {
    pub(super) text: text::Service,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            text: text::Service::new(),
        }
    }

    pub(in crate::scratch) fn text_service(&self) -> text::Service {
        self.text.clone()
    }

    pub(super) fn label_width(&self, label: &str) -> i32 {
        self.text.label_width(label)
    }

    pub(super) fn text_area_layout(
        &self,
        text_area: &view::control::TextArea,
        rect: geometry::Rect,
    ) -> text::Area {
        self.text.text_area_layout(text_area, rect)
    }

    pub(super) fn text_area_position_at(
        &self,
        text_area: &view::control::TextArea,
        layout: &text::Area,
        rect: geometry::Rect,
        position: geometry::Point,
    ) -> Option<text_engine::buffer::Position> {
        self.text
            .text_area_position_at(text_area, layout, rect, position)
    }

    pub(super) fn text_field_layout(
        &self,
        text_box: &view::control::TextBox,
        rect: geometry::Rect,
    ) -> text::Field {
        self.text.text_field_layout(text_box, rect)
    }

    pub(super) fn text_field_position_at(
        &self,
        text_box: &view::control::TextBox,
        layout: &text::Field,
        rect: geometry::Rect,
        position: geometry::Point,
    ) -> Option<text_engine::buffer::Position> {
        self.text
            .text_field_position_at(text_box, layout, rect, position)
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
