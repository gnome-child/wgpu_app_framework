use crate::geometry::{area, point};
use crate::paint;
use crate::paint::Rect;
use crate::render;
use crate::render::DrawStats;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DrawReport {
    pub(crate) stats: DrawStats,
    pub(crate) batch_prepare: std::time::Duration,
    pub(crate) acquire_outcome: render::surface::AcquireOutcome,
    pub(crate) present_timing: Option<render::surface::PresentTiming>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommitReadiness {
    Pending,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SurfacePath {
    PackedPremultiplied,
    Direct,
    SampledComposition,
}

pub struct Renderer {
    format: wgpu::TextureFormat,
    filter_renderer: render::filter::Renderer,
    popup_packer: render::popup_pack::Packer,
    text_renderer: render::text_renderer::TextRenderer,
    retained: render::retained::Realizer,
    ready_stacks: Vec<ReadyStack>,
    #[cfg(feature = "renderer-debug")]
    ready_debug_commits: Vec<ReadyDebugCommit>,
}

struct ReadyStack {
    stack: std::sync::Weak<crate::scene::Stack>,
    viewport: render::Viewport,
}

#[cfg(feature = "renderer-debug")]
struct ReadyDebugCommit {
    commit: std::sync::Weak<crate::scene::Commit>,
    viewport: render::Viewport,
    properties: crate::scene::PropertySerial,
}

#[derive(Debug, Clone)]
pub(in crate::render) enum PlanStep {
    Layer(PreparedLayer),
    Shapes(render::retained::ShapeBatch),
    Pane(PreparedPane),
    Text(render::text_renderer::RetainedBatch),
    PushClip(PreparedClip),
    PopClip,
    Group(PreparedGroup),
    Scroll(PreparedScroll),
}

#[derive(Debug, Clone)]
pub(in crate::render) struct PreparedLayer {
    pub(in crate::render) state_index: usize,
    pub(in crate::render) bounds: Rect,
    pub(in crate::render) opacity: f32,
    pub(in crate::render) force_group: bool,
    pub(in crate::render) render_batches: Vec<PlanStep>,
}

#[derive(Debug, Clone)]
pub(in crate::render) struct PreparedPane {
    pub(in crate::render) pane: paint::Pane,
    pub(in crate::render) base: Option<render::retained::ShapeBatch>,
    pub(in crate::render) surface_layers: Vec<Option<render::retained::ShapeBatch>>,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::render) struct PreparedClip {
    pub(in crate::render) node: Option<crate::composition::tree::NodeId>,
    pub(in crate::render) fallback: paint::Clip,
    pub(in crate::render) scene_origin: [f32; 2],
}

#[derive(Debug, Clone)]
pub(in crate::render) struct PreparedGroup {
    pub(in crate::render) node: Option<crate::composition::tree::NodeId>,
    pub(in crate::render) bounds: Rect,
    pub(in crate::render) opacity: f32,
    pub(in crate::render) render_batches: Vec<PlanStep>,
}

#[derive(Debug, Clone)]
pub(in crate::render) struct PreparedScroll {
    pub(in crate::render) node: crate::composition::tree::NodeId,
    pub(in crate::render) viewport: paint::Rect,
    pub(in crate::render) baseline: crate::interaction::ScrollOffset,
    pub(in crate::render) render_batches: Vec<PlanStep>,
}

#[derive(Default)]
struct CommandStats {
    draw_calls: usize,
    pipeline_changes: usize,
    bind_group_changes: usize,
    property_upload_bytes: usize,
    scroll_layer_cache_hits: usize,
    scroll_layer_cache_misses: usize,
    effect_intermediate_clears: usize,
    effect_intermediate_clear_bytes: u64,
    effect_intermediate_composites: usize,
    effect_intermediate_composite_bytes: u64,
    largest_effect_intermediate_bytes: u64,
}

impl CommandStats {
    fn record_draws(&mut self, count: usize, binds_resources: bool) {
        self.draw_calls += count;
        self.pipeline_changes += count;
        if binds_resources {
            self.bind_group_changes += count;
        }
    }

    fn record_filter_work(&mut self, work: render::filter::FilterWork) {
        self.effect_intermediate_clears += work.intermediate_clears;
        self.effect_intermediate_clear_bytes = self
            .effect_intermediate_clear_bytes
            .saturating_add(work.intermediate_clear_bytes);
        self.effect_intermediate_composites += work.intermediate_composites;
        self.effect_intermediate_composite_bytes = self
            .effect_intermediate_composite_bytes
            .saturating_add(work.intermediate_composite_bytes);
        self.largest_effect_intermediate_bytes = self
            .largest_effect_intermediate_bytes
            .max(work.largest_intermediate_bytes);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ClipFrame {
    PassThrough,
    Scissor(paint::Clip),
    Layer {
        clip: paint::Clip,
        layer_index: usize,
        dirty: bool,
    },
}

impl ClipFrame {
    fn clip(self) -> Option<paint::Clip> {
        match self {
            Self::PassThrough => None,
            Self::Scissor(clip) | Self::Layer { clip, .. } => Some(clip),
        }
    }

    fn layer_index(self) -> Option<usize> {
        match self {
            Self::Layer { layer_index, .. } => Some(layer_index),
            Self::PassThrough | Self::Scissor(_) => None,
        }
    }
}

struct PlanEncoder<'a> {
    render_context: &'a render::Context,
    filter_renderer: &'a render::filter::Renderer,
    text_renderer: &'a mut render::text_renderer::TextRenderer,
    retained_shapes: Option<&'a render::retained::Shapes>,
    layer_states: &'a [LayerState<'a>],
    layer_index: Option<usize>,
    output_target: render::filter::Target,
    encoder: &'a mut wgpu::CommandEncoder,
    base_view: &'a wgpu::TextureView,
    backdrop_view: &'a wgpu::TextureView,
    backdrop_target: render::filter::Target,
    clear_color: wgpu::Color,
    viewport: render::Viewport,
    layers: Vec<render::filter::Layer>,
    clip_stack: Vec<ClipFrame>,
    base_dirty: bool,
    inside_group: bool,
    text_render_error: &'a mut Option<render::text_renderer::Error>,
    draw_passes: &'a mut usize,
    command_stats: &'a mut CommandStats,
    scroll_translation: [f32; 2],
    scroll_clip: Option<paint::Rect>,
}

struct PlanEncoderInput<'a> {
    render_context: &'a render::Context,
    filter_renderer: &'a render::filter::Renderer,
    text_renderer: &'a mut render::text_renderer::TextRenderer,
    retained_shapes: Option<&'a render::retained::Shapes>,
    layer_states: &'a [LayerState<'a>],
    layer_index: Option<usize>,
    encoder: &'a mut wgpu::CommandEncoder,
    base_view: &'a wgpu::TextureView,
    backdrop_view: &'a wgpu::TextureView,
    backdrop_target: render::filter::Target,
    clear_color: wgpu::Color,
    viewport: render::Viewport,
    base_dirty: bool,
    inside_group: bool,
    text_render_error: &'a mut Option<render::text_renderer::Error>,
    draw_passes: &'a mut usize,
    command_stats: &'a mut CommandStats,
}

struct LayerState<'a> {
    bindings: render::retained::PropertyBindings,
    properties: &'a crate::scene::Properties,
}

impl Renderer {
    pub fn new(render_context: &render::Context, format: render::surface::Format) -> Self {
        let format = format.into_wgpu();
        Self {
            format,
            filter_renderer: render::filter::Renderer::new(render_context, format),
            popup_packer: render::popup_pack::Packer::new(render_context),
            text_renderer: render::text_renderer::TextRenderer::new(render_context, format),
            retained: render::retained::Realizer::new(render_context, format),
            ready_stacks: Vec::new(),
            #[cfg(feature = "renderer-debug")]
            ready_debug_commits: Vec::new(),
        }
    }

    pub fn clear(
        &mut self,
        render_context: &render::Context,
        canvas: &mut render::Canvas,
    ) -> render::Result<()> {
        let clear_color = canvas.color();

        let _ = canvas.draw(render_context, |encoder, frame| {
            let view = frame.create_view();

            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        })?;

        Ok(())
    }

    pub(crate) fn synchronize_stack(
        &mut self,
        render_context: &render::Context,
        canvas: &render::Canvas,
        stack: &std::sync::Arc<crate::scene::Stack>,
        budget: std::time::Duration,
        deadline: std::time::Duration,
    ) -> render::Result<CommitReadiness> {
        let viewport = render::Viewport::from_canvas(canvas);
        let started = std::time::Instant::now();
        for layer in stack.layers() {
            let remaining = budget.saturating_sub(started.elapsed());
            if remaining.is_zero()
                || self.retained.synchronize_layer(
                    render_context,
                    viewport,
                    layer,
                    &mut self.text_renderer,
                    remaining,
                    deadline,
                )? == render::retained::Synchronize::Pending
            {
                return Ok(CommitReadiness::Pending);
            }
        }
        self.ready_stacks
            .retain(|candidate| candidate.stack.strong_count() > 0);
        Ok(
            if self.ready_stacks.iter().any(|candidate| {
                candidate.viewport == viewport
                    && candidate
                        .stack
                        .upgrade()
                        .is_some_and(|candidate| std::sync::Arc::ptr_eq(&candidate, stack))
            }) {
                CommitReadiness::Ready
            } else {
                CommitReadiness::Pending
            },
        )
    }

