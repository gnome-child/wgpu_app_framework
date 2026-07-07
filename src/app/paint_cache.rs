use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Range;

use crate::app::scroll;
use crate::geometry::{Rect, area, point};
use crate::{paint, ui, widget};

const COMPOSITOR_SAMPLE_PADDING: f32 = 2.0;
const MAX_SCROLL_LAYERS_PER_TARGET: usize = 7;

#[derive(Debug, Clone)]
pub(crate) struct RetainedPaint {
    scene: paint::Scene,
    scroll_records: HashMap<ui::Path, ui::ScrollPaintRecord>,
    scroll_layers: HashMap<ui::Path, Vec<ScrollLayer>>,
}

#[derive(Debug, Clone, Copy)]
struct ScrollLayer {
    id: paint::LayerId,
    metrics: widget::scroll::Metrics,
    coverage: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LayerHit {
    pub(crate) replaced_items: usize,
    pub(crate) skipped_text_surfaces: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LayerMiss {
    MissingLayer,
}

impl RetainedPaint {
    pub(crate) fn new(scene: paint::Scene, scroll_records: ui::ScrollPaintRecords) -> Self {
        let scroll_layers = eligible_scroll_layers(&scene, &scroll_records);
        Self {
            scene,
            scroll_records,
            scroll_layers,
        }
    }

    pub(crate) fn scene(&self) -> &paint::Scene {
        &self.scene
    }

    pub(crate) fn scroll_record(&self, path: &ui::Path) -> Option<&ui::ScrollPaintRecord> {
        self.scroll_records.get(path)
    }

    pub(crate) fn scroll_range(&self, path: &ui::Path) -> Option<Range<usize>> {
        self.scroll_record(path).map(|record| record.target.clone())
    }

    pub(crate) fn scroll_targets(&self) -> Vec<ui::Path> {
        self.scroll_records.keys().cloned().collect()
    }

    #[cfg(test)]
    pub(crate) fn translate_scroll_content(
        &mut self,
        path: &ui::Path,
        metrics: widget::scroll::Metrics,
    ) -> Option<usize> {
        let record = self.scroll_record(path)?.clone();
        if !scroll_metrics_are_translate_compatible(record.metrics, metrics) {
            return None;
        }

        let old_offset = record.metrics.offset();
        let new_offset = metrics.offset();
        let delta = point::logical(
            old_offset.x() - new_offset.x(),
            old_offset.y() - new_offset.y(),
        );
        let translated = self.scene.translate_items(record.content.clone(), delta);
        if let Some(record) = self.scroll_records.get_mut(path) {
            record.metrics = metrics;
        }

        Some(translated)
    }

    pub(crate) fn replace_scroll_content_with_layer(
        &mut self,
        path: &ui::Path,
        metrics: widget::scroll::Metrics,
        hit: scroll::RetainedLayerHit,
    ) -> Result<LayerHit, LayerMiss> {
        self.replace_scroll_content_with_layer_source(
            path,
            metrics,
            hit.layer_index(),
            hit.source(),
        )
    }

    pub(crate) fn replace_scroll_content_with_current_layer(
        &mut self,
        path: &ui::Path,
        metrics: widget::scroll::Metrics,
    ) -> Result<LayerHit, LayerMiss> {
        let layers = self
            .scroll_layers
            .get(path)
            .ok_or(LayerMiss::MissingLayer)?;
        let (layer_index, source) = layers
            .iter()
            .copied()
            .enumerate()
            .find_map(|(index, layer)| {
                scroll::RetainedLayer::new(layer.metrics, layer.coverage)
                    .source_for_metrics(metrics)
                    .ok()
                    .map(|source| (index, source))
            })
            .ok_or(LayerMiss::MissingLayer)?;

        self.replace_scroll_content_with_layer_source(path, metrics, layer_index, source)
    }

    fn replace_scroll_content_with_layer_source(
        &mut self,
        path: &ui::Path,
        metrics: widget::scroll::Metrics,
        layer_index: usize,
        source: Rect,
    ) -> Result<LayerHit, LayerMiss> {
        let record = self
            .scroll_record(path)
            .cloned()
            .ok_or(LayerMiss::MissingLayer)?;
        let content = self
            .scene
            .items()
            .get(record.content.clone())
            .ok_or(LayerMiss::MissingLayer)?;
        let content_is_existing_layer = !content.is_empty()
            && content
                .iter()
                .all(|item| matches!(item, paint::Item::Layer(_)));
        if !content_is_existing_layer && !scroll_content_is_layer_eligible(&self.scene, &record) {
            return Err(LayerMiss::MissingLayer);
        }
        let layer = self
            .scroll_layers
            .get(path)
            .and_then(|layers| layers.get(layer_index))
            .copied()
            .ok_or(LayerMiss::MissingLayer)?;

        let replaced_items = record.content.end.saturating_sub(record.content.start);
        let skipped_text_surfaces = if content_is_existing_layer {
            0
        } else {
            content.iter().map(text_surface_count).sum()
        };
        let layer_item = paint::Item::Layer(paint::Layer {
            id: layer.id,
            rect: metrics.viewport(),
            source,
            sampling: paint::LayerSampling::PixelAligned,
        });
        let old_len = replaced_items;
        self.scene
            .replace_items(record.content.clone(), [layer_item]);
        let new_content = record.content.start..record.content.start + 1;

        if old_len != 1 {
            self.shift_records_after(record.content.end, 1isize - old_len as isize);
        }

        if let Some(record) = self.scroll_records.get_mut(path) {
            record.content = new_content;
            record.metrics = metrics;
            record.target.end = shift_index(record.target.end, 1isize - old_len as isize);
        }

        Ok(LayerHit {
            replaced_items,
            skipped_text_surfaces,
        })
    }

    pub(crate) fn layer_updates(&self) -> Vec<paint::LayerUpdate> {
        self.scroll_records
            .iter()
            .flat_map(|(path, record)| self.layer_updates_for_record(path, record))
            .collect::<Vec<_>>()
    }

    pub(crate) fn layer_update_for_path(&self, path: &ui::Path) -> Option<paint::LayerUpdate> {
        let record = self.scroll_record(path)?;
        self.layer_updates_for_record(path, record)
            .into_iter()
            .next()
    }

    pub(crate) fn scroll_layer_metrics(&self, path: &ui::Path) -> Option<widget::scroll::Metrics> {
        self.scroll_layers
            .get(path)
            .and_then(|layers| layers.first())
            .map(|layer| layer.metrics)
    }

    pub(crate) fn scroll_layer_eligible(&self, path: &ui::Path) -> bool {
        self.scroll_record(path)
            .is_some_and(|record| scroll_content_is_layer_eligible(&self.scene, record))
    }

    #[cfg(test)]
    pub(crate) fn scroll_layer_coverage(&self, path: &ui::Path) -> Option<Rect> {
        self.scroll_layers
            .get(path)
            .and_then(|layers| layers.first())
            .map(|layer| layer.coverage)
    }

    pub(crate) fn retained_scroll_layers(&self) -> Vec<(ui::Path, Vec<scroll::RetainedLayer>)> {
        self.scroll_layers
            .iter()
            .map(|(path, layers)| {
                (
                    path.clone(),
                    layers
                        .iter()
                        .map(|layer| scroll::RetainedLayer::new(layer.metrics, layer.coverage))
                        .collect(),
                )
            })
            .collect()
    }

    pub(crate) fn remove_scroll_layers(&mut self, path: &ui::Path) {
        self.scroll_layers.remove(path);
    }

    pub(crate) fn replace_scroll_chrome(
        &mut self,
        path: &ui::Path,
        items: Vec<paint::Item>,
    ) -> bool {
        let Some(record) = self.scroll_record(path).cloned() else {
            return false;
        };

        let old_len = record.chrome.end.saturating_sub(record.chrome.start);
        let new_len = items.len();
        self.scene.replace_items(record.chrome.clone(), items);
        let new_chrome = record.chrome.start..record.chrome.start + new_len;

        if old_len != new_len {
            self.shift_records_after(record.chrome.end, new_len as isize - old_len as isize);
        }

        if let Some(record) = self.scroll_records.get_mut(path) {
            record.chrome = new_chrome;
            record.target.end = shift_index(record.target.end, new_len as isize - old_len as isize);
        }

        true
    }

    #[cfg(test)]
    pub(crate) fn replace_scroll_range(
        &mut self,
        path: &ui::Path,
        items: Vec<paint::Item>,
    ) -> bool {
        self.replace_scroll_target(path, items, ui::ScrollPaintRecords::default())
    }

    pub(crate) fn replace_scroll_target(
        &mut self,
        path: &ui::Path,
        items: Vec<paint::Item>,
        mut records: ui::ScrollPaintRecords,
    ) -> bool {
        let Some(record) = self.scroll_record(path).cloned() else {
            return false;
        };
        let range = record.target;

        let old_len = range.end.saturating_sub(range.start);
        let new_len = items.len();
        self.scene.replace_items(range.clone(), items);
        let new_range = range.start..range.start + new_len;

        self.scroll_records.retain(|existing_path, existing| {
            existing_path == path || !range_contains_range(&range, &existing.target)
        });

        if old_len != new_len {
            let delta = new_len as isize - old_len as isize;
            self.shift_records_after(range.end, delta);
        }
        for record in records.values_mut() {
            offset_record(record, range.start);
        }

        if records.is_empty() {
            if let Some(record) = self.scroll_records.get_mut(path) {
                record.target = new_range;
                record.content = range.start..range.start;
                record.chrome = range.start..range.start;
            }
        } else {
            self.scroll_records.extend(records);
        }
        self.scroll_layers = eligible_scroll_layers(&self.scene, &self.scroll_records);

        true
    }

    fn layer_updates_for_record(
        &self,
        path: &ui::Path,
        record: &ui::ScrollPaintRecord,
    ) -> Vec<paint::LayerUpdate> {
        if !scroll_content_is_layer_eligible(&self.scene, record) {
            return Vec::new();
        }
        let Some(content) = self.scene.items().get(record.content.clone()) else {
            return Vec::new();
        };
        self.scroll_layers
            .get(path)
            .into_iter()
            .flat_map(|layers| layers.iter().copied())
            .filter_map(|layer| layer_update_for_content(content, layer))
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn update_scroll_layer_from_recorded_scene(
        &mut self,
        path: &ui::Path,
        metrics: widget::scroll::Metrics,
        coverage: Rect,
        scene: &paint::Scene,
        records: &ui::ScrollPaintRecords,
    ) -> Option<paint::LayerUpdate> {
        let record = records.get(path)?;
        if !scroll_content_is_layer_eligible(scene, record) {
            self.scroll_layers.remove(path);
            return None;
        }
        let mut layer_scene = paint::Scene::new();
        for item in scene.items().get(record.content.clone())?.iter().cloned() {
            if !matches!(item, paint::Item::Layer(_)) && item_intersects_rect(&item, coverage) {
                layer_scene.replace_items(layer_scene.len()..layer_scene.len(), [item]);
            }
        }
        if layer_scene.is_empty() {
            return None;
        }

        let layer = ScrollLayer {
            id: self
                .scroll_layers
                .get(path)
                .and_then(|layers| layers.first())
                .map(|layer| layer.id)
                .unwrap_or_else(|| layer_id_for_path_and_index(path, 0)),
            metrics,
            coverage,
        };
        self.scroll_layers.insert(path.clone(), vec![layer]);
        Some(paint::LayerUpdate {
            id: layer.id,
            coverage,
            scene: layer_scene
                .translated(point::logical(-coverage.origin.x(), -coverage.origin.y())),
        })
    }

    pub(crate) fn update_scroll_layers_from_recorded_scenes<'a, I>(
        &mut self,
        path: &ui::Path,
        metrics: widget::scroll::Metrics,
        tiles: I,
    ) -> Vec<paint::LayerUpdate>
    where
        I: IntoIterator<Item = (Rect, &'a paint::Scene, &'a ui::ScrollPaintRecords)>,
    {
        let mut layers = self.scroll_layers.get(path).cloned().unwrap_or_default();
        let mut updates = Vec::new();

        for (coverage, scene, records) in tiles {
            let Some(record) = records.get(path) else {
                continue;
            };
            if !scroll_content_is_layer_eligible(scene, record) {
                continue;
            }
            let Some(content) = scene.items().get(record.content.clone()) else {
                continue;
            };
            let layer = ScrollLayer {
                id: layers
                    .iter()
                    .find(|layer| same_scroll_layer_identity(**layer, metrics, coverage))
                    .map(|layer| layer.id)
                    .unwrap_or_else(|| layer_id_for_path_and_metrics(path, metrics, coverage)),
                metrics,
                coverage,
            };
            let Some(update) = layer_update_for_content(content, layer) else {
                continue;
            };
            if let Some(existing) = layers.iter_mut().find(|existing| {
                existing.id == layer.id || same_scroll_layer_identity(**existing, metrics, coverage)
            }) {
                *existing = layer;
            } else {
                layers.push(layer);
            }
            updates.push(update);
        }

        prune_scroll_layers(&mut layers, metrics);

        if layers.is_empty() {
            self.scroll_layers.remove(path);
        } else {
            self.scroll_layers.insert(path.clone(), layers);
        }

        updates
    }

    fn shift_records_after(&mut self, boundary: usize, delta: isize) {
        for record in self.scroll_records.values_mut() {
            shift_range_after(&mut record.target, boundary, delta);
            shift_range_after(&mut record.content, boundary, delta);
            shift_range_after(&mut record.chrome, boundary, delta);
        }
    }
}

#[cfg(test)]
fn layer_id_for_path_and_index(path: &ui::Path, index: usize) -> paint::LayerId {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    index.hash(&mut hasher);
    paint::LayerId(hasher.finish())
}

fn layer_id_for_path_and_metrics(
    path: &ui::Path,
    metrics: widget::scroll::Metrics,
    coverage: Rect,
) -> paint::LayerId {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    metrics.offset().x().to_bits().hash(&mut hasher);
    metrics.offset().y().to_bits().hash(&mut hasher);
    coverage.origin.x().to_bits().hash(&mut hasher);
    coverage.origin.y().to_bits().hash(&mut hasher);
    coverage.area.width().to_bits().hash(&mut hasher);
    coverage.area.height().to_bits().hash(&mut hasher);
    paint::LayerId(hasher.finish())
}

fn shift_index(index: usize, delta: isize) -> usize {
    if delta.is_negative() {
        index.saturating_sub(delta.unsigned_abs())
    } else {
        index.saturating_add(delta as usize)
    }
}

fn shift_range_after(range: &mut Range<usize>, boundary: usize, delta: isize) {
    if range.start >= boundary {
        range.start = shift_index(range.start, delta);
        range.end = shift_index(range.end, delta);
    }
}

fn range_contains_range(outer: &Range<usize>, inner: &Range<usize>) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}

fn offset_record(record: &mut ui::ScrollPaintRecord, offset: usize) {
    offset_range(&mut record.target, offset);
    offset_range(&mut record.content, offset);
    offset_range(&mut record.chrome, offset);
}

fn offset_range(range: &mut Range<usize>, offset: usize) {
    range.start = range.start.saturating_add(offset);
    range.end = range.end.saturating_add(offset);
}

fn eligible_scroll_layers(
    scene: &paint::Scene,
    records: &HashMap<ui::Path, ui::ScrollPaintRecord>,
) -> HashMap<ui::Path, Vec<ScrollLayer>> {
    records
        .iter()
        .filter(|(_, record)| scroll_content_is_layer_eligible(scene, record))
        .map(|(path, record)| {
            (
                path.clone(),
                vec![ScrollLayer {
                    metrics: record.metrics,
                    coverage: scroll_record_coverage(scene, record),
                    id: layer_id_for_path_and_metrics(
                        path,
                        record.metrics,
                        scroll_record_coverage(scene, record),
                    ),
                }],
            )
        })
        .collect()
}

fn same_scroll_layer_identity(
    layer: ScrollLayer,
    metrics: widget::scroll::Metrics,
    coverage: Rect,
) -> bool {
    layer.metrics.offset() == metrics.offset() && same_rect(layer.coverage, coverage)
}

fn prune_scroll_layers(layers: &mut Vec<ScrollLayer>, metrics: widget::scroll::Metrics) {
    layers.sort_by(|left, right| {
        layer_distance_squared(*left, metrics).total_cmp(&layer_distance_squared(*right, metrics))
    });
    layers.truncate(MAX_SCROLL_LAYERS_PER_TARGET);
}

fn layer_distance_squared(layer: ScrollLayer, metrics: widget::scroll::Metrics) -> f32 {
    let layer_center = rect_center(layer.coverage);
    let viewport_center = rect_center(metrics.viewport());
    let x = layer_center.x() - viewport_center.x();
    let y = layer_center.y() - viewport_center.y();
    x * x + y * y
}

fn rect_center(rect: Rect) -> point::Logical {
    point::logical(
        rect.origin.x() + rect.area.width() * 0.5,
        rect.origin.y() + rect.area.height() * 0.5,
    )
}

fn scroll_content_is_layer_eligible(scene: &paint::Scene, record: &ui::ScrollPaintRecord) -> bool {
    scene
        .items()
        .get(record.content.clone())
        .is_some_and(|items| {
            !items.is_empty()
                && items.iter().all(|item| {
                    !matches!(
                        item,
                        paint::Item::Filter(_)
                            | paint::Item::Layer(_)
                            | paint::Item::Text(_)
                            | paint::Item::TextSurface(_)
                            | paint::Item::TextViewport(_)
                    )
                })
        })
}

fn layer_update_for_content(
    content: &[paint::Item],
    layer: ScrollLayer,
) -> Option<paint::LayerUpdate> {
    let mut scene = paint::Scene::new();
    for item in content.iter().cloned() {
        if !matches!(item, paint::Item::Layer(_)) && item_intersects_rect(&item, layer.coverage) {
            scene.replace_items(scene.len()..scene.len(), [item]);
        }
    }
    let scene = scene.translated(point::logical(
        -layer.coverage.origin.x(),
        -layer.coverage.origin.y(),
    ));

    (!scene.is_empty()).then_some(paint::LayerUpdate {
        id: layer.id,
        coverage: layer.coverage,
        scene,
    })
}

#[cfg(test)]
fn scroll_metrics_are_translate_compatible(
    old: widget::scroll::Metrics,
    new: widget::scroll::Metrics,
) -> bool {
    same_rect(old.viewport(), new.viewport()) && old.active_axes() == new.active_axes()
}

fn content_coverage(scene: &paint::Scene, record: &ui::ScrollPaintRecord) -> Option<Rect> {
    scene
        .items()
        .get(record.content.clone())?
        .iter()
        .filter_map(item_rect)
        .reduce(union_rect)
}

fn scroll_record_coverage(scene: &paint::Scene, record: &ui::ScrollPaintRecord) -> Rect {
    let coverage = content_coverage(scene, record)
        .map(|coverage| union_rect(coverage, record.metrics.viewport()))
        .unwrap_or_else(|| record.metrics.viewport());

    expand_rect(coverage, COMPOSITOR_SAMPLE_PADDING)
}

fn item_rect(item: &paint::Item) -> Option<Rect> {
    match item {
        paint::Item::Quad(item) => Some(item.rect),
        paint::Item::Text(item) => Some(item.rect),
        paint::Item::TextSurface(item) => Some(item.rect),
        paint::Item::TextViewport(item) => Some(item.rect),
        paint::Item::Icon(item) => Some(item.rect),
        paint::Item::Shadow(item) => Some(item.rect),
        paint::Item::Tint(item) => Some(item.rect),
        paint::Item::Outline(item) => Some(item.rect),
        paint::Item::Filter(item) => Some(item.rect),
        paint::Item::Layer(item) => Some(item.rect),
        paint::Item::Clip(item) => Some(item.rect),
        paint::Item::PopClip => None,
    }
}

fn text_surface_count(item: &paint::Item) -> usize {
    match item {
        paint::Item::TextSurface(_) => 1,
        paint::Item::TextViewport(item) => item.surfaces.len(),
        _ => 0,
    }
}

fn item_intersects_rect(item: &paint::Item, rect: Rect) -> bool {
    item_rect(item).is_none_or(|item| rect_intersects_rect(item, rect))
}

fn rect_intersects_rect(left: Rect, right: Rect) -> bool {
    let left_x = left.origin.x();
    let left_y = left.origin.y();
    let left_right = left_x + left.area.width();
    let left_bottom = left_y + left.area.height();
    let right_x = right.origin.x();
    let right_y = right.origin.y();
    let right_right = right_x + right.area.width();
    let right_bottom = right_y + right.area.height();

    left_x < right_right && left_right > right_x && left_y < right_bottom && left_bottom > right_y
}

fn union_rect(left: Rect, right: Rect) -> Rect {
    let left_x = left.origin.x().min(right.origin.x());
    let top_y = left.origin.y().min(right.origin.y());
    let right_x = (left.origin.x() + left.area.width()).max(right.origin.x() + right.area.width());
    let bottom_y =
        (left.origin.y() + left.area.height()).max(right.origin.y() + right.area.height());

    Rect::new(
        point::logical(left_x, top_y),
        area::logical((right_x - left_x).max(0.0), (bottom_y - top_y).max(0.0)),
    )
}

fn expand_rect(rect: Rect, amount: f32) -> Rect {
    Rect::rounded(
        point::logical(rect.origin.x() - amount, rect.origin.y() - amount),
        area::logical(
            rect.area.width() + amount * 2.0,
            rect.area.height() + amount * 2.0,
        ),
        rect.rounding,
    )
}

fn same_rect(left: Rect, right: Rect) -> bool {
    left.origin == right.origin && same_area(left.area, right.area)
}

#[cfg(test)]
fn source_geometry_is_1_to_1(source: Rect, destination: Rect) -> bool {
    same_area(source.area, destination.area)
}

fn same_area(left: area::Logical, right: area::Logical) -> bool {
    left.width().to_bits() == right.width().to_bits()
        && left.height().to_bits() == right.height().to_bits()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::app::scroll;
    use crate::geometry::{Rect, area, point};
    use crate::{paint, text, ui, widget};

    use super::*;

    const ROOT: ui::Id = ui::Id::new("root");
    const FIRST: ui::Id = ui::Id::new("first");
    const SECOND: ui::Id = ui::Id::new("second");

    fn quad(x: f32) -> paint::Item {
        quad_rect(x, 0.0, 1.0, 1.0)
    }

    fn quad_rect(x: f32, y: f32, width: f32, height: f32) -> paint::Item {
        paint::Item::Quad(paint::Quad {
            rect: Rect::new(point::logical(x, y), area::logical(width, height)),
            rasterization: paint::Rasterization::default(),
            transform: paint::Transform::identity(),
            style: paint::Style {
                fill: Some(paint::Fill::Brush(paint::Brush::solid(paint::Color::BLACK))),
                stroke: None,
                tint: None,
            },
        })
    }

    fn label(text: &str, x: f32, y: f32, width: f32, height: f32) -> paint::Item {
        paint::Item::Text(paint::Text {
            rect: Rect::new(point::logical(x, y), area::logical(width, height)),
            document: text::document::Document::plain(text),
            wrap: paint::TextWrap::None,
            vertical_align: paint::TextVerticalAlign::Start,
        })
    }

    fn text_viewport(x: f32, y: f32, width: f32, height: f32) -> paint::Item {
        paint::Item::TextViewport(paint::TextViewport {
            rect: Rect::new(point::logical(x, y), area::logical(width, height)),
            surfaces: Vec::new(),
        })
    }

    #[test]
    fn replace_scroll_range_updates_target_and_following_ranges() {
        let first = ui::Path::new([ROOT, FIRST]);
        let second = ui::Path::new([ROOT, SECOND]);
        let mut scene = paint::Scene::new();
        for index in 0..5 {
            scene.replace_items(index..index, [quad(index as f32)]);
        }
        let mut records = HashMap::new();
        records.insert(first.clone(), record(1..2));
        records.insert(second.clone(), record(3..4));
        let mut retained = RetainedPaint::new(scene, records);

        assert!(retained.replace_scroll_range(&first, vec![quad(10.0), quad(11.0)]));

        assert_eq!(retained.scroll_range(&first), Some(1..3));
        assert_eq!(retained.scroll_range(&second), Some(4..5));
        assert_eq!(retained.scene().items().len(), 6);
    }

    fn record(target: Range<usize>) -> ui::ScrollPaintRecord {
        record_with_ranges(target.clone(), target, 0..0)
    }

    fn record_with_ranges(
        target: Range<usize>,
        content: Range<usize>,
        chrome: Range<usize>,
    ) -> ui::ScrollPaintRecord {
        let metrics = widget::scroll::Metrics::resolve(
            Rect::new(point::logical(0.0, 0.0), area::logical(100.0, 100.0)),
            Rect::new(point::logical(0.0, 0.0), area::logical(100.0, 100.0)),
            area::logical(100.0, 200.0),
            point::logical(0.0, 0.0),
            widget::scroll::Axes::vertical(),
            widget::scroll::Bars::vertical(),
            widget::scroll::Style::default(),
        );
        ui::ScrollPaintRecord {
            target,
            content,
            chrome,
            metrics,
        }
    }

    #[test]
    fn translate_scroll_content_moves_content_but_not_clip_or_chrome() {
        let path = ui::Path::new([ROOT, FIRST]);
        let mut scene = paint::Scene::new();
        scene.push_clip(paint::Clip {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(100.0, 100.0)),
        });
        scene.push_quad(match quad(10.0) {
            paint::Item::Quad(quad) => quad,
            _ => unreachable!(),
        });
        scene.push_quad(match quad(20.0) {
            paint::Item::Quad(quad) => quad,
            _ => unreachable!(),
        });

        let record = record_with_ranges(0..3, 1..2, 2..3);
        let metrics = record.metrics.with_offset(point::logical(0.0, 24.0));
        let mut retained = RetainedPaint::new(scene, HashMap::from([(path.clone(), record)]));

        assert_eq!(retained.translate_scroll_content(&path, metrics), Some(1));

        let items = retained.scene().items();
        assert!(matches!(
            items[0],
            paint::Item::Clip(paint::Clip {
                rect: Rect { origin, .. },
            }) if origin == point::logical(0.0, 0.0)
        ));
        assert!(matches!(
            items[1],
            paint::Item::Quad(paint::Quad {
                rect: Rect { origin, .. },
                ..
            }) if origin == point::logical(10.0, -24.0)
        ));
        assert!(matches!(
            items[2],
            paint::Item::Quad(paint::Quad {
                rect: Rect { origin, .. },
                ..
            }) if origin == point::logical(20.0, 0.0)
        ));
    }

