use super::effects::liquid_depth_displacement;
use super::geometry::{
    blur_kernel_radius_px, blur_radius_px, blur_sigma_px, prepare_clip, prepare_filter,
    source_rect_for_prepared_destination,
};
use super::params::{
    noise_material_position_data, physical_rect_data, physical_source_rect_data, source_scale_data,
    source_step_data, with_texture_area as params_with_texture_area,
};
use super::*;

use crate::paint::Rect;
use crate::render::silhouette::edges;

#[test]
fn filter_shape_snaps_and_raster_bounds_expand_by_one_physical_pixel() {
    let rect = Rect::new(
        paint::point::logical(10.2, 20.3),
        paint::area::logical(40.4, 30.8),
    );
    let prepared = prepare_filter(rect, 2.0)
        .expect("filter should prepare")
        .with_blur_sigma(30.0, 2.0);

    assert_eq!(edges(prepared.shape_rect), (10.0, 20.5, 50.5, 51.0));
    assert_eq!(edges(prepared.raster_rect), (9.5, 20.0, 51.0, 51.5));
    assert_eq!(prepared.blur_amount, 0.0);
    assert_eq!(prepared.blur_sigma_px, 60.0);
    assert_eq!(prepared.blur_radius_px, 180.0);
}

#[test]
fn filter_preserves_rounded_shape_metadata_for_composite() {
    let rect = Rect::rounded(
        paint::point::logical(10.0, 20.0),
        paint::area::logical(80.0, 30.0),
        crate::paint::Rounding::relative(1.0),
    );
    let prepared = prepare_filter(rect, 1.0).expect("filter should prepare");
    let vertices = composite_vertices(paint::area::logical(100.0, 100.0), prepared);

    assert_eq!(prepared.rounding[0], 15.0);
    assert_eq!(vertices.len(), 6);
    assert_eq!(vertices[0].rect, [10.0, 20.0, 80.0, 30.0]);
    assert_eq!(vertices[0].rounding, [15.0, 15.0, 15.0, 15.0]);
}

#[test]
fn clip_preserves_rounded_shape_metadata_for_layer_composite() {
    let rect = Rect::rounded(
        paint::point::logical(8.0, 12.0),
        paint::area::logical(48.0, 20.0),
        crate::paint::Rounding::relative(1.0),
    );
    let prepared = prepare_clip(rect, 1.0).expect("clip should prepare");
    let vertices = composite_vertices(paint::area::logical(100.0, 100.0), prepared);

    assert_eq!(prepared.blur_amount, 0.0);
    assert_eq!(prepared.blur_sigma_px, 0.0);
    assert_eq!(prepared.blur_radius_px, 0.0);
    assert_eq!(vertices[0].rect, [8.0, 12.0, 48.0, 20.0]);
    assert_eq!(vertices[0].rounding, [10.0, 10.0, 10.0, 10.0]);
}

#[test]
fn layer_source_rect_tracks_snapped_destination_without_scaling() {
    let destination = Rect::new(
        paint::point::logical(10.2, 20.3),
        paint::area::logical(40.4, 30.8),
    );
    let source = Rect::new(
        paint::point::logical(4.0, 8.0),
        paint::area::logical(40.4, 30.8),
    );
    let prepared = prepare_clip(destination, 2.0).expect("clip should prepare");

    let source = source_rect_for_prepared_destination(destination, prepared, source);

    assert_eq!(edges(prepared.shape_rect), (10.0, 20.5, 50.5, 51.0));
    assert_edges_close(edges(source), (3.8, 8.2, 44.3, 38.7));
    assert_eq!(source.area, prepared.shape_rect.area);
}

