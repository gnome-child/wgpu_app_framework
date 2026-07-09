use crate::paint;

use super::geometry::{prepare_clip, source_rect_for_prepared_destination};
use super::params::AlphaMode;
use super::pass::{CompositePass, PassLabels};
use super::state::Renderer;
use super::storage::LayerComposite;

impl Renderer {
    pub(in crate::render) fn composite_layer(&self, pass: LayerComposite<'_>) {
        let Some(prepared) = prepare_clip(pass.clip.rect, pass.target.scale_factor) else {
            return;
        };
        let source_rect = pass.source_rect.unwrap_or_else(|| {
            source_rect_for_prepared_destination(pass.clip.rect, prepared, pass.clip.rect)
        });

        self.composite_pass(CompositePass {
            render_context: pass.render_context,
            encoder: pass.encoder,
            source: pass.source.source(paint::LayerSampling::PixelAligned),
            output: pass.output,
            target: pass.target,
            prepared,
            source_rect,
            opacity: pass.opacity,
            alpha_mode: AlphaMode::Source,
            scissor: pass.scissor,
            labels: PassLabels {
                bind_group: "Layer Composite Bind Group",
                vertex_buffer: "Layer Composite Vertex Buffer",
                pass: "Layer Composite Pass",
            },
        });
    }
}
