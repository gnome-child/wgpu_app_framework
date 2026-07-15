use crate::paint;
use crate::render;
use crate::render::batch;
#[cfg(test)]
use crate::text::layout as text_layout;
use crate::text::layout::{InlineCache, InlineStats};

use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Weak};

use thiserror::Error;

pub(in crate::render) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Prepare(#[from] glyphon::PrepareError),

    #[error(transparent)]
    Render(#[from] glyphon::RenderError),
}

pub(in crate::render) struct TextRenderer {
    cache: glyphon::Cache,
    atlas: glyphon::TextAtlas,
    renderers: Vec<glyphon::TextRenderer>,
    viewports: Vec<glyphon::Viewport>,
    swash_cache: glyphon::SwashCache,
    inline_cache: InlineCache,
    retained: HashMap<render::retained::ResourceKey, RetainedText>,
    retained_transforms: HashMap<RetainedTextTransform, RetainedTransformViewport>,
}

struct RetainedText {
    owners: Vec<Weak<crate::scene::Node>>,
    renderer: glyphon::TextRenderer,
    viewport: glyphon::Viewport,
    has_text: bool,
}

struct RetainedTransformViewport {
    owners: Vec<Weak<crate::scene::Node>>,
    viewport: glyphon::Viewport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RetainedTextTransform {
    scroll: crate::composition::tree::NodeId,
    target: Option<crate::composition::tree::NodeId>,
    resolution: [u32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(in crate::render) struct RetainedBatch {
    key: render::retained::ResourceKey,
    transform: Option<RetainedTextTransform>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in crate::render) struct RetainedTextReport {
    pub(in crate::render) batch: Option<RetainedBatch>,
    pub(in crate::render) stats: InlineStats,
    pub(in crate::render) prepare_calls: usize,
    pub(in crate::render) resource_creations: usize,
    pub(in crate::render) resource_removals: usize,
}

struct PreparedText<'a> {
    buffer: PreparedTextBuffer<'a>,
    left: f32,
    top: f32,
    bounds: glyphon::TextBounds,
    default_color: glyphon::Color,
    stats: InlineStats,
}

enum PreparedTextBuffer<'a> {
    Shared(Rc<RefCell<glyphon::cosmic_text::Buffer>>),
    Borrowed(Ref<'a, glyphon::cosmic_text::Buffer>),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in crate::render) struct TextBatchReport {
    pub(in crate::render) has_text: bool,
    pub(in crate::render) stats: InlineStats,
}

impl TextRenderer {
    pub(in crate::render) fn new(
        render_context: &render::Context,
        format: wgpu::TextureFormat,
    ) -> Self {
        let cache = glyphon::Cache::new(render_context.device());
        let atlas = glyphon::TextAtlas::new(
            render_context.device(),
            render_context.queue(),
            &cache,
            format,
        );

        Self {
            cache,
            atlas,
            renderers: Vec::new(),
            viewports: Vec::new(),
            swash_cache: glyphon::SwashCache::new(),
            inline_cache: InlineCache::new(),
            retained: HashMap::new(),
            retained_transforms: HashMap::new(),
        }
    }

    pub(in crate::render) fn prepare_frame(
        &mut self,
        render_context: &render::Context,
        batch_count: usize,
    ) {
        self.ensure_renderers(batch_count, render_context);
    }

    pub(in crate::render) fn prepare_batch(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        renderer_index: usize,
        glyphs: &[batch::Glyph<'_>],
    ) -> Result<TextBatchReport> {
        if glyphs.is_empty() {
            return Ok(TextBatchReport::default());
        }

        self.update_viewport(render_context, renderer_index, viewport);
        let Self {
            atlas,
            renderers,
            viewports,
            swash_cache,
            inline_cache,
            ..
        } = self;
        prepare_glyphs(
            render_context,
            viewport.scale_factor(),
            [0.0; 2],
            inline_cache,
            atlas,
            swash_cache,
            &mut renderers[renderer_index],
            &viewports[renderer_index],
            glyphs,
        )
    }

    pub(in crate::render) fn prepare_retained(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        node: &Arc<crate::scene::Node>,
        content_index: usize,
        glyphs: &[batch::Glyph<'_>],
        target_origin: [f32; 2],
        target_size: [f32; 2],
        scroll: Option<crate::composition::tree::NodeId>,
        scroll_root: Option<crate::composition::tree::NodeId>,
    ) -> Result<RetainedTextReport> {
        let resource_removals = self.prune_retained();
        let (transform, transform_creation) = self.retain_transform_viewport(
            render_context,
            viewport,
            node,
            target_size,
            scroll,
            scroll_root,
        );
        let key = render::retained::ResourceKey::for_target(
            node,
            content_index,
            0,
            viewport.scale_factor(),
            target_origin,
            target_size,
        );
        if let Some(entry) = self.retained.get_mut(&key) {
            entry.owners.retain(|owner| owner.strong_count() > 0);
            if !entry
                .owners
                .iter()
                .filter_map(Weak::upgrade)
                .any(|owner| Arc::ptr_eq(&owner, node))
            {
                entry.owners.push(Arc::downgrade(node));
            }
            return Ok(RetainedTextReport {
                batch: entry.has_text.then_some(RetainedBatch { key, transform }),
                resource_removals,
                resource_creations: transform_creation,
                ..RetainedTextReport::default()
            });
        }

        let mut renderer = glyphon::TextRenderer::new(
            &mut self.atlas,
            render_context.device(),
            wgpu::MultisampleState::default(),
            None,
        );
        let mut glyph_viewport = glyphon::Viewport::new(render_context.device(), &self.cache);
        let target_viewport = render::Viewport::from_logical_area(
            crate::geometry::area::logical(target_size[0], target_size[1]),
            viewport.scale_factor(),
        );
        update_glyphon_viewport(render_context, &mut glyph_viewport, target_viewport);
        let grid = paint::Grid::new(viewport.scale_factor());
        let raster_origin = [
            grid.snap_text_origin(target_origin[0]),
            grid.snap_text_origin(target_origin[1]),
        ];
        let report = prepare_glyphs(
            render_context,
            viewport.scale_factor(),
            raster_origin,
            &mut self.inline_cache,
            &mut self.atlas,
            &mut self.swash_cache,
            &mut renderer,
            &glyph_viewport,
            glyphs,
        )?;
        let has_text = report.has_text;
        self.retained.insert(
            key,
            RetainedText {
                owners: vec![Arc::downgrade(node)],
                renderer,
                viewport: glyph_viewport,
                has_text,
            },
        );

        Ok(RetainedTextReport {
            batch: has_text.then_some(RetainedBatch { key, transform }),
            stats: report.stats,
            prepare_calls: 1,
            resource_creations: 1 + transform_creation,
            resource_removals,
        })
    }

    pub(in crate::render) fn render(
        &mut self,
        renderer_index: usize,
        pass: &mut wgpu::RenderPass<'_>,
    ) -> Result<()> {
        self.renderers[renderer_index]
            .render(&self.atlas, &self.viewports[renderer_index], pass)
            .map_err(Error::from)
    }

    pub(in crate::render) fn render_retained(
        &mut self,
        render_context: &render::Context,
        batch: RetainedBatch,
        scroll_translation: [f32; 2],
        scale_factor: f32,
        pass: &mut wgpu::RenderPass<'_>,
    ) -> Result<usize> {
        let Self {
            atlas,
            retained,
            retained_transforms,
            ..
        } = self;
        let Some(entry) = retained.get(&batch.key) else {
            return Ok(0);
        };
        let mut property_upload_bytes = 0;
        let viewport = if let Some(transform) = batch.transform {
            let Some(transform) = retained_transforms.get_mut(&transform) else {
                return Ok(0);
            };
            let grid = paint::Grid::new(scale_factor);
            let offset = [
                grid.snap_text_origin(scroll_translation[0]) as i32,
                grid.snap_text_origin(scroll_translation[1]) as i32,
            ];
            if transform
                .viewport
                .update_render_offset(render_context.queue(), offset)
            {
                property_upload_bytes = std::mem::size_of::<[u32; 4]>();
            }
            &transform.viewport
        } else {
            &entry.viewport
        };
        entry
            .renderer
            .render(atlas, viewport, pass)
            .map_err(Error::from)?;
        Ok(property_upload_bytes)
    }

    pub(in crate::render) fn retained_resource_count(&self) -> usize {
        self.retained
            .len()
            .saturating_add(self.retained_transforms.len())
    }

    pub(in crate::render) fn retained_resource_bytes(&self) -> usize {
        self.retained
            .len()
            .saturating_mul(std::mem::size_of::<RetainedText>())
            .saturating_add(
                self.retained_transforms
                    .len()
                    .saturating_mul(std::mem::size_of::<RetainedTransformViewport>()),
            )
    }

    pub(in crate::render) fn collect_retained(&mut self) -> usize {
        self.prune_retained()
    }

    pub(in crate::render) fn trim(&mut self) -> Result<()> {
        self.prune_retained();
        self.atlas.trim();
        for retained in self.retained.values() {
            retained.renderer.retain_prepared(&mut self.atlas)?;
        }
        Ok(())
    }

    fn update_viewport(
        &mut self,
        render_context: &render::Context,
        renderer_index: usize,
        viewport: render::Viewport,
    ) {
        update_glyphon_viewport(
            render_context,
            &mut self.viewports[renderer_index],
            viewport,
        );
    }

    fn ensure_renderers(&mut self, count: usize, render_context: &render::Context) {
        while self.renderers.len() < count {
            self.renderers.push(glyphon::TextRenderer::new(
                &mut self.atlas,
                render_context.device(),
                wgpu::MultisampleState::default(),
                None,
            ));
            self.viewports
                .push(glyphon::Viewport::new(render_context.device(), &self.cache));
        }
    }

    fn prune_retained(&mut self) -> usize {
        let before = self.retained.len();
        self.retained
            .retain(|_, entry| entry.owners.iter().any(|owner| owner.strong_count() > 0));
        let retained_removed = before.saturating_sub(self.retained.len());
        let transforms_before = self.retained_transforms.len();
        self.retained_transforms.retain(|_, entry| {
            entry.owners.retain(|owner| owner.strong_count() > 0);
            !entry.owners.is_empty()
        });
        retained_removed
            .saturating_add(transforms_before.saturating_sub(self.retained_transforms.len()))
    }

    fn retain_transform_viewport(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        node: &Arc<crate::scene::Node>,
        target_size: [f32; 2],
        scroll: Option<crate::composition::tree::NodeId>,
        target: Option<crate::composition::tree::NodeId>,
    ) -> (Option<RetainedTextTransform>, usize) {
        let Some(scroll) = scroll else {
            return (None, 0);
        };
        let target_viewport = render::Viewport::from_logical_area(
            crate::geometry::area::logical(target_size[0], target_size[1]),
            viewport.scale_factor(),
        );
        let physical = target_viewport.physical_area();
        let key = RetainedTextTransform {
            scroll,
            target,
            resolution: [physical.width(), physical.height()],
        };
        if let Some(entry) = self.retained_transforms.get_mut(&key) {
            entry.owners.retain(|owner| owner.strong_count() > 0);
            if !entry
                .owners
                .iter()
                .filter_map(Weak::upgrade)
                .any(|owner| Arc::ptr_eq(&owner, node))
            {
                entry.owners.push(Arc::downgrade(node));
            }
            return (Some(key), 0);
        }

        let mut glyph_viewport = glyphon::Viewport::new(render_context.device(), &self.cache);
        update_glyphon_viewport(render_context, &mut glyph_viewport, target_viewport);
        self.retained_transforms.insert(
            key,
            RetainedTransformViewport {
                owners: vec![Arc::downgrade(node)],
                viewport: glyph_viewport,
            },
        );
        (Some(key), 1)
    }
}

fn prepare_glyphs(
    render_context: &render::Context,
    scale_factor: f32,
    raster_origin: [f32; 2],
    inline_cache: &mut InlineCache,
    atlas: &mut glyphon::TextAtlas,
    swash_cache: &mut glyphon::SwashCache,
    renderer: &mut glyphon::TextRenderer,
    viewport: &glyphon::Viewport,
    glyphs: &[batch::Glyph<'_>],
) -> Result<TextBatchReport> {
    let mut prepared = Vec::with_capacity(glyphs.len());
    let mut stats = InlineStats::default();

    for glyph in glyphs {
        match glyph {
            batch::Glyph::Text(text) => {
                if let Some(glyph) = prepare_text(inline_cache, text, scale_factor) {
                    stats.add(glyph.stats);
                    prepared.push(glyph);
                }
            }
            batch::Glyph::TextViewport(text) => {
                prepared.extend(prepare_text_viewport(text, scale_factor));
            }
            batch::Glyph::Icon(icon) => {
                if let Some(glyph) = prepare_icon(inline_cache, icon, scale_factor) {
                    stats.add(glyph.stats);
                    prepared.push(glyph);
                }
            }
        }
    }

    if prepared.is_empty() {
        return Ok(TextBatchReport {
            has_text: false,
            stats,
        });
    }

    for text in &mut prepared {
        localize_text_area(
            &mut text.left,
            &mut text.top,
            &mut text.bounds,
            raster_origin,
        );
    }

    let borrowed = prepared
        .iter()
        .filter_map(|text| match &text.buffer {
            PreparedTextBuffer::Shared(buffer) => Some(buffer.borrow()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut borrowed_index = 0_usize;
    let text_areas = prepared
        .iter()
        .map(|text| {
            let buffer = match &text.buffer {
                PreparedTextBuffer::Borrowed(buffer) => buffer,
                PreparedTextBuffer::Shared(_) => {
                    let buffer = &*borrowed[borrowed_index];
                    borrowed_index += 1;
                    buffer
                }
            };

            glyphon::TextArea {
                buffer,
                left: text.left,
                top: text.top,
                scale: scale_factor,
                bounds: text.bounds,
                default_color: text.default_color,
                custom_glyphs: &[],
            }
        })
        .collect::<Vec<_>>();

    renderer.prepare(
        render_context.device(),
        render_context.queue(),
        inline_cache.font_system_mut(),
        atlas,
        viewport,
        text_areas,
        swash_cache,
    )?;

    Ok(TextBatchReport {
        has_text: true,
        stats,
    })
}

fn localize_text_area(
    left: &mut f32,
    top: &mut f32,
    bounds: &mut glyphon::TextBounds,
    raster_origin: [f32; 2],
) {
    let raster_origin_i32 = [raster_origin[0] as i32, raster_origin[1] as i32];
    *left -= raster_origin[0];
    *top -= raster_origin[1];
    bounds.left -= raster_origin_i32[0];
    bounds.right -= raster_origin_i32[0];
    bounds.top -= raster_origin_i32[1];
    bounds.bottom -= raster_origin_i32[1];
}

fn update_glyphon_viewport(
    render_context: &render::Context,
    viewport: &mut glyphon::Viewport,
    target: render::Viewport,
) {
    let physical_area = target.physical_area();
    viewport.update(
        render_context.queue(),
        glyphon::Resolution {
            width: physical_area.width(),
            height: physical_area.height(),
        },
    );
}

fn prepare_text(
    inline_cache: &mut InlineCache,
    text: &paint::Text,
    scale_factor: f32,
) -> Option<PreparedText<'static>> {
    let grid = paint::Grid::new(scale_factor);
    let width = text.rect.area.width().max(0.0);
    let height = text.rect.area.height().max(0.0);
    let prepared = inline_cache.prepare_text(
        &text.document,
        width,
        height,
        wrap(text.wrap),
        text.overflow,
    )?;

    let clip_left = text.rect.origin.x() * scale_factor;
    let clip_top = text.rect.origin.y() * scale_factor;
    let clip_right = clip_left + width * scale_factor;
    let clip_bottom = clip_top + height * scale_factor;
    let left = grid.snap_text_origin(text.rect.origin.x());
    let top = grid.snap_centered_text_origin(text.rect.origin.y(), height, prepared.content_height);

    Some(PreparedText {
        buffer: PreparedTextBuffer::Shared(prepared.buffer),
        left,
        top,
        bounds: glyphon::TextBounds {
            left: clip_left.floor() as i32,
            top: clip_top.floor() as i32,
            right: clip_right.ceil() as i32,
            bottom: clip_bottom.ceil() as i32,
        },
        default_color: prepared.default_color,
        stats: prepared.stats,
    })
}

fn prepare_text_viewport<'a>(
    viewport: &'a paint::TextViewport,
    scale_factor: f32,
) -> impl Iterator<Item = PreparedText<'a>> + 'a {
    viewport.surfaces.iter().filter_map(move |surface| {
        prepare_text_surface_in_bounds(surface, viewport.rect, scale_factor)
    })
}

fn prepare_text_surface_in_bounds<'a>(
    text: &'a paint::TextSurface,
    bounds_rect: crate::paint::Rect,
    scale_factor: f32,
) -> Option<PreparedText<'a>> {
    let grid = paint::Grid::new(scale_factor);
    let width = text.rect.area.width().max(0.0);
    let height = text.rect.area.height().max(0.0);
    if width <= 0.0 || height <= 0.0 {
        return None;
    }

    let clip_left = bounds_rect.origin.x() * scale_factor;
    let clip_top = bounds_rect.origin.y() * scale_factor;
    let clip_right = clip_left + bounds_rect.area.width().max(0.0) * scale_factor;
    let clip_bottom = clip_top + bounds_rect.area.height().max(0.0) * scale_factor;
    let left = grid.snap_text_origin(text.rect.origin.x());
    let top = grid.snap_text_origin(text.rect.origin.y());

    Some(PreparedText {
        buffer: PreparedTextBuffer::Borrowed(text.buffer.borrow()),
        left,
        top,
        bounds: glyphon::TextBounds {
            left: clip_left.floor() as i32,
            top: clip_top.floor() as i32,
            right: clip_right.ceil() as i32,
            bottom: clip_bottom.ceil() as i32,
        },
        default_color: glyphon_color_from_linear_paint(text.default_color),
        stats: InlineStats::default(),
    })
}

fn wrap(wrap: paint::TextWrap) -> glyphon::Wrap {
    match wrap {
        paint::TextWrap::WordOrGlyph => glyphon::Wrap::WordOrGlyph,
        paint::TextWrap::None => glyphon::Wrap::None,
    }
}

fn prepare_icon(
    inline_cache: &mut InlineCache,
    icon: &paint::Icon,
    scale_factor: f32,
) -> Option<PreparedText<'static>> {
    let Some(glyph) = icon.icon.glyph() else {
        log::debug!("skipping missing icon glyph: {:?}", icon.icon);
        return None;
    };

    let width = icon.rect.area.width().max(0.0);
    let height = icon.rect.area.height().max(0.0);
    let prepared = inline_cache.prepare_icon(glyph, icon.size, width, height)?;
    let buffer_height = height.min(prepared.line_height);

    let clip_left = icon.rect.origin.x() * scale_factor;
    let clip_top = icon.rect.origin.y() * scale_factor;
    let clip_right = clip_left + width * scale_factor;
    let clip_bottom = clip_top + height * scale_factor;
    let left = clip_left;
    let top = (icon.rect.origin.y() + (height - buffer_height).max(0.0) * 0.5) * scale_factor;

    Some(PreparedText {
        buffer: PreparedTextBuffer::Shared(prepared.buffer),
        left,
        top,
        bounds: glyphon::TextBounds {
            left: clip_left.floor() as i32,
            top: clip_top.floor() as i32,
            right: clip_right.ceil() as i32,
            bottom: clip_bottom.ceil() as i32,
        },
        default_color: glyphon_color_from_linear_paint(icon.color),
        stats: prepared.stats,
    })
}

fn glyphon_color_from_linear_paint(color: paint::Color) -> glyphon::Color {
    glyphon::Color::rgba(
        crate::color::linear_unit_to_srgb_byte(color.r),
        crate::color::linear_unit_to_srgb_byte(color.g),
        crate::color::linear_unit_to_srgb_byte(color.b),
        crate::color::unit_to_byte(color.a),
    )
}

#[cfg(test)]
mod tests {
    use crate::geometry::{area, point};
    use crate::paint::{self, Rect};
    use crate::text::document::{Align, Block, Run, Style, Weight};
    use crate::{icon, text};

    use super::*;

    fn centered_text(document: text::document::Document, height: f32) -> paint::Text {
        paint::Text {
            rect: Rect::new(point::logical(4.0, 7.0), area::logical(240.0, height)),
            document,
            wrap: paint::TextWrap::None,
            vertical_align: paint::TextVerticalAlign::Center,
            overflow: crate::text::Overflow::Clip,
        }
    }

    fn label_text(
        value: &str,
        color: text::Color,
        size: f32,
        weight: Weight,
        origin_x: f32,
    ) -> paint::Text {
        let mut block = Block::new(Align::Start);
        block.push_run(Run::new(
            value,
            Style::default()
                .with_color(color)
                .with_size(size)
                .with_weight(weight),
        ));

        paint::Text {
            rect: Rect::new(point::logical(origin_x, 0.0), area::logical(160.0, 22.0)),
            document: text::document::Document::from_block(block),
            wrap: paint::TextWrap::None,
            vertical_align: paint::TextVerticalAlign::Center,
            overflow: crate::text::Overflow::Clip,
        }
    }

    fn document_height(document: &text::document::Document) -> f32 {
        let mut font_system = text_layout::glyphon_font_system();
        text_layout::measure_document_with_glyphon(
            &mut font_system,
            document,
            text::layout::Measure::bounded(area::logical(240.0, 1_000.0)),
        )
        .height()
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.01,
            "expected {actual} to be within 0.01 of {expected}"
        );
    }

    #[test]
    fn retained_text_localizes_after_global_physical_grid_preparation() {
        let mut left = 195.0;
        let mut top = 927.0;
        let mut bounds = glyphon::TextBounds {
            left: 195,
            top: 927,
            right: 558,
            bottom: 949,
        };

        localize_text_area(&mut left, &mut top, &mut bounds, [24.0, 921.0]);

        assert_eq!([left, top], [171.0, 6.0]);
        assert_eq!(
            [bounds.left, bounds.top, bounds.right, bounds.bottom],
            [171, 6, 534, 28]
        );
    }

    #[test]
    fn centered_multiline_text_uses_prepared_content_height() {
        let document = text::document::Document::plain("one\ntwo\nthree");
        let content_height = document_height(&document);
        let text = centered_text(document, content_height);
        let mut cache = InlineCache::new();

        let prepared = prepare_text(&mut cache, &text, 1.0).expect("text should prepare");

        assert_close(prepared.top, text.rect.origin.y());
    }

    #[test]
    fn centered_single_line_text_keeps_existing_centering() {
        let document = text::document::Document::plain("one");
        let content_height = document_height(&document);
        let rect_height = content_height + 40.0;
        let text = centered_text(document, rect_height);
        let mut cache = InlineCache::new();

        let prepared = prepare_text(&mut cache, &text, 1.0).expect("text should prepare");

        assert_close(prepared.top, text.rect.origin.y() + 20.0);
    }

    #[test]
    fn prepared_text_cache_ignores_rect_origin() {
        let mut cache = InlineCache::new();
        let first = label_text("Command", text::Color::BLACK, 12.0, Weight::Normal, 0.0);
        let second = label_text("Command", text::Color::BLACK, 12.0, Weight::Normal, 40.0);

        let first = prepare_text(&mut cache, &first, 1.0).expect("first label should prepare");
        let second = prepare_text(&mut cache, &second, 1.0).expect("second label should prepare");

        assert_eq!(first.stats.text_cache_misses, 1);
        assert_eq!(first.stats.text_shape_calls, 1);
        assert_eq!(second.stats.text_cache_hits, 1);
        assert_eq!(second.stats.text_shape_calls, 0);
    }

    #[test]
    fn prepared_text_cache_uses_current_color_without_reshaping() {
        let mut cache = InlineCache::new();
        let red = label_text("Command", text::Color::RED, 12.0, Weight::Normal, 0.0);
        let black = label_text("Command", text::Color::BLACK, 12.0, Weight::Normal, 0.0);

        let _ = prepare_text(&mut cache, &red, 1.0).expect("red label should prepare");
        let black = prepare_text(&mut cache, &black, 1.0).expect("black label should prepare");

        assert_eq!(black.stats.text_cache_hits, 1);
        assert_eq!(black.stats.text_shape_calls, 0);
        assert_eq!(
            black.default_color,
            text_layout::glyphon_color(text::Color::BLACK)
        );
    }

    #[test]
    fn prepared_text_cache_misses_when_bounds_change() {
        let mut cache = InlineCache::new();
        let first = label_text("Command", text::Color::BLACK, 12.0, Weight::Normal, 0.0);
        let mut second = first.clone();
        second.rect.area = area::logical(180.0, 22.0);

        let _ = prepare_text(&mut cache, &first, 1.0).expect("first label should prepare");
        let second = prepare_text(&mut cache, &second, 1.0).expect("second label should prepare");

        assert_eq!(second.stats.text_cache_misses, 1);
        assert_eq!(second.stats.text_shape_calls, 1);
    }

    #[test]
    fn multi_color_text_stays_on_uncached_path() {
        let mut cache = InlineCache::new();
        let mut block = Block::new(Align::Start);
        block.push_run(Run::new(
            "Red",
            Style::default()
                .with_color(text::Color::RED)
                .with_size(12.0),
        ));
        block.push_run(Run::new(
            "Black",
            Style::default()
                .with_color(text::Color::BLACK)
                .with_size(12.0),
        ));
        let rich = paint::Text {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(160.0, 22.0)),
            document: text::document::Document::from_block(block),
            wrap: paint::TextWrap::None,
            vertical_align: paint::TextVerticalAlign::Center,
            overflow: crate::text::Overflow::Clip,
        };

        let first = prepare_text(&mut cache, &rich, 1.0).expect("rich text should prepare");
        let second = prepare_text(&mut cache, &rich, 1.0).expect("rich text should prepare again");

        assert_eq!(first.stats.text_cache_hits, 0);
        assert_eq!(second.stats.text_cache_hits, 0);
        assert_eq!(second.stats.text_shape_calls, 1);
    }

    #[test]
    fn prepared_icon_cache_uses_current_color_without_reshaping() {
        let mut cache = InlineCache::new();
        let icon = icon::Icon::phosphor(icon::Id::new("command"));
        let red = paint::Icon {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(18.0, 18.0)),
            icon,
            color: paint::Color::RED,
            size: 12.0,
        };
        let black = paint::Icon {
            color: paint::Color::BLACK,
            ..red
        };

        let _ = prepare_icon(&mut cache, &red, 1.0).expect("red icon should prepare");
        let black = prepare_icon(&mut cache, &black, 1.0).expect("black icon should prepare");

        assert_eq!(black.stats.icon_cache_hits, 1);
        assert_eq!(black.stats.icon_shape_calls, 0);
        assert_eq!(
            black.default_color,
            glyphon_color_from_linear_paint(paint::Color::BLACK)
        );
    }
}
