use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::geometry::{Rect, area, point};
use crate::{text, ui, widget};

#[derive(Debug, Default, Clone)]
pub struct State {
    projections: HashMap<ui::Path, Projection>,
    text_area_models: HashMap<ui::Path, TextAreaModel>,
    diagnostics: Diagnostics,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Diagnostics {
    pub generic_scroll_projections: usize,
    pub text_area_surfaces: usize,
    pub text_area_targets: usize,
    pub text_area_skipped_by_filter: usize,
    pub text_area_resolves: usize,
    pub text_area_model_reuses: usize,
    pub text_area_model_updates: usize,
    pub text_area_idle_refinements: usize,
    pub projection_count: usize,
}

#[derive(Debug, Clone)]
pub struct Projection {
    metrics: widget::scroll::Metrics,
    text_area: Option<TextAreaProjection>,
}

#[derive(Debug, Clone)]
pub struct TextAreaProjection {
    metrics: widget::scroll::Metrics,
    layout: text::layout::TextFieldLayout,
    surfaces: Vec<text::layout::TextAreaSurface>,
}

#[derive(Debug, Clone, Copy)]
struct TextAreaModel {
    key: text::layout::AreaScrollKey,
    content_size: area::Logical,
}

impl State {
    #[cfg(test)]
    pub fn resolve(
        composition: &ui::Composition,
        text_field_states: &HashMap<ui::Path, text::view::TextViewState>,
        text_engine: &mut text::layout::Engine,
        now: Instant,
    ) -> Self {
        let mut state = Self::default();
        state.sync(composition, text_field_states, text_engine, now);
        state
    }

    #[cfg(test)]
    pub fn sync(
        &mut self,
        composition: &ui::Composition,
        text_field_states: &HashMap<ui::Path, text::view::TextViewState>,
        text_engine: &mut text::layout::Engine,
        now: Instant,
    ) {
        self.sync_filtered(composition, text_field_states, text_engine, now, None);
    }

    pub fn sync_filtered(
        &mut self,
        composition: &ui::Composition,
        text_field_states: &HashMap<ui::Path, text::view::TextViewState>,
        text_engine: &mut text::layout::Engine,
        now: Instant,
        text_area_targets: Option<&HashSet<ui::Path>>,
    ) {
        let mut projections = HashMap::new();
        let mut live_text_areas = HashSet::new();
        let mut diagnostics = Diagnostics::default();

        for (path, metrics) in composition.widget_metrics_iter() {
            if let Some(metrics) = (*metrics).scroll() {
                diagnostics.generic_scroll_projections += 1;
                projections.insert(
                    path.clone(),
                    Projection {
                        metrics,
                        text_area: None,
                    },
                );
            }
        }

        for (path, surface) in composition.text_surfaces() {
            if !surface.is_area() {
                continue;
            }
            diagnostics.text_area_surfaces += 1;
            live_text_areas.insert(path.clone());
            if text_area_targets.is_some_and(|targets| !targets.contains(path)) {
                diagnostics.text_area_skipped_by_filter += 1;
                continue;
            }
            diagnostics.text_area_targets += 1;

            let text_state = text_field_states.get(path).cloned().unwrap_or_default();
            let hint = self.text_area_models.get(path).map(TextAreaModel::hint);
            if hint.is_some() {
                diagnostics.text_area_model_reuses += 1;
            }
            let Some((metrics, paint_layout, key, content_size)) = composition
                .text_area_scroll_paint_layout_with_content_hint(
                    path,
                    text_state,
                    text_engine,
                    now,
                    hint,
                )
            else {
                continue;
            };
            diagnostics.text_area_resolves += 1;
            let (layout, surfaces) = paint_layout.into_parts();

            self.text_area_models
                .insert(path.clone(), TextAreaModel { key, content_size });
            diagnostics.text_area_model_updates += 1;
            projections.insert(
                path.clone(),
                Projection {
                    metrics,
                    text_area: Some(TextAreaProjection {
                        metrics,
                        layout,
                        surfaces,
                    }),
                },
            );
        }

        self.text_area_models
            .retain(|path, _| live_text_areas.contains(path));
        diagnostics.projection_count = projections.len();
        self.projections = projections;
        self.diagnostics = diagnostics;
    }

