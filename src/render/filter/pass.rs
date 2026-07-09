use bytemuck::{Pod, Zeroable};

use crate::paint::{self, Rect};
use crate::render;
use crate::render::silhouette::{edges, rect_data, rounding_data};

use super::chain::FilterSourceSpace;
use super::params::AlphaMode;
use super::{PreparedFilter, Target, TextureSource};

pub(super) struct BlurPass<'a> {
    pub(super) render_context: &'a render::Context,
    pub(super) encoder: &'a mut wgpu::CommandEncoder,
    pub(super) output: &'a wgpu::TextureView,
    pub(super) target: Target,
    pub(super) prepared: PreparedFilter,
    pub(super) direction: [f32; 2],
    pub(super) source: TextureSource<'a>,
    pub(super) source_rect: Rect,
    pub(super) source_space: FilterSourceSpace,
    pub(super) labels: BlurLabels,
}

pub(super) struct RefractionPass<'a> {
    pub(super) render_context: &'a render::Context,
    pub(super) encoder: &'a mut wgpu::CommandEncoder,
    pub(super) source: &'a wgpu::TextureView,
    pub(super) source_area: paint::area::Physical,
    pub(super) source_logical_area: paint::area::Logical,
    pub(super) source_rect: Rect,
    pub(super) source_sampling: paint::LayerSampling,
    pub(super) output: &'a wgpu::TextureView,
    pub(super) target: Target,
    pub(super) prepared: PreparedFilter,
    pub(super) effect: [f32; 4],
    pub(super) alpha_mode: AlphaMode,
    pub(super) scissor: Option<render::Scissor>,
}

pub(super) struct EffectPass<'a> {
    pub(super) render_context: &'a render::Context,
    pub(super) encoder: &'a mut wgpu::CommandEncoder,
    pub(super) source: &'a wgpu::TextureView,
    pub(super) source_area: paint::area::Physical,
    pub(super) source_logical_area: paint::area::Logical,
    pub(super) source_rect: Rect,
    pub(super) source_sampling: paint::LayerSampling,
    pub(super) output: &'a wgpu::TextureView,
    pub(super) target: Target,
    pub(super) prepared: PreparedFilter,
    pub(super) effect: [f32; 4],
    pub(super) alpha_mode: AlphaMode,
    pub(super) pipeline: &'a wgpu::RenderPipeline,
    pub(super) scissor: Option<render::Scissor>,
    pub(super) labels: PassLabels,
}

pub(super) struct CompositePass<'a> {
    pub(super) render_context: &'a render::Context,
    pub(super) encoder: &'a mut wgpu::CommandEncoder,
    pub(super) source: TextureSource<'a>,
    pub(super) output: &'a wgpu::TextureView,
    pub(super) target: Target,
    pub(super) prepared: PreparedFilter,
    pub(super) source_rect: Rect,
    pub(super) opacity: f32,
    pub(super) alpha_mode: AlphaMode,
    pub(super) scissor: Option<render::Scissor>,
    pub(super) labels: PassLabels,
}

#[derive(Clone, Copy)]
pub(super) struct PassLabels {
    pub(super) bind_group: &'static str,
    pub(super) vertex_buffer: &'static str,
    pub(super) pass: &'static str,
}

#[derive(Clone, Copy)]
pub(super) struct BlurLabels {
    pub(super) bind_group: &'static str,
    pub(super) pass: &'static str,
}

impl BlurLabels {
    pub(super) const fn new(bind_group: &'static str, pass: &'static str) -> Self {
        Self { bind_group, pass }
    }
}

impl PassLabels {
    pub(super) const fn new(
        bind_group: &'static str,
        vertex_buffer: &'static str,
        pass: &'static str,
    ) -> Self {
        Self {
            bind_group,
            vertex_buffer,
            pass,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct CompositeVertex {
    pub(super) position: [f32; 2],
    pub(super) local_position: [f32; 2],
    pub(super) rect: [f32; 4],
    pub(super) rounding: [f32; 4],
}

impl CompositeVertex {
    pub(super) fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
            2 => Float32x4,
            3 => Float32x4
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

pub(super) fn composite_vertices(
    canvas_area: paint::area::Logical,
    prepared: PreparedFilter,
) -> [CompositeVertex; 6] {
    let to_clip = |x: f32, y: f32| -> [f32; 2] {
        [
            (x / canvas_area.width()) * 2.0 - 1.0,
            1.0 - (y / canvas_area.height()) * 2.0,
        ]
    };
    let (x0, y0, x1, y1) = edges(prepared.raster_rect);
    let rect = rect_data(prepared.shape_rect);
    let rounding = rounding_data(prepared.rounding);
    let vertex = |x: f32, y: f32| CompositeVertex {
        position: to_clip(x, y),
        local_position: [x, y],
        rect,
        rounding,
    };

    [
        vertex(x0, y0),
        vertex(x0, y1),
        vertex(x1, y1),
        vertex(x0, y0),
        vertex(x1, y1),
        vertex(x1, y0),
    ]
}
