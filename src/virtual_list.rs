use std::{
    cell::RefCell,
    collections::HashSet,
    ops::{Deref, Range},
    rc::Rc,
};

use crate::{interaction, scene, view, widget::Widget};

mod variable;

const DEFAULT_OVERSCAN: usize = 2;
const INITIAL_ROWS: usize = 32;

/// Stable logical identity supplied by a virtual-list provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Key(u64);

/// Synchronous source for a flat, uniform-height virtual list.
pub trait Provider {
    fn len(&self) -> usize;
    fn key(&self, index: usize) -> Key;
    fn index_of(&self, key: Key) -> Option<usize>;
    fn row(&self, index: usize) -> view::Node;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A provided container that materializes only visible rows plus bounded pins.
pub struct VirtualList {
    model: Model,
    width: Option<view::Dimension>,
    height: Option<view::Dimension>,
    max_height: Option<i32>,
    background: Option<scene::Brush>,
}

#[derive(Clone)]
pub(crate) struct Model {
    id: interaction::Id,
    heights: Heights,
    overscan: usize,
    provider: Rc<dyn Provider>,
    selectable: bool,
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
}

#[derive(Clone)]
pub(crate) struct Request {
    id: interaction::Id,
    range: Range<usize>,
    measurements: Option<Measurements>,
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

impl From<u64> for Key {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl VirtualList {
    pub fn new(
        id: impl Into<interaction::Id>,
        row_height: i32,
        provider: impl Provider + 'static,
    ) -> Self {
        Self {
            model: Model::new(id.into(), row_height, Rc::new(provider)),
            width: None,
            height: None,
            max_height: None,
            background: None,
        }
    }

    /// Creates a virtual list whose materialized rows determine their heights.
    pub fn variable(
        id: impl Into<interaction::Id>,
        estimated_row_height: i32,
        provider: impl Provider + 'static,
    ) -> Self {
        Self {
            model: Model::variable(id.into(), estimated_row_height, Rc::new(provider)),
            width: None,
            height: None,
            max_height: None,
            background: None,
        }
    }

    pub fn overscan(mut self, rows: usize) -> Self {
        self.model.overscan = rows.min(32);
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
        self.model.selectable = true;
        self
    }
}

impl Widget for VirtualList {
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

        view::Node::virtual_list(self.model).with_style(style)
    }
}

impl Model {
    pub(crate) fn same_scene_state(&self, other: &Self) -> bool {
        self.id == other.id
            && self.row_height() == other.row_height()
            && self.overscan == other.overscan
            && self.selectable == other.selectable
            && self.len() == other.len()
    }

    fn new(id: interaction::Id, row_height: i32, provider: Rc<dyn Provider>) -> Self {
        Self {
            id,
            heights: Heights::Uniform(row_height.max(1)),
            overscan: DEFAULT_OVERSCAN,
            provider,
            selectable: false,
        }
    }

    fn variable(id: interaction::Id, estimate: i32, provider: Rc<dyn Provider>) -> Self {
        Self {
            id,
            heights: Heights::Variable(Measurements::new(estimate)),
            overscan: DEFAULT_OVERSCAN,
            provider,
            selectable: false,
        }
    }

    pub(crate) fn id(&self) -> interaction::Id {
        self.id
    }

    pub(crate) fn len(&self) -> usize {
        self.provider.len()
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
                Request::new(self.id, range)
            }
            Heights::Variable(measurements) => {
                let materialization = measurements.borrow_mut().request(
                    offset_y,
                    viewport_height,
                    self.overscan,
                    Vec::new(),
                    self.provider.as_ref(),
                );
                Request::variable(self.id, materialization.range(), measurements.clone())
            }
        }
    }

    pub(crate) fn provider(&self) -> &dyn Provider {
        self.provider.as_ref()
    }

