use crate::scratch::{geometry, layout, theme::Theme, view};

use super::super::{Brush, Quad, Rounding, Scene, Stroke};

pub(super) fn paint(frame: &layout::frame::Frame, scene: &mut Scene, theme: &Theme) {
    let Some(slider) = frame.slider() else {
        return;
    };
    let slider_theme = theme.slider();
    let rect = frame.rect();
    let track = layout::control::slider_track_rect(rect, theme);
    let filled_width = ((track.width() as f64) * slider_fraction(slider)).round() as i32;
    let fill = geometry::Rect::new(track.x(), track.y(), filled_width, track.height());
    let thumb_center = track.x().saturating_add(filled_width);
    let thumb = geometry::Rect::new(
        thumb_center.saturating_sub(slider_theme.thumb_width / 2),
        rect.y()
            .saturating_add((rect.height().saturating_sub(slider_theme.thumb_height)) / 2),
        slider_theme.thumb_width,
        slider_theme.thumb_height,
    );

    scene.push_quad(Quad::new(track, slider_theme.track).with_rounding(Rounding::relative(1.0)));
    scene.push_quad(Quad::new(fill, slider_theme.value).with_rounding(Rounding::relative(1.0)));
    scene.push_quad(
        Quad::new(thumb, slider_theme.thumb)
            .with_rounding(Rounding::relative(1.0))
            .with_stroke(Stroke::new(Brush::solid(slider_theme.thumb_outline), 1.0)),
    );
}

fn slider_fraction(slider: &view::control::Slider) -> f64 {
    let span = slider.end() - slider.start();
    if span.abs() <= f64::EPSILON {
        return 0.0;
    }

    ((slider.value() - slider.start()) / span).clamp(0.0, 1.0)
}
