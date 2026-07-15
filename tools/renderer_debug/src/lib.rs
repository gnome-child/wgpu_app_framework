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
}
