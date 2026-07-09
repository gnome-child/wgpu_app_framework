use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::render) struct Vertex {
    pub(in crate::render) position: [f32; 2],
    pub(in crate::render) local_position: [f32; 2],
    pub(in crate::render) outer_rect: [f32; 4],
    pub(in crate::render) outer_rounding: [f32; 4],
    pub(in crate::render) inner_rect: [f32; 4],
    pub(in crate::render) inner_rounding: [f32; 4],
    pub(in crate::render) color: [f32; 4],
    pub(in crate::render) color_to: [f32; 4],
    pub(in crate::render) brush_points: [f32; 4],
    pub(in crate::render) params: [f32; 4],
}

impl Vertex {
    pub(in crate::render) fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 10] = wgpu::vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
            2 => Float32x4,
            3 => Float32x4,
            4 => Float32x4,
            5 => Float32x4,
            6 => Float32x4,
            7 => Float32x4,
            8 => Float32x4,
            9 => Float32x4
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}
