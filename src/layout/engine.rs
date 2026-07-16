use std::{cell::RefCell, rc::Rc, time::Instant};

use crate::text as text_engine;

use super::super::{composition, geometry, scene, theme, view};
use super::{Text, text};

pub(crate) struct Engine {
    pub(super) text: text::Service,
}

impl Engine {
    pub(crate) fn new() -> Self {
        Self {
            text: text::Service::new(),
        }
    }

    pub(crate) fn text_caret_map(&self) -> Rc<RefCell<dyn text_engine::selection::CaretMap>> {
        self.text.caret_map()
    }

    pub(super) fn label_width_with_style(
        &self,
        label: &str,
        style: super::super::theme::TypeStyle,
    ) -> i32 {
        self.text.label_width_with_style(label, style)
    }

    #[cfg(test)]
    pub(crate) fn test_label_width_with_style(
        &self,
        label: &str,
        style: super::super::theme::TypeStyle,
    ) -> i32 {
        self.label_width_with_style(label, style)
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

    pub(super) fn resolve_selectable_text(
        &self,
        source: &str,
        width: i32,
        style: super::super::theme::TypeStyle,
        wrap: view::Wrap,
        overflow: text_engine::Overflow,
    ) -> text::Selectable {
        self.text
            .resolve_selectable(source, width, style, wrap, overflow)
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
        owner: composition::tree::NodeId,
        text_area: &view::TextArea,
        rect: geometry::Rect,
        visible_frame: geometry::Rect,
        visible_content: geometry::Rect,
        theme: &theme::Theme,
        color: scene::Color,
        now: Instant,
    ) -> text::Area {
        self.text.text_area_layout(
            owner,
            text_area,
            rect,
            visible_frame,
            visible_content,
            theme,
            color,
            now,
        )
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

    pub(crate) fn take_text_diagnostics(&mut self) -> Text {
        self.text.take_diagnostics()
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
