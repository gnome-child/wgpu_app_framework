use crate::{geometry, layout, theme::Theme, view};

use super::super::{Brush, Quad, Rounding, Scene, Stroke, Transform, Visuals};

pub(super) fn paint(frame: &layout::Frame, scene: &mut Scene, theme: &Theme, visuals: &Visuals) {
    let Some(slider) = frame.slider() else {
        return;
    };
    let rect = frame.rect();
    let track = frame
        .slider_track_rect()
        .unwrap_or_else(|| layout::slider_track_rect(rect, frame.label_width(), theme));
    let filled_width = ((track.width() as f64) * slider_fraction(slider)).round() as i32;
    let fill = geometry::Rect::new(track.x(), track.y(), filled_width, track.height());
    let thumb = layout::slider_thumb_rect(rect, slider, frame.label_width(), theme);
    let slider_theme = theme.slider();
    let scale_y = frame
        .target()
        .map(|target| visuals.slider_track_scale_y(target))
        .unwrap_or_else(|| super::super::VisualScalar::resting(1.0));
    let transform =
        Transform::scale_y_about_rect_center(track, scale_y.value()).with_motion(scale_y.motion());

    scene.push_quad(
        Quad::new(track, slider_theme.track)
            .with_rounding(Rounding::relative(1.0))
            .with_transform(transform),
    );
    scene.push_quad(
        Quad::new(fill, slider_theme.value)
            .with_rounding(Rounding::relative(1.0))
            .with_transform(transform),
    );
    let mut thumb_quad =
        Quad::new(thumb, slider_theme.thumb).with_rounding(Rounding::relative(1.0));
    if slider_theme.thumb_outline.channels().3 > 0 {
        thumb_quad =
            thumb_quad.with_stroke(Stroke::new(Brush::solid(slider_theme.thumb_outline), 1.0));
    }
    scene.push_quad(thumb_quad);
}

fn slider_fraction(slider: &view::Slider) -> f64 {
    let span = slider.end() - slider.start();
    if span.abs() <= f64::EPSILON {
        return 0.0;
    }

    ((slider.value() - slider.start()) / span).clamp(0.0, 1.0)
}
