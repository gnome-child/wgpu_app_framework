use crate::render::silhouette;

const SOURCE: &str = include_str!("../filter.wgsl");

pub(super) fn module_source() -> String {
    silhouette::wgsl_module_source(SOURCE)
}

#[cfg(test)]
fn raw_source() -> &'static str {
    SOURCE
}

#[cfg(test)]
mod tests {
    use super::raw_source;

    #[test]
    fn uses_named_alpha_mode_helper() {
        let source = raw_source();

        assert!(source.contains("alpha_mode: vec4<f32>"));
        assert!(source.contains("fn filter_alpha"));
        assert!(source.contains("fn composite_source"));
        assert!(source.contains("fn filtered_source"));
        assert!(source.contains("params.alpha_mode.x"));
    }

    #[test]
    fn source_alpha_composite_never_round_trips_through_straight_rgb() {
        let source = raw_source();
        let helper = source
            .split("fn composite_source")
            .nth(1)
            .expect("composite helper should exist")
            .split("fn filtered_source")
            .next()
            .expect("composite helper should precede filtered helper");

        assert!(helper.contains("return associate(unpremultiply(color), coverage);"));
        assert!(helper.contains("return vec4<f32>(color.rgb * coverage, color.a * coverage);"));
        assert_eq!(
            helper.matches("unpremultiply(color)").count(),
            1,
            "only shape-alpha replacement may unpremultiply a composite source"
        );

        for fragment in ["fs_composite", "fs_refraction", "fs_composite_pixel"] {
            let body = source
                .split(&format!("fn {fragment}"))
                .nth(1)
                .unwrap_or_else(|| panic!("{fragment} fragment should exist"))
                .split("@fragment")
                .next()
                .expect("fragment body should be bounded");

            assert!(body.contains("composite_source(color"));
            assert!(!body.contains("unpremultiply(color)"));
        }
    }

    #[test]
    fn color_changing_filters_reassociate_their_straight_result() {
        let source = raw_source();

        for fragment in ["fs_luminosity", "fs_noise"] {
            let body = source
                .split(&format!("fn {fragment}"))
                .nth(1)
                .unwrap_or_else(|| panic!("{fragment} fragment should exist"))
                .split("@fragment")
                .next()
                .expect("fragment body should be bounded");

            assert!(body.contains("let rgb = unpremultiply(color);"));
            assert!(body.contains("return filtered_source(adjusted, color.a, alpha, 1.0);"));
        }
    }

    #[test]
    fn noise_fragment_uses_material_local_coordinates() {
        let noise_body = raw_source()
            .split("fn fs_noise")
            .nth(1)
            .expect("noise fragment should exist")
            .split("fn fs_composite_pixel")
            .next()
            .expect("noise fragment should precede pixel composite");

        assert!(noise_body.contains("material_position_for_local"));
        assert!(
            !noise_body.contains("source_position_for_local(in.local_position) / vec2<f32>(128.0)")
        );
    }
}
