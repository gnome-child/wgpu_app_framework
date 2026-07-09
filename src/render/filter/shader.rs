use crate::render::silhouette;

const SOURCE: &str = include_str!("../filter.wgsl");

pub(super) fn module_source() -> String {
    silhouette::wgsl_module_source(SOURCE)
}

#[cfg(test)]
pub(super) fn raw_source() -> &'static str {
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
        assert!(source.contains("fn filter_source_rgb"));
        assert!(source.contains("params.alpha_mode.x"));
    }

    #[test]
    fn shape_alpha_mode_reads_source_rgb_without_unpremultiply() {
        let source = raw_source();
        let helper = source
            .split("fn filter_source_rgb")
            .nth(1)
            .expect("source rgb helper should exist")
            .split("fn filter_alpha")
            .next()
            .expect("source rgb helper should precede alpha helper");

        assert!(helper.contains("return color.rgb;"));
        assert!(helper.contains("return unpremultiply(color);"));

        for fragment in [
            "fs_composite",
            "fs_liquid",
            "fs_luminosity",
            "fs_noise",
            "fs_composite_pixel",
        ] {
            let body = source
                .split(&format!("fn {fragment}"))
                .nth(1)
                .unwrap_or_else(|| panic!("{fragment} fragment should exist"))
                .split("@fragment")
                .next()
                .expect("fragment body should be bounded");

            assert!(body.contains("filter_source_rgb(color)"));
            assert!(!body.contains("unpremultiply(color)"));
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
