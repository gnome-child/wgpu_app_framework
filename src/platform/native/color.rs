use crate::{paint, scene, text};

pub(in crate::platform::native) fn paint_color(color: scene::Color) -> paint::Color {
    let (r, g, b, a) = color.channels();
    paint::Color::rgba(
        crate::color::srgb_byte_to_linear(r),
        crate::color::srgb_byte_to_linear(g),
        crate::color::srgb_byte_to_linear(b),
        crate::color::byte_to_unit(a),
    )
}

pub(in crate::platform::native) fn text_color(color: scene::Color) -> text::Color {
    let (r, g, b, a) = color.channels();
    text::Color::rgba(
        crate::color::byte_to_unit(r),
        crate::color::byte_to_unit(g),
        crate::color::byte_to_unit(b),
        crate::color::byte_to_unit(a),
    )
}