    #[cfg(feature = "renderer-debug")]
    fn synchronize_commit_for_viewport(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        commit: &std::sync::Arc<crate::scene::Commit>,
        properties: crate::scene::PropertySerial,
        budget: std::time::Duration,
        deadline: std::time::Duration,
    ) -> render::Result<CommitReadiness> {
        let readiness = self.retained.synchronize_bounded(
            render_context,
            viewport,
            commit,
            &mut self.text_renderer,
            budget,
            deadline,
        )?;
        if readiness == render::retained::Synchronize::Pending {
            return Ok(CommitReadiness::Pending);
        }
        self.ready_debug_commits
            .retain(|candidate| candidate.commit.strong_count() > 0);
        Ok(
            if self.ready_debug_commits.iter().any(|candidate| {
                candidate.viewport == viewport
                    && candidate.properties == properties
                    && candidate
                        .commit
                        .upgrade()
                        .is_some_and(|candidate| std::sync::Arc::ptr_eq(&candidate, commit))
            }) {
                CommitReadiness::Ready
            } else {
                CommitReadiness::Pending
            },
        )
    }

    pub(crate) fn cancel_stack_synchronization(
        &mut self,
        stack: &std::sync::Arc<crate::scene::Stack>,
        protected: Option<&std::sync::Arc<crate::scene::Stack>>,
    ) {
        for layer in stack.layers() {
            self.retained.cancel_layer_synchronization(layer);
            let commit = layer.drawable_commit();
            let is_protected = protected.is_some_and(|protected| {
                protected
                    .layers()
                    .iter()
                    .any(|layer| std::sync::Arc::ptr_eq(layer.drawable_commit(), commit))
            });
            if !is_protected {
                self.retained.cancel_property_state(commit);
                self.text_renderer.cancel_retained_transform_state(commit);
            }
        }
        self.ready_stacks.retain(|candidate| {
            candidate
                .stack
                .upgrade()
                .is_some_and(|candidate| !std::sync::Arc::ptr_eq(&candidate, stack))
        });
    }

    pub(crate) fn advance_stack_after_present(
        &mut self,
        render_context: &render::Context,
        canvas: &render::Canvas,
        stack: &std::sync::Arc<crate::scene::Stack>,
        deadline: std::time::Duration,
    ) -> render::Result<()> {
        let viewport = render::Viewport::from_canvas(canvas);
        self.ready_stacks.retain(|candidate| {
            candidate.stack.strong_count() > 0
                && !(candidate.viewport == viewport
                    && candidate
                        .stack
                        .upgrade()
                        .is_some_and(|candidate| std::sync::Arc::ptr_eq(&candidate, stack)))
        });

        let started = std::time::Instant::now();
        for layer in stack.layers() {
            let Some(_) = self.retained.prepare_candidate_layer(
                render_context,
                viewport,
                layer,
                &mut self.text_renderer,
            )?
            else {
                return Ok(());
            };
        }
        self.retained.record_candidate_layer_slice(
            viewport,
            stack.base(),
            started.elapsed(),
            deadline,
        );
        self.ready_stacks.push(ReadyStack {
            stack: std::sync::Arc::downgrade(stack),
            viewport,
        });
        Ok(())
    }

    #[cfg(feature = "renderer-debug")]
    fn advance_candidate_for_viewport_after_present(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        commit: &std::sync::Arc<crate::scene::Commit>,
        properties: &crate::scene::Properties,
        deadline: std::time::Duration,
    ) -> render::Result<()> {
        self.ready_debug_commits.retain(|candidate| {
            candidate.commit.strong_count() > 0
                && !(candidate.viewport == viewport
                    && candidate
                        .commit
                        .upgrade()
                        .is_some_and(|candidate| std::sync::Arc::ptr_eq(&candidate, commit))
                    && candidate.properties != properties.serial())
        });
        if self.ready_debug_commits.iter().any(|candidate| {
            candidate.viewport == viewport
                && candidate.properties == properties.serial()
                && candidate
                    .commit
                    .upgrade()
                    .is_some_and(|candidate| std::sync::Arc::ptr_eq(&candidate, commit))
        }) {
            return Ok(());
        }

        let started = std::time::Instant::now();
        let Some(_) = self.retained.prepare_candidate(
            render_context,
            viewport,
            commit,
            properties,
            &mut self.text_renderer,
        )?
        else {
            return Ok(());
        };
        self.retained
            .record_candidate_slice(viewport, commit, started.elapsed(), deadline);
        self.ready_debug_commits.push(ReadyDebugCommit {
            commit: std::sync::Arc::downgrade(commit),
            viewport,
            properties: properties.serial(),
        });
        Ok(())
    }

    pub fn draw_stack(
        &mut self,
        render_context: &render::Context,
        canvas: &mut render::Canvas,
        stack: &std::sync::Arc<crate::scene::Stack>,
    ) -> render::Result<DrawReport> {
        let batch_prepare_started = std::time::Instant::now();
        let main_viewport = render::Viewport::from_canvas(canvas);
        let mut direct_surface_eligible = true;
        let mut batches = Vec::with_capacity(stack.layers().len());
        let mut layer_states = Vec::with_capacity(stack.layers().len());
        let mut stats = DrawStats::default();

        for layer in stack.layers() {
            if layer.opacity() <= 0.0 {
                continue;
            }
            let prepared = self.retained.prepare_layer(
                render_context,
                main_viewport,
                layer,
                &mut self.text_renderer,
            )?;
            direct_surface_eligible &= !prepared.plan.requires_surface_sampling();
            stats.add(prepared.stats);
            let state_index = layer_states.len();
            layer_states.push(LayerState {
                bindings: prepared.properties,
                properties: layer.properties(),
            });
            batches.push(PlanStep::Layer(PreparedLayer {
                state_index,
                bounds: render::scene::to_paint_rect_value_at_scale(
                    layer.bounds(),
                    main_viewport.scale_factor(),
                ),
                opacity: layer.opacity(),
                force_group: layer.force_group(),
                render_batches: prepared.plan.batches().to_vec(),
            }));
        }
        stats.clipped_nodes = stats.clip_batches;
        stats.direct_surface_plans = usize::from(direct_surface_eligible);
        stats.surface_sampling_plans = usize::from(!direct_surface_eligible);

        self.draw_prepared(
            render_context,
            canvas,
            render::surface_color(stack.clear()),
            main_viewport,
            &batches,
            &layer_states,
            direct_surface_eligible,
            stats,
            batch_prepare_started.elapsed(),
        )
    }

