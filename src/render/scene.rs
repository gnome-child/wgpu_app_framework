use crate::geometry::{area, point};
use crate::{paint, text};

use crate::{geometry, scene};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PopupProjection {
    visual_bounds: paint::Rect,
    panel_bounds: paint::Rect,
    scale_factor: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Scale(paint::Grid);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PhysicalRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rounding: [f32; 4],
}

pub(in crate::render) enum PreparedContent {
    Quad(paint::Quad),
    Rule(paint::Rule),
    Text(paint::Text),
    TextViewport(paint::TextViewport),
    Icon(paint::Icon),
    Shadow(paint::Shadow),
    Pane(paint::Pane),
    Outline(paint::Outline),
}

impl PreparedContent {
    pub(in crate::render) fn bounds(&self, scale_factor: f32) -> paint::Rect {
        let grid = paint::Grid::new(scale_factor);
        match self {
            Self::Quad(quad) => quad.transform().transformed_rect(quad.rect()),
            Self::Rule(rule) => rule.rect,
            Self::Text(text) => text.rect,
            Self::TextViewport(text) => text.rect,
            Self::Icon(icon) => icon.rect,
            Self::Shadow(shadow) => paint::shadow_visual_bounds(*shadow, grid),
            Self::Pane(pane) => paint::pane_effect_bounds(pane, grid),
            Self::Outline(outline) => paint::expand_rect(
                outline.rect,
                outline.offset.max(0.0) + outline.width.max(0.0) + grid.logical_pixel(),
            ),
        }
    }
}

impl PopupProjection {
    pub(crate) fn resolve(
        source: &scene::Scene,
        scale_factor: f32,
        include_visual_reach: bool,
    ) -> Self {
        let grid = paint::Grid::new(scale_factor);
        let panel_bounds = into_paint_rect_at_scale(geometry::Rect::from_size(source.size()), grid);
        let shadow = source.shadows().into_iter().next().copied();
        let visual_bounds = if include_visual_reach {
            shadow
                .iter()
                .map(|shadow| paint::shadow_visual_bounds(to_paint_shadow(shadow, grid), grid))
                .fold(panel_bounds, paint::union_visual_bounds)
        } else {
            panel_bounds
        };
        Self {
            visual_bounds,
            panel_bounds,
            scale_factor: grid.scale_factor(),
        }
    }

    fn panel_offset(self) -> point::Logical {
        point::logical(
            self.panel_bounds.origin.x() - self.visual_bounds.origin.x(),
            self.panel_bounds.origin.y() - self.visual_bounds.origin.y(),
        )
    }

    pub(crate) fn panel_offset_logical(self) -> geometry::Point {
        let offset = self.panel_offset();
        geometry::Point::new(offset.x().round() as i32, offset.y().round() as i32)
    }

    pub(crate) fn visual_bounds_at(self, panel: geometry::Rect) -> geometry::Rect {
        geometry::Rect::new(
            panel
                .x()
                .saturating_add(self.visual_bounds.origin.x().round() as i32),
            panel
                .y()
                .saturating_add(self.visual_bounds.origin.y().round() as i32),
            self.visual_bounds.area.width().round() as i32,
            self.visual_bounds.area.height().round() as i32,
        )
    }

    #[cfg(test)]
    fn panel_offset_physical(self) -> (i32, i32) {
        let offset = self.panel_offset();
        (
            (offset.x() * self.scale_factor).round() as i32,
            (offset.y() * self.scale_factor).round() as i32,
        )
    }

    pub(crate) fn visual_offset_physical(self) -> (i32, i32) {
        (
            (self.visual_bounds.origin.x() * self.scale_factor).round() as i32,
            (self.visual_bounds.origin.y() * self.scale_factor).round() as i32,
        )
    }

    pub(crate) fn logical_area(self) -> area::Logical {
        self.visual_bounds.area
    }
}

impl Scale {
    pub(crate) fn new(scale_factor: f32) -> Self {
        Self(paint::Grid::new(scale_factor))
    }

    pub(crate) fn factor(self) -> f32 {
        self.0.scale_factor()
    }
}

impl PhysicalRect {
    pub(crate) fn x(self) -> f32 {
        self.x
    }

    pub(crate) fn y(self) -> f32 {
        self.y
    }

    pub(crate) fn width(self) -> f32 {
        self.width
    }

    pub(crate) fn height(self) -> f32 {
        self.height
    }

    pub(crate) fn rounding(self) -> [f32; 4] {
        self.rounding
    }
}

pub(crate) fn physical_rounded_rect(
    rect: geometry::Rect,
    rounding: scene::Rounding,
    scale: Scale,
) -> PhysicalRect {
    let rect = into_paint_rounded_rect_at_scale(rect, rounding, scale.0);
    let factor = scale.factor();
    let rounding = rect
        .rounding
        .resolve(rect.area)
        .map(|radius| radius * factor);
    PhysicalRect {
        x: rect.origin.x() * factor,
        y: rect.origin.y() * factor,
        width: rect.area.width() * factor,
        height: rect.area.height() * factor,
        rounding,
    }
}

pub(in crate::render) fn prepare_content(
    content: &scene::Content,
    scale_factor: f32,
) -> PreparedContent {
    let grid = paint::Grid::new(scale_factor);
    match content {
        scene::Content::Quad(quad) => PreparedContent::Quad(to_paint_quad(quad, grid)),
        scene::Content::Rule(rule) => PreparedContent::Rule(to_paint_rule(rule, grid)),
        scene::Content::Text(text) => PreparedContent::Text(to_paint_text(text, grid)),
        scene::Content::TextViewport(text) => {
            PreparedContent::TextViewport(to_paint_text_viewport(text, grid))
        }
        scene::Content::Icon(icon) => PreparedContent::Icon(to_paint_icon(icon, grid)),
        scene::Content::Shadow(shadow) => PreparedContent::Shadow(to_paint_shadow(shadow, grid)),
        scene::Content::Pane(pane) => PreparedContent::Pane(to_paint_pane(pane, grid)),
        scene::Content::Outline(outline) => {
            PreparedContent::Outline(to_paint_outline(outline, grid))
        }
    }
}

