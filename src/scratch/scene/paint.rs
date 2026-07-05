use crate::text;

use super::super::{context, geometry, layout, view};
use super::primitive::TextColor;
use super::{Color, Outline, Quad, Scene, Text, TextSurface, TextViewport, TextWrap};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Layer {
    Base,
    Chrome,
    Popup,
}

pub(super) fn paint_layout(layout: &layout::Layout, scene: &mut Scene) {
    for layer in [Layer::Base, Layer::Chrome, Layer::Popup] {
        for frame in layout
            .frames()
            .iter()
            .filter(|frame| layer_for(frame) == layer)
        {
            paint_frame(frame, scene);
        }
    }
}

pub(super) fn paint_frame(frame: &layout::frame::Frame, scene: &mut Scene) {
    if let Some(fill) = fill_for(frame.role()) {
        scene.push_quad(Quad::new(frame.rect(), fill));
    }

    if let Some(text_area) = frame.text_area_layout() {
        paint_text_area_layout(frame.rect(), text_area, scene);
    }

    if let Some(value) = text_for(frame) {
        scene.push_text(Text::new(
            frame.rect(),
            value,
            text_color_for(frame.role()),
            text_wrap_for(frame),
        ));
    }

    if let Some(color) = outline_color_for(frame) {
        scene.push_outline(Outline::new(frame.rect(), color));
    }
}

fn layer_for(frame: &layout::frame::Frame) -> Layer {
    match frame.role() {
        view::node::Role::MenuBar | view::node::Role::Menu => Layer::Chrome,
        view::node::Role::Popup | view::node::Role::Separator => Layer::Popup,
        view::node::Role::Binding if frame.binding_source() == Some(context::Source::Menu) => {
            Layer::Popup
        }
        _ => Layer::Base,
    }
}

fn into_scene_text_color(color: text::Color) -> TextColor {
    let (r, g, b, a) = color.channels();
    TextColor::rgba(r, g, b, a)
}

fn paint_text_area_layout(
    rect: geometry::Rect,
    text_area: &layout::text::TextAreaLayout,
    scene: &mut Scene,
) {
    for span in text_area.layout().selection_spans() {
        if let Some(span) = clip_rect(
            span_rect(rect, span.x(), span.y(), span.width(), span.height()),
            rect,
        ) {
            scene.push_quad(Quad::new(span, Color::rgba(76, 132, 255, 96)));
        }
    }

    scene.push_text_viewport(TextViewport::new(
        rect,
        text_area
            .render_surfaces()
            .iter()
            .map(|surface| {
                TextSurface::new(
                    geometry::Rect::new(
                        rect.x().saturating_add(surface.x().round() as i32),
                        rect.y().saturating_add(surface.y().round() as i32),
                        surface.width().ceil().max(0.0) as i32,
                        surface.height().ceil().max(0.0) as i32,
                    ),
                    surface.buffer(),
                    into_scene_text_color(surface.default_color()),
                )
            })
            .collect(),
    ));

    if let Some(caret) = text_area.layout().caret() {
        if let Some(caret) = clip_rect(
            span_rect(rect, caret.x(), caret.y(), 1.0, caret.height()),
            rect,
        ) {
            scene.push_quad(Quad::new(caret, Color::rgb(26, 29, 33)));
        }
    }
}

fn span_rect(rect: geometry::Rect, x: f32, y: f32, width: f32, height: f32) -> geometry::Rect {
    geometry::Rect::new(
        rect.x().saturating_add(x.floor() as i32),
        rect.y().saturating_add(y.floor() as i32),
        width.ceil().max(0.0) as i32,
        height.ceil().max(0.0) as i32,
    )
}

fn clip_rect(rect: geometry::Rect, bounds: geometry::Rect) -> Option<geometry::Rect> {
    let left = rect.x().max(bounds.x());
    let top = rect.y().max(bounds.y());
    let right = rect
        .x()
        .saturating_add(rect.width())
        .min(bounds.x().saturating_add(bounds.width()));
    let bottom = rect
        .y()
        .saturating_add(rect.height())
        .min(bounds.y().saturating_add(bounds.height()));

    (right > left && bottom > top)
        .then(|| geometry::Rect::new(left, top, right - left, bottom - top))
}

fn fill_for(role: view::node::Role) -> Option<Color> {
    match role {
        view::node::Role::Root => Some(Color::rgb(20, 22, 25)),
        view::node::Role::MenuBar => Some(Color::rgb(34, 37, 42)),
        view::node::Role::Menu => Some(Color::rgb(40, 44, 50)),
        view::node::Role::Popup => Some(Color::rgb(32, 35, 40)),
        view::node::Role::Binding => Some(Color::rgb(38, 42, 48)),
        view::node::Role::Separator => Some(Color::rgb(78, 84, 94)),
        view::node::Role::TextArea => Some(Color::rgb(245, 247, 250)),
        view::node::Role::Button => Some(Color::rgb(44, 49, 56)),
        view::node::Role::Checkbox | view::node::Role::Radio => Some(Color::rgb(31, 35, 40)),
        view::node::Role::Slider => Some(Color::rgb(31, 35, 40)),
        view::node::Role::TextBox => Some(Color::rgb(245, 247, 250)),
        view::node::Role::Panel => Some(Color::rgb(28, 31, 36)),
        view::node::Role::Label => None,
        view::node::Role::Stack => None,
    }
}

fn text_for(frame: &layout::frame::Frame) -> Option<&str> {
    frame.label_text().or_else(|| frame.text())
}

fn text_color_for(role: view::node::Role) -> Color {
    match role {
        view::node::Role::TextArea | view::node::Role::TextBox => Color::rgb(26, 29, 33),
        view::node::Role::Separator => Color::rgb(78, 84, 94),
        _ => Color::rgb(238, 241, 245),
    }
}

fn text_wrap_for(frame: &layout::frame::Frame) -> TextWrap {
    match frame.text_wrap() {
        Some(view::control::Wrap::None) => TextWrap::None,
        Some(view::control::Wrap::Word) | None => TextWrap::WordOrGlyph,
    }
}

fn outline_color_for(frame: &layout::frame::Frame) -> Option<Color> {
    if frame.is_focused()
        && matches!(
            frame.role(),
            view::node::Role::TextArea | view::node::Role::TextBox
        )
    {
        return Some(Color::rgb(76, 132, 255));
    }

    matches!(
        frame.role(),
        view::node::Role::Popup
            | view::node::Role::TextArea
            | view::node::Role::Button
            | view::node::Role::Slider
            | view::node::Role::TextBox
            | view::node::Role::Panel
    )
    .then_some(Color::rgb(75, 80, 88))
}
