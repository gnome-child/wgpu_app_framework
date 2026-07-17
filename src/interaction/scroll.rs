use std::{cmp::Ordering, collections::HashMap};

use super::{ScrollbarAxis, Target};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Scroll {
    offsets: Vec<ScrollEntry>,
    sessions: HashMap<Target, ScrollSession>,
    reveal_requests: Vec<Reveal>,
    next_revision: u64,
    revisions: HashMap<Target, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScrollEntry {
    target: Target,
    horizontal: AxisAdjustment,
    vertical: AxisAdjustment,
    resident_accepted: ScrollOffset,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ScrollSession {
    active_source: Option<ScrollSource>,
    active_unit: Option<ScrollUnit>,
    last_timestamp: Option<std::time::Instant>,
    last_update: Option<(std::time::Instant, ScrollDelta)>,
    velocity: ScrollDelta,
    kinetic_velocity: Option<ScrollDelta>,
    edge_behavior: EdgeBehavior,
    elastic_displacement: ScrollDelta,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Coordinate {
    whole: i64,
    fraction: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AxisConfiguration {
    lower: Coordinate,
    upper: Coordinate,
    page: Coordinate,
    step: Coordinate,
    page_increment: Coordinate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AxisAdjustment {
    configuration: AxisConfiguration,
    value: Coordinate,
    revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Reveal {
    Viewport(Target),
    ActiveDescendant { viewport: Target },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ScrollOffset {
    x: Coordinate,
    y: Coordinate,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ScrollDelta {
    x: f64,
    y: f64,
    sample: Option<ScrollSample>,
}

impl Eq for ScrollDelta {}

impl PartialEq for ScrollDelta {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

#[derive(Debug, Clone, Copy)]
struct ScrollSample {
    source: ScrollSource,
    unit: ScrollUnit,
    timestamp: std::time::Instant,
    phase: ScrollPhase,
    velocity: [f64; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScrollEvent {
    source: ScrollSource,
    unit: ScrollUnit,
    timestamp: std::time::Instant,
    phase: ScrollPhase,
    delta: ScrollDelta,
    velocity: ScrollDelta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum ScrollSource {
    Wheel,
    Touchpad,
    Touchscreen,
    Scrollbar,
    Keyboard,
    Reveal,
    Programmatic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScrollUnit {
    Line,
    Pixel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum ScrollPhase {
    Begin,
    Update,
    End,
    Cancel,
    Deceleration,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScrollOutcome {
    applied: ScrollDelta,
    remaining: ScrollDelta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScrollSessionDisposition {
    Ignored,
    Tracked,
    Apply(ScrollDelta),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum EdgeBehavior {
    #[default]
    Clamped,
    Elastic {
        resistance_millis: u16,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScrollUpdate {
    Relative(ScrollDelta),
    Absolute(ScrollOffset),
    Geometry(ScrollOffset),
}

impl Scroll {
    pub(super) fn handle_session_event(
        &mut self,
        target: &Target,
        event: ScrollEvent,
    ) -> ScrollSessionDisposition {
        self.sessions
            .entry(target.clone())
            .or_default()
            .handle(event)
    }

    pub(super) fn resolve_edge(
        &mut self,
        target: &Target,
        outcome: ScrollOutcome,
    ) -> ScrollOutcome {
        self.sessions
            .entry(target.clone())
            .or_default()
            .resolve_edge(outcome)
    }

    pub(crate) fn revision(&self, target: &Target) -> u64 {
        self.revisions.get(target).copied().unwrap_or_default()
    }

    pub(crate) fn kinetic_velocity(&self, target: &Target) -> Option<ScrollDelta> {
        self.sessions
            .get(target)
            .and_then(|session| session.kinetic_velocity)
    }

    pub(crate) fn resident_offset(&self, target: &Target) -> ScrollOffset {
        self.offsets
            .iter()
            .find(|entry| &entry.target == target)
            .map(|entry| entry.resident_accepted)
            .unwrap_or_default()
    }

    #[cfg(test)]
    pub(crate) fn offset(&self, target: &Target) -> ScrollOffset {
        self.resident_offset(target)
    }

    pub(crate) fn desired_offset(&self, target: &Target) -> ScrollOffset {
        self.offsets
            .iter()
            .find(|entry| &entry.target == target)
            .map(ScrollEntry::desired)
            .unwrap_or_default()
    }

    pub(super) fn configure(
        &mut self,
        target: Target,
        maximum: ScrollOffset,
        page: ScrollOffset,
    ) -> Option<ScrollOffset> {
        let index = self.entry_index_or_insert(target.clone());
        let before = self.offsets[index].desired();
        let horizontal_changed = self.offsets[index]
            .horizontal
            .configure(maximum.x(), page.x());
        let vertical_changed = self.offsets[index]
            .vertical
            .configure(maximum.y(), page.y());
        let desired = self.offsets[index].desired();
        if horizontal_changed || vertical_changed {
            self.mark_changed(target);
        }
        (desired != before).then_some(desired)
    }

    pub(crate) fn should_reveal(&self, target: &Target) -> bool {
        self.reveal_requests
            .iter()
            .any(|request| matches!(request, Reveal::Viewport(viewport) if viewport == target))
    }

    pub(crate) fn active_descendant_targets(&self) -> Vec<Target> {
        self.reveal_requests
            .iter()
            .filter_map(|request| match request {
                Reveal::ActiveDescendant { viewport } => Some(viewport.clone()),
                Reveal::Viewport(_) => None,
            })
            .collect()
    }

    pub(super) fn request(&mut self, target: Target, update: ScrollUpdate) -> Option<ScrollOffset> {
        let index = self.entry_index_or_insert(target.clone());
        let before = self.offsets[index].desired();
        match update {
            ScrollUpdate::Relative(delta) => {
                self.offsets[index].horizontal.update(delta.x());
                self.offsets[index].vertical.update(delta.y());
            }
            ScrollUpdate::Absolute(offset) | ScrollUpdate::Geometry(offset) => {
                self.offsets[index].horizontal.set(offset.x);
                self.offsets[index].vertical.set(offset.y);
            }
        }
        let desired = self.offsets[index].desired();
        if before == desired {
            return None;
        }
        self.mark_changed(target);
        Some(desired)
    }

    pub(super) fn accept_resident(
        &mut self,
        target: Target,
        resident_accepted: ScrollOffset,
    ) -> Option<ScrollOffset> {
        let before = self.resident_offset(&target);
        if before == resident_accepted {
            return None;
        }

        let index = self.entry_index_or_insert(target.clone());
        self.offsets[index].resident_accepted = resident_accepted;
        self.mark_changed(target);
        Some(resident_accepted)
    }

    pub(super) fn project_desired(&mut self) {
        let mut changed = Vec::new();
        for entry in &mut self.offsets {
            let desired = entry.desired();
            if entry.resident_accepted != desired {
                entry.resident_accepted = desired;
                changed.push(entry.target.clone());
            }
        }
        for target in changed {
            self.mark_changed(target);
        }
    }

    pub(super) fn reveal(&mut self, target: Target) -> bool {
        let request = Reveal::Viewport(target);
        if self.reveal_requests.contains(&request) {
            return false;
        }

        self.reveal_requests.push(request);
        true
    }

    pub(super) fn reveal_active_descendant(&mut self, viewport: Target) -> bool {
        let request = Reveal::ActiveDescendant { viewport };
        if self.reveal_requests.contains(&request) {
            return false;
        }

        self.reveal_requests.push(request);
        true
    }

    pub(super) fn clear_reveal(&mut self, target: &Target) -> bool {
        let Some(index) = self
            .reveal_requests
            .iter()
            .position(|request| request.viewport() == target)
        else {
            return false;
        };

        self.reveal_requests.remove(index);
        true
    }

    pub(super) fn prune_removed(
        &mut self,
        removed_nodes: &[super::super::composition::tree::NodeId],
        removed_elements: &[super::Id],
    ) -> bool {
        let before_offsets = self.offsets.len();
        let before_sessions = self.sessions.len();
        let before_reveals = self.reveal_requests.len();
        let before_revisions = self.revisions.len();
        self.offsets.retain(|entry| {
            !entry
                .target
                .matches_removed_identity(removed_nodes, removed_elements, &[])
        });
        self.sessions.retain(|target, _| {
            !target.matches_removed_identity(removed_nodes, removed_elements, &[])
        });
        self.reveal_requests.retain(|request| {
            !request
                .viewport()
                .matches_removed_identity(removed_nodes, removed_elements, &[])
        });
        self.revisions.retain(|target, _| {
            !target.matches_removed_identity(removed_nodes, removed_elements, &[])
        });
        before_offsets != self.offsets.len()
            || before_sessions != self.sessions.len()
            || before_reveals != self.reveal_requests.len()
            || before_revisions != self.revisions.len()
    }

    fn mark_changed(&mut self, target: Target) {
        self.next_revision = self.next_revision.saturating_add(1);
        self.revisions.insert(target, self.next_revision);
    }

    fn entry_index_or_insert(&mut self, target: Target) -> usize {
        if let Some(index) = self.offsets.iter().position(|entry| entry.target == target) {
            return index;
        }
        self.offsets.push(ScrollEntry {
            target,
            horizontal: AxisAdjustment::unconfigured(),
            vertical: AxisAdjustment::unconfigured(),
            resident_accepted: ScrollOffset::default(),
        });
        self.offsets.len() - 1
    }
}

impl ScrollEntry {
    fn desired(&self) -> ScrollOffset {
        ScrollOffset {
            x: self.horizontal.value,
            y: self.vertical.value,
        }
    }
}

impl AxisAdjustment {
    fn unconfigured() -> Self {
        Self {
            configuration: AxisConfiguration {
                lower: Coordinate::MIN,
                upper: Coordinate::MAX,
                page: Coordinate::ZERO,
                step: Coordinate::ONE,
                page_increment: Coordinate::ONE,
            },
            value: Coordinate::ZERO,
            revision: 0,
        }
    }

    fn maximum(self) -> Coordinate {
        self.configuration
            .upper
            .saturating_sub(self.configuration.page)
            .max(self.configuration.lower)
    }

    fn configure(&mut self, maximum: i32, page: i32) -> bool {
        let page = Coordinate::from_i64(i64::from(page.max(0)));
        let maximum = Coordinate::from_i64(i64::from(maximum.max(0)));
        let configuration = AxisConfiguration {
            lower: Coordinate::ZERO,
            upper: maximum.saturating_add(page),
            page,
            step: Coordinate::ONE,
            page_increment: page.max(Coordinate::ONE),
        };
        let before_configuration = self.configuration;
        let before_value = self.value;
        self.configuration = configuration;
        self.value = self.value.clamp(configuration.lower, self.maximum());
        let changed = self.configuration != before_configuration || self.value != before_value;
        if changed {
            self.revision = self.revision.saturating_add(1);
        }
        changed
    }

    fn set(&mut self, value: Coordinate) {
        let value = value.clamp(self.configuration.lower, self.maximum());
        if value != self.value {
            self.value = value;
            self.revision = self.revision.saturating_add(1);
        }
    }

    fn update(&mut self, delta: f64) {
        self.set(self.value.add_delta(delta));
    }
}

impl ScrollSession {
    fn handle(&mut self, event: ScrollEvent) -> ScrollSessionDisposition {
        if self
            .last_timestamp
            .is_some_and(|timestamp| event.timestamp < timestamp)
        {
            return ScrollSessionDisposition::Ignored;
        }
        self.last_timestamp = Some(event.timestamp);
        self.active_unit = Some(event.unit);

        match event.phase {
            ScrollPhase::Begin => {
                self.interrupt(event.source, event.timestamp);
                self.active_unit = Some(event.unit);
                if !event.delta.is_zero() {
                    self.observe_update(event);
                }
                disposition_for(event.delta)
            }
            ScrollPhase::Update => {
                if self.active_source != Some(event.source) {
                    self.interrupt(event.source, event.timestamp);
                    self.active_unit = Some(event.unit);
                } else {
                    self.kinetic_velocity = None;
                }
                self.observe_update(event);
                disposition_for(event.delta)
            }
            ScrollPhase::End => {
                if self
                    .active_source
                    .is_some_and(|source| source != event.source)
                {
                    return ScrollSessionDisposition::Ignored;
                }
                if !event.delta.is_zero() {
                    self.observe_update(event);
                }
                self.active_source = None;
                let terminal = if event.velocity.is_zero() {
                    self.velocity
                } else {
                    event.velocity
                };
                self.kinetic_velocity = (!terminal.is_zero()).then_some(terminal);
                self.last_update = None;
                disposition_for(event.delta)
            }
            ScrollPhase::Cancel => {
                if self
                    .active_source
                    .is_some_and(|source| source != event.source)
                {
                    return ScrollSessionDisposition::Ignored;
                }
                self.active_source = None;
                self.last_update = None;
                self.velocity = ScrollDelta::default();
                self.kinetic_velocity = None;
                self.elastic_displacement = ScrollDelta::default();
                ScrollSessionDisposition::Tracked
            }
            ScrollPhase::Deceleration => {
                if self.kinetic_velocity.is_none() {
                    return ScrollSessionDisposition::Ignored;
                }
                if !event.velocity.is_zero() {
                    self.kinetic_velocity = Some(event.velocity);
                } else if event.delta.is_zero() {
                    self.kinetic_velocity = None;
                }
                disposition_for(event.delta)
            }
        }
    }

    fn interrupt(&mut self, source: ScrollSource, timestamp: std::time::Instant) {
        self.active_source = Some(source);
        self.last_timestamp = Some(timestamp);
        self.last_update = None;
        self.velocity = ScrollDelta::default();
        self.kinetic_velocity = None;
        self.elastic_displacement = ScrollDelta::default();
    }

    fn observe_update(&mut self, event: ScrollEvent) {
        self.velocity = if !event.velocity.is_zero() {
            event.velocity
        } else if let Some((timestamp, _)) = self.last_update {
            let seconds = event
                .timestamp
                .saturating_duration_since(timestamp)
                .as_secs_f64();
            if seconds > 0.0 {
                event.delta.scaled(1.0 / seconds)
            } else {
                self.velocity
            }
        } else {
            self.velocity
        };
        self.last_update = Some((event.timestamp, event.delta));
    }

    fn resolve_edge(&mut self, outcome: ScrollOutcome) -> ScrollOutcome {
        let EdgeBehavior::Elastic { resistance_millis } = self.edge_behavior else {
            return outcome;
        };
        if outcome.remaining.is_zero() {
            return outcome;
        }
        let resistance = f64::from(resistance_millis.clamp(1, 1_000)) / 1_000.0;
        self.elastic_displacement = self
            .elastic_displacement
            .plus(outcome.remaining.scaled(resistance));
        ScrollOutcome {
            applied: outcome.applied.plus(outcome.remaining),
            remaining: ScrollDelta::default(),
        }
    }
}

fn disposition_for(delta: ScrollDelta) -> ScrollSessionDisposition {
    if delta.is_zero() {
        ScrollSessionDisposition::Tracked
    } else {
        ScrollSessionDisposition::Apply(delta)
    }
}

impl Coordinate {
    const FRACTION_BITS: u32 = 32;
    const SCALE: i128 = 1_i128 << Self::FRACTION_BITS;
    const INTEGRAL_SNAP_TICKS: i128 = 8;
    const ZERO: Self = Self {
        whole: 0,
        fraction: 0,
    };
    const ONE: Self = Self {
        whole: 1,
        fraction: 0,
    };
    const MIN: Self = Self {
        whole: i64::MIN,
        fraction: 0,
    };
    const MAX: Self = Self {
        whole: i64::MAX,
        fraction: u32::MAX,
    };

    fn from_i64(value: i64) -> Self {
        Self {
            whole: value,
            fraction: 0,
        }
    }

    fn ticks(self) -> i128 {
        i128::from(self.whole) * Self::SCALE + i128::from(self.fraction)
    }

    fn from_ticks(ticks: i128) -> Self {
        let minimum = Self::MIN.ticks();
        let maximum = Self::MAX.ticks();
        let ticks = ticks.clamp(minimum, maximum);
        let remainder = ticks.rem_euclid(Self::SCALE);
        let ticks = if remainder <= Self::INTEGRAL_SNAP_TICKS {
            ticks - remainder
        } else if Self::SCALE - remainder <= Self::INTEGRAL_SNAP_TICKS {
            ticks.saturating_add(Self::SCALE - remainder)
        } else {
            ticks
        }
        .clamp(minimum, maximum);
        Self {
            whole: ticks.div_euclid(Self::SCALE) as i64,
            fraction: ticks.rem_euclid(Self::SCALE) as u32,
        }
    }

    fn saturating_add(self, other: Self) -> Self {
        Self::from_ticks(self.ticks().saturating_add(other.ticks()))
    }

    fn saturating_sub(self, other: Self) -> Self {
        Self::from_ticks(self.ticks().saturating_sub(other.ticks()))
    }

    fn add_delta(self, delta: f64) -> Self {
        let delta_ticks = (normalized_scroll_component(delta) * Self::SCALE as f64).round() as i128;
        Self::from_ticks(self.ticks().saturating_add(delta_ticks))
    }

    fn clamp(self, lower: Self, upper: Self) -> Self {
        self.max(lower).min(upper)
    }

    fn trunc_i32(self) -> i32 {
        let whole = if self.whole < 0 && self.fraction != 0 {
            self.whole.saturating_add(1)
        } else {
            self.whole
        };
        whole.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
    }

    #[cfg(test)]
    fn as_f64(self) -> f64 {
        self.whole as f64 + f64::from(self.fraction) / Self::SCALE as f64
    }

    fn difference(self, other: Self) -> f64 {
        (self.ticks() - other.ticks()) as f64 / Self::SCALE as f64
    }
}

impl Reveal {
    fn viewport(&self) -> &Target {
        match self {
            Self::Viewport(viewport) | Self::ActiveDescendant { viewport } => viewport,
        }
    }
}

impl ScrollOffset {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x: Coordinate::from_i64(i64::from(x)),
            y: Coordinate::from_i64(i64::from(y)),
        }
    }

    pub fn x(self) -> i32 {
        self.x.trunc_i32()
    }

    pub fn y(self) -> i32 {
        self.y.trunc_i32()
    }

    pub(crate) fn clamped(self, minimum: Self, maximum: Self) -> Self {
        Self {
            x: self.x.clamp(minimum.x, maximum.x),
            y: self.y.clamp(minimum.y, maximum.y),
        }
    }

    pub(crate) fn with_x(mut self, x: i32) -> Self {
        self.x = Coordinate::from_i64(i64::from(x));
        self
    }

    pub(crate) fn with_y(mut self, y: i32) -> Self {
        self.y = Coordinate::from_i64(i64::from(y));
        self
    }

    pub(crate) fn with_axis_from(mut self, source: Self, axis: ScrollbarAxis) -> Self {
        match axis {
            ScrollbarAxis::Horizontal => self.x = source.x,
            ScrollbarAxis::Vertical => self.y = source.y,
        }
        self
    }

    pub(crate) fn axis_cmp(self, other: Self, axis: ScrollbarAxis) -> Ordering {
        match axis {
            ScrollbarAxis::Horizontal => self.x.cmp(&other.x),
            ScrollbarAxis::Vertical => self.y.cmp(&other.y),
        }
    }

    pub(crate) fn same_axis(self, other: Self, axis: ScrollbarAxis) -> bool {
        self.axis_cmp(other, axis).is_eq()
    }

    pub(crate) fn componentwise_max(self, other: Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }

    pub(crate) fn componentwise_min(self, other: Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    pub(crate) fn lies_within(self, minimum: Self, maximum: Self) -> bool {
        self.x >= minimum.x && self.x <= maximum.x && self.y >= minimum.y && self.y <= maximum.y
    }

    pub(crate) fn translation_to(self, current: Self) -> [f32; 2] {
        [
            self.x.difference(current.x) as f32,
            self.y.difference(current.y) as f32,
        ]
    }

    pub(crate) fn delta_to(self, current: Self) -> ScrollDelta {
        ScrollDelta::from_logical_pixels(current.x.difference(self.x), current.y.difference(self.y))
    }

    #[cfg(test)]
    fn precise_x(self) -> f64 {
        self.x.as_f64()
    }

    #[cfg(test)]
    fn precise_y(self) -> f64 {
        self.y.as_f64()
    }

    #[cfg(test)]
    pub(crate) fn precise_components_for_test(self) -> [f64; 2] {
        [self.x.as_f64(), self.y.as_f64()]
    }
}

impl ScrollDelta {
    pub fn new(x: i32, y: i32) -> Self {
        Self::from_logical_pixels(f64::from(x), f64::from(y))
    }

    pub fn horizontal(x: i32) -> Self {
        Self::new(x, 0)
    }

    pub fn vertical(y: i32) -> Self {
        Self::new(0, y)
    }

    pub(crate) fn from_logical_pixels(x: f64, y: f64) -> Self {
        Self {
            x: normalized_scroll_component(x),
            y: normalized_scroll_component(y),
            sample: None,
        }
    }

    pub(crate) fn from_physical_pixels(x: f64, y: f64, scale_factor: f64) -> Self {
        let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        Self::from_logical_pixels(x / scale_factor, y / scale_factor)
    }

    pub fn x(self) -> f64 {
        self.x
    }

    pub fn y(self) -> f64 {
        self.y
    }

    pub(crate) fn with_session(
        mut self,
        source: ScrollSource,
        unit: ScrollUnit,
        timestamp: std::time::Instant,
        phase: ScrollPhase,
    ) -> Self {
        self.sample = Some(ScrollSample {
            source,
            unit,
            timestamp,
            phase,
            velocity: [0.0, 0.0],
        });
        self
    }

    pub(crate) fn session_event(self, fallback: ScrollSource) -> ScrollEvent {
        let sample = self.sample.unwrap_or(ScrollSample {
            source: fallback,
            unit: ScrollUnit::Pixel,
            timestamp: std::time::Instant::now(),
            phase: ScrollPhase::Update,
            velocity: [0.0, 0.0],
        });
        ScrollEvent {
            source: sample.source,
            unit: sample.unit,
            timestamp: sample.timestamp,
            phase: sample.phase,
            delta: Self {
                sample: None,
                ..self
            },
            velocity: ScrollDelta::from_logical_pixels(sample.velocity[0], sample.velocity[1]),
        }
    }

    pub(crate) fn is_zero(self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }

    fn plus(self, other: Self) -> Self {
        Self::from_logical_pixels(self.x + other.x, self.y + other.y)
    }

    fn scaled(self, factor: f64) -> Self {
        Self::from_logical_pixels(self.x * factor, self.y * factor)
    }

    fn subtract_applied(self, applied: Self) -> Self {
        let tolerance = 1.0 / Coordinate::SCALE as f64;
        let normalize = |value: f64| if value.abs() <= tolerance { 0.0 } else { value };
        Self::from_logical_pixels(normalize(self.x - applied.x), normalize(self.y - applied.y))
    }
}

impl ScrollEvent {
    pub(crate) fn new(
        source: ScrollSource,
        unit: ScrollUnit,
        timestamp: std::time::Instant,
        phase: ScrollPhase,
        delta: ScrollDelta,
    ) -> Self {
        Self {
            source,
            unit,
            timestamp,
            phase,
            delta: ScrollDelta {
                sample: None,
                ..delta
            },
            velocity: ScrollDelta::default(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn with_velocity(mut self, velocity: ScrollDelta) -> Self {
        self.velocity = ScrollDelta {
            sample: None,
            ..velocity
        };
        self
    }

    pub(crate) fn with_delta(mut self, delta: ScrollDelta) -> Self {
        self.delta = ScrollDelta {
            sample: None,
            ..delta
        };
        self
    }

    pub(crate) fn delta(self) -> ScrollDelta {
        self.delta
    }

    #[allow(dead_code)]
    pub(crate) fn phase(self) -> ScrollPhase {
        self.phase
    }

    #[allow(dead_code)]
    pub(crate) fn source(self) -> ScrollSource {
        self.source
    }

    #[allow(dead_code)]
    pub(crate) fn unit(self) -> ScrollUnit {
        self.unit
    }

    pub(crate) fn timestamp(self) -> std::time::Instant {
        self.timestamp
    }
}

impl ScrollOutcome {
    pub(crate) fn from_offsets(
        input: ScrollDelta,
        before: ScrollOffset,
        after: ScrollOffset,
    ) -> Self {
        let applied = before.delta_to(after);
        Self {
            applied,
            remaining: input.subtract_applied(applied),
        }
    }

    pub(crate) fn unconsumed(input: ScrollDelta) -> Self {
        Self {
            applied: ScrollDelta::default(),
            remaining: input,
        }
    }

    pub(crate) fn then(self, next: Self) -> Self {
        Self {
            applied: self.applied.plus(next.applied),
            remaining: next.remaining,
        }
    }

    pub(crate) fn applied(self) -> ScrollDelta {
        self.applied
    }

    pub(crate) fn remaining(self) -> ScrollDelta {
        self.remaining
    }
}

fn normalized_scroll_component(value: f64) -> f64 {
    if !value.is_finite() || value == 0.0 {
        return 0.0;
    }
    value.clamp(f64::from(i32::MIN), f64::from(i32::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_session_preserves_lifecycle_velocity_deceleration_and_interruption() {
        let mut session = ScrollSession::default();
        let started = std::time::Instant::now();
        let begin = ScrollEvent::new(
            ScrollSource::Touchpad,
            ScrollUnit::Pixel,
            started,
            ScrollPhase::Begin,
            ScrollDelta::from_logical_pixels(0.25, 0.5),
        );
        assert_eq!(
            session.handle(begin),
            ScrollSessionDisposition::Apply(ScrollDelta::from_logical_pixels(0.25, 0.5))
        );

        let updated_at = started + std::time::Duration::from_millis(10);
        let update = ScrollEvent::new(
            ScrollSource::Touchpad,
            ScrollUnit::Pixel,
            updated_at,
            ScrollPhase::Update,
            ScrollDelta::from_logical_pixels(1.25, -0.5),
        );
        assert_eq!(
            session.handle(update),
            ScrollSessionDisposition::Apply(ScrollDelta::from_logical_pixels(1.25, -0.5))
        );

        let ended_at = updated_at + std::time::Duration::from_millis(10);
        let terminal = ScrollDelta::from_logical_pixels(80.0, -25.0);
        assert_eq!(
            session.handle(
                ScrollEvent::new(
                    ScrollSource::Touchpad,
                    ScrollUnit::Pixel,
                    ended_at,
                    ScrollPhase::End,
                    ScrollDelta::default(),
                )
                .with_velocity(terminal),
            ),
            ScrollSessionDisposition::Tracked
        );
        assert_eq!(session.kinetic_velocity, Some(terminal));

        let deceleration = ScrollDelta::from_logical_pixels(0.75, -0.25);
        assert_eq!(
            session.handle(ScrollEvent::new(
                ScrollSource::Touchpad,
                ScrollUnit::Pixel,
                ended_at + std::time::Duration::from_millis(10),
                ScrollPhase::Deceleration,
                deceleration,
            )),
            ScrollSessionDisposition::Apply(deceleration)
        );

        let interrupted_at = ended_at + std::time::Duration::from_millis(20);
        assert_eq!(
            session.handle(ScrollEvent::new(
                ScrollSource::Wheel,
                ScrollUnit::Line,
                interrupted_at,
                ScrollPhase::Begin,
                ScrollDelta::vertical(28),
            )),
            ScrollSessionDisposition::Apply(ScrollDelta::vertical(28))
        );
        assert_eq!(session.active_source, Some(ScrollSource::Wheel));
        assert_eq!(session.active_unit, Some(ScrollUnit::Line));
        assert_eq!(session.kinetic_velocity, None);
        assert_eq!(session.velocity, ScrollDelta::default());

        assert_eq!(
            session.handle(ScrollEvent::new(
                ScrollSource::Wheel,
                ScrollUnit::Line,
                updated_at,
                ScrollPhase::Update,
                ScrollDelta::vertical(1),
            )),
            ScrollSessionDisposition::Ignored,
            "a stale sample must not mutate the active direct-input session"
        );
        assert_eq!(session.active_source, Some(ScrollSource::Wheel));

        assert_eq!(
            session.handle(ScrollEvent::new(
                ScrollSource::Wheel,
                ScrollUnit::Line,
                interrupted_at + std::time::Duration::from_millis(1),
                ScrollPhase::Cancel,
                ScrollDelta::default(),
            )),
            ScrollSessionDisposition::Tracked
        );
        assert_eq!(session.active_source, None);
        assert_eq!(session.kinetic_velocity, None);
    }

    #[test]
    fn scroll_outcome_preserves_exact_independent_axis_remainders() {
        let mut scroll = Scroll::default();
        let target = Target::scroll("outcome.axes", "Outcome Axes");
        scroll.configure(
            target.clone(),
            ScrollOffset::new(10, 100),
            ScrollOffset::new(5, 20),
        );

        let input = ScrollDelta::from_logical_pixels(12.25, 40.5);
        let before = scroll.desired_offset(&target);
        let after = scroll
            .request(target.clone(), ScrollUpdate::Relative(input))
            .expect("diagonal input should change both configured axes");
        let outcome = ScrollOutcome::from_offsets(input, before, after);
        assert_eq!(
            outcome.applied(),
            ScrollDelta::from_logical_pixels(10.0, 40.5)
        );
        assert_eq!(
            outcome.remaining(),
            ScrollDelta::from_logical_pixels(2.25, 0.0)
        );

        let reverse = ScrollDelta::from_logical_pixels(-12.0, -50.0);
        let before = after;
        let after = scroll
            .request(target, ScrollUpdate::Relative(reverse))
            .expect("reverse input should move both axes back to their lower bounds");
        let outcome = ScrollOutcome::from_offsets(reverse, before, after);
        assert_eq!(
            outcome.applied(),
            ScrollDelta::from_logical_pixels(-10.0, -40.5)
        );
        assert_eq!(
            outcome.remaining(),
            ScrollDelta::from_logical_pixels(-2.0, -9.5)
        );
    }

    #[test]
    fn clamped_edges_handoff_remainder_while_elastic_edges_absorb_it_privately() {
        let outcome = ScrollOutcome {
            applied: ScrollDelta::from_logical_pixels(10.0, 20.0),
            remaining: ScrollDelta::from_logical_pixels(2.0, -4.0),
        };
        let mut clamped = ScrollSession::default();
        assert_eq!(clamped.resolve_edge(outcome), outcome);
        assert_eq!(clamped.elastic_displacement, ScrollDelta::default());

        let mut elastic = ScrollSession {
            edge_behavior: EdgeBehavior::Elastic {
                resistance_millis: 250,
            },
            ..ScrollSession::default()
        };
        let resolved = elastic.resolve_edge(outcome);
        assert_eq!(
            resolved.applied(),
            ScrollDelta::from_logical_pixels(12.0, 16.0)
        );
        assert_eq!(resolved.remaining(), ScrollDelta::default());
        assert_eq!(
            elastic.elastic_displacement,
            ScrollDelta::from_logical_pixels(0.5, -1.0)
        );
    }

    #[test]
    fn target_label_does_not_change_scroll_identity() {
        let mut scroll = Scroll::default();
        let first = Target::scroll("same.scroll", "First Label");
        let second = Target::scroll("same.scroll", "Second Label");

        assert_eq!(
            scroll.request(
                first.clone(),
                ScrollUpdate::Relative(ScrollDelta::vertical(42))
            ),
            Some(ScrollOffset::new(0, 42))
        );
        assert_eq!(scroll.offset(&first), ScrollOffset::default());
        assert_eq!(
            scroll.accept_resident(first, ScrollOffset::new(0, 42)),
            Some(ScrollOffset::new(0, 42))
        );
        assert_eq!(scroll.offset(&second), ScrollOffset::new(0, 42));

        assert!(scroll.reveal(second));
        assert!(scroll.should_reveal(&Target::scroll("same.scroll", "Third Label")));
    }

    #[test]
    fn relative_absolute_and_geometry_share_one_request_admission_law() {
        let mut scroll = Scroll::default();
        let target = Target::scroll("shared.scroll", "Shared");

        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Relative(ScrollDelta::new(18, 24)),
            ),
            Some(ScrollOffset::new(18, 24))
        );
        assert_eq!(scroll.offset(&target), ScrollOffset::default());
        assert_eq!(scroll.desired_offset(&target), ScrollOffset::new(18, 24));
        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Absolute(ScrollOffset::new(40, 60)),
            ),
            Some(ScrollOffset::new(40, 60))
        );
        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Geometry(ScrollOffset::new(40, 60)),
            ),
            None
        );
        assert_eq!(scroll.offset(&target), ScrollOffset::default());
        assert_eq!(
            scroll.accept_resident(target.clone(), ScrollOffset::new(40, 60)),
            Some(ScrollOffset::new(40, 60))
        );
        assert_eq!(scroll.offset(&target), ScrollOffset::new(40, 60));
    }

    #[test]
    fn relative_requests_accumulate_against_desired_while_admitted_stays_visible() {
        let mut scroll = Scroll::default();
        let target = Target::scroll("pending.scroll", "Pending");

        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Relative(ScrollDelta::vertical(30)),
            ),
            Some(ScrollOffset::new(0, 30))
        );
        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Relative(ScrollDelta::vertical(40)),
            ),
            Some(ScrollOffset::new(0, 70))
        );

        assert_eq!(scroll.offset(&target), ScrollOffset::default());
        assert_eq!(scroll.desired_offset(&target), ScrollOffset::new(0, 70));
    }

    #[test]
    fn intermediate_admission_preserves_a_farther_desired_offset() {
        let mut scroll = Scroll::default();
        let target = Target::scroll("progress.scroll", "Progress");

        scroll.request(
            target.clone(),
            ScrollUpdate::Absolute(ScrollOffset::new(0, 300)),
        );
        assert_eq!(
            scroll.accept_resident(target.clone(), ScrollOffset::new(0, 120)),
            Some(ScrollOffset::new(0, 120))
        );
        assert_eq!(scroll.offset(&target), ScrollOffset::new(0, 120));
        assert_eq!(scroll.desired_offset(&target), ScrollOffset::new(0, 300));

        assert_eq!(
            scroll.accept_resident(target.clone(), ScrollOffset::new(0, 300)),
            Some(ScrollOffset::new(0, 300))
        );
        assert_eq!(scroll.offset(&target), ScrollOffset::new(0, 300));
        assert_eq!(scroll.desired_offset(&target), ScrollOffset::new(0, 300));
    }

    #[test]
    fn desired_projection_changes_only_the_projection_clone() {
        let mut scroll = Scroll::default();
        let target = Target::scroll("projection.scroll", "Projection");
        scroll.request(
            target.clone(),
            ScrollUpdate::Absolute(ScrollOffset::new(0, 240)),
        );

        let mut projection = scroll.clone();
        projection.project_desired();

        assert_eq!(scroll.offset(&target), ScrollOffset::default());
        assert_eq!(projection.offset(&target), ScrollOffset::new(0, 240));
        assert_eq!(
            projection.desired_offset(&target),
            ScrollOffset::new(0, 240)
        );
    }

    #[test]
    fn source_revisions_advance_only_when_the_named_target_changes() {
        let mut scroll = Scroll::default();
        let first = Target::scroll("revision.first", "First");
        let second = Target::scroll("revision.second", "Second");

        assert_eq!(scroll.revision(&first), 0);
        assert_eq!(scroll.revision(&second), 0);
        scroll.request(
            first.clone(),
            ScrollUpdate::Absolute(ScrollOffset::new(12, 24)),
        );
        let requested = scroll.revision(&first);
        assert!(requested > 0);
        assert_eq!(scroll.revision(&second), 0);

        assert_eq!(
            scroll.request(
                first.clone(),
                ScrollUpdate::Geometry(ScrollOffset::new(12, 24)),
            ),
            None
        );
        assert_eq!(scroll.revision(&first), requested);

        scroll.accept_resident(first.clone(), ScrollOffset::new(12, 24));
        let accepted = scroll.revision(&first);
        assert!(accepted > requested);

        scroll.request(
            first.clone(),
            ScrollUpdate::Relative(ScrollDelta::horizontal(5)),
        );
        let pending = scroll.revision(&first);
        assert!(pending > accepted);
        let mut projection = scroll.clone();
        projection.project_desired();
        assert!(projection.revision(&first) > pending);
        assert_eq!(scroll.revision(&first), pending);
    }

    fn apply_physical_trace(scale: f64, physical_y: &[f64]) -> (Scroll, Target, usize) {
        let mut scroll = Scroll::default();
        let target = Target::scroll("precision.trace", "Precision Trace");
        let mut visual_updates = 0;
        for physical_y in physical_y {
            visual_updates += usize::from(
                scroll
                    .request(
                        target.clone(),
                        ScrollUpdate::Relative(ScrollDelta::from_physical_pixels(
                            0.0,
                            *physical_y,
                            scale,
                        )),
                    )
                    .is_some(),
            );
        }
        (scroll, target, visual_updates)
    }

    fn require_sum_preserved(scale: f64, physical_y: &[f64]) -> (Scroll, Target, usize) {
        let (scroll, target, visual_updates) = apply_physical_trace(scale, physical_y);
        let logical_total = physical_y.iter().sum::<f64>() / scale;
        let desired = scroll.desired_offset(&target);
        assert_eq!(desired.y(), logical_total.trunc() as i32);
        assert!(
            (desired.precise_y() - logical_total).abs()
                <= physical_y.len() as f64 / Coordinate::SCALE as f64,
            "scale={scale} desired={desired:?} logical_total={logical_total}"
        );
        (scroll, target, visual_updates)
    }

    fn require_tiny_trace(scale: f64) {
        let (_, _, visual_updates) = require_sum_preserved(scale, &[0.4; 5]);
        assert_eq!(visual_updates, 5);
    }

    fn require_reversal_trace(scale: f64) {
        let physical = [1.2, 1.2, -0.6, -0.6, -1.2];
        let mut scroll = Scroll::default();
        let target = Target::scroll("precision.reversal", "Precision Reversal");
        let mut maximum = 0;
        for physical_y in physical {
            scroll.request(
                target.clone(),
                ScrollUpdate::Relative(ScrollDelta::from_physical_pixels(0.0, physical_y, scale)),
            );
            maximum = maximum.max(scroll.desired_offset(&target).y());
        }
        let desired = scroll.desired_offset(&target);
        assert!(maximum > 0, "scale={scale} reversal never moved visually");
        assert_eq!(desired, ScrollOffset::default());
        assert_eq!(desired.precise_y(), 0.0);
    }

    fn require_burst_trace(scale: f64) {
        let physical = [0.3, 0.3, 0.3, 4.1, 0.2, 0.2, 0.2, 0.4];
        let (scroll, target, visual_updates) = require_sum_preserved(scale, &physical);
        assert_eq!(visual_updates, physical.len());
        assert_eq!(scroll.revision(&target), visual_updates as u64);
    }

    #[test]
    fn input_precision_case_tiny_scale_100() {
        require_tiny_trace(1.0);
    }

    #[test]
    fn input_precision_case_tiny_scale_125() {
        require_tiny_trace(1.25);
    }

    #[test]
    fn input_precision_case_tiny_scale_150() {
        require_tiny_trace(1.5);
    }

    #[test]
    fn input_precision_case_tiny_scale_175() {
        require_tiny_trace(1.75);
    }

    #[test]
    fn input_precision_case_tiny_scale_200() {
        require_tiny_trace(2.0);
    }

    #[test]
    fn input_precision_case_reversal_scale_100() {
        require_reversal_trace(1.0);
    }

    #[test]
    fn input_precision_case_reversal_scale_125() {
        require_reversal_trace(1.25);
    }

    #[test]
    fn input_precision_case_reversal_scale_150() {
        require_reversal_trace(1.5);
    }

    #[test]
    fn input_precision_case_reversal_scale_175() {
        require_reversal_trace(1.75);
    }

    #[test]
    fn input_precision_case_reversal_scale_200() {
        require_reversal_trace(2.0);
    }

    #[test]
    fn input_precision_case_burst_coalescing_scale_100() {
        require_burst_trace(1.0);
    }

    #[test]
    fn input_precision_case_burst_coalescing_scale_125() {
        require_burst_trace(1.25);
    }

    #[test]
    fn input_precision_case_burst_coalescing_scale_150() {
        require_burst_trace(1.5);
    }

    #[test]
    fn input_precision_case_burst_coalescing_scale_175() {
        require_burst_trace(1.75);
    }

    #[test]
    fn input_precision_case_burst_coalescing_scale_200() {
        require_burst_trace(2.0);
    }

    #[test]
    fn input_precision_case_thumb_absolute_replaces_fractional_position() {
        let mut scroll = Scroll::default();
        let target = Target::scroll("precision.thumb", "Precision Thumb");
        let fractional = scroll
            .request(
                target.clone(),
                ScrollUpdate::Relative(ScrollDelta::from_logical_pixels(0.0, 0.75)),
            )
            .unwrap();
        assert_eq!(fractional.precise_y(), 0.75);
        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Absolute(ScrollOffset::new(0, 40)),
            ),
            Some(ScrollOffset::new(0, 40))
        );
        let desired = scroll
            .request(
                target.clone(),
                ScrollUpdate::Relative(ScrollDelta::from_logical_pixels(0.0, 0.5)),
            )
            .unwrap();
        assert_eq!(desired.precise_y(), 40.5);
        assert_eq!(desired.y(), 40);
    }

    #[test]
    fn input_precision_case_keyboard_integral_delta_preserves_fractional_position() {
        let mut scroll = Scroll::default();
        let target = Target::scroll("precision.keyboard", "Precision Keyboard");
        scroll.request(
            target.clone(),
            ScrollUpdate::Relative(ScrollDelta::from_logical_pixels(0.0, -0.4)),
        );
        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Relative(ScrollDelta::vertical(28)),
            ),
            Some(ScrollOffset {
                x: Coordinate::ZERO,
                y: Coordinate::from_ticks(
                    Coordinate::from_i64(28).ticks()
                        - (0.4 * Coordinate::SCALE as f64).round() as i128,
                ),
            })
        );
        assert!((scroll.desired_offset(&target).precise_y() - 27.6).abs() < 1.0e-9);
        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Relative(ScrollDelta::from_logical_pixels(0.0, 1.4)),
            ),
            Some(ScrollOffset::new(0, 29))
        );
    }

    #[test]
    fn input_precision_case_reveal_geometry_replaces_fractional_position() {
        let mut scroll = Scroll::default();
        let target = Target::scroll("precision.reveal", "Precision Reveal");
        scroll.request(
            target.clone(),
            ScrollUpdate::Relative(ScrollDelta::from_logical_pixels(0.0, 0.75)),
        );
        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Geometry(ScrollOffset::new(0, 72)),
            ),
            Some(ScrollOffset::new(0, 72))
        );
        assert_eq!(scroll.desired_offset(&target).precise_y(), 72.0);
    }

    #[test]
    fn input_precision_case_programmatic_absolute_is_exact_after_reverse_fraction() {
        let mut scroll = Scroll::default();
        let target = Target::scroll("precision.programmatic", "Precision Programmatic");
        scroll.request(
            target.clone(),
            ScrollUpdate::Relative(ScrollDelta::from_logical_pixels(0.0, -0.75)),
        );
        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Absolute(ScrollOffset::new(36, 84)),
            ),
            Some(ScrollOffset::new(36, 84))
        );
        assert_eq!(scroll.desired_offset(&target), ScrollOffset::new(36, 84));
        assert_eq!(scroll.desired_offset(&target).precise_x(), 36.0);
        assert_eq!(scroll.desired_offset(&target).precise_y(), 84.0);
    }

    #[test]
    fn configured_adjustments_clamp_continuous_axes_and_observe_atomic_geometry() {
        let mut scroll = Scroll::default();
        let target = Target::scroll("adjustment.continuous", "Continuous Adjustment");

        assert_eq!(
            scroll.configure(
                target.clone(),
                ScrollOffset::new(100, 200),
                ScrollOffset::new(20, 40),
            ),
            None
        );
        let configured_target_revision = scroll.revision(&target);
        assert!(configured_target_revision > 0);
        let entry = scroll
            .offsets
            .iter()
            .find(|entry| entry.target == target)
            .unwrap();
        assert_eq!(entry.horizontal.configuration.lower, Coordinate::ZERO);
        assert_eq!(entry.horizontal.maximum(), Coordinate::from_i64(100));
        assert_eq!(
            entry.horizontal.configuration.page,
            Coordinate::from_i64(20)
        );
        assert_eq!(entry.horizontal.configuration.step, Coordinate::ONE);
        assert_eq!(
            entry.horizontal.configuration.page_increment,
            Coordinate::from_i64(20)
        );
        let configured_revision = entry.horizontal.revision;

        let desired = scroll
            .request(
                target.clone(),
                ScrollUpdate::Relative(ScrollDelta::from_logical_pixels(0.25, 0.75)),
            )
            .unwrap();
        assert_eq!(desired.precise_x(), 0.25);
        assert_eq!(desired.precise_y(), 0.75);
        assert_eq!(desired.x(), 0);
        assert_eq!(desired.y(), 0);

        let clamped = scroll
            .request(
                target.clone(),
                ScrollUpdate::Relative(ScrollDelta::from_logical_pixels(150.0, 250.0)),
            )
            .unwrap();
        assert_eq!(clamped, ScrollOffset::new(100, 200));

        scroll.configure(
            target.clone(),
            ScrollOffset::new(100, 200),
            ScrollOffset::new(20, 40),
        );
        assert_eq!(
            scroll.revision(&target),
            configured_target_revision + 2,
            "only the two value updates since configuration should advance observation"
        );
        let unchanged = scroll
            .offsets
            .iter()
            .find(|entry| entry.target == target)
            .unwrap();
        assert_eq!(unchanged.horizontal.revision, configured_revision + 2);

        assert_eq!(
            scroll.configure(
                target.clone(),
                ScrollOffset::new(50, 80),
                ScrollOffset::new(10, 16),
            ),
            Some(ScrollOffset::new(50, 80))
        );
        let reconfigured = scroll
            .offsets
            .iter()
            .find(|entry| entry.target == target)
            .unwrap();
        assert_eq!(reconfigured.horizontal.maximum(), Coordinate::from_i64(50));
        assert_eq!(
            reconfigured.horizontal.configuration.page,
            Coordinate::from_i64(10)
        );
        assert_eq!(reconfigured.horizontal.revision, configured_revision + 3);
        assert_eq!(
            scroll.revision(&target),
            configured_target_revision + 3,
            "a clamp-producing atomic reconfiguration advances one target observation"
        );
    }

    #[test]
    fn wide_coordinates_rebase_before_renderer_float_projection() {
        let baseline = ScrollOffset {
            x: Coordinate::from_ticks(
                Coordinate::from_i64(20_000_000).ticks() + Coordinate::SCALE / 4,
            ),
            y: Coordinate::from_ticks(
                Coordinate::from_i64(30_000_000).ticks() + Coordinate::SCALE / 2,
            ),
        };
        let current = ScrollOffset {
            x: Coordinate::from_ticks(baseline.x.ticks() + Coordinate::SCALE / 2),
            y: Coordinate::from_ticks(baseline.y.ticks() - Coordinate::SCALE / 4),
        };

        assert_eq!(baseline.translation_to(current), [-0.5, 0.25]);
        assert_eq!(
            baseline.precise_x() as f32 - current.precise_x() as f32,
            0.0,
            "a global f32 conversion would lose the local half-pixel displacement"
        );
        assert_eq!(
            baseline.precise_y() as f32 - current.precise_y() as f32,
            0.0,
            "a global f32 conversion would lose the local quarter-pixel displacement"
        );
    }
}