pub(crate) fn to_paint_clip_value_at_scale(clip: scene::Clip, scale_factor: f32) -> paint::Clip {
    to_paint_clip_at_scale(&clip, paint::Grid::new(scale_factor))
}

pub(crate) fn to_paint_rect_value_at_scale(rect: geometry::Rect, scale_factor: f32) -> paint::Rect {
    into_paint_rect_at_scale(rect, paint::Grid::new(scale_factor))
}

fn to_paint_quad(quad: &scene::Quad, grid: paint::Grid) -> paint::Quad {
    to_paint_quad_with_transform(quad, quad.transform(), grid)
}

fn to_paint_quad_with_transform(
    quad: &scene::Quad,
    transform: scene::Transform,
    grid: paint::Grid,
) -> paint::Quad {
    let rect = into_paint_rounded_rect_at_scale(quad.rect(), quad.rounding(), grid);
    let transform = to_paint_transform(transform);

    paint::Quad::resolved_for_grid(
        rect,
        to_paint_style(quad.style()),
        to_paint_rasterization(quad.rasterization()),
        transform,
        grid,
    )
}

fn to_paint_rule(rule: &scene::Rule, grid: paint::Grid) -> paint::Rule {
    let rect = paint::Rect::new(
        point::logical(rule.rect().x() as f32, rule.rect().y() as f32),
        area::logical(rule.rect().width() as f32, rule.rect().height() as f32),
    );
    let axis = to_paint_axis(rule.axis());
    let rect = match axis {
        paint::Axis::Horizontal => grid.snap_horizontal_rule_rect(rect, rule.thickness_px()),
        paint::Axis::Vertical => grid.snap_vertical_rule_rect(rect, rule.thickness_px()),
    };

    paint::Rule {
        axis,
        rect,
        brush: paint::Brush::solid(super::color::paint_color(rule.color())),
        thickness_px: rule.thickness_px(),
    }
}

fn to_paint_text(text: &scene::Text, grid: paint::Grid) -> paint::Text {
    let mut block = text::document::Block::new(into_text_align(text.align()));
    block.push_run(text::document::Run::new(
        text.value().to_owned(),
        text::document::Style::default()
            .with_size(text.style().size())
            .with_weight(text.style().weight())
            .with_color(super::color::text_color(text.color())),
    ));

    paint::Text {
        rect: into_paint_rect_at_scale(text.rect(), grid),
        document: text::document::Document::from_block(block),
        wrap: into_paint_text_wrap(text.wrap()),
        vertical_align: paint::TextVerticalAlign::Center,
        overflow: text.overflow(),
    }
}

fn to_paint_text_viewport(text: &scene::TextViewport, grid: paint::Grid) -> paint::TextViewport {
    paint::TextViewport {
        rect: into_paint_rect_at_scale(text.rect(), grid),
        surfaces: text
            .surfaces()
            .iter()
            .map(|surface| to_paint_text_surface(surface, grid))
            .collect(),
    }
}

fn to_paint_text_surface(surface: &scene::TextSurface, grid: paint::Grid) -> paint::TextSurface {
    let (r, g, b, a) = surface.default_color().channels();

    paint::TextSurface {
        rect: into_paint_rect_at_scale(surface.rect(), grid),
        buffer: surface.buffer(),
        default_color: paint::Color::rgba(
            crate::color::srgb_unit_to_linear(r),
            crate::color::srgb_unit_to_linear(g),
            crate::color::srgb_unit_to_linear(b),
            a.clamp(0.0, 1.0),
        ),
    }
}

fn to_paint_icon(icon: &scene::Icon, grid: paint::Grid) -> paint::Icon {
    paint::Icon {
        rect: into_paint_rect_at_scale(icon.rect(), grid),
        icon: icon.icon(),
        color: super::color::paint_color(icon.color()),
        size: icon.size(),
    }
}

fn to_paint_shadow(shadow: &scene::Shadow, grid: paint::Grid) -> paint::Shadow {
    paint::Shadow {
        rect: into_paint_rounded_rect_at_scale(shadow.rect(), shadow.rounding(), grid),
        brush: paint::Brush::solid(super::color::paint_color(shadow.color())),
        blur: shadow.blur(),
        spread: shadow.spread(),
        offset: point::logical(shadow.offset().x(), shadow.offset().y()),
    }
}

fn to_paint_pane(pane: &scene::Pane, grid: paint::Grid) -> paint::Pane {
    paint::Pane::new(
        into_paint_rounded_rect_at_scale(pane.rect(), pane.rounding(), grid),
        to_paint_material(pane.material()),
    )
}

fn to_paint_material(material: &scene::Material) -> paint::Material {
    match material {
        scene::Material::Solid(brush) => paint::Material::Solid(to_paint_brush(*brush)),
        scene::Material::Glass(glass) => paint::Material::Glass(to_paint_glass(glass)),
    }
}

fn to_paint_glass(glass: &scene::Glass) -> paint::Glass {
    paint::Glass {
        fallback: to_paint_brush(glass.fallback()),
        base: match glass.base() {
            scene::GlassBase::FrameworkBackdrop => paint::GlassBase::FrameworkBackdrop,
            scene::GlassBase::Transparent => paint::GlassBase::Transparent,
            scene::GlassBase::Fallback => paint::GlassBase::Fallback,
        },
        backdrop_layers: glass
            .backdrop_layers()
            .iter()
            .copied()
            .map(to_paint_backdrop_layer)
            .collect(),
        surface_layers: glass
            .surface_layers()
            .iter()
            .copied()
            .map(to_paint_surface_layer)
            .collect(),
    }
}

