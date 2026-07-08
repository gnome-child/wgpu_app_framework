use std::rc::Rc;

use super::super::super::{
    buffer::Position,
    document::Style,
    edit::{Area, PreeditProjection, ViewState},
};
use super::super::{
    engine::Engine,
    map::TextLayoutMap,
    output::{TextAreaPaintLayout, TextAreaSurface},
    text_area::DisplaySegment as TextAreaDisplaySegment,
};
use crate::paint;

impl Engine {
    pub fn text_area_position_at_for_area(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: paint::area::Logical,
        position: paint::point::Logical,
        state: ViewState,
    ) -> Option<Position> {
        let projection = PreeditProjection::new(area_model.buffer(), area_model.state(), &state);
        let segments = self.text_area_display_segments(
            area_model,
            &projection.buffer,
            !projection.has_preedit(),
            style,
            viewport,
            &state,
        );
        self.text_area_position_at_for_segments(
            &segments,
            position,
            state.scroll_x(),
            projection.buffer.len(),
        )
    }

    pub fn text_area_position_at_for_paint_layout(
        &mut self,
        area_model: &Area,
        position: paint::point::Logical,
        state: ViewState,
        observed_layout: &TextAreaPaintLayout,
    ) -> Option<Position> {
        let projection = PreeditProjection::new(area_model.buffer(), area_model.state(), &state);
        self.text_area_position_at_for_surfaces(
            observed_layout.interaction_surfaces(),
            position,
            observed_layout.layout.scroll_x(),
            projection.buffer.len(),
        )
    }

    pub fn text_area_position_at_for_observed_surfaces(
        &mut self,
        area_model: &Area,
        position: paint::point::Logical,
        state: ViewState,
        scroll_x: f32,
        observed_surfaces: &[TextAreaSurface],
    ) -> Option<Position> {
        let projection = PreeditProjection::new(area_model.buffer(), area_model.state(), &state);
        self.text_area_position_at_for_surfaces(
            observed_surfaces,
            position,
            scroll_x,
            projection.buffer.len(),
        )
    }

    fn text_area_position_at_for_segments(
        &mut self,
        segments: &[TextAreaDisplaySegment],
        position: paint::point::Logical,
        scroll_x: f32,
        text_len: usize,
    ) -> Option<Position> {
        if segments.is_empty() {
            return Some(Position::new(0));
        }

        let mut nearest = None::<(f32, &TextAreaDisplaySegment)>;
        for segment in segments {
            let top = segment.y;
            let bottom = segment.y + segment.display.height.max(1.0);
            if position.y() >= top && position.y() <= bottom {
                nearest = Some((0.0, segment));
                break;
            }
            let distance = if position.y() < top {
                top - position.y()
            } else {
                position.y() - bottom
            };
            if nearest
                .as_ref()
                .is_none_or(|(best_distance, _)| distance < *best_distance)
            {
                nearest = Some((distance, segment));
            }
        }

        let segment = nearest.map(|(_, segment)| segment)?;
        let buffer = segment.display.buffer.borrow();
        self.text_area_position_in_line_buffer(
            &buffer,
            segment.display.source_start,
            position.x() + scroll_x,
            position.y() - segment.y,
            text_len,
        )
    }

    fn text_area_position_at_for_surfaces(
        &mut self,
        surfaces: &[TextAreaSurface],
        position: paint::point::Logical,
        scroll_x: f32,
        text_len: usize,
    ) -> Option<Position> {
        if surfaces.is_empty() {
            return Some(Position::new(0));
        }

        let mut nearest = None::<(f32, &TextAreaSurface)>;
        for surface in surfaces {
            let top = surface.y;
            let bottom = surface.y + surface.height.max(1.0);
            if position.y() >= top && position.y() <= bottom {
                nearest = Some((0.0, surface));
                break;
            }
            let distance = if position.y() < top {
                top - position.y()
            } else {
                position.y() - bottom
            };
            if nearest
                .as_ref()
                .is_none_or(|(best_distance, _)| distance < *best_distance)
            {
                nearest = Some((distance, surface));
            }
        }

        let surface = nearest.map(|(_, surface)| surface)?;
        let buffer = surface.buffer.borrow();
        self.text_area_position_in_line_buffer(
            &buffer,
            surface.source_start,
            position.x() + scroll_x,
            position.y() - surface.y,
            text_len,
        )
    }

    fn text_area_position_in_line_buffer(
        &mut self,
        buffer: &glyphon::Buffer,
        source_start: usize,
        x: f32,
        y: f32,
        text_len: usize,
    ) -> Option<Position> {
        let map = TextLayoutMap::from_line_starts(Rc::new(vec![source_start]));
        let local = map.hit_with_observer(buffer, x, y, |runs| {
            self.diagnostics.text_area_hit_run_scans += runs;
            #[cfg(test)]
            {
                self.interaction_stats.hit_run_scans += runs;
            }
        })?;
        Some(Position::with_affinity(
            local.index.min(text_len),
            local.affinity,
        ))
    }
}