    fn draw_prepared(
        &mut self,
        render_context: &render::Context,
        canvas: &mut render::Canvas,
        clear_color: wgpu::Color,
        main_viewport: render::Viewport,
        batches: &[PlanStep],
        layer_states: &[LayerState<'_>],
        direct_surface_eligible: bool,
        mut stats: DrawStats,
        batch_prepare: std::time::Duration,
    ) -> render::Result<DrawReport> {
        validate_retained_layers(
            batches,
            main_viewport.scale_factor(),
            render_context.device().limits().max_texture_dimension_2d,
        )?;
        let preserve_surface_alpha =
            canvas.composite_alpha_mode() == wgpu::CompositeAlphaMode::PreMultiplied;
        let surface_format = canvas.surface().format();
        let surface_path = surface_path(
            canvas.surface().windows_popup_support().is_available(),
            preserve_surface_alpha,
            direct_surface_eligible,
        );
        let pack_premultiplied_surface = surface_path == SurfacePath::PackedPremultiplied;
        let direct_surface = surface_path == SurfacePath::Direct;
        if pack_premultiplied_surface {
            debug_assert_eq!(
                self.format,
                canvas.surface().render_format().into_wgpu(),
                "popup pack path must render its scene into an sRGB source format"
            );
            log::debug!(
                target: "wgpu_l3::native_popup",
                "native popup present path PackedPremultipliedSrgbForWindows: scene_format={:?}, surface_format={:?}, alpha_mode={:?}",
                self.format,
                surface_format,
                canvas.composite_alpha_mode()
            );
        }
        let filter_target = if direct_surface {
            render::filter::Target::from_viewport(main_viewport)
        } else {
            self.filter_renderer.prepare(render_context, canvas)
        };
        let filter_renderer = &self.filter_renderer;
        let popup_packer = &self.popup_packer;
        let retained_shapes = (!layer_states.is_empty()).then(|| self.retained.shapes());
        let text_renderer = &mut self.text_renderer;
        let mut text_render_error = None;
        let mut draw_passes = 0;
        let mut command_stats = CommandStats::default();

        let surface_report = if pack_premultiplied_surface {
            canvas.draw(render_context, |encoder, frame| {
                draw_passes += 1;
                let view = frame.create_view();
                filter_renderer.clear_composition(encoder, clear_color);
                let Some(composition_view) = filter_renderer.composition_view() else {
                    return;
                };
                let mut scene_encoder = PlanEncoder::new(PlanEncoderInput {
                    render_context,
                    filter_renderer,
                    text_renderer,
                    retained_shapes,
                    layer_states,
                    layer_index: None,
                    encoder,
                    base_view: composition_view,
                    backdrop_view: composition_view,
                    backdrop_target: render::filter::Target::from_viewport(main_viewport),
                    clear_color,
                    viewport: main_viewport,
                    base_dirty: true,
                    inside_group: false,
                    text_render_error: &mut text_render_error,
                    draw_passes: &mut draw_passes,
                    command_stats: &mut command_stats,
                });
                scene_encoder.encode(batches);
                if text_render_error.is_some() {
                    return;
                }

                popup_packer.pack_to_view(
                    render_context,
                    encoder,
                    composition_view,
                    &view,
                    surface_format.into_wgpu(),
                );
                command_stats.record_draws(1, true);
                draw_passes += 1;
            })?
        } else if direct_surface {
            canvas.draw(render_context, |encoder, frame| {
                draw_passes += 1;
                let view = frame.create_view();
                {
                    let _clear = begin_main_pass(encoder, &view, clear_color, true);
                }
                let mut scene_encoder = PlanEncoder::new(PlanEncoderInput {
                    render_context,
                    filter_renderer,
                    text_renderer,
                    retained_shapes,
                    layer_states,
                    layer_index: None,
                    encoder,
                    base_view: &view,
                    backdrop_view: &view,
                    backdrop_target: filter_target,
                    clear_color,
                    viewport: main_viewport,
                    base_dirty: true,
                    inside_group: false,
                    text_render_error: &mut text_render_error,
                    draw_passes: &mut draw_passes,
                    command_stats: &mut command_stats,
                });
                scene_encoder.encode(batches);
            })?
        } else {
            canvas.draw(render_context, |encoder, frame| {
                draw_passes += 1;
                let view = frame.create_view();
                filter_renderer.clear_composition(encoder, clear_color);
                let Some(composition_view) = filter_renderer.composition_view() else {
                    return;
                };
                let mut scene_encoder = PlanEncoder::new(PlanEncoderInput {
                    render_context,
                    filter_renderer,
                    text_renderer,
                    retained_shapes,
                    layer_states,
                    layer_index: None,
                    encoder,
                    base_view: composition_view,
                    backdrop_view: composition_view,
                    backdrop_target: render::filter::Target::from_viewport(main_viewport),
                    clear_color,
                    viewport: main_viewport,
                    base_dirty: true,
                    inside_group: false,
                    text_render_error: &mut text_render_error,
                    draw_passes: &mut draw_passes,
                    command_stats: &mut command_stats,
                });
                scene_encoder.encode(batches);
                if text_render_error.is_some() {
                    return;
                }

                filter_renderer.blit_to_view(render_context, encoder, &view, filter_target);
                command_stats.record_draws(1, true);
                draw_passes += 1;
            })?
        };

        if let Some(error) = text_render_error {
            return Err(error.into());
        }

        self.text_renderer.trim()?;
        stats.draw_calls = command_stats.draw_calls;
        stats.draw_passes = draw_passes;
        stats.resource_transition_boundaries = draw_passes.saturating_sub(1);
        stats.pipeline_changes = command_stats.pipeline_changes;
        stats.bind_group_changes = command_stats.bind_group_changes;
        stats.property_upload_bytes = stats
            .property_upload_bytes
            .saturating_add(command_stats.property_upload_bytes);
        stats.scroll_layer_cache_hits = command_stats.scroll_layer_cache_hits;
        stats.scroll_layer_cache_misses = command_stats.scroll_layer_cache_misses;
        stats.effect_intermediate_clears = command_stats.effect_intermediate_clears;
        stats.effect_intermediate_clear_bytes = command_stats.effect_intermediate_clear_bytes;
        stats.effect_intermediate_composites = command_stats.effect_intermediate_composites;
        stats.effect_intermediate_composite_bytes =
            command_stats.effect_intermediate_composite_bytes;
        stats.largest_effect_intermediate_bytes = command_stats.largest_effect_intermediate_bytes;
        stats.filter_layer_pool_entries = self.filter_renderer.layer_pool_entries();
        stats.filter_scratch_pool_entries = self.filter_renderer.scratch_pool_entries();
        if surface_report.present_timing().is_some() {
            let surface_bytes = u64::from(canvas.physical_area().width())
                .saturating_mul(u64::from(canvas.physical_area().height()))
                .saturating_mul(4);
            record_surface_traffic(&mut stats, surface_path, surface_bytes);
        }

        Ok(DrawReport {
            stats,
            batch_prepare,
            acquire_outcome: surface_report.acquire(),
            present_timing: surface_report.present_timing(),
        })
    }

    #[cfg(feature = "renderer-debug")]
    pub(super) fn draw_commit_offscreen_debug(
        &mut self,
        render_context: &render::Context,
        commit: &std::sync::Arc<crate::scene::Commit>,
        properties: &crate::scene::Properties,
        width: u32,
        height: u32,
        scale_factor: f32,
        pack_popup: bool,
    ) -> Result<(Vec<[f32; 4]>, DrawStats), String> {
        let (pixels, stats, _) = self.draw_commit_offscreen_debug_timed(
            render_context,
            commit,
            properties,
            width,
            height,
            scale_factor,
            pack_popup,
        )?;
        Ok((pixels, stats))
    }

    #[cfg(feature = "renderer-debug")]
    pub(super) fn draw_commit_offscreen_debug_timed(
        &mut self,
        render_context: &render::Context,
        commit: &std::sync::Arc<crate::scene::Commit>,
        properties: &crate::scene::Properties,
        width: u32,
        height: u32,
        scale_factor: f32,
        pack_popup: bool,
    ) -> Result<(Vec<[f32; 4]>, DrawStats, std::time::Duration), String> {
        let format = self.format;
        if pack_popup && format != wgpu::TextureFormat::Rgba8UnormSrgb {
            return Err("popup debug packing requires an sRGB renderer".to_owned());
        }
        let texture = render_context
            .device()
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Retained Renderer Debug Target"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let packed_texture = pack_popup.then(|| {
            render_context
                .device()
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("Retained Renderer Debug Packed Popup Target"),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                })
        });
        let packed_view = packed_texture
            .as_ref()
            .map(|texture| texture.create_view(&wgpu::TextureViewDescriptor::default()));
        let viewport = render::Viewport::from_logical_area(
            area::logical(width as f32 / scale_factor, height as f32 / scale_factor),
            scale_factor,
        );
        self.filter_renderer.prepare_target(
            render_context,
            render::filter::Target::from_viewport(viewport),
        );
        let prepare_started = std::time::Instant::now();
        let prepared = self
            .retained
            .prepare(
                render_context,
                viewport,
                commit,
                properties,
                &mut self.text_renderer,
            )
            .map_err(|error| error.to_string())?;
        let batch_prepare = prepare_started.elapsed();
        let render::retained::Prepared {
            plan,
            properties: bindings,
            stats: prepared_stats,
        } = prepared;
        let layer_states = [LayerState {
            bindings,
            properties,
        }];
        let clear_color = render::surface_color(commit.clear());
        let mut text_render_error = None;
        let mut draw_passes = 1;
        let mut command_stats = CommandStats::default();
        let mut encoder =
            render_context
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Retained Renderer Debug Encoder"),
                });
        {
            let _clear = begin_main_pass(&mut encoder, &view, clear_color, true);
        }
        {
            let mut scene_encoder = PlanEncoder::new(PlanEncoderInput {
                render_context,
                filter_renderer: &self.filter_renderer,
                text_renderer: &mut self.text_renderer,
                retained_shapes: Some(self.retained.shapes()),
                layer_states: &layer_states,
                layer_index: Some(0),
                encoder: &mut encoder,
                base_view: &view,
                backdrop_view: &view,
                backdrop_target: render::filter::Target::from_viewport(viewport),
                clear_color,
                viewport,
                base_dirty: true,
                inside_group: false,
                text_render_error: &mut text_render_error,
                draw_passes: &mut draw_passes,
                command_stats: &mut command_stats,
            });
            scene_encoder.encode(plan.batches());
        }
        if let Some(error) = text_render_error {
            return Err(error.to_string());
        }
        if let Some(packed_view) = &packed_view {
            self.popup_packer.pack_to_view(
                render_context,
                &mut encoder,
                &view,
                packed_view,
                wgpu::TextureFormat::Rgba8Unorm,
            );
            draw_passes += 1;
        }

        let readback_texture = packed_texture.as_ref().unwrap_or(&texture);
        let pixels = read_texture(
            render_context,
            encoder,
            readback_texture,
            width,
            height,
            "Retained Renderer Debug Readback",
        )?;
        self.text_renderer
            .trim()
            .map_err(|error| error.to_string())?;
        let mut stats = prepared_stats;
        stats.draw_calls = command_stats.draw_calls + usize::from(pack_popup);
        stats.draw_passes = draw_passes;
        stats.resource_transition_boundaries = draw_passes.saturating_sub(1);
        stats.pipeline_changes = command_stats.pipeline_changes + usize::from(pack_popup);
        stats.bind_group_changes = command_stats.bind_group_changes + usize::from(pack_popup);
        stats.property_upload_bytes = stats
            .property_upload_bytes
            .saturating_add(command_stats.property_upload_bytes);
        stats.scroll_layer_cache_hits = command_stats.scroll_layer_cache_hits;
        stats.scroll_layer_cache_misses = command_stats.scroll_layer_cache_misses;
        stats.effect_intermediate_clears = command_stats.effect_intermediate_clears;
        stats.effect_intermediate_clear_bytes = command_stats.effect_intermediate_clear_bytes;
        stats.effect_intermediate_composites = command_stats.effect_intermediate_composites;
        stats.effect_intermediate_composite_bytes =
            command_stats.effect_intermediate_composite_bytes;
        stats.largest_effect_intermediate_bytes = command_stats.largest_effect_intermediate_bytes;
        Ok((pixels, stats, batch_prepare))
    }

    #[cfg(feature = "renderer-debug")]
    pub(super) fn synchronize_commit_offscreen_debug(
        &mut self,
        render_context: &render::Context,
        commit: &std::sync::Arc<crate::scene::Commit>,
        properties: &crate::scene::Properties,
        width: u32,
        height: u32,
        scale_factor: f32,
        budget: std::time::Duration,
    ) -> Result<CommitReadiness, String> {
        properties
            .require_compatible(commit)
            .map_err(|error| error.to_string())?;
        let viewport = render::Viewport::from_logical_area(
            area::logical(width as f32 / scale_factor, height as f32 / scale_factor),
            scale_factor,
        );
        self.synchronize_commit_for_viewport(
            render_context,
            viewport,
            commit,
            properties.serial(),
            budget,
            std::time::Duration::MAX,
        )
        .map_err(|error| error.to_string())
    }

    #[cfg(feature = "renderer-debug")]
    pub(super) fn advance_candidate_after_present_offscreen_debug(
        &mut self,
        render_context: &render::Context,
        commit: &std::sync::Arc<crate::scene::Commit>,
        properties: &crate::scene::Properties,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> Result<(), String> {
        properties
            .require_compatible(commit)
            .map_err(|error| error.to_string())?;
        let viewport = render::Viewport::from_logical_area(
            area::logical(width as f32 / scale_factor, height as f32 / scale_factor),
            scale_factor,
        );
        self.advance_candidate_for_viewport_after_present(
            render_context,
            viewport,
            commit,
            properties,
            std::time::Duration::MAX,
        )
        .map_err(|error| error.to_string())
    }

    #[cfg(feature = "renderer-debug")]
    pub(super) fn cancel_commit_synchronization_debug(
        &mut self,
        commit: &std::sync::Arc<crate::scene::Commit>,
    ) {
        self.retained.cancel_synchronization(commit);
        self.text_renderer.cancel_retained_transform_state(commit);
        self.ready_debug_commits.retain(|candidate| {
            candidate
                .commit
                .upgrade()
                .is_some_and(|candidate| !std::sync::Arc::ptr_eq(&candidate, commit))
        });
    }

    #[cfg(feature = "renderer-debug")]
    pub(super) fn retained_state_counts_debug(&self) -> (usize, usize) {
        self.retained.debug_state_counts()
    }

    #[cfg(feature = "renderer-debug")]
    pub(super) fn retained_plan_signature_debug(
        &self,
        commit: &std::sync::Arc<crate::scene::Commit>,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> Option<String> {
        let viewport = render::Viewport::from_logical_area(
            area::logical(width as f32 / scale_factor, height as f32 / scale_factor),
            scale_factor,
        );
        self.retained.debug_plan_signature(commit, viewport)
    }

    #[cfg(feature = "renderer-debug")]
    pub(super) fn retained_resource_state_debug(&self) -> (usize, usize) {
        let (shape_count, shape_bytes) = self.retained.debug_resource_state();
        (
            shape_count.saturating_add(self.text_renderer.retained_resource_count()),
            shape_bytes.saturating_add(self.text_renderer.retained_resource_bytes()),
        )
    }
}

fn validate_retained_layer(
    kind: &'static str,
    logical_area: area::Logical,
    scale_factor: f32,
    limit: u32,
) -> render::Result<()> {
    let physical = logical_area.to_physical(scale_factor).clamp_min(1);
    if physical.width() > limit || physical.height() > limit {
        return Err(render::Error::RetainedLayerLimit {
            kind,
            width: physical.width(),
            height: physical.height(),
            limit,
        });
    }
    Ok(())
}

fn validate_retained_layers(
    batches: &[PlanStep],
    scale_factor: f32,
    limit: u32,
) -> render::Result<()> {
    for batch in batches {
        match batch {
            PlanStep::Layer(layer) => {
                if layer.force_group || layer.opacity < 1.0 {
                    validate_retained_layer("layer", layer.bounds.area, scale_factor, limit)?;
                }
                validate_retained_layers(&layer.render_batches, scale_factor, limit)?;
            }
            PlanStep::Group(group) => {
                validate_retained_layer("group", group.bounds.area, scale_factor, limit)?;
                validate_retained_layers(&group.render_batches, scale_factor, limit)?;
            }
            PlanStep::Scroll(scroll) => {
                validate_retained_layers(&scroll.render_batches, scale_factor, limit)?;
            }
            PlanStep::Shapes(_)
            | PlanStep::Pane(_)
            | PlanStep::Text(_)
            | PlanStep::PushClip(_)
            | PlanStep::PopClip => {}
        }
    }
    Ok(())
}

impl DrawStats {
    pub(in crate::render) fn add(&mut self, other: Self) {
        self.scene_items += other.scene_items;
        self.render_batches += other.render_batches;
        self.glyph_batches += other.glyph_batches;
        self.text_surfaces += other.text_surfaces;
        self.inline_text_cache_hits += other.inline_text_cache_hits;
        self.inline_text_cache_misses += other.inline_text_cache_misses;
        self.inline_text_shape_calls += other.inline_text_shape_calls;
        self.inline_icon_cache_hits += other.inline_icon_cache_hits;
        self.inline_icon_cache_misses += other.inline_icon_cache_misses;
        self.inline_icon_shape_calls += other.inline_icon_shape_calls;
        self.scene_node_realization_rebuilds += other.scene_node_realization_rebuilds;
        self.quad_prepare_calls += other.quad_prepare_calls;
        self.quad_instances += other.quad_instances;
        self.text_prepare_calls += other.text_prepare_calls;
        self.quad_vertices += other.quad_vertices;
        self.geometry_upload_bytes += other.geometry_upload_bytes;
        self.content_upload_bytes += other.content_upload_bytes;
        self.property_upload_bytes += other.property_upload_bytes;
        self.viewport_property_upload_bytes += other.viewport_property_upload_bytes;
        self.node_property_upload_bytes += other.node_property_upload_bytes;
        self.scroll_property_upload_bytes += other.scroll_property_upload_bytes;
        self.text_property_upload_bytes += other.text_property_upload_bytes;
        self.geometry_buffer_creations += other.geometry_buffer_creations;
        self.retained_gpu_resource_count += other.retained_gpu_resource_count;
        self.retained_gpu_resource_bytes += other.retained_gpu_resource_bytes;
        self.retained_gpu_resource_creations += other.retained_gpu_resource_creations;
        self.retained_gpu_resource_replacements += other.retained_gpu_resource_replacements;
        self.retained_gpu_resource_removals += other.retained_gpu_resource_removals;
        self.commit_preparation_slices += other.commit_preparation_slices;
        self.commit_preparation_total_nanos = self
            .commit_preparation_total_nanos
            .saturating_add(other.commit_preparation_total_nanos);
        self.commit_preparation_max_nanos = self
            .commit_preparation_max_nanos
            .max(other.commit_preparation_max_nanos);
        self.commit_preparation_deadline_misses += other.commit_preparation_deadline_misses;
        self.render_plan_rebuilds += other.render_plan_rebuilds;
        self.render_plan_reuses += other.render_plan_reuses;
        self.direct_surface_plans += other.direct_surface_plans;
        self.surface_sampling_plans += other.surface_sampling_plans;
        self.draw_calls += other.draw_calls;
        self.draw_passes += other.draw_passes;
        self.explicit_copy_commands += other.explicit_copy_commands;
        self.resource_transition_boundaries += other.resource_transition_boundaries;
        self.pipeline_changes += other.pipeline_changes;
        self.bind_group_changes += other.bind_group_changes;
        self.scroll_layer_cache_hits += other.scroll_layer_cache_hits;
        self.scroll_layer_cache_misses += other.scroll_layer_cache_misses;
        self.clip_batches += other.clip_batches;
        self.opaque_nodes += other.opaque_nodes;
        self.blended_nodes += other.blended_nodes;
        self.opacity_unclassified_nodes += other.opacity_unclassified_nodes;
        self.clipped_nodes += other.clipped_nodes;
        self.effect_island_nodes += other.effect_island_nodes;
        self.effect_intermediate_clears += other.effect_intermediate_clears;
        self.effect_intermediate_clear_bytes += other.effect_intermediate_clear_bytes;
        self.effect_intermediate_composites += other.effect_intermediate_composites;
        self.effect_intermediate_composite_bytes += other.effect_intermediate_composite_bytes;
        self.largest_effect_intermediate_bytes = self
            .largest_effect_intermediate_bytes
            .max(other.largest_effect_intermediate_bytes);
        self.culled_nodes += other.culled_nodes;
        self.group_composites += other.group_composites;
        self.filter_layer_pool_entries += other.filter_layer_pool_entries;
        self.filter_scratch_pool_entries += other.filter_scratch_pool_entries;
        self.ordinary_surface_clears += other.ordinary_surface_clears;
        self.ordinary_surface_clear_bytes += other.ordinary_surface_clear_bytes;
        self.extra_full_surface_intermediate_clears += other.extra_full_surface_intermediate_clears;
        self.extra_full_surface_intermediate_clear_bytes +=
            other.extra_full_surface_intermediate_clear_bytes;
        self.full_surface_blits += other.full_surface_blits;
        self.full_surface_blit_bytes += other.full_surface_blit_bytes;
        self.popup_surface_packs += other.popup_surface_packs;
        self.popup_surface_pack_bytes += other.popup_surface_pack_bytes;
    }
}

fn begin_main_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    view: &'a wgpu::TextureView,
    clear_color: wgpu::Color,
    clear: bool,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Main Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: if clear {
                    wgpu::LoadOp::Clear(clear_color)
                } else {
                    wgpu::LoadOp::Load
                },
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    })
}

