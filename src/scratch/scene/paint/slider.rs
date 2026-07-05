use crate::scratch::{geometry, layout, theme::Theme, view};

use super::super::{Brush, Quad, Rounding, Scene, Stroke};

pub(super) fn paint(frame: &layout::frame::Frame, scene: &mut Scene, theme: &Theme) {
    let Some(slider) = frame.slider() else {
        return;
    };
    let metrics = theme.metrics();
    let controls = theme.controls();
    let rect = frame.rect();
    let track = layout::control::slider_track_rect(rect, metrics);
    let filled_width = ((track.width() as f64) * slider_fraction(slider)).round() as i32;
    let fill = geometry::Rect::new(track.x(), track.y(), filled_width, track.height());
    let thumb_center = track.x().saturating_add(filled_width);
    let thumb = geometry::Rect::new(
        thumb_center.saturating_sub(metrics.slider_thumb_width / 2),
        rect.y()
            .saturating_add((rect.height().saturating_sub(metrics.slider_thumb_height)) / 2),
        metrics.slider_thumb_width,
        metrics.slider_thumb_height,
    );

    scene.push_quad(Quad::new(track, controls.slider_track).with_rounding(Rounding::relative(1.0)));
    scene.push_quad(Quad::new(fill, controls.slider_value).with_rounding(Rounding::relative(1.0)));
    scene.push_quad(
        Quad::new(thumb, controls.slider_thumb)
            .with_rounding(Rounding::relative(1.0))
            .with_stroke(Stroke::new(
                Brush::solid(controls.slider_thumb_outline),
                1.0,
            )),
    );
}

fn slider_fraction(slider: &view::control::Slider) -> f64 {
    let span = slider.end() - slider.start();
    if span.abs() <= f64::EPSILON {
        return 0.0;
    }

    ((slider.value() - slider.start()) / span).clamp(0.0, 1.0)
}