#[test]
fn layer_source_rect_data_uses_source_texture_scale() {
    let destination = Rect::new(
        paint::point::logical(10.2, 20.3),
        paint::area::logical(40.4, 30.8),
    );
    let source = Rect::new(
        paint::point::logical(4.0, 8.0),
        paint::area::logical(40.4, 30.8),
    );
    let prepared = prepare_clip(destination, 2.0).expect("clip should prepare");
    let source = source_rect_for_prepared_destination(destination, prepared, source);

    let source_data = physical_source_rect_data(
        source,
        paint::area::logical(100.0, 80.0),
        paint::area::physical(200, 160),
        2.0,
        paint::LayerSampling::Filtered,
    );
    let source_size = source.area.to_physical(2.0);

    assert_eq!(source_data[0], 8.0);
    assert_eq!(source_data[1], 16.0);
    assert_eq!(source_data[2], source_size.width() as f32);
    assert_eq!(source_data[3], source_size.height() as f32);
}

#[test]
fn layer_source_rect_data_does_not_assume_destination_scale() {
    let source = Rect::new(
        paint::point::logical(4.0, 8.0),
        paint::area::logical(40.0, 30.0),
    );

    let source_data = physical_source_rect_data(
        source,
        paint::area::logical(80.0, 100.0),
        paint::area::physical(100, 150),
        2.0,
        paint::LayerSampling::Filtered,
    );

    assert_eq!(source_data, [5.0, 12.0, 50.0, 45.0]);
}

#[test]
fn pixel_aligned_source_rect_data_uses_target_scale() {
    let source = Rect::new(
        paint::point::logical(2.0, 4.0),
        paint::area::logical(801.2, 1047.2),
    );

    let source_data = physical_source_rect_data(
        source,
        paint::area::logical(805.2, 2138.4),
        paint::area::physical(1007, 2673),
        1.25,
        paint::LayerSampling::PixelAligned,
    );

    assert_eq!(source_data, [3.0, 5.0, 1002.0, 1309.0]);
}

#[test]
fn target_rect_data_is_physical_destination_space() {
    let target = Rect::new(
        paint::point::logical(12.0, 8.0),
        paint::area::logical(160.0, 80.0),
    );

    assert_eq!(physical_rect_data(target, 1.25), [15.0, 10.0, 200.0, 100.0]);
}

#[test]
fn alpha_mode_params_encode_shape_and_source_modes() {
    let prepared = prepare_filter(
        Rect::new(
            paint::point::logical(0.0, 0.0),
            paint::area::logical(160.0, 80.0),
        ),
        1.0,
    )
    .expect("filter should prepare");
    let input = |alpha_mode| ParamInput {
        target_scale_factor: 1.0,
        texture_area: paint::area::physical(160, 80),
        texture_logical_area: paint::area::logical(160.0, 80.0),
        prepared,
        source_rect: prepared.shape_rect,
        direction: [0.0, 0.0],
        effect: [1.0, 0.0, 0.0, 0.0],
        alpha_mode,
        sampling: paint::LayerSampling::Filtered,
    };

    let source = params_with_texture_area(input(AlphaMode::Source));
    let shape = params_with_texture_area(input(AlphaMode::Shape));

    assert_eq!(source.alpha_mode, [0.0, 0.0, 0.0, 0.0]);
    assert_eq!(shape.alpha_mode, [1.0, 0.0, 0.0, 0.0]);
}

#[test]
fn blur_intermediate_source_rect_is_local_after_first_pass() {
    let local_filter_rect = Rect::new(
        paint::point::logical(0.0, 0.0),
        paint::area::logical(160.0, 80.0),
    );
    let global_backdrop_rect = Rect::new(
        paint::point::logical(240.0, 120.0),
        paint::area::logical(160.0, 80.0),
    );
    let prepared = prepare_filter(local_filter_rect, 1.25).expect("filter should prepare");

    assert_eq!(
        physical_source_rect_data(
            global_backdrop_rect,
            paint::area::logical(800.0, 600.0),
            paint::area::physical(1000, 750),
            1.25,
            paint::LayerSampling::PixelAligned,
        ),
        [300.0, 150.0, 200.0, 100.0]
    );
    assert_eq!(
        physical_source_rect_data(
            prepared.shape_rect,
            paint::area::logical(160.0, 80.0),
            paint::area::physical(200, 100),
            1.25,
            paint::LayerSampling::PixelAligned,
        ),
        [0.0, 0.0, 200.0, 100.0]
    );
}

