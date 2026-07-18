use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    ops::{Deref, Range},
    rc::Rc,
};

use crate::{interaction, scene, view, widget::Widget};

mod variable;

const DEFAULT_OVERSCAN: usize = 2;
const INITIAL_ROWS: usize = 32;
const MAX_TRANSITION_MATERIALIZED_ROWS: usize = 80;
const MAX_LEADING_RUNWAY_VIEWPORTS: usize = 2;
const MAX_RECYCLED_SLOTS: usize = 32;

/// Stable logical identity supplied by a list model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Key(u64);

/// Process-local identity for one recycled list presentation slot.
///
/// Item identity remains [`Key`] and position remains an index. A slot may be
/// rebound to several items during its lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Slot(u64);

/// One observable membership mutation between model revisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Insert {
        index: usize,
        count: usize,
    },
    Remove {
        index: usize,
        count: usize,
    },
    Replace {
        index: usize,
        removed: usize,
        added: usize,
    },
    Move {
        from: usize,
        to: usize,
        count: usize,
    },
}

/// Observable membership and stable identity for a list.
pub trait Model {
    fn len(&self) -> usize;
    fn key(&self, index: usize) -> Key;
    fn index_of(&self, key: Key) -> Option<usize>;

    /// Monotonic membership revision for insert/remove/replace/move events.
    fn membership_revision(&self) -> u64;

    /// Ordered mutations after `revision`. A changed membership revision must
    /// return at least one event that transforms the old length into the new.
    fn changes_since(&self, revision: u64) -> Vec<Change>;

    /// Revision of the content currently associated with one stable key.
    fn item_revision(&self, index: usize) -> u64;