#[cfg(feature = "renderer-debug")]
fn read_texture(
    render_context: &render::Context,
    mut encoder: wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
    label: &'static str,
) -> Result<Vec<[f32; 4]>, String> {
    let bytes_per_pixel = 4u32;
    let unpadded_bytes_per_row = width * bytes_per_pixel;
    let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(alignment) * alignment;
    let output = render_context
        .device()
        .create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: padded_bytes_per_row as u64 * height as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    render_context.queue().submit(Some(encoder.finish()));

    let slice = output.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    render_context
        .device()
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .map_err(|error| error.to_string())?;
    receiver
        .recv()
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;

    let data = slice.get_mapped_range();
    let mut pixels = Vec::with_capacity((width * height) as usize);
    for y in 0..height as usize {
        for x in 0..width as usize {
            let offset = y * padded_bytes_per_row as usize + x * bytes_per_pixel as usize;
            pixels.push([
                data[offset] as f32 / 255.0,
                data[offset + 1] as f32 / 255.0,
                data[offset + 2] as f32 / 255.0,
                data[offset + 3] as f32 / 255.0,
            ]);
        }
    }
    drop(data);
    output.unmap();
    Ok(pixels)
}

fn is_draw_batch(batch: &PlanStep) -> bool {
    matches!(batch, PlanStep::Shapes(_) | PlanStep::Text(_))
}

