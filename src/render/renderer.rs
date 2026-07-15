use crate::geometry::{area, point};
use crate::paint;
use crate::paint::Rect;
use crate::render;
use crate::render::DrawStats;
use crate::render::batch::{ItemBatch, item_batches};

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
    Prepared,
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
    quad_pipeline: wgpu::RenderPipeline,
    quad_arena: render::quad::Arena,
    filter_renderer: render::filter::Renderer,
    popup_packer: render::popup_pack::Packer,
    text_renderer: render::text_renderer::TextRenderer,
    retained: render::retained::Realizer,
    scroll_layers: Vec<CachedScrollLayer>,
}

struct CachedScrollLayer {
    node: crate::composition::tree::NodeId,
    revision: render::retained::ScrollLayerRevision,
    segment: usize,
    scale_bits: u32,
    layer_bounds: paint::Rect,
    property_values: Box<[crate::scene::PropertyValue]>,
    layer: render::filter::Layer,
}

struct ScrollSegment<'a> {
    scroll: &'a PreparedScroll,
    index: usize,
    batches: &'a [RenderBatch],
}

#[derive(Clone)]
pub(in crate::render) enum ShapeBatch {
    Immediate(render::quad::Batch),
    Retained(render::retained::ShapeBatch),
}

#[derive(Clone, Copy)]
pub(in crate::render) enum TextBatch {
    Immediate { renderer_index: usize },
    Retained(render::text_renderer::RetainedBatch),
}

#[derive(Clone)]
pub(in crate::render) enum RenderBatch {
    Shapes(ShapeBatch),
    Pane(PreparedPane),
    Text(TextBatch),
    PushClip(PreparedClip),
    PopClip,
    Group(PreparedGroup),
    Scroll(PreparedScroll),
}

#[derive(Clone)]
pub(in crate::render) struct PreparedPane {
    pub(in crate::render) pane: paint::Pane,
    pub(in crate::render) base: Option<ShapeBatch>,
    pub(in crate::render) surface_layers: Vec<Option<ShapeBatch>>,
}

struct PreparedScene {
    render_batches: Vec<RenderBatch>,
    stats: DrawStats,
}

#[derive(Clone, Copy)]
pub(in crate::render) struct PreparedClip {
    pub(in crate::render) node: Option<crate::composition::tree::NodeId>,
    pub(in crate::render) fallback: paint::Clip,
    pub(in crate::render) scene_origin: [f32; 2],
}

#[derive(Clone)]
pub(in crate::render) struct PreparedGroup {
    pub(in crate::render) node: Option<crate::composition::tree::NodeId>,
    pub(in crate::render) bounds: Rect,
    pub(in crate::render) opacity: f32,
    pub(in crate::render) render_batches: Vec<RenderBatch>,
}

#[derive(Clone)]
pub(in crate::render) struct PreparedScroll {
    pub(in crate::render) node: crate::composition::tree::NodeId,
    pub(in crate::render) revision: render::retained::ScrollLayerRevision,
    pub(in crate::render) viewport: paint::Rect,
    pub(in crate::render) layer_bounds: paint::Rect,
    pub(in crate::render) baseline: crate::interaction::ScrollOffset,
    pub(in crate::render) render_batches: Vec<RenderBatch>,
}

#[derive(Default)]
struct CommandStats {
    draw_calls: usize,
    pipeline_changes: usize,
    bind_group_changes: usize,
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

struct SceneEncoder<'a> {
    render_context: &'a render::Context,
    quad_pipeline: &'a wgpu::RenderPipeline,
    quad_vertex_buffer: &'a wgpu::Buffer,
    filter_renderer: &'a render::filter::Renderer,
    text_renderer: &'a mut render::text_renderer::TextRenderer,
    retained_shapes: Option<&'a render::retained::Shapes>,
    retained_properties: Option<&'a render::retained::PropertyBindings>,
    scene_properties: Option<&'a crate::scene::Properties>,
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
    scroll_layers: Option<&'a mut Vec<CachedScrollLayer>>,
    scroll_origin: [f32; 2],
    scroll_translation: [f32; 2],
    scroll_clip: Option<paint::Rect>,
}

struct SceneEncoderInput<'a> {
    render_context: &'a render::Context,
    quad_pipeline: &'a wgpu::RenderPipeline,
    quad_vertex_buffer: &'a wgpu::Buffer,
    filter_renderer: &'a render::filter::Renderer,
    text_renderer: &'a mut render::text_renderer::TextRenderer,
    retained_shapes: Option<&'a render::retained::Shapes>,
    retained_properties: Option<&'a render::retained::PropertyBindings>,
    scene_properties: Option<&'a crate::scene::Properties>,
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

impl Renderer {
    pub fn new(render_context: &render::Context, format: render::surface::Format) -> Self {
        let format = format.into_wgpu();
        Self {
            format,
            quad_pipeline: render::quad::pipeline(render_context, format),
            quad_arena: render::quad::Arena::new(render_context),
            filter_renderer: render::filter::Renderer::new(render_context, format),
            popup_packer: render::popup_pack::Packer::new(render_context),
            text_renderer: render::text_renderer::TextRenderer::new(render_context, format),
            retained: render::retained::Realizer::new(render_context, format),
            scroll_layers: Vec::new(),
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

    pub fn draw(
        &mut self,
        render_context: &render::Context,
        canvas: &mut render::Canvas,
        scene: &paint::Scene,
    ) -> render::Result<DrawReport> {
        let batch_prepare_started = std::time::Instant::now();
        let clear_color = scene
            .clear_color()
            .map(render::color_to_wgpu)
            .unwrap_or_else(|| canvas.color());
        let main_viewport = render::Viewport::from_canvas(canvas);
        let item_batches = item_batches(scene.items());
        let scene_text_batches = glyph_batch_count(&item_batches);
        let text_batch_count = scene_text_batches;

        if text_batch_count > 0 {
            self.text_renderer
                .prepare_frame(render_context, text_batch_count);
        }

        let mut text_renderer_index = 0;
        let mut stats = DrawStats::default();
        self.quad_arena.begin_frame();
        let prepared_scene = self.prepare_scene_batches(
            render_context,
            main_viewport,
            &item_batches,
            &mut text_renderer_index,
        )?;
        let geometry_upload = self.quad_arena.upload(render_context);
        stats.add(prepared_scene.stats);
        stats.quad_vertices = geometry_upload.vertex_count;
        stats.geometry_upload_bytes = geometry_upload.bytes;
        stats.content_upload_bytes = geometry_upload.bytes;
        stats.geometry_buffer_creations = usize::from(geometry_upload.buffer_created);
        stats.scene_items = scene.items().len();
        stats.opacity_unclassified_nodes = scene.items().len();
        stats.clipped_nodes = stats.clip_batches;
        stats.render_plan_rebuilds = 1;
        let batch_prepare = batch_prepare_started.elapsed();

        self.draw_prepared(
            render_context,
            canvas,
            clear_color,
            main_viewport,
            &prepared_scene.render_batches,
            None,
            None,
            false,
            stats,
            batch_prepare,
        )
    }

    pub(crate) fn synchronize_commit(
        &mut self,
        render_context: &render::Context,
        canvas: &render::Canvas,
        commit: &std::sync::Arc<crate::scene::Commit>,
        properties: &crate::scene::Properties,
        budget: std::time::Duration,
        deadline: std::time::Duration,
    ) -> render::Result<CommitReadiness> {
        let viewport = render::Viewport::from_canvas(canvas);
        self.synchronize_commit_for_viewport(
            render_context,
            viewport,
            commit,
            properties,
            budget,
            deadline,
        )
    }

    fn synchronize_commit_for_viewport(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        commit: &std::sync::Arc<crate::scene::Commit>,
        properties: &crate::scene::Properties,
        budget: std::time::Duration,
        deadline: std::time::Duration,
    ) -> render::Result<CommitReadiness> {
        let started_at = std::time::Instant::now();
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

        loop {
            let slice_started = std::time::Instant::now();
            let prepared =
                self.retained
                    .prepare_pending(render_context, viewport, commit, properties)?;
            let ready =
                self.prepare_one_scroll_layer(render_context, viewport, &prepared, properties)?;
            self.retained.record_preparation_slice(
                viewport,
                commit,
                slice_started.elapsed(),
                deadline,
            );
            if ready {
                return Ok(CommitReadiness::Ready);
            }
            if started_at.elapsed() >= budget {
                return Ok(CommitReadiness::Prepared);
            }
        }
    }

    pub(crate) fn cancel_commit_synchronization(
        &mut self,
        commit: &std::sync::Arc<crate::scene::Commit>,
    ) {
        self.retained.cancel_synchronization(commit);
    }

    fn prepare_one_scroll_layer(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        prepared: &render::retained::Prepared,
        properties: &crate::scene::Properties,
    ) -> render::Result<bool> {
        validate_retained_layers(
            prepared.plan.batches(),
            viewport.scale_factor(),
            render_context.device().limits().max_texture_dimension_2d,
        )?;
        self.scroll_layers.retain(|entry| entry.revision.is_alive());
        let mut segments = Vec::new();
        collect_scroll_segments(prepared.plan.batches(), &mut segments);
        let scale_bits = viewport.scale_factor().to_bits();

        for segment in segments {
            let property_values = segment.scroll.revision.property_values(properties);
            if scroll_layer_cache_position(
                &mut self.scroll_layers,
                segment.scroll,
                segment.index,
                scale_bits,
                &property_values,
            )
            .is_some()
            {
                continue;
            }
            make_scroll_layer_room(
                &mut self.scroll_layers,
                &self.filter_renderer,
                segment.scroll,
                segment.index,
                scale_bits,
            );

            let target = render::filter::Target::from_logical_area(
                segment.scroll.layer_bounds.area,
                viewport.scale_factor(),
            );
            let layer = self.filter_renderer.create_layer(
                render_context,
                target,
                "Pending Retained Scroll Layer Texture",
            );
            let mut encoder =
                render_context
                    .device()
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Pending Retained Scroll Layer Encoder"),
                    });
            self.filter_renderer.clear_layer(&mut encoder, &layer);
            let mut text_render_error = None;
            let mut draw_passes = 1;
            let mut command_stats = CommandStats::default();
            {
                let mut scene_encoder = SceneEncoder::retained(
                    SceneEncoderInput {
                        render_context,
                        quad_pipeline: &self.quad_pipeline,
                        quad_vertex_buffer: self.quad_arena.buffer(),
                        filter_renderer: &self.filter_renderer,
                        text_renderer: &mut self.text_renderer,
                        retained_shapes: Some(self.retained.shapes()),
                        retained_properties: Some(&prepared.properties),
                        scene_properties: Some(properties),
                        encoder: &mut encoder,
                        base_view: layer.view(),
                        backdrop_view: layer.view(),
                        backdrop_target: target,
                        clear_color: wgpu::Color::TRANSPARENT,
                        viewport: render::Viewport::from_logical_area(
                            segment.scroll.layer_bounds.area,
                            viewport.scale_factor(),
                        ),
                        base_dirty: false,
                        inside_group: true,
                        text_render_error: &mut text_render_error,
                        draw_passes: &mut draw_passes,
                        command_stats: &mut command_stats,
                    },
                    &mut self.scroll_layers,
                );
                scene_encoder.encode(segment.batches);
            }
            if let Some(error) = text_render_error {
                return Err(error.into());
            }
            render_context
                .queue()
                .submit(std::iter::once(encoder.finish()));
            self.scroll_layers.push(CachedScrollLayer {
                node: segment.scroll.node,
                revision: segment.scroll.revision.clone(),
                segment: segment.index,
                scale_bits,
                layer_bounds: segment.scroll.layer_bounds,
                property_values,
                layer,
            });
            return Ok(false);
        }

        Ok(true)
    }