    pub(crate) fn index_at_offset(&self, offset: i32) -> usize {
        match &self.heights {
            Heights::Uniform(height) => (offset.max(0) / (*height).max(1)) as usize,
            Heights::Variable(region) => region.borrow().index_for_offset(offset),
        }
    }

    pub(crate) fn contains_key(&self, key: Key) -> bool {
        self.provider.index_of(key).is_some()
    }

    pub(crate) fn is_selectable(&self) -> bool {
        self.selectable
    }

    pub(crate) fn reconcile_selection(&self, selection: &mut crate::selection::Selection) -> bool {
        selection.reconcile(self.provider.as_ref())
    }

    pub(crate) fn select_row(
        &self,
        selection: &mut crate::selection::Selection,
        key: Key,
        index: usize,
        extend: bool,
        toggle: bool,
    ) -> bool {
        selection.click(self.provider.as_ref(), key, index, extend, toggle)
    }

    pub(crate) fn select_all(&self, selection: &mut crate::selection::Selection) -> bool {
        selection.select_all(self.provider.as_ref())
    }

    pub(crate) fn move_selection(
        &self,
        selection: &mut crate::selection::Selection,
        movement: crate::selection::Move,
        extend: bool,
    ) -> bool {
        selection.move_active(self.provider.as_ref(), movement, extend)
    }

    pub(crate) fn key_at(&self, index: usize) -> Option<Key> {
        (index < self.len()).then(|| self.provider.key(index))
    }

    pub(crate) fn index_of(&self, key: Key) -> Option<usize> {
        self.provider.index_of(key)
    }

    pub(crate) fn initial_materialization(&self) -> Materialization {
        Materialization::new(0..self.len().min(INITIAL_ROWS), Vec::new())
    }

    pub(crate) fn materialize(
        &mut self,
        requested: &Materialization,
        measurements: Option<&Measurements>,
    ) -> Vec<view::Node> {
        if matches!(self.heights, Heights::Variable(_))
            && let Some(measurements) = measurements
        {
            self.heights = Heights::Variable(measurements.clone());
        }
        let len = self.len();
        let start = requested.range.start.min(len);
        let end = requested.range.end.max(start).min(len);
        let mut rows = (start..end)
            .map(|index| (index, self.provider.key(index)))
            .collect::<Vec<_>>();

        for key in &requested.pins {
            if rows.iter().any(|(_, row_key)| row_key == key) {
                continue;
            }
            if let Some(index) = self.provider.index_of(*key).filter(|index| *index < len) {
                rows.push((index, *key));
            }
        }
        rows.sort_unstable_by_key(|(index, _)| *index);

        let mut unique = HashSet::with_capacity(rows.len());
        rows.retain(|(_, key)| unique.insert(*key));
        rows.into_iter()
            .map(|(index, key)| {
                self.provider
                    .row(index)
                    .with_provided_row(self.id, key, index)
            })
            .collect()
    }
}

impl Materialization {
    pub(crate) fn new(range: Range<usize>, mut pins: Vec<Key>) -> Self {
        pins.sort_unstable();
        pins.dedup();
        Self { range, pins }
    }

    pub(crate) fn with_range(&self, range: Range<usize>) -> Self {
        Self::new(range, self.pins.clone())
    }

    pub(crate) fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    pub(crate) fn with_pins(&self, pins: Vec<Key>) -> Self {
        Self::new(self.range.clone(), pins)
    }

    pub(crate) fn with_pin(&self, pin: Key) -> Self {
        let mut pins = self.pins.clone();
        pins.push(pin);
        Self::new(self.range.clone(), pins)
    }
}

impl Request {
    pub(crate) fn new(id: interaction::Id, range: Range<usize>) -> Self {
        Self {
            id,
            range,
            measurements: None,
        }
    }

    pub(crate) fn variable(
        id: interaction::Id,
        range: Range<usize>,
        measurements: Measurements,
    ) -> Self {
        Self {
            id,
            range,
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
}
