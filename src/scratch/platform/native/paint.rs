use crate::{geometry as paint_geometry, paint, text};

use crate::scratch::{geometry, scene};

pub(in crate::scratch::platform::native) fn to_paint_scene(source: &scene::Scene) -> paint::Scene {
    let mut scene = paint::Scene::new();
    scene.clear(into_paint_color(source.clear()));

    for primitive in source.primitives() {
        match primitive {
            scene::Primitive::Quad(quad) => scene.push_quad(to_paint_quad(quad)),
            scene::Primitive::Text(text) => scene.push_text(to_paint_text(text)),
            scene::Primitive::TextViewport(text) => {
                scene.push_text_viewport(to_paint_text_viewport(text));
            }
            scene::Primitive::Outline(outline) => scene.push_outline(to_paint_outline(outline)),
        }
    }

    scene
}

fn into_paint_color(color: scene::Color) -> paint::Color {
    let (r, g, b, a) = color.channels();
    paint::Color::rgba(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    )
}

fn to_paint_quad(quad: &scene::Quad) -> paint::Quad {
    paint::Quad {
        rect: into_paint_rect(quad.rect()),
        style: paint::Style {
            fill: Some(paint::Fill::Brush(paint::Brush::solid(into_paint_color(
                quad.fill(),
            )))),
            stroke: None,
            tint: None,
        },
        rasterization: paint::Rasterization::default(),
    }
}

fn to_paint_text(text: &scene::Text) -> paint::Text {
    paint::Text {
        rect: into_paint_rect(text.rect()),
        document: text::document::Document::plain(text.value().to_owned()),
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

fn to_paint_outline(outline: &scene::Outline) -> paint::Outline {
    paint::Outline {
        rect: into_paint_rect(outline.rect()),
        brush: paint::Brush::solid(into_paint_color(outline.color())),
        width: 1.0,
        offset: 0.0,
    }
}

fn into_paint_rect(rect: geometry::Rect) -> paint_geometry::Rect {
    paint_geometry::Rect::new(
        paint_geometry::point::logical(rect.x() as f32, rect.y() as f32),
        paint_geometry::area::logical(rect.width() as f32, rect.height() as f32),
    )
}

fn into_paint_text_wrap(wrap: scene::TextWrap) -> paint::TextWrap {
    match wrap {
        scene::TextWrap::None => paint::TextWrap::None,
        scene::TextWrap::WordOrGlyph => paint::TextWrap::WordOrGlyph,
    }
}
