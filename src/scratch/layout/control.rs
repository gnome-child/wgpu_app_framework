use super::super::{
    geometry::{Point, Rect},
    theme,
};

pub(in crate::scratch) fn slider_track_rect(rect: Rect, metrics: &theme::Metrics) -> Rect {
    let track_left = rect
        .x()
        .saturating_add(metrics.slider_inset)
        .saturating_add(metrics.slider_label_width)
        .saturating_add(metrics.slider_gap);
    let track_right = rect.right().saturating_sub(metrics.slider_inset);
    let track_width = track_right.saturating_sub(track_left);
    let track_y = rect
        .y()
        .saturating_add((rect.height().saturating_sub(metrics.slider_track_height)) / 2);

    Rect::new(
        track_left,
        track_y,
        track_width,
        metrics.slider_track_height,
    )
}

pub(in crate::scratch) fn slider_fraction_at(
    rect: Rect,
    metrics: &theme::Metrics,
    point: Point,
) -> f64 {
    let track = slider_track_rect(rect, metrics);
    let width = track.width().max(1) as f64;
    let offset = point.x().saturating_sub(track.x()) as f64;

    offset / width
}
