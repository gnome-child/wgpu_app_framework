use crate::icon;
use crate::text;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

pub(crate) mod area;
mod grid;
pub(crate) mod point;
mod rect;

pub(crate) use grid::Grid;
pub(crate) use rect::{Radius, Rect, Rounding};

pub(crate) const MAX_FILTER_BLUR_RADIUS_PX: f32 = 256.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    clear_color: Option<Color>,
    items: Vec<Item>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            clear_color: None,
            items: Vec::new(),
        }
    }

    pub fn clear(&mut self, color: Color) {
        self.clear_color = Some(color);
    }

    pub fn clear_color(&self) -> Option<Color> {
        self.clear_color
    }

    pub fn push_quad(&mut self, quad: Quad) {
        self.items.push(Item::Quad(quad));
    }

    pub fn push_rule(&mut self, rule: Rule) {
        self.items.push(Item::Rule(rule));
    }

    pub fn push_text(&mut self, text: Text) {
        if !text.document.is_empty() {
            self.items.push(Item::Text(text));
        }
    }

    pub fn push_text_viewport(&mut self, text: TextViewport) {
        if !text.surfaces.is_empty() {
            self.items.push(Item::TextViewport(text));
        }
    }

    pub fn push_icon(&mut self, icon: Icon) {
        self.items.push(Item::Icon(icon));
    }

    pub fn push_shadow(&mut self, shadow: Shadow) {
        self.items.push(Item::Shadow(shadow));
    }

    pub fn push_pane(&mut self, pane: Pane) {
        if pane.rect.area.width() > 0.0 && pane.rect.area.height() > 0.0 {
            self.items.push(Item::Pane(pane));
        }
    }

    pub fn push_outline(&mut self, outline: Outline) {
        self.items.push(Item::Outline(outline));
    }

    pub fn push_group(&mut self, group: Group) {
        if group.opacity > 0.0
            && group.bounds.area.width() > 0.0
            && group.bounds.area.height() > 0.0
            && !group.items.is_empty()
        {
            self.items.push(Item::Group(group));
        }
    }

    pub fn push_clip(&mut self, clip: Clip) {
        self.items.push(Item::Clip(clip));
    }

    pub fn pop_clip(&mut self) {
        self.items.push(Item::PopClip);
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Quad(Quad),
    Rule(Rule),
    Text(Text),
    TextViewport(TextViewport),
    Icon(Icon),
    Shadow(Shadow),
    Pane(Pane),
    Outline(Outline),
    Clip(Clip),
    PopClip,
    Group(Group),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    rect: Rect,
    style: Style,
    rasterization: Rasterization,
    transform: Transform,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rule {
    pub axis: Axis,
    pub rect: Rect,
    pub brush: Brush,
    pub thickness_px: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub origin: point::Logical,
    pub translate: point::Logical,
    pub scale_x: f32,
    pub scale_y: f32,
    pub motion: Motion,
    pub scale_motion: Option<ScaleMotion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Moving,
    Resting,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaleMotion {
    pub from_x: f32,
    pub from_y: f32,
    pub to_x: f32,
    pub to_y: f32,
    pub progress: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rasterization {
    pub edge_mode: EdgeMode,
}

impl Default for Rasterization {
    fn default() -> Self {
        Self {
            edge_mode: EdgeMode::Antialiased,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeMode {
    Antialiased,
    Hard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub rect: Rect,
    pub document: text::document::Document,
    pub wrap: TextWrap,
    pub vertical_align: TextVerticalAlign,
    pub overflow: text::Overflow,
}

#[derive(Clone)]
pub struct TextSurface {
    pub rect: Rect,
    pub buffer: Rc<RefCell<glyphon::Buffer>>,
    pub default_color: Color,
}

#[derive(Clone)]
pub struct TextViewport {
    pub rect: Rect,
    pub surfaces: Vec<TextSurface>,
}

impl fmt::Debug for TextSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextSurface")
            .field("rect", &self.rect)
            .field("default_color", &self.default_color)
            .finish_non_exhaustive()
    }
}

impl PartialEq for TextSurface {
    fn eq(&self, other: &Self) -> bool {
        self.rect == other.rect && self.default_color == other.default_color
    }
}

impl fmt::Debug for TextViewport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextViewport")
            .field("rect", &self.rect)
            .field("surfaces", &self.surfaces)
            .finish()
    }
}

impl PartialEq for TextViewport {
    fn eq(&self, other: &Self) -> bool {
        self.rect == other.rect && self.surfaces == other.surfaces
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LayerSampling {
    #[default]
    Filtered,
    PixelAligned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWrap {
    WordOrGlyph,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextVerticalAlign {
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Icon {
    pub rect: Rect,
    pub icon: icon::Icon,
    pub color: Color,
    pub size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    pub rect: Rect,
    pub brush: Brush,
    pub blur: f32,
    pub spread: f32,
    pub offset: point::Logical,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pane {
    pub rect: Rect,
    pub source_rect: Option<Rect>,
    pub material: Material,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Outline {
    pub rect: Rect,
    pub brush: Brush,
    pub width: f32,
    pub offset: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Filter {
    pub(crate) rect: Rect,
    pub(crate) source_rect: Option<Rect>,
    pub(crate) ops: Vec<FilterOp>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Clip {
    pub rect: Rect,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub bounds: Rect,
    pub opacity: f32,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FilterOp {
    Blur { amount: f32 },
    BackdropBlur(BackdropBlur),
    Refraction(Refraction),
    Luminosity(Luminosity),
    Noise(Noise),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackdropBlur {
    pub sigma: f32,
    pub edge_mode: BackdropEdgeMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackdropEdgeMode {
    Mirror,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Refraction {
    pub displacement: f32,
    pub splay: f32,
    pub feather: f32,
    pub curve: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Luminosity {
    pub color: Color,
    pub opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Noise {
    pub opacity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Material {
    Solid(Brush),
    Glass(Glass),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Glass {
    pub fallback: Brush,
    pub base: GlassBase,
    pub backdrop_layers: Vec<BackdropLayer>,
    pub surface_layers: Vec<SurfaceLayer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlassBase {
    FrameworkBackdrop,
    Transparent,
    Fallback,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackdropLayer {
    Blur(BackdropBlur),
    Refraction(Refraction),
    Luminosity(Luminosity),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurfaceLayer {
    Tint { brush: Brush, opacity: f32 },
    Noise(Noise),
}

impl Quad {
    pub(crate) fn resolved_for_grid(
        rect: Rect,
        style: Style,
        rasterization: Rasterization,
        transform: Transform,
        grid: Grid,
    ) -> Self {
        let (mut rect, transform) = transform.resolve_rect(rect, grid);
        if transform.motion == Motion::Resting {
            rect = grid.snap_rect(rect);
        }

        Self {
            rect,
            style,
            rasterization,
            transform,
        }
    }

    #[cfg(test)]
    pub(crate) fn unchecked_for_test(
        rect: Rect,
        style: Style,
        rasterization: Rasterization,
        transform: Transform,
    ) -> Self {
        Self {
            rect,
            style,
            rasterization,
            transform,
        }
    }

    pub(crate) fn rect(&self) -> Rect {
        self.rect
    }

    pub(crate) fn style(&self) -> Style {
        self.style
    }

    pub(crate) fn rasterization(&self) -> Rasterization {
        self.rasterization
    }

    pub(crate) fn transform(&self) -> Transform {
        self.transform
    }

    fn bounds(self) -> Rect {
        self.transform.transformed_rect(self.rect)
    }

    fn translated_for_group(self, origin: point::Logical, grid: Grid) -> Self {
        let dx = -origin.x();
        let dy = -origin.y();
        let rect = translate_rect(self.rect, dx, dy);
        let mut transform = self.transform;
        transform.origin = point::logical(transform.origin.x() + dx, transform.origin.y() + dy);

        Self::resolved_for_grid(rect, self.style, self.rasterization, transform, grid)
    }

    #[cfg(test)]
    pub(crate) fn set_rect_for_test(&mut self, rect: Rect) {
        self.rect = rect;
    }

    #[cfg(test)]
    pub(crate) fn set_transform_for_test(&mut self, transform: Transform) {
        self.transform = transform;
    }
}

impl Pane {
    pub fn new(rect: Rect, material: Material) -> Self {
        Self {
            rect,
            source_rect: None,
            material,
        }
    }

    fn translated_for_group(mut self, origin: point::Logical) -> Self {
        let source_rect = self.source_rect.unwrap_or(self.rect);
        self.rect = translate_rect(self.rect, -origin.x(), -origin.y());
        self.source_rect = Some(source_rect);
        self
    }
}

impl Filter {
    pub(crate) fn stack(rect: Rect, ops: impl IntoIterator<Item = FilterOp>) -> Self {
        Self {
            rect,
            source_rect: None,
            ops: ops.into_iter().map(FilterOp::clamped).collect(),
        }
    }
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            origin: point::logical(0.0, 0.0),
            translate: point::logical(0.0, 0.0),
            scale_x: 1.0,
            scale_y: 1.0,
            motion: Motion::Resting,
            scale_motion: None,
        }
    }

    #[cfg(test)]
    pub fn scale_about(origin: point::Logical, scale_x: f32, scale_y: f32) -> Self {
        Self {
            origin,
            scale_x: sanitized_scale(scale_x),
            scale_y: sanitized_scale(scale_y),
            ..Self::identity()
        }
    }

    #[cfg(test)]
    pub fn scale_y_about(origin: point::Logical, scale_y: f32) -> Self {
        Self::scale_about(origin, 1.0, scale_y)
    }

    pub fn is_identity(self) -> bool {
        self.translate.x() == 0.0
            && self.translate.y() == 0.0
            && self.scale_x == 1.0
            && self.scale_y == 1.0
    }

    #[cfg(test)]
    pub fn with_motion(mut self, motion: Motion) -> Self {
        self.motion = motion;
        self
    }

    pub fn with_scale(mut self, scale_x: f32, scale_y: f32) -> Self {
        self.scale_x = sanitized_scale(scale_x);
        self.scale_y = sanitized_scale(scale_y);
        self
    }

    #[cfg(test)]
    pub fn with_scale_motion(
        mut self,
        from_x: f32,
        from_y: f32,
        to_x: f32,
        to_y: f32,
        progress: f32,
    ) -> Self {
        self.scale_motion = Some(ScaleMotion {
            from_x: sanitized_scale(from_x),
            from_y: sanitized_scale(from_y),
            to_x: sanitized_scale(to_x),
            to_y: sanitized_scale(to_y),
            progress: sanitized_progress(progress),
        });
        self
    }

    pub fn transformed_rect(self, rect: Rect) -> Rect {
        if self.is_identity() {
            return rect;
        }

        let left = rect.origin.x();
        let top = rect.origin.y();
        let right = left + rect.area.width();
        let bottom = top + rect.area.height();
        let x0 = self.transform_x(left);
        let x1 = self.transform_x(right);
        let y0 = self.transform_y(top);
        let y1 = self.transform_y(bottom);
        let left = x0.min(x1);
        let top = y0.min(y1);
        let right = x0.max(x1);
        let bottom = y0.max(y1);

        Rect::rounded(
            point::logical(left, top),
            area::logical((right - left).max(0.0), (bottom - top).max(0.0)),
            rect.rounding,
        )
    }

    pub(crate) fn resolve_rect(self, rect: Rect, grid: Grid) -> (Rect, Self) {
        if self.motion == Motion::Moving
            && let Some(scale_motion) = self.scale_motion
        {
            return (
                self.scaled_motion_rect(rect, scale_motion, grid),
                Self {
                    motion: Motion::Moving,
                    ..Self::identity()
                },
            );
        }

        if self.motion == Motion::Resting && !self.is_identity() {
            return (
                grid.snap_rect_with_stable_size(self.transformed_rect(rect)),
                Self::identity(),
            );
        }

        (rect, self)
    }

    fn scaled_motion_rect(self, rect: Rect, scale_motion: ScaleMotion, grid: Grid) -> Rect {
        let from = grid.snap_rect_with_stable_size(
            self.with_scale(scale_motion.from_x, scale_motion.from_y)
                .transformed_rect(rect),
        );
        let to = grid.snap_rect_with_stable_size(
            self.with_scale(scale_motion.to_x, scale_motion.to_y)
                .transformed_rect(rect),
        );

        lerp_rect(from, to, scale_motion.progress)
    }

    fn transform_x(self, x: f32) -> f32 {
        ((x - self.origin.x()) * self.scale_x) + self.origin.x() + self.translate.x()
    }

    fn transform_y(self, y: f32) -> f32 {
        ((y - self.origin.y()) * self.scale_y) + self.origin.y() + self.translate.y()
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

fn sanitized_scale(scale: f32) -> f32 {
    if scale.is_finite() { scale } else { 1.0 }
}

#[cfg(test)]
fn sanitized_progress(progress: f32) -> f32 {
    if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

fn lerp_rect(from: Rect, to: Rect, progress: f32) -> Rect {
    let progress = progress.clamp(0.0, 1.0);
    let left = lerp(from.origin.x(), to.origin.x(), progress);
    let top = lerp(from.origin.y(), to.origin.y(), progress);
    let right = lerp(
        from.origin.x() + from.area.width(),
        to.origin.x() + to.area.width(),
        progress,
    );
    let bottom = lerp(
        from.origin.y() + from.area.height(),
        to.origin.y() + to.area.height(),
        progress,
    );

    Rect::rounded(
        point::logical(left, top),
        area::logical((right - left).max(0.0), (bottom - top).max(0.0)),
        to.rounding,
    )
}

fn lerp(from: f32, to: f32, progress: f32) -> f32 {
    from + ((to - from) * progress)
}

impl FilterOp {
    pub(crate) fn blur(amount: f32) -> Self {
        Self::Blur {
            amount: amount.clamp(0.0, 1.0),
        }
    }

    pub(crate) fn backdrop_blur(params: BackdropBlur) -> Self {
        Self::BackdropBlur(params.clamped())
    }

    pub(crate) fn refraction(params: Refraction) -> Self {
        Self::Refraction(params)
    }

    pub(crate) fn luminosity(params: Luminosity) -> Self {
        Self::Luminosity(params.clamped())
    }

    pub(crate) fn noise(params: Noise) -> Self {
        Self::Noise(params.clamped())
    }

    fn clamped(self) -> Self {
        match self {
            Self::Blur { amount } => Self::blur(amount),
            Self::BackdropBlur(params) => Self::backdrop_blur(params),
            Self::Refraction(params) => Self::Refraction(params),
            Self::Luminosity(params) => Self::luminosity(params),
            Self::Noise(params) => Self::noise(params),
        }
    }
}

impl BackdropBlur {
    fn clamped(self) -> Self {
        Self {
            sigma: self.sigma.max(0.0),
            edge_mode: self.edge_mode,
        }
    }
}

impl Luminosity {
    fn clamped(self) -> Self {
        Self {
            color: self.color,
            opacity: self.opacity.clamp(0.0, 1.0),
        }
    }
}

impl Noise {
    fn clamped(self) -> Self {
        Self {
            opacity: self.opacity.clamp(0.0, 1.0),
        }
    }
}

pub(crate) fn filter_blur_radius_px(amount: f32, scale_factor: f32) -> f32 {
    (amount.clamp(0.0, 1.0) * MAX_FILTER_BLUR_RADIUS_PX * scale_factor.max(0.0001))
        .clamp(0.0, MAX_FILTER_BLUR_RADIUS_PX)
}

pub(crate) fn filter_blur_sigma_px(sigma: f32, scale_factor: f32) -> f32 {
    sigma.max(0.0) * scale_factor.max(0.0001)
}

pub(crate) fn filter_blur_kernel_radius_px(sigma: f32, scale_factor: f32) -> f32 {
    (filter_blur_sigma_px(sigma, scale_factor) * 3.0)
        .ceil()
        .clamp(0.0, MAX_FILTER_BLUR_RADIUS_PX)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
    pub tint: Option<Brush>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fill {
    Brush(Brush),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    pub brush: Brush,
    pub width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Brush {
    Solid(Color),
    Gradient(Gradient),
}

impl Brush {
    pub const fn solid(color: Color) -> Self {
        Self::Solid(color)
    }

    pub fn linear_gradient(from: Color, to: Color) -> Self {
        Self::Gradient(Gradient::linear(from, to))
    }

    #[cfg(test)]
    pub fn dimmed(self, factor: f32) -> Self {
        match self {
            Self::Solid(color) => Self::Solid(color.dimmed(factor)),
            Self::Gradient(gradient) => Self::Gradient(gradient.dimmed(factor)),
        }
    }

    pub fn is_visible(self) -> bool {
        match self {
            Self::Solid(color) => color.a > 0.0,
            Self::Gradient(gradient) => gradient.is_visible(),
        }
    }
}

impl From<Color> for Brush {
    fn from(color: Color) -> Self {
        Self::solid(color)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gradient {
    Linear(LinearGradient),
}

impl Gradient {
    pub fn linear(from: Color, to: Color) -> Self {
        Self::Linear(LinearGradient::new(
            UnitPoint::TOP_LEFT,
            UnitPoint::BOTTOM_RIGHT,
            from,
            to,
        ))
    }

    #[cfg(test)]
    pub fn dimmed(self, factor: f32) -> Self {
        match self {
            Self::Linear(gradient) => Self::Linear(gradient.dimmed(factor)),
        }
    }

    pub fn is_visible(self) -> bool {
        match self {
            Self::Linear(gradient) => gradient.is_visible(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearGradient {
    start: UnitPoint,
    end: UnitPoint,
    from: Color,
    to: Color,
}

impl LinearGradient {
    pub const fn new(start: UnitPoint, end: UnitPoint, from: Color, to: Color) -> Self {
        Self {
            start,
            end,
            from,
            to,
        }
    }

    pub fn start(self) -> UnitPoint {
        self.start
    }

    pub fn end(self) -> UnitPoint {
        self.end
    }

    pub fn from(self) -> Color {
        self.from
    }

    pub fn to(self) -> Color {
        self.to
    }

    #[cfg(test)]
    pub fn dimmed(self, factor: f32) -> Self {
        Self {
            start: self.start,
            end: self.end,
            from: self.from.dimmed(factor),
            to: self.to.dimmed(factor),
        }
    }

    pub fn is_visible(self) -> bool {
        self.from.a > 0.0 || self.to.a > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitPoint {
    x: f32,
    y: f32,
}

impl UnitPoint {
    pub const TOP_LEFT: Self = Self { x: 0.0, y: 0.0 };
    pub const BOTTOM_RIGHT: Self = Self { x: 1.0, y: 1.0 };

    #[cfg(test)]
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
        }
    }

    pub fn x(self) -> f32 {
        self.x
    }

    pub fn y(self) -> f32 {
        self.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    #[cfg(test)]
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    #[cfg(test)]
    pub const RED: Self = Self {
        r: 1.0,
        b: 0.0,
        g: 0.0,
        a: 1.0,
    };

    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    #[cfg(test)]
    pub fn dimmed(self, factor: f32) -> Self {
        let factor = factor.max(0.0);

        Self {
            r: self.r * factor,
            g: self.g * factor,
            b: self.b * factor,
            a: self.a,
        }
    }
}

pub(crate) fn group_from_items(items: &[Item], opacity: f32, grid: Grid) -> Option<Group> {
    let opacity = opacity.clamp(0.0, 1.0);
    if opacity <= 0.0 {
        return None;
    }

    let bounds = group_bounds(items, grid)?;
    let items = translate_items_for_group(items, bounds.origin, grid);
    if items.is_empty() {
        return None;
    }

    Some(Group {
        bounds,
        opacity,
        items,
    })
}

pub(crate) fn shadow_visual_bounds(shadow: Shadow, grid: Grid) -> Rect {
    item_bounds(&Item::Shadow(shadow), grid).expect("a shadow always owns visual bounds")
}

pub(crate) fn union_visual_bounds(a: Rect, b: Rect) -> Rect {
    union_rect(a, b)
}

impl Scene {
    pub(crate) fn translated_from_origin(mut self, origin: point::Logical, grid: Grid) -> Self {
        self.items = translate_items_for_group(&self.items, origin, grid);
        self
    }
}

fn group_bounds(items: &[Item], grid: Grid) -> Option<Rect> {
    items
        .iter()
        .filter_map(|item| item_bounds(item, grid))
        .reduce(union_rect)
        .map(|rect| grid.snap_rect(rect))
}

fn item_bounds(item: &Item, grid: Grid) -> Option<Rect> {
    match item {
        Item::Quad(quad) => Some(quad.bounds()),
        Item::Rule(rule) => Some(rule.rect),
        Item::Text(text) => Some(text.rect),
        Item::TextViewport(text) => Some(text.rect),
        Item::Icon(icon) => Some(icon.rect),
        Item::Shadow(shadow) => {
            let spread = shadow.spread.max(0.0);
            let blur = shadow.blur.max(0.0) + grid.logical_pixel();
            Some(expand_rect(
                offset_rect(expand_rect(shadow.rect, spread), shadow.offset),
                blur,
            ))
        }
        Item::Outline(outline) => Some(expand_rect(
            outline.rect,
            outline.offset.max(0.0) + outline.width.max(0.0) + grid.logical_pixel(),
        )),
        Item::Pane(pane) => Some(expand_rect(pane.rect, pane_outset(pane, grid))),
        Item::Clip(_) | Item::PopClip => None,
        Item::Group(group) => Some(group.bounds),
    }
}

fn pane_outset(pane: &Pane, grid: Grid) -> f32 {
    match &pane.material {
        Material::Solid(_) => 0.0,
        Material::Glass(glass) => glass
            .backdrop_layers
            .iter()
            .map(|layer| backdrop_layer_outset(*layer, grid))
            .fold(0.0, f32::max),
    }
}

fn backdrop_layer_outset(layer: BackdropLayer, grid: Grid) -> f32 {
    let scale_factor = grid.scale_factor();
    match layer {
        BackdropLayer::Blur(blur) => {
            filter_blur_kernel_radius_px(blur.sigma, scale_factor) / scale_factor
        }
        // Refraction displaces the source sample but still writes inside
        // the shaped filter rect. Grow this when a future op owns pixels
        // outside its rect instead of only sampling from outside it.
        BackdropLayer::Refraction(_) | BackdropLayer::Luminosity(_) => 0.0,
    }
}

fn translate_items_for_group(items: &[Item], origin: point::Logical, grid: Grid) -> Vec<Item> {
    items
        .iter()
        .map(|item| translate_item_for_group(item, origin, grid))
        .collect()
}

fn translate_item_for_group(item: &Item, origin: point::Logical, grid: Grid) -> Item {
    let dx = -origin.x();
    let dy = -origin.y();

    match item {
        Item::Quad(quad) => Item::Quad(quad.translated_for_group(origin, grid)),
        Item::Rule(rule) => {
            let mut rule = *rule;
            rule.rect = translate_rect(rule.rect, dx, dy);
            Item::Rule(rule)
        }
        Item::Text(text) => {
            let mut text = text.clone();
            text.rect = translate_rect(text.rect, dx, dy);
            Item::Text(text)
        }
        Item::TextViewport(text) => {
            let mut text = text.clone();
            text.rect = translate_rect(text.rect, dx, dy);
            text.surfaces = text
                .surfaces
                .into_iter()
                .map(|mut surface| {
                    surface.rect = translate_rect(surface.rect, dx, dy);
                    surface
                })
                .collect();
            Item::TextViewport(text)
        }
        Item::Icon(icon) => {
            let mut icon = *icon;
            icon.rect = translate_rect(icon.rect, dx, dy);
            Item::Icon(icon)
        }
        Item::Shadow(shadow) => {
            let mut shadow = *shadow;
            shadow.rect = translate_rect(shadow.rect, dx, dy);
            Item::Shadow(shadow)
        }
        Item::Outline(outline) => {
            let mut outline = *outline;
            outline.rect = translate_rect(outline.rect, dx, dy);
            Item::Outline(outline)
        }
        Item::Pane(pane) => Item::Pane(pane.clone().translated_for_group(origin)),
        Item::Clip(clip) => {
            let mut clip = *clip;
            clip.rect = translate_rect(clip.rect, dx, dy);
            Item::Clip(clip)
        }
        Item::PopClip => Item::PopClip,
        Item::Group(group) => {
            let mut group = group.clone();
            group.bounds = translate_rect(group.bounds, dx, dy);
            group.items = translate_items_for_group(&group.items, origin, grid);
            Item::Group(group)
        }
    }
}

fn union_rect(a: Rect, b: Rect) -> Rect {
    let left = a.origin.x().min(b.origin.x());
    let top = a.origin.y().min(b.origin.y());
    let right = rect_right(a).max(rect_right(b));
    let bottom = rect_bottom(a).max(rect_bottom(b));

    Rect::new(
        point::logical(left, top),
        area::logical((right - left).max(0.0), (bottom - top).max(0.0)),
    )
}

fn expand_rect(rect: Rect, amount: f32) -> Rect {
    let amount = amount.max(0.0);
    Rect::rounded(
        point::logical(rect.origin.x() - amount, rect.origin.y() - amount),
        area::logical(
            rect.area.width() + amount * 2.0,
            rect.area.height() + amount * 2.0,
        ),
        rect.rounding,
    )
}

fn offset_rect(rect: Rect, offset: point::Logical) -> Rect {
    translate_rect(rect, offset.x(), offset.y())
}

fn translate_rect(rect: Rect, dx: f32, dy: f32) -> Rect {
    Rect::rounded(
        point::logical(rect.origin.x() + dx, rect.origin.y() + dy),
        rect.area,
        rect.rounding,
    )
}

fn rect_right(rect: Rect) -> f32 {
    rect.origin.x() + rect.area.width()
}

fn rect_bottom(rect: Rect) -> f32 {
    rect.origin.y() + rect.area.height()
}

#[cfg(test)]
mod tests {
    use crate::icon;
    use crate::paint;

    use super::*;

    fn solid_quad(x: f32) -> Quad {
        Quad {
            rect: Rect::new(point::logical(x, 0.0), area::logical(10.0, 10.0)),
            rasterization: Rasterization::default(),
            transform: Transform::identity(),
            style: Style {
                fill: Some(Fill::Brush(Brush::solid(Color::RED))),
                stroke: None,
                tint: None,
            },
        }
    }

    #[test]
    fn new_scene_is_empty() {
        let scene = Scene::new();

        assert_eq!(scene.clear_color(), None);
        assert!(scene.items().is_empty());
    }

    #[test]
    fn clear_color_is_stored() {
        let mut scene = Scene::new();

        scene.clear(Color::BLACK);

        assert_eq!(scene.clear_color(), Some(Color::BLACK));
        assert!(scene.items().is_empty());
    }

    #[test]
    fn pushed_items_preserve_order() {
        let mut scene = Scene::new();
        let first = solid_quad(1.0);
        let text = Text {
            rect: Rect::new(point::logical(1.5, 0.0), area::logical(10.0, 10.0)),
            document: text::document::Document::plain("Label"),
            wrap: TextWrap::WordOrGlyph,
            vertical_align: TextVerticalAlign::Center,
            overflow: text::Overflow::Clip,
        };
        let icon = Icon {
            rect: Rect::new(point::logical(1.6, 0.0), area::logical(10.0, 10.0)),
            icon: icon::Icon::phosphor(icon::Id::new("check")),
            color: Color::BLACK,
            size: 16.0,
        };
        let shadow = Shadow {
            rect: Rect::new(point::logical(1.7, 0.0), area::logical(10.0, 10.0)),
            brush: Brush::solid(Color::rgba(0.0, 0.0, 0.0, 0.35)),
            blur: 16.0,
            spread: 1.0,
            offset: point::logical(0.0, 4.0),
        };
        let clip = Clip {
            rect: Rect::new(point::logical(1.73, 0.0), area::logical(10.0, 10.0)),
        };
        let outline = Outline {
            rect: Rect::new(point::logical(1.75, 0.0), area::logical(10.0, 10.0)),
            brush: Brush::solid(Color::BLACK),
            width: 2.0,
            offset: 1.0,
        };
        let second = Quad {
            rect: Rect::rounded(
                point::logical(2.0, 0.0),
                area::logical(10.0, 10.0),
                paint::Rounding::none(),
            ),
            ..solid_quad(2.0)
        };

        scene.push_quad(first);
        scene.push_icon(icon);
        scene.push_text(text.clone());
        scene.push_shadow(shadow);
        scene.push_clip(clip);
        scene.push_outline(outline);
        scene.pop_clip();
        scene.push_quad(second);

        assert_eq!(
            scene.items(),
            &[
                Item::Quad(first),
                Item::Icon(icon),
                Item::Text(text),
                Item::Shadow(shadow),
                Item::Clip(clip),
                Item::Outline(outline),
                Item::PopClip,
                Item::Quad(second)
            ]
        );
    }

    #[test]
    fn shadow_item_preserves_shape_and_cutout_data() {
        let mut scene = Scene::new();
        let shadow = Shadow {
            rect: Rect::rounded(
                point::logical(0.0, 0.0),
                area::logical(20.0, 10.0),
                paint::Rounding::relative(1.0),
            ),
            brush: Brush::solid(Color::rgba(0.0, 0.0, 0.0, 0.3)),
            blur: 18.0,
            spread: 1.0,
            offset: point::logical(0.0, 6.0),
        };

        scene.push_shadow(shadow);

        assert_eq!(scene.items(), &[Item::Shadow(shadow)]);
    }

    #[test]
    fn filter_ops_clamp_parameters() {
        assert_eq!(FilterOp::blur(2.0), FilterOp::Blur { amount: 1.0 });
    }

    #[test]
    fn transform_scales_rect_about_origin() {
        let rect = Rect::new(point::logical(10.0, 20.0), area::logical(40.0, 10.0));
        let transform = Transform::scale_y_about(point::logical(30.0, 25.0), 1.5);

        assert_eq!(
            transform.transformed_rect(rect),
            Rect::new(point::logical(10.0, 17.5), area::logical(40.0, 15.0))
        );
    }

    #[test]
    fn moving_scale_motion_resolves_to_moving_subpixel_geometry_at_fractional_scale() {
        let grid = Grid::new(1.25);
        let rect = Rect::new(point::logical(10.0, 20.0), area::logical(40.0, 4.0));
        let transform = Transform::scale_y_about(point::logical(30.0, 22.0), 1.5)
            .with_motion(Motion::Moving)
            .with_scale_motion(1.0, 1.0, 1.0, 1.5, 0.5);

        let (resolved, resolved_transform) = transform.resolve_rect(rect, grid);

        assert_eq!(resolved_transform.motion, Motion::Moving);
        assert!(resolved_transform.is_identity());
        assert_eq!(resolved_transform.scale_motion, None);
        assert!(!grid.rect_is_aligned(resolved));
    }

    #[test]
    fn quad_constructor_snaps_resting_identity_rects_to_grid() {
        let grid = Grid::new(1.25);
        let quad = Quad::resolved_for_grid(
            Rect::new(point::logical(10.0, 20.0), area::logical(33.0, 11.0)),
            Style {
                fill: Some(Fill::Brush(Brush::solid(Color::RED))),
                stroke: None,
                tint: None,
            },
            Rasterization::default(),
            Transform::identity(),
            grid,
        );

        assert!(quad.transform().is_identity());
        assert!(grid.rect_is_aligned(quad.rect()));
    }

    #[test]
    fn clip_commands_preserve_order_and_shape() {
        let mut scene = Scene::new();
        let clip = Clip {
            rect: Rect::rounded(
                point::logical(0.0, 0.0),
                area::logical(20.0, 10.0),
                paint::Rounding::relative(0.5),
            ),
        };
        let quad = solid_quad(1.0);

        scene.push_clip(clip);
        scene.push_quad(quad);
        scene.pop_clip();

        assert_eq!(
            scene.items(),
            &[Item::Clip(clip), Item::Quad(quad), Item::PopClip]
        );
    }

    #[test]
    fn group_bounds_include_shadow_blur_and_spread() {
        let shadow = Shadow {
            rect: Rect::new(point::logical(10.0, 20.0), area::logical(40.0, 30.0)),
            brush: Brush::solid(Color::BLACK),
            blur: 8.0,
            spread: 2.0,
            offset: point::logical(0.0, 4.0),
        };
        let group = group_from_items(&[Item::Shadow(shadow)], 0.5, Grid::new(1.0))
            .expect("shadow should produce group bounds");

        assert_eq!(
            group.bounds,
            Rect::new(point::logical(-1.0, 13.0), area::logical(62.0, 52.0))
        );
    }

    #[test]
    fn group_translation_preserves_pane_source_rect() {
        let pane = Pane::new(
            Rect::new(point::logical(20.0, 30.0), area::logical(50.0, 40.0)),
            Material::Glass(Glass {
                fallback: Brush::solid(Color::BLACK),
                base: GlassBase::FrameworkBackdrop,
                backdrop_layers: vec![BackdropLayer::Blur(BackdropBlur {
                    sigma: 10.0,
                    edge_mode: BackdropEdgeMode::Mirror,
                })],
                surface_layers: vec![SurfaceLayer::Noise(Noise { opacity: 0.1 })],
            }),
        );
        let group = group_from_items(&[Item::Pane(pane.clone())], 0.5, Grid::new(1.0))
            .expect("pane should produce group");
        let [Item::Pane(local)] = group.items.as_slice() else {
            panic!("expected translated pane");
        };

        assert_eq!(
            local.rect,
            Rect::new(point::logical(30.0, 30.0), area::logical(50.0, 40.0))
        );
        assert_eq!(local.source_rect, Some(pane.rect));
    }

    #[test]
    fn group_translation_preserves_moving_quad_motion_at_fractional_scale() {
        let grid = Grid::new(1.25);
        let rect = Rect::new(point::logical(10.0, 20.0), area::logical(40.0, 4.0));
        let transform = Transform::scale_y_about(point::logical(30.0, 22.0), 1.5)
            .with_motion(Motion::Moving)
            .with_scale_motion(1.0, 1.0, 1.0, 1.5, 0.5);
        let quad = Quad::resolved_for_grid(
            rect,
            Style {
                fill: Some(Fill::Brush(Brush::solid(Color::RED))),
                stroke: None,
                tint: None,
            },
            Rasterization::default(),
            transform,
            grid,
        );

        let group = group_from_items(&[Item::Quad(quad)], 0.5, grid)
            .expect("moving quad should produce a group");
        let [Item::Quad(local)] = group.items.as_slice() else {
            panic!("expected translated quad");
        };

        assert_eq!(local.transform().motion, Motion::Moving);
        assert_eq!(local.transform().scale_motion, None);
        assert!(!grid.rect_is_aligned(local.rect()));
    }

    #[test]
    fn group_bounds_include_pane_backdrop_blur_kernel_spread() {
        let pane = Pane::new(
            Rect::new(point::logical(20.0, 30.0), area::logical(50.0, 40.0)),
            Material::Glass(Glass {
                fallback: Brush::solid(Color::BLACK),
                base: GlassBase::FrameworkBackdrop,
                backdrop_layers: vec![BackdropLayer::Blur(BackdropBlur {
                    sigma: 44.55,
                    edge_mode: BackdropEdgeMode::Mirror,
                })],
                surface_layers: Vec::new(),
            }),
        );

        for scale in [1.0, 1.5] {
            let group = group_from_items(&[Item::Pane(pane.clone())], 0.5, Grid::new(scale))
                .expect("pane should produce group bounds");

            assert_eq!(
                group.bounds,
                Rect::new(point::logical(-114.0, -104.0), area::logical(318.0, 308.0)),
                "scale {scale} should reserve the pane blur kernel"
            );
        }
    }

    #[test]
    fn color_converts_into_solid_brush() {
        let brush: Brush = Color::RED.into();

        assert_eq!(brush, Brush::Solid(Color::RED));
    }

    #[test]
    fn linear_gradient_brush_preserves_rgba_stops() {
        let from = Color::rgba(0.1, 0.2, 0.3, 0.4);
        let to = Color::rgba(0.5, 0.6, 0.7, 0.8);
        let Brush::Gradient(Gradient::Linear(gradient)) = Brush::linear_gradient(from, to) else {
            panic!("expected linear gradient brush");
        };

        assert_eq!(gradient.start(), UnitPoint::TOP_LEFT);
        assert_eq!(gradient.end(), UnitPoint::BOTTOM_RIGHT);
        assert_eq!(gradient.from(), from);
        assert_eq!(gradient.to(), to);
    }

    #[test]
    fn unit_point_clamps_to_normalized_range() {
        let point = UnitPoint::new(-1.0, 2.0);

        assert_eq!(point.x(), 0.0);
        assert_eq!(point.y(), 1.0);
    }

    #[test]
    fn brush_dim_preserves_alpha_and_dims_gradient_stops() {
        let brush = Brush::linear_gradient(
            Color::rgba(1.0, 0.5, 0.25, 0.4),
            Color::rgba(0.5, 0.25, 0.125, 0.8),
        )
        .dimmed(0.5);
        let Brush::Gradient(Gradient::Linear(gradient)) = brush else {
            panic!("expected linear gradient brush");
        };

        assert_eq!(gradient.from(), Color::rgba(0.5, 0.25, 0.125, 0.4));
        assert_eq!(gradient.to(), Color::rgba(0.25, 0.125, 0.0625, 0.8));
    }

    #[test]
    fn empty_text_is_not_pushed() {
        let mut scene = Scene::new();

        scene.push_text(Text {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
            document: text::document::Document::plain(""),
            wrap: TextWrap::WordOrGlyph,
            vertical_align: TextVerticalAlign::Center,
            overflow: text::Overflow::Clip,
        });

        assert!(scene.items().is_empty());
    }
}
