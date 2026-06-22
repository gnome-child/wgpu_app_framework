use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::geometry::{Rect, area, point};
use crate::{text, ui, widget};

const COMPOSITOR_SAMPLE_PADDING: f32 = 2.0;

#[derive(Debug, Default, Clone)]
pub struct Driver {
    projections: HashMap<ui::Path, Projection>,
    metrics: HashMap<ui::Path, widget::scroll::Metrics>,
    adjustments: HashMap<ui::Path, widget::scroll::Adjustment>,
    text_area_models: HashMap<ui::Path, TextAreaModel>,
    retained_layers: HashMap<ui::Path, Vec<RetainedLayer>>,
    shifted_text_area_targets: HashSet<ui::Path>,
    pending_offsets: HashMap<ui::Path, point::Logical>,
    smooth_wheel_scrolls: HashMap<ui::Path, SmoothWheelScroll>,
    wheel_bursts: HashMap<ui::Path, WheelBurst>,
    precision_pixel_wheel_targets: HashSet<ui::Path>,
    motion: Motion,
    pending_diagnostics: Diagnostics,
    diagnostics: Diagnostics,
    last_scroll_diagnostics: LastScrollDiagnostics,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Motion {
    wheel_line_unit_pixels: f32,
    wheel_snap_duration: Duration,
    wheel_min_snap_duration: Duration,
    wheel_impulse_linear_mix: f32,
    wheel_burst_window: Duration,
    wheel_burst_acceleration_start: usize,
    wheel_burst_acceleration_step: f32,
    wheel_burst_acceleration_max: f32,
    wheel_pixel_impulse_unit: f32,
    wheel_snap_distance: f32,
    min_frame_delta: Duration,
    max_frame_delta: Duration,
}

impl Motion {
    pub(crate) fn line_delta_pixels(
        self,
        metrics: widget::scroll::Metrics,
        delta: point::Logical,
    ) -> point::Logical {
        point::logical(
            delta.x() * self.line_stride(metrics.viewport().area.width()),
            delta.y() * self.line_stride(metrics.viewport().area.height()),
        )
    }

    pub(crate) fn fallback_line_delta_pixels(self, delta: point::Logical) -> point::Logical {
        point::logical(
            delta.x() * self.wheel_line_unit_pixels,
            delta.y() * self.wheel_line_unit_pixels,
        )
    }

    pub(crate) fn pixel_impulse_delta(
        self,
        metrics: widget::scroll::Metrics,
        delta: point::Logical,
    ) -> point::Logical {
        let x = self.pixel_impulse_axis(delta.x(), metrics.viewport().area.width());
        let y = self.pixel_impulse_axis(delta.y(), metrics.viewport().area.height());

        point::logical(x, y)
    }

    pub(crate) fn advance(
        self,
        from: point::Logical,
        target: point::Logical,
        started_at: Instant,
        duration: Duration,
        frame: crate::animation::Frame,
    ) -> MotionStep {
        if self.distance(from, target) <= self.wheel_snap_distance {
            return MotionStep::Settled(target);
        }

        let elapsed = frame.now().saturating_duration_since(started_at);
        let progress = if duration.is_zero() {
            1.0
        } else {
            (elapsed.as_secs_f32() / duration.as_secs_f32()).clamp(0.0, 1.0)
        };
        let alpha = wheel_impulse_ease(progress, self.wheel_impulse_linear_mix);
        let next = point::logical(
            from.x() + (target.x() - from.x()) * alpha,
            from.y() + (target.y() - from.y()) * alpha,
        );

        if progress >= 1.0 || self.distance(next, target) <= self.wheel_snap_distance {
            MotionStep::Settled(target)
        } else {
            MotionStep::Advanced { offset: next }
        }
    }

    fn line_stride(self, _viewport_extent: f32) -> f32 {
        self.wheel_line_unit_pixels
    }

    fn pixel_impulse_axis(self, delta: f32, viewport_extent: f32) -> f32 {
        if delta.abs() <= f32::EPSILON {
            return 0.0;
        }

        let notches = (delta.abs() / self.wheel_pixel_impulse_unit).max(1.0);
        delta.signum() * self.line_stride(viewport_extent) * notches
    }

    fn frame_delta(self, delta: Duration) -> Duration {
        if delta.is_zero() {
            Duration::from_secs_f32(1.0 / 60.0)
        } else {
            delta.clamp(self.min_frame_delta, self.max_frame_delta)
        }
    }

    fn snap_duration(
        self,
        metrics: widget::scroll::Metrics,
        from: point::Logical,
        target: point::Logical,
    ) -> Duration {
        let distance = self.distance(from, target);
        if distance <= self.wheel_snap_distance {
            return Duration::ZERO;
        }

        let viewport_extent = if (target.y() - from.y()).abs() >= (target.x() - from.x()).abs() {
            metrics.viewport().area.height()
        } else {
            metrics.viewport().area.width()
        };
        let reference = self.line_stride(viewport_extent).max(1.0);
        let ratio = (distance / reference).max(0.0);
        let min = self.wheel_min_snap_duration.as_secs_f32();
        let max = self.wheel_snap_duration.as_secs_f32();
        let duration = if ratio <= 1.0 {
            min + (max - min) * ratio
        } else {
            (max / ratio.sqrt()).max(min)
        };

        Duration::from_secs_f32(duration)
    }

    fn distance(self, a: point::Logical, b: point::Logical) -> f32 {
        let dx = a.x() - b.x();
        let dy = a.y() - b.y();

        (dx * dx + dy * dy).sqrt()
    }