    pub fn draw_commit(
        &mut self,
        render_context: &render::Context,
        canvas: &mut render::Canvas,
        commit: &std::sync::Arc<crate::scene::Commit>,
        properties: &crate::scene::Properties,
        overlays: &crate::scene::Scene,
    ) -> render::Result<DrawReport> {
        let batch_prepare_started = std::time::Instant::now();
        let main_viewport = render::Viewport::from_canvas(canvas);
        let prepared = self.retained.prepare(
            render_context,
            main_viewport,
            commit,
            properties,
            &mut self.text_renderer,
        )?;
        let mut direct_surface_eligible = !prepared.plan.requires_surface_sampling();
        let mut batches = prepared.plan.batches().to_vec();
        let mut stats = prepared.stats;

        if !overlays.primitives().is_empty() {
            let overlay =
                render::scene::to_paint_scene_at_scale(overlays, main_viewport.scale_factor());
            let overlay_batches = item_batches(overlay.items());
            let text_batch_count = glyph_batch_count(&overlay_batches);
            if text_batch_count > 0 {
                self.text_renderer
                    .prepare_frame(render_context, text_batch_count);
            }
            let mut text_renderer_index = 0;
            self.quad_arena.begin_frame();
            let prepared_overlay = self.prepare_scene_batches(
                render_context,
                main_viewport,
                &overlay_batches,
                &mut text_renderer_index,
            )?;
            let geometry_upload = self.quad_arena.upload(render_context);
            stats.add(prepared_overlay.stats);
            stats.quad_vertices += geometry_upload.vertex_count;
            stats.geometry_upload_bytes += geometry_upload.bytes;
            stats.content_upload_bytes += geometry_upload.bytes;
            stats.geometry_buffer_creations += usize::from(geometry_upload.buffer_created);
            stats.scene_items += overlay.items().len();
            stats.blended_nodes += overlay.items().len();
            direct_surface_eligible &= !requires_surface_sampling(&prepared_overlay.render_batches);
            batches.extend(prepared_overlay.render_batches);
        }
        stats.clipped_nodes = stats.clip_batches;
        stats.direct_surface_plans = usize::from(direct_surface_eligible);
        stats.surface_sampling_plans = usize::from(!direct_surface_eligible);

        self.draw_prepared(
            render_context,
            canvas,
            render::surface_color(commit.clear()),
            main_viewport,
            &batches,
            Some(&prepared.properties),
            Some(properties),
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
        batches: &[RenderBatch],
        retained_properties: Option<&render::retained::PropertyBindings>,
        scene_properties: Option<&crate::scene::Properties>,
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
        let quad_pipeline = &self.quad_pipeline;
        let quad_vertex_buffer = self.quad_arena.buffer();
        let filter_renderer = &self.filter_renderer;
        let popup_packer = &self.popup_packer;
        let retained_shapes = retained_properties.map(|_| self.retained.shapes());
        let text_renderer = &mut self.text_renderer;
        self.scroll_layers.retain(|entry| entry.revision.is_alive());
        let scroll_layers = &mut self.scroll_layers;
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
                let mut scene_encoder = SceneEncoder::retained(
                    SceneEncoderInput {
                        render_context,
                        quad_pipeline,
                        quad_vertex_buffer,
                        filter_renderer,
                        text_renderer,
                        retained_shapes,
                        retained_properties,
                        scene_properties,
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
                    },
                    &mut *scroll_layers,
                );
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
                let mut scene_encoder = SceneEncoder::retained(
                    SceneEncoderInput {
                        render_context,
                        quad_pipeline,
                        quad_vertex_buffer,
                        filter_renderer,
                        text_renderer,
                        retained_shapes,
                        retained_properties,
                        scene_properties,
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
                    },
                    &mut *scroll_layers,
                );
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
                let mut scene_encoder = SceneEncoder::retained(
                    SceneEncoderInput {
                        render_context,
                        quad_pipeline,
                        quad_vertex_buffer,
                        filter_renderer,
                        text_renderer,
                        retained_shapes,
                        retained_properties,
                        scene_properties,
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
                    },
                    &mut *scroll_layers,
                );
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

        self.text_renderer.trim();
        stats.draw_calls = command_stats.draw_calls;
        stats.draw_passes = draw_passes;
        stats.resource_transition_boundaries = draw_passes.saturating_sub(1);
        stats.pipeline_changes = command_stats.pipeline_changes;
        stats.bind_group_changes = command_stats.bind_group_changes;
        stats.scroll_layer_cache_hits = command_stats.scroll_layer_cache_hits;
        stats.scroll_layer_cache_misses = command_stats.scroll_layer_cache_misses;
        let (scroll_layer_resources, scroll_layer_bytes) =
            scroll_layer_resource_state(&self.scroll_layers);
        stats.retained_gpu_resource_count = stats
            .retained_gpu_resource_count
            .saturating_add(scroll_layer_resources);
        stats.retained_gpu_resource_bytes = stats
            .retained_gpu_resource_bytes
            .saturating_add(scroll_layer_bytes);
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
    pub(super) fn draw_offscreen_debug(
        &mut self,
        render_context: &render::Context,
        scene: &paint::Scene,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> Result<Vec<[f32; 4]>, String> {
        self.draw_offscreen_debug_mode(render_context, scene, width, height, scale_factor, false)
    }

    #[cfg(feature = "renderer-debug")]
    pub(super) fn draw_offscreen_popup_debug(
        &mut self,
        render_context: &render::Context,
        scene: &paint::Scene,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> Result<Vec<[f32; 4]>, String> {
        self.draw_offscreen_debug_mode(render_context, scene, width, height, scale_factor, true)
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
            self.scroll_layers.retain(|entry| entry.revision.is_alive());
            let mut scene_encoder = SceneEncoder::retained(
                SceneEncoderInput {
                    render_context,
                    quad_pipeline: &self.quad_pipeline,
                    quad_vertex_buffer: self.quad_arena.buffer(),
                    filter_renderer: &self.filter_renderer,
                    text_renderer: &mut self.text_renderer,
                    retained_shapes: Some(self.retained.shapes()),
                    retained_properties: Some(&prepared.properties),
                    scene_properties: Some(properties),
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
                },
                &mut self.scroll_layers,
            );
            scene_encoder.encode(prepared.plan.batches());
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
        let mut stats = prepared.stats;
        stats.draw_calls = command_stats.draw_calls + usize::from(pack_popup);
        stats.draw_passes = draw_passes;
        stats.resource_transition_boundaries = draw_passes.saturating_sub(1);
        stats.pipeline_changes = command_stats.pipeline_changes + usize::from(pack_popup);
        stats.bind_group_changes = command_stats.bind_group_changes + usize::from(pack_popup);
        stats.scroll_layer_cache_hits = command_stats.scroll_layer_cache_hits;
        stats.scroll_layer_cache_misses = command_stats.scroll_layer_cache_misses;
        let (scroll_layer_resources, scroll_layer_bytes) =
            scroll_layer_resource_state(&self.scroll_layers);
        stats.retained_gpu_resource_count = stats
            .retained_gpu_resource_count
            .saturating_add(scroll_layer_resources);
        stats.retained_gpu_resource_bytes = stats
            .retained_gpu_resource_bytes
            .saturating_add(scroll_layer_bytes);
        stats.effect_intermediate_clears = command_stats.effect_intermediate_clears;
        stats.effect_intermediate_clear_bytes = command_stats.effect_intermediate_clear_bytes;
        stats.effect_intermediate_composites = command_stats.effect_intermediate_composites;
        stats.effect_intermediate_composite_bytes =
            command_stats.effect_intermediate_composite_bytes;
        stats.largest_effect_intermediate_bytes = command_stats.largest_effect_intermediate_bytes;
        Ok((pixels, stats))
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
        let viewport = render::Viewport::from_logical_area(
            area::logical(width as f32 / scale_factor, height as f32 / scale_factor),
            scale_factor,
        );
        self.synchronize_commit_for_viewport(
            render_context,
            viewport,
            commit,
            properties,
            budget,
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
    }

    #[cfg(feature = "renderer-debug")]
    pub(super) fn retained_state_counts_debug(&self) -> (usize, usize) {
        self.retained.debug_state_counts()
    }

    #[cfg(feature = "renderer-debug")]
    pub(super) fn retained_resource_state_debug(&self) -> (usize, usize) {
        let (shape_count, shape_bytes) = self.retained.debug_resource_state();
        let (scroll_count, scroll_bytes) = scroll_layer_resource_state(&self.scroll_layers);
        (
            shape_count
                .saturating_add(self.text_renderer.retained_resource_count())
                .saturating_add(scroll_count),
            shape_bytes
                .saturating_add(self.text_renderer.retained_resource_bytes())
                .saturating_add(scroll_bytes),
        )
    }

    #[cfg(feature = "renderer-debug")]
    fn draw_offscreen_debug_mode(
        &mut self,
        render_context: &render::Context,
        scene: &paint::Scene,
        width: u32,
        height: u32,
        scale_factor: f32,
        pack_popup: bool,
    ) -> Result<Vec<[f32; 4]>, String> {
        let format = self.format;
        if pack_popup && format != wgpu::TextureFormat::Rgba8UnormSrgb {
            return Err("popup debug packing requires an sRGB renderer".to_owned());
        }
        let texture = render_context
            .device()
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Renderer Debug Target"),
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
                    label: Some("Renderer Debug Packed Popup Target"),
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
        let item_batch_list = item_batches(scene.items());
        let text_batch_count = glyph_batch_count(&item_batch_list);
        if text_batch_count > 0 {
            self.text_renderer
                .prepare_frame(render_context, text_batch_count);
        }
        let mut text_renderer_index = 0;
        self.quad_arena.begin_frame();
        let prepared_scene = self
            .prepare_scene_batches(
                render_context,
                viewport,
                &item_batch_list,
                &mut text_renderer_index,
            )
            .map_err(|error| error.to_string())?;
        self.quad_arena.upload(render_context);

        let clear_color = scene
            .clear_color()
            .map(render::color_to_wgpu)
            .unwrap_or(wgpu::Color::TRANSPARENT);
        let mut text_render_error = None;
        let mut draw_passes = 1;
        let mut command_stats = CommandStats::default();
        let mut encoder =
            render_context
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Renderer Debug Encoder"),
                });
        {
            let _clear = begin_main_pass(&mut encoder, &view, clear_color, true);
        }
        let mut scene_encoder = SceneEncoder::new(SceneEncoderInput {
            render_context,
            quad_pipeline: &self.quad_pipeline,
            quad_vertex_buffer: self.quad_arena.buffer(),
            filter_renderer: &self.filter_renderer,
            text_renderer: &mut self.text_renderer,
            retained_shapes: None,
            retained_properties: None,
            scene_properties: None,
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
        scene_encoder.encode(&prepared_scene.render_batches);
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
        }

        let bytes_per_pixel = 4u32;
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(alignment) * alignment;
        let output = render_context
            .device()
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("Renderer Debug Readback"),
                size: padded_bytes_per_row as u64 * height as u64,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
        let readback_texture = packed_texture.as_ref().unwrap_or(&texture);
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: readback_texture,
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

    fn prepare_scene_batches(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        item_batch_list: &[ItemBatch<'_>],
        text_renderer_index: &mut usize,
    ) -> render::Result<PreparedScene> {
        let mut render_batches = Vec::with_capacity(item_batch_list.len());
        let mut stats = DrawStats {
            render_batches: item_batch_list.len(),
            glyph_batches: item_batch_list
                .iter()
                .map(glyph_batch_count_for_batch)
                .sum(),
            text_surfaces: item_batch_list
                .iter()
                .map(text_surface_count_for_batch)
                .sum(),
            clip_batches: item_batch_list
                .iter()
                .filter(|batch| matches!(batch, ItemBatch::PushClip(_) | ItemBatch::PopClip))
                .count(),
            group_composites: item_batch_list
                .iter()
                .filter(|batch| matches!(batch, ItemBatch::Group(_)))
                .count(),
            effect_island_nodes: item_batch_list
                .iter()
                .filter(|batch| {
                    matches!(batch, ItemBatch::Group(_))
                        || matches!(
                            batch,
                            ItemBatch::Pane(pane)
                                if matches!(pane.material, paint::Material::Glass(_))
                        )
                })
                .count(),
            ..DrawStats::default()
        };

        for batch in item_batch_list {
            match batch {
                ItemBatch::Shapes(shapes) => {
                    stats.quad_prepare_calls += 1;
                    if let Some(batch) =
                        render::quad::prepare_batch(&mut self.quad_arena, viewport, shapes)
                    {
                        stats.quad_vertices += batch.vertex_count() as usize;
                        render_batches.push(RenderBatch::Shapes(ShapeBatch::Immediate(batch)));
                    }
                }
                ItemBatch::Pane(pane) => {
                    let (prepared, quad_prepare_calls) = self.prepare_pane(viewport, pane);
                    stats.quad_prepare_calls += quad_prepare_calls;
                    if matches!(pane.material, paint::Material::Solid(_)) {
                        if let Some(batch) = prepared.base {
                            render_batches.push(RenderBatch::Shapes(batch));
                        }
                    } else {
                        render_batches.push(RenderBatch::Pane(prepared));
                    }
                }
                ItemBatch::PushClip(clip) => {
                    render_batches.push(RenderBatch::PushClip(PreparedClip {
                        node: None,
                        fallback: **clip,
                        scene_origin: [0.0, 0.0],
                    }));
                }
                ItemBatch::PopClip => {
                    render_batches.push(RenderBatch::PopClip);
                }
                ItemBatch::Group(group) => {
                    let group_viewport = render::Viewport::from_logical_area(
                        group.bounds.area,
                        viewport.scale_factor(),
                    );
                    let group_batches = item_batches(&group.items);
                    let prepared = self.prepare_scene_batches(
                        render_context,
                        group_viewport,
                        &group_batches,
                        text_renderer_index,
                    )?;
                    stats.add(prepared.stats);
                    render_batches.push(RenderBatch::Group(PreparedGroup {
                        node: None,
                        bounds: group.bounds,
                        opacity: group.opacity,
                        render_batches: prepared.render_batches,
                    }));
                }
                ItemBatch::Glyphs(glyphs) => {
                    stats.text_prepare_calls += 1;
                    let report = self.text_renderer.prepare_batch(
                        render_context,
                        viewport,
                        *text_renderer_index,
                        glyphs,
                    )?;
                    stats.inline_text_cache_hits += report.stats.text_cache_hits;
                    stats.inline_text_cache_misses += report.stats.text_cache_misses;
                    stats.inline_text_shape_calls += report.stats.text_shape_calls;
                    stats.inline_icon_cache_hits += report.stats.icon_cache_hits;
                    stats.inline_icon_cache_misses += report.stats.icon_cache_misses;
                    stats.inline_icon_shape_calls += report.stats.icon_shape_calls;
                    if report.has_text {
                        render_batches.push(RenderBatch::Text(TextBatch::Immediate {
                            renderer_index: *text_renderer_index,
                        }));
                    }

                    *text_renderer_index += 1;
                }
            }
        }
        stats.render_batches = render_batches.len();

        Ok(PreparedScene {
            render_batches,
            stats,
        })
    }

    fn prepare_pane(
        &mut self,
        viewport: render::Viewport,
        pane: &paint::Pane,
    ) -> (PreparedPane, usize) {
        let base_brush = match &pane.material {
            paint::Material::Solid(brush) => Some(*brush),
            paint::Material::Glass(glass) if glass.base == paint::GlassBase::Fallback => {
                Some(glass.fallback)
            }
            paint::Material::Glass(_) => None,
        };
        let surface_brushes = match &pane.material {
            paint::Material::Solid(_) => Vec::new(),
            paint::Material::Glass(glass) => glass
                .surface_layers
                .iter()
                .map(|layer| match *layer {
                    paint::SurfaceLayer::Tint { brush, opacity } => {
                        Some(render::material::brush_with_opacity(brush, opacity))
                    }
                    paint::SurfaceLayer::Noise(_) => None,
                })
                .collect::<Vec<_>>(),
        };

        let quad_prepare_calls = base_brush
            .iter()
            .chain(surface_brushes.iter().flatten())
            .filter(|brush| brush.is_visible())
            .count();
        let prepared = PreparedPane {
            pane: pane.clone(),
            base: base_brush.and_then(|brush| {
                prepare_brush_batch(&mut self.quad_arena, viewport, pane.rect, brush)
                    .map(ShapeBatch::Immediate)
            }),
            surface_layers: surface_brushes
                .into_iter()
                .map(|brush| {
                    brush.and_then(|brush| {
                        prepare_brush_batch(&mut self.quad_arena, viewport, pane.rect, brush)
                            .map(ShapeBatch::Immediate)
                    })
                })
                .collect(),
        };
        (prepared, quad_prepare_calls)
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
    batches: &[RenderBatch],
    scale_factor: f32,
    limit: u32,
) -> render::Result<()> {
    for batch in batches {
        match batch {
            RenderBatch::Group(group) => {
                validate_retained_layer("group", group.bounds.area, scale_factor, limit)?;
                validate_retained_layers(&group.render_batches, scale_factor, limit)?;
            }
            RenderBatch::Scroll(scroll) => {
                validate_retained_layer("scroll", scroll.layer_bounds.area, scale_factor, limit)?;
                validate_retained_layers(&scroll.render_batches, scale_factor, limit)?;
            }
            RenderBatch::Shapes(_)
            | RenderBatch::Pane(_)
            | RenderBatch::Text(_)
            | RenderBatch::PushClip(_)
            | RenderBatch::PopClip => {}
        }
    }
    Ok(())
}

fn collect_scroll_segments<'a>(batches: &'a [RenderBatch], segments: &mut Vec<ScrollSegment<'a>>) {
    for batch in batches {
        match batch {
            RenderBatch::Group(group) => collect_scroll_segments(&group.render_batches, segments),
            RenderBatch::Scroll(scroll) => {
                let mut index = 0;
                while index < scroll.render_batches.len() {
                    if matches!(scroll.render_batches[index], RenderBatch::Scroll(_)) {
                        index += 1;
                        continue;
                    }
                    let start = index;
                    while index < scroll.render_batches.len()
                        && !matches!(scroll.render_batches[index], RenderBatch::Scroll(_))
                    {
                        index += 1;
                    }
                    if start != index {
                        segments.push(ScrollSegment {
                            scroll,
                            index: start,
                            batches: &scroll.render_batches[start..index],
                        });
                    }
                }
                collect_scroll_segments(&scroll.render_batches, segments);
            }
            RenderBatch::Shapes(_)
            | RenderBatch::Pane(_)
            | RenderBatch::Text(_)
            | RenderBatch::PushClip(_)
            | RenderBatch::PopClip => {}
        }
    }
}

fn same_scroll_layer_structure(
    entry: &CachedScrollLayer,
    scroll: &PreparedScroll,
    segment: usize,
    scale_bits: u32,
) -> bool {
    entry.node == scroll.node
        && entry.revision.matches(&scroll.revision)
        && entry.segment == segment
        && entry.scale_bits == scale_bits
        && entry.layer_bounds == scroll.layer_bounds
}

fn scroll_layer_cache_position(
    layers: &mut [CachedScrollLayer],
    scroll: &PreparedScroll,
    segment: usize,
    scale_bits: u32,
    property_values: &[crate::scene::PropertyValue],
) -> Option<usize> {
    let position = layers.iter().position(|entry| {
        same_scroll_layer_structure(entry, scroll, segment, scale_bits)
            && entry.property_values.as_ref() == property_values
    })?;
    layers[position].revision = scroll.revision.clone();
    Some(position)
}

fn make_scroll_layer_room(
    layers: &mut Vec<CachedScrollLayer>,
    filter_renderer: &render::filter::Renderer,
    scroll: &PreparedScroll,
    segment: usize,
    scale_bits: u32,
) {
    const ACTIVE_AND_PENDING_VARIANTS: usize = 2;
    let matching = layers
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            same_scroll_layer_structure(entry, scroll, segment, scale_bits).then_some(index)
        })
        .collect::<Vec<_>>();
    if matching.len() >= ACTIVE_AND_PENDING_VARIANTS {
        let retired = layers.remove(matching[0]);
        filter_renderer.recycle_layer(retired.layer);
    }
}

fn scroll_layer_resource_state(layers: &[CachedScrollLayer]) -> (usize, usize) {
    (
        layers.len(),
        layers.iter().fold(0_usize, |bytes, entry| {
            bytes.saturating_add(entry.layer.resource_bytes())
        }),
    )
}

fn prepare_brush_batch(
    arena: &mut render::quad::Arena,
    viewport: render::Viewport,
    rect: Rect,
    brush: paint::Brush,
) -> Option<render::quad::Batch> {
    if !brush.is_visible() {
        return None;
    }
    let quad = paint::Quad::resolved_for_grid(
        rect,
        paint::Style {
            fill: Some(paint::Fill::Brush(brush)),
            stroke: None,
            tint: None,
        },
        paint::Rasterization::default(),
        paint::Transform::identity(),
        paint::Grid::new(viewport.scale_factor()),
    );
    render::quad::prepare_batch(arena, viewport, &[render::batch::Shape::Quad(&quad)])
}

impl DrawStats {
    #[cfg(test)]
    fn from_scene(scene: &paint::Scene, batches: &[ItemBatch<'_>]) -> Self {
        Self {
            scene_items: scene.items().len(),
            render_batches: batches.len(),
            glyph_batches: batches.iter().map(glyph_batch_count_for_batch).sum(),
            text_surfaces: scene.items().iter().map(text_surface_count_for_item).sum(),
            quad_vertices: 0,
            clip_batches: scene
                .items()
                .iter()
                .filter(|item| matches!(item, paint::Item::Clip(_) | paint::Item::PopClip))
                .count(),
            group_composites: scene
                .items()
                .iter()
                .filter(|item| matches!(item, paint::Item::Group(_)))
                .count(),
            ..Self::default()
        }
    }

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

fn glyph_batch_count(batches: &[ItemBatch<'_>]) -> usize {
    batches.iter().map(glyph_batch_count_for_batch).sum()
}

fn glyph_batch_count_for_batch(batch: &ItemBatch<'_>) -> usize {
    match batch {
        ItemBatch::Glyphs(_) => 1,
        ItemBatch::Group(group) => glyph_batch_count(&item_batches(&group.items)),
        _ => 0,
    }
}

fn text_surface_count_for_batch(batch: &ItemBatch<'_>) -> usize {
    match batch {
        ItemBatch::Glyphs(glyphs) => glyphs
            .iter()
            .map(|glyph| match glyph {
                crate::render::batch::Glyph::TextViewport(viewport) => viewport.surfaces.len(),
                _ => 0,
            })
            .sum(),
        ItemBatch::Group(group) => item_batches(&group.items)
            .iter()
            .map(text_surface_count_for_batch)
            .sum(),
        _ => 0,
    }
}

#[cfg(test)]
fn text_surface_count_for_item(item: &paint::Item) -> usize {
    match item {
        paint::Item::TextViewport(viewport) => viewport.surfaces.len(),
        paint::Item::Group(group) => group.items.iter().map(text_surface_count_for_item).sum(),
        _ => 0,
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

fn is_draw_batch(batch: &RenderBatch) -> bool {
    matches!(batch, RenderBatch::Shapes(_) | RenderBatch::Text(_))
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

pub(in crate::render) fn requires_surface_sampling(batches: &[RenderBatch]) -> bool {
    batches.iter().any(|batch| match batch {
        RenderBatch::Pane(prepared) => match &prepared.pane.material {
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
        RenderBatch::Group(group) => requires_surface_sampling(&group.render_batches),
        RenderBatch::Scroll(scroll) => requires_surface_sampling(&scroll.render_batches),
        RenderBatch::Shapes(_)
        | RenderBatch::Text(_)
        | RenderBatch::PushClip(_)
        | RenderBatch::PopClip => false,
    })
}

fn draw_run_end(render_batches: &[RenderBatch], start: usize) -> usize {
    render_batches[start..]
        .iter()
        .position(|batch| !is_draw_batch(batch))
        .map_or(render_batches.len(), |offset| start + offset)
}

impl<'a> SceneEncoder<'a> {
    fn new(input: SceneEncoderInput<'a>) -> Self {
        Self {
            render_context: input.render_context,
            quad_pipeline: input.quad_pipeline,
            quad_vertex_buffer: input.quad_vertex_buffer,
            filter_renderer: input.filter_renderer,
            text_renderer: input.text_renderer,
            retained_shapes: input.retained_shapes,
            retained_properties: input.retained_properties,
            scene_properties: input.scene_properties,
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
            scroll_layers: None,
            scroll_origin: [0.0, 0.0],
            scroll_translation: [0.0, 0.0],
            scroll_clip: None,
        }
    }

    fn retained(
        input: SceneEncoderInput<'a>,
        scroll_layers: &'a mut Vec<CachedScrollLayer>,
    ) -> Self {
        let mut encoder = Self::new(input);
        encoder.scroll_layers = Some(scroll_layers);
        encoder
    }

    fn encode(&mut self, render_batches: &[RenderBatch]) {
        self.encode_batches(render_batches);

        while !self.clip_stack.is_empty() {
            self.composite_clip_layer();
        }

        for layer in self.layers.drain(..) {
            self.filter_renderer.recycle_layer(layer);
        }
    }

    fn encode_batches(&mut self, render_batches: &[RenderBatch]) {
        let mut index = 0;
        while index < render_batches.len() {
            if is_draw_batch(&render_batches[index]) {
                let end = draw_run_end(render_batches, index);
                self.encode_draw_run(&render_batches[index..end]);
                index = end;
                continue;
            }

            match &render_batches[index] {
                RenderBatch::Shapes(_) | RenderBatch::Text(_) => unreachable!(),
                RenderBatch::Pane(pane) => self.encode_pane(pane),
                RenderBatch::PushClip(clip) => self.push_clip(self.resolve_clip(*clip)),
                RenderBatch::PopClip => self.composite_clip_layer(),
                RenderBatch::Group(group) => self.encode_group(group),
                RenderBatch::Scroll(scroll) => self.encode_scroll(scroll),
            }
            index += 1;
        }
    }

    fn encode_draw_run(&mut self, batches: &[RenderBatch]) {
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
                    RenderBatch::Shapes(ShapeBatch::Immediate(batch)) => {
                        pass.set_pipeline(self.quad_pipeline);
                        pass.set_scissor_rect(
                            scissor.x(),
                            scissor.y(),
                            scissor.width(),
                            scissor.height(),
                        );
                        pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
                        pass.draw(batch.vertex_range(), 0..1);
                        self.command_stats.record_draws(1, false);
                    }
                    RenderBatch::Shapes(ShapeBatch::Retained(batch)) => {
                        if let (Some(shapes), Some(properties)) =
                            (self.retained_shapes, self.retained_properties)
                        {
                            shapes.draw(&mut pass, batch, properties);
                            self.command_stats.record_draws(1, true);
                        }
                    }
                    RenderBatch::Text(TextBatch::Immediate { renderer_index }) => {
                        if let Err(error) = self.text_renderer.render(*renderer_index, &mut pass) {
                            *self.text_render_error = Some(error);
                            break;
                        }
                        self.command_stats.record_draws(1, true);
                    }
                    RenderBatch::Text(TextBatch::Retained(batch)) => {
                        if let Err(error) = self.text_renderer.render_retained(*batch, &mut pass) {
                            *self.text_render_error = Some(error);
                            break;
                        }
                        self.command_stats.record_draws(1, true);
                    }
                    RenderBatch::Pane(_)
                    | RenderBatch::PushClip(_)
                    | RenderBatch::PopClip
                    | RenderBatch::Group(_)
                    | RenderBatch::Scroll(_) => unreachable!(),
                }
            }
        }
        self.mark_current_dirty();
    }

    fn encode_shapes(&mut self, batch: &ShapeBatch) {
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
            match batch {
                ShapeBatch::Immediate(batch) => {
                    pass.set_pipeline(self.quad_pipeline);
                    pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
                    pass.draw(batch.vertex_range(), 0..1);
                    self.command_stats.record_draws(1, false);
                }
                ShapeBatch::Retained(batch) => {
                    if let (Some(shapes), Some(properties)) =
                        (self.retained_shapes, self.retained_properties)
                    {
                        shapes.draw(&mut pass, batch, properties);
                        self.command_stats.record_draws(1, true);
                    }
                }
            }
        }
        self.mark_current_dirty();
    }

    fn encode_pane(&mut self, prepared: &PreparedPane) {
        let pane = &prepared.pane;
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
        if opacity <= 0.0 {
            return;
        }

        let group_target = render::filter::Target::from_logical_area(
            group.bounds.area,
            self.viewport.scale_factor(),
        );
        let group_layer = self.filter_renderer.create_layer(
            self.render_context,
            group_target,
            "Group Layer Texture",
        );
        self.filter_renderer.clear_layer(self.encoder, &group_layer);
        *self.draw_passes += 1;

        let mut scroll_layers = self.scroll_layers.take();
        {
            let group_view = group_layer.view();
            let input = SceneEncoderInput {
                render_context: self.render_context,
                quad_pipeline: self.quad_pipeline,
                quad_vertex_buffer: self.quad_vertex_buffer,
                filter_renderer: self.filter_renderer,
                text_renderer: self.text_renderer,
                retained_shapes: self.retained_shapes,
                retained_properties: self.retained_properties,
                scene_properties: self.scene_properties,
                encoder: self.encoder,
                base_view: group_view,
                backdrop_view: self.base_view,
                backdrop_target: self.output_target,
                clear_color: wgpu::Color::TRANSPARENT,
                viewport: render::Viewport::from_logical_area(
                    group.bounds.area,
                    self.viewport.scale_factor(),
                ),
                base_dirty: false,
                inside_group: true,
                text_render_error: self.text_render_error,
                draw_passes: self.draw_passes,
                command_stats: self.command_stats,
            };
            if let Some(layers) = scroll_layers.as_deref_mut() {
                let mut group_encoder = SceneEncoder::retained(input, layers);
                group_encoder.encode(&group.render_batches);
            } else {
                let mut group_encoder = SceneEncoder::new(input);
                group_encoder.encode(&group.render_batches);
            }
        }
        self.scroll_layers = scroll_layers;

        if let Some(output) = current_target_view(self.base_view, &self.layers, &self.clip_stack) {
            self.filter_renderer
                .composite_layer(render::filter::LayerComposite {
                    render_context: self.render_context,
                    encoder: self.encoder,
                    source: &group_layer,
                    output,
                    target: self.output_target,
                    clip: paint::Clip { rect: group.bounds },
                    source_rect: Some(group_layer_source_rect(group.bounds)),
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
        let parent_origin = self.scroll_origin;
        let parent_translation = self.scroll_translation;
        let parent_clip = self.scroll_clip;
        let current = self
            .scene_properties
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
        let viewport = translated_rect(
            scroll.viewport,
            [
                parent_origin[0] + parent_translation[0],
                parent_origin[1] + parent_translation[1],
            ],
        );
        let Some(visible) = parent_clip
            .map(|clip| intersect_rect(clip, viewport))
            .unwrap_or(Some(viewport))
        else {
            return;
        };
        let destination = translated_rect(
            scroll.layer_bounds,
            [
                parent_origin[0] + translation[0],
                parent_origin[1] + translation[1],
            ],
        );

        let mut index = 0;
        while index < scroll.render_batches.len() {
            if let RenderBatch::Scroll(nested) = &scroll.render_batches[index] {
                self.scroll_origin = [
                    parent_origin[0] + scroll.layer_bounds.origin.x(),
                    parent_origin[1] + scroll.layer_bounds.origin.y(),
                ];
                self.scroll_translation = translation;
                self.scroll_clip = Some(visible);
                self.encode_scroll(nested);
                index += 1;
                continue;
            }

            let start = index;
            while index < scroll.render_batches.len()
                && !matches!(scroll.render_batches[index], RenderBatch::Scroll(_))
            {
                index += 1;
            }
            self.encode_scroll_segment(
                scroll,
                start,
                &scroll.render_batches[start..index],
                destination,
                visible,
            );
        }

        self.scroll_origin = parent_origin;
        self.scroll_translation = parent_translation;
        self.scroll_clip = parent_clip;
    }

    fn encode_scroll_segment(
        &mut self,
        scroll: &PreparedScroll,
        segment: usize,
        batches: &[RenderBatch],
        destination: paint::Rect,
        visible: paint::Rect,
    ) {
        if batches.is_empty() {
            return;
        }
        let Some(layers) = self.scroll_layers.take() else {
            self.encode_batches(batches);
            return;
        };
        let scale_bits = self.viewport.scale_factor().to_bits();
        let property_values = self
            .scene_properties
            .map(|properties| scroll.revision.property_values(properties))
            .unwrap_or_default();
        let position =
            scroll_layer_cache_position(layers, scroll, segment, scale_bits, &property_values);
        let position = if let Some(position) = position {
            self.command_stats.scroll_layer_cache_hits += 1;
            position
        } else {
            self.command_stats.scroll_layer_cache_misses += 1;
            make_scroll_layer_room(layers, self.filter_renderer, scroll, segment, scale_bits);
            let target = render::filter::Target::from_logical_area(
                scroll.layer_bounds.area,
                self.viewport.scale_factor(),
            );
            let layer = self.filter_renderer.create_layer(
                self.render_context,
                target,
                "Retained Scroll Layer Texture",
            );
            self.filter_renderer.clear_layer(self.encoder, &layer);
            *self.draw_passes += 1;
            {
                let input = SceneEncoderInput {
                    render_context: self.render_context,
                    quad_pipeline: self.quad_pipeline,
                    quad_vertex_buffer: self.quad_vertex_buffer,
                    filter_renderer: self.filter_renderer,
                    text_renderer: self.text_renderer,
                    retained_shapes: self.retained_shapes,
                    retained_properties: self.retained_properties,
                    scene_properties: self.scene_properties,
                    encoder: self.encoder,
                    base_view: layer.view(),
                    backdrop_view: self.backdrop_view,
                    backdrop_target: target,
                    clear_color: wgpu::Color::TRANSPARENT,
                    viewport: render::Viewport::from_logical_area(
                        scroll.layer_bounds.area,
                        self.viewport.scale_factor(),
                    ),
                    base_dirty: false,
                    inside_group: true,
                    text_render_error: self.text_render_error,
                    draw_passes: self.draw_passes,
                    command_stats: self.command_stats,
                };
                let mut layer_encoder = SceneEncoder::retained(input, &mut *layers);
                layer_encoder.encode(batches);
            }
            layers.push(CachedScrollLayer {
                node: scroll.node,
                revision: scroll.revision.clone(),
                segment,
                scale_bits,
                layer_bounds: scroll.layer_bounds,
                property_values,
                layer,
            });
            layers.len() - 1
        };

        let Some(output) = current_target_view(self.base_view, &self.layers, &self.clip_stack)
        else {
            self.scroll_layers = Some(layers);
            return;
        };
        let Some(scroll_scissor) = clip_scissor(
            visible,
            self.viewport.physical_area(),
            self.viewport.scale_factor(),
        ) else {
            self.scroll_layers = Some(layers);
            return;
        };
        let Some(scissor) = current_scissor(
            &self.clip_stack,
            self.viewport.physical_area(),
            self.viewport.scale_factor(),
        )
        .and_then(|current| intersect_scissor(current, scroll_scissor)) else {
            self.scroll_layers = Some(layers);
            return;
        };
        self.filter_renderer
            .composite_layer(render::filter::LayerComposite {
                render_context: self.render_context,
                encoder: self.encoder,
                source: &layers[position].layer,
                output,
                target: self.output_target,
                clip: paint::Clip { rect: destination },
                source_rect: Some(group_layer_source_rect(scroll.layer_bounds)),
                scissor: Some(scissor),
                opacity: 1.0,
            });
        self.command_stats.record_draws(1, true);
        *self.draw_passes += 1;
        self.mark_current_dirty();
        self.scroll_layers = Some(layers);
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
                let crate::scene::PropertyValue::Clip { rect, .. } =
                    self.scene_properties?
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
        resolved
    }

    fn resolve_opacity(&self, group: &PreparedGroup) -> f32 {
        let Some(node) = group.node else {
            return group.opacity;
        };
        match self.scene_properties.and_then(|properties| {
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
    use crate::text;

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
    fn draw_runs_cross_shape_text_pipeline_changes_but_stop_at_semantic_boundaries() {
        let batches = [
            RenderBatch::Shapes(ShapeBatch::Immediate(
                render::quad::Batch::from_vertex_range(0..6),
            )),
            RenderBatch::Text(TextBatch::Immediate { renderer_index: 1 }),
            RenderBatch::PopClip,
            RenderBatch::Text(TextBatch::Immediate { renderer_index: 2 }),
        ];

        assert_eq!(draw_run_end(&batches, 0), 2);
        assert!(!is_draw_batch(&batches[2]));
        assert_eq!(draw_run_end(&batches, 3), 4);
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
    fn draw_stats_summarize_scene_and_batches() {
        let mut scene = paint::Scene::new();
        scene.push_quad(paint::Quad::unchecked_for_test(
            Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
            paint::Style {
                fill: Some(paint::Fill::Brush(paint::Brush::solid(paint::Color::BLACK))),
                stroke: None,
                tint: None,
            },
            paint::Rasterization::default(),
            paint::Transform::identity(),
        ));
        scene.push_text(paint::Text {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
            document: text::document::Document::plain("hello"),
            wrap: paint::TextWrap::WordOrGlyph,
            vertical_align: paint::TextVerticalAlign::Center,
            overflow: crate::text::Overflow::Clip,
        });
        scene.push_icon(paint::Icon {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
            icon: crate::icon::Icon::phosphor(crate::icon::Id::new("check")),
            color: paint::Color::BLACK,
            size: 16.0,
        });
        scene.push_clip(paint::Clip {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
        });
        scene.pop_clip();
        scene.push_group(paint::Group {
            bounds: Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
            opacity: 0.5,
            items: vec![
                paint::Item::Quad(paint::Quad::unchecked_for_test(
                    Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
                    paint::Style {
                        fill: Some(paint::Fill::Brush(paint::Brush::solid(paint::Color::BLACK))),
                        stroke: None,
                        tint: None,
                    },
                    paint::Rasterization::default(),
                    paint::Transform::identity(),
                )),
                paint::Item::Text(paint::Text {
                    rect: Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
                    document: text::document::Document::plain("group"),
                    wrap: paint::TextWrap::WordOrGlyph,
                    vertical_align: paint::TextVerticalAlign::Center,
                    overflow: crate::text::Overflow::Clip,
                }),
            ],
        });

        let batches = item_batches(scene.items());
        let stats = DrawStats::from_scene(&scene, &batches);

        assert_eq!(stats.scene_items, 6);
        assert_eq!(stats.render_batches, 5);
        assert_eq!(stats.glyph_batches, 2);
        assert_eq!(stats.text_surfaces, 0);
        assert_eq!(stats.clip_batches, 2);
        assert_eq!(stats.group_composites, 1);
    }

    #[test]
    fn group_layer_source_rect_is_local_texture_space() {
        let bounds = Rect::new(point::logical(240.0, 120.0), area::logical(160.0, 80.0));

        let source = group_layer_source_rect(bounds);

        assert_eq!(source.origin, point::logical(0.0, 0.0));
        assert_eq!(source.area, bounds.area);
    }

    #[test]
    fn retained_layer_limit_rejects_the_reported_oversized_scroll_texture() {
        let error = validate_retained_layer("scroll", area::logical(760.0, 16_862.0), 1.0, 8_192)
            .expect_err("the reported layer must be rejected before WGPU texture creation");

        assert!(matches!(
            error,
            render::Error::RetainedLayerLimit {
                kind: "scroll",
                width: 760,
                height: 16_862,
                limit: 8_192,
            }
        ));
    }

    #[test]
    fn group_filter_encoding_is_local_but_group_composite_back_is_parent_space() {
        let source = std::fs::read_to_string(file!()).expect("renderer source should be readable");
        let encode_group = source
            .split("fn encode_group")
            .nth(1)
            .expect("encode_group should exist")
            .split("fn push_clip")
            .next()
            .expect("encode_group should precede push_clip");

        assert!(
            encode_group.contains("let group_target = render::filter::Target::from_logical_area")
        );
        assert!(encode_group.contains("base_view: group_view"));
        assert!(encode_group.contains("viewport: render::Viewport::from_logical_area"));
        assert!(encode_group.contains("target: self.output_target"));
        assert!(encode_group.contains("source_rect: Some(group_layer_source_rect(group.bounds))"));
    }

    #[test]
    #[ignore = "GPU readback diagnostic for native popup alpha investigation"]
    fn direct_premultiplied_alpha_witness_preserves_alpha_and_rgb() {
        let sample = pollster::block_on(read_direct_premultiplied_alpha_witness())
            .expect("alpha witness should render and read back");

        assert!(
            (sample[3] - 0.5).abs() <= 0.03,
            "alpha should survive as half coverage, got rgba={sample:?}"
        );
        assert!(
            (sample[0] - 0.5).abs() <= 0.03,
            "red channel should be premultiplied to half intensity, got rgba={sample:?}"
        );
        assert!(
            sample[1].abs() <= 0.01 && sample[2].abs() <= 0.01,
            "green/blue channels should remain zero, got rgba={sample:?}"
        );
    }

    #[test]
    #[ignore = "GPU readback diagnostic for native popup foreground alpha investigation"]
    fn premultiplied_quad_aa_edge_witness_preserves_fractional_coverage() {
        let mut scene = paint::Scene::new();
        scene.clear(paint::Color::rgba(0.0, 0.0, 0.0, 0.0));
        scene.push_quad(paint::Quad::unchecked_for_test(
            Rect::rounded(
                point::logical(4.0, 4.0),
                area::logical(24.0, 24.0),
                paint::Rounding::fixed(8.0),
            ),
            paint::Style {
                fill: Some(paint::Fill::Brush(paint::Brush::solid(paint::Color::rgba(
                    1.0, 0.0, 0.0, 1.0,
                )))),
                stroke: None,
                tint: None,
            },
            paint::Rasterization {
                edge_mode: paint::EdgeMode::Antialiased,
            },
            paint::Transform::identity(),
        ));

        let pixels = pollster::block_on(read_premultiplied_scene_pixels(scene, 32, 32))
            .expect("quad witness should render and read back");
        let sample = fractional_red_sample(&pixels)
            .expect("rounded quad should produce at least one fractional AA edge pixel");

        assert_premultiplied_red(sample, "quad AA edge");
    }

    #[test]
    #[ignore = "GPU readback witness for resolved native material coverage"]
    fn resolved_glass_base_witness_distinguishes_transparent_from_fallback() {
        fn scene(base: paint::GlassBase) -> paint::Scene {
            let mut scene = paint::Scene::new();
            scene.clear(paint::Color::rgba(0.0, 0.0, 0.0, 0.0));
            scene.push_pane(paint::Pane::new(
                Rect::new(point::logical(2.0, 2.0), area::logical(12.0, 12.0)),
                paint::Material::Glass(paint::Glass {
                    fallback: paint::Brush::solid(paint::Color::rgba(1.0, 0.0, 0.0, 1.0)),
                    base,
                    backdrop_layers: Vec::new(),
                    surface_layers: Vec::new(),
                }),
            ));
            scene
        }

        let transparent = pollster::block_on(read_premultiplied_scene_pixels(
            scene(paint::GlassBase::Transparent),
            16,
            16,
        ))
        .expect("transparent resolved material should render and read back");
        let fallback = pollster::block_on(read_premultiplied_scene_pixels(
            scene(paint::GlassBase::Fallback),
            16,
            16,
        ))
        .expect("fallback resolved material should render and read back");

        assert_eq!(transparent[8 * 16 + 8], [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(fallback[8 * 16 + 8], [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    #[ignore = "GPU readback witness for alpha-preserving material noise"]
    fn resolved_glass_noise_modulates_rgb_without_claiming_alpha() {
        let mut scene = paint::Scene::new();
        scene.clear(paint::Color::rgba(0.0, 0.0, 0.0, 0.0));
        scene.push_pane(paint::Pane::new(
            Rect::new(point::logical(2.0, 2.0), area::logical(12.0, 12.0)),
            paint::Material::Glass(paint::Glass {
                fallback: paint::Brush::solid(paint::Color::BLACK),
                base: paint::GlassBase::Transparent,
                backdrop_layers: Vec::new(),
                surface_layers: vec![
                    paint::SurfaceLayer::Tint {
                        brush: paint::Brush::solid(paint::Color::BLACK),
                        opacity: 0.10,
                    },
                    paint::SurfaceLayer::Noise(paint::Noise { opacity: 0.022 }),
                ],
            }),
        ));

        let pixels = pollster::block_on(read_premultiplied_scene_pixels(scene, 16, 16))
            .expect("resolved glass noise should render and read back");
        let sample = pixels[8 * 16 + 8];

        assert!(
            (sample[3] - 0.10).abs() <= 0.02,
            "noise must preserve the tint's source alpha, got {sample:?}"
        );
        assert!(
            sample[0] <= sample[3] && sample[1] <= sample[3] && sample[2] <= sample[3],
            "noise output must remain premultiplied, got {sample:?}"
        );
        assert_eq!(pixels[0], [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    #[ignore = "GPU readback diagnostic for native popup foreground alpha investigation"]
    fn premultiplied_glyph_witness_preserves_fractional_coverage() {
        let mut scene = paint::Scene::new();
        scene.clear(paint::Color::rgba(0.0, 0.0, 0.0, 0.0));
        scene.push_text(paint::Text {
            rect: Rect::new(point::logical(2.0, 2.0), area::logical(60.0, 28.0)),
            document: text::document::Document::plain("Ag")
                .with_color(text::Color::RED)
                .with_size(24.0),
            wrap: paint::TextWrap::None,
            vertical_align: paint::TextVerticalAlign::Center,
            overflow: crate::text::Overflow::Clip,
        });

        let pixels = pollster::block_on(read_premultiplied_scene_pixels(scene, 64, 32))
            .expect("glyph witness should render and read back");
        let sample = fractional_red_sample(&pixels)
            .expect("glyph mask should produce at least one fractional coverage pixel");

        assert_premultiplied_red(sample, "glyph fractional coverage");
    }

    #[test]
    #[ignore = "GPU readback diagnostic for associated group composite alpha"]
    fn premultiplied_group_witness_applies_opacity_once() {
        let mut scene = paint::Scene::new();
        scene.clear(paint::Color::rgba(0.0, 0.0, 0.0, 0.0));
        scene.push_group(paint::Group {
            bounds: Rect::new(point::logical(0.0, 0.0), area::logical(16.0, 16.0)),
            opacity: 0.5,
            items: vec![paint::Item::Quad(paint::Quad::unchecked_for_test(
                Rect::new(point::logical(2.0, 2.0), area::logical(12.0, 12.0)),
                paint::Style {
                    fill: Some(paint::Fill::Brush(paint::Brush::solid(paint::Color::rgba(
                        1.0, 0.0, 0.0, 0.5,
                    )))),
                    stroke: None,
                    tint: None,
                },
                paint::Rasterization::default(),
                paint::Transform::identity(),
            ))],
        });

        let pixels = pollster::block_on(read_premultiplied_scene_pixels(scene, 16, 16))
            .expect("group witness should render and read back");
        let sample = pixels[8 * 16 + 8];

        assert!(
            (sample[3] - 0.25).abs() <= 0.03,
            "half-alpha content at half group opacity should produce quarter alpha, got {sample:?}"
        );
        assert!(
            (sample[0] - 0.25).abs() <= 0.03,
            "associated red should receive group opacity exactly once, got {sample:?}"
        );
        assert_premultiplied_red(sample, "group opacity");
    }

    #[test]
    #[ignore = "GPU readback diagnostic for native popup sRGB premultiplied packing"]
    fn popup_pack_witness_encodes_srgb_before_premultiplying() {
        let sample = pollster::block_on(read_popup_pack_witness())
            .expect("popup pack witness should render and read back");
        let expected_alpha = 128u8;
        let expected_rgb = (srgb_encode_for_test(0.25) * 0.5 * 255.0).round() as u8;

        assert!(
            sample[3].abs_diff(expected_alpha) <= 2,
            "packed alpha should stay half coverage, got rgba bytes={sample:?}"
        );
        for (index, channel) in sample[0..3].iter().enumerate() {
            assert!(
                channel.abs_diff(expected_rgb) <= 2,
                "packed channel {index} should be sRGB-encoded then premultiplied; expected {expected_rgb}, got rgba bytes={sample:?}"
            );
            assert!(
                *channel <= sample[3].saturating_add(2),
                "premultiplied byte invariant should hold after packing; got rgba bytes={sample:?}"
            );
        }
    }

    async fn read_direct_premultiplied_alpha_witness() -> Result<[f32; 4], String> {
        let context = render::Context::new(render::context::Options {
            device_label: "wgpu_l3 alpha witness",
            backends: wgpu::Backends::PRIMARY,
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
        })
        .await
        .map_err(|error| error.to_string())?;

        let width = 16;
        let height = 16;
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let texture = context.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Alpha Witness Target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut renderer = Renderer::new(&context, render::surface::Format::from_wgpu(format));
        let mut scene = paint::Scene::new();
        scene.clear(paint::Color::rgba(0.0, 0.0, 0.0, 0.0));
        scene.push_quad(paint::Quad::unchecked_for_test(
            Rect::new(point::logical(2.0, 2.0), area::logical(12.0, 12.0)),
            paint::Style {
                fill: Some(paint::Fill::Brush(paint::Brush::solid(paint::Color::rgba(
                    1.0, 0.0, 0.0, 0.5,
                )))),
                stroke: None,
                tint: None,
            },
            paint::Rasterization {
                edge_mode: paint::EdgeMode::Hard,
            },
            paint::Transform::identity(),
        ));

        let viewport =
            render::Viewport::from_logical_area(area::logical(width as f32, height as f32), 1.0);
        let item_batch_list = item_batches(scene.items());
        let mut text_renderer_index = 0;
        renderer.quad_arena.begin_frame();
        let prepared_scene = renderer
            .prepare_scene_batches(
                &context,
                viewport,
                &item_batch_list,
                &mut text_renderer_index,
            )
            .map_err(|error| error.to_string())?;
        renderer.quad_arena.upload(&context);
        let mut text_render_error = None;
        let mut draw_passes = 0;
        let mut command_stats = CommandStats::default();
        let mut encoder =
            context
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Alpha Witness Encoder"),
                });
        {
            let _clear = begin_main_pass(&mut encoder, &view, wgpu::Color::TRANSPARENT, true);
        }
        let mut scene_encoder = SceneEncoder::new(SceneEncoderInput {
            render_context: &context,
            quad_pipeline: &renderer.quad_pipeline,
            quad_vertex_buffer: renderer.quad_arena.buffer(),
            filter_renderer: &renderer.filter_renderer,
            text_renderer: &mut renderer.text_renderer,
            retained_shapes: None,
            retained_properties: None,
            scene_properties: None,
            encoder: &mut encoder,
            base_view: &view,
            backdrop_view: &view,
            backdrop_target: render::filter::Target::from_viewport(viewport),
            clear_color: wgpu::Color::TRANSPARENT,
            viewport,
            base_dirty: true,
            inside_group: false,
            text_render_error: &mut text_render_error,
            draw_passes: &mut draw_passes,
            command_stats: &mut command_stats,
        });
        scene_encoder.encode(&prepared_scene.render_batches);

        let bytes_per_pixel = 4u32;
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
        let output_buffer_size = padded_bytes_per_row as u64 * height as u64;
        let output = context.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Alpha Witness Readback"),
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
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
        context.queue().submit(Some(encoder.finish()));

        let slice = output.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        context
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
        let x = 8usize;
        let y = 8usize;
        let offset = y * padded_bytes_per_row as usize + x * bytes_per_pixel as usize;
        let pixel = [
            data[offset] as f32 / 255.0,
            data[offset + 1] as f32 / 255.0,
            data[offset + 2] as f32 / 255.0,
            data[offset + 3] as f32 / 255.0,
        ];
        drop(data);
        output.unmap();

        Ok(pixel)
    }

    async fn read_popup_pack_witness() -> Result<[u8; 4], String> {
        let context = render::Context::new(render::context::Options {
            device_label: "wgpu_l3 popup pack witness",
            backends: wgpu::Backends::PRIMARY,
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
        })
        .await
        .map_err(|error| error.to_string())?;

        let width = 16;
        let height = 16;
        let scene_format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let output_format = wgpu::TextureFormat::Rgba8Unorm;
        let scene_texture = context.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Popup Pack Witness sRGB Scene"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: scene_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let output_texture = context.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Popup Pack Witness Output"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: output_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let scene_view = scene_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut renderer =
            Renderer::new(&context, render::surface::Format::from_wgpu(scene_format));
        let mut scene = paint::Scene::new();
        scene.clear(paint::Color::rgba(0.0, 0.0, 0.0, 0.0));
        scene.push_quad(paint::Quad::unchecked_for_test(
            Rect::new(point::logical(2.0, 2.0), area::logical(12.0, 12.0)),
            paint::Style {
                fill: Some(paint::Fill::Brush(paint::Brush::solid(paint::Color::rgba(
                    0.25, 0.25, 0.25, 0.5,
                )))),
                stroke: None,
                tint: None,
            },
            paint::Rasterization {
                edge_mode: paint::EdgeMode::Hard,
            },
            paint::Transform::identity(),
        ));

        let viewport =
            render::Viewport::from_logical_area(area::logical(width as f32, height as f32), 1.0);
        let item_batch_list = item_batches(scene.items());
        let mut text_renderer_index = 0;
        renderer.quad_arena.begin_frame();
        let prepared_scene = renderer
            .prepare_scene_batches(
                &context,
                viewport,
                &item_batch_list,
                &mut text_renderer_index,
            )
            .map_err(|error| error.to_string())?;
        renderer.quad_arena.upload(&context);
        let mut text_render_error = None;
        let mut draw_passes = 0;
        let mut command_stats = CommandStats::default();
        let mut encoder =
            context
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Popup Pack Witness Encoder"),
                });
        {
            let _clear = begin_main_pass(&mut encoder, &scene_view, wgpu::Color::TRANSPARENT, true);
        }
        let mut scene_encoder = SceneEncoder::new(SceneEncoderInput {
            render_context: &context,
            quad_pipeline: &renderer.quad_pipeline,
            quad_vertex_buffer: renderer.quad_arena.buffer(),
            filter_renderer: &renderer.filter_renderer,
            text_renderer: &mut renderer.text_renderer,
            retained_shapes: None,
            retained_properties: None,
            scene_properties: None,
            encoder: &mut encoder,
            base_view: &scene_view,
            backdrop_view: &scene_view,
            backdrop_target: render::filter::Target::from_viewport(viewport),
            clear_color: wgpu::Color::TRANSPARENT,
            viewport,
            base_dirty: true,
            inside_group: false,
            text_render_error: &mut text_render_error,
            draw_passes: &mut draw_passes,
            command_stats: &mut command_stats,
        });
        scene_encoder.encode(&prepared_scene.render_batches);
        if let Some(error) = text_render_error {
            return Err(error.to_string());
        }
        renderer.popup_packer.pack_to_view(
            &context,
            &mut encoder,
            &scene_view,
            &output_view,
            output_format,
        );

        let bytes_per_pixel = 4u32;
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
        let output_buffer_size = padded_bytes_per_row as u64 * height as u64;
        let output = context.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Popup Pack Witness Readback"),
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &output_texture,
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
        context.queue().submit(Some(encoder.finish()));

        let slice = output.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        context
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
        let x = 8usize;
        let y = 8usize;
        let offset = y * padded_bytes_per_row as usize + x * bytes_per_pixel as usize;
        let pixel = [
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ];
        drop(data);
        output.unmap();

        Ok(pixel)
    }

    fn srgb_encode_for_test(value: f32) -> f32 {
        if value <= 0.003_130_8 {
            12.92 * value
        } else {
            1.055 * value.powf(1.0 / 2.4) - 0.055
        }
    }

    async fn read_premultiplied_scene_pixels(
        scene: paint::Scene,
        width: u32,
        height: u32,
    ) -> Result<Vec<[f32; 4]>, String> {
        let context = render::Context::new(render::context::Options {
            device_label: "wgpu_l3 foreground alpha witness",
            backends: wgpu::Backends::PRIMARY,
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
        })
        .await
        .map_err(|error| error.to_string())?;

        let format = wgpu::TextureFormat::Rgba8Unorm;
        let texture = context.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Foreground Alpha Witness Target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut renderer = Renderer::new(&context, render::surface::Format::from_wgpu(format));
        let viewport =
            render::Viewport::from_logical_area(area::logical(width as f32, height as f32), 1.0);
        let item_batch_list = item_batches(scene.items());
        let text_batch_count = glyph_batch_count(&item_batch_list);
        if text_batch_count > 0 {
            renderer
                .text_renderer
                .prepare_frame(&context, text_batch_count);
        }
        let mut text_renderer_index = 0;
        renderer.quad_arena.begin_frame();
        let prepared_scene = renderer
            .prepare_scene_batches(
                &context,
                viewport,
                &item_batch_list,
                &mut text_renderer_index,
            )
            .map_err(|error| error.to_string())?;
        renderer.quad_arena.upload(&context);
        let mut text_render_error = None;
        let mut draw_passes = 0;
        let mut command_stats = CommandStats::default();
        let mut encoder =
            context
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Foreground Alpha Witness Encoder"),
                });
        {
            let _clear = begin_main_pass(&mut encoder, &view, wgpu::Color::TRANSPARENT, true);
        }
        let mut scene_encoder = SceneEncoder::new(SceneEncoderInput {
            render_context: &context,
            quad_pipeline: &renderer.quad_pipeline,
            quad_vertex_buffer: renderer.quad_arena.buffer(),
            filter_renderer: &renderer.filter_renderer,
            text_renderer: &mut renderer.text_renderer,
            retained_shapes: None,
            retained_properties: None,
            scene_properties: None,
            encoder: &mut encoder,
            base_view: &view,
            backdrop_view: &view,
            backdrop_target: render::filter::Target::from_viewport(viewport),
            clear_color: wgpu::Color::TRANSPARENT,
            viewport,
            base_dirty: true,
            inside_group: false,
            text_render_error: &mut text_render_error,
            draw_passes: &mut draw_passes,
            command_stats: &mut command_stats,
        });
        scene_encoder.encode(&prepared_scene.render_batches);

        let bytes_per_pixel = 4u32;
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
        let output_buffer_size = padded_bytes_per_row as u64 * height as u64;
        let output = context.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Foreground Alpha Witness Readback"),
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
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
        context.queue().submit(Some(encoder.finish()));

        let slice = output.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        context
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

    fn fractional_red_sample(pixels: &[[f32; 4]]) -> Option<[f32; 4]> {
        pixels
            .iter()
            .copied()
            .find(|pixel| pixel[3] > 0.1 && pixel[3] < 0.9 && pixel[0] > 0.02)
    }

    fn assert_premultiplied_red(sample: [f32; 4], label: &str) {
        assert!(
            sample[0] <= sample[3] + 0.04,
            "{label} red must not exceed alpha on premultiplied output, got rgba={sample:?}"
        );
        assert!(
            (sample[0] - sample[3]).abs() <= 0.06,
            "{label} red should track fractional alpha, got rgba={sample:?}"
        );
        assert!(
            sample[1] <= 0.02 && sample[2] <= 0.02,
            "{label} green/blue should stay near zero, got rgba={sample:?}"
        );
    }
}