fn surface_path(
    pack_premultiplied_surface: bool,
    preserve_surface_alpha: bool,
    direct_surface_eligible: bool,
) -> SurfacePath {
    if pack_premultiplied_surface {
        SurfacePath::PackedPremultiplied
    } else if preserve_surface_alpha || direct_surface_eligible {
        SurfacePath::Direct
    } else {
        SurfacePath::SampledComposition
    }
}

fn record_surface_traffic(stats: &mut DrawStats, path: SurfacePath, surface_bytes: u64) {
    match path {
        SurfacePath::PackedPremultiplied => {
            stats.extra_full_surface_intermediate_clears = 1;
            stats.extra_full_surface_intermediate_clear_bytes = surface_bytes;
            stats.popup_surface_packs = 1;
            stats.popup_surface_pack_bytes = surface_bytes.saturating_mul(2);
        }
        SurfacePath::Direct => {
            stats.ordinary_surface_clears = 1;
            stats.ordinary_surface_clear_bytes = surface_bytes;
        }
        SurfacePath::SampledComposition => {
            stats.extra_full_surface_intermediate_clears = 1;
            stats.extra_full_surface_intermediate_clear_bytes = surface_bytes;
            stats.full_surface_blits = 1;
            stats.full_surface_blit_bytes = surface_bytes.saturating_mul(2);
        }
    }
}

pub(in crate::render) fn requires_surface_sampling(batches: &[PlanStep]) -> bool {
    batches.iter().any(|batch| match batch {
        PlanStep::Pane(prepared) => match &prepared.pane.material {
            paint::Material::Solid(_) => false,
            paint::Material::Glass(glass) => {
                (glass.base == paint::GlassBase::FrameworkBackdrop
                    && !render::material::backdrop_filter(&prepared.pane, glass)
                        .ops
                        .is_empty())
                    || glass.surface_layers.iter().any(|layer| {
                        matches!(
                            layer,
                            paint::SurfaceLayer::Noise(noise) if noise.opacity > 0.0
                        )
                    })
            }
        },
        PlanStep::Layer(layer) => requires_surface_sampling(&layer.render_batches),
        PlanStep::Group(group) => requires_surface_sampling(&group.render_batches),
        PlanStep::Scroll(scroll) => requires_surface_sampling(&scroll.render_batches),
        PlanStep::Shapes(_) | PlanStep::Text(_) | PlanStep::PushClip(_) | PlanStep::PopClip => {
            false
        }
    })
}

fn draw_run_end(render_batches: &[PlanStep], start: usize) -> usize {
    render_batches[start..]
        .iter()
        .position(|batch| !is_draw_batch(batch))
        .map_or(render_batches.len(), |offset| start + offset)
}

impl<'a> PlanEncoder<'a> {
    fn new(input: PlanEncoderInput<'a>) -> Self {
        Self {
            render_context: input.render_context,
            filter_renderer: input.filter_renderer,
            text_renderer: input.text_renderer,
            retained_shapes: input.retained_shapes,
            layer_states: input.layer_states,
            layer_index: input.layer_index,
            output_target: render::filter::Target::from_viewport(input.viewport),
            encoder: input.encoder,
            base_view: input.base_view,
            backdrop_view: input.backdrop_view,
            backdrop_target: input.backdrop_target,
            clear_color: input.clear_color,
            viewport: input.viewport,
            layers: Vec::new(),
            clip_stack: Vec::new(),
            base_dirty: input.base_dirty,
            inside_group: input.inside_group,
            text_render_error: input.text_render_error,
            draw_passes: input.draw_passes,
            command_stats: input.command_stats,
            scroll_translation: [0.0, 0.0],
            scroll_clip: None,
        }
    }

    fn encode(&mut self, render_batches: &[PlanStep]) {
        self.encode_batches(render_batches);

        while !self.clip_stack.is_empty() {
            self.composite_clip_layer();
        }

        for layer in self.layers.drain(..) {
            self.filter_renderer.recycle_layer(layer);
        }
    }

    fn current_properties(&self) -> Option<&crate::scene::Properties> {
        self.layer_index
            .and_then(|index| self.layer_states.get(index))
            .map(|state| state.properties)
    }

    fn encode_batches(&mut self, render_batches: &[PlanStep]) {
        let mut index = 0;
        while index < render_batches.len() {
            if is_draw_batch(&render_batches[index]) {
                let end = draw_run_end(render_batches, index);
                self.encode_draw_run(&render_batches[index..end]);
                index = end;
                continue;
            }

            match &render_batches[index] {
                PlanStep::Shapes(_) | PlanStep::Text(_) => unreachable!(),
                PlanStep::Layer(layer) => self.encode_layer(layer),
                PlanStep::Pane(pane) => self.encode_pane(pane),
                PlanStep::PushClip(clip) => self.push_clip(self.resolve_clip(*clip)),
                PlanStep::PopClip => self.composite_clip_layer(),
                PlanStep::Group(group) => self.encode_group(group),
                PlanStep::Scroll(scroll) => self.encode_scroll(scroll),
            }
            index += 1;
        }
    }

    fn encode_layer(&mut self, layer: &PreparedLayer) {
        let previous = self.layer_index.replace(layer.state_index);
        if layer.force_group || layer.opacity < 1.0 {
            self.encode_group_batches(layer.bounds, layer.opacity, &layer.render_batches);
        } else {
            self.encode_batches(&layer.render_batches);
        }
        self.layer_index = previous;
    }

