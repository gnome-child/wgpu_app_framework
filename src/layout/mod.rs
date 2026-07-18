use super::{
    composition::{self, tree},
    geometry::{Point, Rect, Size},
    interaction, keymap, scene, session,
    theme::Theme,
    view,
};
use crate::animation;
use std::{
    collections::{HashMap, HashSet},
    ops::Index,
    ops::Range,
    sync::Arc,
};

mod algorithm;
mod chrome;
mod control;
mod engine;
mod flow;
mod frame;
mod hit;
mod measure;
mod path;
pub(crate) mod table;
mod text;
mod typography;
mod viewport;

pub(crate) use chrome::Chrome;
pub(crate) use control::{
    choice_label_rect, choice_mark_rect, control_content_extent, menu_row_parts, palette_row_parts,
    slider_label_rect, slider_thumb_rect, slider_track_rect, table_choice_label_rect,
    table_choice_mark_rect, table_content_rect, table_header_label_rect, table_sort_indicator_rect,
};
pub(crate) use engine::Engine;
pub(crate) use frame::SceneKey as FrameSceneKey;
pub(crate) use frame::{Clip as FrameClip, Frame};
pub(crate) use hit::Hit;
pub use text::Text;
pub(crate) use typography::{
    label_style_for, section_header_text, shortcut_run_gap, shortcut_text_style,
};
pub(crate) use viewport::Viewport;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PopupSurfaces {
    InFrame,
    Native,
}

#[derive(Clone)]
pub(crate) struct Layout {
    size: Size,
    frames: FrameList,
    chrome: Vec<Chrome>,
    table_tracks: Vec<table::Track>,
    scene_scroll_paths: HashMap<composition::tree::NodeId, Vec<composition::tree::NodeId>>,
    scroll_projections: Vec<ScrollProjection>,
    virtual_list_requests: Vec<crate::list::Request>,
    native_popup_owners: HashMap<composition::tree::NodeId, interaction::Id>,
    residency_deltas: Vec<crate::list::AppliedResidencyDelta>,
    residency_predecessors: HashMap<interaction::Id, RowSequence>,
    frames_constructed: usize,
    frames_reused: usize,
}

#[derive(Clone)]
pub(crate) struct FrameList {
    segments: Arc<[FrameSegment]>,
    len: usize,
    rows: usize,
}

#[derive(Clone)]
enum FrameSegment {
    Ordinary(Arc<[Frame]>),
    VirtualRows(RowSequence),
}

pub(crate) struct FrameIter<'a> {
    frames: &'a FrameList,
    front: usize,
    back: usize,
}

pub(crate) struct VirtualRowIter<'a> {
    frames: &'a FrameList,
    front: usize,
    back: usize,
}

#[derive(Clone)]
pub(crate) struct VirtualRowFragment {
    root: composition::tree::NodeId,
    list: interaction::Id,
    key: crate::list::Key,
    frames: Arc<[Frame]>,
}

impl VirtualRowFragment {
    pub(crate) fn root(&self) -> composition::tree::NodeId {
        self.root
    }

    pub(crate) fn list(&self) -> interaction::Id {
        self.list
    }

    pub(crate) fn key(&self) -> crate::list::Key {
        self.key
    }

    pub(crate) fn frames(&self) -> &[Frame] {
        &self.frames
    }

    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.frames, &other.frames)
    }
}

#[derive(Clone, Default)]
pub(crate) struct RowSequence {
    root: Option<Arc<RowNode>>,
    identity: Arc<()>,
}

struct RowNode {
    fragment: VirtualRowFragment,
    priority: u64,
    left: Option<Arc<RowNode>>,
    right: Option<Arc<RowNode>>,
    rows: usize,
    frames: usize,
}

impl FrameList {
    fn from_flat(frames: Vec<Frame>, rows: &[VirtualRowFragment]) -> Self {
        let row_roots = rows
            .iter()
            .cloned()
            .map(|row| (row.root(), row))
            .collect::<HashMap<_, _>>();
        let mut segments = Vec::new();
        let mut ordinary = Vec::new();
        let mut row_run = Vec::new();
        let mut row_list = None;
        let mut index = 0;
        while index < frames.len() {
            if let Some(row) = row_roots.get(&frames[index].node_id()) {
                if !ordinary.is_empty() {
                    flush_row_run(&mut segments, &mut row_run);
                    row_list = None;
                    segments.push(FrameSegment::Ordinary(Arc::from(std::mem::take(
                        &mut ordinary,
                    ))));
                }
                if row_list.is_some_and(|list| list != row.list()) {
                    flush_row_run(&mut segments, &mut row_run);
                }
                row_list = Some(row.list());
                debug_assert!(
                    frames[index..]
                        .iter()
                        .zip(row.frames())
                        .all(|(frame, row_frame)| frame.node_id() == row_frame.node_id()),
                    "virtual-row chunk must preserve flat frame order"
                );
                row_run.push(row.clone());
                index = index.saturating_add(row.frames().len());
            } else {
                flush_row_run(&mut segments, &mut row_run);
                row_list = None;
                ordinary.push(frames[index].clone());
                index += 1;
            }
        }
        flush_row_run(&mut segments, &mut row_run);
        if !ordinary.is_empty() {
            segments.push(FrameSegment::Ordinary(Arc::from(ordinary)));
        }
        Self {
            segments: Arc::from(segments),
            len: frames.len(),
            rows: rows.len(),
        }
    }

    fn from_sparse(frames: Vec<Frame>, mut rows: HashMap<interaction::Id, RowSequence>) -> Self {
        let mut segments = Vec::new();
        let mut ordinary = Vec::new();
        let mut len = 0_usize;
        let mut row_count = 0_usize;
        for frame in frames {
            let list = frame.virtual_list_request().map(crate::list::Request::id);
            ordinary.push(frame);
            len += 1;
            if let Some(list) = list
                && let Some(sequence) = rows.remove(&list)
            {
                segments.push(FrameSegment::Ordinary(Arc::from(std::mem::take(
                    &mut ordinary,
                ))));
                len = len.saturating_add(sequence.frame_len());
                row_count = row_count.saturating_add(sequence.len());
                segments.push(FrameSegment::VirtualRows(sequence));
            }
        }
        if !ordinary.is_empty() {
            segments.push(FrameSegment::Ordinary(Arc::from(ordinary)));
        }
        assert!(
            rows.is_empty(),
            "every retained row sequence needs a list frame"
        );
        Self {
            segments: Arc::from(segments),
            len,
            rows: row_count,
        }
    }

    fn row_sequences(&self) -> HashMap<interaction::Id, RowSequence> {
        self.segments
            .iter()
            .filter_map(|segment| match segment {
                FrameSegment::VirtualRows(rows) => {
                    rows.get(0).map(|row| (row.list(), rows.clone()))
                }
                FrameSegment::Ordinary(_) => None,
            })
            .collect()
    }

    pub(crate) fn iter(&self) -> FrameIter<'_> {
        FrameIter {
            frames: self,
            front: 0,
            back: self.len,
        }
    }

    fn virtual_rows(&self) -> VirtualRowIter<'_> {
        VirtualRowIter {
            frames: self,
            front: 0,
            back: self.rows,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    fn frame(&self, mut index: usize) -> Option<&Frame> {
        for segment in self.segments.iter() {
            match segment {
                FrameSegment::Ordinary(frames) => {
                    if index < frames.len() {
                        return frames.get(index);
                    }
                    index -= frames.len();
                }
                FrameSegment::VirtualRows(rows) => {
                    if index < rows.frame_len() {
                        return rows.frame(index);
                    }
                    index -= rows.frame_len();
                }
            }
        }
        None
    }

    fn virtual_row(&self, mut index: usize) -> Option<&VirtualRowFragment> {
        for segment in self.segments.iter() {
            if let FrameSegment::VirtualRows(rows) = segment {
                if index < rows.len() {
                    return rows.get(index);
                }
                index -= rows.len();
            }
        }
        None
    }
}

fn flush_row_run(segments: &mut Vec<FrameSegment>, rows: &mut Vec<VirtualRowFragment>) {
    if !rows.is_empty() {
        segments.push(FrameSegment::VirtualRows(RowSequence::from_rows(
            std::mem::take(rows),
        )));
    }
}

impl RowSequence {
    fn from_rows(rows: Vec<VirtualRowFragment>) -> Self {
        let root = rows.into_iter().fold(None, |root, fragment| {
            merge_rows(root, Some(row_node(fragment, None, None)))
        });
        Self {
            root,
            identity: Arc::new(()),
        }
    }

    pub(crate) fn len(&self) -> usize {
        row_count(&self.root)
    }

    fn frame_len(&self) -> usize {
        frame_count(&self.root)
    }

    pub(crate) fn get(&self, mut index: usize) -> Option<&VirtualRowFragment> {
        let mut node = self.root.as_deref()?;
        loop {
            let left = row_count(&node.left);
            if index < left {
                node = node.left.as_deref()?;
            } else if index == left {
                return Some(&node.fragment);
            } else {
                index -= left + 1;
                node = node.right.as_deref()?;
            }
        }
    }

