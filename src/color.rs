#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    pub fn channels(self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }
}

pub(crate) fn srgb_byte_to_linear(value: u8) -> f32 {
    srgb_unit_to_linear(byte_to_unit(value))
}

pub(crate) fn srgb_unit_to_linear(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

pub(crate) fn linear_unit_to_srgb_byte(value: f32) -> u8 {
    let value = value.clamp(0.0, 1.0);
    let srgb = if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    };
    unit_to_byte(srgb)
}

pub(crate) fn byte_to_unit(value: u8) -> f32 {
    value as f32 / 255.0
}

pub(crate) fn unit_to_byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

pub(crate) fn aabbggrr(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((a as u32) << 24) | ((b as u32) << 16) | ((g as u32) << 8) | r as u32
}

pub(crate) fn bbggrr(r: u8, g: u8, b: u8) -> u32 {
    ((b as u32) << 16) | ((g as u32) << 8) | r as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srgb_bytes_decode_and_reencode_exactly() {
        for value in [0, 1, 10, 64, 128, 200, 254, 255] {
            assert_eq!(linear_unit_to_srgb_byte(srgb_byte_to_linear(value)), value);
        }
    }

    #[test]
    fn transfer_function_uses_piecewise_srgb() {
        assert!((srgb_byte_to_linear(128) - 0.215_860_53).abs() < 0.000_001);
        assert!((srgb_byte_to_linear(64) - 0.051_269_468).abs() < 0.000_001);
        assert!((srgb_byte_to_linear(32) - 0.014_443_844).abs() < 0.000_001);
    }

    #[test]
    fn accent_gradient_uses_named_aabbggrr_order() {
        assert_eq!(aabbggrr(0x11, 0x22, 0x33, 0x44), 0x4433_2211);
    }

    #[test]
    fn windows_colorref_uses_named_bbggrr_order() {
        assert_eq!(bbggrr(0x11, 0x22, 0x33), 0x0033_2211);
    }

    #[test]
    fn conversion_sites_delegate_to_the_color_owner() {
        let render_color = include_str!("render/color.rs");
        let render_scene = include_str!("render/scene.rs");
        let layout_text = include_str!("layout/text.rs");
        let text_system = include_str!("text/layout/system.rs");
        let text_renderer = include_str!("render/text_renderer.rs");
        let sys = include_str!("platform/native/sys/mod.rs");

        for source in [
            render_color,
            render_scene,
            layout_text,
            text_system,
            text_renderer,
            sys,
        ] {
            assert!(source.contains("crate::color::"));
        }
        assert!(!render_color.contains("powf"));
        assert!(!sys.contains("<<"));
    }

    #[test]
    fn semantic_color_value_has_one_lower_owner() {
        let scene = include_str!("scene/mod.rs");
        let window_facts = include_str!("window/facts.rs");
        let window_options = include_str!("window/options.rs");

        assert!(scene.contains("pub use crate::color::Color;"));
        assert!(!window_facts.contains("scene::Color"));
        assert!(!window_options.contains("scene::Color"));
    }
}
