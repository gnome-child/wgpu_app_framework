use std::cell::RefCell;
use std::collections::{HashMap, hash_map::Entry};

use crate::render;

const POPUP_PACK_WGSL: &str = include_str!("popup_pack.wgsl");

pub(crate) struct Packer {
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    pipelines: RefCell<HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>>,
}

impl Packer {
    pub(crate) fn new(render_context: &render::Context) -> Self {
        let bind_group_layout =
            render_context
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Popup Pack Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });
        let sampler = render_context
            .device()
            .create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Popup Pack Sampler"),
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

        Self {
            bind_group_layout,
            sampler,
            pipelines: RefCell::new(HashMap::new()),
        }
    }

    pub(crate) fn pack_to_view(
        &self,
        render_context: &render::Context,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        output: &wgpu::TextureView,
        output_format: wgpu::TextureFormat,
    ) {
        let mut pipelines = self.pipelines.borrow_mut();
        let pipeline = match pipelines.entry(output_format) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                entry.insert(self.create_pipeline(render_context, output_format))
            }
        };
        let bind_group = render_context
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Popup Pack Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(source),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Popup Premultiplied sRGB Pack Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }

    fn create_pipeline(
        &self,
        render_context: &render::Context,
        output_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let shader = render_context
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("popup_pack.wgsl"),
                source: wgpu::ShaderSource::Wgsl(POPUP_PACK_WGSL.into()),
            });
        let pipeline_layout =
            render_context
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Popup Pack Pipeline Layout"),
                    bind_group_layouts: &[Some(&self.bind_group_layout)],
                    immediate_size: 0,
                });
        render_context
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Popup Premultiplied sRGB Pack Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(render::alpha::color_target(
                        output_format,
                        render::alpha::FragmentOutput::Replace,
                    ))],
                    compilation_options: Default::default(),
                }),
                multiview_mask: None,
                cache: None,
            })
    }
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = super::POPUP_PACK_WGSL;

    #[test]
    fn popup_pack_shader_uses_piecewise_srgb_encoding() {
        assert!(SOURCE.contains("0.0031308"));
        assert!(SOURCE.contains("12.92"));
        assert!(SOURCE.contains("1.055 * pow(v, 1.0 / 2.4) - 0.055"));
        assert!(
            !SOURCE.contains("2.2"),
            "popup pack shader must not approximate sRGB with gamma 2.2"
        );
    }

    #[test]
    fn popup_pack_shader_repremultiplies_after_encoding() {
        assert!(SOURCE.contains("color.rgb / alpha"));
        assert!(SOURCE.contains("srgb_encode(straight) * alpha"));
        assert!(SOURCE.contains("return vec4<f32>(packed_rgb, alpha)"));
    }
}
