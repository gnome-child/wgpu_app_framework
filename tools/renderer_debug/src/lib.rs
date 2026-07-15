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
            assert!(receipt.recreated().scene_node_realization_rebuilds() > 0);
            assert_eq!(receipt.retired().gpu_resource_count(), 4);
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
        assert_eq!(partial.retired().gpu_resource_count(), 4);
        assert!(partial.retired().gpu_resource_removals() > 0);
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
        assert_eq!(receipt.settled().gpu_resource_count(), 4);
        assert!(receipt.settled().gpu_resource_removals() > 0);
        assert!(receipt.peak_resources() > receipt.settled().gpu_resource_count());
        assert!(receipt.peak_bytes() >= receipt.settled().gpu_resource_bytes());
    }
}