    fn encode_draw_run(&mut self, batches: &[PlanStep]) {
        let retained_shapes = self.retained_shapes;
        let current_bindings = self
            .layer_index
            .and_then(|index| self.layer_states.get(index))
            .map(|state| &state.bindings);
        let Some(scissor) = current_scissor(
            &self.clip_stack,
            self.viewport.physical_area(),
            self.viewport.scale_factor(),
        ) else {
            return;
        };
        let Some(target_view) = current_target_view(self.base_view, &self.layers, &self.clip_stack)
        else {
            return;
        };
        *self.draw_passes += 1;
        {
            let mut pass = begin_main_pass(self.encoder, target_view, self.clear_color, false);
            pass.set_scissor_rect(scissor.x(), scissor.y(), scissor.width(), scissor.height());

            for batch in batches {
                match batch {
                    PlanStep::Shapes(batch) => {
                        if let (Some(shapes), Some(properties)) =
                            (retained_shapes, current_bindings)
                        {
                            shapes.draw(&mut pass, batch, properties);
                            self.command_stats.record_draws(1, true);
                        }
                    }
                    PlanStep::Text(batch) => {
                        match self.text_renderer.render_retained(
                            *batch,
                            self.scroll_translation,
                            self.viewport.scale_factor(),
                            &mut pass,
                        ) {
                            Ok(()) => {}
                            Err(error) => {
                                *self.text_render_error = Some(error);
                                break;
                            }
                        }
                        self.command_stats.record_draws(1, true);
                    }
                    PlanStep::Layer(_)
                    | PlanStep::Pane(_)
                    | PlanStep::PushClip(_)
                    | PlanStep::PopClip
                    | PlanStep::Group(_)
                    | PlanStep::Scroll(_) => unreachable!(),
                }
            }
        }
        self.mark_current_dirty();
    }

    fn encode_shapes(&mut self, batch: &render::retained::ShapeBatch) {
        let retained_shapes = self.retained_shapes;
        let current_bindings = self
            .layer_index
            .and_then(|index| self.layer_states.get(index))
            .map(|state| &state.bindings);
        let Some(scissor) = current_scissor(
            &self.clip_stack,
            self.viewport.physical_area(),
            self.viewport.scale_factor(),
        ) else {
            return;
        };
        let Some(target_view) = current_target_view(self.base_view, &self.layers, &self.clip_stack)
        else {
            return;
        };
        *self.draw_passes += 1;
        {
            let mut pass = begin_main_pass(self.encoder, target_view, self.clear_color, false);
            pass.set_scissor_rect(scissor.x(), scissor.y(), scissor.width(), scissor.height());
            if let (Some(shapes), Some(properties)) = (retained_shapes, current_bindings) {
                shapes.draw(&mut pass, batch, properties);
                self.command_stats.record_draws(1, true);
            }
        }
        self.mark_current_dirty();
    }

    fn encode_pane(&mut self, prepared: &PreparedPane) {
        let mut pane = prepared.pane.clone();
        pane.rect = translated_rect(pane.rect, self.scroll_translation);
        pane.source_rect = pane
            .source_rect
            .map(|source| translated_rect(source, self.scroll_translation));
        let pane = &pane;
        let Some(scissor) = current_scissor(
            &self.clip_stack,
            self.viewport.physical_area(),
            self.viewport.scale_factor(),
        ) else {
            return;
        };

        match &pane.material {
            paint::Material::Solid(_) => {
                if let Some(batch) = prepared.base.as_ref() {
                    self.encode_shapes(batch);
                }
            }
            paint::Material::Glass(glass) => {
                log::debug!(
                    target: "wgpu_l3::render::material",
                    "pane glass rect={:?} source_rect={:?} group={} target_area={:?} opacity_context=entry layers={:?}",
                    pane.rect,
                    pane.source_rect,
                    self.inside_group,
                    self.output_target.physical_area(),
                    render::material::layer_sequence(glass),
                );

                match glass.base {
                    paint::GlassBase::Fallback => {
                        if let Some(batch) = prepared.base.as_ref() {
                            self.encode_shapes(batch);
                        }
                    }
                    paint::GlassBase::Transparent => {}
                    paint::GlassBase::FrameworkBackdrop => {
                        let filter = render::material::backdrop_filter(pane, glass);
                        if !filter.ops.is_empty() {
                            let Some(output) =
                                current_target_view(self.base_view, &self.layers, &self.clip_stack)
                            else {
                                return;
                            };
                            let source = render::material::backdrop_source(
                                self.inside_group,
                                self.current_target_dirty(),
                                pane,
                                output,
                                self.output_target,
                                self.backdrop_view,
                                self.backdrop_target,
                            );
                            let scratch = pane_filter_scratch(pane, self.viewport);
                            let filter_draws = filter_draw_calls(&filter);
                            let work = self.filter_renderer.draw(render::filter::FilterDraw {
                                render_context: self.render_context,
                                encoder: self.encoder,
                                target: self.output_target,
                                scratch,
                                owner: render::filter::FilterOwner::PaneBackdrop,
                                source,
                                output,
                                filter,
                                scissor: Some(scissor),
                            });
                            self.command_stats.record_filter_work(work);
                            *self.draw_passes +=
                                work.intermediate_clears + work.intermediate_composites;
                            self.command_stats.record_draws(filter_draws, true);
                            self.mark_current_dirty();
                        }
                    }
                }

                for (layer, prepared_layer) in
                    glass.surface_layers.iter().zip(&prepared.surface_layers)
                {
                    match *layer {
                        paint::SurfaceLayer::Tint { .. } => {
                            if let Some(batch) = prepared_layer.as_ref() {
                                self.encode_shapes(batch);
                            }
                        }
                        paint::SurfaceLayer::Noise(noise) => {
                            if noise.opacity <= 0.0 {
                                continue;
                            }

                            let Some(output) =
                                current_target_view(self.base_view, &self.layers, &self.clip_stack)
                            else {
                                return;
                            };
                            let filter =
                                paint::Filter::stack(pane.rect, [paint::FilterOp::noise(noise)]);
                            let scratch = pane_filter_scratch(pane, self.viewport);
                            let filter_draws = filter_draw_calls(&filter);
                            let work = self.filter_renderer.draw(render::filter::FilterDraw {
                                render_context: self.render_context,
                                encoder: self.encoder,
                                target: self.output_target,
                                scratch,
                                owner: render::filter::FilterOwner::PaneSurfaceNoise,
                                source: render::filter::FilterSource::Local {
                                    texture: render::filter::TextureSource::new(
                                        output,
                                        self.output_target.physical_area(),
                                        self.output_target.logical_area(),
                                        paint::LayerSampling::PixelAligned,
                                    ),
                                    local_rect: pane.rect,
                                },
                                output,
                                filter,
                                scissor: Some(scissor),
                            });
                            self.command_stats.record_filter_work(work);
                            *self.draw_passes +=
                                work.intermediate_clears + work.intermediate_composites;
                            self.command_stats.record_draws(filter_draws, true);
                            self.mark_current_dirty();
                        }
                    }
                }
            }
        }
    }

    fn encode_group(&mut self, group: &PreparedGroup) {
        let opacity = self.resolve_opacity(group);
        self.encode_group_batches(group.bounds, opacity, &group.render_batches);
    }

    fn encode_group_batches(
        &mut self,
        bounds: paint::Rect,
        opacity: f32,
        render_batches: &[PlanStep],
    ) {
        if opacity <= 0.0 {
            return;
        }

        let bounds = translated_rect(bounds, self.scroll_translation);
        let group_target =
            render::filter::Target::from_logical_area(bounds.area, self.viewport.scale_factor());
        let group_layer = self.filter_renderer.create_layer(
            self.render_context,
            group_target,
            "Group Layer Texture",
        );
        self.filter_renderer.clear_layer(self.encoder, &group_layer);
        *self.draw_passes += 1;

        {
            let group_view = group_layer.view();
            let input = PlanEncoderInput {
                render_context: self.render_context,
                filter_renderer: self.filter_renderer,
                text_renderer: self.text_renderer,
                retained_shapes: self.retained_shapes,
                layer_states: self.layer_states,
                layer_index: self.layer_index,
                encoder: self.encoder,
                base_view: group_view,
                backdrop_view: self.base_view,
                backdrop_target: self.output_target,
                clear_color: wgpu::Color::TRANSPARENT,
                viewport: render::Viewport::from_logical_area(
                    bounds.area,
                    self.viewport.scale_factor(),
                ),
                base_dirty: false,
                inside_group: true,
                text_render_error: self.text_render_error,
                draw_passes: self.draw_passes,
                command_stats: self.command_stats,
            };
            let mut group_encoder = PlanEncoder::new(input);
            group_encoder.encode(render_batches);
        }

        if let Some(output) = current_target_view(self.base_view, &self.layers, &self.clip_stack) {
            self.filter_renderer
                .composite_layer(render::filter::LayerComposite {
                    render_context: self.render_context,
                    encoder: self.encoder,
                    source: &group_layer,
                    output,
                    target: self.output_target,
                    clip: paint::Clip { rect: bounds },
                    source_rect: Some(group_layer_source_rect(bounds)),
                    scissor: current_scissor(
                        &self.clip_stack,
                        self.viewport.physical_area(),
                        self.viewport.scale_factor(),
                    ),
                    opacity,
                });
            self.command_stats.record_draws(1, true);
            *self.draw_passes += 1;
            self.mark_current_dirty();
        }
        self.filter_renderer.recycle_layer(group_layer);
    }

