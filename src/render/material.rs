use crate::paint;
use crate::render;

pub(super) fn backdrop_filter(pane: &paint::Pane, glass: &paint::Glass) -> paint::Filter {
    let mut filter = paint::Filter::stack(
        pane.rect,
        glass.backdrop_layers.iter().copied().map(backdrop_layer_op),
    );
    filter.source_rect = pane.source_rect;
    filter
}

pub(super) fn backdrop_source<'a>(
    inside_group: bool,
    current_target_dirty: bool,
    pane: &paint::Pane,
    output: &'a wgpu::TextureView,
    output_target: render::filter::Target,
    backdrop_view: &'a wgpu::TextureView,
    backdrop_target: render::filter::Target,
) -> render::filter::FilterSource<'a> {
    let source_rect = pane.source_rect.unwrap_or(pane.rect);
    if inside_group {
        return render::filter::FilterSource::Backdrop {
            texture: render::filter::TextureSource::new(
                backdrop_view,
                backdrop_target.physical_area(),
                backdrop_target.logical_area(),
                paint::LayerSampling::PixelAligned,
            ),
            global_rect: source_rect,
        };
    }

    if current_target_dirty {
        render::filter::FilterSource::Local {
            texture: render::filter::TextureSource::new(
                output,
                output_target.physical_area(),
                output_target.logical_area(),
                paint::LayerSampling::PixelAligned,
            ),
            local_rect: pane.rect,
        }
    } else {
        render::filter::FilterSource::Backdrop {
            texture: render::filter::TextureSource::new(
                backdrop_view,
                backdrop_target.physical_area(),
                backdrop_target.logical_area(),
                paint::LayerSampling::PixelAligned,
            ),
            global_rect: source_rect,
        }
    }
}

pub(super) fn layer_sequence(glass: &paint::Glass) -> Vec<&'static str> {
    let mut layers = Vec::new();
    for layer in &glass.backdrop_layers {
        layers.push(match layer {
            paint::BackdropLayer::Blur(_) => "backdrop-blur",
            paint::BackdropLayer::Refraction(_) => "refraction",
            paint::BackdropLayer::Luminosity(_) => "luminosity",
        });
    }
    for layer in &glass.surface_layers {
        layers.push(match layer {
            paint::SurfaceLayer::Tint { .. } => "tint",
            paint::SurfaceLayer::Noise(_) => "noise",
        });
    }
    layers
}

pub(super) fn brush_with_opacity(brush: paint::Brush, opacity: f32) -> paint::Brush {
    let opacity = opacity.clamp(0.0, 1.0);
    match brush {
        paint::Brush::Solid(color) => paint::Brush::solid(color_with_opacity(color, opacity)),
        paint::Brush::Gradient(gradient) => {
            paint::Brush::Gradient(gradient_with_opacity(gradient, opacity))
        }
    }
}

fn backdrop_layer_op(layer: paint::BackdropLayer) -> paint::FilterOp {
    match layer {
        paint::BackdropLayer::Blur(blur) => paint::FilterOp::backdrop_blur(blur),
        paint::BackdropLayer::Refraction(refraction) => paint::FilterOp::refraction(refraction),
        paint::BackdropLayer::Luminosity(luminosity) => paint::FilterOp::luminosity(luminosity),
    }
}

fn gradient_with_opacity(gradient: paint::Gradient, opacity: f32) -> paint::Gradient {
    match gradient {
        paint::Gradient::Linear(gradient) => paint::Gradient::Linear(paint::LinearGradient::new(
            gradient.start(),
            gradient.end(),
            color_with_opacity(gradient.from(), opacity),
            color_with_opacity(gradient.to(), opacity),
        )),
    }
}

fn color_with_opacity(color: paint::Color, opacity: f32) -> paint::Color {
    paint::Color::rgba(color.r, color.g, color.b, color.a * opacity)
}
