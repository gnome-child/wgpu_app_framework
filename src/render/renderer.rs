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
    pub(crate) acquire_outcome: render::AcquireOutcome,
    pub(crate) present_timing: Option<render::PresentTiming>,
}

pub struct Renderer {
    format: wgpu::TextureFormat,
    quad_pipeline: wgpu::RenderPipeline,
    quad_arena: render::quad::Arena,
    filter_renderer: render::filter::Renderer,
    popup_packer: render::popup_pack::Packer,
    text_renderer: render::text_renderer::TextRenderer,
}

enum RenderBatch {
    Shapes(render::quad::Batch),
    Pane(PreparedPane),
    Text { renderer_index: usize },
    PushClip(paint::Clip),
    PopClip,
    Group(PreparedGroup),
}

struct PreparedPane {
    pane: paint::Pane,
    base: Option<render::quad::Batch>,
    surface_layers: Vec<Option<render::quad::Batch>>,
}

struct PreparedScene {
    render_batches: Vec<RenderBatch>,
    stats: DrawStats,
}

struct PreparedGroup {
    bounds: Rect,
    opacity: f32,
    render_batches: Vec<RenderBatch>,
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
}

struct SceneEncoderInput<'a> {
    render_context: &'a render::Context,
    quad_pipeline: &'a wgpu::RenderPipeline,
    quad_vertex_buffer: &'a wgpu::Buffer,
    filter_renderer: &'a render::filter::Renderer,
    text_renderer: &'a mut render::text_renderer::TextRenderer,
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
}

