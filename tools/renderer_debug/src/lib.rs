use wgpu_l3::renderer_debug::Image;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tolerance {
    Exact,
    PerChannel(f32),
    Silhouette {
        channel: f32,
        differing_pixels: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Difference {
    pub differing_pixels: usize,
    pub maximum_channel_delta: f32,
}

pub fn compare(
    reference: &Image,
    candidate: &Image,
    tolerance: Tolerance,
) -> Result<Difference, String> {
    if reference.width() != candidate.width() || reference.height() != candidate.height() {
        return Err(format!(
            "image dimensions differ: reference={}x{}, candidate={}x{}",
            reference.width(),
            reference.height(),
            candidate.width(),
            candidate.height()
        ));
    }

    let mut difference = Difference {
        differing_pixels: 0,
        maximum_channel_delta: 0.0,
    };
    for (reference, candidate) in reference.pixels().iter().zip(candidate.pixels()) {
        let maximum = reference
            .iter()
            .zip(candidate)
            .map(|(reference, candidate)| (reference - candidate).abs())
            .fold(0.0_f32, f32::max);
        if maximum > 0.0 {
            difference.differing_pixels += 1;
            difference.maximum_channel_delta = difference.maximum_channel_delta.max(maximum);
        }
    }

    let accepted = match tolerance {
        Tolerance::Exact => difference.differing_pixels == 0,
        Tolerance::PerChannel(channel) => difference.maximum_channel_delta <= channel,
        Tolerance::Silhouette {
            channel,
            differing_pixels,
        } => {
            difference.maximum_channel_delta <= channel
                && difference.differing_pixels <= differing_pixels
        }
    };
    if accepted {
        Ok(difference)
    } else {
        Err(format!(
            "pixel comparison exceeded {tolerance:?}: {difference:?}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgpu_l3::renderer_debug::{Case, Harness, Work};

    const RETAINED_GLOBAL_RESOURCES: usize = 8;
    const ONE_SCROLL_PROPERTY_UPLOAD: usize = std::mem::size_of::<[f32; 4]>();

    fn image(pixels: Vec<[f32; 4]>) -> Image {
        Image::new(2, 1, pixels).expect("test image dimensions should match")
    }

    #[test]
    fn exact_comparison_accepts_identical_pixels() {
        let image = image(vec![[0.0; 4], [1.0; 4]]);

        assert_eq!(
            compare(&image, &image, Tolerance::Exact),
            Ok(Difference {
                differing_pixels: 0,
                maximum_channel_delta: 0.0,
            })
        );
    }

    #[test]
    fn bounded_comparison_reports_and_limits_channel_delta() {
        let reference = image(vec![[0.0; 4], [0.5; 4]]);
        let candidate = image(vec![[0.0; 4], [0.51; 4]]);

        let difference = compare(&reference, &candidate, Tolerance::PerChannel(0.011))
            .expect("delta should fit tolerance");
        assert_eq!(difference.differing_pixels, 1);
        assert!(difference.maximum_channel_delta > 0.009);
        assert!(compare(&reference, &candidate, Tolerance::PerChannel(0.009)).is_err());
    }

    #[test]
    fn silhouette_comparison_limits_changed_pixel_count() {
        let reference = image(vec![[0.0; 4], [0.5; 4]]);
        let candidate = image(vec![[0.01; 4], [0.51; 4]]);

        assert!(
            compare(
                &reference,
                &candidate,
                Tolerance::Silhouette {
                    channel: 0.011,
                    differing_pixels: 2,
                },
            )
            .is_ok()
        );
        assert!(
            compare(
                &reference,
                &candidate,
                Tolerance::Silhouette {
                    channel: 0.011,
                    differing_pixels: 1,
                },
            )
            .is_err()
        );
    }

    fn assert_zero_content_work(work: Work) {
        assert_eq!(work.scene_node_realization_rebuilds(), 0);
        assert_eq!(work.primitive_prepare_calls(), 0);
        assert_eq!(work.text_prepare_calls(), 0);
        assert_eq!(work.text_shape_calls(), 0);
        assert_eq!(work.content_upload_bytes(), 0);
        assert_eq!(work.gpu_resource_creations(), 0);
        assert_eq!(work.gpu_resource_replacements(), 0);
    }

    fn assert_no_scroll_offscreen(work: Work) {
        assert_eq!(work.scroll_layer_cache_hits(), 0);
        assert_eq!(work.scroll_layer_cache_misses(), 0);
    }

    #[test]
    #[ignore = "requires a locally available GPU adapter"]
    fn ordered_production_commit_is_pixel_exact() {
        let mut harness = pollster::block_on(Harness::new(1.25)).expect("GPU harness should open");
        let (legacy, retained, _) = harness
            .render_pair(Case::OrderedGroup)
            .expect("ordered commit should render through both paths");

        assert_eq!(
            compare(&legacy, &retained, Tolerance::Exact),
            Ok(Difference {
                differing_pixels: 0,
                maximum_channel_delta: 0.0,
            })
        );
    }

    #[test]
    #[ignore = "requires a locally available GPU adapter"]
    fn retained_gpu_lifecycle_and_partial_update_receipts_are_bounded() {
        let mut harness = pollster::block_on(Harness::new(1.0)).expect("GPU harness should open");
        for case in [
            Case::SolidQuad,
            Case::Text,
            Case::OrderedGroup,
            Case::TransparentPopup,
        ] {
            let receipt = harness
                .retention_receipt(case)
                .expect("retention receipt should render");
            assert!(receipt.first().scene_node_realization_rebuilds() > 0);
            assert!(receipt.first().gpu_resource_creations() > 0);
            assert_zero_content_work(receipt.unchanged());
            assert_eq!(receipt.unchanged().render_plan_reuses(), 1);
            assert_eq!(receipt.unchanged().property_upload_bytes(), 0);
            assert_eq!(
                receipt.unchanged().opaque_nodes() + receipt.unchanged().blended_nodes(),
                receipt.first().opaque_nodes() + receipt.first().blended_nodes()
            );
            assert_eq!(receipt.unchanged().opacity_unclassified_nodes(), 0);
            assert!(receipt.recreated().scene_node_realization_rebuilds() > 0);
            assert_eq!(
                receipt.retired().gpu_resource_count(),
                RETAINED_GLOBAL_RESOURCES
            );
            assert!(receipt.retired().gpu_resource_removals() > 0);
        }

        let partial = harness
            .partial_update_receipt()
            .expect("partial update receipt should render");
        assert_eq!(partial.changed().scene_node_realization_rebuilds(), 1);
        assert_eq!(partial.changed().primitive_prepare_calls(), 1);
        assert!(partial.changed().content_upload_bytes() > 0);
        assert_eq!(partial.changed().gpu_resource_creations(), 1);
        assert_zero_content_work(partial.surviving());
        assert_eq!(partial.surviving().render_plan_reuses(), 1);
        assert!(partial.surviving().gpu_resource_removals() > 0);
        assert_eq!(
            partial.retired().gpu_resource_count(),
            RETAINED_GLOBAL_RESOURCES
        );
        assert!(partial.retired().gpu_resource_removals() > 0);
    }

    #[test]
    #[ignore = "requires a locally available GPU adapter"]
    fn retained_scroll_tick_is_pixel_exact_and_reuses_all_content_work() {
        for scale_factor in [1.0, 1.25, 1.5, 2.0] {
            let mut harness =
                pollster::block_on(Harness::new(scale_factor)).expect("GPU harness should open");
            let receipt = harness.scroll_tick_receipt().unwrap_or_else(|error| {
                panic!("retained scroll property tick should render at {scale_factor}x: {error}")
            });

            assert!(receipt.initial().scene_node_realization_rebuilds() > 0);
            assert_no_scroll_offscreen(receipt.initial());
            assert_zero_content_work(receipt.tick());
            assert_eq!(receipt.tick().render_plan_reuses(), 1);
            assert_eq!(
                receipt.tick().property_upload_bytes(),
                ONE_SCROLL_PROPERTY_UPLOAD
            );
            assert_no_scroll_offscreen(receipt.tick());
            assert_zero_content_work(receipt.unchanged());
            assert_eq!(receipt.unchanged().render_plan_reuses(), 1);
            assert_eq!(receipt.unchanged().property_upload_bytes(), 0);
            assert_no_scroll_offscreen(receipt.unchanged());
        }
    }

    #[test]
    #[ignore = "requires a locally available GPU adapter"]
    fn unrelated_semantic_commit_reuses_retained_scroll_subtree() {
        let mut harness = pollster::block_on(Harness::new(1.0)).expect("GPU harness should open");
        let receipt = harness
            .scroll_unrelated_semantic_receipt()
            .expect("an unrelated semantic commit should preserve the retained scroll subtree");

        assert_no_scroll_offscreen(receipt.initial());
        assert_no_scroll_offscreen(receipt.changed());
        assert_eq!(receipt.changed().property_upload_bytes(), 0);
        assert_no_scroll_offscreen(receipt.unchanged());
        assert_eq!(receipt.unchanged().render_plan_reuses(), 1);
    }

    #[test]
    #[ignore = "requires a locally available GPU adapter"]
    fn pending_semantic_realization_yields_to_exact_active_output() {
        let mut harness = pollster::block_on(Harness::new(1.0)).expect("GPU harness should open");
        let receipt = harness
            .pending_active_receipt()
            .expect("bounded pending preparation should preserve the active output");

        assert!(receipt.preparation_slices() > 1);
        assert!(receipt.active_draws() > 0);
        assert_eq!(receipt.peak_pending_states(), 1);
        assert!(receipt.peak_resources() > 0);
        assert!(receipt.peak_bytes() > 0);
        assert_eq!(
            receipt.activated().commit_preparation_slices(),
            receipt.preparation_slices()
        );
        assert!(receipt.activated().commit_preparation_max_nanos() > 0);
        assert_eq!(receipt.activated().commit_preparation_deadline_misses(), 0);
        assert_eq!(receipt.activated().render_plan_rebuilds(), 1);
        assert_no_scroll_offscreen(receipt.activated());
    }

    #[test]
    #[ignore = "requires a locally available GPU adapter"]
    fn retained_text_survives_shared_atlas_pressure_without_repreparation() {
        let mut harness = pollster::block_on(Harness::new(1.0)).expect("GPU harness should open");
        let receipt = harness
            .text_atlas_retention_receipt()
            .expect("new semantic text must not evict glyphs referenced by active retained text");

        assert!(receipt.pressure().text_prepare_calls() > 0);
        assert_zero_content_work(receipt.surviving());
        assert_eq!(receipt.surviving().property_upload_bytes(), 0);
        assert_eq!(receipt.surviving().render_plan_reuses(), 1);
    }

    #[test]
    #[ignore = "requires a locally available GPU adapter"]
    fn pending_resize_replaces_stale_viewport_state_without_residue() {
        let mut harness = pollster::block_on(Harness::new(1.0)).expect("GPU harness should open");
        harness
            .pending_resize_receipt()
            .expect("resize must replace pending and ready viewport state exactly");
    }

    #[test]
    #[ignore = "requires a locally available GPU adapter"]
    fn control_gallery_incremental_activation_matches_synchronous_pixels() {
        const WARMUP: usize = 1;
        const SAMPLES: usize = 8;
        let (warmup, receipts) = pollster::block_on(async {
            let warmup =
                wgpu_l3::diagnostics::compare_control_gallery_incremental_activation(1.0).await?;
            let mut receipts = Vec::with_capacity(SAMPLES);
            for _ in 0..SAMPLES {
                receipts.push(
                    wgpu_l3::diagnostics::compare_control_gallery_incremental_activation(1.0)
                        .await?,
                );
            }
            Ok::<_, String>((warmup, receipts))
        })
        .expect("incremental preparation must preserve the production gallery scene");
        let mut samples = receipts
            .iter()
            .map(|receipt| receipt.batch_prepare())
            .collect::<Vec<_>>();
        samples.sort_unstable();
        let p50 = samples[samples.len() / 2];
        let p95 = samples[samples.len() - 1];
        let environment = receipts[0].environment();
        eprintln!(
            "control-gallery activation receipt: workload=initial-production-commit scale=1.0 warmup={} samples={} adapter={:?} backend={} device_type={} os={} architecture={} warmup_us={} p50_us={} p95_us={} max_us={} acceptance_us=4167",
            WARMUP,
            SAMPLES,
            environment.adapter_name(),
            environment.backend(),
            environment.device_type(),
            environment.os(),
            environment.architecture(),
            warmup.batch_prepare().as_micros(),
            p50.as_micros(),
            p95.as_micros(),
            samples
                .last()
                .expect("activation sample should exist")
                .as_micros(),
        );
        assert!(warmup.slices() > 1);
        assert!(receipts.iter().all(|receipt| receipt.slices() > 1));
        assert!(p95 < std::time::Duration::from_micros(4_167));
        assert!(
            receipts
                .iter()
                .all(|receipt| receipt.activated().commit_preparation_slices() > 0)
        );
    }

    #[test]
    #[ignore = "requires a locally available GPU adapter"]
    fn control_gallery_pending_transition_preserves_exact_active_output() {
        pollster::block_on(wgpu_l3::diagnostics::compare_control_gallery_pending_transition(1.0))
            .expect("pending production resources must not change the active gallery output");
    }

    #[test]
    #[ignore = "requires a locally available GPU adapter"]
    fn control_gallery_pending_property_refresh_advances_active_output() {
        pollster::block_on(
            wgpu_l3::diagnostics::compare_control_gallery_pending_property_refresh(1.0),
        )
        .expect("pending semantic preparation must not stall compatible active properties");
    }

    #[test]
    #[ignore = "requires a locally available GPU adapter"]
    fn control_gallery_caret_blink_preserves_complete_output() {
        pollster::block_on(wgpu_l3::diagnostics::compare_control_gallery_caret_blink(
            1.0,
        ))
        .expect("caret blink commits must preserve complete active and candidate output");
    }

    #[test]
    #[ignore = "requires a locally available GPU adapter"]
    fn control_gallery_pending_scroll_keeps_visible_active_output() {
        for scale_factor in [1.0, 1.25, 1.5, 2.0] {
            pollster::block_on(wgpu_l3::diagnostics::compare_control_gallery_pending_scroll(
                scale_factor,
            ))
            .unwrap_or_else(|error| {
                panic!(
                    "pending gallery scroll must keep a visible complete active scene at {scale_factor}x: {error}"
                )
            });
        }
    }

    #[test]
    #[ignore = "requires a locally available GPU adapter"]
    fn control_gallery_property_tick_is_blend_equivalent_offscreen() {
        for scale_factor in [1.0, 1.25, 1.5, 2.0] {
            pollster::block_on(wgpu_l3::diagnostics::compare_control_gallery_property_tick(
                scale_factor,
            ))
            .unwrap_or_else(|error| {
                panic!(
                    "the production gallery property tick must match its compatibility oracle at {scale_factor}x: {error}"
                )
            });
        }
    }

    #[test]
    #[ignore = "requires a locally available GPU adapter"]
    fn retained_gpu_high_water_settles_after_commit_churn() {
        let mut harness = pollster::block_on(Harness::new(1.0)).expect("GPU harness should open");
        let receipt = harness
            .churn_receipt(64)
            .expect("churn receipt should render");

        assert_eq!(receipt.iterations(), 64);
        assert_eq!(
            receipt.post_warm_resource_range().0,
            receipt.post_warm_resource_range().1
        );
        assert_eq!(
            receipt.post_warm_byte_range().0,
            receipt.post_warm_byte_range().1
        );
        assert_eq!(
            receipt.settled().gpu_resource_count(),
            RETAINED_GLOBAL_RESOURCES
        );
        assert!(receipt.settled().gpu_resource_removals() > 0);
        assert!(receipt.peak_resources() > receipt.settled().gpu_resource_count());
        assert!(receipt.peak_bytes() >= receipt.settled().gpu_resource_bytes());
    }

    #[test]
    #[ignore = "requires a locally available GPU adapter"]
    fn semantic_work_routes_ordinary_content_direct_and_bounds_effect_scratch() {
        let mut harness = pollster::block_on(Harness::new(1.0)).expect("GPU harness should open");

        let ordinary = harness
            .work_receipt(Case::Rule)
            .expect("ordinary work receipt should render");
        assert_eq!(ordinary.direct_surface_plans(), 1);
        assert_eq!(ordinary.surface_sampling_plans(), 0);
        assert_eq!(ordinary.opacity_unclassified_nodes(), 0);
        assert_eq!(ordinary.opaque_nodes(), 1);
        assert_eq!(ordinary.blended_nodes(), 0);
        assert_eq!(ordinary.effect_intermediate_clears(), 0);
        assert_eq!(ordinary.effect_intermediate_composites(), 0);
        assert_eq!(ordinary.explicit_copy_commands(), 0);
        assert_eq!(
            ordinary.resource_transition_boundaries(),
            ordinary.draw_passes().saturating_sub(1)
        );

        let glass = harness
            .work_receipt(Case::GlassPane)
            .expect("glass work receipt should render");
        assert_eq!(glass.direct_surface_plans(), 0);
        assert_eq!(glass.surface_sampling_plans(), 1);
        assert_eq!(glass.opacity_unclassified_nodes(), 0);
        assert!(glass.effect_intermediate_clears() > 0);
        assert!(glass.effect_intermediate_clear_bytes() > 0);
        assert!(glass.effect_intermediate_composites() > 0);
        assert!(glass.effect_intermediate_composite_bytes() > 0);
        assert!(glass.largest_effect_intermediate_bytes() < glass.target_bytes());
        assert!(glass.draw_passes() > ordinary.draw_passes());
        assert_eq!(glass.explicit_copy_commands(), 0);
        assert_eq!(
            glass.resource_transition_boundaries(),
            glass.draw_passes().saturating_sub(1)
        );
    }
}