    #[test]
    fn replace_scroll_content_with_layer_leaves_clip_and_chrome_fixed() {
        let path = ui::Path::new([ROOT, FIRST]);
        let mut scene = paint::Scene::new();
        scene.push_clip(paint::Clip {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(100.0, 100.0)),
        });
        scene.push_quad(match quad_rect(0.0, 24.0, 100.0, 100.0) {
            paint::Item::Quad(quad) => quad,
            _ => unreachable!(),
        });
        scene.push_quad(match quad(20.0) {
            paint::Item::Quad(quad) => quad,
            _ => unreachable!(),
        });

        let record = record_with_ranges(0..3, 1..2, 2..3);
        let base_metrics = record.metrics;
        let metrics = record.metrics.with_offset(point::logical(0.0, 24.0));
        let mut retained = RetainedPaint::new(scene, HashMap::from([(path.clone(), record)]));
        let mut scroll_driver = scroll::Driver::default();
        scroll_driver.record_retained_layer(
            path.clone(),
            base_metrics,
            retained
                .scroll_layer_coverage(&path)
                .expect("retained layer should have coverage"),
        );
        let hit_plan = scroll_driver
            .retained_layer_hit(&path, metrics)
            .expect("compatible metrics should use retained scroll layer");

        let hit = retained
            .replace_scroll_content_with_layer(&path, metrics, hit_plan)
            .expect("compatible metrics should use layer");