    fn burst_multiplier(self, event_count: usize) -> f32 {
        if event_count <= self.wheel_burst_acceleration_start {
            return 1.0;
        }

        let accelerated = event_count - self.wheel_burst_acceleration_start;
        (1.0 + accelerated as f32 * self.wheel_burst_acceleration_step)
            .min(self.wheel_burst_acceleration_max)
    }
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            wheel_line_unit_pixels: 28.0,
            wheel_snap_duration: Duration::from_millis(140),
            wheel_min_snap_duration: Duration::from_millis(18),
            wheel_impulse_linear_mix: 0.2,
            wheel_burst_window: Duration::from_millis(120),
            wheel_burst_acceleration_start: 2,
            wheel_burst_acceleration_step: 0.45,
            wheel_burst_acceleration_max: 4.0,
            wheel_pixel_impulse_unit: 120.0,
            wheel_snap_distance: 0.5,
            min_frame_delta: Duration::from_millis(1),
            max_frame_delta: Duration::from_millis(33),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum MotionStep {
    Advanced { offset: point::Logical },
    Settled(point::Logical),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Diagnostics {
    pub wheel_events: usize,
    pub wheel_line_events: usize,
    pub wheel_pixel_events: usize,
    pub wheel_pixel_precision_events: usize,
    pub wheel_pixel_impulse_events: usize,
    pub thumb_drag_moves: usize,
    pub scroll_offset_changes: usize,
    pub scroll_redraw_requests: usize,
    pub queued_scroll_updates: usize,
    pub pending_scroll_updates: usize,
    pub pending_scroll_applications: usize,
    pub frame_scroll_commits: usize,
    pub generic_scroll_projections: usize,
    pub text_area_surfaces: usize,
    pub text_area_targets: usize,
    pub text_area_skipped_by_filter: usize,
    pub text_area_resolves: usize,
    pub text_area_projection_reuses: usize,
    pub text_area_projection_shifts: usize,
    pub text_area_projection_shift_misses: usize,
    pub text_area_projection_cold_jumps: usize,
    pub text_area_model_reuses: usize,
    pub text_area_model_updates: usize,
    pub text_area_idle_refinements: usize,
    pub text_area_idle_refinements_suppressed: usize,
    pub async_scroll_projection_sync_skips: usize,
    pub async_scroll_reconciles: usize,
    pub retained_scroll_translations: usize,
    pub retained_scroll_translated_items: usize,
    pub retained_scroll_chrome_repaints: usize,
    pub retained_scroll_target_repaint_fallbacks: usize,
    pub retained_scroll_layer_hits: usize,
    pub retained_scroll_layer_replaced_items: usize,
    pub retained_scroll_layer_text_prepare_skips: usize,
    pub retained_scroll_layer_missing: usize,
    pub retained_scroll_layer_metrics_misses: usize,
    pub retained_scroll_layer_coverage_misses: usize,
    pub retained_scroll_layer_geometry_misses: usize,
    pub retained_scroll_layer_projection_misses: usize,
    pub retained_scroll_layer_rebuilds: usize,
    pub projection_count: usize,
    pub last_scroll: LastScrollDiagnostics,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LastScrollDiagnostics {
    pub wheel_events: usize,
    pub wheel_line_events: usize,
    pub wheel_pixel_events: usize,
    pub wheel_pixel_precision_events: usize,
    pub wheel_pixel_impulse_events: usize,
    pub thumb_drag_moves: usize,
    pub scroll_offset_changes: usize,
    pub retained_scroll_layer_hits: usize,
    pub retained_scroll_layer_text_prepare_skips: usize,
    pub retained_scroll_target_repaint_fallbacks: usize,
    pub retained_scroll_layer_missing: usize,
    pub retained_scroll_layer_metrics_misses: usize,
    pub retained_scroll_layer_coverage_misses: usize,
    pub retained_scroll_layer_geometry_misses: usize,
    pub retained_scroll_layer_projection_misses: usize,
    pub retained_scroll_layer_rebuilds: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum WheelDelta {
    Lines(point::Logical),
    Pixels {
        delta: point::Logical,
        phase: WheelPhase,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WheelPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct Projection {
    metrics: widget::scroll::Metrics,
    generation: u64,
    text_area: Option<TextAreaProjection>,
}

#[derive(Debug, Clone)]
pub struct TextAreaProjection {
    metrics: widget::scroll::Metrics,
    layout: text::layout::TextFieldLayout,
    surfaces: Vec<text::layout::TextAreaSurface>,
    render_surfaces: Vec<text::layout::TextAreaSurface>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RetainedLayer {
    metrics: widget::scroll::Metrics,
    coverage: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RetainedLayerHit {
    layer_index: usize,
    source: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetainedLayerMiss {
    MissingLayer,
    MetricsChanged,
    CoverageMiss,
    GeometryMismatch,
}

#[derive(Debug, Clone)]
struct TextAreaModel {
    key: text::layout::AreaScrollKey,
    content_size: area::Logical,
    state: text::view::TextViewState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SmoothWheelScroll {
    from: point::Logical,
    target: point::Logical,
    started_at: Option<Instant>,
    duration: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct WheelBurst {
    last_at: Instant,
    direction: point::Logical,
    event_count: usize,
}

impl Driver {
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
        let mut live_scroll_targets = HashSet::new();
        let mut diagnostics = self.take_pending_diagnostics();

        for (path, metrics) in composition.widget_metrics_iter() {
            if let Some(metrics) = (*metrics).scroll() {
                let metrics = self.metrics_preserving_existing_offset(path, metrics);
                live_scroll_targets.insert(path.clone());
                self.metrics.insert(path.clone(), metrics);
                self.adjustments.insert(
                    path.clone(),
                    self.adjustment_preserving_scroll_target(path, metrics),
                );
                diagnostics.generic_scroll_projections += 1;
                projections.insert(
                    path.clone(),
                    Projection {
                        metrics,
                        generation: self
                            .projections
                            .get(path)
                            .map(Projection::generation)
                            .unwrap_or_default(),
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
            let state_offset = point::logical(text_state.scroll_x(), text_state.scroll_y());
            let scroll_offset = projections
                .get(path)
                .map(Projection::metrics)
                .or_else(|| self.projections.get(path).map(Projection::metrics))
                .map(widget::scroll::Metrics::offset)
                .or_else(|| {
                    self.adjustments
                        .get(path)
                        .map(|adjustment| adjustment.offset())
                })
                .unwrap_or(state_offset);
            let layout_state = text_state
                .clone()
                .with_scroll(scroll_offset.x(), scroll_offset.y());
            let current_key = composition.text_area_scroll_key(path);
            if let (Some(model), Some(current_key), Some(existing)) = (
                self.text_area_models.get(path),
                current_key,
                self.projections
                    .get(path)
                    .and_then(Projection::text_area)
                    .cloned(),
            ) {
                if model.key == current_key
                    && model.state.same_except_scroll(&text_state)
                    && existing.metrics().offset() == scroll_offset
                    && !text_state.caret_visibility_pending()
                    && !self.shifted_text_area_targets.contains(path)
                {
                    diagnostics.text_area_projection_reuses += 1;
                    diagnostics.text_area_model_reuses += 1;
                    diagnostics.projection_count += 1;
                    self.metrics.insert(path.clone(), existing.metrics());
                    projections.insert(
                        path.clone(),
                        Projection {
                            metrics: existing.metrics(),
                            generation: self
                                .projections
                                .get(path)
                                .map(Projection::generation)
                                .unwrap_or_default(),
                            text_area: Some(existing),
                        },
                    );
                    continue;
                }
            }
            let hint = self.text_area_models.get(path).map(TextAreaModel::hint);
            if hint.is_some() {
                diagnostics.text_area_model_reuses += 1;
            }
            let Some((metrics, paint_layout, key, content_size)) = composition
                .text_area_scroll_render_layout_with_content_hint(
                    path,
                    layout_state,
                    text_engine,
                    now,
                    hint,
                )
            else {
                continue;
            };
            diagnostics.text_area_resolves += 1;
            let (layout, surfaces, render_surfaces) = paint_layout.into_projection_parts();
            self.metrics.insert(path.clone(), metrics);
            self.adjustments.insert(
                path.clone(),
                self.adjustment_preserving_scroll_target(path, metrics),
            );

            self.text_area_models.insert(
                path.clone(),
                TextAreaModel {
                    key,
                    content_size,
                    state: text_state,
                },
            );
            self.shifted_text_area_targets.remove(path);
            diagnostics.text_area_model_updates += 1;
            projections.insert(
                path.clone(),
                Projection {
                    metrics,
                    generation: self
                        .projections
                        .get(&path)
                        .map(Projection::generation)
                        .unwrap_or_default(),
                    text_area: Some(TextAreaProjection {
                        metrics,
                        layout,
                        surfaces,
                        render_surfaces,
                    }),
                },
            );
        }

        self.text_area_models
            .retain(|path, _| live_text_areas.contains(path));
        diagnostics.projection_count = projections.len();
        self.projections = projections;
        self.adjustments
            .retain(|path, _| live_scroll_targets.contains(path) || live_text_areas.contains(path));
        self.metrics
            .retain(|path, _| live_scroll_targets.contains(path) || live_text_areas.contains(path));
        self.retained_layers
            .retain(|path, _| self.metrics.contains_key(path));
        self.shifted_text_area_targets
            .retain(|path| self.metrics.contains_key(path));
        self.publish_diagnostics(diagnostics);
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

            let Some(model) = self.text_area_models.get(&path).cloned() else {
                continue;
            };
            let text_state = text_field_states.get(&path).cloned().unwrap_or_default();
            let layout_state = self
                .projections
                .get(&path)
                .map(Projection::metrics)
                .map(widget::scroll::Metrics::offset)
                .map(|offset| text_state.clone().with_scroll(offset.x(), offset.y()))
                .unwrap_or_else(|| text_state.clone());
            let Some((metrics, key, content_size)) = composition
                .text_area_scroll_metrics_with_content_hint(
                    &path,
                    layout_state,
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
                self.text_area_models.insert(
                    path.clone(),
                    TextAreaModel {
                        key,
                        content_size,
                        state: text_state.clone(),
                    },
                );
                if let Some(projection) = self.projections.get_mut(&path) {
                    projection.set_metrics(metrics);
                }
            }
            refined += 1;
        }

        self.diagnostics.text_area_idle_refinements += refined;
        refined > 0
    }

    pub fn observe_text_area(
        &mut self,
        composition: &ui::Composition,
        text_field_states: &HashMap<ui::Path, text::view::TextViewState>,
        path: &ui::Path,
        text_engine: &mut text::layout::Engine,
        now: Instant,
    ) -> bool {
        if !composition
            .text_surface(path)
            .is_some_and(text::Surface::is_area)
        {
            return false;
        }

        let text_state = text_field_states.get(path).cloned().unwrap_or_default();
        let state_offset = point::logical(text_state.scroll_x(), text_state.scroll_y());
        let scroll_offset = self
            .metrics(path)
            .map(widget::scroll::Metrics::offset)
            .or_else(|| {
                self.adjustments
                    .get(path)
                    .map(|adjustment| adjustment.offset())
            })
            .unwrap_or(state_offset);
        let layout_state = text_state
            .clone()
            .with_scroll(scroll_offset.x(), scroll_offset.y());
        let hint = self.text_area_models.get(path).map(TextAreaModel::hint);
        let Some((metrics, paint_layout, key, content_size)) = composition
            .text_area_scroll_paint_layout_with_content_hint(
                path,
                layout_state,
                text_engine,
                now,
                hint,
            )
        else {
            return false;
        };
        let (layout, surfaces, render_surfaces) = paint_layout.into_projection_parts();
        let generation = self
            .projections
            .get(path)
            .map(Projection::generation)
            .unwrap_or_default();

        self.metrics.insert(path.clone(), metrics);
        self.adjustments.insert(
            path.clone(),
            self.adjustment_preserving_scroll_target(path, metrics),
        );
        self.text_area_models.insert(
            path.clone(),
            TextAreaModel {
                key,
                content_size,
                state: text_state,
            },
        );
        self.shifted_text_area_targets.remove(path);
        self.projections.insert(
            path.clone(),
            Projection {
                metrics,
                generation,
                text_area: Some(TextAreaProjection {
                    metrics,
                    layout,
                    surfaces,
                    render_surfaces,
                }),
            },
        );
        true
    }

    pub fn clear(&mut self) {
        self.projections.clear();
        self.metrics.clear();
        self.adjustments.clear();
        self.text_area_models.clear();
        self.retained_layers.clear();
        self.shifted_text_area_targets.clear();
        self.pending_offsets.clear();
        self.smooth_wheel_scrolls.clear();
        self.wheel_bursts.clear();
        self.precision_pixel_wheel_targets.clear();
        self.pending_diagnostics = Diagnostics::default();
        self.diagnostics = Diagnostics::default();
        self.last_scroll_diagnostics = LastScrollDiagnostics::default();
    }

    pub fn is_empty(&self) -> bool {
        self.projections.is_empty() && self.metrics.is_empty()
    }

    pub fn queue_offset(&mut self, path: &ui::Path, offset: point::Logical) -> bool {
        self.smooth_wheel_scrolls.remove(path);
        self.wheel_bursts.remove(path);
        self.queue_visual_offset(path, offset, true)
    }

    pub fn wheel_delta_smooths(&mut self, path: &ui::Path, delta: WheelDelta) -> bool {
        let smooth = match delta {
            WheelDelta::Lines(_) => true,
            WheelDelta::Pixels { phase, .. } => match phase {
                WheelPhase::Started => {
                    self.precision_pixel_wheel_targets.insert(path.clone());
                    self.pending_diagnostics.wheel_pixel_precision_events += 1;
                    false
                }
                WheelPhase::Moved => {
                    if self.precision_pixel_wheel_targets.contains(path) {
                        self.pending_diagnostics.wheel_pixel_precision_events += 1;
                        false
                    } else {
                        self.pending_diagnostics.wheel_pixel_impulse_events += 1;
                        true
                    }
                }
                WheelPhase::Ended | WheelPhase::Cancelled => {
                    self.precision_pixel_wheel_targets.remove(path);
                    self.pending_diagnostics.wheel_pixel_precision_events += 1;
                    false
                }
            },
        };
        trace_scroll(format_args!(
            "wheel classify target={path:?} delta={delta:?} smooth={smooth}"
        ));
        smooth
    }

    pub(crate) fn queue_wheel_offset_at(
        &mut self,
        path: &ui::Path,
        offset: point::Logical,
        now: Instant,
    ) -> bool {
        let Some(current_metrics) = self.metrics(path) else {
            return false;
        };
        let raw_target = current_metrics.with_offset(offset).offset();
        let current_target = self
            .adjustments
            .get(path)
            .map(|adjustment| adjustment.offset())
            .unwrap_or_else(|| current_metrics.offset());
        if current_metrics.offset() == raw_target && current_target == raw_target {
            return false;
        }

        let requested_delta = offset_delta(current_target, raw_target);
        let multiplier = self.wheel_burst_multiplier(path, requested_delta, now);
        let target = if multiplier > 1.0 {
            current_metrics
                .with_offset(point::logical(
                    current_target.x() + requested_delta.x() * multiplier,
                    current_target.y() + requested_delta.y() * multiplier,
                ))
                .offset()
        } else {
            raw_target
        };
        self.adjustments.insert(
            path.clone(),
            current_metrics.with_offset(target).adjustment(),
        );
        let visual = current_metrics.offset();
        let active = self.smooth_wheel_scrolls.get(path).copied();
        let preserve_curve = active.is_some_and(|scroll| {
            let old_delta = offset_delta(visual, scroll.target);
            let new_delta = offset_delta(visual, target);

            dot(old_delta, new_delta) > 0.0
                && self.motion.distance(visual, scroll.target) > self.motion.wheel_snap_distance
        });
        let from = if preserve_curve {
            active
                .expect("active smooth wheel scroll should be present")
                .from
        } else {
            visual
        };
        let started_at = if preserve_curve {
            active
                .expect("active smooth wheel scroll should be present")
                .started_at
                .unwrap_or(now)
        } else {
            now
        };
        let duration = self.motion.snap_duration(current_metrics, from, target);
        trace_scroll(format_args!(
            "wheel target target={path:?} visual={:?} current_target={:?} raw_target={:?} requested={:?} multiplier={multiplier:.2} final_target={target:?} preserve_curve={preserve_curve} from={from:?} duration={duration:?}",
            current_metrics.offset(),
            current_target,
            raw_target,
            requested_delta,
        ));
        self.smooth_wheel_scrolls.insert(
            path.clone(),
            SmoothWheelScroll {
                from,
                target,
                started_at: Some(started_at),
                duration,
            },
        );
        self.pending_diagnostics.queued_scroll_updates += 1;
        self.pending_diagnostics.pending_scroll_updates += 1;
        self.pending_offsets
            .insert(path.clone(), current_metrics.offset());

        true
    }

    fn wheel_burst_multiplier(
        &mut self,
        path: &ui::Path,
        delta: point::Logical,
        now: Instant,
    ) -> f32 {
        if self.motion.distance(point::logical(0.0, 0.0), delta) <= self.motion.wheel_snap_distance
        {
            return 1.0;
        }

        let existing = self.wheel_bursts.get(path).copied();
        let same_burst = existing.is_some_and(|burst| {
            now.saturating_duration_since(burst.last_at) <= self.motion.wheel_burst_window
                && dot(burst.direction, delta) > 0.0
        });
        let event_count = if same_burst {
            existing
                .expect("same burst should have existing burst state")
                .event_count
                + 1
        } else {
            1
        };

        self.wheel_bursts.insert(
            path.clone(),
            WheelBurst {
                last_at: now,
                direction: delta,
                event_count,
            },
        );
        self.motion.burst_multiplier(event_count)
    }

    pub fn wheel_delta_pixels(
        &self,
        metrics: widget::scroll::Metrics,
        delta: WheelDelta,
        swap_axes: bool,
    ) -> point::Logical {
        let pixels = match delta {
            WheelDelta::Pixels { delta, .. } => delta,
            WheelDelta::Lines(delta) => self.motion.line_delta_pixels(metrics, delta),
        };

        if swap_axes {
            point::logical(pixels.y(), pixels.x())
        } else {
            pixels
        }
    }

    pub fn wheel_impulse_delta_pixels(
        &self,
        metrics: widget::scroll::Metrics,
        delta: WheelDelta,
        swap_axes: bool,
    ) -> point::Logical {
        let pixels = match delta {
            WheelDelta::Lines(delta) => self.motion.line_delta_pixels(metrics, delta),
            WheelDelta::Pixels { delta, .. } => self.motion.pixel_impulse_delta(metrics, delta),
        };

        if swap_axes {
            let swapped = point::logical(pixels.y(), pixels.x());
            trace_scroll(format_args!(
                "wheel impulse delta raw={delta:?} normalized={swapped:?} viewport={:?} swapped=true",
                metrics.viewport().area
            ));
            swapped
        } else {
            trace_scroll(format_args!(
                "wheel impulse delta raw={delta:?} normalized={pixels:?} viewport={:?} swapped=false",
                metrics.viewport().area
            ));
            pixels
        }
    }

    pub(crate) fn fallback_wheel_delta_pixels(&self, delta: WheelDelta) -> point::Logical {
        delta.fallback_pixels(self.motion)
    }

    fn queue_visual_offset(
        &mut self,
        path: &ui::Path,
        offset: point::Logical,
        update_adjustment: bool,
    ) -> bool {
        let Some(current_metrics) = self.metrics(path) else {
            return false;
        };
        let metrics = current_metrics.with_offset(offset);
        if current_metrics.offset() == metrics.offset() {
            if update_adjustment {
                self.adjustments.insert(path.clone(), metrics.adjustment());
            }
            return false;
        }

        self.metrics.insert(path.clone(), metrics);
        if update_adjustment {
            self.adjustments.insert(path.clone(), metrics.adjustment());
        }
        self.pending_diagnostics.scroll_offset_changes += 1;
        self.pending_diagnostics.queued_scroll_updates += 1;
        self.pending_diagnostics.pending_scroll_updates += 1;
        self.pending_offsets.insert(path.clone(), metrics.offset());

        if let Some(projection) = self.projections.get_mut(path) {
            match projection.set_metrics(metrics) {
                ProjectionUpdate::TextAreaShifted => {
                    trace_scroll(format_args!(
                        "projection shift target={path:?} offset={:?}",
                        metrics.offset()
                    ));
                    self.pending_diagnostics.text_area_projection_shifts += 1;
                    self.shifted_text_area_targets.insert(path.clone());
                }
                ProjectionUpdate::TextAreaDropped => {
                    trace_scroll(format_args!(
                        "projection drop target={path:?} offset={:?}",
                        metrics.offset()
                    ));
                    self.pending_diagnostics.text_area_projection_shift_misses += 1;
                    self.shifted_text_area_targets.remove(path);
                }
                ProjectionUpdate::None => {}
            }
        }
        true
    }

    pub fn update_offset(&mut self, path: &ui::Path, offset: point::Logical) {
        self.queue_offset(path, offset);
    }

    pub fn drain_pending_offsets(&mut self) -> HashMap<ui::Path, point::Logical> {
        let pending = std::mem::take(&mut self.pending_offsets);
        let count = pending.len();
        if count > 0 {
            self.pending_diagnostics.pending_scroll_applications += count;
            self.pending_diagnostics.frame_scroll_commits += count;
        }
        pending
    }

    pub fn advance_smooth_wheel_scrolls(&mut self, frame: crate::animation::Frame) -> bool {
        if self.smooth_wheel_scrolls.is_empty() {
            return false;
        }

        let targets = self
            .smooth_wheel_scrolls
            .iter()
            .map(|(path, scroll)| (path.clone(), *scroll))
            .collect::<Vec<_>>();
        let mut advanced = false;

        for (path, scroll) in targets {
            let Some(current_metrics) = self.metrics(&path) else {
                self.smooth_wheel_scrolls.remove(&path);
                self.adjustments.remove(&path);
                continue;
            };
            let target = current_metrics.with_offset(scroll.target).offset();
            self.adjustments.insert(
                path.clone(),
                current_metrics.with_offset(target).adjustment(),
            );

            let started_at = scroll.started_at.unwrap_or_else(|| {
                frame
                    .now()
                    .checked_sub(self.motion.frame_delta(frame.delta()))
                    .unwrap_or_else(|| frame.now())
            });
            let step = self
                .motion
                .advance(scroll.from, target, started_at, scroll.duration, frame);
            let next = match step {
                MotionStep::Advanced { offset } => offset,
                MotionStep::Settled(next) => {
                    self.smooth_wheel_scrolls.remove(&path);
                    next
                }
            };
            if let Some(scroll) = self.smooth_wheel_scrolls.get_mut(&path) {
                scroll.started_at = Some(started_at);
            }
            advanced |= self.queue_visual_offset(&path, next, false);
        }

        advanced
    }

    pub fn has_active_smooth_wheel_scrolls(&self) -> bool {
        !self.smooth_wheel_scrolls.is_empty()
    }

    pub fn record_thumb_drag_move(&mut self) {
        self.pending_diagnostics.thumb_drag_moves += 1;
    }

    pub fn record_wheel_event(&mut self, delta: WheelDelta) {
        if self.pending_diagnostics.wheel_events == 0 && self.smooth_wheel_scrolls.is_empty() {
            self.last_scroll_diagnostics = LastScrollDiagnostics::default();
        }
        self.pending_diagnostics.wheel_events += 1;
        self.last_scroll_diagnostics.wheel_events += 1;
        match delta {
            WheelDelta::Lines(_) => {
                self.pending_diagnostics.wheel_line_events += 1;
                self.last_scroll_diagnostics.wheel_line_events += 1;
            }
            WheelDelta::Pixels { .. } => {
                self.pending_diagnostics.wheel_pixel_events += 1;
                self.last_scroll_diagnostics.wheel_pixel_events += 1;
            }
        }
        self.diagnostics.last_scroll = self.last_scroll_diagnostics;
    }

    pub fn record_scroll_redraw_request(&mut self) {
        self.pending_diagnostics.scroll_redraw_requests += 1;
    }

    pub fn record_idle_refinement_suppressed_by_scroll(&mut self) {
        self.pending_diagnostics
            .text_area_idle_refinements_suppressed += 1;
    }

    pub fn publish_pending_scroll_diagnostics(&mut self) {
        let pending = self.take_pending_diagnostics();
        if pending != Diagnostics::default() {
            self.publish_diagnostics(pending);
        }
    }

    fn metrics_preserving_existing_offset(
        &self,
        path: &ui::Path,
        metrics: widget::scroll::Metrics,
    ) -> widget::scroll::Metrics {
        self.metrics
            .get(path)
            .copied()
            .or_else(|| self.projections.get(path).map(Projection::metrics))
            .map(|existing| metrics.with_offset(existing.offset()))
            .or_else(|| {
                self.adjustments
                    .get(path)
                    .map(|adjustment| metrics.with_offset(adjustment.offset()))
            })
            .unwrap_or(metrics)
    }

    fn adjustment_preserving_scroll_target(
        &self,
        path: &ui::Path,
        metrics: widget::scroll::Metrics,
    ) -> widget::scroll::Adjustment {
        let target = self
            .smooth_wheel_scrolls
            .get(path)
            .map(|scroll| scroll.target)
            .or_else(|| {
                self.adjustments
                    .get(path)
                    .map(|adjustment| adjustment.offset())
            })
            .unwrap_or_else(|| metrics.offset());

        metrics.with_offset(target).adjustment()
    }

    pub fn record_async_projection_sync_skip(&mut self, targets: usize) {
        self.diagnostics.async_scroll_projection_sync_skips += targets;
        self.remember_scroll_diagnostics();
    }

    pub fn record_async_scroll_reconcile(&mut self) {
        self.diagnostics.async_scroll_reconciles += 1;
        self.remember_scroll_diagnostics();
    }

    pub fn record_retained_layer_hit(&mut self, hit: crate::app::paint_cache::LayerHit) {
        self.diagnostics.retained_scroll_layer_hits += 1;
        self.diagnostics.retained_scroll_layer_replaced_items += hit.replaced_items;
        self.diagnostics.retained_scroll_layer_text_prepare_skips += hit.skipped_text_surfaces;
        self.diagnostics.retained_scroll_chrome_repaints += 1;
        self.remember_scroll_diagnostics();
    }

    pub fn record_retained_layer_miss(&mut self, miss: RetainedLayerMiss) {
        match miss {
            RetainedLayerMiss::MissingLayer => {
                self.diagnostics.retained_scroll_layer_missing += 1;
            }
            RetainedLayerMiss::MetricsChanged => {
                self.diagnostics.retained_scroll_layer_metrics_misses += 1;
            }
            RetainedLayerMiss::CoverageMiss => {
                self.diagnostics.retained_scroll_layer_coverage_misses += 1;
            }
            RetainedLayerMiss::GeometryMismatch => {
                self.diagnostics.retained_scroll_layer_geometry_misses += 1;
            }
        }
        self.remember_scroll_diagnostics();
    }

    pub fn record_retained_projection_miss(&mut self) {
        self.diagnostics.retained_scroll_layer_projection_misses += 1;
        self.remember_scroll_diagnostics();
    }

    pub fn record_retained_layer_rebuild(&mut self) {
        self.diagnostics.retained_scroll_layer_rebuilds += 1;
        self.remember_scroll_diagnostics();
    }

    pub fn record_retained_repaint_fallback(&mut self) {
        self.diagnostics.retained_scroll_target_repaint_fallbacks += 1;
        self.remember_scroll_diagnostics();
    }

    pub fn metrics(&self, path: &ui::Path) -> Option<widget::scroll::Metrics> {
        self.metrics
            .get(path)
            .copied()
            .or_else(|| self.projections.get(path).map(Projection::metrics))
    }

    pub fn visual_offset(&self, path: &ui::Path) -> Option<point::Logical> {
        self.metrics(path)
            .map(widget::scroll::Metrics::offset)
            .or_else(|| {
                self.adjustments
                    .get(path)
                    .map(|adjustment| adjustment.offset())
            })
    }

    pub fn target_offset(&self, path: &ui::Path) -> Option<point::Logical> {
        self.adjustments
            .get(path)
            .map(|adjustment| adjustment.offset())
            .or_else(|| self.metrics(path).map(widget::scroll::Metrics::offset))
    }

    pub(crate) fn metric_paths(&self) -> impl Iterator<Item = &ui::Path> {
        self.metrics.keys()
    }

    pub(crate) fn set_retained_layers<I>(&mut self, layers: I)
    where
        I: IntoIterator<Item = (ui::Path, Vec<RetainedLayer>)>,
    {
        self.retained_layers = layers.into_iter().collect();
    }

    #[cfg(test)]
    pub(crate) fn record_retained_layer(
        &mut self,
        path: ui::Path,
        metrics: widget::scroll::Metrics,
        coverage: Rect,
    ) {
        self.retained_layers
            .insert(path, vec![RetainedLayer::new(metrics, coverage)]);
    }

    #[cfg(test)]
    pub(crate) fn retained_layer_metrics(
        &self,
        path: &ui::Path,
    ) -> Option<widget::scroll::Metrics> {
        self.retained_layers
            .get(path)
            .and_then(|layers| layers.first())
            .map(|layer| layer.metrics())
    }

    pub(crate) fn retained_layer_hit(
        &self,
        path: &ui::Path,
        metrics: widget::scroll::Metrics,
    ) -> Result<RetainedLayerHit, RetainedLayerMiss> {
        let layers = self
            .retained_layers
            .get(path)
            .ok_or(RetainedLayerMiss::MissingLayer)?;
        if layers.is_empty() {
            return Err(RetainedLayerMiss::MissingLayer);
        }

        let mut miss = RetainedLayerMiss::MetricsChanged;
        for (index, layer) in layers.iter().copied().enumerate() {
            match layer.source_for_metrics(metrics) {
                Ok(source) => {
                    return Ok(RetainedLayerHit {
                        layer_index: index,
                        source,
                    });
                }
                Err(RetainedLayerMiss::MissingLayer) => {}
                Err(RetainedLayerMiss::MetricsChanged) => {}
                Err(RetainedLayerMiss::GeometryMismatch) => {
                    miss = RetainedLayerMiss::GeometryMismatch;
                }
                Err(RetainedLayerMiss::CoverageMiss) => {
                    if !matches!(miss, RetainedLayerMiss::GeometryMismatch) {
                        miss = RetainedLayerMiss::CoverageMiss;
                    }
                }
            }
        }

        Err(miss)
    }

    pub(crate) fn plan_retained_layer_coverage(
        &self,
        path: &ui::Path,
        metrics: widget::scroll::Metrics,
    ) -> Rect {
        const ADAPTIVE_MULTIPLIER: f32 = 3.0;
        const MAX_GUARD_VIEWPORTS: f32 = 12.0;
        // Keep the retained backing store within conservative texture limits.
        // Larger/faster gestures belong to tiled/exposed-band updates instead
        // of one expanding texture.
        const MAX_RETAINED_LAYER_LOGICAL_SPAN: f32 = 4096.0;

        let viewport = metrics.viewport();
        let active_axes = metrics.active_axes();
        let previous_offset = self
            .reference_retained_layer_metrics(path, metrics)
            .map(widget::scroll::Metrics::offset)
            .unwrap_or_else(|| metrics.offset());
        let delta = point::logical(
            metrics.offset().x() - previous_offset.x(),
            metrics.offset().y() - previous_offset.y(),
        );
        let horizontal = guard_pair(
            active_axes.horizontal(),
            viewport.area.width(),
            metrics.offset().x(),
            metrics.max_offset().x(),
            delta.x(),
            ADAPTIVE_MULTIPLIER,
            MAX_GUARD_VIEWPORTS,
            MAX_RETAINED_LAYER_LOGICAL_SPAN,
        );
        let vertical = guard_pair(
            active_axes.vertical(),
            viewport.area.height(),
            metrics.offset().y(),
            metrics.max_offset().y(),
            delta.y(),
            ADAPTIVE_MULTIPLIER,
            MAX_GUARD_VIEWPORTS,
            MAX_RETAINED_LAYER_LOGICAL_SPAN,
        );

        let coverage = expand_rect(
            Rect::new(
                point::logical(
                    viewport.origin.x() - horizontal.before,
                    viewport.origin.y() - vertical.before,
                ),
                area::logical(
                    viewport.area.width() + horizontal.before + horizontal.after,
                    viewport.area.height() + vertical.before + vertical.after,
                ),
            ),
            COMPOSITOR_SAMPLE_PADDING,
        );

        clamp_retained_layer_rect_to_content(metrics, coverage)
    }

    pub(crate) fn plan_retained_layer_coverages(
        &self,
        path: &ui::Path,
        metrics: widget::scroll::Metrics,
    ) -> Vec<Rect> {
        const EXTRA_DIRECTIONAL_LAYERS: usize = 2;

        let base = self.plan_retained_layer_coverage(path, metrics);
        let previous_offset = self
            .reference_retained_layer_metrics(path, metrics)
            .map(widget::scroll::Metrics::offset)
            .unwrap_or_else(|| metrics.offset());
        let delta = point::logical(
            metrics.offset().x() - previous_offset.x(),
            metrics.offset().y() - previous_offset.y(),
        );
        let active_axes = metrics.active_axes();
        let viewport = metrics.viewport().area;
        let mut coverages = vec![base];

        if active_axes.vertical() && delta.y() != 0.0 {
            let stride = (base.area.height() - viewport.height())
                .max(viewport.height())
                .max(1.0);
            let direction = delta.y().signum();
            for index in 1..=EXTRA_DIRECTIONAL_LAYERS {
                coverages.push(clamp_retained_layer_rect_to_content(
                    metrics,
                    translate_rect(base, point::logical(0.0, direction * stride * index as f32)),
                ));
            }
        } else if active_axes.horizontal() && delta.x() != 0.0 {
            let stride = (base.area.width() - viewport.width())
                .max(viewport.width())
                .max(1.0);
            let direction = delta.x().signum();
            for index in 1..=EXTRA_DIRECTIONAL_LAYERS {
                coverages.push(clamp_retained_layer_rect_to_content(
                    metrics,
                    translate_rect(base, point::logical(direction * stride * index as f32, 0.0)),
                ));
            }
        }

        coverages.dedup_by(|left, right| same_rect(*left, *right));
        coverages
    }

    fn reference_retained_layer_metrics(
        &self,
        path: &ui::Path,
        metrics: widget::scroll::Metrics,
    ) -> Option<widget::scroll::Metrics> {
        self.retained_layers
            .get(path)?
            .iter()
            .copied()
            .filter(|layer| scroll_metrics_are_translate_compatible(layer.metrics(), metrics))
            .min_by(|left, right| {
                offset_distance_squared(left.metrics().offset(), metrics.offset()).total_cmp(
                    &offset_distance_squared(right.metrics().offset(), metrics.offset()),
                )
            })
            .map(RetainedLayer::metrics)
    }

    pub fn text_area(&self, path: &ui::Path) -> Option<&TextAreaProjection> {
        self.projections.get(path).and_then(Projection::text_area)
    }

    pub(crate) fn from_scroll_metrics(path: ui::Path, metrics: widget::scroll::Metrics) -> Self {
        let mut state = Self::default();
        state.metrics.insert(path.clone(), metrics);
        state.adjustments.insert(path.clone(), metrics.adjustment());
        state.projections.insert(
            path,
            Projection {
                metrics,
                generation: 0,
                text_area: None,
            },
        );
        state
    }

    pub fn text_area_projection_shifted(&self, path: &ui::Path) -> bool {
        self.shifted_text_area_targets.contains(path)
    }

    pub fn diagnostics(&self) -> Diagnostics {
        let mut diagnostics = self.diagnostics;
        diagnostics.last_scroll = self.last_scroll_diagnostics;
        diagnostics
    }

    pub fn widget_hit(&self, position: point::Logical) -> Option<widget::Hit> {
        self.metrics
            .iter()
            .filter_map(|(path, metrics)| {
                metrics
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

impl Driver {
    fn publish_diagnostics(&mut self, mut diagnostics: Diagnostics) {
        if diagnostics.has_scroll_activity() {
            self.last_scroll_diagnostics = diagnostics.last_scroll_snapshot();
        }
        diagnostics.last_scroll = self.last_scroll_diagnostics;
        self.diagnostics = diagnostics;
    }

    fn remember_scroll_diagnostics(&mut self) {
        if self.diagnostics.has_scroll_activity() {
            self.last_scroll_diagnostics = self.diagnostics.last_scroll_snapshot();
            self.diagnostics.last_scroll = self.last_scroll_diagnostics;
        }
    }
}

impl RetainedLayer {
    pub(crate) fn new(metrics: widget::scroll::Metrics, coverage: Rect) -> Self {
        Self { metrics, coverage }
    }

    pub(crate) fn metrics(self) -> widget::scroll::Metrics {
        self.metrics
    }

    pub(crate) fn source_for_metrics(
        self,
        metrics: widget::scroll::Metrics,
    ) -> Result<Rect, RetainedLayerMiss> {
        if !scroll_metrics_are_translate_compatible(self.metrics, metrics) {
            return Err(RetainedLayerMiss::MetricsChanged);
        }

        let scroll_delta = point::logical(
            metrics.offset().x() - self.metrics.offset().x(),
            metrics.offset().y() - self.metrics.offset().y(),
        );
        let source_rect = translate_rect(metrics.viewport(), scroll_delta);
        if !same_area(source_rect.area, metrics.viewport().area) {
            return Err(RetainedLayerMiss::GeometryMismatch);
        }

        let required_source_rect = clamp_retained_layer_rect_to_content(
            self.metrics,
            expand_rect(source_rect, COMPOSITOR_SAMPLE_PADDING),
        );
        if !rect_contains_rect(self.coverage, required_source_rect) {
            return Err(RetainedLayerMiss::CoverageMiss);
        }

        Ok(translate_rect(
            source_rect,
            point::logical(-self.coverage.origin.x(), -self.coverage.origin.y()),
        ))
    }
}

impl RetainedLayerHit {
    pub(crate) fn layer_index(self) -> usize {
        self.layer_index
    }

    pub(crate) fn source(self) -> Rect {
        self.source
    }
}

impl WheelDelta {
    #[cfg(test)]
    pub(crate) fn pixels(delta: point::Logical) -> Self {
        Self::Pixels {
            delta,
            phase: WheelPhase::Started,
        }
    }

    pub(crate) fn pixels_with_phase(delta: point::Logical, phase: WheelPhase) -> Self {
        Self::Pixels { delta, phase }
    }

    pub(crate) fn lines(delta: point::Logical) -> Self {
        Self::Lines(delta)
    }

    pub(crate) fn fallback_pixels(self, motion: Motion) -> point::Logical {
        match self {
            Self::Pixels { delta, .. } => delta,
            Self::Lines(delta) => motion.fallback_line_delta_pixels(delta),
        }
    }
}

impl Diagnostics {
    fn has_scroll_activity(self) -> bool {
        self.wheel_events > 0
            || self.thumb_drag_moves > 0
            || self.scroll_offset_changes > 0
            || self.retained_scroll_layer_hits > 0
            || self.retained_scroll_target_repaint_fallbacks > 0
            || self.retained_scroll_layer_missing > 0
            || self.retained_scroll_layer_metrics_misses > 0
            || self.retained_scroll_layer_coverage_misses > 0
            || self.retained_scroll_layer_geometry_misses > 0
            || self.retained_scroll_layer_projection_misses > 0
            || self.retained_scroll_layer_rebuilds > 0
            || self.text_area_projection_cold_jumps > 0
            || self.async_scroll_projection_sync_skips > 0
            || self.async_scroll_reconciles > 0
    }

    fn last_scroll_snapshot(self) -> LastScrollDiagnostics {
        LastScrollDiagnostics {
            wheel_events: self.wheel_events,
            wheel_line_events: self.wheel_line_events,
            wheel_pixel_events: self.wheel_pixel_events,
            wheel_pixel_precision_events: self.wheel_pixel_precision_events,
            wheel_pixel_impulse_events: self.wheel_pixel_impulse_events,
            thumb_drag_moves: self.thumb_drag_moves,
            scroll_offset_changes: self.scroll_offset_changes,
            retained_scroll_layer_hits: self.retained_scroll_layer_hits,
            retained_scroll_layer_text_prepare_skips: self.retained_scroll_layer_text_prepare_skips,
            retained_scroll_target_repaint_fallbacks: self.retained_scroll_target_repaint_fallbacks,
            retained_scroll_layer_missing: self.retained_scroll_layer_missing,
            retained_scroll_layer_metrics_misses: self.retained_scroll_layer_metrics_misses,
            retained_scroll_layer_coverage_misses: self.retained_scroll_layer_coverage_misses,
            retained_scroll_layer_geometry_misses: self.retained_scroll_layer_geometry_misses,
            retained_scroll_layer_projection_misses: self.retained_scroll_layer_projection_misses,
            retained_scroll_layer_rebuilds: self.retained_scroll_layer_rebuilds,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProjectionUpdate {
    None,
    TextAreaShifted,
    TextAreaDropped,
}

impl Projection {
    pub fn metrics(&self) -> widget::scroll::Metrics {
        self.metrics
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn text_area(&self) -> Option<&TextAreaProjection> {
        self.text_area.as_ref()
    }

    fn set_metrics(&mut self, metrics: widget::scroll::Metrics) -> ProjectionUpdate {
        let offset_changed = self.metrics.offset() != metrics.offset();
        let old_metrics = self.metrics;
        self.metrics = metrics;
        if offset_changed {
            self.generation = self.generation.wrapping_add(1);
        }
        if let Some(text_area) = self.text_area.as_mut() {
            if offset_changed {
                if text_area.translate_for_metrics(old_metrics, metrics) {
                    return ProjectionUpdate::TextAreaShifted;
                }
                self.text_area = None;
                return ProjectionUpdate::TextAreaDropped;
            }
            text_area.metrics = metrics;
        }
        ProjectionUpdate::None
    }
}

impl TextAreaProjection {
    pub fn with_render_surfaces(
        metrics: widget::scroll::Metrics,
        layout: text::layout::TextFieldLayout,
        surfaces: Vec<text::layout::TextAreaSurface>,
        render_surfaces: Vec<text::layout::TextAreaSurface>,
    ) -> Self {
        Self {
            metrics,
            layout,
            surfaces,
            render_surfaces,
        }
    }

    pub fn metrics(&self) -> widget::scroll::Metrics {
        self.metrics
    }

    pub fn layout(&self) -> &text::layout::TextFieldLayout {
        &self.layout
    }

    #[cfg(test)]
    pub fn surfaces(&self) -> &[text::layout::TextAreaSurface] {
        &self.surfaces
    }

    pub fn render_surfaces(&self) -> impl Iterator<Item = &text::layout::TextAreaSurface> {
        let viewport = self.metrics.viewport().area;
        self.render_surfaces
            .iter()
            .filter(move |surface| surface_intersects_viewport(surface, viewport))
    }

    pub fn observed_area(&self) -> text::view::ObservedArea<'_> {
        text::view::ObservedArea::new(
            self.metrics.viewport(),
            self.metrics.offset(),
            self.layout.content_area(),
            &self.surfaces,
        )
    }

    pub fn scroll_anchor(&self, area_model: &text::Area) -> Option<text::ScrollAnchor> {
        text::View::scroll_anchor_for_text_area(
            area_model,
            self.observed_area(),
            &self.render_surfaces,
        )
    }

    fn translate_for_metrics(
        &mut self,
        old_metrics: widget::scroll::Metrics,
        metrics: widget::scroll::Metrics,
    ) -> bool {
        if !same_area(old_metrics.viewport().area, metrics.viewport().area)
            || !same_area(old_metrics.content_size(), metrics.content_size())
            || old_metrics.max_offset() != metrics.max_offset()
        {
            return false;
        }

        if !self.covers_metrics(old_metrics, metrics) {
            return false;
        }

        let old_offset = old_metrics.offset();
        let new_offset = metrics.offset();
        let viewport = metrics.viewport().area;
        let surfaces = self
            .surfaces
            .iter()
            .map(|surface| surface.translated_for_scroll(old_offset, new_offset, viewport))
            .collect::<Vec<_>>();
        if !surfaces.is_empty() && !surfaces_cover_viewport(&surfaces, viewport) {
            return false;
        }
        let render_surfaces = self
            .render_surfaces
            .iter()
            .map(|surface| surface.translated_for_scroll(old_offset, new_offset, viewport))
            .collect::<Vec<_>>();
        if !surfaces_cover_viewport(&render_surfaces, viewport) {
            return false;
        }
        self.layout = self
            .layout
            .translated_for_scroll(new_offset.x(), new_offset.y());
        self.surfaces = surfaces;
        self.render_surfaces = render_surfaces;
        self.metrics = metrics;
        true
    }

    fn covers_metrics(
        &self,
        old_metrics: widget::scroll::Metrics,
        metrics: widget::scroll::Metrics,
    ) -> bool {
        if !same_area(old_metrics.viewport().area, metrics.viewport().area)
            || !same_area(old_metrics.content_size(), metrics.content_size())
            || old_metrics.max_offset() != metrics.max_offset()
        {
            return false;
        }

        if self.surfaces.is_empty() {
            return surfaces_cover_viewport_after_scroll(
                &self.render_surfaces,
                old_metrics.offset(),
                metrics.offset(),
                metrics.viewport().area,
            );
        }

        surfaces_cover_viewport_after_scroll(
            &self.surfaces,
            old_metrics.offset(),
            metrics.offset(),
            metrics.viewport().area,
        )
    }

    #[cfg(test)]
    pub fn buffer(&self) -> std::rc::Rc<std::cell::RefCell<glyphon::Buffer>> {
        self.surfaces
            .first()
            .or_else(|| self.render_surfaces.first())
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

fn same_area(left: area::Logical, right: area::Logical) -> bool {
    left.width().to_bits() == right.width().to_bits()
        && left.height().to_bits() == right.height().to_bits()
}

fn same_rect(left: Rect, right: Rect) -> bool {
    left.origin == right.origin && same_area(left.area, right.area)
}

fn scroll_metrics_are_translate_compatible(
    old: widget::scroll::Metrics,
    new: widget::scroll::Metrics,
) -> bool {
    same_rect(old.viewport(), new.viewport()) && old.active_axes() == new.active_axes()
}

fn translate_rect(rect: Rect, delta: point::Logical) -> Rect {
    Rect::rounded(
        point::logical(rect.origin.x() + delta.x(), rect.origin.y() + delta.y()),
        rect.area,
        rect.rounding,
    )
}

fn rect_contains_rect(outer: Rect, inner: Rect) -> bool {
    let outer_left = outer.origin.x();
    let outer_top = outer.origin.y();
    let outer_right = outer_left + outer.area.width();
    let outer_bottom = outer_top + outer.area.height();
    let inner_left = inner.origin.x();
    let inner_top = inner.origin.y();
    let inner_right = inner_left + inner.area.width();
    let inner_bottom = inner_top + inner.area.height();

    outer_left <= inner_left
        && outer_top <= inner_top
        && outer_right >= inner_right
        && outer_bottom >= inner_bottom
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

fn clamp_retained_layer_rect_to_content(metrics: widget::scroll::Metrics, rect: Rect) -> Rect {
    let viewport = metrics.viewport();
    let bounds = Rect::new(
        point::logical(
            viewport.origin.x() - metrics.offset().x(),
            viewport.origin.y() - metrics.offset().y(),
        ),
        metrics.content_size(),
    );
    intersect_rect(rect, bounds).unwrap_or(viewport)
}

fn intersect_rect(left: Rect, right: Rect) -> Option<Rect> {
    let left_x = left.origin.x().max(right.origin.x());
    let top_y = left.origin.y().max(right.origin.y());
    let right_x = (left.origin.x() + left.area.width()).min(right.origin.x() + right.area.width());
    let bottom_y =
        (left.origin.y() + left.area.height()).min(right.origin.y() + right.area.height());

    (right_x > left_x && bottom_y > top_y).then(|| {
        Rect::rounded(
            point::logical(left_x, top_y),
            area::logical(right_x - left_x, bottom_y - top_y),
            left.rounding,
        )
    })
}

#[derive(Debug, Clone, Copy)]
struct GuardPair {
    before: f32,
    after: f32,
}

fn guard_pair(
    enabled: bool,
    viewport: f32,
    offset: f32,
    max_offset: f32,
    delta: f32,
    adaptive_multiplier: f32,
    max_viewports: f32,
    max_total_span: f32,
) -> GuardPair {
    if !enabled || viewport <= 0.0 {
        return GuardPair {
            before: 0.0,
            after: 0.0,
        };
    }

    let cap = viewport * max_viewports;
    let adaptive = (delta.abs() * adaptive_multiplier).max(viewport).min(cap);
    let mut before = viewport;
    let mut after = viewport;
    if delta > 0.0 {
        after = adaptive;
    } else if delta < 0.0 {
        before = adaptive;
    }

    before = before.min(offset.max(0.0));
    after = after.min((max_offset - offset).max(0.0));

    let max_extra = (max_total_span.max(viewport) - viewport).max(0.0);
    let total = before + after;
    if total > max_extra && total > 0.0 {
        if delta > 0.0 {
            before = before.min(viewport).min(max_extra * 0.33);
            after = after.min((max_extra - before).max(0.0));
        } else if delta < 0.0 {
            after = after.min(viewport).min(max_extra * 0.33);
            before = before.min((max_extra - after).max(0.0));
        } else {
            let scale = max_extra / total;
            before *= scale;
            after *= scale;
        }
    }

    GuardPair { before, after }
}

fn surfaces_cover_viewport(
    surfaces: &[text::layout::TextAreaSurface],
    viewport: area::Logical,
) -> bool {
    if surfaces.is_empty() {
        return false;
    }

    let top = surfaces
        .iter()
        .map(text::layout::TextAreaSurface::y)
        .fold(f32::INFINITY, f32::min);
    let bottom = surfaces
        .iter()
        .map(|surface| surface.y() + surface.height().max(1.0))
        .fold(f32::NEG_INFINITY, f32::max);
    let horizontal = surfaces
        .iter()
        .all(|surface| surface.x() <= 0.0 && surface.x() + surface.width() >= viewport.width());

    top <= 0.0 && bottom >= viewport.height() && horizontal
}

fn surfaces_cover_viewport_after_scroll(
    surfaces: &[text::layout::TextAreaSurface],
    old_scroll: point::Logical,
    new_scroll: point::Logical,
    viewport: area::Logical,
) -> bool {
    if surfaces.is_empty() {
        return false;
    }

    let dx = old_scroll.x() - new_scroll.x();
    let dy = old_scroll.y() - new_scroll.y();
    let top = surfaces
        .iter()
        .map(|surface| surface.y() + dy)
        .fold(f32::INFINITY, f32::min);
    let bottom = surfaces
        .iter()
        .map(|surface| surface.y() + dy + surface.height().max(1.0))
        .fold(f32::NEG_INFINITY, f32::max);
    let horizontal = surfaces.iter().all(|surface| {
        let x = surface.x() + dx;
        x <= 0.0 && x + surface.width() >= viewport.width()
    });

    top <= 0.0 && bottom >= viewport.height() && horizontal
}

fn surface_intersects_viewport(
    surface: &text::layout::TextAreaSurface,
    viewport: area::Logical,
) -> bool {
    let left = surface.x();
    let top = surface.y();
    let right = left + surface.width();
    let bottom = top + surface.height().max(1.0);

    right > 0.0 && bottom > 0.0 && left < viewport.width() && top < viewport.height()
}

fn trace_scroll(args: std::fmt::Arguments<'_>) {
    if std::env::var_os("WGPU_L3_SCROLL_TRACE").is_some() {
        eprintln!("[wgpu_l3 scroll] {args}");
    }
}

fn offset_distance_squared(left: point::Logical, right: point::Logical) -> f32 {
    let x = left.x() - right.x();
    let y = left.y() - right.y();
    x * x + y * y
}

fn offset_delta(from: point::Logical, to: point::Logical) -> point::Logical {
    point::logical(to.x() - from.x(), to.y() - from.y())
}

fn dot(left: point::Logical, right: point::Logical) -> f32 {
    left.x() * right.x() + left.y() * right.y()
}

fn wheel_impulse_ease(progress: f32, linear_mix: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    let linear_mix = linear_mix.clamp(0.0, 1.0);
    let eased = ease_out_cubic(progress);

    (progress * linear_mix + eased * (1.0 - linear_mix)).clamp(0.0, 1.0)
}

fn ease_out_cubic(progress: f32) -> f32 {
    1.0 - (1.0 - progress.clamp(0.0, 1.0)).powi(3)
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::rc::Rc;
    use std::time::{Duration, Instant};

    use crate::animation;
    use crate::geometry::{Rect, area, point};
    use crate::{action, layout, paint, text, ui, widget, window};

    use super::{Driver, Motion, MotionStep, WheelDelta, WheelPhase};

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

    fn multiline_buffer(line_count: usize) -> text::Buffer {
        let text = (0..line_count)
            .map(|line| {
                format!("line {line} with enough text to wrap in a narrow text editor viewport")
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
    fn motion_line_stride_is_platform_line_unit() {
        let motion = Motion::default();

        assert_eq!(motion.line_stride(56.0), 28.0);
        assert_eq!(motion.line_stride(800.0), 28.0);
        assert_eq!(motion.line_stride(2_000.0), 28.0);
    }

    #[test]
    fn motion_wheel_duration_is_inverse_delta_for_fast_bursts() {
        let (composition, mut text_engine, path) = text_area_composition();
        let text_states = HashMap::new();
        let now = Instant::now();
        let driver = Driver::resolve(&composition, &text_states, &mut text_engine, now);
        let metrics = driver
            .metrics(&path)
            .expect("text area should have scroll metrics");
        let motion = Motion::default();
        let start = metrics.offset();
        let reference = motion.line_stride(metrics.viewport().area.height());
        let small = metrics
            .with_offset(point::logical(start.x(), start.y() + reference * 0.25))
            .offset();
        let single = metrics
            .with_offset(point::logical(start.x(), start.y() + reference))
            .offset();
        let burst = metrics
            .with_offset(point::logical(start.x(), start.y() + reference * 4.0))
            .offset();

        let small_duration = motion.snap_duration(metrics, start, small);
        let single_duration = motion.snap_duration(metrics, start, single);
        let burst_duration = motion.snap_duration(metrics, start, burst);

        assert!(
            small_duration < single_duration,
            "small clamped deltas near extents should not use the full notch duration"
        );
        assert!(
            burst_duration < single_duration,
            "large wheel bursts should shorten duration instead of slowing down: burst={burst_duration:?}, single={single_duration:?}"
        );
    }

    #[test]
    fn motion_advance_is_time_based_and_settles_without_overshoot() {
        let motion = Motion::default();
        let now = Instant::now();
        let current = point::logical(0.0, 0.0);
        let target = point::logical(0.0, 120.0);
        let duration = Duration::from_millis(180);

        let step = motion.advance(
            current,
            target,
            now,
            duration,
            animation::Frame::new(now + Duration::from_millis(16), Some(now)),
        );
        let MotionStep::Advanced { offset: next } = step else {
            panic!("first frame should advance toward the target");
        };
        assert!(next.y() > current.y());
        assert!(next.y() < target.y());

        let step = motion.advance(
            point::logical(0.0, 119.8),
            target,
            now,
            duration,
            animation::Frame::new(now + Duration::from_millis(16), Some(now)),
        );
        assert_eq!(step, MotionStep::Settled(target));
    }

    #[test]
    fn motion_larger_snap_target_advances_faster_than_single_notch() {
        let motion = Motion::default();
        let now = Instant::now();
        let frame = animation::Frame::new(now + Duration::from_millis(16), Some(now));
        let duration = Duration::from_millis(180);

        let MotionStep::Advanced { offset: single } = motion.advance(
            point::logical(0.0, 0.0),
            point::logical(0.0, 192.0),
            now,
            duration,
            frame,
        ) else {
            panic!("single notch should advance");
        };
        let MotionStep::Advanced { offset: burst } = motion.advance(
            point::logical(0.0, 0.0),
            point::logical(0.0, 768.0),
            now,
            duration,
            frame,
        ) else {
            panic!("burst target should advance");
        };

        assert!(
            burst.y() > single.y() * 3.5,
            "larger accumulated wheel snap target should move faster: single={single:?}, burst={burst:?}"
        );
    }

    #[test]
    fn motion_single_wheel_notch_spans_multiple_frames() {
        let motion = Motion::default();
        let now = Instant::now();
        let target = point::logical(0.0, 192.0);
        let mut current = point::logical(0.0, 0.0);
        let duration = Duration::from_millis(180);
        let mut advanced_frames = 0;

        for index in 0..10 {
            let frame = animation::Frame::new(
                now + Duration::from_millis(16 * (index + 1)),
                Some(now + Duration::from_millis(16 * index)),
            );
            match motion.advance(current, target, now, duration, frame) {
                MotionStep::Advanced { offset } => {
                    if index == 0 {
                        assert!(
                            offset.y() > 4.0,
                            "single notch should move visibly on the first frame: {offset:?}"
                        );
                        assert!(
                            offset.y() < target.y() * 0.25,
                            "single notch should not jump most of the target on the first frame: {offset:?}"
                        );
                    }
                    current = offset;
                    advanced_frames += 1;
                }
                MotionStep::Settled(_) => break,
            }
        }

        assert!(
            advanced_frames >= 4,
            "single wheel notch should remain visually smooth across several frames, got {advanced_frames}"
        );
    }

    #[test]
    fn smooth_wheel_scroll_advances_visual_offset_toward_target() {
        let (composition, mut text_engine, path) = text_area_composition();
        let text_states = HashMap::new();
        let now = Instant::now();
        let mut driver = Driver::resolve(&composition, &text_states, &mut text_engine, now);
        let metrics = driver
            .metrics(&path)
            .expect("text area should have scroll metrics");
        let start = metrics.offset();
        let target = metrics
            .with_offset(point::logical(start.x(), start.y() + 80.0))
            .offset();
        assert!(target.y() > start.y());

        assert!(driver.queue_wheel_offset_at(&path, target, now));
        assert_eq!(
            driver
                .metrics(&path)
                .expect("visual metrics should remain available")
                .offset(),
            start
        );
        assert!(driver.has_active_smooth_wheel_scrolls());

        let frame = animation::Frame::new(now + Duration::from_millis(16), Some(now));
        assert!(driver.advance_smooth_wheel_scrolls(frame));
        let advanced = driver
            .metrics(&path)
            .expect("visual metrics should advance")
            .offset();
        assert!(advanced.y() > start.y());
        assert!(advanced.y() < target.y());

        for index in 1..90 {
            let frame = animation::Frame::new(
                now + Duration::from_millis(16 * (index + 1)),
                Some(now + Duration::from_millis(16 * index)),
            );
            driver.advance_smooth_wheel_scrolls(frame);
            if !driver.has_active_smooth_wheel_scrolls() {
                break;
            }
        }

        assert!(!driver.has_active_smooth_wheel_scrolls());
        assert_eq!(
            driver
                .metrics(&path)
                .expect("visual metrics should settle")
                .offset(),
            target
        );
    }

    #[test]
    fn direct_scroll_cancels_smooth_wheel_scroll() {
        let (composition, mut text_engine, path) = text_area_composition();
        let text_states = HashMap::new();
        let now = Instant::now();
        let mut driver = Driver::resolve(&composition, &text_states, &mut text_engine, now);
        let metrics = driver
            .metrics(&path)
            .expect("text area should have scroll metrics");
        let target = metrics.with_offset(point::logical(0.0, 80.0)).offset();
        let direct = metrics.with_offset(point::logical(0.0, 24.0)).offset();

        assert!(driver.queue_wheel_offset_at(&path, target, now));
        assert!(driver.has_active_smooth_wheel_scrolls());
        assert!(driver.queue_offset(&path, direct));

        assert!(!driver.has_active_smooth_wheel_scrolls());
        assert_eq!(
            driver
                .metrics(&path)
                .expect("direct metrics should be applied")
                .offset(),
            direct
        );
    }

    #[test]
    fn extending_wheel_target_increases_next_frame_delta() {
        let (composition, mut text_engine, path) = text_area_composition();
        let text_states = HashMap::new();
        let now = Instant::now();
        let mut driver = Driver::resolve(&composition, &text_states, &mut text_engine, now);
        let metrics = driver
            .metrics(&path)
            .expect("text area should have scroll metrics");
        let first_target = metrics.with_offset(point::logical(0.0, 80.0)).offset();
        let second_target = metrics.with_offset(point::logical(0.0, 180.0)).offset();

        assert!(driver.queue_wheel_offset_at(&path, first_target, now));
        assert!(driver.advance_smooth_wheel_scrolls(animation::Frame::new(
            now + Duration::from_millis(16),
            Some(now),
        )));
        let after_first = driver
            .metrics(&path)
            .expect("visual metrics should advance")
            .offset();
        let first_frame_delta = after_first.y() - metrics.offset().y();

        assert!(driver.queue_wheel_offset_at(
            &path,
            second_target,
            now + Duration::from_millis(16)
        ));
        assert_eq!(
            driver
                .smooth_wheel_scrolls
                .get(&path)
                .expect("target extension should keep smooth scroll active")
                .target,
            second_target
        );
        let retargeted = driver
            .smooth_wheel_scrolls
            .get(&path)
            .expect("target extension should keep smooth scroll active");
        assert_eq!(retargeted.from, metrics.offset());
        assert_eq!(retargeted.started_at, Some(now));
        assert!(
            retargeted.duration < Motion::default().wheel_snap_duration,
            "retargeted fast wheel motion should use inverse-delta duration"
        );
        assert!(driver.advance_smooth_wheel_scrolls(animation::Frame::new(
            now + Duration::from_millis(32),
            Some(now + Duration::from_millis(16)),
        )));
        let after_second = driver
            .metrics(&path)
            .expect("visual metrics should advance toward extended target")
            .offset();
        let second_frame_delta = after_second.y() - after_first.y();

        assert!(
            second_frame_delta > first_frame_delta,
            "extending the snap target should increase the next frame motion: first={first_frame_delta}, second={second_frame_delta}"
        );
    }

    #[test]
    fn repeated_wheel_events_accumulate_snap_target_before_frame() {
        let (composition, mut text_engine, path) =
            text_area_composition_for(text::Area::new(multiline_buffer(200)));
        let text_states = HashMap::new();
        let now = Instant::now();
        let mut driver = Driver::resolve(&composition, &text_states, &mut text_engine, now);
        let metrics = driver
            .metrics(&path)
            .expect("text area should have scroll metrics");
        let first_target = metrics.with_offset(point::logical(0.0, 192.0)).offset();
        let second_target = metrics.with_offset(point::logical(0.0, 384.0)).offset();

        assert!(driver.queue_wheel_offset_at(&path, first_target, now));
        assert!(driver.queue_wheel_offset_at(&path, second_target, now));

        assert_eq!(
            driver
                .smooth_wheel_scrolls
                .get(&path)
                .expect("repeated wheel event should keep smooth scroll active")
                .target,
            second_target
        );
    }

    #[test]
    fn full_sync_preserves_in_flight_wheel_target() {
        let (composition, mut text_engine, path) =
            text_area_composition_for(text::Area::new(multiline_buffer(200)));
        let text_states = HashMap::new();
        let now = Instant::now();
        let mut driver = Driver::resolve(&composition, &text_states, &mut text_engine, now);
        let metrics = driver
            .metrics(&path)
            .expect("text area should have scroll metrics");
        let target = metrics.with_offset(point::logical(0.0, 320.0)).offset();

        assert!(driver.queue_wheel_offset_at(&path, target, now));
        assert!(driver.advance_smooth_wheel_scrolls(animation::Frame::new(
            now + Duration::from_millis(16),
            Some(now),
        )));
        let visual = driver
            .metrics(&path)
            .expect("visual metrics should advance")
            .offset();
        assert!(
            visual.y() > 0.0 && visual.y() < target.y(),
            "test should leave an in-flight target beyond the visual offset"
        );

        driver.sync(&composition, &text_states, &mut text_engine, now);

        assert_eq!(
            driver
                .target_offset(&path)
                .expect("wheel target should survive full sync"),
            target
        );
        assert_eq!(
            driver
                .metrics(&path)
                .expect("visual metrics should survive full sync")
                .offset(),
            visual
        );
    }

    #[test]
    fn rapid_wheel_burst_accelerates_target_distance() {
        let (composition, mut text_engine, path) =
            text_area_composition_for(text::Area::new(multiline_buffer(200)));
        let text_states = HashMap::new();
        let now = Instant::now();
        let mut driver = Driver::resolve(&composition, &text_states, &mut text_engine, now);
        let metrics = driver
            .metrics(&path)
            .expect("text area should have scroll metrics");
        let stride = driver
            .wheel_delta_pixels(metrics, WheelDelta::lines(point::logical(0.0, -1.0)), false)
            .y()
            .abs();

        let first_raw = metrics
            .with_offset(point::logical(0.0, metrics.offset().y() + stride))
            .offset();
        assert!(driver.queue_wheel_offset_at(&path, first_raw, now));
        let first_target = driver
            .target_offset(&path)
            .expect("first wheel target should be recorded");

        let second_raw = metrics
            .with_offset(point::logical(0.0, first_target.y() + stride))
            .offset();
        assert!(driver.queue_wheel_offset_at(&path, second_raw, now + Duration::from_millis(20)));
        let second_target = driver
            .target_offset(&path)
            .expect("second wheel target should be recorded");

        let third_raw = metrics
            .with_offset(point::logical(0.0, second_target.y() + stride))
            .offset();
        assert!(driver.queue_wheel_offset_at(&path, third_raw, now + Duration::from_millis(40)));
        let third_target = driver
            .target_offset(&path)
            .expect("third wheel target should be recorded");

        let first_delta = first_target.y() - metrics.offset().y();
        let third_delta = third_target.y() - second_target.y();

        assert!(
            third_delta > first_delta,
            "rapid wheel bursts should amplify same-direction target distance: first={first_delta}, third={third_delta}"
        );
    }

    #[test]
    fn wheel_burst_resets_after_pause() {
        let (composition, mut text_engine, path) =
            text_area_composition_for(text::Area::new(multiline_buffer(200)));
        let text_states = HashMap::new();
        let now = Instant::now();
        let mut driver = Driver::resolve(&composition, &text_states, &mut text_engine, now);
        let metrics = driver
            .metrics(&path)
            .expect("text area should have scroll metrics");
        let stride = driver
            .wheel_delta_pixels(metrics, WheelDelta::lines(point::logical(0.0, -1.0)), false)
            .y()
            .abs();

        let first = metrics.with_offset(point::logical(0.0, stride)).offset();
        assert!(driver.queue_wheel_offset_at(&path, first, now));
        let second = metrics
            .with_offset(point::logical(0.0, stride * 2.0))
            .offset();
        assert!(driver.queue_wheel_offset_at(&path, second, now + Duration::from_millis(20)));
        let paused = driver
            .target_offset(&path)
            .expect("second target should be recorded");
        let raw_after_pause = metrics
            .with_offset(point::logical(0.0, paused.y() + stride))
            .offset();
        assert!(driver.queue_wheel_offset_at(
            &path,
            raw_after_pause,
            now + Duration::from_millis(240)
        ));
        let after_pause = driver
            .target_offset(&path)
            .expect("paused wheel target should be recorded");

        assert!(
            (after_pause.y() - paused.y() - stride).abs() <= f32::EPSILON,
            "isolated wheel events after a pause should return to the baseline stride"
        );
    }

    #[test]
    fn moved_pixel_delta_without_precision_start_uses_impulse_wheel_path() {
        let (composition, mut text_engine, path) =
            text_area_composition_for(text::Area::new(multiline_buffer(200)));
        let text_states = HashMap::new();
        let now = Instant::now();
        let mut driver = Driver::resolve(&composition, &text_states, &mut text_engine, now);
        let metrics = driver
            .metrics(&path)
            .expect("text area should have scroll metrics");
        let delta = WheelDelta::pixels_with_phase(point::logical(0.0, -24.0), WheelPhase::Moved);

        assert!(driver.wheel_delta_smooths(&path, delta));
        let pixels = driver.wheel_delta_pixels(metrics, delta, false);
        let target = metrics.wheel_offset(pixels);
        assert!(driver.queue_wheel_offset_at(&path, target, now));

        assert!(driver.has_active_smooth_wheel_scrolls());
        driver.publish_pending_scroll_diagnostics();
        let diagnostics = driver.diagnostics();
        assert_eq!(diagnostics.wheel_pixel_impulse_events, 1);
        assert_eq!(diagnostics.wheel_pixel_precision_events, 0);
    }

    #[test]
    fn tiny_pixel_impulse_normalizes_to_notch_distance() {
        let (composition, mut text_engine, path) =
            text_area_composition_for(text::Area::new(multiline_buffer(200)));
        let text_states = HashMap::new();
        let now = Instant::now();
        let mut driver = Driver::resolve(&composition, &text_states, &mut text_engine, now);
        let metrics = driver
            .metrics(&path)
            .expect("text area should have scroll metrics");
        let tiny = WheelDelta::pixels_with_phase(point::logical(0.0, -1.0), WheelPhase::Moved);

        assert!(driver.wheel_delta_smooths(&path, tiny));
        assert_eq!(
            driver.wheel_impulse_delta_pixels(metrics, tiny, false),
            driver.wheel_delta_pixels(metrics, WheelDelta::lines(point::logical(0.0, -1.0)), false)
        );
    }

    #[test]
    fn precision_pixel_delta_with_started_phase_stays_direct() {
        let (composition, mut text_engine, path) =
            text_area_composition_for(text::Area::new(multiline_buffer(200)));
        let text_states = HashMap::new();
        let now = Instant::now();
        let mut driver = Driver::resolve(&composition, &text_states, &mut text_engine, now);

        assert!(!driver.wheel_delta_smooths(
            &path,
            WheelDelta::pixels_with_phase(point::logical(0.0, -4.0), WheelPhase::Started)
        ));
        assert!(!driver.wheel_delta_smooths(
            &path,
            WheelDelta::pixels_with_phase(point::logical(0.0, -8.0), WheelPhase::Moved)
        ));
        assert!(!driver.wheel_delta_smooths(
            &path,
            WheelDelta::pixels_with_phase(point::logical(0.0, -2.0), WheelPhase::Ended)
        ));

        driver.publish_pending_scroll_diagnostics();
        let diagnostics = driver.diagnostics();
        assert_eq!(diagnostics.wheel_pixel_impulse_events, 0);
        assert_eq!(diagnostics.wheel_pixel_precision_events, 3);
    }

    #[test]
    fn unchanged_extent_target_keeps_wheel_animation_active_until_visual_offset_arrives() {
        let (composition, mut text_engine, path) =
            text_area_composition_for(text::Area::new(multiline_buffer(200)));
        let text_states = HashMap::new();
        let now = Instant::now();
        let mut driver = Driver::resolve(&composition, &text_states, &mut text_engine, now);
        let metrics = driver
            .metrics(&path)
            .expect("text area should have scroll metrics");
        let max = metrics.max_offset();
        assert!(max.y() > 200.0);
        let visual = point::logical(max.x(), max.y() - 24.0);
        let target_metrics = metrics.with_offset(max);

        driver
            .metrics
            .insert(path.clone(), metrics.with_offset(visual));
        driver
            .adjustments
            .insert(path.clone(), target_metrics.adjustment());
        assert!(driver.queue_wheel_offset_at(&path, max, now));
        assert_eq!(
            driver
                .smooth_wheel_scrolls
                .get(&path)
                .expect("wheel scroll should stay active while moving toward extent")
                .target,
            max
        );
    }

    #[test]
    fn fast_snap_target_moves_more_than_slow_snap_target_on_first_frame() {
        let path = ui::Path::new([ROOT, AREA]);
        let viewport = Rect::new(point::logical(0.0, 0.0), area::logical(120.0, 56.0));
        let metrics = widget::scroll::Metrics::resolve(
            viewport,
            viewport,
            area::logical(120.0, 5_000.0),
            point::logical(0.0, 0.0),
            widget::scroll::Axes::vertical(),
            widget::scroll::Bars::vertical(),
            widget::scroll::Style::default(),
        );
        let now = Instant::now();
        let mut slow = Driver::from_scroll_metrics(path.clone(), metrics);
        let slow_target = metrics.with_offset(point::logical(0.0, 192.0)).offset();
        assert!(slow.queue_wheel_offset_at(&path, slow_target, now));
        assert!(slow.advance_smooth_wheel_scrolls(animation::Frame::new(
            now + Duration::from_millis(16),
            Some(now),
        )));
        let slow_delta = slow
            .metrics(&path)
            .expect("slow metrics should advance")
            .offset()
            .y()
            - metrics.offset().y();

        let mut fast = Driver::from_scroll_metrics(path.clone(), metrics);
        let fast_target = metrics.with_offset(point::logical(0.0, 768.0)).offset();
        assert!(fast.queue_wheel_offset_at(&path, fast_target, now));
        assert!(fast.advance_smooth_wheel_scrolls(animation::Frame::new(
            now + Duration::from_millis(16),
            Some(now),
        )));
        let fast_delta = fast
            .metrics(&path)
            .expect("fast metrics should advance")
            .offset()
            .y()
            - metrics.offset().y();

        assert!(
            fast_delta > slow_delta * 3.5,
            "fast accumulated snap target should move more than a single notch: slow={slow_delta}, fast={fast_delta}"
        );
    }

    #[test]
    fn wheel_line_delta_uses_platform_line_units() {
        let (composition, mut text_engine, path) = text_area_composition();
        let text_states = HashMap::new();
        let driver = Driver::resolve(&composition, &text_states, &mut text_engine, Instant::now());
        let metrics = driver
            .metrics(&path)
            .expect("text area should have scroll metrics");

        assert_eq!(
            driver
                .wheel_delta_pixels(metrics, WheelDelta::lines(point::logical(0.0, -3.0)), false,),
            point::logical(0.0, -84.0)
        );
        assert_eq!(
            driver.wheel_delta_pixels(
                metrics,
                WheelDelta::pixels(point::logical(0.0, -12.0)),
                false,
            ),
            point::logical(0.0, -12.0)
        );
    }

    #[test]
    fn text_area_projection_carries_metrics_and_prepared_surface() {
        let (composition, mut text_engine, path) = text_area_composition();
        let projections = Driver::resolve(
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
    fn retained_layer_coverage_expands_for_large_active_scroll_delta() {
        let path = ui::Path::new([ROOT, AREA]);
        let viewport = Rect::new(point::logical(0.0, 0.0), area::logical(100.0, 100.0));
        let metrics = widget::scroll::Metrics::resolve(
            viewport,
            viewport,
            area::logical(100.0, 5000.0),
            point::logical(0.0, 0.0),
            widget::scroll::Axes::vertical(),
            widget::scroll::Bars::vertical(),
            widget::scroll::Style::default(),
        );
        let mut driver = Driver::default();
        driver.record_retained_layer(path.clone(), metrics, metrics.viewport());

        let scrolled = metrics.with_offset(point::logical(0.0, 800.0));
        let coverage = driver.plan_retained_layer_coverage(&path, scrolled);
        let viewport_height = scrolled.viewport().area.height();

        assert!(coverage.origin.y() <= scrolled.viewport().origin.y() - viewport_height);
        assert!(coverage.area.height() >= viewport_height * 10.0);
    }

    #[test]
    fn retained_layer_coverage_clamps_sample_padding_to_content_bounds() {
        let path = ui::Path::new([ROOT, AREA]);
        let viewport = Rect::new(point::logical(0.0, 0.0), area::logical(100.0, 100.0));
        let metrics = widget::scroll::Metrics::resolve(
            viewport,
            viewport,
            area::logical(100.0, 5000.0),
            point::logical(0.0, 0.0),
            widget::scroll::Axes::vertical(),
            widget::scroll::Bars::vertical(),
            widget::scroll::Style::default(),
        );
        let driver = Driver::default();

        let coverage = driver.plan_retained_layer_coverage(&path, metrics);
        let hit = super::RetainedLayer::new(metrics, coverage)
            .source_for_metrics(metrics)
            .expect("initial retained layer should cover the visible viewport");

        assert_eq!(coverage.origin.y(), metrics.viewport().origin.y());
        assert_eq!(hit.origin.y(), 0.0);
    }

    #[test]
    fn retained_layer_hit_uses_region_that_contains_current_source_rect() {
        let path = ui::Path::new([ROOT, AREA]);
        let viewport = Rect::new(point::logical(0.0, 0.0), area::logical(100.0, 100.0));
        let metrics = widget::scroll::Metrics::resolve(
            viewport,
            viewport,
            area::logical(100.0, 5000.0),
            point::logical(0.0, 0.0),
            widget::scroll::Axes::vertical(),
            widget::scroll::Bars::vertical(),
            widget::scroll::Style::default(),
        );
        let shifted_metrics = metrics.with_offset(point::logical(0.0, 1000.0));
        let mut driver = Driver::default();
        driver.set_retained_layers([(
            path.clone(),
            vec![
                super::RetainedLayer::new(
                    metrics,
                    Rect::new(point::logical(-2.0, -2.0), area::logical(104.0, 204.0)),
                ),
                super::RetainedLayer::new(
                    shifted_metrics,
                    Rect::new(point::logical(-2.0, -2.0), area::logical(104.0, 204.0)),
                ),
            ],
        )]);

        let hit = driver
            .retained_layer_hit(&path, metrics.with_offset(point::logical(0.0, 1050.0)))
            .expect("second retained region should cover the scrolled viewport");

        assert_eq!(hit.layer_index(), 1);
        assert_eq!(
            hit.source(),
            Rect::new(point::logical(2.0, 52.0), area::logical(90.0, 100.0))
        );
    }

    #[test]
    fn wrapped_text_area_vertical_overflow_does_not_activate_horizontal_scrollbar() {
        let (composition, mut text_engine, path) = text_area_composition();
        let mut projections = Driver::resolve(
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
        let projections = Driver::resolve(
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
        let projections = Driver::resolve(
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
        let projections = Driver::resolve(
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
        let projections = Driver::resolve(
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
        let text = (0..500)
            .map(|line| format!("line {line} with enough text to wrap in a narrow area"))
            .collect::<Vec<_>>()
            .join("\n");
        let (composition, mut text_engine, path) =
            text_area_composition_for(text::Area::new(text::Buffer::from_multiline_text(text)));
        let initial_state = text::view::TextViewState::default();
        let mut projections = Driver::resolve(
            &composition,
            &HashMap::from([(path.clone(), initial_state.clone())]),
            &mut text_engine,
            Instant::now(),
        );
        assert!(projections.text_area(&path).is_some());
        let offset = point::logical(
            0.0,
            projections
                .metrics(&path)
                .expect("metrics should exist")
                .max_offset()
                .y(),
        );
        assert!(offset.y() > 120.0);

        assert!(projections.queue_offset(&path, offset));

        assert_eq!(
            projections
                .metrics(&path)
                .expect("metrics should remain")
                .offset(),
            offset
        );
        assert!(projections.text_area(&path).is_none());
        assert_eq!(
            projections.drain_pending_offsets(),
            HashMap::from([(path.clone(), offset)])
        );

        projections.sync(
            &composition,
            &HashMap::from([(path.clone(), initial_state.with_scroll_y(offset.y()))]),
            &mut text_engine,
            Instant::now(),
        );
        let rebuilt = projections
            .text_area(&path)
            .expect("sync should rebuild text area projection");
        assert_eq!(rebuilt.metrics().offset(), offset);
    }

    #[test]
    fn full_sync_preserves_driver_offset_for_scrollless_text_area_state() {
        let (composition, mut text_engine, path) = text_area_composition();
        let mut projections = Driver::resolve(
            &composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
        );
        let offset = point::logical(
            0.0,
            projections
                .metrics(&path)
                .expect("metrics should exist")
                .max_offset()
                .y()
                .min(96.0),
        );
        assert!(offset.y() > 0.0);

        assert!(projections.queue_offset(&path, offset));
        assert_eq!(
            projections.drain_pending_offsets(),
            HashMap::from([(path.clone(), offset)])
        );

        projections.sync(
            &composition,
            &HashMap::from([(path.clone(), text::view::TextViewState::default())]),
            &mut text_engine,
            Instant::now(),
        );

        assert_eq!(
            projections
                .metrics(&path)
                .expect("metrics should survive full sync")
                .offset(),
            offset
        );
        assert_eq!(
            projections
                .text_area(&path)
                .expect("text area projection should survive full sync")
                .metrics()
                .offset(),
            offset
        );
    }

    #[test]
    fn full_sync_preserves_adjustment_after_projection_is_missing() {
        let (composition, mut text_engine, path) = text_area_composition();
        let mut projections = Driver::resolve(
            &composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
        );
        let offset = point::logical(
            0.0,
            projections
                .metrics(&path)
                .expect("metrics should exist")
                .max_offset()
                .y()
                .min(96.0),
        );
        assert!(offset.y() > 0.0);

        assert!(projections.queue_offset(&path, offset));
        projections.drain_pending_offsets();
        projections.projections.clear();

        projections.sync(
            &composition,
            &HashMap::from([(path.clone(), text::view::TextViewState::default())]),
            &mut text_engine,
            Instant::now(),
        );

        assert_eq!(
            projections
                .metrics(&path)
                .expect("metrics should be rebuilt from durable adjustment")
                .offset(),
            offset
        );
        assert_eq!(
            projections
                .text_area(&path)
                .expect("text area projection should be rebuilt")
                .metrics()
                .offset(),
            offset
        );
    }

    #[test]
    fn painting_text_area_uses_adjustment_when_projection_is_missing() {
        let (composition, mut text_engine, path) = text_area_composition();
        let mut projections = Driver::resolve(
            &composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
        );
        let offset = point::logical(0.0, 40.0);
        assert!(
            projections
                .metrics(&path)
                .expect("metrics should exist")
                .max_offset()
                .y()
                > offset.y()
        );
        assert!(projections.queue_offset(&path, offset));
        projections.drain_pending_offsets();
        projections.projections.clear();

        let mut scene = paint::Scene::new();
        composition.paint_at_recording_scroll_ranges(
            &action::Registry::<()>::new(),
            window::Id::new(1),
            ui::Interaction::default(),
            &HashMap::from([(path.clone(), text::view::TextViewState::default())]),
            &mut text_engine,
            crate::animation::Frame::new(Instant::now(), None),
            Some(&projections),
            &mut scene,
        );

        let first_text_surface_y = scene.items().iter().find_map(|item| match item {
            paint::Item::TextSurface(surface) => Some(surface.rect.origin.y()),
            paint::Item::TextViewport(viewport) => viewport
                .surfaces
                .first()
                .map(|surface| surface.rect.origin.y()),
            _ => None,
        });

        assert!(
            first_text_surface_y.is_some_and(|y| y < 0.0),
            "text area paint should use scroll-driver adjustment, not stored zero text state: {first_text_surface_y:?}"
        );
    }

    #[test]
    fn painting_text_area_uses_supplied_scroll_metrics_viewport_when_projection_is_missing() {
        let (composition, mut text_engine, path) =
            text_area_composition_for(text::Area::new(multiline_buffer(200)));
        let projections = Driver::resolve(
            &composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
        );
        let metrics = projections
            .metrics(&path)
            .expect("text area should have scroll metrics");
        let viewport = metrics.viewport();
        let layer_viewport = Rect::new(
            viewport.origin,
            area::logical(viewport.area.width(), viewport.area.height() * 2.0),
        );
        let layer_metrics = metrics.with_layer_viewport(layer_viewport, metrics.offset());
        let layer_scroll = Driver::from_scroll_metrics(path.clone(), layer_metrics);

        let mut scene = paint::Scene::new();
        composition.paint_at_recording_scroll_ranges(
            &action::Registry::<()>::new(),
            window::Id::new(1),
            ui::Interaction::default(),
            &HashMap::from([(path.clone(), text::view::TextViewState::default())]),
            &mut text_engine,
            crate::animation::Frame::new(Instant::now(), None),
            Some(&layer_scroll),
            &mut scene,
        );

        let viewport = scene
            .items()
            .iter()
            .find_map(|item| match item {
                paint::Item::TextViewport(viewport) => Some(viewport),
                _ => None,
            })
            .expect("text area should paint a text viewport");

        assert_eq!(viewport.rect, layer_viewport);
    }

    #[test]
    fn full_sync_clamps_existing_offset_when_content_height_changes() {
        let (large_composition, mut text_engine, path) =
            text_area_composition_for(text::Area::new(multiline_buffer(200)));
        let mut projections = Driver::resolve(
            &large_composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
        );
        let old_offset = point::logical(
            0.0,
            projections
                .metrics(&path)
                .expect("large metrics should exist")
                .max_offset()
                .y(),
        );
        assert!(old_offset.y() > 0.0);
        assert!(projections.queue_offset(&path, old_offset));
        projections.drain_pending_offsets();

        let (small_composition, mut small_text_engine, _) =
            text_area_composition_for(text::Area::new(multiline_buffer(12)));
        let expected = Driver::resolve(
            &small_composition,
            &HashMap::new(),
            &mut small_text_engine,
            Instant::now(),
        )
        .metrics(&path)
        .expect("small metrics should exist")
        .max_offset();
        assert!(expected.y() > 0.0);
        assert!(expected.y() < old_offset.y());

        projections.sync(
            &small_composition,
            &HashMap::from([(path.clone(), text::view::TextViewState::default())]),
            &mut text_engine,
            Instant::now(),
        );

        assert_eq!(
            projections
                .metrics(&path)
                .expect("metrics should survive changed content height")
                .offset(),
            point::logical(0.0, expected.y())
        );
    }

    #[test]
    fn small_scroll_offsets_refresh_shifted_text_area_projection_on_next_sync() {
        let (composition, mut text_engine, path) = text_area_composition();
        let initial_state = text::view::TextViewState::default();
        let mut projections = Driver::resolve(
            &composition,
            &HashMap::from([(path.clone(), initial_state.clone())]),
            &mut text_engine,
            Instant::now(),
        );
        assert!(projections.text_area(&path).is_some());

        projections.record_wheel_event(WheelDelta::lines(point::logical(0.0, -1.0)));
        projections.record_wheel_event(WheelDelta::lines(point::logical(0.0, -1.0)));
        projections.record_scroll_redraw_request();
        assert!(projections.queue_offset(&path, point::logical(0.0, 1.0)));
        assert!(projections.queue_offset(&path, point::logical(0.0, 2.0)));
        assert_eq!(
            projections
                .metrics(&path)
                .expect("metrics should update cheaply")
                .offset(),
            point::logical(0.0, 2.0)
        );
        assert!(
            projections.text_area(&path).is_some(),
            "small scroll should shift retained projection"
        );
        assert_eq!(
            projections.drain_pending_offsets(),
            HashMap::from([(path.clone(), point::logical(0.0, 2.0))])
        );

        text_engine.reset_diagnostics();
        projections.sync(
            &composition,
            &HashMap::from([(path.clone(), initial_state.with_scroll_y(2.0))]),
            &mut text_engine,
            Instant::now(),
        );
        let diagnostics = projections.diagnostics();
        assert!(
            projections.text_area(&path).is_some(),
            "shifted projection should refresh against the committed scroll offset"
        );
        assert_eq!(diagnostics.wheel_events, 2);
        assert_eq!(diagnostics.scroll_redraw_requests, 1);
        assert_eq!(diagnostics.pending_scroll_updates, 2);
        assert_eq!(diagnostics.pending_scroll_applications, 1);
        assert_eq!(diagnostics.text_area_projection_shifts, 2);
        assert_eq!(diagnostics.text_area_projection_reuses, 0);
        assert_eq!(diagnostics.text_area_resolves, 1);
        assert_eq!(text_engine.diagnostics().text_area_paint_layout_calls, 1);
    }

    #[test]
    fn projection_sync_preserves_last_scroll_input_diagnostics() {
        let (composition, mut text_engine, path) = text_area_composition();
        let initial_state = text::view::TextViewState::default();
        let now = Instant::now();
        let mut projections = Driver::resolve(
            &composition,
            &HashMap::from([(path.clone(), initial_state.clone())]),
            &mut text_engine,
            now,
        );
        let metrics = projections
            .metrics(&path)
            .expect("text area should have scroll metrics");
        let delta = WheelDelta::lines(point::logical(0.0, -1.0));
        let pixels = projections.wheel_delta_pixels(metrics, delta, false);
        let target = metrics.wheel_offset(pixels);

        projections.record_wheel_event(delta);
        assert_eq!(projections.diagnostics().last_scroll.wheel_events, 1);
        assert!(projections.queue_wheel_offset_at(&path, target, now));
        assert!(
            projections.advance_smooth_wheel_scrolls(animation::Frame::new(
                now + Duration::from_millis(16),
                Some(now),
            ))
        );
        assert_eq!(
            projections
                .drain_pending_offsets()
                .keys()
                .collect::<Vec<_>>(),
            vec![&path]
        );
        projections.publish_pending_scroll_diagnostics();

        projections.sync(
            &composition,
            &HashMap::from([(path.clone(), initial_state.with_scroll_y(target.y()))]),
            &mut text_engine,
            now + Duration::from_millis(16),
        );
        let diagnostics = projections.diagnostics();

        assert_eq!(
            diagnostics.last_scroll.wheel_events, 1,
            "projection sync should not erase the input that drove the scroll frame"
        );
        assert_eq!(diagnostics.last_scroll.wheel_line_events, 1);
    }

    #[test]
    fn vertical_projection_shift_preserves_text_surface_geometry() {
        let (composition, mut text_engine, path) = text_area_composition();
        let mut projections = Driver::resolve(
            &composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
        );
        let before = projections
            .text_area(&path)
            .expect("text area should have a projection")
            .render_surfaces()
            .map(|surface| (surface.width(), surface.height()))
            .collect::<Vec<_>>();
        assert!(!before.is_empty());

        assert!(projections.queue_offset(&path, point::logical(0.0, 2.0)));
        let after = projections
            .text_area(&path)
            .expect("vertical scroll should shift projection")
            .render_surfaces()
            .map(|surface| (surface.width(), surface.height()))
            .collect::<Vec<_>>();

        assert_eq!(after, before);
    }

    #[test]
    fn fast_vertical_projection_shift_uses_text_render_guard() {
        let (composition, mut text_engine, path) =
            text_area_composition_for(text::Area::new(multiline_buffer(500)));
        let mut projections = Driver::resolve(
            &composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
        );
        let metrics = projections
            .metrics(&path)
            .expect("text area should have scroll metrics");
        let offset = point::logical(0.0, 300.0);

        assert!(
            metrics.max_offset().y() > offset.y(),
            "fixture must be tall enough to exercise a fast visual scroll step"
        );
        assert!(projections.queue_offset(&path, offset));

        assert!(
            projections.text_area(&path).is_some(),
            "a fast scroll step inside the render guard should shift the glyphon surface, not drop the text projection"
        );
        assert!(projections.text_area_projection_shifted(&path));
    }

    #[test]
    fn text_area_projection_keeps_render_surface_without_observed_coverage() {
        let (composition, mut text_engine, path) =
            text_area_composition_for(text::Area::new(multiline_buffer(200)));
        let projections = Driver::resolve(
            &composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
        );
        let projection = projections
            .text_area(&path)
            .expect("text area should have a projection");
        let retained = projection.surfaces().len();
        let rendered = projection.render_surfaces().count();

        assert_eq!(rendered, 1);
        assert_eq!(
            retained, 0,
            "scroll paint should not retain observed line surfaces unless interaction or overlays need them"
        );
        assert!(
            projection
                .scroll_anchor(
                    composition
                        .text_area(&path)
                        .expect("composition should expose the text area model")
                )
                .is_some(),
            "render-only text projections must still provide a scroll anchor"
        );
    }

    #[test]
    fn horizontal_projection_scroll_reuses_covered_text_projection() {
        let area_model = text::Area::new(long_text_area_buffer()).no_wrap();
        let (composition, mut text_engine, path) = text_area_composition_for(area_model);
        let mut projections = Driver::resolve(
            &composition,
            &HashMap::new(),
            &mut text_engine,
            Instant::now(),
        );
        let offset = point::logical(2.0, 0.0);

        assert!(
            projections
                .metrics(&path)
                .expect("metrics should exist")
                .max_offset()
                .x()
                > offset.x()
        );
        assert!(projections.text_area(&path).is_some());
        assert!(projections.queue_offset(&path, offset));

        assert!(
            projections.text_area(&path).is_some(),
            "horizontal text scroll should translate when retained line surfaces still cover the viewport"
        );
    }

    #[test]
    fn text_area_smooth_wheel_retains_target_outside_projection_coverage() {
        let (composition, mut text_engine, path) =
            text_area_composition_for(text::Area::new(multiline_buffer(200)));
        let now = Instant::now();
        let mut projections = Driver::resolve(&composition, &HashMap::new(), &mut text_engine, now);
        let metrics = projections
            .metrics(&path)
            .expect("text area should have scroll metrics");
        let target = metrics.max_offset();

        assert!(projections.queue_wheel_offset_at(&path, target, now));
        assert!(projections.has_active_smooth_wheel_scrolls());
        assert_eq!(
            projections
                .target_offset(&path)
                .expect("smooth wheel target should survive for the frame loop"),
            target
        );
        assert_eq!(
            projections
                .metrics(&path)
                .expect("visual metrics should not jump at input time")
                .offset(),
            metrics.offset()
        );
    }

    #[test]
    fn text_area_thumb_size_is_stable_after_scroll_sync() {
        let (composition, mut text_engine, path) = text_area_composition();
        let mut projections = Driver::default();
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
        let mut projections = Driver::default();
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
        let mut projections = Driver::default();

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

impl Driver {
    fn take_pending_diagnostics(&mut self) -> Diagnostics {
        let diagnostics = self.pending_diagnostics;
        self.pending_diagnostics = Diagnostics::default();
        diagnostics
    }
}
