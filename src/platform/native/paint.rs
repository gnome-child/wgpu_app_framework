use crate::{paint, text};

use crate::{geometry, scene};

#[cfg(test)]
pub(in crate::platform::native) fn to_paint_scene(source: &scene::Scene) -> paint::Scene {
    to_paint_scene_at_scale(source, 1.0)
}

pub(in crate::platform::native) fn to_paint_scene_at_scale(
    source: &scene::Scene,
    scale_factor: f32,
) -> paint::Scene {
    let grid = paint::Grid::new(scale_factor);
    let mut scene = paint::Scene::new();
    scene.clear(super::color::paint_color(source.clear()));

    for primitive in source.primitives() {
        match primitive {
            scene::Primitive::Quad(quad) => scene.push_quad(to_paint_quad(quad, grid)),
            scene::Primitive::Rule(rule) => scene.push_rule(to_paint_rule(rule, grid)),
            scene::Primitive::Text(text) => scene.push_text(to_paint_text(text, grid)),
            scene::Primitive::TextViewport(text) => {
                scene.push_text_viewport(to_paint_text_viewport(text, grid));
            }
            scene::Primitive::Icon(icon) => scene.push_icon(to_paint_icon(icon, grid)),
            scene::Primitive::Shadow(shadow) => scene.push_shadow(to_paint_shadow(shadow, grid)),
            scene::Primitive::Pane(pane) => scene.push_pane(to_paint_pane(pane, grid)),
            scene::Primitive::Clip(clip) => scene.push_clip(to_paint_clip_at_scale(clip, grid)),
            scene::Primitive::PopClip => scene.pop_clip(),
            scene::Primitive::Outline(outline) => {
                scene.push_outline(to_paint_outline(outline, grid));
            }
            scene::Primitive::Group(group) => {
                if let Some(group) = to_paint_group(group, grid) {
                    scene.push_group(group);
                }
            }
        }
    }

    scene
}

fn push_paint_primitive(primitive: &scene::Primitive, scene: &mut paint::Scene, grid: paint::Grid) {
    match primitive {
        scene::Primitive::Quad(quad) => scene.push_quad(to_paint_quad(quad, grid)),
        scene::Primitive::Rule(rule) => scene.push_rule(to_paint_rule(rule, grid)),
        scene::Primitive::Text(text) => scene.push_text(to_paint_text(text, grid)),
        scene::Primitive::TextViewport(text) => {
            scene.push_text_viewport(to_paint_text_viewport(text, grid));
        }
        scene::Primitive::Icon(icon) => scene.push_icon(to_paint_icon(icon, grid)),
        scene::Primitive::Shadow(shadow) => scene.push_shadow(to_paint_shadow(shadow, grid)),
        scene::Primitive::Pane(pane) => scene.push_pane(to_paint_pane(pane, grid)),
        scene::Primitive::Clip(clip) => scene.push_clip(to_paint_clip_at_scale(clip, grid)),
        scene::Primitive::PopClip => scene.pop_clip(),
        scene::Primitive::Outline(outline) => scene.push_outline(to_paint_outline(outline, grid)),
        scene::Primitive::Group(group) => {
            if let Some(group) = to_paint_group(group, grid) {
                scene.push_group(group);
            }
        }
    }
}

fn to_paint_group(group: &scene::Group, grid: paint::Grid) -> Option<paint::Group> {
    let mut scene = paint::Scene::new();
    for primitive in group.primitives() {
        push_paint_primitive(primitive, &mut scene, grid);
    }

    paint::group_from_items(scene.items(), group.opacity(), grid)
}

fn to_paint_quad(quad: &scene::Quad, grid: paint::Grid) -> paint::Quad {
    let rect = into_paint_rounded_rect_at_scale(quad.rect(), quad.rounding(), grid);
    let transform = to_paint_transform(quad.transform());

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
        paint::point::logical(rule.rect().x() as f32, rule.rect().y() as f32),
        paint::area::logical(rule.rect().width() as f32, rule.rect().height() as f32),
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
        offset: paint::point::logical(shadow.offset().x(), shadow.offset().y()),
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
        width: outline.width(),
        offset: outline.offset(),
    }
}

fn into_paint_rect_at_scale(rect: geometry::Rect, grid: paint::Grid) -> paint::Rect {
    grid.snap_rect(paint::Rect::new(
        paint::point::logical(rect.x() as f32, rect.y() as f32),
        paint::area::logical(rect.width() as f32, rect.height() as f32),
    ))
}

#[cfg(test)]
fn into_paint_rounded_rect(rect: geometry::Rect, rounding: scene::Rounding) -> paint::Rect {
    into_paint_rounded_rect_at_scale(rect, rounding, paint::Grid::new(1.0))
}