impl Renderer {
    pub fn new(render_context: &render::Context, format: wgpu::TextureFormat) -> Self {
        Self {
            format,
            quad_pipeline: render::quad::pipeline(render_context, format),
            quad_arena: render::quad::Arena::new(render_context),
            filter_renderer: render::filter::Renderer::new(render_context, format),
            popup_packer: render::popup_pack::Packer::new(render_context),
            text_renderer: render::text_renderer::TextRenderer::new(render_context, format),
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
        stats.geometry_buffer_creations = usize::from(geometry_upload.buffer_created);
        stats.scene_items = scene.items().len();
        let batch_prepare = batch_prepare_started.elapsed();

        let preserve_surface_alpha =
            canvas.composite_alpha_mode() == wgpu::CompositeAlphaMode::PreMultiplied;
        let surface_format = canvas.surface().config().format;
        let pack_premultiplied_surface = render::supports_windows_premultiplied_popup_pack(
            surface_format,
            canvas.composite_alpha_mode(),
        );
        if pack_premultiplied_surface {
            debug_assert_eq!(
                self.format,
                render::scene_format_for_surface_format(surface_format),
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
        let filter_target = if preserve_surface_alpha && !pack_premultiplied_surface {
            render::filter::Target::from_viewport(main_viewport)
        } else {
            self.filter_renderer.prepare(render_context, canvas)
        };
        let quad_pipeline = &self.quad_pipeline;
        let quad_vertex_buffer = self.quad_arena.buffer();
        let filter_renderer = &self.filter_renderer;
        let popup_packer = &self.popup_packer;
        let text_renderer = &mut self.text_renderer;
        let mut text_render_error = None;
        let mut draw_passes = 0;

        let surface_report = if pack_premultiplied_surface {
            canvas.draw(render_context, |encoder, frame| {
                let view = frame.create_view();
                filter_renderer.clear_composition(encoder, clear_color);
                let Some(composition_view) = filter_renderer.composition_view() else {
                    return;
                };
                let mut scene_encoder = SceneEncoder::new(SceneEncoderInput {
                    render_context,
                    quad_pipeline,
                    quad_vertex_buffer,
                    filter_renderer,
                    text_renderer,
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
                });
                scene_encoder.encode(&prepared_scene.render_batches);
                if text_render_error.is_some() {
                    return;
                }

                popup_packer.pack_to_view(
                    render_context,
                    encoder,
                    composition_view,
                    &view,
                    surface_format,
                );
            })?
        } else if preserve_surface_alpha {
            canvas.draw(render_context, |encoder, frame| {
                let view = frame.create_view();
                {
                    let _clear = begin_main_pass(encoder, &view, clear_color, true);
                }
                let mut scene_encoder = SceneEncoder::new(SceneEncoderInput {
                    render_context,
                    quad_pipeline,
                    quad_vertex_buffer,
                    filter_renderer,
                    text_renderer,
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
                });
                scene_encoder.encode(&prepared_scene.render_batches);
            })?
        } else {
            canvas.draw(render_context, |encoder, frame| {
                let view = frame.create_view();
                filter_renderer.clear_composition(encoder, clear_color);
                let Some(composition_view) = filter_renderer.composition_view() else {
                    return;
                };
                let mut scene_encoder = SceneEncoder::new(SceneEncoderInput {
                    render_context,
                    quad_pipeline,
                    quad_vertex_buffer,
                    filter_renderer,
                    text_renderer,
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
                });
                scene_encoder.encode(&prepared_scene.render_batches);
                if text_render_error.is_some() {
                    return;
                }

                filter_renderer.blit_to_view(render_context, encoder, &view, filter_target);
            })?
        };

        if let Some(error) = text_render_error {
            return Err(error.into());
        }

        self.text_renderer.trim();
        stats.draw_passes = draw_passes;
        stats.filter_layer_pool_entries = self.filter_renderer.layer_pool_entries();
        stats.filter_scratch_pool_entries = self.filter_renderer.scratch_pool_entries();

        Ok(DrawReport {
            stats,
            batch_prepare,
            acquire_outcome: surface_report.acquire(),
            present_timing: surface_report.present_timing(),
        })
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
            ..DrawStats::default()
        };

        for batch in item_batch_list {
            match batch {
                ItemBatch::Shapes(shapes) => {
                    if let Some(batch) =
                        render::quad::prepare_batch(&mut self.quad_arena, viewport, shapes)
                    {
                        stats.quad_vertices += batch.vertex_count() as usize;
                        render_batches.push(RenderBatch::Shapes(batch));
                    }
                }
                ItemBatch::Pane(pane) => {
                    let prepared = self.prepare_pane(viewport, pane);
                    if matches!(pane.material, paint::Material::Solid(_)) {
                        if let Some(batch) = prepared.base {
                            render_batches.push(RenderBatch::Shapes(batch));
                        }
                    } else {
                        render_batches.push(RenderBatch::Pane(prepared));
                    }
                }
                ItemBatch::PushClip(clip) => {
                    render_batches.push(RenderBatch::PushClip(**clip));
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
                        bounds: group.bounds,
                        opacity: group.opacity,
                        render_batches: prepared.render_batches,
                    }));
                }
                ItemBatch::Glyphs(glyphs) => {
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
                        render_batches.push(RenderBatch::Text {
                            renderer_index: *text_renderer_index,
                        });
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

    fn prepare_pane(&mut self, viewport: render::Viewport, pane: &paint::Pane) -> PreparedPane {
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

        PreparedPane {
            pane: pane.clone(),
            base: base_brush.and_then(|brush| {
                prepare_brush_batch(&mut self.quad_arena, viewport, pane.rect, brush)
            }),
            surface_layers: surface_brushes
                .into_iter()
                .map(|brush| {
                    brush.and_then(|brush| {
                        prepare_brush_batch(&mut self.quad_arena, viewport, pane.rect, brush)
                    })
                })
                .collect(),
        }
    }
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

    fn add(&mut self, other: Self) {
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
        self.quad_vertices += other.quad_vertices;
        self.geometry_upload_bytes += other.geometry_upload_bytes;
        self.geometry_buffer_creations += other.geometry_buffer_creations;
        self.clip_batches += other.clip_batches;
        self.group_composites += other.group_composites;
        self.filter_layer_pool_entries += other.filter_layer_pool_entries;
        self.filter_scratch_pool_entries += other.filter_scratch_pool_entries;
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

fn is_draw_batch(batch: &RenderBatch) -> bool {
    matches!(batch, RenderBatch::Shapes(_) | RenderBatch::Text { .. })
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
        }
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
                RenderBatch::Shapes(_) | RenderBatch::Text { .. } => unreachable!(),
                RenderBatch::Pane(pane) => self.encode_pane(pane),
                RenderBatch::PushClip(clip) => self.push_clip(*clip),
                RenderBatch::PopClip => self.composite_clip_layer(),
                RenderBatch::Group(group) => self.encode_group(group),
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
                    RenderBatch::Shapes(batch) => {
                        pass.set_pipeline(self.quad_pipeline);
                        pass.set_scissor_rect(
                            scissor.x(),
                            scissor.y(),
                            scissor.width(),
                            scissor.height(),
                        );
                        pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
                        pass.draw(batch.vertex_range(), 0..1);
                    }
                    RenderBatch::Text { renderer_index } => {
                        if let Err(error) = self.text_renderer.render(*renderer_index, &mut pass) {
                            *self.text_render_error = Some(error);
                            break;
                        }
                    }
                    RenderBatch::Pane(_)
                    | RenderBatch::PushClip(_)
                    | RenderBatch::PopClip
                    | RenderBatch::Group(_) => unreachable!(),
                }
            }
        }
        self.mark_current_dirty();
    }