    fn frame(&self, mut index: usize) -> Option<&Frame> {
        let mut node = self.root.as_deref()?;
        loop {
            let left = frame_count(&node.left);
            if index < left {
                node = node.left.as_deref()?;
                continue;
            }
            index -= left;
            if index < node.fragment.frames().len() {
                return node.fragment.frames().get(index);
            }
            index -= node.fragment.frames().len();
            node = node.right.as_deref()?;
        }
    }

    fn edited(
        &self,
        delta: crate::list::AppliedResidencyDelta,
        front: Vec<VirtualRowFragment>,
        back: Vec<VirtualRowFragment>,
    ) -> Self {
        assert!(
            !delta.is_reset(),
            "reset residency cannot edit a retained sequence"
        );
        assert_eq!(front.len(), delta.insert_front());
        assert_eq!(back.len(), delta.insert_back());
        let (_, after_front) = split_rows(self.root.clone(), delta.remove_front());
        let keep = row_count(&after_front).saturating_sub(delta.remove_back());
        let (middle, _) = split_rows(after_front, keep);
        let front = RowSequence::from_rows(front).root;
        let back = RowSequence::from_rows(back).root;
        Self {
            root: merge_rows(merge_rows(front, middle), back),
            identity: Arc::new(()),
        }
    }

    pub(crate) fn identity(&self) -> std::sync::Weak<()> {
        Arc::downgrade(&self.identity)
    }
}

fn row_count(node: &Option<Arc<RowNode>>) -> usize {
    node.as_ref().map_or(0, |node| node.rows)
}

fn frame_count(node: &Option<Arc<RowNode>>) -> usize {
    node.as_ref().map_or(0, |node| node.frames)
}

fn row_priority(key: crate::list::Key) -> u64 {
    crate::persistent::sequence_priority(key.value())
}

fn row_node(
    fragment: VirtualRowFragment,
    left: Option<Arc<RowNode>>,
    right: Option<Arc<RowNode>>,
) -> Arc<RowNode> {
    let rows = 1 + row_count(&left) + row_count(&right);
    let frames = fragment.frames().len() + frame_count(&left) + frame_count(&right);
    Arc::new(RowNode {
        priority: row_priority(fragment.key()),
        fragment,
        left,
        right,
        rows,
        frames,
    })
}

fn merge_rows(left: Option<Arc<RowNode>>, right: Option<Arc<RowNode>>) -> Option<Arc<RowNode>> {
    match (left, right) {
        (None, right) => right,
        (left, None) => left,
        (Some(left), Some(right)) if left.priority >= right.priority => {
            let merged = merge_rows(left.right.clone(), Some(right));
            Some(row_node(left.fragment.clone(), left.left.clone(), merged))
        }
        (Some(left), Some(right)) => {
            let merged = merge_rows(Some(left), right.left.clone());
            Some(row_node(
                right.fragment.clone(),
                merged,
                right.right.clone(),
            ))
        }
    }
}

fn split_rows(
    root: Option<Arc<RowNode>>,
    count: usize,
) -> (Option<Arc<RowNode>>, Option<Arc<RowNode>>) {
    let Some(root) = root else {
        return (None, None);
    };
    let left_count = row_count(&root.left);
    if count <= left_count {
        let (left, middle) = split_rows(root.left.clone(), count);
        let right = Some(row_node(root.fragment.clone(), middle, root.right.clone()));
        (left, right)
    } else {
        let (middle, right) = split_rows(root.right.clone(), count.saturating_sub(left_count + 1));
        let left = Some(row_node(root.fragment.clone(), root.left.clone(), middle));
        (left, right)
    }
}

impl<'a> Iterator for FrameIter<'a> {
    type Item = &'a Frame;

    fn next(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }
        let frame = self.frames.frame(self.front);
        self.front += 1;
        frame
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.back - self.front;
        (remaining, Some(remaining))
    }
}

impl DoubleEndedIterator for FrameIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }
        self.back -= 1;
        self.frames.frame(self.back)
    }
}

impl ExactSizeIterator for FrameIter<'_> {}

impl<'a> Iterator for VirtualRowIter<'a> {
    type Item = &'a VirtualRowFragment;

    fn next(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }
        let row = self.frames.virtual_row(self.front);
        self.front += 1;
        row
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.back - self.front;
        (remaining, Some(remaining))
    }
}

impl DoubleEndedIterator for VirtualRowIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }
        self.back -= 1;
        self.frames.virtual_row(self.back)
    }
}

impl ExactSizeIterator for VirtualRowIter<'_> {}