fn to_paint_backdrop_layer(layer: scene::BackdropLayer) -> paint::BackdropLayer {
    match layer {
        scene::BackdropLayer::Blur(blur) => paint::BackdropLayer::Blur(paint::BackdropBlur {
            sigma: blur.sigma(),
            edge_mode: to_paint_backdrop_edge_mode(blur.edge_mode()),
        }),
        scene::BackdropLayer::Refraction(refraction) => {
            let refraction = refraction.clamped();
            paint::BackdropLayer::Refraction(paint::Refraction {
                displacement: refraction.displacement(),
                splay: refraction.splay(),
                feather: refraction.feather(),
                curve: refraction.curve(),
            })
        }
        scene::BackdropLayer::Luminosity(luminosity) => {
            paint::BackdropLayer::Luminosity(paint::Luminosity {
                color: super::color::paint_color(luminosity.color()),
                opacity: luminosity.opacity(),
            })
        }
    }
}

fn to_paint_surface_layer(layer: scene::SurfaceLayer) -> paint::SurfaceLayer {
    match layer {
        scene::SurfaceLayer::Tint { brush, opacity } => paint::SurfaceLayer::Tint {
            brush: to_paint_brush(brush),
            opacity,
        },
        scene::SurfaceLayer::Noise(noise) => paint::SurfaceLayer::Noise(paint::Noise {
            opacity: noise.opacity(),
        }),
    }
}

fn to_paint_backdrop_edge_mode(edge_mode: scene::BackdropEdgeMode) -> paint::BackdropEdgeMode {
    match edge_mode {
        scene::BackdropEdgeMode::Mirror => paint::BackdropEdgeMode::Mirror,
    }
}

#[cfg(test)]
fn to_paint_clip(clip: &scene::Clip) -> paint::Clip {
    to_paint_clip_at_scale(clip, paint::Grid::new(1.0))
}

fn to_paint_clip_at_scale(clip: &scene::Clip, grid: paint::Grid) -> paint::Clip {
    paint::Clip {
        rect: into_paint_rounded_rect_at_scale(clip.rect(), clip.rounding(), grid),
    }
}

fn to_paint_outline(outline: &scene::Outline, grid: paint::Grid) -> paint::Outline {
    paint::Outline {
        rect: into_paint_rounded_rect_at_scale(outline.rect(), outline.rounding(), grid),
        brush: paint::Brush::solid(super::color::paint_color(outline.color())),
        width: outline.width() * grid.logical_pixel(),
        offset: outline.offset(),
    }
}

fn into_paint_rect_at_scale(rect: geometry::Rect, grid: paint::Grid) -> paint::Rect {
    grid.snap_rect(paint::Rect::new(
        point::logical(rect.x() as f32, rect.y() as f32),
        area::logical(rect.width() as f32, rect.height() as f32),
    ))
}

#[cfg(test)]
fn into_paint_rounded_rect(rect: geometry::Rect, rounding: scene::Rounding) -> paint::Rect {
    into_paint_rounded_rect_at_scale(rect, rounding, paint::Grid::new(1.0))
}

fn into_paint_rounded_rect_at_scale(
    rect: geometry::Rect,
    rounding: scene::Rounding,
    grid: paint::Grid,
) -> paint::Rect {
    grid.snap_rect(paint::Rect::rounded(
        point::logical(rect.x() as f32, rect.y() as f32),
        area::logical(rect.width() as f32, rect.height() as f32),
        into_paint_rounding(rounding),
    ))
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
        edge_mode: to_paint_edge_mode(rasterization.edge_mode()),
    }
}

fn to_paint_transform(transform: scene::Transform) -> paint::Transform {
    paint::Transform {
        origin: point::logical(transform.origin_x(), transform.origin_y()),
        translate: point::logical(transform.translate_x(), transform.translate_y()),
        scale_x: transform.scale_x(),
        scale_y: transform.scale_y(),
        motion: to_paint_motion(transform.motion()),
        scale_motion: transform.scale_motion().map(to_paint_scale_motion),
    }
}

fn to_paint_motion(motion: scene::Motion) -> paint::Motion {
    match motion {
        scene::Motion::Moving => paint::Motion::Moving,
        scene::Motion::Resting => paint::Motion::Resting,
    }
}

fn to_paint_scale_motion(scale_motion: scene::ScaleMotion) -> paint::ScaleMotion {
    paint::ScaleMotion {
        from_x: scale_motion.from_x(),
        from_y: scale_motion.from_y(),
        to_x: scale_motion.to_x(),
        to_y: scale_motion.to_y(),
        progress: scale_motion.progress(),
    }
}

fn to_paint_axis(axis: scene::Axis) -> paint::Axis {
    match axis {
        scene::Axis::Horizontal => paint::Axis::Horizontal,
        scene::Axis::Vertical => paint::Axis::Vertical,
    }
}

fn to_paint_edge_mode(edge_mode: scene::EdgeMode) -> paint::EdgeMode {
    match edge_mode {
        scene::EdgeMode::Antialiased => paint::EdgeMode::Antialiased,
        scene::EdgeMode::Hard => paint::EdgeMode::Hard,
    }
}

fn into_paint_rounding(rounding: scene::Rounding) -> paint::Rounding {
    paint::Rounding::new(
        into_paint_radius(rounding.top_left()),
        into_paint_radius(rounding.top_right()),
        into_paint_radius(rounding.bottom_right()),
        into_paint_radius(rounding.bottom_left()),
    )
}