    pub fn refine_idle_text_area_models(
        &mut self,
        composition: &ui::Composition,
        text_field_states: &HashMap<ui::Path, text::view::TextViewState>,
        text_engine: &mut text::layout::Engine,
        now: Instant,
        max_refinements: usize,
    ) -> bool {
        if max_refinements == 0 {
            return false;
        }

        let mut refined = 0;
        let paths = self
            .projections
            .iter()
            .filter_map(|(path, projection)| projection.text_area().is_some().then_some(path))
            .cloned()
            .collect::<Vec<_>>();

        for path in paths {
            if refined >= max_refinements {
                break;
            }

            let Some(model) = self.text_area_models.get(&path).copied() else {
                continue;
            };
            let text_state = text_field_states.get(&path).cloned().unwrap_or_default();
            let Some((metrics, key, content_size)) = composition
                .text_area_scroll_metrics_with_content_hint(
                    &path,
                    text_state,
                    text_engine,
                    now,
                    Some(model.hint()),
                )
            else {
                continue;
            };

            let changed = model.key != key
                || model.content_size.width().to_bits() != content_size.width().to_bits()
                || model.content_size.height().to_bits() != content_size.height().to_bits();
            if changed {
                self.text_area_models
                    .insert(path.clone(), TextAreaModel { key, content_size });
                if let Some(projection) = self.projections.get_mut(&path) {
                    projection.set_metrics(metrics);
                }
            }
            refined += 1;
        }

        self.diagnostics.text_area_idle_refinements += refined;
        refined > 0
    }

    pub fn clear(&mut self) {
        self.projections.clear();
        self.text_area_models.clear();
        self.diagnostics = Diagnostics::default();
    }

    pub fn is_empty(&self) -> bool {
        self.projections.is_empty()
    }

    pub fn update_offset(&mut self, path: &ui::Path, offset: point::Logical) {
        if let Some(projection) = self.projections.get_mut(path) {
            projection.set_metrics(projection.metrics().with_offset(offset));
        }
    }

    pub fn metrics(&self, path: &ui::Path) -> Option<widget::scroll::Metrics> {
        self.projections.get(path).map(Projection::metrics)
    }

    pub fn text_area(&self, path: &ui::Path) -> Option<&TextAreaProjection> {
        self.projections.get(path).and_then(Projection::text_area)
    }

    pub fn diagnostics(&self) -> Diagnostics {
        self.diagnostics
    }

    pub fn widget_hit(&self, position: point::Logical) -> Option<widget::Hit> {
        self.projections
            .iter()
            .filter_map(|(path, projection)| {
                projection
                    .metrics()
                    .hit_test(position)
                    .map(|part| widget::Hit::new(path.clone(), widget::Part::Scroll(part)))
            })
            .max_by_key(|hit| hit.target().ids().len())
    }

    pub fn scroll_target_in_frame(
        &self,
        frame: &ui::Frame,
        position: point::Logical,
    ) -> Option<ui::Path> {
        if !rect_contains(frame.rect(), position) {
            return None;
        }

        for child in frame.children().iter().rev() {
            if let Some(target) = self.scroll_target_in_frame(child, position) {
                return Some(target);
            }
        }

        self.metrics(frame.path())
            .is_some_and(|metrics| metrics.max_offset().x() > 0.0 || metrics.max_offset().y() > 0.0)
            .then(|| frame.path().clone())
    }
}

impl Projection {
    pub fn metrics(&self) -> widget::scroll::Metrics {
        self.metrics
    }

    pub fn text_area(&self) -> Option<&TextAreaProjection> {
        self.text_area.as_ref()
    }

    fn set_metrics(&mut self, metrics: widget::scroll::Metrics) {
        let offset_changed = self.metrics.offset() != metrics.offset();
        self.metrics = metrics;
        if offset_changed {
            self.text_area = None;
        } else if let Some(text_area) = self.text_area.as_mut() {
            text_area.metrics = metrics;
        }
    }
}

impl TextAreaProjection {
    pub fn new(
        metrics: widget::scroll::Metrics,
        layout: text::layout::TextFieldLayout,
        surfaces: Vec<text::layout::TextAreaSurface>,
    ) -> Self {
        Self {
            metrics,
            layout,
            surfaces,
        }
    }

    pub fn metrics(&self) -> widget::scroll::Metrics {
        self.metrics
    }

    pub fn layout(&self) -> &text::layout::TextFieldLayout {
        &self.layout
    }

    pub fn surfaces(&self) -> &[text::layout::TextAreaSurface] {
        &self.surfaces
    }

    pub fn observed_area(&self) -> text::view::ObservedArea<'_> {
        text::view::ObservedArea::new(
            self.metrics.viewport(),
            self.metrics.offset(),
            self.layout.content_area(),
            &self.surfaces,
        )
    }

