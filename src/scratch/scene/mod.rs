mod color;
mod paint;
mod presentation;
mod primitive;

pub use color::Color;
pub use presentation::Presentation;
pub use primitive::{Outline, Primitive, Quad, Text, TextSurface, TextViewport, TextWrap};

use super::{geometry, layout};

const DEFAULT_CLEAR: Color = Color::rgb(20, 22, 25);

#[derive(Clone)]
pub struct Scene {
    size: geometry::Size,
    clear: Color,
    primitives: Vec<Primitive>,
}

impl Scene {
    pub fn paint(layout: &layout::Layout) -> Self {
        Self::paint_with_clear(layout, DEFAULT_CLEAR)
    }

    pub fn paint_with_clear(layout: &layout::Layout, clear: Color) -> Self {
        let mut scene = Self::new_with_clear(layout.size(), clear);

        paint::paint_layout(layout, &mut scene);

        scene
    }

    pub fn new(size: geometry::Size) -> Self {
        Self::new_with_clear(size, DEFAULT_CLEAR)
    }

    pub fn new_with_clear(size: geometry::Size, clear: Color) -> Self {
        Self {
            size,
            clear,
            primitives: Vec::new(),
        }
    }

    pub fn size(&self) -> geometry::Size {
        self.size
    }

    pub fn clear(&self) -> Color {
        self.clear
    }

    pub fn primitives(&self) -> &[Primitive] {
        &self.primitives
    }

    pub fn is_empty(&self) -> bool {
        self.primitives.is_empty()
    }

    pub fn quads(&self) -> Vec<&Quad> {
        self.primitives
            .iter()
            .filter_map(|primitive| match primitive {
                Primitive::Quad(quad) => Some(quad),
                _ => None,
            })
            .collect()
    }

    pub fn texts(&self) -> Vec<&Text> {
        self.primitives
            .iter()
            .filter_map(|primitive| match primitive {
                Primitive::Text(text) => Some(text),
                _ => None,
            })
            .collect()
    }

    pub fn text_viewports(&self) -> Vec<&TextViewport> {
        self.primitives
            .iter()
            .filter_map(|primitive| match primitive {
                Primitive::TextViewport(text) => Some(text),
                _ => None,
            })
            .collect()
    }

    pub fn outlines(&self) -> Vec<&Outline> {
        self.primitives
            .iter()
            .filter_map(|primitive| match primitive {
                Primitive::Outline(outline) => Some(outline),
                _ => None,
            })
            .collect()
    }

    pub(super) fn default_clear() -> Color {
        DEFAULT_CLEAR
    }

    pub(super) fn push_quad(&mut self, quad: Quad) {
        if quad.rect().width() > 0 && quad.rect().height() > 0 {
            self.primitives.push(Primitive::Quad(quad));
        }
    }

    pub(super) fn push_text(&mut self, text: Text) {
        if !text.value().is_empty() && text.rect().width() > 0 && text.rect().height() > 0 {
            self.primitives.push(Primitive::Text(text));
        }
    }

    pub(super) fn push_text_viewport(&mut self, text: TextViewport) {
        if !text.surfaces().is_empty() && text.rect().width() > 0 && text.rect().height() > 0 {
            self.primitives.push(Primitive::TextViewport(text));
        }
    }

    pub(super) fn push_outline(&mut self, outline: Outline) {
        if outline.rect().width() > 0 && outline.rect().height() > 0 {
            self.primitives.push(Primitive::Outline(outline));
        }
    }
}