#[test]
fn first_backdrop_sample_can_use_global_texture_extent_for_local_target() {
    let prepared = prepare_filter(
        Rect::new(
            paint::point::logical(0.0, 0.0),
            paint::area::logical(160.0, 80.0),
        ),
        1.5,
    )
    .expect("filter should prepare");
    let params = params_with_texture_area(ParamInput {
        target_scale_factor: 1.5,
        texture_area: paint::area::physical(1200, 900),
        texture_logical_area: paint::area::logical(800.0, 600.0),
        prepared,
        source_rect: Rect::new(
            paint::point::logical(240.0, 120.0),
            paint::area::logical(160.0, 80.0),
        ),
        direction: [1.0, 0.0],
        effect: [1.0, 0.0, 0.0, 0.0],
        alpha_mode: AlphaMode::Shape,
        sampling: paint::LayerSampling::PixelAligned,
    });

    assert_eq!(params.texture_size, [1200.0, 900.0]);
    assert_eq!(params.source_rect, [360.0, 180.0, 240.0, 120.0]);
    assert_eq!(params.target_rect, [0.0, 0.0, 240.0, 120.0]);
}

#[test]
fn group_intermediate_params_use_target_local_texture_extent() {
    let prepared = prepare_filter(
        Rect::new(
            paint::point::logical(0.0, 0.0),
            paint::area::logical(160.0, 80.0),
        ),
        1.5,
    )
    .expect("filter should prepare");
    let params = params_with_texture_area(ParamInput {
        target_scale_factor: 1.5,
        texture_area: paint::area::physical(240, 120),
        texture_logical_area: paint::area::logical(160.0, 80.0),
        prepared,
        source_rect: prepared.shape_rect,
        direction: [0.0, 1.0],
        effect: [1.0, 0.0, 0.0, 0.0],
        alpha_mode: AlphaMode::Shape,
        sampling: paint::LayerSampling::Filtered,
    });

    assert_eq!(params.texture_size, [240.0, 120.0]);
    assert_eq!(params.source_rect, [0.0, 0.0, 240.0, 120.0]);
    assert_eq!(params.target_rect, [0.0, 0.0, 240.0, 120.0]);
}

#[test]
fn promoted_blur_first_pass_samples_global_and_writes_local() {
    let pane = paint::Pane::new(
        Rect::new(
            paint::point::logical(20.0, 30.0),
            paint::area::logical(50.0, 40.0),
        ),
        paint::Material::Glass(paint::Glass {
            fallback: paint::Brush::solid(paint::Color::BLACK),
            backdrop_layers: vec![paint::BackdropLayer::Blur(paint::BackdropBlur {
                sigma: 44.55,
                edge_mode: paint::BackdropEdgeMode::Mirror,
            })],
            surface_layers: Vec::new(),
        }),
    );
    let group = paint::group_from_items(&[paint::Item::Pane(pane)], 0.5, paint::Grid::new(1.5))
        .expect("pane should produce group");
    let [paint::Item::Pane(local)] = group.items.as_slice() else {
        panic!("expected translated pane");
    };
    let prepared = prepare_filter(local.rect, 1.5)
        .expect("filter should prepare")
        .with_blur_sigma(44.55, 1.5);
    let params = params_with_texture_area(ParamInput {
        target_scale_factor: 1.5,
        texture_area: paint::area::physical(1200, 900),
        texture_logical_area: paint::area::logical(800.0, 600.0),
        prepared,
        source_rect: local
            .source_rect
            .expect("group pane keeps global source rect"),
        direction: [1.0, 0.0],
        effect: [prepared.blur_sigma_px, 0.0, 0.0, 0.0],
        alpha_mode: AlphaMode::Source,
        sampling: paint::LayerSampling::PixelAligned,
    });

    assert_eq!(params.source_rect, [30.0, 45.0, 75.0, 60.0]);
    assert_eq!(params.target_rect, [201.0, 201.0, 75.0, 60.0]);
    assert!(rect_fits_in_area(prepared.raster_rect, group.bounds.area));
}