fn into_paint_radius(radius: scene::Radius) -> paint::Radius {
    match radius {
        scene::Radius::Relative(value) => paint::Radius::relative(value),
        scene::Radius::Fixed(value) => paint::Radius::fixed(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Command, Runtime, command, context::Context, control_gallery, geometry, glass_tuner, input,
        interaction, layout, response::Response, session, target::Target, theme::Theme, view,
        widget, window,
    };
    use std::time::{Duration, Instant};

    struct NativeSetLevel;

    impl Command for NativeSetLevel {
        type Args = f64;
        type Output = ();

        const NAME: &'static str = "native.set-level";
    }

    #[derive(Clone)]
    struct NativeSliderState {
        value: f64,
    }

    impl crate::State for NativeSliderState {}

    impl Target<NativeSetLevel> for NativeSliderState {
        fn state(&self, _: &f64, _: &Context) -> command::State {
            command::State::enabled()
        }

        fn invoke(&mut self, value: f64, _: &mut Context) -> Response<()> {
            self.value = value;
            Response::changed(())
        }
    }

    #[derive(Clone, Default)]
    struct NativeTextBoxState;

    impl crate::State for NativeTextBoxState {}

    fn prepared_content(source: &scene::Scene, scale: f32) -> Vec<PreparedContent> {
        let (commit, _) = scene::Commit::test_pair(source);
        commit
            .nodes()
            .iter()
            .flat_map(|node| node.content())
            .map(|content| prepare_content(content, scale))
            .collect()
    }

    #[test]
    fn text_box_surface_color_is_preserved_in_prepared_viewport() {
        let view = widget::view(|ui| {
            ui.text_box(widget::TextBox::new("query"));
        });
        let mut engine = layout::Engine::new();
        let layout = layout::Layout::compose(&view, geometry::Size::new(160, 40), &mut engine);
        let source_scene = scene::Scene::paint_with_theme(&layout, &Theme::dark());
        let prepared = prepared_content(&source_scene, 1.0);
        let viewport = prepared
            .iter()
            .find_map(|content| match content {
                PreparedContent::TextViewport(text) => Some(text),
                _ => None,
            })
            .expect("text box should prepare a retained text viewport");
        let surface = viewport
            .surfaces
            .first()
            .expect("text box viewport should contain a surface");
        let expected = super::super::color::paint_color(scene::Color::rgb(245, 245, 247));

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
    fn popup_material_layers_prepare_for_renderer() {
        let theme = Theme::dark();
        let view = view::View::new(
            view::Node::root()
                .child(view::Node::floating_panel("panel").child(view::Node::label("Row"))),
        );
        let mut engine = layout::Engine::new();
        let layout = layout::Layout::compose_with_theme(
            &view,
            geometry::Size::new(240, 160),
            &mut engine,
            &theme,
        );
        let source_scene = scene::Scene::paint_with_theme(&layout, &theme);
        let prepared = prepared_content(&source_scene, 1.0);
        let pane = prepared
            .iter()
            .find_map(|content| match content {
                PreparedContent::Pane(pane) => Some(pane),
                _ => None,
            })
            .expect("popup should prepare a retained pane");
        assert_eq!(pane.rect.rounding, paint::Rounding::fixed(10.0));
        let paint::Material::Glass(glass) = &pane.material else {
            panic!("popup pane should carry glass material");
        };
        assert_eq!(glass.base, paint::GlassBase::FrameworkBackdrop);
        assert!(glass.backdrop_layers.iter().any(|layer| {
            matches!(layer, paint::BackdropLayer::Blur(blur) if blur.sigma == 44.55)
        }));
        assert!(glass.backdrop_layers.iter().any(|layer| {
            matches!(
                layer,
                paint::BackdropLayer::Luminosity(luminosity) if luminosity.opacity == 0.92
            )
        }));
        assert!(glass.surface_layers.iter().any(|layer| {
            matches!(layer, paint::SurfaceLayer::Noise(noise) if noise.opacity == 0.022)
        }));
        assert!(glass.surface_layers.iter().any(|layer| {
            matches!(
                layer,
                paint::SurfaceLayer::Tint { brush, opacity }
                    if *brush == paint::Brush::solid(super::super::color::paint_color(scene::Color::rgb(28, 28, 30)))
                        && *opacity == 0.40
            )
        }));
    }

    #[test]
    fn popup_projection_reuses_shadow_reach_for_every_prepared_consumer() {
        for theme in [Theme::light(), Theme::dark()] {
            let view = view::View::new(
                view::Node::root()
                    .child(view::Node::floating_panel("panel").child(view::Node::label("Row"))),
            );
            let mut engine = layout::Engine::new();
            let layout = layout::Layout::compose_with_theme(
                &view,
                geometry::Size::new(240, 160),
                &mut engine,
                &theme,
            );
            let source = scene::Scene::paint_with_theme(&layout, &theme);
            assert_eq!(source.shadows().len(), 1);

            for scale in [1.0, 1.25, 1.5, 2.0] {
                let panel_only = PopupProjection::resolve(&source, scale, false);
                let expanded = PopupProjection::resolve(&source, scale, true);
                assert!(expanded.logical_area().width() > panel_only.logical_area().width());
                assert!(expanded.logical_area().height() > panel_only.logical_area().height());

                let (offset_x, offset_y) = expanded.panel_offset_physical();
                assert!(offset_x > 0 && offset_y > 0);
                let prepared = prepared_content(&source, scale);
                let pane = prepared
                    .iter()
                    .find_map(|content| match content {
                        PreparedContent::Pane(pane) => Some(pane),
                        _ => None,
                    })
                    .expect("floating panel should remain in retained preparation");
                let (visual_x, visual_y) = expanded.visual_offset_physical();
                assert_eq!(
                    (
                        (pane.rect.origin.x() * scale).round() as i32 - visual_x,
                        (pane.rect.origin.y() * scale).round() as i32 - visual_y,
                    ),
                    (offset_x, offset_y),
                );
                let outline = prepared
                    .iter()
                    .find_map(|content| match content {
                        PreparedContent::Outline(outline) => Some(outline),
                        _ => None,
                    })
                    .expect("floating panel should retain its one prepared border");
                assert!((outline.width * scale - 1.0).abs() <= f32::EPSILON);
            }
        }
    }

    #[test]
    fn popup_projection_survives_native_material_resolution_stripping_the_shadow() {
        let theme = Theme::dark();
        let view = view::View::new(
            view::Node::root()
                .child(view::Node::floating_panel("panel").child(view::Node::label("Row"))),
        );
        let mut engine = layout::Engine::new();
        let layout = layout::Layout::compose_with_theme(
            &view,
            geometry::Size::new(240, 160),
            &mut engine,
            &theme,
        );
        let authored = scene::Scene::paint_with_theme(&layout, &theme);
        let resolved =
            authored.resolve_material(scene::MaterialRenderer::NativePopup { opaque: false }, &[]);
        assert_eq!(authored.shadows().len(), 1);
        assert!(resolved.scene().shadows().is_empty());

        for scale in [1.0, 1.25, 1.5, 2.0] {
            let projection = PopupProjection::resolve(&authored, scale, true);
            let panel_offset = projection.panel_offset();
            assert!(panel_offset.x() > 0.0 && panel_offset.y() > 0.0);

            let prepared = prepared_content(resolved.scene(), scale);
            let panel = prepared
                .iter()
                .find_map(|content| match content {
                    PreparedContent::Quad(quad) => Some(quad.rect()),
                    _ => None,
                })
                .expect("resolved popup panel should remain in retained preparation");
            let visual_offset = projection.visual_offset_physical();
            let painted = (
                (panel.origin.x() * scale).round() as i32 - visual_offset.0,
                (panel.origin.y() * scale).round() as i32 - visual_offset.1,
            );
            let projected = projection.panel_offset_physical();
            assert!((painted.0 - projected.0).abs() <= 1);
            assert!((painted.1 - projected.1).abs() <= 1);
        }
    }

    #[test]
    fn refraction_constraints_resolve_before_paint_projection() {
        let projected = to_paint_backdrop_layer(scene::BackdropLayer::Refraction(
            scene::Refraction::new(99.0, -2.0, -3.0, 0.0),
        ));
        let paint::BackdropLayer::Refraction(refraction) = projected else {
            panic!("refraction should remain a refraction projection");
        };

        assert_eq!(refraction.displacement, 4.0);
        assert_eq!(refraction.splay, 0.0);
        assert_eq!(refraction.feather, 0.0);
        assert_eq!(refraction.curve, 0.1);
    }

    #[test]
    fn rounded_clip_converts_to_renderer_paint_clip() {
        let rect = geometry::Rect::new(4, 8, 80, 32);
        let clip = scene::Clip::new(rect).with_rounding(scene::Rounding::fixed(10.0));
        let paint = to_paint_clip(&clip);

        assert_eq!(
            paint.rect,
            into_paint_rounded_rect(rect, scene::Rounding::fixed(10.0))
        );
    }

    #[test]
    fn layout_to_paint_boundary_snaps_to_fractional_logical_device_grid() {
        let rect = geometry::Rect::new(10, 20, 33, 11);
        let grid = paint::Grid::new(1.25);
        let paint = into_paint_rounded_rect_at_scale(rect, scene::Rounding::none(), grid);

        // 10.0 * 1.25 = 12.5 device px, so the exact tie rounds toward zero.
        assert_approx_eq(paint.origin.x(), 9.6);
        assert_approx_eq(paint.origin.y(), 20.0);
        assert_approx_eq(paint.origin.x() + paint.area.width(), 43.2);
        assert_approx_eq(paint.origin.y() + paint.area.height(), 31.2);
        assert!(grid.rect_is_aligned(paint));
    }

    #[test]
    fn scale_change_resnaps_layout_boundary_truth() {
        let rect = geometry::Rect::new(10, 20, 33, 11);
        let at_1x =
            into_paint_rounded_rect_at_scale(rect, scene::Rounding::none(), paint::Grid::new(1.0));
        let at_125 =
            into_paint_rounded_rect_at_scale(rect, scene::Rounding::none(), paint::Grid::new(1.25));

        assert_ne!(at_1x, at_125);
        assert!(paint::Grid::new(1.0).rect_is_aligned(at_1x));
        assert!(paint::Grid::new(1.25).rect_is_aligned(at_125));
    }

    #[test]
    fn scroll_viewport_clip_survives_in_retained_order() {
        let view = widget::view(|ui| {
            ui.column(|ui| {
                ui.add(
                    widget::Scroll::new()
                        .id("scroll.native")
                        .height(view::Dimension::fixed(56))
                        .children(|ui| {
                            for index in 0..6 {
                                ui.label(format!("Row {index}"));
                            }
                        }),
                );
            });
        });
        let mut engine = layout::Engine::new();
        let layout = layout::Layout::compose(&view, geometry::Size::new(200, 120), &mut engine);
        let source_scene = scene::Scene::paint_with_theme(&layout, &Theme::dark());
        let (commit, _) = scene::Commit::test_pair(&source_scene);
        let order = commit
            .order()
            .expect("painted scene should retain draw order");

        assert!(
            order
                .iter()
                .any(|draw| matches!(draw, scene::Draw::PushClip { .. })),
            "scroll children should retain clip pushes"
        );
        assert!(
            order
                .iter()
                .any(|draw| matches!(draw, scene::Draw::PopClip)),
            "scroll children should retain clip pops"
        );
    }

    #[test]
    fn transform_converts_to_renderer_paint_space() {
        let transform = scene::Transform::scale_about(12.0, 18.0, 1.25, 1.5);
        let paint = to_paint_transform(transform);

        assert_eq!(paint.origin, point::logical(12.0, 18.0));
        assert_eq!(paint.translate, point::logical(0.0, 0.0));
        assert_eq!(paint.scale_x, 1.25);
        assert_eq!(paint.scale_y, 1.5);
        assert_eq!(paint.motion, paint::Motion::Resting);
    }

    #[test]
    fn settled_slider_hover_transform_prepares_snapped_geometry() {
        let mut app = Runtime::new(NativeSliderState { value: 5.0 })
            .commands(|commands| {
                commands.register::<NativeSetLevel>(crate::command::Spec::new("Set Level"));
            })
            .responders(|responders| {
                responders.app().target::<NativeSetLevel>();
            })
            .started(|cx| {
                cx.open_window(window::Options::new("Native Slider"));
            })
            .view(|state, _| {
                widget::view(|ui| {
                    ui.slider(
                        widget::Slider::new("Level", state.value, 0.0..=10.0)
                            .on_change::<NativeSetLevel>(),
                    );
                })
            });

        app.start();

        let window = app.session().windows()[0].id();
        let size = geometry::Size::new(240, 80);
        let theme = Theme::default();
        let start = Instant::now();
        let initial = app
            .show_scene_at(window, size, start)
            .expect("slider should render");
        let slider = initial
            .layout()
            .find_role(view::Role::Slider)
            .into_iter()
            .next()
            .expect("slider should be laid out");
        let track = layout::slider_track_rect(slider.rect(), slider.label_width(), &theme);
        let hover = geometry::Point::new(track.x() + track.width() / 2, track.y() + 1);

        app.pointer_move_at(window, size, hover)
            .expect("hovering the slider should be handled");
        let settled = app
            .show_scene_at(window, size, start + Duration::from_millis(180))
            .expect("settled slider should render");
        let track_color = super::super::color::paint_color(theme.slider().track);
        for scale in [1.0, 1.25, 1.5] {
            let grid = paint::Grid::new(scale);
            let expected_track =
                into_paint_rounded_rect_at_scale(track, scene::Rounding::relative(1.0), grid);
            let prepared = prepared_content(settled.scene(), scale);
            let track_quad = find_slider_track_quad_by_fill(
                &prepared,
                paint::Fill::Brush(paint::Brush::solid(track_color)),
                expected_track,
            )
            .expect("slider track should prepare a retained renderer quad");

            assert!(
                track_quad.transform().is_identity(),
                "settled track should bake to identity at scale {scale}"
            );
            assert!(
                grid.rect_is_aligned(track_quad.rect()),
                "settled track should be aligned at scale {scale}: {:?}",
                track_quad.rect()
            );
            assert_no_unaligned_resting_quads(&prepared, scale);
        }
    }

    #[test]
    fn mid_hover_slider_transform_remains_moving_at_fractional_scale() {
        let mut app = Runtime::new(NativeSliderState { value: 5.0 })
            .commands(|commands| {
                commands.register::<NativeSetLevel>(crate::command::Spec::new("Set Level"));
            })
            .responders(|responders| {
                responders.app().target::<NativeSetLevel>();
            })
            .started(|cx| {
                cx.open_window(window::Options::new("Native Slider"));
            })
            .view(|state, _| {
                widget::view(|ui| {
                    ui.slider(
                        widget::Slider::new("Level", state.value, 0.0..=10.0)
                            .on_change::<NativeSetLevel>(),
                    );
                })
            });

        app.start();

        let window = app.session().windows()[0].id();
        let size = geometry::Size::new(240, 80);
        let theme = Theme::default();
        let start = Instant::now();
        let initial = app
            .show_scene_at(window, size, start)
            .expect("slider should render");
        let slider = initial
            .layout()
            .find_role(view::Role::Slider)
            .into_iter()
            .next()
            .expect("slider should be laid out");
        let track = layout::slider_track_rect(slider.rect(), slider.label_width(), &theme);
        let hover = geometry::Point::new(track.x() + track.width() / 2, track.y() + 1);

        app.pointer_move_at(window, size, hover)
            .expect("hovering the slider should be handled");
        app.show_scene_at(window, size, start)
            .expect("hovered slider should start transition");
        let mid = app
            .show_scene_at(window, size, start + Duration::from_millis(90))
            .expect("mid-animation slider should render");
        let scale = 1.25;
        let grid = paint::Grid::new(scale);
        let expected_track =
            into_paint_rounded_rect_at_scale(track, scene::Rounding::relative(1.0), grid);
        let prepared = prepared_content(mid.scene(), scale);
        let track_quad = find_slider_track_quad_by_fill(
            &prepared,
            paint::Fill::Brush(paint::Brush::solid(super::super::color::paint_color(
                theme.slider().track,
            ))),
            expected_track,
        )
        .expect("slider track should prepare a retained renderer quad");

        assert_eq!(track_quad.transform().motion, paint::Motion::Moving);
        assert!(track_quad.transform().is_identity());
        assert_no_unaligned_resting_quads(&prepared, scale);
    }

    #[test]
    fn glass_tuner_promoted_panel_has_no_resting_unaligned_quads_at_fractional_scale() {
        let mut app = glass_tuner::app(glass_tuner::State::default());

        app.start();

        let window = app.session().windows()[0].id();
        let size = glass_tuner::window_size();
        let start = Instant::now();
        let initial = app
            .show_scene_at(window, size, start)
            .expect("glass tuner should render");
        let slider = initial
            .layout()
            .find_role(view::Role::Slider)
            .into_iter()
            .next()
            .expect("glass tuner should lay out acrylic sliders");
        let track =
            layout::slider_track_rect(slider.rect(), slider.label_width(), &Theme::default());
        let hover = geometry::Point::new(track.x() + track.width() / 2, track.y() + 1);

        app.pointer_move_at(window, size, hover)
            .expect("hovering a glass tuner slider should be handled");
        app.show_scene_at(window, size, start)
            .expect("hovered glass tuner should start transition");
        let mid = app
            .show_scene_at(window, size, start + Duration::from_millis(90))
            .expect("mid-animation glass tuner should render");
        let prepared = prepared_content(mid.scene(), 1.25);

        assert_no_unaligned_resting_quads(&prepared, 1.25);
    }

    fn find_slider_track_quad_by_fill(
        content: &[PreparedContent],
        fill: paint::Fill,
        expected_track: paint::Rect,
    ) -> Option<&paint::Quad> {
        content.iter().find_map(|content| match content {
            PreparedContent::Quad(quad)
                if quad.style().fill == Some(fill)
                    && approximately_equal(quad.rect().origin.x(), expected_track.origin.x())
                    && approximately_equal(
                        quad.rect().area.width(),
                        expected_track.area.width(),
                    )
                    && quad.rect().area.height() <= 10.0 =>
            {
                Some(quad)
            }
            _ => None,
        })
    }

    fn approximately_equal(left: f32, right: f32) -> bool {
        (left - right).abs() <= 0.001
    }

    fn assert_no_unaligned_resting_quads(content: &[PreparedContent], scale: f32) {
        let grid = paint::Grid::new(scale);
        for content in content {
            if let PreparedContent::Quad(quad) = content
                && quad.transform().motion == paint::Motion::Resting
            {
                assert!(
                    grid.rect_is_aligned(quad.rect()),
                    "resting quad should be aligned at scale {scale}: {:?}",
                    quad
                );
            }
        }
    }

    #[test]
    fn focus_outline_offset_and_rounding_prepare_for_renderer() {
        let theme = Theme::default();
        let mut app = control_gallery::app(control_gallery::State::default());

        app.start();

        let window = app.session().windows()[0].id();
        let size = geometry::Size::new(760, 520);
        app.show_scene(window, size)
            .expect("control gallery should render before focus");
        app.handle_input(
            window,
            input::Input::key_down(input::Key::Tab, input::Modifiers::default()),
        )
        .expect("tab should focus first control");

        let focused = app
            .show_scene(window, size)
            .expect("focused control should render");
        let frame = focused
            .layout()
            .frames()
            .iter()
            .find(|frame| frame.is_focused())
            .expect("focused frame should be present");
        let prepared = prepared_content(focused.scene(), 1.0);
        let expected_brush =
            paint::Brush::solid(super::super::color::paint_color(theme.focus().color));
        let outline = prepared
            .iter()
            .find_map(|content| match content {
                PreparedContent::Outline(outline) if outline.brush == expected_brush => {
                    Some(outline)
                }
                _ => None,
            })
            .expect("focus outline should prepare for the retained renderer");

        assert_eq!(
            outline.rect,
            into_paint_rounded_rect(frame.rect(), theme.control().rounding)
        );
        assert_eq!(outline.width, theme.focus().width as f32);
        assert_eq!(outline.offset, theme.focus().offset);
    }

    #[test]
    fn focused_text_box_outline_gaps_are_centered_at_fractional_scale() {
        let focus = session::Focus::text("field");
        let view_focus = focus.clone();
        let mut app = Runtime::new(NativeTextBoxState)
            .started(|cx| {
                cx.open_window(window::Options::new("Native Text Box Focus"));
            })
            .view(move |_, _| {
                widget::view(|ui| {
                    ui.text_box(widget::TextBox::new("").focus(view_focus.clone()));
                })
            });

        app.start();

        let window = app.session().windows()[0].id();
        let size = geometry::Size::new(240, 80);
        app.present(window)
            .expect("view should be presented before focus");
        app.handle_input(window, input::Input::focus(focus))
            .expect("text box focus should be handled");

        let focused = app
            .show_scene(window, size)
            .expect("focused text box should render");
        let prepared = prepared_content(focused.scene(), 1.5);
        let expected_brush = paint::Brush::solid(super::super::color::paint_color(
            Theme::default().focus().color,
        ));
        let outline = prepared
            .iter()
            .find_map(|content| match content {
                PreparedContent::Outline(outline) if outline.brush == expected_brush => {
                    Some(outline)
                }
                _ => None,
            })
            .expect("focused text box should prepare a focus outline");
        let grid = paint::Grid::new(1.5);
        let outset = grid.snap_outset(outline.rect, outline.offset, outline.width);
        let scale = 1.5;
        let left_gap = (outset.base_rect.origin.x() - outset.inner_rect.origin.x()) * scale;
        let top_gap = (outset.base_rect.origin.y() - outset.inner_rect.origin.y()) * scale;
        let right_gap = (outset.inner_rect.origin.x() + outset.inner_rect.area.width()
            - outset.base_rect.origin.x()
            - outset.base_rect.area.width())
            * scale;
        let bottom_gap = (outset.inner_rect.origin.y() + outset.inner_rect.area.height()
            - outset.base_rect.origin.y()
            - outset.base_rect.area.height())
            * scale;

        assert_approx_eq(left_gap, top_gap);
        assert_approx_eq(left_gap, right_gap);
        assert_approx_eq(left_gap, bottom_gap);
        assert!(grid.rect_is_aligned(outset.inner_rect));
        assert!(grid.rect_is_aligned(outset.outer_rect));
    }

    #[test]
    fn clipped_focus_scope_survives_renderer_projection_at_supported_scales() {
        let focus = session::Focus::text("native.clipped.focus").keyboard();
        let mut app = Runtime::new(NativeTextBoxState)
            .started(|cx| {
                cx.open_window(window::Options::new("Native Clipped Focus"));
            })
            .view(move |_, _| {
                widget::view(|ui| {
                    ui.add(
                        widget::Scroll::new()
                            .id("native.clipped.focus")
                            .height(view::Dimension::fixed(48))
                            .children(|ui| {
                                ui.text_box(widget::TextBox::new("Focused").focus(focus));
                                for index in 0..5 {
                                    ui.label(format!("Row {index}"));
                                }
                            }),
                    );
                })
            });
        app.start();

        let window = app.session().windows()[0].id();
        let size = geometry::Size::new(240, 100);
        assert!(app.focus(window, focus));
        let initial = app
            .show_scene(window, size)
            .expect("focused scroll should render");
        let viewport = initial
            .layout()
            .find_role(view::Role::Scroll)
            .into_iter()
            .next()
            .and_then(layout::Frame::viewport)
            .expect("scroll should expose a viewport")
            .rect();
        app.scroll_at(
            window,
            size,
            geometry::Point::new(
                viewport.x() + viewport.width() / 2,
                viewport.y() + viewport.height() / 2,
            ),
            interaction::ScrollDelta::vertical(16),
        )
        .expect("scroll should be handled");
        let focused = app
            .show_scene(window, size)
            .expect("partly clipped focus should render");
        let focus_brush = paint::Brush::solid(super::super::color::paint_color(
            Theme::default().focus().color,
        ));

        for scale in [1.0, 1.25, 1.5, 2.0] {
            let (commit, _) = scene::Commit::test_pair(focused.scene());
            let order = commit
                .order()
                .expect("focused scene should retain draw order");
            let outline_index = order
                .iter()
                .position(|draw| {
                    let scene::Draw::Content { node, index, .. } = draw else {
                        return false;
                    };
                    let Some(content) = commit
                        .nodes()
                        .iter()
                        .find(|candidate| candidate.id() == *node)
                        .and_then(|node| node.content().get(*index))
                    else {
                        return false;
                    };
                    matches!(
                        prepare_content(content, scale),
                        PreparedContent::Outline(outline) if outline.brush == focus_brush
                    )
                })
                .expect("retained draw order should retain the focus outline");
            let clip_index = order[..outline_index]
                .iter()
                .rposition(|draw| matches!(draw, scene::Draw::PushClip { .. }))
                .expect("retained draw order should retain the focus clip");
            let pop_index = order[outline_index + 1..]
                .iter()
                .position(|draw| matches!(draw, scene::Draw::PopClip))
                .map(|offset| outline_index + 1 + offset)
                .expect("retained draw order should close the focus clip");

            assert!(clip_index < outline_index, "scale {scale}");
            assert!(outline_index < pop_index, "scale {scale}");
        }
    }

    #[test]
    fn expanded_resized_table_rules_remain_aligned_at_supported_scales() {
        let mut state = control_gallery::State::default();
        state.expanded_rows = true;
        let mut app = control_gallery::app(state);
        app.start();

        let window = app.session().windows()[0].id();
        let initial = app
            .show_scene(window, geometry::Size::new(760, 700))
            .expect("expanded gallery table should render");
        let count = initial
            .layout()
            .table_tracks()
            .iter()
            .find(|track| {
                track
                    .column_identity()
                    .is_some_and(|cell| cell.column() == interaction::Id::new("count"))
            })
            .expect("Count/Enabled boundary");
        let start = geometry::Point::new(
            count.boundary(),
            count
                .divider_hit_rect()
                .expect("Count divider hit zone")
                .y()
                + 1,
        );
        let end = geometry::Point::new(start.x() + 36, start.y());
        drop(initial);
        app.pointer_down_at(window, geometry::Size::new(760, 700), start)
            .expect("Count divider should capture");
        app.pointer_move_at(window, geometry::Size::new(760, 700), end)
            .expect("Count divider should move");
        app.pointer_up_at(window, geometry::Size::new(760, 700), end)
            .expect("Count divider should release");
        let rendered = app
            .show_scene(window, geometry::Size::new(760, 700))
            .expect("resized expanded gallery table should render");
        assert!(rendered.layout().table_tracks().iter().any(|track| {
            track
                .column_identity()
                .is_some_and(|cell| cell.column() == interaction::Id::new("count"))
                && track.boundary() == end.x()
        }));

        for scale in [1.0, 1.25, 1.5, 2.0] {
            let prepared = prepared_content(rendered.scene(), scale);
            let rules = prepared
                .iter()
                .filter_map(|content| match content {
                    PreparedContent::Rule(rule) => Some(rule),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let grid = paint::Grid::new(scale);
            let (commit, _) = scene::Commit::test_pair(rendered.scene());
            let order = commit
                .order()
                .expect("table scene should retain draw order");

            assert!(!rules.is_empty(), "scale {scale}");
            assert!(rules.iter().all(|rule| rule.thickness_px == 1));
            assert!(
                rules.iter().all(|rule| grid.rect_is_aligned(rule.rect)),
                "scale {scale}"
            );
            assert_eq!(
                order
                    .iter()
                    .filter(|draw| matches!(draw, scene::Draw::PushClip { .. }))
                    .count(),
                order
                    .iter()
                    .filter(|draw| matches!(draw, scene::Draw::PopClip))
                    .count(),
                "scale {scale}"
            );
        }
    }

    fn assert_approx_eq(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.0001,
            "expected {actual} to be near {expected}"
        );
    }
}