    /// Monotonic revision covering every key, order, and item value consulted
    /// by [`Factory::bind`] during residency-only presentation.
    ///
    /// Returning `None` keeps the model correct but disables the keyed
    /// residency fast path. A model may return `Some(revision)` only when an
    /// unchanged value proves that retained overlap cannot need rebinding.
    fn residency_revision(&self) -> Option<u64> {
        None
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Setup, binding, and teardown for recycled list presentation slots.
pub trait Factory {
    /// Compatibility revision for slot-local state and binding semantics.
    fn revision(&self) -> u64;

    /// Allocates slot-local listeners or resources before the first bind.
    fn setup(&self, _slot: Slot) {}

    /// Projects one item into an already-setup presentation slot.
    fn bind(&self, slot: Slot, index: usize) -> view::Node;

    /// Releases every item-specific listener or resource installed by bind.
    fn unbind(&self, _slot: Slot, _key: Key, _index: usize) {}

    /// Releases slot-local state after its final unbind.
    fn teardown(&self, _slot: Slot) {}
}

/// A native list that materializes only the visible page, runway, and pins.
pub struct List {
    state: State,
    width: Option<view::Dimension>,
    height: Option<view::Dimension>,
    max_height: Option<i32>,
    background: Option<scene::Brush>,
    configuration: Option<crate::scroll::Configuration>,
}

#[derive(Clone)]
pub(crate) struct State {
    id: interaction::Id,
    heights: Heights,
    overscan: usize,
    model: Rc<dyn Model>,
    factory: Rc<dyn Factory>,
    selectable: bool,
    prepared_runway: Option<Range<usize>>,
    slots: Rc<RefCell<Slots>>,
}

#[derive(Clone)]
enum Heights {
    Uniform(i32),
    Variable(Measurements),
}

/// Retained keyed block-size geometry for one variable virtual sequence.
///
/// This is deliberately independent from [`Materialization`]: measurements
/// own item extents and offsets, while materialization owns which keys exist
/// in the current view.
#[derive(Clone)]
pub(crate) struct Measurements(Rc<RefCell<variable::Region>>);

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Materialization {
    range: Range<usize>,
    pins: Vec<Key>,
    runway: bool,
}

/// Owner-local work performed while reconciling one or more virtual sequences.
///
/// These are observation currencies, not invalidation inputs. They deliberately
/// describe keyed set work at the point where both the old and desired
/// memberships are available.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct MaterializationStats {
    pub(crate) lists: usize,
    pub(crate) old_interval_start: Option<usize>,
    pub(crate) old_interval_end: Option<usize>,
    pub(crate) new_interval_start: Option<usize>,
    pub(crate) new_interval_end: Option<usize>,
    pub(crate) resident_rows_before: usize,
    pub(crate) resident_rows_after: usize,
    pub(crate) entering_rows: usize,
    pub(crate) departing_rows: usize,
    pub(crate) overlapping_rows: usize,
    pub(crate) revised_rows: usize,
    pub(crate) moved_rows: usize,
    pub(crate) membership_changes: usize,
    pub(crate) membership_revision_max: u64,
    pub(crate) provider_binds: usize,
    pub(crate) slots_rebound: usize,
    pub(crate) view_nodes_cloned: usize,
}

/// Ordered edits produced by the virtual-list owner for a residency-only
/// presentation.
///
/// The retained middle is intentionally absent: downstream owners must keep
/// it in place rather than receiving, cloning, or revalidating it.
pub(crate) struct ResidencyDelta {
    list: interaction::Id,
    remove_front: usize,
    remove_back: usize,
    insert_front: Vec<view::Node>,
    insert_back: Vec<view::Node>,
    reset: Option<Vec<view::Node>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AppliedResidencyDelta {
    list: interaction::Id,
    remove_front: usize,
    remove_back: usize,
    insert_front: usize,
    insert_back: usize,
    reset: bool,
}

pub(crate) struct ResidencyDeltaParts {
    pub(crate) list: interaction::Id,
    pub(crate) remove_front: usize,
    pub(crate) remove_back: usize,
    pub(crate) insert_front: Vec<view::Node>,
    pub(crate) insert_back: Vec<view::Node>,
    pub(crate) reset: Option<Vec<view::Node>>,
}

#[derive(Clone)]
pub(crate) struct Request {
    id: interaction::Id,
    range: Range<usize>,
    limit: usize,
    measurements: Option<Measurements>,
}

#[derive(Default)]
struct Slots {
    next: u64,
    membership_revision: Option<u64>,
    factory_revision: Option<u64>,
    len: usize,
    active: HashMap<Key, BoundSlot>,
    order: VecDeque<Key>,
    range: Option<Range<usize>>,
    pins: Vec<Key>,
    residency_revision: Option<u64>,
    recycled: Vec<AvailableSlot>,
}

struct AvailableSlot {
    id: Slot,
    setup_factory: Rc<dyn Factory>,
}

struct BoundSlot {
    slot: AvailableSlot,
    key: Key,
    index: usize,
    revision: u64,
    node: view::Node,
    binding_factory: Rc<dyn Factory>,
}

#[derive(Debug, Clone, Copy)]
struct DesiredItem {
    key: Key,
    index: usize,
    revision: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Backward,
    Forward,
}

impl Measurements {
    fn new(estimate: i32) -> Self {
        Self(Rc::new(RefCell::new(variable::Region::new(estimate))))
    }
}

impl Deref for Measurements {
    type Target = RefCell<variable::Region>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for Measurements {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Measurements {}

impl Key {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

impl Slot {
    pub const fn value(self) -> u64 {
        self.0
    }

    #[cfg(test)]
    pub(crate) const fn from_test_value(value: u64) -> Self {
        Self(value)
    }
}

impl Change {
    fn next_len(self, len: usize) -> usize {
        match self {
            Self::Insert { index, count } => {
                assert!(
                    index <= len,
                    "list insertion index exceeds the prior length"
                );
                len.saturating_add(count)
            }
            Self::Remove { index, count } => {
                assert!(
                    index.saturating_add(count) <= len,
                    "list removal range exceeds the prior length"
                );
                len - count
            }
            Self::Replace {
                index,
                removed,
                added,
            } => {
                assert!(
                    index.saturating_add(removed) <= len,
                    "list replacement range exceeds the prior length"
                );
                len.saturating_sub(removed).saturating_add(added)
            }
            Self::Move { from, to, count } => {
                assert!(
                    from.saturating_add(count) <= len && to <= len.saturating_sub(count),
                    "list movement range exceeds the prior length"
                );
                len
            }
        }
    }
}

impl From<u64> for Key {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl List {
    pub fn new<M, F>(id: impl Into<interaction::Id>, row_height: i32, model: M, factory: F) -> Self
    where
        M: Model + 'static,
        F: Factory + 'static,
    {
        Self {
            state: State::new(id.into(), row_height, Rc::new(model), Rc::new(factory)),
            width: None,
            height: None,
            max_height: None,
            background: None,
            configuration: None,
        }
    }

    /// Creates a list whose materialized items determine their block extents.
    pub fn variable<M, F>(
        id: impl Into<interaction::Id>,
        estimated_row_height: i32,
        model: M,
        factory: F,
    ) -> Self
    where
        M: Model + 'static,
        F: Factory + 'static,
    {
        Self {
            state: State::variable(
                id.into(),
                estimated_row_height,
                Rc::new(model),
                Rc::new(factory),
            ),
            width: None,
            height: None,
            max_height: None,
            background: None,
            configuration: None,
        }
    }

    pub fn overscan(mut self, rows: usize) -> Self {
        self.state.overscan = rows.min(32);
        self
    }

    pub fn configuration(mut self, configuration: crate::scroll::Configuration) -> Self {
        self.configuration = Some(configuration);
        self
    }

    pub fn width(mut self, width: view::Dimension) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: view::Dimension) -> Self {
        self.height = Some(height);
        self
    }

    pub fn max_height(mut self, height: i32) -> Self {
        self.max_height = Some(height.max(0));
        self
    }

    pub fn background(mut self, background: scene::Brush) -> Self {
        self.background = Some(background);
        self
    }

    pub fn selectable(mut self) -> Self {
        self.state.selectable = true;
        self
    }
}

impl Widget for List {
    fn into_node(self) -> view::Node {
        let mut style = view::Style::new();
        if let Some(width) = self.width {
            style = style.with_width(width);
        }
        if let Some(height) = self.height {
            style = style.with_height(height);
        }
        if let Some(max_height) = self.max_height {
            style = style.with_max_height(max_height);
        }
        if let Some(background) = self.background {
            style = style.with_background(background);
        }

        let node = view::Node::virtual_list(self.state).with_style(style);
        match self.configuration {
            Some(configuration) => node.with_scroll_configuration(configuration),
            None => node,
        }
    }
}

impl State {
    pub(crate) fn same_scene_state(&self, other: &Self) -> bool {
        self.id == other.id
            && self.row_height() == other.row_height()
            && self.overscan == other.overscan
            && self.selectable == other.selectable
            && self.len() == other.len()
    }

    fn new(
        id: interaction::Id,
        row_height: i32,
        model: Rc<dyn Model>,
        factory: Rc<dyn Factory>,
    ) -> Self {
        Self {
            id,
            heights: Heights::Uniform(row_height.max(1)),
            overscan: DEFAULT_OVERSCAN,
            model,
            factory,
            selectable: false,
            prepared_runway: None,
            slots: Rc::new(RefCell::new(Slots::default())),
        }
    }

    fn variable(
        id: interaction::Id,
        estimate: i32,
        model: Rc<dyn Model>,
        factory: Rc<dyn Factory>,
    ) -> Self {
        Self {
            id,
            heights: Heights::Variable(Measurements::new(estimate)),
            overscan: DEFAULT_OVERSCAN,
            model,
            factory,
            selectable: false,
            prepared_runway: None,
            slots: Rc::new(RefCell::new(Slots::default())),
        }
    }

    pub(crate) fn reuse_slots_from(&mut self, previous: &Self) {
        if self.id == previous.id {
            self.slots = Rc::clone(&previous.slots);
        }
    }

    pub(crate) fn id(&self) -> interaction::Id {
        self.id
    }

    pub(crate) fn len(&self) -> usize {
        self.model.len()
    }

    pub(crate) fn row_height(&self) -> i32 {
        match &self.heights {
            Heights::Uniform(height) => *height,
            Heights::Variable(region) => region.borrow().estimate(),
        }
    }

    pub(crate) fn measurements(&self) -> Option<Measurements> {
        match &self.heights {
            Heights::Uniform(_) => None,
            Heights::Variable(measurements) => Some(measurements.clone()),
        }
    }

    pub(crate) fn request_for_viewport(&self, offset_y: i32, viewport_height: i32) -> Request {
        let request = self.base_request_for_viewport(offset_y, viewport_height);
        self.prepared_runway
            .as_ref()
            .filter(|prepared| {
                prepared.start <= request.range.start && prepared.end >= request.range.end
            })
            .map_or(request.clone(), |prepared| {
                request.with_range(prepared.clone())
            })
    }

    fn base_request_for_viewport(&self, offset_y: i32, viewport_height: i32) -> Request {
        match &self.heights {
            Heights::Uniform(row_height) => {
                let row_height = (*row_height).max(1);
                let visible_start = (offset_y.max(0) / row_height) as usize;
                let visible_end =
                    ((offset_y.max(0) as i64 + viewport_height.max(0) as i64 + row_height as i64
                        - 1)
                        / row_height as i64) as usize;
                let range = visible_start.saturating_sub(self.overscan)
                    ..visible_end.saturating_add(self.overscan).min(self.len());
                Request::new(self.id, range, self.len())
            }
            Heights::Variable(measurements) => {
                let materialization = measurements.borrow_mut().request(
                    offset_y,
                    viewport_height,
                    self.overscan,
                    Vec::new(),
                    self.model.as_ref(),
                );
                Request::variable(
                    self.id,
                    materialization.range(),
                    self.len(),
                    measurements.clone(),
                )
            }
        }
    }

    pub(crate) fn request_for_transition(
        &self,
        offset_y: i32,
        viewport_height: i32,
        baseline_y: i32,
    ) -> Request {
        let request = self.base_request_for_viewport(offset_y, viewport_height);
        let direction = if offset_y > baseline_y {
            Some(Direction::Forward)
        } else if offset_y < baseline_y {
            Some(Direction::Backward)
        } else {
            None
        };
        let visible_rows = (viewport_height.max(1) as usize)
            .div_ceil(self.row_height().max(1) as usize)
            .max(1);
        let distance_rows =
            (offset_y.abs_diff(baseline_y) as usize).div_ceil(self.row_height().max(1) as usize);
        let runway_budget = MAX_TRANSITION_MATERIALIZED_ROWS.saturating_sub(request.range.len());
        let leading_goal = distance_rows
            .min(visible_rows.saturating_mul(MAX_LEADING_RUNWAY_VIEWPORTS))
            .max(visible_rows);
        let leading = leading_goal.min(runway_budget);
        let trailing = visible_rows
            .div_ceil(2)
            .max(self.overscan)
            .min(runway_budget.saturating_sub(leading));
        direction.map_or(request.clone(), |direction| {
            request.with_runway(direction, leading, trailing)
        })
    }

    pub(crate) fn model(&self) -> &dyn Model {
        self.model.as_ref()
    }

    pub(crate) fn index_at_offset(&self, offset: i32) -> usize {
        match &self.heights {
            Heights::Uniform(height) => (offset.max(0) / (*height).max(1)) as usize,
            Heights::Variable(region) => region.borrow().index_for_offset(offset),
        }
    }

    pub(crate) fn contains_key(&self, key: Key) -> bool {
        self.model.index_of(key).is_some()
    }

    pub(crate) fn is_selectable(&self) -> bool {
        self.selectable
    }

    pub(crate) fn reconcile_selection(&self, selection: &mut crate::selection::Selection) -> bool {
        selection.reconcile(self.model.as_ref())
    }

    pub(crate) fn select_row(
        &self,
        selection: &mut crate::selection::Selection,
        key: Key,
        index: usize,
        extend: bool,
        toggle: bool,
    ) -> bool {
        selection.click(self.model.as_ref(), key, index, extend, toggle)
    }

    pub(crate) fn select_all(&self, selection: &mut crate::selection::Selection) -> bool {
        selection.select_all(self.model.as_ref())
    }

    pub(crate) fn move_selection(
        &self,
        selection: &mut crate::selection::Selection,
        movement: crate::selection::Move,
        extend: bool,
    ) -> bool {
        selection.move_active(self.model.as_ref(), movement, extend)
    }

    pub(crate) fn key_at(&self, index: usize) -> Option<Key> {
        (index < self.len()).then(|| self.model.key(index))
    }

    pub(crate) fn index_of(&self, key: Key) -> Option<usize> {
        self.model.index_of(key)
    }

    pub(crate) fn initial_materialization(&self) -> Materialization {
        Materialization::new(0..self.len().min(INITIAL_ROWS), Vec::new())
    }

    pub(crate) fn materialize(
        &mut self,
        requested: &Materialization,
        measurements: Option<&Measurements>,
    ) -> (Vec<view::Node>, MaterializationStats) {
        if matches!(self.heights, Heights::Variable(_))
            && let Some(measurements) = measurements
        {
            self.heights = Heights::Variable(measurements.clone());
        }
        let len = self.len();
        let start = requested.range.start.min(len);
        let end = requested.range.end.max(start).min(len);
        self.prepared_runway = requested.runway.then_some(start..end);
        let mut rows = (start..end)
            .map(|index| self.desired_item(index))
            .collect::<Vec<_>>();

        for key in &requested.pins {
            if rows.iter().any(|item| item.key == *key) {
                continue;
            }
            if let Some(index) = self.model.index_of(*key).filter(|index| *index < len) {
                let item = self.desired_item(index);
                assert_eq!(
                    item.key, *key,
                    "list model index_of must round-trip every pinned stable key"
                );
                rows.push(item);
            }
        }
        rows.sort_unstable_by_key(|item| item.index);

        let mut unique = HashSet::with_capacity(rows.len());
        for item in &rows {
            assert!(
                unique.insert(item.key),
                "list models must expose globally unique stable keys"
            );
        }
        self.slots.borrow_mut().materialize(
            self.id,
            self.model.as_ref(),
            Rc::clone(&self.factory),
            start..end,
            requested.pins.clone(),
            rows,
        )
    }

    /// Applies a residency-only request without materializing retained
    /// overlap. Models which cannot prove a stable residency revision fall
    /// back to an explicit reset; the fallback stays correct and visible in
    /// the returned delta and counters.
    pub(crate) fn materialize_residency(
        &mut self,
        requested: &Materialization,
        measurements: Option<&Measurements>,
    ) -> (ResidencyDelta, MaterializationStats) {
        if matches!(self.heights, Heights::Variable(_))
            && let Some(measurements) = measurements
        {
            self.heights = Heights::Variable(measurements.clone());
        }
        let len = self.len();
        let start = requested.range.start.min(len);
        let end = requested.range.end.max(start).min(len);
        self.prepared_runway = requested.runway.then_some(start..end);
        let range = start..end;

        if requested.pins.is_empty()
            && let Some(result) = self.slots.borrow_mut().materialize_residency(
                self.id,
                self.model.as_ref(),
                Rc::clone(&self.factory),
                range.clone(),
            )
        {
            return result;
        }

        let (nodes, stats) = self.materialize(requested, measurements);
        (ResidencyDelta::reset(self.id, nodes), stats)
    }

    fn desired_item(&self, index: usize) -> DesiredItem {
        let key = self.model.key(index);
        assert_eq!(
            self.model.index_of(key),
            Some(index),
            "list model key and index_of must be an exact stable-identity inverse"
        );
        DesiredItem {
            key,
            index,
            revision: self.model.item_revision(index),
        }
    }
}

impl Slots {
    fn materialize(
        &mut self,
        list: interaction::Id,
        model: &dyn Model,
        factory: Rc<dyn Factory>,
        range: Range<usize>,
        pins: Vec<Key>,
        desired: Vec<DesiredItem>,
    ) -> (Vec<view::Node>, MaterializationStats) {
        self.observe_factory(factory.as_ref());
        let (membership_revision, membership_changes) = self.observe_membership(model);
        let mut previous = std::mem::take(&mut self.active);
        let resident_rows_before = previous.len();
        let old_interval_start = previous.values().map(|bound| bound.index).min();
        let old_interval_end = previous
            .values()
            .map(|bound| bound.index.saturating_add(1))
            .max();
        let new_interval_start = desired.iter().map(|item| item.index).min();
        let new_interval_end = desired
            .iter()
            .map(|item| item.index.saturating_add(1))
            .max();
        let mut active = HashMap::with_capacity(desired.len());
        let mut nodes = Vec::with_capacity(desired.len());
        let mut stats = MaterializationStats {
            lists: 1,
            old_interval_start,
            old_interval_end,
            new_interval_start,
            new_interval_end,
            resident_rows_before,
            resident_rows_after: desired.len(),
            membership_changes,
            membership_revision_max: membership_revision,
            view_nodes_cloned: desired.len(),
            ..MaterializationStats::default()
        };

        let desired_keys = desired.iter().map(|item| item.key).collect::<HashSet<_>>();
        let departing = previous
            .keys()
            .filter(|key| !desired_keys.contains(key))
            .copied()
            .collect::<Vec<_>>();
        stats.departing_rows = departing.len();
        for key in departing {
            let bound = previous
                .remove(&key)
                .expect("departing key came from the active slot set");
            bound
                .binding_factory
                .unbind(bound.slot.id, bound.key, bound.index);
            self.recycled.push(bound.slot);
        }

        let mut order = VecDeque::with_capacity(desired.len());
        for item in desired {
            let bound = if let Some(mut bound) = previous.remove(&item.key) {
                stats.overlapping_rows = stats.overlapping_rows.saturating_add(1);
                if bound.index != item.index {
                    stats.moved_rows = stats.moved_rows.saturating_add(1);
                }
                if bound.revision == item.revision {
                    bound.index = item.index;
                    bound
                } else {
                    stats.revised_rows = stats.revised_rows.saturating_add(1);
                    stats.provider_binds = stats.provider_binds.saturating_add(1);
                    bound
                        .binding_factory
                        .unbind(bound.slot.id, bound.key, bound.index);
                    Self::bind(bound.slot, Rc::clone(&factory), item)
                }
            } else {
                stats.entering_rows = stats.entering_rows.saturating_add(1);
                stats.provider_binds = stats.provider_binds.saturating_add(1);
                let slot = if let Some(slot) = self.recycled.pop() {
                    stats.slots_rebound = stats.slots_rebound.saturating_add(1);
                    slot
                } else {
                    self.next = self.next.saturating_add(1).max(1);
                    let id = Slot(self.next);
                    factory.setup(id);
                    AvailableSlot {
                        id,
                        setup_factory: Rc::clone(&factory),
                    }
                };
                Self::bind(slot, Rc::clone(&factory), item)
            };
            nodes.push(bound.node.clone().with_provided_row(
                list,
                bound.key,
                bound.slot.id,
                bound.index,
            ));
            order.push_back(bound.key);
            active.insert(bound.key, bound);
        }

        debug_assert!(previous.is_empty());
        while self.recycled.len() > MAX_RECYCLED_SLOTS {
            if let Some(slot) = self.recycled.pop() {
                slot.setup_factory.teardown(slot.id);
            }
        }
        self.active = active;
        self.order = order;
        self.range = Some(range);
        self.pins = pins;
        self.residency_revision = model.residency_revision();
        (nodes, stats)
    }

    fn materialize_residency(
        &mut self,
        list: interaction::Id,
        model: &dyn Model,
        factory: Rc<dyn Factory>,
        desired: Range<usize>,
    ) -> Option<(ResidencyDelta, MaterializationStats)> {
        let old = self.range.clone()?;
        let revision = model.residency_revision()?;
        if self.residency_revision != Some(revision)
            || !self.pins.is_empty()
            || self.factory_revision != Some(factory.revision())
            || self.membership_revision != Some(model.membership_revision())
            || self.order.len() != old.len()
            || self.active.len() != old.len()
        {
            return None;
        }

        let overlap_start = old.start.max(desired.start);
        let overlap_end = old.end.min(desired.end);
        let has_overlap = overlap_start < overlap_end;
        let (remove_front, remove_back, enter_front, enter_back) = if has_overlap {
            (
                overlap_start - old.start,
                old.end - overlap_end,
                desired.start..overlap_start,
                overlap_end..desired.end,
            )
        } else {
            (old.len(), 0, desired.start..desired.start, desired.clone())
        };

        let resident_rows_before = old.len();
        let recycled_before = self.recycled.len();
        for _ in 0..remove_front {
            let key = self
                .order
                .pop_front()
                .expect("front residency departure must name an active key");
            self.depart(key);
        }
        for _ in 0..remove_back {
            let key = self
                .order
                .pop_back()
                .expect("back residency departure must name an active key");
            self.depart(key);
        }

        let mut inserted_front = Vec::with_capacity(enter_front.len());
        for index in enter_front.clone().rev() {
            let item = desired_item(model, index);
            let bound = self.bind_entering(Rc::clone(&factory), item);
            inserted_front.push(bound.node.clone().with_provided_row(
                list,
                bound.key,
                bound.slot.id,
                bound.index,
            ));
            self.order.push_front(bound.key);
            self.active.insert(bound.key, bound);
        }
        inserted_front.reverse();

        let mut inserted_back = Vec::with_capacity(enter_back.len());
        for index in enter_back.clone() {
            let item = desired_item(model, index);
            let bound = self.bind_entering(Rc::clone(&factory), item);
            inserted_back.push(bound.node.clone().with_provided_row(
                list,
                bound.key,
                bound.slot.id,
                bound.index,
            ));
            self.order.push_back(bound.key);
            self.active.insert(bound.key, bound);
        }

        while self.recycled.len() > MAX_RECYCLED_SLOTS {
            if let Some(slot) = self.recycled.pop() {
                slot.setup_factory.teardown(slot.id);
            }
        }
        self.range = Some(desired.clone());
        self.len = model.len();
        self.residency_revision = Some(revision);
        debug_assert_eq!(self.order.len(), desired.len());
        debug_assert_eq!(self.active.len(), desired.len());

        let entering = enter_front.len().saturating_add(enter_back.len());
        let departing = remove_front.saturating_add(remove_back);
        let stats = MaterializationStats {
            lists: 1,
            old_interval_start: Some(old.start),
            old_interval_end: Some(old.end),
            new_interval_start: Some(desired.start),
            new_interval_end: Some(desired.end),
            resident_rows_before,
            resident_rows_after: desired.len(),
            entering_rows: entering,
            departing_rows: departing,
            overlapping_rows: if has_overlap {
                overlap_end - overlap_start
            } else {
                0
            },
            membership_revision_max: self.membership_revision.unwrap_or_default(),
            provider_binds: entering,
            slots_rebound: entering.min(recycled_before.saturating_add(departing)),
            view_nodes_cloned: entering,
            ..MaterializationStats::default()
        };
        Some((
            ResidencyDelta {
                list,
                remove_front,
                remove_back,
                insert_front: inserted_front,
                insert_back: inserted_back,
                reset: None,
            },
            stats,
        ))
    }

    fn depart(&mut self, key: Key) {
        let bound = self
            .active
            .remove(&key)
            .expect("residency departure must name an active slot");
        bound
            .binding_factory
            .unbind(bound.slot.id, bound.key, bound.index);
        self.recycled.push(bound.slot);
    }

    fn bind_entering(&mut self, factory: Rc<dyn Factory>, item: DesiredItem) -> BoundSlot {
        let slot = if let Some(slot) = self.recycled.pop() {
            slot
        } else {
            self.next = self.next.saturating_add(1).max(1);
            let id = Slot(self.next);
            factory.setup(id);
            AvailableSlot {
                id,
                setup_factory: Rc::clone(&factory),
            }
        };
        Self::bind(slot, factory, item)
    }

    fn bind(slot: AvailableSlot, factory: Rc<dyn Factory>, item: DesiredItem) -> BoundSlot {
        let node = factory.bind(slot.id, item.index);
        BoundSlot {
            slot,
            key: item.key,
            index: item.index,
            revision: item.revision,
            node,
            binding_factory: factory,
        }
    }

    fn observe_membership(&mut self, model: &dyn Model) -> (u64, usize) {
        let revision = model.membership_revision();
        let mut observed_changes = 0;
        if let Some(previous) = self.membership_revision
            && revision != previous
        {
            let changes = model.changes_since(previous);
            observed_changes = changes.len();
            assert!(
                !changes.is_empty(),
                "a changed list membership revision must describe its mutations"
            );
            let resolved_len = changes
                .into_iter()
                .fold(self.len, |len, change| change.next_len(len));
            assert_eq!(
                resolved_len,
                model.len(),
                "list membership mutations must transform the prior length into the current length"
            );
        }
        self.membership_revision = Some(revision);
        self.len = model.len();
        (revision, observed_changes)
    }

    fn observe_factory(&mut self, factory: &dyn Factory) {
        let revision = factory.revision();
        if (!self.active.is_empty() || !self.recycled.is_empty())
            && self.factory_revision != Some(revision)
        {
            self.retire();
        }
        self.factory_revision = Some(revision);
    }

    fn retire(&mut self) {
        for (_, bound) in self.active.drain() {
            bound
                .binding_factory
                .unbind(bound.slot.id, bound.key, bound.index);
            bound.slot.setup_factory.teardown(bound.slot.id);
        }
        for slot in self.recycled.drain(..) {
            slot.setup_factory.teardown(slot.id);
        }
        self.order.clear();
        self.range = None;
        self.pins.clear();
        self.residency_revision = None;
    }

    #[cfg(test)]
    fn slot_for(&self, key: Key) -> Option<Slot> {
        self.active.get(&key).map(|bound| bound.slot.id)
    }
}

fn desired_item(model: &dyn Model, index: usize) -> DesiredItem {
    let key = model.key(index);
    assert_eq!(
        model.index_of(key),
        Some(index),
        "list model key and index_of must be an exact stable-identity inverse"
    );
    DesiredItem {
        key,
        index,
        revision: model.item_revision(index),
    }
}

impl ResidencyDelta {
    fn reset(list: interaction::Id, nodes: Vec<view::Node>) -> Self {
        Self {
            list,
            remove_front: 0,
            remove_back: 0,
            insert_front: Vec::new(),
            insert_back: Vec::new(),
            reset: Some(nodes),
        }
    }

    #[cfg(test)]
    pub(crate) fn list(&self) -> interaction::Id {
        self.list
    }

    #[cfg(test)]
    pub(crate) fn remove_front(&self) -> usize {
        self.remove_front
    }

    #[cfg(test)]
    pub(crate) fn remove_back(&self) -> usize {
        self.remove_back
    }

    #[cfg(test)]
    pub(crate) fn insert_front(&self) -> &[view::Node] {
        &self.insert_front
    }

    #[cfg(test)]
    pub(crate) fn insert_back(&self) -> &[view::Node] {
        &self.insert_back
    }

    #[cfg(test)]
    pub(crate) fn reset_nodes(&self) -> Option<&[view::Node]> {
        self.reset.as_deref()
    }

    #[cfg(test)]
    pub(crate) fn is_keyed(&self) -> bool {
        self.reset.is_none()
    }

    pub(crate) fn into_parts(self) -> ResidencyDeltaParts {
        ResidencyDeltaParts {
            list: self.list,
            remove_front: self.remove_front,
            remove_back: self.remove_back,
            insert_front: self.insert_front,
            insert_back: self.insert_back,
            reset: self.reset,
        }
    }
}

impl AppliedResidencyDelta {
    pub(crate) fn new(parts: &ResidencyDeltaParts) -> Self {
        Self {
            list: parts.list,
            remove_front: parts.remove_front,
            remove_back: parts.remove_back,
            insert_front: parts.insert_front.len(),
            insert_back: parts.insert_back.len(),
            reset: parts.reset.is_some(),
        }
    }

    pub(crate) fn list(self) -> interaction::Id {
        self.list
    }

    pub(crate) fn remove_front(self) -> usize {
        self.remove_front
    }

    pub(crate) fn remove_back(self) -> usize {
        self.remove_back
    }

    pub(crate) fn insert_front(self) -> usize {
        self.insert_front
    }

    pub(crate) fn insert_back(self) -> usize {
        self.insert_back
    }

    pub(crate) fn is_reset(self) -> bool {
        self.reset
    }
}

impl MaterializationStats {
    pub(crate) fn add(&mut self, other: Self) {
        self.lists = self.lists.saturating_add(other.lists);
        self.old_interval_start = min_option(self.old_interval_start, other.old_interval_start);
        self.old_interval_end = max_option(self.old_interval_end, other.old_interval_end);
        self.new_interval_start = min_option(self.new_interval_start, other.new_interval_start);
        self.new_interval_end = max_option(self.new_interval_end, other.new_interval_end);
        self.resident_rows_before = self
            .resident_rows_before
            .saturating_add(other.resident_rows_before);
        self.resident_rows_after = self
            .resident_rows_after
            .saturating_add(other.resident_rows_after);
        self.entering_rows = self.entering_rows.saturating_add(other.entering_rows);
        self.departing_rows = self.departing_rows.saturating_add(other.departing_rows);
        self.overlapping_rows = self.overlapping_rows.saturating_add(other.overlapping_rows);
        self.revised_rows = self.revised_rows.saturating_add(other.revised_rows);
        self.moved_rows = self.moved_rows.saturating_add(other.moved_rows);
        self.membership_changes = self
            .membership_changes
            .saturating_add(other.membership_changes);
        self.membership_revision_max = self
            .membership_revision_max
            .max(other.membership_revision_max);
        self.provider_binds = self.provider_binds.saturating_add(other.provider_binds);
        self.slots_rebound = self.slots_rebound.saturating_add(other.slots_rebound);
        self.view_nodes_cloned = self
            .view_nodes_cloned
            .saturating_add(other.view_nodes_cloned);
    }
}

fn min_option(left: Option<usize>, right: Option<usize>) -> Option<usize> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (left, right) => left.or(right),
    }
}

fn max_option(left: Option<usize>, right: Option<usize>) -> Option<usize> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (left, right) => left.or(right),
    }
}

impl Drop for Slots {
    fn drop(&mut self) {
        self.retire();
    }
}

impl Materialization {
    pub(crate) fn new(range: Range<usize>, mut pins: Vec<Key>) -> Self {
        pins.sort_unstable();
        pins.dedup();
        Self {
            range,
            pins,
            runway: false,
        }
    }