#[test]
fn high_sigma_blur_write_rect_fits_in_inflated_group_target() {
    let pane = paint::Pane::new(
        Rect::new(
            paint::point::logical(20.0, 30.0),
            paint::area::logical(50.0, 40.0),
        ),
        paint::Material::Glass(paint::Glass {
            fallback: paint::Brush::solid(paint::Color::BLACK),
            backdrop_layers: vec![paint::BackdropLayer::Blur(paint::BackdropBlur {
                sigma: 44.55,
                edge_mode: paint::BackdropEdgeMode::Mirror,
            })],
            surface_layers: Vec::new(),
        }),
    );

    for scale in [1.0, 1.5] {
        let group = paint::group_from_items(
            &[paint::Item::Pane(pane.clone())],
            0.5,
            paint::Grid::new(scale),
        )
        .expect("pane should produce group");
        let [paint::Item::Pane(local)] = group.items.as_slice() else {
            panic!("expected translated pane");
        };
        let prepared = prepare_filter(local.rect, scale)
            .expect("filter should prepare")
            .with_blur_sigma(44.55, scale);

        assert!(
            rect_fits_in_area(prepared.raster_rect, group.bounds.area),
            "scale {scale} should keep the blur write rect inside target-local scratch"
        );
    }
}

#[test]
fn blur_pass_labels_distinguish_horizontal_and_vertical_stages() {
    let source = filter_module_source();

    for label in [
        "Filter Backdrop Blur Horizontal Pass",
        "Filter Backdrop Blur Vertical Pass",
        "Filter Blur Horizontal Pass",
        "Filter Blur Vertical Pass",
    ] {
        assert!(source.contains(label), "missing blur pass label {label}");
    }
}

#[test]
fn composite_pass_logs_coverage_and_alpha_params() {
    let source = filter_module_source();
    let log = source
        .split("target: \"wgpu_l3::render::filter_params\"")
        .find(|entry| entry.contains("coverage_rect") && entry.contains("alpha_flags"))
        .expect("composite pass should log coverage and alpha params");

    assert!(log.contains("source_rect"));
    assert!(log.contains("target_rect"));
    assert!(log.contains("texture_size"));
    assert!(log.contains("target_area"));
}

#[test]
fn filter_op_composites_use_context_output() {
    let source = filter_module_source();
    let draw_body = source
        .split("pub(crate) fn draw")
        .nth(1)
        .expect("draw function should exist")
        .split("pub(crate) fn composite_layer")
        .next()
        .expect("draw body should precede layer composite");

    assert!(
        !draw_body.contains("output: pass.output"),
        "filter op composites must use the chain output, not raw draw-pass output"
    );
    assert!(draw_body.contains("output: chain.output()"));
    assert!(draw_body.contains("chain.local_intermediate"));
    assert!(draw_body.contains("mark_output_as_current"));
}

#[test]
fn noise_material_coordinates_are_panel_local_across_group_translation() {
    let scale = 1.5;
    let inline_rect = Rect::new(
        paint::point::logical(240.0, 120.0),
        paint::area::logical(160.0, 80.0),
    );
    let promoted_rect = Rect::new(
        paint::point::logical(0.0, 0.0),
        paint::area::logical(160.0, 80.0),
    );
    let inline_prepared = prepare_filter(inline_rect, scale).expect("filter should prepare");
    let promoted_prepared = prepare_filter(promoted_rect, scale).expect("filter should prepare");
    let inline_params = params_with_texture_area(ParamInput {
        target_scale_factor: scale,
        texture_area: paint::area::physical(1200, 900),
        texture_logical_area: paint::area::logical(800.0, 600.0),
        prepared: inline_prepared,
        source_rect: inline_prepared.shape_rect,
        direction: [0.0, 0.0],
        effect: [1.0, 0.0, 0.0, 0.0],
        alpha_mode: AlphaMode::Shape,
        sampling: paint::LayerSampling::PixelAligned,
    });
    let promoted_params = params_with_texture_area(ParamInput {
        target_scale_factor: scale,
        texture_area: paint::area::physical(1200, 900),
        texture_logical_area: paint::area::logical(800.0, 600.0),
        prepared: promoted_prepared,
        source_rect: inline_prepared.shape_rect,
        direction: [0.0, 0.0],
        effect: [1.0, 0.0, 0.0, 0.0],
        alpha_mode: AlphaMode::Shape,
        sampling: paint::LayerSampling::PixelAligned,
    });

    assert_eq!(
        noise_material_position_data([264.0, 138.0], inline_params),
        noise_material_position_data([24.0, 18.0], promoted_params)
    );
}