impl<'a> IntoIterator for &'a FrameList {
    type Item = &'a Frame;
    type IntoIter = FrameIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Index<usize> for FrameList {
    type Output = Frame;

    fn index(&self, index: usize) -> &Self::Output {
        self.frame(index).expect("frame index out of bounds")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScrollProjection {
    node: composition::tree::NodeId,
    target: interaction::Target,
    viewport: Viewport,
    geometry_space: ScrollGeometrySpace,
    layer_bounds: Rect,
    residency: ScrollResidency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScrollGeometrySpace {
    /// Descendant geometry was authored relative to the commit's resident
    /// property value and moves by the delta from that value.
    BaselineRelative,
    /// Descendant geometry was authored in stable scroll-content coordinates
    /// and moves by the complete submitted property value.
    ContentLocal,
}

#[derive(Clone)]
pub(crate) struct ResidencyDemand {
    target: interaction::Target,
    desired: interaction::Offset,
    preparation: interaction::Offset,
    virtual_lists: Vec<crate::list::Request>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScrollPropertyAcceptance {
    replenishment: Option<interaction::Offset>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ScrollResidency {
    Complete(Proof),
    Empty,
    Incomplete(IncompleteResidency),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IncompleteResidency {
    reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Proof {
    node: composition::tree::NodeId,
    target: interaction::Target,
    requested: Option<Requested>,
    rows: Vec<Row>,
    viewport: Viewport,
    baseline: interaction::Offset,
    bounds: Rect,
    accepted: Accepted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Requested {
    list: interaction::Id,
    range: Range<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Row {
    node: composition::tree::NodeId,
    list: interaction::Id,
    key: crate::list::Key,
    index: usize,
    rect: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Accepted {
    minimum: interaction::Offset,
    maximum: interaction::Offset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Rows {
    rows: Vec<Row>,
    bounds: Rect,
}

impl ScrollProjection {
    pub(crate) fn node(&self) -> composition::tree::NodeId {
        self.node
    }

    pub(crate) fn target(&self) -> &interaction::Target {
        &self.target
    }

    pub(crate) fn viewport(&self) -> Viewport {
        self.viewport
    }

    pub(crate) fn geometry_space(&self) -> ScrollGeometrySpace {
        self.geometry_space
    }

    #[cfg(any(test, feature = "renderer-debug"))]
    pub(crate) fn layer_bounds(&self) -> Rect {
        self.layer_bounds
    }

    pub(crate) fn resident_bounds(&self) -> Option<Rect> {
        match &self.residency {
            ScrollResidency::Complete(proof) => Some(proof.bounds),
            ScrollResidency::Empty | ScrollResidency::Incomplete(_) => None,
        }
    }

    pub(crate) fn is_scene_drawable(&self) -> bool {
        matches!(self.residency, ScrollResidency::Complete(_))
    }

    pub(crate) fn accepted_offsets(&self) -> Option<(interaction::Offset, interaction::Offset)> {
        match &self.residency {
            ScrollResidency::Complete(proof) => {
                Some((proof.accepted.minimum, proof.accepted.maximum))
            }
            ScrollResidency::Empty | ScrollResidency::Incomplete(_) => None,
        }
    }
}

impl IncompleteResidency {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl ResidencyDemand {
    pub(crate) fn target(&self) -> &interaction::Target {
        &self.target
    }

    pub(crate) fn desired(&self) -> interaction::Offset {
        self.desired
    }

    pub(crate) fn preparation(&self) -> interaction::Offset {
        self.preparation
    }

    pub(crate) fn virtual_lists(&self) -> &[crate::list::Request] {
        &self.virtual_lists
    }

    pub(crate) fn prepares_proactively(&self) -> bool {
        self.preparation != self.desired && !self.virtual_lists.is_empty()
    }
}

impl ScrollPropertyAcceptance {
    pub(crate) fn replenishment(self) -> Option<interaction::Offset> {
        self.replenishment
    }
}

impl Proof {
    fn new(
        node: composition::tree::NodeId,
        target: interaction::Target,
        requested: Option<Requested>,
        rows: Vec<Row>,
        viewport: Viewport,
        geometry_space: ScrollGeometrySpace,
        required: Rect,
        bounds: Rect,
    ) -> Option<Self> {
        if bounds.width() <= 0 || bounds.height() <= 0 || !contains_rect(bounds, required) {
            return None;
        }
        let baseline = viewport.resolved_scroll();
        let accepted = Accepted::for_resident(viewport, baseline, geometry_space, bounds)?;
        let proof = Self {
            node,
            target,
            requested,
            rows,
            viewport,
            baseline,
            bounds,
            accepted,
        };
        proof
            .accepts(node, &proof.target.clone(), baseline)
            .then_some(proof)
    }

    fn accepts(
        &self,
        node: composition::tree::NodeId,
        target: &interaction::Target,
        offset: interaction::Offset,
    ) -> bool {
        self.node == node
            && &self.target == target
            && self.viewport.resolve(offset) == offset
            && self.accepted.contains(offset)
            && self.requested.as_ref().is_none_or(|requested| {
                self.rows.len() == requested.range.len()
                    && self.rows.iter().enumerate().all(|(position, row)| {
                        row.list == requested.list
                            && row.index == requested.range.start.saturating_add(position)
                            && row.rect.width() > 0
                            && row.rect.height() > 0
                    })
            })
            && self.viewport.resolved_scroll() == self.baseline
    }
}

impl Accepted {
    fn for_resident(
        viewport: Viewport,
        baseline: interaction::Offset,
        geometry_space: ScrollGeometrySpace,
        bounds: Rect,
    ) -> Option<Self> {
        let rect = viewport.rect();
        let required = viewport.viewport_content_coverage()?;
        let content = viewport.content();
        let maximum = viewport.max_scroll();
        let geometry_baseline = match geometry_space {
            ScrollGeometrySpace::BaselineRelative => baseline,
            ScrollGeometrySpace::ContentLocal => interaction::Offset::default(),
        };
        let (minimum_x, maximum_x) = accepted_axis(
            bounds.x(),
            bounds.right(),
            rect.x(),
            required.x(),
            required.right(),
            content.width(),
            geometry_baseline.x(),
            maximum.x(),
        )?;
        let (minimum_y, maximum_y) = accepted_axis(
            bounds.y(),
            bounds.bottom(),
            rect.y(),
            required.y(),
            required.bottom(),
            content.height(),
            geometry_baseline.y(),
            maximum.y(),
        )?;
        Some(Self {
            minimum: interaction::Offset::new(minimum_x, minimum_y),
            maximum: interaction::Offset::new(maximum_x, maximum_y),
        })
    }

    fn contains(self, offset: interaction::Offset) -> bool {
        offset.lies_within(self.minimum, self.maximum)
    }

    fn replenishment(
        self,
        viewport: Viewport,
        previous: interaction::Offset,
        offset: interaction::Offset,
    ) -> Option<interaction::Offset> {
        let legal = viewport.max_scroll();
        let rect = viewport.rect();
        let x = accepted_axis_replenishment(
            self.minimum.x(),
            self.maximum.x(),
            legal.x(),
            rect.width(),
            previous.x(),
            offset.x(),
        );
        let y = accepted_axis_replenishment(
            self.minimum.y(),
            self.maximum.y(),
            legal.y(),
            rect.height(),
            previous.y(),
            offset.y(),
        );
        (x.is_some() || y.is_some()).then(|| {
            let offset = x.map_or(offset, |x| offset.with_x(x));
            y.map_or(offset, |y| offset.with_y(y))
        })
    }
}

fn accepted_axis_replenishment(
    accepted_minimum: i32,
    accepted_maximum: i32,
    legal_maximum: i32,
    viewport_extent: i32,
    previous: i32,
    offset: i32,
) -> Option<i32> {
    let threshold = viewport_extent.max(1);
    if offset > previous && accepted_maximum < legal_maximum {
        (accepted_maximum.saturating_sub(offset) <= threshold).then(|| {
            if offset == accepted_maximum {
                accepted_maximum.saturating_add(1).min(legal_maximum)
            } else {
                accepted_maximum
            }
        })
    } else if offset < previous && accepted_minimum > 0 {
        (offset.saturating_sub(accepted_minimum) <= threshold).then(|| {
            if offset == accepted_minimum {
                accepted_minimum.saturating_sub(1)
            } else {
                accepted_minimum
            }
        })
    } else {
        None
    }
}

#[allow(clippy::too_many_arguments)]
fn accepted_axis(
    resident_start: i32,
    resident_end: i32,
    viewport_start: i32,
    visible_start: i32,
    visible_end: i32,
    content_extent: i32,
    baseline: i32,
    maximum: i32,
) -> Option<(i32, i32)> {
    let logical_start = resident_start
        .saturating_sub(viewport_start)
        .saturating_add(baseline);
    let logical_end = resident_end
        .saturating_sub(viewport_start)
        .saturating_add(baseline);
    let visible_start = visible_start.saturating_sub(viewport_start);
    let visible_end = visible_end.saturating_sub(viewport_start);
    let minimum = if logical_start <= 0 {
        0
    } else {
        logical_start
            .saturating_sub(visible_start)
            .clamp(0, maximum)
    };
    let maximum = if logical_end >= content_extent {
        maximum
    } else {
        logical_end.saturating_sub(visible_end).clamp(0, maximum)
    };
    (minimum <= maximum).then_some((minimum, maximum))
}

fn union_rect(left: Rect, right: Rect) -> Rect {
    let x = left.x().min(right.x());
    let y = left.y().min(right.y());
    Rect::new(
        x,
        y,
        left.right().max(right.right()).saturating_sub(x),
        left.bottom().max(right.bottom()).saturating_sub(y),
    )
}

fn translate_rect_by_offset(rect: Rect, offset: interaction::Offset) -> Rect {
    Rect::new(
        rect.x().saturating_add(offset.x()),
        rect.y().saturating_add(offset.y()),
        rect.width(),
        rect.height(),
    )
}

fn intersect_rect(left: Rect, right: Rect) -> Option<Rect> {
    let x = left.x().max(right.x());
    let y = left.y().max(right.y());
    let right_edge = left.right().min(right.right());
    let bottom_edge = left.bottom().min(right.bottom());
    (right_edge > x && bottom_edge > y).then(|| {
        Rect::new(
            x,
            y,
            right_edge.saturating_sub(x),
            bottom_edge.saturating_sub(y),
        )
    })
}

fn contains_rect(outer: Rect, inner: Rect) -> bool {
    outer.x() <= inner.x()
        && outer.y() <= inner.y()
        && outer.right() >= inner.right()
        && outer.bottom() >= inner.bottom()
}

impl Layout {
    #[cfg(test)]
    pub(crate) fn compose(view: &view::View, size: Size, engine: &mut Engine) -> Self {
        Self::compose_with_theme(view, size, engine, &Theme::default())
    }

    #[cfg(test)]
    pub(crate) fn compose_with_theme(
        view: &view::View,
        size: Size,
        engine: &mut Engine,
        theme: &Theme,
    ) -> Self {
        Self::compose_with_theme_at(
            view,
            size,
            engine,
            theme,
            animation::Frame::new(std::time::Instant::now()),
            keymap::Profile::default(),
        )
    }

    #[cfg(test)]
    pub(crate) fn compose_with_theme_at(
        view: &view::View,
        size: Size,
        engine: &mut Engine,
        theme: &Theme,
        frame: animation::Frame,
        keymap: keymap::Profile,
    ) -> Self {
        let tree = tree::Layout::new(view);
        let changes = tree::Changes::default();
        Self::compose_view_tree_with_theme_at(
            view,
            tree.root(),
            &changes,
            size,
            engine,
            theme,
            frame,
            keymap,
            PopupSurfaces::InFrame,
            None,
        )
    }

    #[cfg(test)]
    pub(crate) fn compose_composition_with_theme_at(
        composition: &composition::Composition,
        size: Size,
        engine: &mut Engine,
        theme: &Theme,
        frame: animation::Frame,
        keymap: keymap::Profile,
        popup_surfaces: PopupSurfaces,
    ) -> Self {
        Self::compose_composition_with_theme_at_reusing(
            composition,
            size,
            engine,
            theme,
            frame,
            keymap,
            popup_surfaces,
            None,
        )
    }

    pub(crate) fn compose_composition_with_theme_at_reusing(
        composition: &composition::Composition,
        size: Size,
        engine: &mut Engine,
        theme: &Theme,
        frame: animation::Frame,
        keymap: keymap::Profile,
        popup_surfaces: PopupSurfaces,
        previous: Option<&Self>,
    ) -> Self {
        Self::compose_view_tree_with_theme_at(
            composition.view(),
            composition.tree().root(),
            composition.changes(),
            size,
            engine,
            theme,
            frame,
            keymap,
            popup_surfaces,
            previous,
        )
    }

    fn compose_view_tree_with_theme_at(
        view: &view::View,
        root: &tree::Node,
        changes: &tree::Changes,
        size: Size,
        engine: &mut Engine,
        theme: &Theme,
        frame: animation::Frame,
        keymap: keymap::Profile,
        popup_surfaces: PopupSurfaces,
        previous: Option<&Self>,
    ) -> Self {
        let size = size.sanitized();
        let composed = algorithm::compose_frames(
            view.root(),
            root,
            changes,
            previous,
            size,
            engine,
            theme,
            frame,
            keymap,
        );
        let frames = if composed.row_sequences.is_empty() {
            FrameList::from_flat(composed.frames, &composed.virtual_row_fragments)
        } else {
            FrameList::from_sparse(composed.frames, composed.row_sequences)
        };
        let chrome = chrome::project(&frames, theme);
        let table_tracks = table::project(&frames);
        let scene_scroll_paths = project_scene_scroll_paths(&frames);
        let scroll_projections =
            project_scroll_projections(&frames, &table_tracks, &scene_scroll_paths);
        let virtual_list_requests = frames
            .iter()
            .filter_map(Frame::virtual_list_request)
            .cloned()
            .collect();
        let native_popup_owners = match popup_surfaces {
            PopupSurfaces::InFrame => HashMap::new(),
            PopupSurfaces::Native => {
                let panels = root_floating_panels(&frames)
                    .filter_map(|panel| Some((panel.target()?.element_id()?, panel)))
                    .collect::<Vec<_>>();
                frames
                    .iter()
                    .filter_map(|frame| {
                        panels
                            .iter()
                            .filter(|(_, panel)| {
                                frame.node_id() == panel.node_id() || frame.is_descendant_of(panel)
                            })
                            .max_by_key(|(_, panel)| panel.path_depth())
                            .map(|(id, _)| (frame.node_id(), *id))
                    })
                    .collect()
            }
        };

        Self {
            size,
            frames,
            chrome,
            table_tracks,
            scene_scroll_paths,
            scroll_projections,
            virtual_list_requests,
            native_popup_owners,
            residency_deltas: changes.residency_deltas().to_vec(),
            residency_predecessors: changes
                .residency_deltas()
                .iter()
                .filter(|delta| !delta.is_reset())
                .filter_map(|delta| {
                    previous?
                        .virtual_row_sequence(delta.list())
                        .cloned()
                        .map(|rows| (delta.list(), rows))
                })
                .collect(),
            frames_constructed: composed.frames_constructed,
            frames_reused: composed.frames_reused,
        }
    }

    pub(crate) fn size(&self) -> Size {
        self.size
    }

    pub(crate) fn frames(&self) -> &FrameList {
        &self.frames
    }

    pub(crate) fn virtual_row_fragments(&self) -> VirtualRowIter<'_> {
        self.frames.virtual_rows()
    }

    /// Returns the structurally shared row sequence for one virtual list.
    /// Looking up a list visits only the bounded ordinary/virtual segment
    /// spine; indexing a row then follows the persistent treap path.
    pub(crate) fn virtual_row_sequence(&self, list: interaction::Id) -> Option<&RowSequence> {
        self.frames
            .segments
            .iter()
            .find_map(|segment| match segment {
                FrameSegment::VirtualRows(rows)
                    if rows.get(0).is_some_and(|row| row.list() == list) =>
                {
                    Some(rows)
                }
                FrameSegment::Ordinary(_) | FrameSegment::VirtualRows(_) => None,
            })
    }

    pub(crate) fn virtual_row_predecessor(&self, list: interaction::Id) -> Option<&RowSequence> {
        self.residency_predecessors.get(&list)
    }

    pub(crate) fn ordinary_frames(&self) -> impl Iterator<Item = &Frame> {
        self.frames
            .segments
            .iter()
            .filter_map(|segment| match segment {
                FrameSegment::Ordinary(frames) => Some(frames.iter()),
                FrameSegment::VirtualRows(_) => None,
            })
            .flatten()
    }

    pub(crate) fn residency_deltas(&self) -> &[crate::list::AppliedResidencyDelta] {
        &self.residency_deltas
    }

    pub(crate) fn frames_constructed(&self) -> usize {
        self.frames_constructed
    }

    pub(crate) fn frames_reused(&self) -> usize {
        self.frames_reused
    }

    pub(crate) fn overflow_tip_for_target(&self, target: &interaction::Target) -> Option<&str> {
        self.frames
            .iter()
            .find(|frame| frame.target() == Some(target))
            .and_then(Frame::overflow_tip)
    }

    pub(crate) fn frame_for_node(&self, node: composition::tree::NodeId) -> Option<&Frame> {
        self.frames.iter().find(|frame| frame.node_id() == node)
    }

    pub(crate) fn scroll_projections(&self) -> &[ScrollProjection] {
        &self.scroll_projections
    }

    pub(crate) fn scene_residency_is_complete(&self) -> bool {
        self.scroll_projections
            .iter()
            .all(|projection| !matches!(projection.residency, ScrollResidency::Incomplete(_)))
    }

    pub(crate) fn scene_residency_incompleteness(&self) -> Vec<String> {
        self.scroll_projections
            .iter()
            .filter_map(|projection| match &projection.residency {
                ScrollResidency::Incomplete(issue) => Some(
                    format!(
                        "node={:?},target={:?},viewport={:?},layer_bounds={:?},reason={}",
                        projection.node,
                        projection.target,
                        projection.viewport,
                        projection.layer_bounds,
                        issue.reason,
                    )
                    .replace(['\r', '\n'], " "),
                ),
                ScrollResidency::Complete(_) | ScrollResidency::Empty => None,
            })
            .collect()
    }

    pub(crate) fn scene_scroll_path(
        &self,
        node: composition::tree::NodeId,
    ) -> &[composition::tree::NodeId] {
        self.scene_scroll_paths
            .get(&node)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Fixed submitted coverage for a retained fragment. Content-local row
    /// bodies carry their viewport clip in local layout coordinates, so the
    /// outer commit clip is reconstructed from the owning viewport geometry,
    /// never from the current property value.
    pub(crate) fn scene_fragment_clip(&self, node: composition::tree::NodeId) -> Option<FrameClip> {
        let frame = self.frames.iter().find(|frame| frame.node_id() == node)?;
        let content_local = self.scene_scroll_path(node).iter().rev().find_map(|owner| {
            self.scroll_projections.iter().find(|projection| {
                projection.node == *owner
                    && projection.geometry_space == ScrollGeometrySpace::ContentLocal
            })
        });
        content_local.map_or_else(
            || frame.clip(),
            |projection| {
                let rounding = frame
                    .clip()
                    .map_or(scene::Rounding::none(), FrameClip::rounding);
                Some(FrameClip::rounded(
                    projection.viewport.visible_content(),
                    rounding,
                ))
            },
        )
    }

    /// A content-local body is clipped by the fixed fragment coverage after
    /// spatial projection. Baking the viewport clip into the body would move
    /// that clip with the row and reintroduce a property dependency.
    pub(crate) fn scene_body_clip(&self, node: composition::tree::NodeId) -> Option<FrameClip> {
        let frame = self.frames.iter().find(|frame| frame.node_id() == node)?;
        let content_local = self.scene_scroll_path(node).iter().any(|owner| {
            self.scroll_projections.iter().any(|projection| {
                projection.node == *owner
                    && projection.geometry_space == ScrollGeometrySpace::ContentLocal
            })
        });
        (!content_local).then(|| frame.clip()).flatten()
    }

    pub(crate) fn scene_scroll_path_is_drawable(&self, node: composition::tree::NodeId) -> bool {
        self.scene_scroll_path(node).iter().all(|owner| {
            let projection = self
                .scroll_projections
                .iter()
                .find(|projection| projection.node == *owner)
                .expect("every scroll ancestor must own one layout projection");
            match &projection.residency {
                ScrollResidency::Complete(_) => true,
                ScrollResidency::Empty => false,
                ScrollResidency::Incomplete(_) => {
                    panic!("incomplete scroll residency cannot enter scene painting")
                }
            }
        })
    }

    pub(crate) fn scene_scroll_node_is_drawable(&self, node: composition::tree::NodeId) -> bool {
        self.scroll_projections
            .iter()
            .any(|projection| projection.node == node && projection.is_scene_drawable())
    }

    pub(crate) fn scroll_property_acceptance(
        &self,
        target: &interaction::Target,
        previous: interaction::Offset,
        offset: interaction::Offset,
    ) -> Option<ScrollPropertyAcceptance> {
        let mut owns_changed_axis = false;
        let mut replenishment: Option<interaction::Offset> = None;
        for projection in self
            .scroll_projections()
            .iter()
            .filter(|projection| &projection.target == target)
        {
            let viewport = projection.viewport;
            let maximum = viewport.max_scroll();
            // Axis ownership is about the requested transition, not the
            // layout's immutable baseline. In particular, returning from an
            // unpresented out-of-residency intent to the resident baseline is
            // still a real accepted transition that must retire the obsolete
            // cold candidate.
            let changes_x = maximum.x() > 0
                && !offset.same_axis(previous, interaction::ScrollbarAxis::Horizontal);
            let changes_y = maximum.y() > 0
                && !offset.same_axis(previous, interaction::ScrollbarAxis::Vertical);
            if !changes_x && !changes_y {
                continue;
            }
            owns_changed_axis = true;
            let resolved = viewport.resolve(offset);
            if (changes_x && !resolved.same_axis(offset, interaction::ScrollbarAxis::Horizontal))
                || (changes_y && !resolved.same_axis(offset, interaction::ScrollbarAxis::Vertical))
            {
                return None;
            }
            let ScrollResidency::Complete(proof) = &projection.residency else {
                return None;
            };
            if !proof.accepts(projection.node, &projection.target, resolved) {
                return None;
            }
            if let Some(candidate) = proof.accepted.replenishment(viewport, previous, resolved) {
                replenishment = Some(replenishment.map_or(candidate, |mut current| {
                    for axis in [
                        interaction::ScrollbarAxis::Horizontal,
                        interaction::ScrollbarAxis::Vertical,
                    ] {
                        let candidate_direction = candidate.axis_cmp(resolved, axis);
                        let candidate_is_further = match candidate_direction {
                            std::cmp::Ordering::Greater => {
                                candidate.axis_cmp(current, axis).is_gt()
                            }
                            std::cmp::Ordering::Less => candidate.axis_cmp(current, axis).is_lt(),
                            std::cmp::Ordering::Equal => false,
                        };
                        if candidate_is_further {
                            current = current.with_axis_from(candidate, axis);
                        }
                    }
                    current
                }));
            }
        }
        owns_changed_axis.then_some(ScrollPropertyAcceptance { replenishment })
    }

    pub(crate) fn resolve_scroll_offset(
        &self,
        target: &interaction::Target,
        offset: interaction::Offset,
    ) -> interaction::Offset {
        let mut found = false;
        let mut maximum = interaction::Offset::default();
        for projection in self
            .scroll_projections
            .iter()
            .filter(|projection| &projection.target == target)
        {
            found = true;
            let candidate = projection.viewport.max_scroll();
            maximum = maximum.componentwise_max(candidate);
        }
        if found {
            offset.clamped(interaction::Offset::default(), maximum)
        } else {
            offset
        }
    }

    pub(crate) fn scroll_adjustment_geometry(
        &self,
        target: &interaction::Target,
    ) -> Option<(interaction::Offset, interaction::Offset)> {
        let mut found = false;
        let mut maximum = interaction::Offset::default();
        let mut page = interaction::Offset::default();
        for projection in self
            .scroll_projections
            .iter()
            .filter(|projection| &projection.target == target)
        {
            found = true;
            let viewport = projection.viewport;
            let candidate = viewport.max_scroll();
            maximum = maximum.componentwise_max(candidate);
            page = page.componentwise_max(interaction::Offset::new(
                viewport.rect().width(),
                viewport.rect().height(),
            ));
        }
        found.then_some((maximum, page))
    }

    pub(crate) fn virtual_resident_node_ids(&self) -> HashSet<composition::tree::NodeId> {
        let roots = self
            .frames
            .iter()
            .filter(|frame| frame.provided_row().is_some())
            .map(Frame::node_id)
            .collect::<HashSet<_>>();
        self.frames
            .iter()
            .filter(|frame| {
                roots.contains(&frame.node_id())
                    || self
                        .frames
                        .iter()
                        .any(|root| roots.contains(&root.node_id()) && frame.is_descendant_of(root))
            })
            .map(Frame::node_id)
            .collect()
    }

    pub(crate) fn residency_content_scroll_node_ids(&self) -> HashSet<composition::tree::NodeId> {
        self.scroll_projections
            .iter()
            .filter_map(|projection| {
                self.frames
                    .iter()
                    .find(|frame| frame.node_id() == projection.node)
                    .and_then(|frame| frame.text_area_layout())
                    .map(|_| projection.node)
            })
            .collect()
    }

    pub(crate) fn residency_demand(
        &self,
        target: &interaction::Target,
        offset: interaction::Offset,
    ) -> Option<ResidencyDemand> {
        self.residency_demand_for(target, offset, offset)
    }

    pub(crate) fn residency_replenishment(
        &self,
        target: &interaction::Target,
        desired: interaction::Offset,
        preparation: interaction::Offset,
    ) -> Option<ResidencyDemand> {
        self.residency_demand_for(target, desired, preparation)
    }

    fn residency_demand_for(
        &self,
        target: &interaction::Target,
        desired: interaction::Offset,
        preparation: interaction::Offset,
    ) -> Option<ResidencyDemand> {
        let proactive = desired != preparation;
        let projections = self
            .scroll_projections
            .iter()
            .filter(|projection| &projection.target == target)
            .collect::<Vec<_>>();
        if projections.is_empty() {
            return None;
        }
        let mut seen = HashSet::new();
        let virtual_lists = projections
            .into_iter()
            .filter_map(|projection| {
                let frame = self
                    .frames
                    .iter()
                    .find(|frame| frame.node_id() == projection.node)?;
                if proactive {
                    frame.virtual_list_request_for_offset(preparation)
                } else {
                    frame.virtual_list_required_request_for_offset(preparation)
                }
            })
            .filter(|request| seen.insert(request.id()))
            .collect();
        Some(ResidencyDemand {
            target: target.clone(),
            desired,
            preparation,
            virtual_lists,
        })
    }

    pub(crate) fn frame_for_focus(&self, focus: session::Focus) -> Option<&Frame> {
        self.frames.iter().find(|frame| {
            frame
                .target()
                .is_some_and(|target| focus.matches_target(target))
        })
    }

    pub(crate) fn chrome(&self) -> &[Chrome] {
        &self.chrome
    }

    pub(crate) fn table_tracks(&self) -> &[table::Track] {
        &self.table_tracks
    }

    pub(crate) fn virtual_list_requests(&self) -> &[crate::list::Request] {
        &self.virtual_list_requests
    }

    /// The floating panels that own independent presentation surfaces.
    ///
    /// Nested floating panels remain content of their nearest root panel; this
    /// census is shared by surface ownership and overlay scene extraction so
    /// interaction and presentation cannot disagree about the boundary.
    pub(crate) fn root_floating_panels(&self) -> impl Iterator<Item = &Frame> {
        self.frames
            .iter()
            .filter(|frame| frame.role() == view::Role::FloatingPanel)
            .filter(|frame| {
                !self.frames.iter().any(|candidate| {
                    candidate.role() == view::Role::FloatingPanel
                        && frame.is_descendant_of(candidate)
                })
            })
    }

    pub(crate) fn virtual_list_page(&self, id: interaction::Id, row_height: i32) -> Option<usize> {
        self.frames
            .iter()
            .find(|frame| {
                frame.role() == view::Role::VirtualList
                    && frame.target().and_then(interaction::Target::element_id) == Some(id)
            })
            .and_then(Frame::viewport)
            .map(|viewport| {
                (viewport.visible_content().height().max(1) as usize / row_height.max(1) as usize)
                    .max(1)
            })
    }

    pub(crate) fn table_scroll_target(
        &self,
        table: interaction::Id,
    ) -> Option<interaction::Target> {
        self.frames
            .iter()
            .find(|frame| {
                frame
                    .table_projection()
                    .is_some_and(|projection| projection.table() == table)
            })
            .and_then(Frame::target)
            .cloned()
    }

    pub(crate) fn is_table_scroll_target(&self, target: &interaction::Target) -> bool {
        self.frames
            .iter()
            .any(|frame| frame.target() == Some(target) && frame.table_projection().is_some())
    }

    pub(crate) fn text_caret(&self) -> Option<(composition::tree::NodeId, Rect)> {
        self.frames
            .iter()
            .find_map(|frame| frame.text_caret_rect().map(|area| (frame.node_id(), area)))
    }

    #[cfg(test)]
    pub(crate) fn hit_test(&self, point: Point) -> Option<Hit> {
        self.hit_test_on_surface(point, crate::popup::Surface::Parent)
    }

    pub(crate) fn hit_test_on_surface(
        &self,
        point: Point,
        surface: crate::popup::Surface,
    ) -> Option<Hit> {
        self.hit_test_on_surface_projected(point, surface, &|_, point| Some((point, [0, 0])))
    }

    pub(crate) fn hit_test_on_surface_projected(
        &self,
        point: Point,
        surface: crate::popup::Surface,
        project: &impl Fn(composition::tree::NodeId, Point) -> Option<(Point, [i32; 2])>,
    ) -> Option<Hit> {
        let table_cell = self
            .frames
            .iter()
            .rev()
            .find_map(|frame| {
                let (point, _) = project(frame.node_id(), point)?;
                (self.surface_accepts_frame(surface, frame)
                    && frame.table_cell().is_some()
                    && frame.rect().contains(point)
                    && frame.clip_contains(point))
                .then(|| frame.table_cell())
            })
            .flatten();
        if let Some((owner, chrome)) = self
            .chrome
            .iter()
            .rev()
            .filter(|chrome| chrome.accepts_hit(point))
            .find_map(|chrome| {
                let owner = self.frames.iter().rev().find(|frame| {
                    frame.node_id() == chrome.owner() && self.surface_accepts_frame(surface, frame)
                })?;
                Some((owner, chrome))
            })
        {
            return Some(Hit::chrome(owner.clone(), chrome.clone()).with_table_cell(table_cell));
        }

        if let Some((track, translation)) = self.table_tracks.iter().rev().find_map(|track| {
            let (point, translation) = project(track.owner_node(), point)?;
            track
                .accepts_resize_hit(point)
                .then_some((track, translation))
        }) {
            let header = self.frames.iter().find(|frame| {
                Some(frame.node_id()) == track.header_node()
                    && self.surface_accepts_frame(surface, frame)
            })?;
            return Some(
                Hit::table_divider(header.clone(), track.divider_target()?)
                    .with_translation(translation)
                    .with_table_cell(table_cell),
            );
        }

        if let Some((frame, target, translation)) = self.frames.iter().rev().find_map(|frame| {
            let (point, translation) = project(frame.node_id(), point)?;
            (self.surface_accepts_frame(surface, frame)
                && frame
                    .input_indicator_rect()
                    .is_some_and(|rect| rect.contains(point))
                && frame.clip_contains(point))
            .then(|| Some((frame, frame.input_indicator_target()?, translation)))
            .flatten()
        }) {
            return Some(
                Hit::indicator(frame.clone(), target)
                    .with_translation(translation)
                    .with_table_cell(table_cell),
            );
        }

        self.frames
            .iter()
            .rev()
            .find_map(|frame| {
                let (point, translation) = project(frame.node_id(), point)?;
                (self.surface_accepts_frame(surface, frame) && frame.accepts_hit(point))
                    .then(|| Hit::new(frame.clone()).with_translation(translation))
            })
            .map(|hit| hit.with_table_cell(table_cell))
    }

    /// Returns the deepest laid-out node under a point, including inert
    /// display nodes that ordinary activation hit testing intentionally skips.
    #[cfg(test)]
    pub(crate) fn context_node_at(&self, point: Point) -> Option<composition::tree::NodeId> {
        self.context_node_at_surface(point, crate::popup::Surface::Parent)
    }

    #[cfg(test)]
    pub(crate) fn context_node_at_surface(
        &self,
        point: Point,
        surface: crate::popup::Surface,
    ) -> Option<composition::tree::NodeId> {
        self.context_node_at_surface_projected(point, surface, &|_, point| Some(point))
    }

    pub(crate) fn context_node_at_surface_projected(
        &self,
        point: Point,
        surface: crate::popup::Surface,
        project: &impl Fn(composition::tree::NodeId, Point) -> Option<Point>,
    ) -> Option<composition::tree::NodeId> {
        self.frames
            .iter()
            .rev()
            .find(|frame| {
                let Some(point) = project(frame.node_id(), point) else {
                    return false;
                };
                self.surface_accepts_frame(surface, frame)
                    && frame.rect().contains(point)
                    && frame.clip_contains(point)
            })
            .map(Frame::node_id)
    }

    pub(crate) fn context_available_for_node(
        &self,
        node: composition::tree::NodeId,
    ) -> Option<Rect> {
        let frame = self.frame_for_node(node)?;
        Some(
            frame
                .clip()
                .map(|clip| clip.rect())
                .unwrap_or_else(|| Rect::from_size(self.size)),
        )
    }

    pub(crate) fn drag_action_for_target_projected(
        &self,
        target: &interaction::Target,
        point: Point,
        engine: &mut Engine,
        project: &impl Fn(composition::tree::NodeId, Point) -> Option<Point>,
    ) -> Option<(view::Role, Option<view::Action>)> {
        if let Some(chrome) = self.chrome.iter().find(|chrome| chrome.target() == target) {
            return Some((
                view::Role::Scroll,
                Some(view::Action::scroll_to(
                    chrome.scroll_target().clone(),
                    chrome.scroll_offset_at(point),
                    chrome.axis(),
                )),
            ));
        }

        if let Some(track) = self
            .table_tracks
            .iter()
            .find(|track| track.divider_target().as_ref() == Some(target))
        {
            let point = project(track.owner_node(), point)?;
            return Some((view::Role::Label, track.resize_action_at(point)));
        }

        self.frames
            .iter()
            .find(|frame| frame.target() == Some(target))
            .map(|frame| {
                let point = project(frame.node_id(), point);
                (
                    frame.role(),
                    point.and_then(|point| frame.drag_action_at_with_engine(point, engine)),
                )
            })
    }

    #[cfg(any(test, feature = "renderer-debug"))]
    pub(crate) fn scroll_target_at(
        &self,
        point: Point,
        delta: interaction::Delta,
    ) -> Option<interaction::Target> {
        self.scroll_target_at_surface(point, delta, crate::popup::Surface::Parent)
    }

    #[cfg(any(test, feature = "renderer-debug"))]
    pub(crate) fn scroll_target_at_surface(
        &self,
        point: Point,
        delta: interaction::Delta,
        surface: crate::popup::Surface,
    ) -> Option<interaction::Target> {
        self.scroll_target_at_surface_projected(
            point,
            delta,
            surface,
            &|_, point| Some(point),
            &|_, viewport| viewport.resolved_scroll(),
        )
    }

    pub(crate) fn scroll_target_at_surface_projected(
        &self,
        point: Point,
        delta: interaction::Delta,
        surface: crate::popup::Surface,
        project: &impl Fn(composition::tree::NodeId, Point) -> Option<Point>,
        offset: &impl Fn(&interaction::Target, Viewport) -> interaction::Offset,
    ) -> Option<interaction::Target> {
        self.frames
            .iter()
            .rev()
            .find(|frame| {
                let Some(point) = project(frame.node_id(), point) else {
                    return false;
                };
                self.surface_accepts_frame(surface, frame)
                    && frame.viewport().is_some_and(|viewport| {
                        viewport.rect().contains(point)
                            && frame.clip_contains(point)
                            && frame.target().is_some_and(|target| {
                                viewport.can_consume_from(offset(target, viewport), delta)
                            })
                    })
            })
            .and_then(Frame::target)
            .cloned()
    }

    pub(crate) fn scroll_target_chain_at_surface_projected(
        &self,
        point: Point,
        surface: crate::popup::Surface,
        project: &impl Fn(composition::tree::NodeId, Point) -> Option<Point>,
    ) -> Vec<interaction::Target> {
        let accepts_point = |frame: &Frame| {
            let Some(point) = project(frame.node_id(), point) else {
                return false;
            };
            self.surface_accepts_frame(surface, frame)
                && frame.viewport().is_some_and(|viewport| {
                    viewport.rect().contains(point) && frame.clip_contains(point)
                })
                && frame.target().is_some()
        };
        let Some(deepest) = self.frames.iter().rev().find(|frame| accepts_point(frame)) else {
            return Vec::new();
        };
        let mut frames = self
            .frames
            .iter()
            .filter(|frame| {
                accepts_point(frame)
                    && (frame.node_id() == deepest.node_id() || deepest.is_descendant_of(frame))
            })
            .collect::<Vec<_>>();
        frames.sort_by_key(|frame| std::cmp::Reverse(frame.path_depth()));

        let mut targets = Vec::new();
        for target in frames.into_iter().filter_map(Frame::target) {
            if !targets.contains(target) {
                targets.push(target.clone());
            }
        }
        targets
    }

    pub(crate) fn scroll_target_chain_for_focus(
        &self,
        focus: session::Focus,
        axis: interaction::ScrollbarAxis,
    ) -> Vec<(interaction::Target, crate::scroll::Direction)> {
        let Some(descendant) = self.frame_for_focus(focus) else {
            return Vec::new();
        };
        let mut frames = self
            .frames
            .iter()
            .filter(|frame| {
                (frame.node_id() == descendant.node_id() || descendant.is_descendant_of(frame))
                    && frame.target().is_some()
                    && frame.viewport().is_some_and(|viewport| match axis {
                        interaction::ScrollbarAxis::Horizontal => viewport.max_scroll().x() > 0,
                        interaction::ScrollbarAxis::Vertical => viewport.max_scroll().y() > 0,
                    })
            })
            .collect::<Vec<_>>();
        frames.sort_by_key(|frame| std::cmp::Reverse(frame.path_depth()));
        let mut targets = Vec::new();
        for frame in frames {
            let Some(target) = frame.target() else {
                continue;
            };
            if targets.iter().any(|(current, _)| current == target) {
                continue;
            }
            let direction = frame
                .scroll_container_layout()
                .map_or(crate::scroll::Direction::LeftToRight, |container| {
                    container.direction()
                });
            targets.push((target.clone(), direction));
        }
        targets
    }

    pub(crate) fn scroll_direction_for_target(
        &self,
        target: &interaction::Target,
    ) -> crate::scroll::Direction {
        self.frames
            .iter()
            .find(|frame| frame.target() == Some(target))
            .and_then(Frame::scroll_container_layout)
            .map_or(crate::scroll::Direction::LeftToRight, |container| {
                container.direction()
            })
    }

    #[cfg(any(test, feature = "renderer-debug"))]
    #[allow(dead_code)]
    pub(crate) fn scroll_target_chain_at(&self, point: Point) -> Vec<interaction::Target> {
        self.scroll_target_chain_at_surface_projected(
            point,
            crate::popup::Surface::Parent,
            &|_, point| Some(point),
        )
    }

    fn surface_accepts_frame(&self, surface: crate::popup::Surface, frame: &Frame) -> bool {
        let owner = self.native_popup_owner(frame);
        match surface {
            crate::popup::Surface::Parent => owner.is_none(),
            crate::popup::Surface::Native(id) => owner == Some(id),
        }
    }

    fn native_popup_owner(&self, frame: &Frame) -> Option<interaction::Id> {
        self.native_popup_owners.get(&frame.node_id()).copied()
    }

    pub(crate) fn reveal_offsets_for_descendant_chain(
        &self,
        viewport_target: Option<&interaction::Target>,
        margin: i32,
        mut accepts_descendant: impl FnMut(&Frame) -> bool,
    ) -> Vec<(interaction::Target, interaction::Offset)> {
        if let Some(viewport_target) = viewport_target {
            let mut found = false;
            let mut resolved = interaction::Offset::default();
            for viewport_frame in self
                .frames
                .iter()
                .filter(|frame| frame.target() == Some(viewport_target))
            {
                let Some(viewport) = viewport_frame.viewport() else {
                    continue;
                };
                let Some(descendant) = self.frames.iter().find(|frame| {
                    frame.is_descendant_of(viewport_frame) && accepts_descendant(frame)
                }) else {
                    continue;
                };
                found = true;
                let candidate = viewport.reveal_rect(descendant.rect(), margin);
                let maximum = viewport.max_scroll();
                resolved = interaction::Offset::new(
                    if maximum.x() > 0 {
                        candidate.x()
                    } else {
                        resolved.x()
                    },
                    if maximum.y() > 0 {
                        candidate.y()
                    } else {
                        resolved.y()
                    },
                );
            }
            return found
                .then(|| vec![(viewport_target.clone(), resolved)])
                .unwrap_or_default();
        }

        let descendant = self.frames.iter().find(|frame| accepts_descendant(frame));
        let Some(descendant) = descendant else {
            return Vec::new();
        };
        let mut ancestors = self
            .frames
            .iter()
            .filter(|frame| {
                frame.viewport().is_some()
                    && frame.target().is_some()
                    && frame.is_eager_scroll_container()
                    && (frame.node_id() == descendant.node_id()
                        || descendant.is_descendant_of(frame))
            })
            .collect::<Vec<_>>();
        ancestors.sort_by_key(|frame| std::cmp::Reverse(frame.path_depth()));

        let mut rect = descendant.rect();
        let mut targets = Vec::new();
        for frame in ancestors {
            let Some(viewport) = frame.viewport() else {
                continue;
            };
            let Some(target) = frame.target() else {
                continue;
            };
            if targets.iter().any(|(current, _)| current == target) {
                continue;
            }
            let current = viewport.resolved_scroll();
            let offset = viewport.reveal_rect(rect, margin);
            rect = Rect::new(
                rect.x()
                    .saturating_add(current.x().saturating_sub(offset.x())),
                rect.y()
                    .saturating_add(current.y().saturating_sub(offset.y())),
                rect.width(),
                rect.height(),
            );
            targets.push((target.clone(), offset));
        }
        targets
    }

    pub(crate) fn reveal_offsets_for_focus(
        &self,
        focus: session::Focus,
        margin: i32,
    ) -> Vec<(interaction::Target, interaction::Offset)> {
        self.reveal_offsets_for_descendant_chain(None, margin, |frame| {
            frame
                .target()
                .is_some_and(|target| focus.matches_target(target))
        })
    }

    #[cfg(test)]
    pub(crate) fn find_role(&self, role: view::Role) -> Vec<&Frame> {
        self.frames
            .iter()
            .filter(|frame| frame.role() == role)
            .collect()
    }
}

fn project_scene_scroll_paths(
    frames: &FrameList,
) -> HashMap<composition::tree::NodeId, Vec<composition::tree::NodeId>> {
    let by_node = frames
        .iter()
        .map(|frame| (frame.node_id(), frame))
        .collect::<HashMap<_, _>>();
    frames
        .iter()
        .map(|frame| {
            let mut ancestry = Vec::new();
            let mut parent = frame.parent();
            while let Some(id) = parent {
                let Some(frame) = by_node.get(&id) else {
                    break;
                };
                if frame.property_scroll_viewport().is_some() {
                    ancestry.push(id);
                }
                parent = frame.parent();
            }
            ancestry.reverse();
            (frame.node_id(), ancestry)
        })
        .collect()
}

fn project_scroll_projections(
    frames: &FrameList,
    table_tracks: &[table::Track],
    scene_scroll_paths: &HashMap<composition::tree::NodeId, Vec<composition::tree::NodeId>>,
) -> Vec<ScrollProjection> {
    let nearest_scroll = |node| {
        scene_scroll_paths
            .get(&node)
            .and_then(|ancestry| ancestry.last())
            .copied()
    };
    let mut descendant_frames = HashMap::<composition::tree::NodeId, Vec<&Frame>>::new();
    for frame in frames {
        if let Some(owner) = nearest_scroll(frame.node_id()) {
            descendant_frames.entry(owner).or_default().push(frame);
        }
    }
    let mut descendant_tracks = HashMap::<composition::tree::NodeId, Vec<&table::Track>>::new();
    for track in table_tracks {
        if let Some(owner) = nearest_scroll(track.owner_node()) {
            descendant_tracks.entry(owner).or_default().push(track);
        }
    }
    let no_frames = Vec::new();
    let no_tracks = Vec::new();
    frames
        .iter()
        .filter_map(|frame| {
            let viewport = frame.property_scroll_viewport()?;
            let target = frame.target()?.clone();
            let node = frame.node_id();
            let geometry_space = frame.scroll_geometry_space();
            let (layer_bounds, residency) = scroll_layer_geometry(
                frame,
                descendant_frames.get(&node).unwrap_or(&no_frames),
                descendant_tracks.get(&node).unwrap_or(&no_tracks),
                viewport,
                geometry_space,
            );
            Some(ScrollProjection {
                node,
                target,
                viewport,
                geometry_space,
                layer_bounds,
                residency,
            })
        })
        .collect()
}

fn scroll_layer_geometry(
    owner_frame: &Frame,
    descendant_frames: &[&Frame],
    descendant_tracks: &[&table::Track],
    viewport: Viewport,
    geometry_space: ScrollGeometrySpace,
) -> (Rect, ScrollResidency) {
    let owner = owner_frame.node_id();
    let explicit_prepared_bounds = owner_frame.text_area_layout().is_some();
    let mut bounds = owner_frame.scroll_resident_bounds();
    for frame in descendant_frames {
        bounds = Some(union_rect(
            bounds.unwrap_or_else(|| frame.rect()),
            frame.rect(),
        ));
    }
    for track in descendant_tracks {
        bounds = Some(union_rect(
            bounds.unwrap_or_else(|| track.rule_rect()),
            track.rule_rect(),
        ));
    }

    let visible = viewport.visible_content();
    let currently_visible = visible.width() > 0 && visible.height() > 0;
    let Some(required) = viewport.viewport_content_coverage() else {
        return (visible, ScrollResidency::Empty);
    };
    let geometry_offset = match geometry_space {
        ScrollGeometrySpace::BaselineRelative => interaction::Offset::default(),
        ScrollGeometrySpace::ContentLocal => viewport.resolved_scroll(),
    };
    let visible = translate_rect_by_offset(visible, geometry_offset);
    let required = translate_rect_by_offset(required, geometry_offset);
    // Residency is the content owner's actual prepared runway. Capping this to a
    // fixed fraction of the viewport throws away ready pixels and forces a
    // candidate activation at the artificial boundary, which presents as an
    // end-of-input jump. Virtual lists and text surfaces already bound their
    // realization; ordinary scrolls retain the content they actually authored.
    let layer_bounds = if explicit_prepared_bounds {
        bounds.unwrap_or_else(|| Rect::new(visible.x(), visible.y(), 0, 0))
    } else {
        bounds.map_or(visible, |bounds| union_rect(bounds, visible))
    };
    let virtual_request = owner_frame.virtual_list_request();
    let residency = match virtual_request {
        Some(request) if !request.range().is_empty() => {
            let requested = request.range();
            let expected_keys = requested
                .clone()
                .map(|index| owner_frame.virtual_list_key_at(index))
                .collect::<Option<Vec<_>>>();
            let rows = descendant_frames
                .iter()
                .filter_map(|frame| {
                    let row = frame.provided_row()?;
                    (row.list() == request.id() && requested.contains(&row.index())).then_some(Row {
                        node: frame.node_id(),
                        list: row.list(),
                        key: row.key(),
                        index: row.index(),
                        rect: frame.rect(),
                    })
                })
                .collect::<Vec<_>>();
            match expected_keys {
                None => ScrollResidency::Incomplete(IncompleteResidency::new(format!(
                    "virtual-list {:?} omitted a key in requested range {:?}",
                    request.id(),
                    requested,
                ))),
                Some(expected_keys) => match exact_virtual_residency(
                    requested.clone(),
                    &expected_keys,
                    &rows,
                    required,
                    layer_bounds,
                ) {
                    Err(reason) => {
                        ScrollResidency::Incomplete(IncompleteResidency::new(reason))
                    }
                    Ok(rows) => match owner_frame.target().cloned() {
                        Some(target) => Proof::new(
                            owner,
                            target,
                            Some(Requested {
                                list: request.id(),
                                range: request.range(),
                            }),
                            rows.rows,
                            viewport,
                            geometry_space,
                            required,
                            rows.bounds,
                        )
                        .map_or_else(
                            || {
                                ScrollResidency::Incomplete(IncompleteResidency::new(format!(
                                    "virtual-list {:?} rows {:?} could not prove baseline {:?} within required {:?} and resident {:?}",
                                    request.id(),
                                    requested,
                                    viewport.resolved_scroll(),
                                    required,
                                    rows.bounds,
                                )))
                            },
                            ScrollResidency::Complete,
                        ),
                        None => ScrollResidency::Incomplete(IncompleteResidency::new(format!(
                            "scroll owner {owner:?} lost its target during virtual residency projection"
                        ))),
                    },
                },
            }
        }
        Some(request) => ScrollResidency::Incomplete(IncompleteResidency::new(format!(
            "virtual-list {:?} requested an empty range {:?}",
            request.id(),
            request.range(),
        ))),
        None => owner_frame
            .target()
            .cloned()
            .and_then(|target| {
                Proof::new(
                    owner,
                    target,
                    None,
                    Vec::new(),
                    viewport,
                    geometry_space,
                    required,
                    layer_bounds,
                )
            })
            .map_or_else(
                || {
                    ScrollResidency::Incomplete(IncompleteResidency::new(format!(
                        "ordinary scroll could not prove baseline {:?} within required {:?} and layer {:?}",
                        viewport.resolved_scroll(),
                        required,
                        layer_bounds,
                    )))
                },
                ScrollResidency::Complete,
            ),
    };

    let residency = if currently_visible || matches!(residency, ScrollResidency::Complete(_)) {
        residency
    } else {
        // A fully clipped nested viewport is drawable only when its prepared
        // content proves complete coverage of the viewport it can expose after
        // an ancestor property move. Incomplete hidden state remains absent
        // without blocking the current scene (for example, a captured virtual
        // row removed by its provider).
        ScrollResidency::Empty
    };

    (layer_bounds, residency)
}

fn exact_virtual_residency(
    requested: Range<usize>,
    expected_keys: &[crate::list::Key],
    rows: &[Row],
    required: Rect,
    layer_bounds: Rect,
) -> Result<Rows, String> {
    if expected_keys.len() != requested.len() {
        return Err(format!(
            "virtual requested range {:?} has {} expected keys",
            requested,
            expected_keys.len(),
        ));
    }
    let mut expected = requested.start;
    let mut previous = None::<Rect>;
    let mut bounds = None::<Rect>;
    let mut keys = HashSet::with_capacity(rows.len());
    let mut nodes = HashSet::with_capacity(rows.len());

    for row in rows.iter().copied() {
        let key = expected_keys.get(expected.saturating_sub(requested.start));
        if row.index != expected {
            return Err(format!(
                "virtual rows are not exact: expected index {expected}, observed {} in requested {:?}; provided_indices={:?}",
                row.index,
                requested,
                rows.iter().map(|row| row.index).collect::<Vec<_>>(),
            ));
        }
        if key != Some(&row.key) {
            return Err(format!(
                "virtual row {expected} key mismatch: expected {key:?}, observed {:?}",
                row.key,
            ));
        }
        if !keys.insert(row.key) {
            return Err(format!("virtual row {expected} repeated key {:?}", row.key));
        }
        if !nodes.insert(row.node) {
            return Err(format!(
                "virtual row {expected} repeated node {:?}",
                row.node
            ));
        }
        if row.rect.width() <= 0 || row.rect.height() <= 0 {
            return Err(format!(
                "virtual row {expected} has non-positive geometry {:?}",
                row.rect,
            ));
        }
        if row.rect.x() > required.x() || row.rect.right() < required.right() {
            return Err(format!(
                "virtual row {expected} width {:?} does not cover required {:?}",
                row.rect, required,
            ));
        }
        if let Some(previous) = previous
            && previous.bottom() != row.rect.y()
        {
            return Err(format!(
                "virtual row {expected} starts at {} after previous bottom {}; previous={:?},row={:?}",
                row.rect.y(),
                previous.bottom(),
                previous,
                row.rect,
            ));
        }
        expected = expected.saturating_add(1);
        previous = Some(row.rect);
        bounds = Some(bounds.map_or(row.rect, |bounds| union_rect(bounds, row.rect)));
    }

    if expected != requested.end {
        return Err(format!(
            "virtual requested range {:?} provided only {} exact rows; provided_indices={:?}",
            requested,
            rows.len(),
            rows.iter().map(|row| row.index).collect::<Vec<_>>(),
        ));
    }
    let bounds = bounds.ok_or_else(|| {
        format!(
            "virtual requested range {:?} produced no resident row bounds",
            requested,
        )
    })?;
    let resident = intersect_rect(bounds, layer_bounds).ok_or_else(|| {
        format!(
            "virtual row bounds {:?} do not intersect layer {:?}",
            bounds, layer_bounds,
        )
    })?;
    if !contains_rect(resident, required) {
        return Err(format!(
            "virtual resident bounds {:?} do not contain required {:?}; row_bounds={:?},layer_bounds={:?}",
            resident, required, bounds, layer_bounds,
        ));
    }
    Ok(Rows {
        rows: rows.to_vec(),
        bounds: resident,
    })
}

fn root_floating_panels(frames: &FrameList) -> impl Iterator<Item = &Frame> {
    frames
        .iter()
        .filter(|frame| frame.role() == view::Role::FloatingPanel)
        .filter(|frame| {
            !frames.iter().any(|candidate| {
                candidate.role() == view::Role::FloatingPanel && frame.is_descendant_of(candidate)
            })
        })
}

#[cfg(test)]
mod placement_tests {
    use super::*;

    fn virtual_row(index: usize, rect: Rect) -> Row {
        let mut identity = index as u64 + 1;
        Row {
            node: composition::tree::NodeId::layout(&mut identity),
            list: interaction::Id::from("test.virtual-list"),
            key: crate::list::Key::new(index as u64),
            index,
            rect,
        }
    }

    #[test]
    fn contextual_floating_panel_uses_the_shared_edge_solver() {
        let panel = view::Node::floating_panel("context")
            .with_panel_placement(
                crate::geometry::placement::Anchor::Point(Point::new(95, 75)),
                Rect::new(0, 0, 100, 80),
            )
            .with_style(
                view::Style::new()
                    .with_width(view::Dimension::fixed(30))
                    .with_height(view::Dimension::fixed(20)),
            );
        let view = view::View::new(view::Node::root().child(panel));
        let mut engine = Engine::new();
        let layout = Layout::compose(&view, Size::new(100, 80), &mut engine);
        let panel = layout
            .find_role(view::Role::FloatingPanel)
            .into_iter()
            .next()
            .expect("context panel should be laid out");

        assert_eq!(panel.rect(), Rect::new(65, 55, 30, 20));
        assert_eq!(
            panel
                .popup_placement()
                .expect("context panel should retain its placement request")
                .resolve(Rect::new(-100, -100, 300, 300)),
            Rect::new(95, 75, 30, 20)
        );
    }

    #[test]
    fn contextual_floating_panel_honors_nested_available_bounds() {
        let available = Rect::new(10, 10, 40, 30);
        let panel = view::Node::floating_panel("nested-context")
            .with_panel_placement(
                crate::geometry::placement::Anchor::Point(Point::new(48, 38)),
                available,
            )
            .with_style(
                view::Style::new()
                    .with_width(view::Dimension::fixed(30))
                    .with_height(view::Dimension::fixed(20)),
            );
        let view = view::View::new(view::Node::root().child(panel));
        let mut engine = Engine::new();
        let layout = Layout::compose(&view, Size::new(200, 160), &mut engine);
        let panel = layout
            .find_role(view::Role::FloatingPanel)
            .into_iter()
            .next()
            .expect("nested context panel should be laid out");

        assert_eq!(panel.rect(), Rect::new(18, 18, 30, 20));
        assert!(panel.rect().x() >= available.x());
        assert!(panel.rect().y() >= available.y());
        assert!(panel.rect().right() <= available.right());
        assert!(panel.rect().bottom() <= available.bottom());
    }

    #[test]
    fn virtual_residency_requires_exact_indices_and_gap_free_pixels() {
        let visible = Rect::new(0, 0, 100, 100);
        let layer = Rect::new(0, 0, 100, 100);
        let complete = [
            virtual_row(10, Rect::new(0, -20, 100, 30)),
            virtual_row(11, Rect::new(0, 10, 100, 30)),
            virtual_row(12, Rect::new(0, 40, 100, 30)),
            virtual_row(13, Rect::new(0, 70, 100, 30)),
        ];

        assert_eq!(
            exact_virtual_residency(
                10..14,
                &complete.iter().map(|row| row.key).collect::<Vec<_>>(),
                &complete,
                visible,
                layer,
            )
            .expect("exact rows should prove residency")
            .bounds,
            layer
        );

        for incomplete in [
            vec![complete[0], complete[2], complete[3]],
            vec![
                complete[0],
                complete[1],
                complete[1],
                complete[2],
                complete[3],
            ],
            vec![complete[0], complete[2], complete[1], complete[3]],
            vec![
                complete[0],
                complete[1],
                virtual_row(12, Rect::new(0, 41, 100, 29)),
                complete[3],
            ],
            vec![
                complete[0],
                complete[1],
                virtual_row(12, Rect::new(1, 40, 99, 30)),
                complete[3],
            ],
            {
                let mut stale = complete.to_vec();
                stale[2].key = crate::list::Key::new(99);
                stale
            },
        ] {
            assert!(
                exact_virtual_residency(
                    10..14,
                    &complete.iter().map(|row| row.key).collect::<Vec<_>>(),
                    &incomplete,
                    visible,
                    layer,
                )
                .is_err(),
                "holes, duplicates, reordering, stale keys, and pixel gaps are not drawable residency"
            );
        }
    }

    #[test]
    fn virtual_residency_requires_content_coverage_not_blank_viewport_tail() {
        let required_content = Rect::new(0, 0, 100, 60);
        let layer = Rect::new(0, 0, 100, 100);
        let rows = [
            virtual_row(0, Rect::new(0, 0, 100, 20)),
            virtual_row(1, Rect::new(0, 20, 100, 20)),
            virtual_row(2, Rect::new(0, 40, 100, 20)),
        ];

        assert_eq!(
            exact_virtual_residency(
                0..3,
                &rows.iter().map(|row| row.key).collect::<Vec<_>>(),
                &rows,
                required_content,
                layer,
            )
            .expect("short exact content should prove residency")
            .bounds,
            required_content,
            "pixels below a short content extent are intentionally blank, not missing rows"
        );
    }
}
