use crate::{geometry as paint_geometry, paint, text};

use crate::scratch::{geometry, scene};

pub(in crate::scratch::platform::native) fn to_paint_scene(source: &scene::Scene) -> paint::Scene {
    let mut scene = paint::Scene::new();
    scene.clear(super::color::paint_color(source.clear()));

    for primitive in source.primitives() {
        match primitive {
            scene::Primitive::Quad(quad) => scene.push_quad(to_paint_quad(quad)),
            scene::Primitive::Text(text) => scene.push_text(to_paint_text(text)),
            scene::Primitive::TextViewport(text) => {
                scene.push_text_viewport(to_paint_text_viewport(text));
            }
            scene::Primitive::Icon(icon) => scene.push_icon(to_paint_icon(icon)),
            scene::Primitive::Shadow(shadow) => scene.push_shadow(to_paint_shadow(shadow)),
            scene::Primitive::Outline(outline) => scene.push_outline(to_paint_outline(outline)),
        }
    }

    scene
}

fn to_paint_quad(quad: &scene::Quad) -> paint::Quad {
    paint::Quad {
        rect: into_paint_rounded_rect(quad.rect(), quad.rounding()),
        style: to_paint_style(quad.style()),
        rasterization: to_paint_rasterization(quad.rasterization()),
    }
}

fn to_paint_text(text: &scene::Text) -> paint::Text {
    paint::Text {
        rect: into_paint_rect(text.rect()),
        document: text::document::Document::plain(text.value().to_owned())
            .with_color(super::color::text_color(text.color())),
        wrap: into_paint_text_wrap(text.wrap()),
        vertical_align: paint::TextVerticalAlign::Center,
    }
}

fn to_paint_text_viewport(text: &scene::TextViewport) -> paint::TextViewport {
    paint::TextViewport {
        rect: into_paint_rect(text.rect()),
        surfaces: text.surfaces().iter().map(to_paint_text_surface).collect(),
    }
}

fn to_paint_text_surface(surface: &scene::TextSurface) -> paint::TextSurface {
    let (r, g, b, a) = surface.default_color().channels();

    paint::TextSurface {
        rect: into_paint_rect(surface.rect()),
        buffer: surface.buffer(),
        default_color: paint::Color::rgba(r, g, b, a),
    }
}

fn to_paint_icon(icon: &scene::Icon) -> paint::Icon {
    paint::Icon {
        rect: into_paint_rect(icon.rect()),
        icon: icon.icon(),
        color: super::color::paint_color(icon.color()),
        size: icon.size(),
    }
}

fn to_paint_shadow(shadow: &scene::Shadow) -> paint::Shadow {
    paint::Shadow {
        rect: into_paint_rect(shadow.rect()),
        brush: paint::Brush::solid(super::color::paint_color(shadow.color())),
        blur: shadow.blur(),
        spread: shadow.spread(),
        offset: paint_geometry::point::logical(shadow.offset().x(), shadow.offset().y()),
    }
}

fn to_paint_outline(outline: &scene::Outline) -> paint::Outline {
    paint::Outline {
        rect: into_paint_rect(outline.rect()),
        brush: paint::Brush::solid(super::color::paint_color(outline.color())),
        width: outline.width(),
        offset: outline.offset(),
    }
}

fn into_paint_rect(rect: geometry::Rect) -> paint_geometry::Rect {
    paint_geometry::Rect::new(
        paint_geometry::point::logical(rect.x() as f32, rect.y() as f32),
        paint_geometry::area::logical(rect.width() as f32, rect.height() as f32),
    )
}

fn into_paint_rounded_rect(
    rect: geometry::Rect,
    rounding: scene::Rounding,
) -> paint_geometry::Rect {
    paint_geometry::Rect::rounded(
        paint_geometry::point::logical(rect.x() as f32, rect.y() as f32),
        paint_geometry::area::logical(rect.width() as f32, rect.height() as f32),
        into_paint_rounding(rounding),
    )
}

fn into_paint_text_wrap(wrap: scene::TextWrap) -> paint::TextWrap {
    match wrap {
        scene::TextWrap::None => paint::TextWrap::None,
        scene::TextWrap::WordOrGlyph => paint::TextWrap::WordOrGlyph,
    }
}

fn to_paint_style(style: scene::Style) -> paint::Style {
    paint::Style {
        fill: style
            .fill()
            .map(|brush| paint::Fill::Brush(to_paint_brush(brush))),
        stroke: style.stroke().map(to_paint_stroke),
        tint: style.tint().map(to_paint_brush),
    }
}

fn to_paint_stroke(stroke: scene::Stroke) -> paint::Stroke {
    paint::Stroke {
        brush: to_paint_brush(stroke.brush()),
        width: stroke.width(),
    }
}

fn to_paint_brush(brush: scene::Brush) -> paint::Brush {
    match brush {
        scene::Brush::Solid(color) => paint::Brush::solid(super::color::paint_color(color)),
    }
}

fn to_paint_rasterization(rasterization: scene::Rasterization) -> paint::Rasterization {
    paint::Rasterization {
        snapping: to_paint_snapping(rasterization.snapping()),
        edge_mode: to_paint_edge_mode(rasterization.edge_mode()),
    }
}

fn to_paint_snapping(snapping: scene::Snapping) -> paint::Snapping {
    match snapping {
        scene::Snapping::Disabled => paint::Snapping::Disabled,
        scene::Snapping::Rect => paint::Snapping::Rect,
        scene::Snapping::FixedWidth { width_px } => paint::Snapping::FixedWidth { width_px },
    }
}

fn to_paint_edge_mode(edge_mode: scene::EdgeMode) -> paint::EdgeMode {
    match edge_mode {
        scene::EdgeMode::Antialiased => paint::EdgeMode::Antialiased,
        scene::EdgeMode::Hard => paint::EdgeMode::Hard,
    }
}

fn into_paint_rounding(rounding: scene::Rounding) -> paint_geometry::rect::Rounding {
    paint_geometry::rect::Rounding::new(
        into_paint_radius(rounding.top_left()),
        into_paint_radius(rounding.top_right()),
        into_paint_radius(rounding.bottom_right()),
        into_paint_radius(rounding.bottom_left()),
    )
}

fn into_paint_radius(radius: scene::Radius) -> paint_geometry::rect::Radius {
    match radius {
        scene::Radius::Relative(value) => paint_geometry::rect::Radius::relative(value),
        scene::Radius::Fixed(value) => paint_geometry::rect::Radius::fixed(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scratch::{geometry, layout, theme::Theme, widget};

    #[test]
    fn text_box_surface_color_is_preserved_in_paint_viewport() {
        let view = widget::view(|ui| {
            ui.text_box(widget::TextBox::new("query"));
        });
        let mut engine = layout::engine::Engine::new();
        let layout = layout::Layout::compose(&view, geometry::Size::new(160, 40), &mut engine);
        let source_scene = scene::Scene::paint_with_theme(&layout, &Theme::dark());
        let paint = to_paint_scene(&source_scene);
        let viewport = paint
            .items()
            .iter()
            .find_map(|item| match item {
                paint::Item::TextViewport(text) => Some(text),
                _ => None,
            })
            .expect("text box should convert to paint text viewport");
        let surface = viewport
            .surfaces
            .first()
            .expect("text box viewport should contain a surface");
        let expected = super::super::color::paint_color(scene::Color::rgb(26, 29, 33));

        assert_eq!(surface.default_color, expected);
        assert_eq!(
            surface
                .buffer
                .borrow()
                .lines
                .first()
                .map(|line| line.text().to_owned()),
            Some("query".to_owned())
        );
    }
}