#[test]
fn noise_material_coordinates_ignore_source_rect() {
    let prepared = prepare_filter(
        Rect::new(
            paint::point::logical(0.0, 0.0),
            paint::area::logical(160.0, 80.0),
        ),
        1.25,
    )
    .expect("filter should prepare");
    let params_a = params_with_texture_area(ParamInput {
        target_scale_factor: 1.25,
        texture_area: paint::area::physical(1000, 750),
        texture_logical_area: paint::area::logical(800.0, 600.0),
        prepared,
        source_rect: prepared.shape_rect,
        direction: [0.0, 0.0],
        effect: [1.0, 0.0, 0.0, 0.0],
        alpha_mode: AlphaMode::Shape,
        sampling: paint::LayerSampling::PixelAligned,
    });
    let params_b = params_with_texture_area(ParamInput {
        target_scale_factor: 1.25,
        texture_area: paint::area::physical(1000, 750),
        texture_logical_area: paint::area::logical(800.0, 600.0),
        prepared,
        source_rect: Rect::new(
            paint::point::logical(300.0, 150.0),
            paint::area::logical(160.0, 80.0),
        ),
        direction: [0.0, 0.0],
        effect: [1.0, 0.0, 0.0, 0.0],
        alpha_mode: AlphaMode::Shape,
        sampling: paint::LayerSampling::PixelAligned,
    });

    assert_ne!(params_a.source_rect, params_b.source_rect);
    assert_eq!(
        noise_material_position_data([32.0, 24.0], params_a),
        noise_material_position_data([32.0, 24.0], params_b)
    );
}

#[test]
fn noise_material_coordinates_use_target_physical_scale() {
    let prepared = prepare_filter(
        Rect::new(
            paint::point::logical(0.0, 0.0),
            paint::area::logical(80.0, 40.0),
        ),
        1.25,
    )
    .expect("filter should prepare");
    let params = params_with_texture_area(ParamInput {
        target_scale_factor: 1.25,
        texture_area: paint::area::physical(100, 50),
        texture_logical_area: paint::area::logical(80.0, 40.0),
        prepared,
        source_rect: prepared.shape_rect,
        direction: [0.0, 0.0],
        effect: [1.0, 0.0, 0.0, 0.0],
        alpha_mode: AlphaMode::Shape,
        sampling: paint::LayerSampling::PixelAligned,
    });

    assert_eq!(
        noise_material_position_data([16.0, 8.0], params),
        [20.0, 10.0]
    );
}

#[test]
fn retained_layer_composite_uses_source_rect_step_by_axis() {
    let source_rect_data = [5.0, 12.0, 48.0, 33.0];

    assert_eq!(
        source_step_data(
            source_rect_data,
            paint::area::logical(32.0, 22.0),
            2.0,
            paint::LayerSampling::Filtered,
        ),
        [1.5, 1.5]
    );
}

