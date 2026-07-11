use std::time::Instant;

use crate::text as text_engine;

use super::super::{diagnostics, geometry, theme, view};
use super::text;

pub(crate) struct Engine {
    pub(super) text: text::Service,
}

impl Engine {
    pub(crate) fn new() -> Self {
        Self {
            text: text::Service::new(),
        }
    }

    pub(crate) fn text_service(&self) -> text::Service {
        self.text.clone()
    }

    pub(super) fn label_width_with_style(
        &self,
        label: &str,
        style: super::super::theme::TypeStyle,
    ) -> i32 {
        self.text.label_width_with_style(label, style)
    }

    pub(super) fn label_size_for_width_with_style(
        &self,
        label: &str,
        width: i32,
        style: super::super::theme::TypeStyle,
    ) -> geometry::Size {
        self.text
            .label_size_for_width_with_style(label, width, style)
    }

    pub(super) fn resolve_label_overflow(
        &self,
        label: &str,
        width: i32,
        style: super::super::theme::TypeStyle,
        overflow: text_engine::Overflow,
    ) -> String {
        self.text.resolve_overflow(label, width, style, overflow)
    }

    pub(super) fn diagnose_author_text_overflow(
        &self,
        label: &str,
        width: i32,
        height: i32,
        style: super::super::theme::TypeStyle,
    ) {
        self.text
            .diagnose_author_overflow(label, width, height, style);
    }

    pub(super) fn text_area_size_for_width(
        &self,
        text_area: &view::TextArea,
        width: i32,
        theme: &theme::Theme,
    ) -> geometry::Size {
        self.text.text_area_size_for_width(text_area, width, theme)
    }

    pub(super) fn text_area_layout(
        &self,
        text_area: &view::TextArea,
        rect: geometry::Rect,
        theme: &theme::Theme,
        now: Instant,
    ) -> text::Area {
        self.text.text_area_layout(text_area, rect, theme, now)
    }

    pub(super) fn text_area_position_at(
        &self,
        text_area: &view::TextArea,
        layout: &text::Area,
        rect: geometry::Rect,
        position: geometry::Point,
    ) -> Option<text_engine::buffer::Position> {
        self.text
            .text_area_position_at(text_area, layout, rect, position)
    }

    pub(super) fn text_field_layout(
        &self,
        text_box: &view::TextBox,
        rect: geometry::Rect,
        theme: &theme::Theme,
        now: Instant,
    ) -> text::Field {
        self.text.text_field_layout(text_box, rect, theme, now)
    }

    pub(super) fn text_field_position_at(
        &self,
        text_box: &view::TextBox,
        layout: &text::Field,
        rect: geometry::Rect,
        position: geometry::Point,
    ) -> Option<text_engine::buffer::Position> {
        self.text
            .text_field_position_at(text_box, layout, rect, position)
    }

    pub(crate) fn take_text_diagnostics(&mut self) -> diagnostics::Text {
        self.text.take_diagnostics()
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