    fn encode_scroll(&mut self, scroll: &PreparedScroll) {
        let parent_translation = self.scroll_translation;
        let parent_clip = self.scroll_clip;
        let current = self
            .current_properties()
            .and_then(|properties| properties.scroll_offset(scroll.node))
            .unwrap_or(scroll.baseline);
        let own_translation = [
            scroll.baseline.x().saturating_sub(current.x()) as f32,
            scroll.baseline.y().saturating_sub(current.y()) as f32,
        ];
        let translation = [
            parent_translation[0] + own_translation[0],
            parent_translation[1] + own_translation[1],
        ];
        let viewport = translated_rect(scroll.viewport, parent_translation);
        let Some(visible) = parent_clip
            .map(|clip| intersect_rect(clip, viewport))
            .unwrap_or(Some(viewport))
        else {
            return;
        };
        self.scroll_translation = translation;
        self.scroll_clip = Some(visible);
        self.push_clip(paint::Clip { rect: visible });
        self.encode_batches(&scroll.render_batches);
        self.composite_clip_layer();
        self.scroll_translation = parent_translation;
        self.scroll_clip = parent_clip;
    }

    fn push_clip(&mut self, clip: paint::Clip) {
        if clip_is_full_target(clip, self.viewport)
            || self
                .clip_stack
                .last()
                .is_some_and(|frame| frame.clip() == Some(clip))
        {
            self.clip_stack.push(ClipFrame::PassThrough);
            return;
        }

        if clip_is_rectangular(clip) {
            self.clip_stack.push(ClipFrame::Scissor(clip));
            return;
        }

        let layer = self.filter_renderer.create_layer(
            self.render_context,
            self.output_target,
            "Clip Layer Texture",
        );
        self.layers.push(layer);
        let layer_index = self.layers.len() - 1;
        self.filter_renderer
            .clear_layer(self.encoder, &self.layers[layer_index]);
        *self.draw_passes += 1;
        self.clip_stack.push(ClipFrame::Layer {
            clip,
            layer_index,
            dirty: false,
        });
    }

    fn resolve_clip(&self, clip: PreparedClip) -> paint::Clip {
        let mut resolved = clip
            .node
            .and_then(|node| {
                let crate::scene::PropertyValue::Clip { rect, .. } = self
                    .current_properties()?
                    .value(crate::scene::PropertyRef::new(
                        node,
                        crate::scene::PropertyKind::Clip,
                    ))?
                else {
                    return None;
                };
                let resolved =
                    render::scene::to_paint_rect_value_at_scale(rect, self.viewport.scale_factor());
                Some(paint::Clip {
                    rect: paint::Rect {
                        origin: resolved.origin,
                        area: resolved.area,
                        rounding: clip.fallback.rect.rounding,
                    },
                })
            })
            .unwrap_or(clip.fallback);
        resolved.rect.origin = crate::geometry::point::logical(
            resolved.rect.origin.x() - clip.scene_origin[0],
            resolved.rect.origin.y() - clip.scene_origin[1],
        );
        resolved.rect = translated_rect(resolved.rect, self.scroll_translation);
        resolved
    }

    fn resolve_opacity(&self, group: &PreparedGroup) -> f32 {
        let Some(node) = group.node else {
            return group.opacity;
        };
        match self.current_properties().and_then(|properties| {
            properties.value(crate::scene::PropertyRef::new(
                node,
                crate::scene::PropertyKind::Opacity,
            ))
        }) {
            Some(crate::scene::PropertyValue::Opacity { value, .. }) => value,
            _ => group.opacity,
        }
    }

    fn composite_clip_layer(&mut self) {
        let Some(frame) = self.clip_stack.pop() else {
            return;
        };
        let ClipFrame::Layer {
            clip, layer_index, ..
        } = frame
        else {
            return;
        };
        let Some(source) = self.layers.get(layer_index) else {
            return;
        };
        let Some(output) = current_target_view(self.base_view, &self.layers, &self.clip_stack)
        else {
            return;
        };

        self.filter_renderer
            .composite_layer(render::filter::LayerComposite {
                render_context: self.render_context,
                encoder: self.encoder,
                source,
                output,
                target: self.output_target,
                clip,
                source_rect: None,
                scissor: clip_scissor(
                    clip.rect,
                    self.viewport.physical_area(),
                    self.viewport.scale_factor(),
                ),
                opacity: 1.0,
            });
        self.command_stats.record_draws(1, true);
        *self.draw_passes += 1;
        self.mark_current_dirty();
    }

    fn current_target_dirty(&self) -> bool {
        self.clip_stack
            .iter()
            .rev()
            .find_map(|frame| match frame {
                ClipFrame::Layer { dirty, .. } => Some(*dirty),
                ClipFrame::PassThrough | ClipFrame::Scissor(_) => None,
            })
            .unwrap_or(self.base_dirty)
    }

    fn mark_current_dirty(&mut self) {
        for frame in self.clip_stack.iter_mut().rev() {
            if let ClipFrame::Layer { dirty, .. } = frame {
                *dirty = true;
                return;
            }
        }
        self.base_dirty = true;
    }
}

fn filter_draw_calls(filter: &paint::Filter) -> usize {
    filter
        .ops
        .iter()
        .map(|operation| match *operation {
            paint::FilterOp::Blur { amount } if amount > 0.0 => 3,
            paint::FilterOp::BackdropBlur(blur) if blur.sigma > 0.0 => 3,
            paint::FilterOp::Refraction(refraction) if refraction.displacement > 0.0 => 2,
            paint::FilterOp::Luminosity(luminosity) if luminosity.opacity > 0.0 => 2,
            paint::FilterOp::Noise(noise) if noise.opacity > 0.0 => 2,
            _ => 0,
        })
        .sum()
}

fn pane_filter_scratch(
    pane: &paint::Pane,
    viewport: render::Viewport,
) -> render::filter::FilterScratch {
    render::filter::FilterScratch::bounded(
        pane.rect,
        paint::pane_effect_bounds(pane, paint::Grid::new(viewport.scale_factor())),
        viewport.scale_factor(),
    )
}

fn group_layer_source_rect(bounds: Rect) -> Rect {
    Rect::new(point::logical(0.0, 0.0), bounds.area)
}

fn translated_rect(mut rect: Rect, translation: [f32; 2]) -> Rect {
    rect.origin = point::logical(
        rect.origin.x() + translation[0],
        rect.origin.y() + translation[1],
    );
    rect
}

fn intersect_rect(left: Rect, right: Rect) -> Option<Rect> {
    let x = left.origin.x().max(right.origin.x());
    let y = left.origin.y().max(right.origin.y());
    let right_edge =
        (left.origin.x() + left.area.width()).min(right.origin.x() + right.area.width());
    let bottom_edge =
        (left.origin.y() + left.area.height()).min(right.origin.y() + right.area.height());
    (right_edge > x && bottom_edge > y).then(|| {
        Rect::new(
            point::logical(x, y),
            area::logical(right_edge - x, bottom_edge - y),
        )
    })
}

fn current_target_view<'a>(
    base_view: &'a wgpu::TextureView,
    layers: &'a [render::filter::Layer],
    clip_stack: &[ClipFrame],
) -> Option<&'a wgpu::TextureView> {
    for frame in clip_stack.iter().rev() {
        if let Some(layer_index) = frame.layer_index() {
            return Some(layers.get(layer_index)?.view());
        }
    }
    Some(base_view)
}

fn current_scissor(
    clips: &[ClipFrame],
    physical_area: area::Physical,
    scale_factor: f32,
) -> Option<render::Scissor> {
    let mut scissor = render::Scissor::new(
        0,
        0,
        physical_area.width().max(1),
        physical_area.height().max(1),
    )?;

    for frame in clips {
        let Some(clip) = frame.clip() else {
            continue;
        };
        scissor = intersect_scissor(
            scissor,
            clip_scissor(clip.rect, physical_area, scale_factor)?,
        )?;
    }

    Some(scissor)
}

fn clip_is_rectangular(clip: paint::Clip) -> bool {
    clip.rect
        .rounding
        .resolve(clip.rect.area)
        .iter()
        .all(|radius| radius.abs() <= f32::EPSILON)
}

fn clip_is_full_target(clip: paint::Clip, viewport: render::Viewport) -> bool {
    if !clip_is_rectangular(clip) {
        return false;
    }
    let area = viewport.logical_area();
    clip.rect.origin.x() <= 0.0
        && clip.rect.origin.y() <= 0.0
        && clip.rect.origin.x() + clip.rect.area.width() >= area.width()
        && clip.rect.origin.y() + clip.rect.area.height() >= area.height()
}

