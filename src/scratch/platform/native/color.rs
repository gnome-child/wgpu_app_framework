use crate::{paint, scratch::scene, text};

pub(in crate::scratch::platform::native) fn paint_color(color: scene::Color) -> paint::Color {
    let (r, g, b, a) = color.channels();

    paint::Color::rgba(
        linear_channel(r),
        linear_channel(g),
        linear_channel(b),
        alpha_channel(a),
    )
}

pub(in crate::scratch::platform::native) fn text_color(color: scene::Color) -> text::Color {
    let color = paint_color(color);

    text::Color::rgba(color.r, color.g, color.b, color.a)
}

fn linear_channel(channel: u8) -> f32 {
    let value = alpha_channel(channel);

    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn alpha_channel(channel: u8) -> f32 {
    channel as f32 / 255.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_color_bytes_convert_to_linear_paint_color() {
        let color = paint_color(scene::Color::rgba(128, 64, 32, 128));

        assert!((color.r - 0.21586053).abs() < 0.000001);
        assert!((color.g - 0.051269468).abs() < 0.000001);
        assert!((color.b - 0.014443844).abs() < 0.000001);
        assert!((color.a - (128.0 / 255.0)).abs() < 0.000001);
    }
}
