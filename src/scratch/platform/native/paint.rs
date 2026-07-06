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
            scene::Primitive::Backdrop(backdrop) => {
                scene.push_backdrop(to_paint_backdrop(backdrop))
            }
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
    let mut block = text::document::Block::new(into_text_align(text.align()));
    block.push_run(text::document::Run::new(
        text.value().to_owned(),
        text::document::Style::default().with_color(super::color::text_color(text.color())),
    ));

    paint::Text {
        rect: into_paint_rect(text.rect()),
        document: text::document::Document::from_block(block),
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
        rect: into_paint_rounded_rect(shadow.rect(), shadow.rounding()),
        brush: paint::Brush::solid(super::color::paint_color(shadow.color())),
        blur: shadow.blur(),
        spread: shadow.spread(),
        offset: paint_geometry::point::logical(shadow.offset().x(), shadow.offset().y()),
    }
}

fn to_paint_backdrop(backdrop: &scene::Backdrop) -> paint::Backdrop {
    paint::Backdrop {
        rect: into_paint_rounded_rect(backdrop.rect(), backdrop.rounding()),
        filter: paint::BackdropFilter::Blur {
            amount: backdrop.blur(),
        },
    }
}

fn to_paint_outline(outline: &scene::Outline) -> paint::Outline {
    paint::Outline {
        rect: into_paint_rounded_rect(outline.rect(), outline.rounding()),
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

fn into_text_align(align: scene::TextAlign) -> text::document::Align {
    match align {
        scene::TextAlign::Start => text::document::Align::Start,
        scene::TextAlign::Center => text::document::Align::Center,
        scene::TextAlign::End => text::document::Align::End,
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
        scene::Brush::LinearGradient { from, to } => paint::Brush::linear_gradient(
            super::color::paint_color(from),
            super::color::paint_color(to),
        ),
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
    use crate::scratch::{control_gallery, geometry, input, layout, theme::Theme, view, widget};

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

    #[test]
    fn popup_backdrop_and_gradient_material_convert_to_native_paint() {
        let theme = Theme::dark();
        let view = view::View::new(
            view::Node::root().child(view::Node::popup("popup").child(view::Node::label("Row"))),
        );
        let mut engine = layout::engine::Engine::new();
        let layout = layout::Layout::compose_with_theme(
            &view,
            geometry::Size::new(240, 160),
            &mut engine,
            &theme,
        );
        let source_scene = scene::Scene::paint_with_theme(&layout, &theme);
        let paint = to_paint_scene(&source_scene);
        let backdrop = paint
            .items()
            .iter()
            .find_map(|item| match item {
                paint::Item::Backdrop(backdrop) => Some(backdrop),
                _ => None,
            })
            .expect("popup should convert to native backdrop");
        let paint::BackdropFilter::Blur { amount } = backdrop.filter;

        assert_eq!(amount, theme.floating_panel().backdrop_blur);
        assert_eq!(
            backdrop.rect.rounding,
            paint_geometry::rect::Rounding::fixed(10.0)
        );

        let material = paint
            .items()
            .iter()
            .find_map(|item| match item {
                paint::Item::Quad(quad) if quad.rect == backdrop.rect => Some(quad),
                _ => None,
            })
            .expect("popup material should convert to native quad");

        assert_eq!(
            material.style.fill,
            Some(paint::Fill::Brush(paint::Brush::linear_gradient(
                super::super::color::paint_color(scene::Color::rgba(28, 28, 30, 87)),
                super::super::color::paint_color(scene::Color::rgba(44, 44, 46, 117)),
            )))
        );
    }

    #[test]
    fn focus_outline_offset_and_rounding_convert_to_native_paint() {
        let theme = Theme::default();
        let mut app = control_gallery::app(control_gallery::State::default());

        app.start();

        let window = app.session().windows()[0].id();
        let size = geometry::Size::new(760, 520);
        app.render_scene(window, size)
            .expect("control gallery should render before focus");
        app.handle_input(
            window,
            input::Input::key_down(input::Key::Tab, input::Modifiers::default()),
        )
        .expect("tab should focus first control");

        let focused = app
            .render_scene(window, size)
            .expect("focused control should render");
        let frame = focused
            .layout()
            .frames()
            .iter()
            .find(|frame| frame.is_focused())
            .expect("focused frame should be present");
        let paint = to_paint_scene(focused.scene());
        let expected_brush =
            paint::Brush::solid(super::super::color::paint_color(theme.focus().color));
        let outline = paint
            .items()
            .iter()
            .find_map(|item| match item {
                paint::Item::Outline(outline) if outline.brush == expected_brush => Some(outline),
                _ => None,
            })
            .expect("focus outline should convert to native paint");

        assert_eq!(
            outline.rect,
            into_paint_rounded_rect(frame.rect(), theme.control().rounding)
        );
        assert_eq!(outline.width, theme.focus().width as f32);
        assert_eq!(outline.offset, theme.focus().offset);
    }
}