pub(super) fn into_paint_rounded_rect_at_scale(
    rect: geometry::Rect,
    rounding: scene::Rounding,
    grid: paint::Grid,
) -> paint::Rect {
    grid.snap_rect(paint::Rect::rounded(
        paint::point::logical(rect.x() as f32, rect.y() as f32),
        paint::area::logical(rect.width() as f32, rect.height() as f32),
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
        origin: paint::point::logical(transform.origin_x(), transform.origin_y()),
        translate: paint::point::logical(transform.translate_x(), transform.translate_y()),
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

    #[test]
    fn text_box_surface_color_is_preserved_in_paint_viewport() {
        let view = widget::view(|ui| {
            ui.text_box(widget::TextBox::new("query"));
        });
        let mut engine = layout::Engine::new();
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
    fn popup_material_layers_convert_to_native_paint() {
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
        let paint = to_paint_scene(&source_scene);
        let pane = paint
            .items()
            .iter()
            .find_map(|item| match item {
                paint::Item::Pane(pane) => Some(pane),
                _ => None,
            })
            .expect("popup should convert to native pane");
        assert_eq!(pane.rect.rounding, paint::Rounding::fixed(10.0));
        let paint::Material::Glass(glass) = &pane.material else {
            panic!("popup pane should carry glass material");
        };
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
                        && *opacity == 0.88
            )
        }));
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
    fn rounded_clip_converts_to_native_paint_clip() {
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
    fn scroll_viewport_clip_converts_to_native_paint_clip_stack() {
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
        let paint = to_paint_scene(&source_scene);

        assert!(
            paint
                .items()
                .iter()
                .any(|item| matches!(item, paint::Item::Clip(_))),
            "scroll children should convert to native clip pushes"
        );
        assert!(
            paint
                .items()
                .iter()
                .any(|item| matches!(item, paint::Item::PopClip)),
            "scroll children should convert to native clip pops"
        );
    }

    #[test]
    fn transform_converts_to_native_paint_space() {
        let transform = scene::Transform::scale_about(12.0, 18.0, 1.25, 1.5);
        let paint = to_paint_transform(transform);

        assert_eq!(paint.origin, paint::point::logical(12.0, 18.0));
        assert_eq!(paint.translate, paint::point::logical(0.0, 0.0));
        assert_eq!(paint.scale_x, 1.25);
        assert_eq!(paint.scale_y, 1.5);
        assert_eq!(paint.motion, paint::Motion::Resting);
    }

    #[test]
    fn settled_slider_hover_transform_converts_to_snapped_geometry() {
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
            .render_scene_at(window, size, start)
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
            .render_scene_at(window, size, start + Duration::from_millis(180))
            .expect("settled slider should render");
        let track_color = super::super::color::paint_color(theme.slider().track);
        for scale in [1.0, 1.25, 1.5] {
            let grid = paint::Grid::new(scale);
            let expected_track =
                into_paint_rounded_rect_at_scale(track, scene::Rounding::relative(1.0), grid);
            let paint = to_paint_scene_at_scale(settled.scene(), scale);
            let track_quad = find_slider_track_quad_by_fill(
                paint.items(),
                paint::Fill::Brush(paint::Brush::solid(track_color)),
                expected_track,
            )
            .expect("slider track should convert to a native paint quad");

            assert!(
                track_quad.transform().is_identity(),
                "settled track should bake to identity at scale {scale}"
            );
            assert!(
                grid.rect_is_aligned(track_quad.rect()),
                "settled track should be aligned at scale {scale}: {:?}",
                track_quad.rect()
            );
            assert_no_unaligned_resting_quads(&paint, scale);
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
            .render_scene_at(window, size, start)
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
        app.render_scene_at(window, size, start)
            .expect("hovered slider should start transition");
        let mid = app
            .render_scene_at(window, size, start + Duration::from_millis(90))
            .expect("mid-animation slider should render");
        let scale = 1.25;
        let grid = paint::Grid::new(scale);
        let expected_track =
            into_paint_rounded_rect_at_scale(track, scene::Rounding::relative(1.0), grid);
        let paint = to_paint_scene_at_scale(mid.scene(), scale);
        let track_quad = find_slider_track_quad_by_fill(
            paint.items(),
            paint::Fill::Brush(paint::Brush::solid(super::super::color::paint_color(
                theme.slider().track,
            ))),
            expected_track,
        )
        .expect("slider track should convert to a native paint quad");

        assert_eq!(track_quad.transform().motion, paint::Motion::Moving);
        assert!(track_quad.transform().is_identity());
        assert_no_unaligned_resting_quads(&paint, scale);
    }

    #[test]
    fn glass_tuner_promoted_panel_has_no_resting_unaligned_quads_at_fractional_scale() {
        let mut app = glass_tuner::app(glass_tuner::State::default());

        app.start();

        let window = app.session().windows()[0].id();
        let size = glass_tuner::window_size();
        let start = Instant::now();
        let initial = app
            .render_scene_at(window, size, start)
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
        app.render_scene_at(window, size, start)
            .expect("hovered glass tuner should start transition");
        let mid = app
            .render_scene_at(window, size, start + Duration::from_millis(90))
            .expect("mid-animation glass tuner should render");
        let paint = to_paint_scene_at_scale(mid.scene(), 1.25);

        assert_no_unaligned_resting_quads(&paint, 1.25);
    }

    fn find_slider_track_quad_by_fill(
        items: &[paint::Item],
        fill: paint::Fill,
        expected_track: paint::Rect,
    ) -> Option<&paint::Quad> {
        items.iter().find_map(|item| match item {
            paint::Item::Quad(quad)
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
            paint::Item::Group(group) => {
                find_slider_track_quad_by_fill(&group.items, fill, expected_track)
            }
            _ => None,
        })
    }

    fn approximately_equal(left: f32, right: f32) -> bool {
        (left - right).abs() <= 0.001
    }

    fn assert_no_unaligned_resting_quads(scene: &paint::Scene, scale: f32) {
        assert_no_unaligned_resting_quads_in_items(scene.items(), scale);
    }

    fn assert_no_unaligned_resting_quads_in_items(items: &[paint::Item], scale: f32) {
        let grid = paint::Grid::new(scale);
        for item in items {
            match item {
                paint::Item::Quad(quad) if quad.transform().motion == paint::Motion::Resting => {
                    assert!(
                        grid.rect_is_aligned(quad.rect()),
                        "resting quad should be aligned at scale {scale}: {:?}",
                        quad
                    );
                }
                paint::Item::Group(group) => {
                    assert_no_unaligned_resting_quads_in_items(&group.items, scale);
                }
                _ => {}
            }
        }
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
            .render_scene(window, size)
            .expect("focused text box should render");
        let paint = to_paint_scene_at_scale(focused.scene(), 1.5);
        let expected_brush = paint::Brush::solid(super::super::color::paint_color(
            Theme::default().focus().color,
        ));
        let outline = paint
            .items()
            .iter()
            .find_map(|item| match item {
                paint::Item::Outline(outline) if outline.brush == expected_brush => Some(outline),
                _ => None,
            })
            .expect("focused text box should convert a focus outline");
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
    fn clipped_focus_scope_survives_native_projection_at_supported_scales() {
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
            .render_scene(window, size)
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
            .render_scene(window, size)
            .expect("partly clipped focus should render");
        let focus_brush = paint::Brush::solid(super::super::color::paint_color(
            Theme::default().focus().color,
        ));

        for scale in [1.0, 1.25, 1.5, 2.0] {
            let projected = to_paint_scene_at_scale(focused.scene(), scale);
            let outline_index = projected
                .items()
                .iter()
                .position(|item| {
                    matches!(item, paint::Item::Outline(outline) if outline.brush == focus_brush)
                })
                .expect("native projection should retain the focus outline");
            let clip_index = projected.items()[..outline_index]
                .iter()
                .rposition(|item| matches!(item, paint::Item::Clip(_)))
                .expect("native projection should retain the focus clip");
            let pop_index = projected.items()[outline_index + 1..]
                .iter()
                .position(|item| matches!(item, paint::Item::PopClip))
                .map(|offset| outline_index + 1 + offset)
                .expect("native projection should close the focus clip");

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
            .render_scene(window, geometry::Size::new(760, 700))
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
            .render_scene(window, geometry::Size::new(760, 700))
            .expect("resized expanded gallery table should render");
        assert!(rendered.layout().table_tracks().iter().any(|track| {
            track
                .column_identity()
                .is_some_and(|cell| cell.column() == interaction::Id::new("count"))
                && track.boundary() == end.x()
        }));

        for scale in [1.0, 1.25, 1.5, 2.0] {
            let projected = to_paint_scene_at_scale(rendered.scene(), scale);
            let rules = projected
                .items()
                .iter()
                .filter_map(|item| match item {
                    paint::Item::Rule(rule) => Some(rule),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let grid = paint::Grid::new(scale);

            assert!(!rules.is_empty(), "scale {scale}");
            assert!(rules.iter().all(|rule| rule.thickness_px == 1));
            assert!(
                rules.iter().all(|rule| grid.rect_is_aligned(rule.rect)),
                "scale {scale}"
            );
            assert_eq!(
                projected
                    .items()
                    .iter()
                    .filter(|item| matches!(item, paint::Item::Clip(_)))
                    .count(),
                projected
                    .items()
                    .iter()
                    .filter(|item| matches!(item, paint::Item::PopClip))
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