    pub(crate) fn with_range(&self, range: Range<usize>) -> Self {
        Self::new(range, self.pins.clone())
    }

    pub(crate) fn with_runway(&self, range: Range<usize>) -> Self {
        Self {
            range,
            pins: self.pins.clone(),
            runway: true,
        }
    }

    pub(crate) fn preserves(&self, range: &Range<usize>) -> bool {
        self.runway && self.range.start <= range.start && self.range.end >= range.end
    }

    pub(crate) fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    pub(crate) fn with_pins(&self, pins: Vec<Key>) -> Self {
        Self::with_pins_and_runway(self.range.clone(), pins, self.runway)
    }

    pub(crate) fn with_pin(&self, pin: Key) -> Self {
        let mut pins = self.pins.clone();
        pins.push(pin);
        Self::with_pins_and_runway(self.range.clone(), pins, self.runway)
    }

    fn with_pins_and_runway(range: Range<usize>, mut pins: Vec<Key>, runway: bool) -> Self {
        pins.sort_unstable();
        pins.dedup();
        Self {
            range,
            pins,
            runway,
        }
    }
}

impl Request {
    pub(crate) fn new(id: interaction::Id, range: Range<usize>, limit: usize) -> Self {
        Self {
            id,
            range,
            limit,
            measurements: None,
        }
    }