    fn encode_shapes(&mut self, batch: &render::quad::Batch) {
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
            pass.set_pipeline(self.quad_pipeline);
            pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
            pass.draw(batch.vertex_range(), 0..1);
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
                            self.filter_renderer.draw(render::filter::FilterDraw {
                                render_context: self.render_context,
                                encoder: self.encoder,
                                target: self.output_target,
                                source,
                                output,
                                filter,
                                scissor: Some(scissor),
                            });
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
                            self.filter_renderer.draw(render::filter::FilterDraw {
                                render_context: self.render_context,
                                encoder: self.encoder,
                                target: self.output_target,
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
                            self.mark_current_dirty();
                        }
                    }
                }
            }
        }
    }

    fn encode_group(&mut self, group: &PreparedGroup) {
        if group.opacity <= 0.0 {
            return;
        }

        if self.clip_stack.iter().any(|frame| frame.clip().is_some()) {
            self.encode_batches(&group.render_batches);
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

        {
            let group_view = group_layer.view();
            let mut group_encoder = SceneEncoder::new(SceneEncoderInput {
                render_context: self.render_context,
                quad_pipeline: self.quad_pipeline,
                quad_vertex_buffer: self.quad_vertex_buffer,
                filter_renderer: self.filter_renderer,
                text_renderer: self.text_renderer,
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
            });
            group_encoder.encode(&group.render_batches);
        }

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
                    opacity: group.opacity,
                });
            self.mark_current_dirty();
        }
        self.filter_renderer.recycle_layer(group_layer);
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
        self.clip_stack.push(ClipFrame::Layer {
            clip,
            layer_index,
            dirty: false,
        });
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

fn group_layer_source_rect(bounds: Rect) -> Rect {
    Rect::new(point::logical(0.0, 0.0), bounds.area)
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
    let left = (rect.origin.x() * scale_factor).floor().max(0.0) as u32;
    let top = (rect.origin.y() * scale_factor).floor().max(0.0) as u32;
    let right = ((rect.origin.x() + rect.area.width()) * scale_factor)
        .ceil()
        .max(0.0)
        .min(canvas_width as f32) as u32;
    let bottom = ((rect.origin.y() + rect.area.height()) * scale_factor)
        .ceil()
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
    fn draw_runs_cross_shape_text_pipeline_changes_but_stop_at_semantic_boundaries() {
        let batches = [
            RenderBatch::Shapes(render::quad::Batch::from_vertex_range(0..6)),
            RenderBatch::Text { renderer_index: 1 },
            RenderBatch::PopClip,
            RenderBatch::Text { renderer_index: 2 },
        ];

        assert_eq!(draw_run_end(&batches, 0), 2);
        assert!(!is_draw_batch(&batches[2]));
        assert_eq!(draw_run_end(&batches, 3), 4);
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
        let context = render::Context::new(render::ContextOptions {
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

        let mut renderer = Renderer::new(&context, format);
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
        let context = render::Context::new(render::ContextOptions {
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

        let mut renderer = Renderer::new(&context, scene_format);
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
        let context = render::Context::new(render::ContextOptions {
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

        let mut renderer = Renderer::new(&context, format);
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