    #[cfg(test)]
    pub fn buffer(&self) -> std::rc::Rc<std::cell::RefCell<glyphon::Buffer>> {
        self.surfaces
            .first()
            .expect("text area projection should have at least one surface")
            .buffer()
    }
}

impl TextAreaModel {
    fn hint(&self) -> (text::layout::AreaScrollKey, area::Logical) {
        (self.key, self.content_size)
    }
}

fn rect_contains(rect: Rect, position: point::Logical) -> bool {
    let x = position.x();
    let y = position.y();
    let left = rect.origin.x();
    let top = rect.origin.y();
    let right = left + rect.area.width();
    let bottom = top + rect.area.height();

    x >= left && x < right && y >= top && y < bottom
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::rc::Rc;
    use std::time::Instant;

    use crate::geometry::{area, point};
    use crate::{action, layout, text, ui, widget, window};

    use super::State;

    const ROOT: ui::Id = ui::Id::new("root");
    const AREA: ui::Id = ui::Id::new("area");

    fn long_text_area_buffer() -> text::Buffer {
        let text = (0..40)
            .map(|line| {
                format!(
                    "line {line} with enough text to overflow horizontally across a narrow area"
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        text::Buffer::from_multiline_text(text)
    }

    fn text_area_composition_for(
        area_model: text::Area,
    ) -> (ui::Composition, text::layout::Engine, ui::Path) {
        let root = ui::Node::container(ROOT, layout::Axis::Vertical)
            .with_size(layout::Size::Fill, layout::Size::Fill)
            .with_child(
                widget::text_area(AREA, area_model)
                    .with_size(layout::Size::Fixed(120.0), layout::Size::Fixed(56.0)),
            );
        let mut tree = ui::Tree::new();
        let mut actions = action::Registry::<()>::new();
        let mut text_engine = text::layout::Engine::new();

        tree.set_root(root);
        let composition = tree
            .compose(
                window::Id::new(1),
                area::logical(160.0, 80.0),
                &mut actions,
                &[],
                &mut text_engine,
            )
            .expect("tree should compose");
        let path = ui::Path::from(ROOT).child(AREA);

        (composition, text_engine, path)
    }

    fn text_area_composition() -> (ui::Composition, text::layout::Engine, ui::Path) {
        text_area_composition_for(text::Area::new(long_text_area_buffer()))
    }

    #[test]
    fn text_area_projection_carries_metrics_and_prepared_surface() {
        let (composition, mut text_engine, path) = text_area_composition();
        let projections = State::resolve(
            &composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
        );
        let projection = projections
            .text_area(&path)
            .expect("text area should have a scroll projection");
        let metrics = projection.metrics();

        assert!(metrics.max_offset().y() > 0.0);
        assert!(metrics.vertical_thumb().is_some());
        assert!(metrics.content_size().height() > metrics.viewport().area.height());
    }

    #[test]
    fn wrapped_text_area_vertical_overflow_does_not_activate_horizontal_scrollbar() {
        let (composition, mut text_engine, path) = text_area_composition();
        let mut projections = State::resolve(
            &composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
        );
        let metrics = projections.metrics(&path).expect("metrics should exist");

        assert!(metrics.max_offset().y() > 0.0);
        assert_eq!(metrics.max_offset().x(), 0.0);
        assert!(metrics.vertical_thumb().is_some());
        assert!(metrics.horizontal_thumb().is_none());

        assert!(projections.refine_idle_text_area_models(
            &composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
            1,
        ));
        let refined = projections.metrics(&path).expect("metrics should remain");

        assert_eq!(refined.max_offset().x(), 0.0);
        assert!(refined.horizontal_thumb().is_none());
    }

    #[test]
    fn wrapped_text_area_pending_caret_visibility_clamps_stale_horizontal_scroll() {
        let (composition, mut text_engine, path) = text_area_composition();
        let now = Instant::now();
        let state = text::view::TextViewState::default()
            .with_scroll(80.0, 0.0)
            .ensure_caret_visible(now);
        let projections = State::resolve(
            &composition,
            &HashMap::from([(path.clone(), state)]),
            &mut text_engine,
            now,
        );
        let metrics = projections.metrics(&path).expect("metrics should exist");

        assert_eq!(metrics.offset().x(), 0.0);
        assert_eq!(metrics.max_offset().x(), 0.0);
        assert!(metrics.horizontal_thumb().is_none());
    }

    #[test]
    fn no_wrap_text_area_long_line_activates_horizontal_scrollbar() {
        let area_model = text::Area::new(long_text_area_buffer()).no_wrap();
        let (composition, mut text_engine, path) = text_area_composition_for(area_model);
        let projections = State::resolve(
            &composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
        );
        let metrics = projections.metrics(&path).expect("metrics should exist");

        assert!(metrics.max_offset().x() > 0.0);
        assert!(metrics.horizontal_thumb().is_some());
    }

    #[test]
    fn repeated_projection_reads_reuse_prepared_surface_handle() {
        let (composition, mut text_engine, path) = text_area_composition();
        let projections = State::resolve(
            &composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
        );
        let first = projections
            .text_area(&path)
            .expect("text area should have a scroll projection")
            .buffer();
        let second = projections
            .text_area(&path)
            .expect("text area should still have a scroll projection")
            .buffer();

        assert!(Rc::ptr_eq(&first, &second));
    }

    #[test]
    fn scrollbar_hit_regions_come_from_cached_projection() {
        let (composition, mut text_engine, path) = text_area_composition();
        let projections = State::resolve(
            &composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
        );
        let metrics = projections.metrics(&path).expect("metrics should exist");
        let thumb = metrics
            .vertical_thumb()
            .expect("overflowing area should have vertical thumb");
        let hit = projections
            .widget_hit(point::logical(
                thumb.origin.x() + 1.0,
                thumb.origin.y() + 1.0,
            ))
            .expect("thumb should hit");

        assert_eq!(hit.target(), &path);
        assert_eq!(
            hit.part(),
            widget::Part::Scroll(widget::scroll::Part::VerticalThumb)
        );
    }

    #[test]
    fn update_offset_drops_stale_text_area_projection_but_keeps_metrics() {
        let (composition, mut text_engine, path) = text_area_composition();
        let mut projections = State::resolve(
            &composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
        );
        assert!(projections.text_area(&path).is_some());
        let offset = point::logical(0.0, 120.0);

        projections.update_offset(&path, offset);

        assert_eq!(
            projections
                .metrics(&path)
                .expect("metrics should remain")
                .offset(),
            offset
        );
        assert!(projections.text_area(&path).is_none());

        projections.sync(
            &composition,
            &HashMap::from([(
                path.clone(),
                text::view::TextViewState::default().with_scroll_y(120.0),
            )]),
            &mut text_engine,
            Instant::now(),
        );
        let rebuilt = projections
            .text_area(&path)
            .expect("sync should rebuild text area projection");
        assert_eq!(rebuilt.metrics().offset(), offset);
    }
    #[test]
    fn text_area_thumb_size_is_stable_after_scroll_sync() {
        let (composition, mut text_engine, path) = text_area_composition();
        let mut projections = State::default();
        let mut states = HashMap::new();

        projections.sync(&composition, &states, &mut text_engine, Instant::now());
        let before = projections
            .metrics(&path)
            .and_then(widget::scroll::Metrics::vertical_thumb)
            .expect("initial thumb");
        states.insert(
            path.clone(),
            text::view::TextViewState::default().with_scroll_y(200.0),
        );
        projections.sync(&composition, &states, &mut text_engine, Instant::now());
        let after = projections
            .metrics(&path)
            .and_then(widget::scroll::Metrics::vertical_thumb)
            .expect("scrolled thumb");

        assert_eq!(before.area.height(), after.area.height());
    }

    #[test]
    fn filtered_sync_records_skipped_text_area_diagnostics() {
        let (composition, mut text_engine, path) = text_area_composition();
        let mut projections = State::default();
        let targets = HashSet::new();

        projections.sync_filtered(
            &composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
            Some(&targets),
        );
        let diagnostics = projections.diagnostics();

        assert_eq!(diagnostics.text_area_surfaces, 1);
        assert_eq!(diagnostics.text_area_skipped_by_filter, 1);
        assert_eq!(diagnostics.text_area_targets, 0);
        assert_eq!(diagnostics.text_area_resolves, 0);
        assert!(projections.text_area(&path).is_none());
    }

    #[test]
    fn idle_refinement_updates_diagnostics_without_losing_projection() {
        let (composition, mut text_engine, path) = text_area_composition();
        let mut projections = State::default();

        projections.sync(
            &composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
        );
        assert!(projections.text_area(&path).is_some());

        assert!(projections.refine_idle_text_area_models(
            &composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
            1,
        ));

        let diagnostics = projections.diagnostics();
        assert_eq!(diagnostics.text_area_idle_refinements, 1);
        assert!(projections.text_area(&path).is_some());
    }
}