fn clip_scissor(
    rect: Rect,
    physical_area: area::Physical,
    scale_factor: f32,
) -> Option<render::Scissor> {
    let scale_factor = scale_factor.max(0.0001);
    let canvas_width = physical_area.width();
    let canvas_height = physical_area.height();
    let left = physical_clip_edge(rect.origin.x(), scale_factor, f32::floor).max(0.0) as u32;
    let top = physical_clip_edge(rect.origin.y(), scale_factor, f32::floor).max(0.0) as u32;
    let right = physical_clip_edge(rect.origin.x() + rect.area.width(), scale_factor, f32::ceil)
        .max(0.0)
        .min(canvas_width as f32) as u32;
    let bottom = physical_clip_edge(
        rect.origin.y() + rect.area.height(),
        scale_factor,
        f32::ceil,
    )
    .max(0.0)
    .min(canvas_height as f32) as u32;
    let left = left.min(canvas_width);
    let top = top.min(canvas_height);

    render::Scissor::new(
        left,
        top,
        right.saturating_sub(left),
        bottom.saturating_sub(top),
    )
}

fn physical_clip_edge(logical: f32, scale_factor: f32, outward: fn(f32) -> f32) -> f32 {
    let physical = logical * scale_factor;
    let nearest = physical.round();

    if (physical - nearest).abs() <= 0.001 {
        nearest
    } else {
        outward(physical)
    }
}

fn intersect_scissor(a: render::Scissor, b: render::Scissor) -> Option<render::Scissor> {
    let left = a.x().max(b.x());
    let top = a.y().max(b.y());
    let right = (a.x() + a.width()).min(b.x() + b.width());
    let bottom = (a.y() + a.height()).min(b.y() + b.height());

    render::Scissor::new(
        left,
        top,
        right.saturating_sub(left),
        bottom.saturating_sub(top),
    )
}

#[cfg(test)]
mod tests {
    use crate::paint::{self, Rect};
    use crate::render;

    use super::*;

    #[test]
    fn semantic_surface_path_keeps_ordinary_work_direct_and_sampling_work_isolated() {
        assert_eq!(surface_path(false, false, true), SurfacePath::Direct);
        assert_eq!(
            surface_path(false, false, false),
            SurfacePath::SampledComposition
        );
        assert_eq!(
            surface_path(true, false, true),
            SurfacePath::PackedPremultiplied
        );
        assert_eq!(surface_path(false, true, false), SurfacePath::Direct);
    }

    #[test]
    fn direct_surface_traffic_has_one_clear_and_literal_zero_intermediate_work() {
        let mut stats = DrawStats::default();

        record_surface_traffic(&mut stats, SurfacePath::Direct, 8_000_000);

        assert_eq!(stats.ordinary_surface_clears, 1);
        assert_eq!(stats.ordinary_surface_clear_bytes, 8_000_000);
        assert_eq!(stats.extra_full_surface_intermediate_clears, 0);
        assert_eq!(stats.extra_full_surface_intermediate_clear_bytes, 0);
        assert_eq!(stats.full_surface_blits, 0);
        assert_eq!(stats.full_surface_blit_bytes, 0);
        assert_eq!(stats.popup_surface_packs, 0);
    }

    #[test]
    fn filter_command_counter_matches_the_passes_each_live_operation_encodes() {
        let filter = paint::Filter::stack(
            Rect::new(point::logical(0.0, 0.0), area::logical(40.0, 30.0)),
            [
                paint::FilterOp::blur(4.0),
                paint::FilterOp::noise(paint::Noise { opacity: 0.02 }),
                paint::FilterOp::blur(0.0),
            ],
        );

        assert_eq!(filter_draw_calls(&filter), 5);
    }

    #[test]
    fn clip_scissor_converts_logical_rect_to_physical_pixels() {
        let scissor = clip_scissor(
            Rect::new(point::logical(2.25, 3.25), area::logical(10.5, 8.5)),
            area::physical(100, 80),
            2.0,
        )
        .expect("clip should produce scissor");

        assert_eq!(scissor.x(), 4);
        assert_eq!(scissor.y(), 6);
        assert_eq!(scissor.width(), 22);
        assert_eq!(scissor.height(), 18);
    }

    #[test]
    fn clip_scissor_preserves_fractional_scale_grid_edges_after_localization() {
        let scissor = clip_scissor(
            Rect::new(point::logical(0.0, 0.0), area::logical(728.0, 30.400_024)),
            area::physical(910, 57),
            1.25,
        )
        .expect("localized grid-aligned clip should produce a scissor");

        assert_eq!(scissor.x(), 0);
        assert_eq!(scissor.y(), 0);
        assert_eq!(scissor.width(), 910);
        assert_eq!(scissor.height(), 38);
    }

    #[test]
    fn current_scissor_intersects_nested_clip_stack() {
        let clips = vec![
            ClipFrame::Layer {
                clip: paint::Clip {
                    rect: Rect::rounded(
                        point::logical(2.0, 2.0),
                        area::logical(20.0, 20.0),
                        paint::Rounding::relative(0.5),
                    ),
                },
                layer_index: 0,
                dirty: false,
            },
            ClipFrame::Scissor(paint::Clip {
                rect: Rect::new(point::logical(10.0, 0.0), area::logical(20.0, 12.0)),
            }),
        ];
        let scissor = current_scissor(&clips, area::physical(100, 80), 1.0)
            .expect("nested clips should intersect");

        assert_eq!(scissor.x(), 10);
        assert_eq!(scissor.y(), 2);
        assert_eq!(scissor.width(), 12);
        assert_eq!(scissor.height(), 10);
    }

    #[test]
    fn clip_frames_keep_scissors_out_of_the_layer_stack() {
        let mut clips = vec![
            ClipFrame::Layer {
                clip: paint::Clip {
                    rect: Rect::rounded(
                        point::logical(2.0, 2.0),
                        area::logical(20.0, 20.0),
                        paint::Rounding::relative(0.5),
                    ),
                },
                layer_index: 0,
                dirty: false,
            },
            ClipFrame::Scissor(paint::Clip {
                rect: Rect::new(point::logical(10.0, 0.0), area::logical(20.0, 12.0)),
            }),
        ];

        assert_eq!(clips.last().and_then(|frame| frame.layer_index()), None);
        assert_eq!(clips.pop().and_then(ClipFrame::layer_index), None);
        assert_eq!(clips.last().and_then(|frame| frame.layer_index()), Some(0));
    }

    #[test]
    fn rounded_clip_keeps_mask_geometry_and_a_bounding_scissor() {
        let clip = paint::Clip {
            rect: Rect::rounded(
                point::logical(4.0, 4.0),
                area::logical(18.0, 12.0),
                paint::Rounding::relative(1.0),
            ),
        };
        let scissor = clip_scissor(clip.rect, area::physical(100, 80), 1.0)
            .expect("clip should produce scissor");

        assert_eq!(clip.rect.rounding.resolve(clip.rect.area)[0], 6.0);
        assert!(!clip_is_rectangular(clip));
        assert_eq!(scissor.x(), 4);
        assert_eq!(scissor.y(), 4);
        assert_eq!(scissor.width(), 18);
        assert_eq!(scissor.height(), 12);
    }

    #[test]
    fn group_layer_source_rect_is_local_texture_space() {
        let bounds = Rect::new(point::logical(240.0, 120.0), area::logical(160.0, 80.0));

        let source = group_layer_source_rect(bounds);

        assert_eq!(source.origin, point::logical(0.0, 0.0));
        assert_eq!(source.area, bounds.area);
    }

    #[test]
    fn retained_layer_limit_rejects_an_oversized_group_effect_texture() {
        let error = validate_retained_layer("group", area::logical(760.0, 16_862.0), 1.0, 8_192)
            .expect_err("an oversized semantic offscreen must be rejected before WGPU creation");

        assert!(matches!(
            error,
            render::Error::RetainedLayerLimit {
                kind: "group",
                width: 760,
                height: 16_862,
                limit: 8_192,
            }
        ));
    }

    #[test]
    fn group_filter_encoding_is_local_but_group_composite_uses_parent_scroll_space() {
        let source = std::fs::read_to_string(file!()).expect("renderer source should be readable");
        let encode_group = source
            .split("fn encode_group_batches")
            .nth(1)
            .expect("encode_group_batches should exist")
            .split("fn push_clip")
            .next()
            .expect("encode_group_batches should precede push_clip");

        assert!(source.contains("self.encode_group_batches(group.bounds, opacity"));
        assert!(encode_group.contains("translated_rect(bounds, self.scroll_translation)"));
        assert!(encode_group.contains("Target::from_logical_area(bounds.area"));
        assert!(encode_group.contains("base_view: group_view"));
        assert!(encode_group.contains("viewport: render::Viewport::from_logical_area"));
        assert!(encode_group.contains("target: self.output_target"));
        assert!(encode_group.contains("source_rect: Some(group_layer_source_rect(bounds))"));
    }
}
