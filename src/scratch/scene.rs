use crate::{geometry as paint_geometry, paint, text};

use super::{geometry, layout, view, window};

const DEFAULT_CLEAR: Color = Color::rgb(20, 22, 25);

#[derive(Clone)]
pub struct Scene {
    size: geometry::Size,
    clear: Color,
    primitives: Vec<Primitive>,
}

#[derive(Clone)]
pub struct Presentation {
    window: window::Id,
    layout: layout::Layout,
    scene: Scene,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Quad(Quad),
    Text(Text),
    TextViewport(TextViewport),
    Outline(Outline),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quad {
    rect: geometry::Rect,
    fill: Color,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    rect: geometry::Rect,
    value: String,
    color: Color,
    wrap: paint::TextWrap,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextViewport {
    rect: geometry::Rect,
    surfaces: Vec<paint::TextSurface>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outline {
    rect: geometry::Rect,
    color: Color,
}

impl Scene {
    pub fn paint(layout: &layout::Layout) -> Self {
        Self::paint_with_clear(layout, DEFAULT_CLEAR)
    }

    pub fn paint_with_clear(layout: &layout::Layout, clear: Color) -> Self {
        let mut scene = Self::new_with_clear(layout.size(), clear);

        for frame in layout.frames() {
            paint_frame(frame, &mut scene);
        }

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

    pub fn to_paint_scene(&self) -> paint::Scene {
        let mut scene = paint::Scene::new();
        scene.clear(self.clear.into_paint_color());

        for primitive in &self.primitives {
            match primitive {
                Primitive::Quad(quad) => scene.push_quad(quad.to_paint_quad()),
                Primitive::Text(text) => scene.push_text(text.to_paint_text()),
                Primitive::TextViewport(text) => {
                    scene.push_text_viewport(text.to_paint_text_viewport())
                }
                Primitive::Outline(outline) => scene.push_outline(outline.to_paint_outline()),
            }
        }

        scene
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

    fn push_quad(&mut self, quad: Quad) {
        if quad.rect.width() > 0 && quad.rect.height() > 0 {
            self.primitives.push(Primitive::Quad(quad));
        }
    }

    fn push_text(&mut self, text: Text) {
        if !text.value.is_empty() && text.rect.width() > 0 && text.rect.height() > 0 {
            self.primitives.push(Primitive::Text(text));
        }
    }

    fn push_text_viewport(&mut self, text: TextViewport) {
        if !text.surfaces.is_empty() && text.rect.width() > 0 && text.rect.height() > 0 {
            self.primitives.push(Primitive::TextViewport(text));
        }
    }

    fn push_outline(&mut self, outline: Outline) {
        if outline.rect.width() > 0 && outline.rect.height() > 0 {
            self.primitives.push(Primitive::Outline(outline));
        }
    }
}

impl Presentation {
    pub(super) fn new(window: window::Id, layout: layout::Layout) -> Self {
        Self::with_canvas_color(window, layout, DEFAULT_CLEAR)
    }

    pub(super) fn with_canvas_color(
        window: window::Id,
        layout: layout::Layout,
        canvas_color: Color,
    ) -> Self {
        let scene = Scene::paint_with_clear(&layout, canvas_color);
        Self {
            window,
            layout,
            scene,
        }
    }

    pub fn window(&self) -> window::Id {
        self.window
    }

    pub fn layout(&self) -> &layout::Layout {
        &self.layout
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn into_scene(self) -> Scene {
        self.scene
    }
}

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    pub fn channels(self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }

    fn into_paint_color(self) -> paint::Color {
        paint::Color::rgba(
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        )
    }
}

impl Quad {
    fn new(rect: geometry::Rect, fill: Color) -> Self {
        Self { rect, fill }
    }

    pub fn rect(&self) -> geometry::Rect {
        self.rect
    }

    pub fn fill(&self) -> Color {
        self.fill
    }

    fn to_paint_quad(&self) -> paint::Quad {
        paint::Quad {
            rect: into_paint_rect(self.rect),
            style: paint::Style {
                fill: Some(paint::Fill::Brush(paint::Brush::solid(
                    self.fill.into_paint_color(),
                ))),
                stroke: None,
                tint: None,
            },
            rasterization: paint::Rasterization::default(),
        }
    }
}

impl Text {
    fn new(
        rect: geometry::Rect,
        value: impl Into<String>,
        color: Color,
        wrap: paint::TextWrap,
    ) -> Self {
        Self {
            rect,
            value: value.into(),
            color,
            wrap,
        }
    }

    pub fn rect(&self) -> geometry::Rect {
        self.rect
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn wrap(&self) -> paint::TextWrap {
        self.wrap
    }

    fn to_paint_text(&self) -> paint::Text {
        paint::Text {
            rect: into_paint_rect(self.rect),
            document: text::document::Document::plain(self.value.clone()),
            wrap: self.wrap,
            vertical_align: paint::TextVerticalAlign::Center,
        }
    }
}

impl TextViewport {
    fn new(rect: geometry::Rect, surfaces: Vec<paint::TextSurface>) -> Self {
        Self { rect, surfaces }
    }

    pub fn rect(&self) -> geometry::Rect {
        self.rect
    }

    pub fn surfaces(&self) -> &[paint::TextSurface] {
        &self.surfaces
    }

    fn to_paint_text_viewport(&self) -> paint::TextViewport {
        paint::TextViewport {
            rect: into_paint_rect(self.rect),
            surfaces: self.surfaces.clone(),
        }
    }
}

impl Outline {
    fn new(rect: geometry::Rect, color: Color) -> Self {
        Self { rect, color }
    }

    pub fn rect(&self) -> geometry::Rect {
        self.rect
    }

    pub fn color(&self) -> Color {
        self.color
    }

    fn to_paint_outline(&self) -> paint::Outline {
        paint::Outline {
            rect: into_paint_rect(self.rect),
            brush: paint::Brush::solid(self.color.into_paint_color()),
            width: 1.0,
            offset: 0.0,
        }
    }
}

fn into_paint_rect(rect: geometry::Rect) -> paint_geometry::Rect {
    paint_geometry::Rect::new(
        paint_geometry::point::logical(rect.x() as f32, rect.y() as f32),
        paint_geometry::area::logical(rect.width() as f32, rect.height() as f32),
    )
}

fn paint_frame(frame: &layout::Frame, scene: &mut Scene) {
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

fn paint_text_area_layout(
    rect: geometry::Rect,
    text_area: &layout::TextAreaLayout,
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
            .map(|surface| paint::TextSurface {
                rect: into_paint_rect(geometry::Rect::new(
                    rect.x().saturating_add(surface.x().round() as i32),
                    rect.y().saturating_add(surface.y().round() as i32),
                    surface.width().ceil().max(0.0) as i32,
                    surface.height().ceil().max(0.0) as i32,
                )),
                buffer: surface.buffer(),
                default_color: surface.default_color(),
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

fn fill_for(role: view::Role) -> Option<Color> {
    match role {
        view::Role::Root => Some(Color::rgb(20, 22, 25)),
        view::Role::MenuBar => Some(Color::rgb(34, 37, 42)),
        view::Role::Menu => Some(Color::rgb(40, 44, 50)),
        view::Role::Popup => Some(Color::rgb(32, 35, 40)),
        view::Role::Command => Some(Color::rgb(38, 42, 48)),
        view::Role::Separator => Some(Color::rgb(78, 84, 94)),
        view::Role::TextArea => Some(Color::rgb(245, 247, 250)),
        view::Role::Button => Some(Color::rgb(44, 49, 56)),
        view::Role::Checkbox | view::Role::Radio => Some(Color::rgb(31, 35, 40)),
        view::Role::Slider => Some(Color::rgb(31, 35, 40)),
        view::Role::TextBox => Some(Color::rgb(245, 247, 250)),
        view::Role::Panel => Some(Color::rgb(28, 31, 36)),
        view::Role::Label => None,
        view::Role::Stack => None,
    }
}

fn text_for(frame: &layout::Frame) -> Option<&str> {
    frame.label_text().or_else(|| frame.text())
}

fn text_color_for(role: view::Role) -> Color {
    match role {
        view::Role::TextArea | view::Role::TextBox => Color::rgb(26, 29, 33),
        view::Role::Separator => Color::rgb(78, 84, 94),
        _ => Color::rgb(238, 241, 245),
    }
}

fn text_wrap_for(frame: &layout::Frame) -> paint::TextWrap {
    match frame.text_wrap() {
        Some(view::Wrap::None) => paint::TextWrap::None,
        Some(view::Wrap::Word) | None => paint::TextWrap::WordOrGlyph,
    }
}

fn outline_color_for(frame: &layout::Frame) -> Option<Color> {
    if frame.is_focused() && matches!(frame.role(), view::Role::TextArea | view::Role::TextBox) {
        return Some(Color::rgb(76, 132, 255));
    }

    matches!(
        frame.role(),
        view::Role::Popup
            | view::Role::TextArea
            | view::Role::Button
            | view::Role::Slider
            | view::Role::TextBox
            | view::Role::Panel
    )
    .then_some(Color::rgb(75, 80, 88))
}