    pub(crate) fn variable(
        id: interaction::Id,
        range: Range<usize>,
        limit: usize,
        measurements: Measurements,
    ) -> Self {
        Self {
            id,
            range,
            limit,
            measurements: Some(measurements),
        }
    }

    pub(crate) fn id(&self) -> interaction::Id {
        self.id
    }

    pub(crate) fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    pub(crate) fn measurements(&self) -> Option<Measurements> {
        self.measurements.clone()
    }

    fn with_range(&self, range: Range<usize>) -> Self {
        Self {
            id: self.id,
            range: range.start.min(self.limit)..range.end.min(self.limit),
            limit: self.limit,
            measurements: self.measurements.clone(),
        }
    }

    fn with_runway(&self, direction: Direction, leading: usize, trailing: usize) -> Self {
        let (before, after) = match direction {
            Direction::Backward => (leading, trailing),
            Direction::Forward => (trailing, leading),
        };
        Self {
            id: self.id,
            range: self.range.start.saturating_sub(before)
                ..self.range.end.saturating_add(after).min(self.limit),
            limit: self.limit,
            measurements: self.measurements.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[derive(Default)]
    struct LifecycleLog {
        setup: Vec<Slot>,
        bind: Vec<(Slot, Key, usize)>,
        unbind: Vec<(Slot, Key, usize)>,
        teardown: Vec<Slot>,
    }

    struct MutableState {
        keys: Vec<Key>,
        item_revisions: HashMap<Key, u64>,
        membership_revision: u64,
        factory_revision: u64,
        changes: Vec<(u64, Change)>,
    }

    #[derive(Clone)]
    struct LifecycleRows {
        state: Rc<RefCell<MutableState>>,
        log: Rc<RefCell<LifecycleLog>>,
    }

    impl LifecycleRows {
        fn new(keys: impl IntoIterator<Item = u64>) -> Self {
            let keys = keys.into_iter().map(Key::new).collect::<Vec<_>>();
            let item_revisions = keys.iter().map(|key| (*key, 0)).collect();
            Self {
                state: Rc::new(RefCell::new(MutableState {
                    keys,
                    item_revisions,
                    membership_revision: 0,
                    factory_revision: 0,
                    changes: Vec::new(),
                })),
                log: Rc::new(RefCell::new(LifecycleLog::default())),
            }
        }

        fn move_item(&self, from: usize, to: usize) {
            let mut state = self.state.borrow_mut();
            let key = state.keys.remove(from);
            state.keys.insert(to, key);
            state.membership_revision += 1;
            let revision = state.membership_revision;
            state
                .changes
                .push((revision, Change::Move { from, to, count: 1 }));
        }

        fn revise(&self, key: Key) {
            let mut state = self.state.borrow_mut();
            *state.item_revisions.get_mut(&key).expect("known key") += 1;
        }

        fn replace(&self, index: usize, key: Key) {
            let mut state = self.state.borrow_mut();
            let previous = std::mem::replace(&mut state.keys[index], key);
            state.item_revisions.remove(&previous);
            state.item_revisions.insert(key, 0);
            state.membership_revision += 1;
            let revision = state.membership_revision;
            state.changes.push((
                revision,
                Change::Replace {
                    index,
                    removed: 1,
                    added: 1,
                },
            ));
        }

        fn remove(&self, index: usize) {
            let mut state = self.state.borrow_mut();
            let key = state.keys.remove(index);
            state.item_revisions.remove(&key);
            state.membership_revision += 1;
            let revision = state.membership_revision;
            state
                .changes
                .push((revision, Change::Remove { index, count: 1 }));
        }

        fn insert(&self, index: usize, key: Key) {
            let mut state = self.state.borrow_mut();
            state.keys.insert(index, key);
            state.item_revisions.insert(key, 0);
            state.membership_revision += 1;
            let revision = state.membership_revision;
            state
                .changes
                .push((revision, Change::Insert { index, count: 1 }));
        }

        fn replace_factory(&self) {
            self.state.borrow_mut().factory_revision += 1;
        }
    }

    impl Model for LifecycleRows {
        fn len(&self) -> usize {
            self.state.borrow().keys.len()
        }

        fn key(&self, index: usize) -> Key {
            self.state.borrow().keys[index]
        }

        fn index_of(&self, key: Key) -> Option<usize> {
            self.state
                .borrow()
                .keys
                .iter()
                .position(|candidate| *candidate == key)
        }

        fn membership_revision(&self) -> u64 {
            self.state.borrow().membership_revision
        }

        fn changes_since(&self, revision: u64) -> Vec<Change> {
            self.state
                .borrow()
                .changes
                .iter()
                .filter_map(|(current, change)| (*current > revision).then_some(*change))
                .collect()
        }

        fn item_revision(&self, index: usize) -> u64 {
            let state = self.state.borrow();
            state.item_revisions[&state.keys[index]]
        }
    }

    impl Factory for LifecycleRows {
        fn revision(&self) -> u64 {
            self.state.borrow().factory_revision
        }

        fn setup(&self, slot: Slot) {
            self.log.borrow_mut().setup.push(slot);
        }

        fn bind(&self, slot: Slot, index: usize) -> view::Node {
            self.log
                .borrow_mut()
                .bind
                .push((slot, self.key(index), index));
            view::Node::label(format!("row {}", self.key(index).value()))
        }

        fn unbind(&self, slot: Slot, key: Key, index: usize) {
            self.log.borrow_mut().unbind.push((slot, key, index));
        }

        fn teardown(&self, slot: Slot) {
            self.log.borrow_mut().teardown.push(slot);
        }
    }

    struct Rows(usize);

    impl Model for Rows {
        fn len(&self) -> usize {
            self.0
        }

        fn key(&self, index: usize) -> Key {
            Key::new(index as u64)
        }

        fn index_of(&self, key: Key) -> Option<usize> {
            let index = key.value() as usize;
            (index < self.0).then_some(index)
        }

        fn membership_revision(&self) -> u64 {
            0
        }

        fn changes_since(&self, _revision: u64) -> Vec<Change> {
            Vec::new()
        }

        fn item_revision(&self, _index: usize) -> u64 {
            0
        }
    }

    impl Factory for Rows {
        fn revision(&self) -> u64 {
            0
        }

        fn bind(&self, _slot: Slot, _index: usize) -> view::Node {
            view::Node::root()
        }
    }

    #[derive(Clone)]
    struct CountingRows {
        len: usize,
        key_queries: Rc<Cell<usize>>,
        index_queries: Rc<Cell<usize>>,
        item_queries: Rc<Cell<usize>>,
        binds: Rc<Cell<usize>>,
    }

    impl CountingRows {
        fn new(len: usize) -> Self {
            Self {
                len,
                key_queries: Rc::new(Cell::new(0)),
                index_queries: Rc::new(Cell::new(0)),
                item_queries: Rc::new(Cell::new(0)),
                binds: Rc::new(Cell::new(0)),
            }
        }

        fn reset_queries(&self) {
            self.key_queries.set(0);
            self.index_queries.set(0);
            self.item_queries.set(0);
            self.binds.set(0);
        }
    }

    impl Model for CountingRows {
        fn len(&self) -> usize {
            self.len
        }

        fn key(&self, index: usize) -> Key {
            self.key_queries.set(self.key_queries.get() + 1);
            Key::new(index as u64)
        }

        fn index_of(&self, key: Key) -> Option<usize> {
            self.index_queries.set(self.index_queries.get() + 1);
            let index = key.value() as usize;
            (index < self.len).then_some(index)
        }

        fn membership_revision(&self) -> u64 {
            0
        }

        fn changes_since(&self, _revision: u64) -> Vec<Change> {
            Vec::new()
        }

        fn item_revision(&self, _index: usize) -> u64 {
            self.item_queries.set(self.item_queries.get() + 1);
            0
        }

        fn residency_revision(&self) -> Option<u64> {
            Some(0)
        }
    }

    impl Factory for CountingRows {
        fn revision(&self) -> u64 {
            0
        }

        fn bind(&self, _slot: Slot, index: usize) -> view::Node {
            self.binds.set(self.binds.get() + 1);
            view::Node::label(format!("row {index}"))
        }
    }

    fn lifecycle_model(provider: &Rc<LifecycleRows>) -> State {
        let model: Rc<dyn Model> = provider.clone();
        let factory: Rc<dyn Factory> = provider.clone();
        State::new(interaction::Id::new("lifecycle.rows"), 20, model, factory)
    }

    fn rebuild_lifecycle_model(previous: &State, provider: &Rc<LifecycleRows>) -> State {
        let mut next = lifecycle_model(provider);
        next.reuse_slots_from(previous);
        next
    }

    #[test]
    fn residency_delta_preserves_every_overlap_without_querying_it() {
        fn exercise(resident: usize) -> (usize, usize, usize, usize) {
            let provider = Rc::new(CountingRows::new(10_000));
            let model: Rc<dyn Model> = provider.clone();
            let factory: Rc<dyn Factory> = provider.clone();
            let mut state = State::new(interaction::Id::new("counting.rows"), 20, model, factory);
            state.materialize(&Materialization::new(0..resident, Vec::new()), None);
            provider.reset_queries();

            let (delta, stats) = state
                .materialize_residency(&Materialization::new(1..resident + 1, Vec::new()), None);
            assert!(delta.is_keyed());
            assert_eq!(delta.list(), interaction::Id::new("counting.rows"));
            assert_eq!(delta.remove_front(), 1);
            assert_eq!(delta.remove_back(), 0);
            assert!(delta.insert_front().is_empty());
            assert_eq!(delta.insert_back().len(), 1);
            assert!(delta.reset_nodes().is_none());
            assert_eq!(stats.entering_rows, 1);
            assert_eq!(stats.departing_rows, 1);
            assert_eq!(stats.overlapping_rows, resident - 1);
            assert_eq!(stats.provider_binds, 1);
            assert_eq!(stats.view_nodes_cloned, 1);

            (
                provider.key_queries.get(),
                provider.index_queries.get(),
                provider.item_queries.get(),
                provider.binds.get(),
            )
        }

        let small = exercise(16);
        let large = exercise(64);
        assert_eq!(small, (1, 1, 1, 1));
        assert_eq!(
            large, small,
            "constant edge delta must ignore resident population"
        );
    }

    #[test]
    fn keyed_residency_composition_work_is_flat_when_population_doubles() {
        fn exercise(resident: usize) -> (usize, usize, usize, usize) {
            let provider = Rc::new(CountingRows::new(10_000));
            let model: Rc<dyn Model> = provider.clone();
            let factory: Rc<dyn Factory> = provider;
            let state = State::new(interaction::Id::new("composition.rows"), 20, model, factory);
            let mut view = view::View::new(view::Node::virtual_list(state));
            let initial = HashMap::from([(
                interaction::Id::new("composition.rows"),
                Materialization::new(0..resident, Vec::new()),
            )]);
            view.materialize_virtual_lists(&initial, &HashMap::new(), None);
            let mut next_node_id = 1;
            let (mut tree, _) = crate::composition::tree::Tree::new(&view, &mut next_node_id);
            let before = tree.root().children()[0]
                .children()
                .iter()
                .filter_map(|node| node.provided_row().map(|row| (row.key(), node.node_id())))
                .collect::<HashMap<_, _>>();

            let next = HashMap::from([(
                interaction::Id::new("composition.rows"),
                Materialization::new(1..resident + 1, Vec::new()),
            )]);
            let (deltas, stats) = view.materialize_virtual_lists_residency(&next, &HashMap::new());
            let changes = tree.reconcile_residency(&view, &deltas, &mut next_node_id);
            let after = tree.root().children()[0]
                .children()
                .iter()
                .filter_map(|node| node.provided_row().map(|row| (row.key(), node.node_id())))
                .collect::<HashMap<_, _>>();

            for key in (1..resident).map(|value| Key::new(value as u64)) {
                assert_eq!(after.get(&key), before.get(&key));
            }
            assert_eq!(stats.view_nodes_cloned, 1);
            (
                changes.nodes_visited(),
                changes.nodes_reconstructed(),
                changes.identities_reused(),
                changes.added().len(),
            )
        }

        let small = exercise(16);
        let large = exercise(64);
        assert_eq!(small, (2, 1, 1, 1));
        assert_eq!(
            large, small,
            "composition work must depend on delta, not overlap"
        );
    }

    #[test]
    fn keyed_slots_reuse_moves_rebind_revisions_and_teardown_exactly() {
        let provider = Rc::new(LifecycleRows::new(0..4));
        let mut model = lifecycle_model(&provider);
        let initial = Materialization::new(0..3, Vec::new());
        let (rows, initial_stats) = model.materialize(&initial, None);
        assert_eq!(initial_stats.entering_rows, 3);
        assert_eq!(initial_stats.overlapping_rows, 0);
        assert_eq!(provider.log.borrow().setup.len(), 3);
        assert_eq!(provider.log.borrow().bind.len(), 3);
        let first_slots = [Key::new(0), Key::new(1), Key::new(2)]
            .map(|key| model.slots.borrow().slot_for(key).expect("bound slot"));
        assert_eq!(
            rows.iter()
                .map(|row| row.provided_row().expect("provided row").key())
                .collect::<Vec<_>>(),
            vec![Key::new(0), Key::new(1), Key::new(2)]
        );

        let mut next = rebuild_lifecycle_model(&model, &provider);
        drop(model);
        next.materialize(&Materialization::new(1..4, Vec::new()), None);
        assert_eq!(provider.log.borrow().setup.len(), 3);
        assert_eq!(provider.log.borrow().bind.len(), 4);
        assert_eq!(provider.log.borrow().unbind.len(), 1);
        assert_eq!(
            next.slots.borrow().slot_for(Key::new(1)),
            Some(first_slots[1])
        );
        assert_eq!(
            next.slots.borrow().slot_for(Key::new(2)),
            Some(first_slots[2])
        );
        assert_eq!(
            next.slots.borrow().slot_for(Key::new(3)),
            Some(first_slots[0])
        );

        let mut full = rebuild_lifecycle_model(&next, &provider);
        drop(next);
        full.materialize(&Materialization::new(0..4, Vec::new()), None);
        let slots_before_move = (0..4)
            .map(Key::new)
            .map(|key| (key, full.slots.borrow().slot_for(key).unwrap()))
            .collect::<HashMap<_, _>>();
        let binds_before_move = provider.log.borrow().bind.len();
        let unbinds_before_move = provider.log.borrow().unbind.len();

        provider.move_item(3, 0);
        let mut moved = rebuild_lifecycle_model(&full, &provider);
        drop(full);
        let (moved_rows, moved_stats) =
            moved.materialize(&Materialization::new(0..4, Vec::new()), None);
        assert_eq!(moved_stats.entering_rows, 0);
        assert_eq!(moved_stats.departing_rows, 0);
        assert_eq!(moved_stats.overlapping_rows, 4);
        assert_eq!(moved_stats.moved_rows, 4);
        assert_eq!(moved_stats.membership_changes, 1);
        assert_eq!(provider.log.borrow().bind.len(), binds_before_move);
        assert_eq!(provider.log.borrow().unbind.len(), unbinds_before_move);
        for key in (0..4).map(Key::new) {
            assert_eq!(
                moved.slots.borrow().slot_for(key),
                Some(slots_before_move[&key])
            );
        }
        assert_eq!(
            moved_rows
                .iter()
                .map(|row| row.provided_row().unwrap().key())
                .collect::<Vec<_>>(),
            vec![Key::new(3), Key::new(0), Key::new(1), Key::new(2)]
        );

        provider.revise(Key::new(1));
        let slot_one = moved.slots.borrow().slot_for(Key::new(1)).unwrap();
        let mut revised = rebuild_lifecycle_model(&moved, &provider);
        drop(moved);
        revised.materialize(&Materialization::new(0..4, Vec::new()), None);
        assert_eq!(provider.log.borrow().bind.len(), binds_before_move + 1);
        assert_eq!(provider.log.borrow().unbind.len(), unbinds_before_move + 1);
        assert_eq!(revised.slots.borrow().slot_for(Key::new(1)), Some(slot_one));

        provider.replace(3, Key::new(9));
        let replaced_slot = revised.slots.borrow().slot_for(Key::new(2)).unwrap();
        let mut replaced = rebuild_lifecycle_model(&revised, &provider);
        drop(revised);
        replaced.materialize(&Materialization::new(0..4, Vec::new()), None);
        assert_eq!(
            replaced.slots.borrow().slot_for(Key::new(9)),
            Some(replaced_slot)
        );

        let removed_slot = replaced.slots.borrow().slot_for(Key::new(0)).unwrap();
        provider.remove(1);
        let mut removed = rebuild_lifecycle_model(&replaced, &provider);
        drop(replaced);
        removed.materialize(&Materialization::new(0..3, Vec::new()), None);
        assert!(removed.slots.borrow().slot_for(Key::new(0)).is_none());

        provider.insert(1, Key::new(8));
        let mut inserted = rebuild_lifecycle_model(&removed, &provider);
        drop(removed);
        inserted.materialize(&Materialization::new(0..4, Vec::new()), None);
        assert_eq!(
            inserted.slots.borrow().slot_for(Key::new(8)),
            Some(removed_slot)
        );

        let old_slot = inserted.slots.borrow().slot_for(Key::new(3)).unwrap();
        provider.replace_factory();
        let mut refactored = rebuild_lifecycle_model(&inserted, &provider);
        drop(inserted);
        refactored.materialize(&Materialization::new(0..4, Vec::new()), None);
        assert_ne!(
            refactored.slots.borrow().slot_for(Key::new(3)),
            Some(old_slot),
            "a changed factory must setup a new slot instead of reusing incompatible state"
        );
        drop(refactored);
        let log = provider.log.borrow();
        assert_eq!(log.setup.len(), 8);
        assert_eq!(log.teardown.len(), 8);
        assert_eq!(log.bind.len(), 12);
        assert_eq!(log.unbind.len(), 12);
    }

    struct DuplicateRows;

    impl Model for DuplicateRows {
        fn len(&self) -> usize {
            2
        }

        fn key(&self, _index: usize) -> Key {
            Key::new(1)
        }

        fn index_of(&self, _key: Key) -> Option<usize> {
            Some(0)
        }

        fn membership_revision(&self) -> u64 {
            0
        }

        fn changes_since(&self, _revision: u64) -> Vec<Change> {
            Vec::new()
        }

        fn item_revision(&self, _index: usize) -> u64 {
            0
        }
    }

    impl Factory for DuplicateRows {
        fn revision(&self) -> u64 {
            0
        }

        fn bind(&self, _slot: Slot, _index: usize) -> view::Node {
            view::Node::root()
        }
    }

    #[test]
    #[should_panic(expected = "exact stable-identity inverse")]
    fn duplicate_provider_keys_fail_instead_of_being_silently_deduplicated() {
        let source = Rc::new(DuplicateRows);
        let model: Rc<dyn Model> = source.clone();
        let factory: Rc<dyn Factory> = source;
        let mut model = State::new(interaction::Id::new("duplicate.rows"), 20, model, factory);
        model.materialize(&Materialization::new(0..2, Vec::new()), None);
    }

    #[test]
    fn transition_runway_is_viewport_relative_and_bounded_for_a_million_rows() {
        let source = Rc::new(Rows(1_000_000));
        let model: Rc<dyn Model> = source.clone();
        let factory: Rc<dyn Factory> = source;
        let model = State::new(interaction::Id::new("runway.rows"), 20, model, factory);
        let visible_rows = 20_usize;
        let baseline = model.request_for_viewport(20_000, 400);
        let forward = model.request_for_transition(20_400, 400, 20_000);
        let backward = model.request_for_transition(19_600, 400, 20_000);

        assert!(forward.range.start < model.index_at_offset(20_400));
        assert!(
            forward.range.end >= model.index_at_offset(21_200),
            "forward preparation must cover the target viewport plus one directional viewport"
        );
        assert!(
            backward.range.start <= model.index_at_offset(19_200),
            "backward preparation must cover the target viewport plus one directional viewport"
        );
        assert!(backward.range.end > model.index_at_offset(20_000));
        assert!(forward.range.len() <= baseline.range.len() + visible_rows * 2);
        assert!(backward.range.len() <= baseline.range.len() + visible_rows * 2);
        assert!(forward.range.end <= model.len());
        assert!(backward.range.end <= model.len());
    }

    #[test]
    fn pin_refresh_preserves_predictive_runway() {
        let visible = 20..30;
        let runway = 19..34;
        let materialization =
            Materialization::new(visible.clone(), Vec::new()).with_runway(runway.clone());

        let refreshed = materialization.with_pins(vec![Key::new(24)]);

        assert_eq!(refreshed.range(), runway);
        assert!(
            refreshed.preserves(&visible),
            "pin refresh must not let layout refinement trim the prepared forward runway"
        );
    }

    #[test]
    fn a_new_materialization_range_never_unions_with_the_previous_drawable() {
        let current = Materialization::new(100..130, Vec::new()).with_runway(100..130);
        let required = 120..145;
        let next = current.with_runway(required.clone());
        assert_eq!(
            next.range(),
            required,
            "a new active materialization must not draw rows retained only by an older candidate"
        );
        assert!(next.preserves(&(120..145)));
    }
}
