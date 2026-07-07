mod color;
mod material;
mod paint;
mod presentation;
mod primitive;
mod visual;

pub use color::Color;
pub use material::{
    BackdropBlur, BackdropEdgeMode, BackdropLayer, Glass, Luminosity, Material, Noise, Refraction,
    SurfaceLayer,
};
pub use presentation::Presentation;
pub use primitive::{
    Brush, Clip, EdgeMode, Filter, FilterOp, Icon, LiquidFilter, Offset, Outline, Primitive, Quad,
    Radius, Rasterization, Rounding, Shadow, Snapping, Stroke, Style, Text, TextAlign, TextStyle,
    TextSurface, TextViewport, TextWrap, Transform,
};
pub(crate) use visual::Target as TargetVisual;
pub(crate) use visual::Visuals;

use super::{geometry, layout, theme::Theme};

const DEFAULT_CLEAR: Color = Color::rgb(17, 18, 20);

#[derive(Clone)]
pub struct Scene {
    size: geometry::Size,
    clear: Color,
    primitives: Vec<Primitive>,
}

impl Scene {
    #[cfg(test)]
    pub(crate) fn paint(layout: &layout::Layout) -> Self {
        Self::paint_with_clear(layout, DEFAULT_CLEAR)
    }

    #[cfg(test)]
    pub(crate) fn paint_with_theme(layout: &layout::Layout, theme: &Theme) -> Self {
        Self::paint_with_clear_and_theme(layout, theme.surfaces().canvas, theme)
    }

    #[cfg(test)]
    pub(crate) fn paint_with_clear(layout: &layout::Layout, clear: Color) -> Self {
        let theme = Theme::default();
        Self::paint_with_clear_and_theme(layout, clear, &theme)
    }

    #[cfg(test)]
    pub(crate) fn paint_with_clear_and_theme(
        layout: &layout::Layout,
        clear: Color,
        theme: &Theme,
    ) -> Self {
        Self::paint_with_clear_theme_and_visuals(layout, clear, theme, &Visuals::default())
    }

    pub(crate) fn paint_with_clear_theme_and_visuals(
        layout: &layout::Layout,
        clear: Color,
        theme: &Theme,
        visuals: &Visuals,
    ) -> Self {
        let mut scene = Self::new_with_clear(layout.size(), clear);

        paint::paint_layout_with_theme(layout, &mut scene, theme, visuals);

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

    pub fn icons(&self) -> Vec<&Icon> {
        self.primitives
            .iter()
            .filter_map(|primitive| match primitive {
                Primitive::Icon(icon) => Some(icon),
                _ => None,
            })
            .collect()
    }

    pub fn shadows(&self) -> Vec<&Shadow> {
        self.primitives
            .iter()
            .filter_map(|primitive| match primitive {
                Primitive::Shadow(shadow) => Some(shadow),
                _ => None,
            })
            .collect()
    }

    pub fn filters(&self) -> Vec<&Filter> {
        self.primitives
            .iter()
            .filter_map(|primitive| match primitive {
                Primitive::Filter(filter) => Some(filter),
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

    pub fn clips(&self) -> Vec<&Clip> {
        self.primitives
            .iter()
            .filter_map(|primitive| match primitive {
                Primitive::Clip(clip) => Some(clip),
                _ => None,
            })
            .collect()
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

    pub(super) fn push_icon(&mut self, icon: Icon) {
        if icon.rect().width() > 0 && icon.rect().height() > 0 && icon.size() > 0.0 {
            self.primitives.push(Primitive::Icon(icon));
        }
    }

    pub(super) fn push_shadow(&mut self, shadow: Shadow) {
        if shadow.rect().width() > 0
            && shadow.rect().height() > 0
            && shadow.color().channels().3 > 0
        {
            self.primitives.push(Primitive::Shadow(shadow));
        }
    }

    pub(super) fn push_filter(&mut self, filter: Filter) {
        if filter.rect().width() > 0 && filter.rect().height() > 0 && !filter.ops().is_empty() {
            self.primitives.push(Primitive::Filter(filter));
        }
    }

    pub(super) fn push_clip(&mut self, clip: Clip) {
        if clip.rect().width() > 0 && clip.rect().height() > 0 {
            self.primitives.push(Primitive::Clip(clip));
        }
    }

    pub(super) fn pop_clip(&mut self) {
        self.primitives.push(Primitive::PopClip);
    }

    pub(super) fn push_outline(&mut self, outline: Outline) {
        if outline.rect().width() > 0 && outline.rect().height() > 0 {
            self.primitives.push(Primitive::Outline(outline));
        }
    }
}