#[test]
fn retained_layer_source_step_does_not_assume_whole_texture_scale() {
    let source_rect_data = [10.0, 20.0, 96.0, 72.0];

    assert_eq!(
        source_scale_data(
            paint::area::logical(200.0, 100.0),
            paint::area::physical(300, 300)
        ),
        [1.5, 3.0]
    );
    assert_eq!(
        source_step_data(
            source_rect_data,
            paint::area::logical(64.0, 48.0),
            2.0,
            paint::LayerSampling::Filtered,
        ),
        [1.5, 1.5]
    );
}

#[test]
fn pixel_aligned_layer_source_step_uses_target_scale_not_source_size() {
    let source_rect_data = [10.0, 20.0, 97.0, 73.0];

    assert_eq!(
        source_step_data(
            source_rect_data,
            paint::area::logical(64.0, 48.0),
            1.25,
            paint::LayerSampling::PixelAligned,
        ),
        [1.25, 1.25]
    );
}

#[test]
fn zero_size_filters_do_not_prepare() {
    let rect = Rect::new(
        paint::point::logical(10.0, 20.0),
        paint::area::logical(0.0, 30.0),
    );

    assert!(prepare_filter(rect, 1.0).is_none());
}

#[test]
fn normalized_blur_amount_maps_to_internal_physical_cap() {
    assert_eq!(blur_radius_px(-1.0, 1.0), 0.0);
    assert_eq!(blur_radius_px(0.5, 1.0), 128.0);
    assert_eq!(blur_radius_px(1.0, 1.0), 256.0);
    assert_eq!(blur_radius_px(1.0, 2.0), 256.0);
}

#[test]
fn blur_sigma_maps_to_dip_kernel_radius() {
    assert_eq!(blur_sigma_px(30.0, 1.0), 30.0);
    assert_eq!(blur_sigma_px(30.0, 2.0), 60.0);
    assert_eq!(blur_kernel_radius_px(-1.0, 1.0), 0.0);
    assert_eq!(blur_kernel_radius_px(30.0, 1.0), 90.0);
    assert_eq!(blur_kernel_radius_px(30.0, 2.0), 180.0);
}

#[test]
fn normalized_liquid_depth_maps_to_logical_cap() {
    assert_eq!(liquid_depth_displacement(-1.0), 0.0);
    assert_eq!(liquid_depth_displacement(0.5), 24.0);
    assert_eq!(liquid_depth_displacement(1.0), 48.0);
    assert_eq!(liquid_depth_displacement(2.0), 48.0);
}

#[test]
fn liquid_effect_clamps_and_preserves_shape_parameters() {
    assert_eq!(liquid_effect(0.5, 2.0, 18.0, 2.0), [24.0, 2.0, 18.0, 2.0]);
    assert_eq!(liquid_effect(2.0, -1.0, -4.0, 0.0), [48.0, 0.0, 0.0, 0.1]);
}

#[test]
fn zero_depth_liquid_is_identity() {
    assert!(liquid_is_identity(0.0));
    assert!(liquid_is_identity(-1.0));
    assert!(!liquid_is_identity(0.01));
}

fn assert_edges_close(left: (f32, f32, f32, f32), right: (f32, f32, f32, f32)) {
    const EPSILON: f32 = 0.0001;
    assert!((left.0 - right.0).abs() <= EPSILON, "{left:?} != {right:?}");
    assert!((left.1 - right.1).abs() <= EPSILON, "{left:?} != {right:?}");
    assert!((left.2 - right.2).abs() <= EPSILON, "{left:?} != {right:?}");
    assert!((left.3 - right.3).abs() <= EPSILON, "{left:?} != {right:?}");
}

fn rect_fits_in_area(rect: Rect, area: paint::area::Logical) -> bool {
    const EPSILON: f32 = 0.0001;
    let (left, top, right, bottom) = edges(rect);

    left >= -EPSILON
        && top >= -EPSILON
        && right <= area.width() + EPSILON
        && bottom <= area.height() + EPSILON
}

fn filter_module_source() -> String {
    [
        include_str!("../filter.rs"),
        include_str!("draw.rs"),
        include_str!("layer.rs"),
        include_str!("encode.rs"),
    ]
    .join("\n")
}
