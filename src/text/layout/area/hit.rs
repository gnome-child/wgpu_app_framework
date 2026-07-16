use crate::geometry::{area, point};
use std::rc::Rc;

use super::super::super::{
    Preedit,
    buffer::Position,
    document::Style,
    surface::{Area, PreeditProjection},
    view::ViewState,
};
use super::super::{
    engine::Engine,
    map::TextLayoutMap,
    output::{TextAreaPaintLayout, TextAreaSurface},
    text_area::DisplaySegment as TextAreaDisplaySegment,
};

impl Engine {
    pub fn text_area_position_at_for_area(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        position: point::Logical,
        state: ViewState,
    ) -> Option<Position> {
        let projection = PreeditProjection::new(area_model.buffer(), area_model.state(), None);
        let segments = self.text_area_display_segments(
            area_model,
            &projection.buffer,
            !projection.has_preedit(),
            style,
            viewport,
            &state,
            None,
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
        position: point::Logical,
        _state: ViewState,
        observed_layout: &TextAreaPaintLayout,
    ) -> Option<Position> {
        let projection = PreeditProjection::new(area_model.buffer(), area_model.state(), None);
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
        position: point::Logical,
        state: ViewState,
        scroll_x: f32,
        observed_surfaces: &[TextAreaSurface],
    ) -> Option<Position> {
        self.text_area_position_at_for_observed_surfaces_with_preedit(
            area_model,
            position,
            state,
            None,
            scroll_x,
            observed_surfaces,
        )
    }

    pub(crate) fn text_area_position_at_for_observed_surfaces_with_preedit(
        &mut self,
        area_model: &Area,
        position: point::Logical,
        _state: ViewState,
        preedit: Option<&Preedit>,
        scroll_x: f32,
        observed_surfaces: &[TextAreaSurface],
    ) -> Option<Position> {
        let projection = PreeditProjection::new(area_model.buffer(), area_model.state(), preedit);
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
        position: point::Logical,
        _scroll_x: f32,
        text_len: usize,
    ) -> Option<Position> {
        if segments.is_empty() {
            return Some(Position::new(0));
        }

        let mut nearest = None::<((f32, f32), &TextAreaDisplaySegment)>;
        for segment in segments {
            let distance = (
                axis_distance(
                    segment.y,
                    segment.y + segment.display.height.max(1.0),
                    position.y(),
                ),
                text_buffer_x_distance(
                    &segment.display.buffer.borrow(),
                    segment.display.text_x,
                    position.x(),
                ),
            );
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
            position.x() - segment.display.text_x,
            position.y() - segment.y,
            text_len,
        )
    }

    fn text_area_position_at_for_surfaces(
        &mut self,
        surfaces: &[TextAreaSurface],
        position: point::Logical,
        _scroll_x: f32,
        text_len: usize,
    ) -> Option<Position> {
        if surfaces.is_empty() {
            return Some(Position::new(0));
        }

        let mut nearest = None::<((f32, f32), &TextAreaSurface)>;
        for surface in surfaces {
            let distance = (
                axis_distance(surface.y, surface.y + surface.height.max(1.0), position.y()),
                text_buffer_x_distance(&surface.buffer.borrow(), surface.text_x, position.x()),
            );
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
            position.x() - surface.text_x,
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

fn axis_distance(start: f32, end: f32, value: f32) -> f32 {
    if value < start {
        start - value
    } else if value > end {
        value - end
    } else {
        0.0
    }
}

fn text_buffer_x_distance(buffer: &glyphon::Buffer, text_x: f32, x: f32) -> f32 {
    let mut left = f32::INFINITY;
    let mut right = f32::NEG_INFINITY;
    for run in buffer.layout_runs() {
        for glyph in run.glyphs {
            left = left.min(text_x + glyph.x);
            right = right.max(text_x + glyph.x + glyph.w);
        }
    }
    if !left.is_finite() || !right.is_finite() {
        return (x - text_x).abs();
    }
    axis_distance(left, right, x)
}
