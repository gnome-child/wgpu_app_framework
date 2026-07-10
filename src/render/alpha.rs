/// Declares what a fragment shader writes before the color target applies its
/// blend state. Render-target storage is premultiplied (associated) RGBA after
/// any blend, regardless of whether the texture format encodes RGB as sRGB.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::render) enum FragmentOutput {
    /// Straight RGB plus alpha; fixed-function blending associates the color.
    Straight,
    /// RGB is already multiplied by alpha; source-over must not multiply again.
    Premultiplied,
    /// The shader's RGBA value replaces the destination exactly.
    Replace,
}

pub(in crate::render) fn color_target(
    format: wgpu::TextureFormat,
    output: FragmentOutput,
) -> wgpu::ColorTargetState {
    let blend = match output {
        FragmentOutput::Straight => Some(wgpu::BlendState::ALPHA_BLENDING),
        FragmentOutput::Premultiplied => Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        FragmentOutput::Replace => None,
    };

    wgpu::ColorTargetState {
        format,
        blend,
        write_mask: wgpu::ColorWrites::ALL,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fragment_output_convention_owns_every_local_blend_state() {
        let format = wgpu::TextureFormat::Rgba8Unorm;

        assert_eq!(
            color_target(format, FragmentOutput::Straight).blend,
            Some(wgpu::BlendState::ALPHA_BLENDING)
        );
        assert_eq!(
            color_target(format, FragmentOutput::Premultiplied).blend,
            Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING)
        );
        assert_eq!(color_target(format, FragmentOutput::Replace).blend, None);
    }
}