        assert_eq!(hit.replaced_items, 1);
        let items = retained.scene().items();
        assert!(matches!(items[0], paint::Item::Clip(_)));
        assert!(matches!(
            items[1],
            paint::Item::Layer(paint::Layer {
                rect: Rect { origin, .. },
                source,
                sampling,
                ..
            }) if origin == point::logical(0.0, 0.0)
                && source == Rect::new(point::logical(2.0, 26.0), area::logical(90.0, 100.0))
                && sampling == paint::LayerSampling::PixelAligned
        ));
        assert!(matches!(
            items[2],
            paint::Item::Quad(paint::Quad {
                rect: Rect { origin, .. },
                ..
            }) if origin == point::logical(20.0, 0.0)
        ));
    }

    #[test]
    fn replace_scroll_content_with_layer_updates_existing_layer_item() {
        let path = ui::Path::new([ROOT, FIRST]);
        let mut scene = paint::Scene::new();
        scene.push_clip(paint::Clip {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(100.0, 100.0)),
        });
        scene.push_quad(match quad_rect(0.0, 0.0, 100.0, 200.0) {
            paint::Item::Quad(quad) => quad,
            _ => unreachable!(),
        });
        scene.push_quad(match quad(20.0) {
            paint::Item::Quad(quad) => quad,
            _ => unreachable!(),
        });

        let record = record_with_ranges(0..3, 1..2, 2..3);
        let base_metrics = record.metrics;
        let mut retained = RetainedPaint::new(scene, HashMap::from([(path.clone(), record)]));
        let mut scroll_driver = scroll::Driver::default();
        scroll_driver.record_retained_layer(
            path.clone(),
            base_metrics,
            retained
                .scroll_layer_coverage(&path)
                .expect("retained layer should have coverage"),
        );

        let first_metrics = base_metrics.with_offset(point::logical(0.0, 24.0));
        let first_hit = scroll_driver
            .retained_layer_hit(&path, first_metrics)
            .expect("first scroll should hit retained layer");
        retained
            .replace_scroll_content_with_layer(&path, first_metrics, first_hit)
            .expect("first hit should replace content");

        let second_metrics = base_metrics.with_offset(point::logical(0.0, 48.0));
        let second_hit = scroll_driver
            .retained_layer_hit(&path, second_metrics)
            .expect("second scroll should hit retained layer");
        retained
            .replace_scroll_content_with_layer(&path, second_metrics, second_hit)
            .expect("existing layer item should be updated");

        let items = retained.scene().items();
        assert!(matches!(
            items[1],
            paint::Item::Layer(paint::Layer {
                rect,
                source,
                ..
            }) if rect == second_metrics.viewport()
                && source == Rect::new(point::logical(2.0, 50.0), area::logical(90.0, 100.0))
        ));
        assert_eq!(retained.scroll_range(&path), Some(0..3));
    }

    #[test]
    fn replace_scroll_content_with_layer_reports_coverage_miss() {
        let path = ui::Path::new([ROOT, FIRST]);
        let mut scene = paint::Scene::new();
        scene.push_clip(paint::Clip {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(100.0, 100.0)),
        });
        scene.push_quad(match quad_rect(0.0, 0.0, 100.0, 20.0) {
            paint::Item::Quad(quad) => quad,
            _ => unreachable!(),
        });

        let record = record_with_ranges(0..2, 1..2, 2..2);
        let base_metrics = record.metrics;
        let metrics = record.metrics.with_offset(point::logical(0.0, 24.0));
        let retained = RetainedPaint::new(scene, HashMap::from([(path.clone(), record)]));
        let mut scroll_driver = scroll::Driver::default();
        scroll_driver.record_retained_layer(
            path.clone(),
            base_metrics,
            retained
                .scroll_layer_coverage(&path)
                .expect("retained layer should have coverage"),
        );

        assert_eq!(
            scroll_driver.retained_layer_hit(&path, metrics),
            Err(scroll::RetainedLayerMiss::CoverageMiss)
        );
    }

    #[test]
    fn layer_source_geometry_requires_one_to_one_sampling() {
        assert!(source_geometry_is_1_to_1(
            Rect::new(point::logical(10.0, 20.0), area::logical(90.0, 100.0)),
            Rect::new(point::logical(0.0, 0.0), area::logical(90.0, 100.0))
        ));
        assert!(!source_geometry_is_1_to_1(
            Rect::new(point::logical(10.0, 20.0), area::logical(120.0, 100.0)),
            Rect::new(point::logical(0.0, 0.0), area::logical(90.0, 100.0))
        ));
    }

    #[test]
    fn initial_scroll_layer_coverage_includes_viewport_when_content_is_narrower() {
        let path = ui::Path::new([ROOT, FIRST]);
        let mut scene = paint::Scene::new();
        scene.push_clip(paint::Clip {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(100.0, 100.0)),
        });
        scene.push_quad(match quad_rect(0.0, 0.0, 40.0, 20.0) {
            paint::Item::Quad(quad) => quad,
            _ => unreachable!(),
        });

        let record = record_with_ranges(0..2, 1..2, 2..2);
        let retained = RetainedPaint::new(scene, HashMap::from([(path.clone(), record)]));

        assert_eq!(
            retained.scroll_layer_coverage(&path),
            Some(Rect::new(
                point::logical(-2.0, -2.0),
                area::logical(94.0, 104.0)
            ))
        );
        assert_eq!(
            retained
                .layer_update_for_path(&path)
                .expect("initial layer update")
                .coverage,
            Rect::new(point::logical(-2.0, -2.0), area::logical(94.0, 104.0))
        );
    }

    #[test]
    fn retained_paint_keeps_non_scroll_text_as_text() {
        let mut scene = paint::Scene::new();
        scene.replace_items(0..0, [label("debug", 12.0, 20.0, 80.0, 24.0)]);

        let retained = RetainedPaint::new(scene, HashMap::new());
        let updates = retained.layer_updates();

        assert!(matches!(
            retained.scene().items()[0],
            paint::Item::Text(paint::Text {
                rect: Rect { origin, .. },
                ..
            }) if origin == point::logical(12.0, 20.0)
        ));
        assert!(updates.is_empty());
    }

    #[test]
    fn retained_paint_keeps_scroll_text_content_out_of_generic_layers() {
        let path = ui::Path::new([ROOT, FIRST]);
        let mut scene = paint::Scene::new();
        scene.replace_items(0..0, [label("line", 0.0, 0.0, 80.0, 24.0)]);
        let record = record_with_ranges(0..1, 0..1, 1..1);

        let retained = RetainedPaint::new(scene, HashMap::from([(path.clone(), record)]));

        assert!(matches!(retained.scene().items()[0], paint::Item::Text(_)));
        assert!(!retained.scroll_layer_eligible(&path));
        assert!(retained.scroll_layer_coverage(&path).is_none());
        assert!(retained.retained_scroll_layers().is_empty());
        assert!(retained.layer_update_for_path(&path).is_none());
    }

    #[test]
    fn retained_paint_keeps_scroll_text_viewport_out_of_generic_layers() {
        let path = ui::Path::new([ROOT, FIRST]);
        let mut scene = paint::Scene::new();
        scene.replace_items(0..0, [text_viewport(0.0, 0.0, 80.0, 100.0)]);
        let record = record_with_ranges(0..1, 0..1, 1..1);

        let retained = RetainedPaint::new(scene, HashMap::from([(path.clone(), record)]));

        assert!(matches!(
            retained.scene().items()[0],
            paint::Item::TextViewport(_)
        ));
        assert!(!retained.scroll_layer_eligible(&path));
        assert!(retained.scroll_layer_coverage(&path).is_none());
        assert!(retained.retained_scroll_layers().is_empty());
        assert!(retained.layer_update_for_path(&path).is_none());
    }

    #[test]
    fn retained_paint_marks_geometry_scroll_content_layer_eligible() {
        let path = ui::Path::new([ROOT, FIRST]);
        let mut scene = paint::Scene::new();
        scene.replace_items(0..0, [quad_rect(0.0, 0.0, 80.0, 24.0)]);
        let record = record_with_ranges(0..1, 0..1, 1..1);

        let retained = RetainedPaint::new(scene, HashMap::from([(path.clone(), record)]));

        assert!(retained.scroll_layer_eligible(&path));
        assert!(retained.scroll_layer_coverage(&path).is_some());
        assert_eq!(retained.retained_scroll_layers().len(), 1);
        assert!(retained.layer_update_for_path(&path).is_some());
    }

    #[test]
    fn retained_layer_update_is_bounded_to_requested_coverage() {
        let path = ui::Path::new([ROOT, FIRST]);
        let mut scene = paint::Scene::new();
        scene.replace_items(
            0..0,
            [
                quad_rect(0.0, 0.0, 80.0, 24.0),
                quad_rect(0.0, 240.0, 80.0, 24.0),
            ],
        );
        let record = record_with_ranges(0..2, 0..2, 2..2);
        let metrics = record.metrics;
        let records = HashMap::from([(path.clone(), record)]);
        let coverage = Rect::new(point::logical(0.0, 0.0), area::logical(100.0, 100.0));
        let mut retained = RetainedPaint::new(paint::Scene::new(), HashMap::new());

        let update = retained
            .update_scroll_layer_from_recorded_scene(&path, metrics, coverage, &scene, &records)
            .expect("covered item should create layer update");

        assert_eq!(update.coverage, coverage);
        assert_eq!(update.scene.items().len(), 1);
        assert!(matches!(
            &update.scene.items()[0],
            paint::Item::Quad(quad)
                if quad.rect == Rect::new(point::logical(0.0, 0.0), area::logical(80.0, 24.0))
        ));
    }

    #[test]
    fn retained_layer_updates_preserve_same_coverage_at_different_scroll_offsets() {
        let path = ui::Path::new([ROOT, FIRST]);
        let coverage = Rect::new(point::logical(-2.0, -2.0), area::logical(104.0, 104.0));
        let mut scene = paint::Scene::new();
        scene.replace_items(0..0, [quad_rect(0.0, 0.0, 80.0, 24.0)]);
        let record = record_with_ranges(0..1, 0..1, 1..1);
        let base_metrics = record.metrics;
        let scrolled_metrics = base_metrics.with_offset(point::logical(0.0, 80.0));
        let records = HashMap::from([(path.clone(), record)]);
        let mut retained = RetainedPaint::new(paint::Scene::new(), HashMap::new());

        assert_eq!(
            retained
                .update_scroll_layers_from_recorded_scenes(
                    &path,
                    base_metrics,
                    [(coverage, &scene, &records)],
                )
                .len(),
            1
        );
        assert_eq!(
            retained
                .update_scroll_layers_from_recorded_scenes(
                    &path,
                    scrolled_metrics,
                    [(coverage, &scene, &records)],
                )
                .len(),
            1
        );

        let layers = retained
            .scroll_layers
            .get(&path)
            .expect("retained layers should exist");
        assert_eq!(layers.len(), 2);
        assert_ne!(layers[0].id, layers[1].id);

        let mut scroll_driver = scroll::Driver::default();
        scroll_driver.set_retained_layers(retained.retained_scroll_layers());

        assert!(
            scroll_driver
                .retained_layer_hit(&path, base_metrics)
                .is_ok()
        );
        assert!(
            scroll_driver
                .retained_layer_hit(&path, scrolled_metrics)
                .is_ok()
        );
    }

    #[test]
    fn retained_paint_does_not_nest_scroll_layers() {
        let path = ui::Path::new([ROOT, FIRST]);
        let mut scene = paint::Scene::new();
        scene.push_layer(paint::Layer {
            id: paint::LayerId(42),
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(80.0, 24.0)),
            source: Rect::new(point::logical(0.0, 0.0), area::logical(80.0, 24.0)),
            sampling: paint::LayerSampling::PixelAligned,
        });
        let record = record_with_ranges(0..1, 0..1, 1..1);

        let retained = RetainedPaint::new(scene, HashMap::from([(path.clone(), record)]));

        assert!(retained.retained_scroll_layers().is_empty());
        assert!(retained.layer_update_for_path(&path).is_none());
    }
}
