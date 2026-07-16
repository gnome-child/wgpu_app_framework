use super::{
    composition::{self, tree},
    geometry::{Point, Rect, Size},
    interaction, keymap, session,
    theme::Theme,
    view,
};
use crate::animation;
use std::{
    collections::{HashMap, HashSet},
    ops::Range,
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
pub(crate) use frame::Frame;
pub(crate) use frame::SceneKey as FrameSceneKey;
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
    frames: Vec<Frame>,
    chrome: Vec<Chrome>,
    table_tracks: Vec<table::Track>,
    scroll_ancestries: HashMap<composition::tree::NodeId, Vec<composition::tree::NodeId>>,
    scroll_projections: Vec<ScrollProjection>,
    virtual_list_requests: Vec<crate::virtual_list::Request>,
    native_popup_owners: HashMap<composition::tree::NodeId, interaction::Id>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScrollProjection {
    node: composition::tree::NodeId,
    target: interaction::Target,
    viewport: Viewport,
    layer_bounds: Rect,
    residency: ScrollResidency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ScrollResidency {
    Complete(Proof),
    Empty,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Proof {
    node: composition::tree::NodeId,
    target: interaction::Target,
    requested: Option<Requested>,
    rows: Vec<Row>,
    viewport: Viewport,
    baseline: interaction::ScrollOffset,
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
    key: crate::virtual_list::Key,
    index: usize,
    rect: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Accepted {
    minimum: interaction::ScrollOffset,
    maximum: interaction::ScrollOffset,
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

    #[cfg(any(test, feature = "renderer-debug"))]
    pub(crate) fn layer_bounds(&self) -> Rect {
        self.layer_bounds
    }

    pub(crate) fn resident_bounds(&self) -> Option<Rect> {
        match &self.residency {
            ScrollResidency::Complete(proof) => Some(proof.bounds),
            ScrollResidency::Empty | ScrollResidency::Incomplete => None,
        }
    }

    pub(crate) fn is_scene_drawable(&self) -> bool {
        matches!(self.residency, ScrollResidency::Complete(_))
    }

    pub(crate) fn accepted_offsets(
        &self,
    ) -> Option<(interaction::ScrollOffset, interaction::ScrollOffset)> {
        match &self.residency {
            ScrollResidency::Complete(proof) => {
                Some((proof.accepted.minimum, proof.accepted.maximum))
            }
            ScrollResidency::Empty | ScrollResidency::Incomplete => None,
        }
    }
}

impl Proof {
    fn new(
        node: composition::tree::NodeId,
        target: interaction::Target,
        requested: Option<Requested>,
        rows: Vec<Row>,
        viewport: Viewport,
        bounds: Rect,
    ) -> Option<Self> {
        if bounds.width() <= 0 || bounds.height() <= 0 {
            return None;
        }
        let baseline = viewport.resolved_scroll();
        let accepted = Accepted::for_resident(viewport, baseline, bounds)?;
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
        offset: interaction::ScrollOffset,
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
        baseline: interaction::ScrollOffset,
        bounds: Rect,
    ) -> Option<Self> {
        let rect = viewport.rect();
        let visible = viewport.visible_content();
        let content = viewport.content();
        let maximum = viewport.max_scroll();
        let (minimum_x, maximum_x) = accepted_axis(
            bounds.x(),
            bounds.right(),
            rect.x(),
            visible.x(),
            visible.right(),
            content.width(),
            baseline.x(),
            maximum.x(),
        )?;
        let (minimum_y, maximum_y) = accepted_axis(
            bounds.y(),
            bounds.bottom(),
            rect.y(),
            visible.y(),
            visible.bottom(),
            content.height(),
            baseline.y(),
            maximum.y(),
        )?;
        Some(Self {
            minimum: interaction::ScrollOffset::new(minimum_x, minimum_y),
            maximum: interaction::ScrollOffset::new(maximum_x, maximum_y),
        })
    }

    fn contains(self, offset: interaction::ScrollOffset) -> bool {
        (self.minimum.x()..=self.maximum.x()).contains(&offset.x())
            && (self.minimum.y()..=self.maximum.y()).contains(&offset.y())
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
        Self::compose_view_tree_with_theme_at(
            view,
            tree.root(),
            size,
            engine,
            theme,
            frame,
            keymap,
            PopupSurfaces::InFrame,
        )
    }

    pub(crate) fn compose_composition_with_theme_at(
        composition: &composition::Composition,
        size: Size,
        engine: &mut Engine,
        theme: &Theme,
        frame: animation::Frame,
        keymap: keymap::Profile,
        popup_surfaces: PopupSurfaces,
    ) -> Self {
        Self::compose_view_tree_with_theme_at(
            composition.view(),
            composition.tree().root(),
            size,
            engine,
            theme,
            frame,
            keymap,
            popup_surfaces,
        )
    }

    fn compose_view_tree_with_theme_at(
        view: &view::View,
        root: &tree::Node,
        size: Size,
        engine: &mut Engine,
        theme: &Theme,
        frame: animation::Frame,
        keymap: keymap::Profile,
        popup_surfaces: PopupSurfaces,
    ) -> Self {
        let size = size.sanitized();
        let frames =
            algorithm::compose_frames(view.root(), root, size, engine, theme, frame, keymap);
        let chrome = chrome::project(&frames, theme);
        let table_tracks = table::project(&frames);
        let scroll_ancestries = project_scroll_ancestries(&frames);
        let scroll_projections =
            project_scroll_projections(&frames, &table_tracks, &scroll_ancestries);
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
            scroll_ancestries,
            scroll_projections,
            virtual_list_requests,
            native_popup_owners,
        }
    }

    pub(crate) fn size(&self) -> Size {
        self.size
    }

    pub(crate) fn frames(&self) -> &[Frame] {
        &self.frames
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
            .all(|projection| !matches!(projection.residency, ScrollResidency::Incomplete))
    }

    pub(crate) fn scroll_ancestry(
        &self,
        node: composition::tree::NodeId,
    ) -> &[composition::tree::NodeId] {
        self.scroll_ancestries
            .get(&node)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub(crate) fn scene_scroll_ancestry_is_drawable(
        &self,
        node: composition::tree::NodeId,
    ) -> bool {
        self.scroll_ancestry(node).iter().all(|owner| {
            let projection = self
                .scroll_projections
                .iter()
                .find(|projection| projection.node == *owner)
                .expect("every scroll ancestor must own one layout projection");
            match &projection.residency {
                ScrollResidency::Complete(_) => true,
                ScrollResidency::Empty => false,
                ScrollResidency::Incomplete => {
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

    pub(crate) fn scroll_property_accepts(
        &self,
        target: &interaction::Target,
        offset: interaction::ScrollOffset,
    ) -> bool {
        let mut owns_changed_axis = false;
        for projection in self
            .scroll_projections()
            .iter()
            .filter(|projection| &projection.target == target)
        {
            let viewport = projection.viewport;
            let maximum = viewport.max_scroll();
            let baseline = viewport.resolved_scroll();
            let changes_x = maximum.x() > 0 && offset.x() != baseline.x();
            let changes_y = maximum.y() > 0 && offset.y() != baseline.y();
            if !changes_x && !changes_y {
                continue;
            }
            owns_changed_axis = true;
            let resolved = viewport.resolve(offset);
            if (changes_x && resolved.x() != offset.x())
                || (changes_y && resolved.y() != offset.y())
            {
                return false;
            }
            let ScrollResidency::Complete(proof) = &projection.residency else {
                return false;
            };
            if !proof.accepts(projection.node, &projection.target, resolved) {
                return false;
            }
        }
        owns_changed_axis
    }

    pub(crate) fn resolve_scroll_offset(
        &self,
        target: &interaction::Target,
        offset: interaction::ScrollOffset,
    ) -> interaction::ScrollOffset {
        let mut found = false;
        let mut maximum = interaction::ScrollOffset::default();
        for projection in self
            .scroll_projections
            .iter()
            .filter(|projection| &projection.target == target)
        {
            found = true;
            let candidate = projection.viewport.max_scroll();
            maximum = interaction::ScrollOffset::new(
                maximum.x().max(candidate.x()),
                maximum.y().max(candidate.y()),
            );
        }
        if found {
            interaction::ScrollOffset::new(
                offset.x().clamp(0, maximum.x()),
                offset.y().clamp(0, maximum.y()),
            )
        } else {
            offset
        }
    }

    pub(crate) fn resident_node_ids(
        &self,
        scroll: composition::tree::NodeId,
    ) -> Vec<composition::tree::NodeId> {
        let Some(projection) = self
            .scroll_projections
            .iter()
            .find(|projection| projection.node == scroll)
        else {
            return Vec::new();
        };
        let ScrollResidency::Complete(proof) = &projection.residency else {
            return Vec::new();
        };
        let row_roots = proof.requested.as_ref().map_or_else(HashSet::new, |_| {
            proof
                .rows
                .iter()
                .map(|row| row.node)
                .collect::<HashSet<_>>()
        });
        self.frames
            .iter()
            .filter(|frame| {
                if frame.node_id() == scroll {
                    return true;
                }
                if self.scroll_ancestry(frame.node_id()).last() != Some(&scroll) {
                    return false;
                }
                row_roots.is_empty()
                    || row_roots.contains(&frame.node_id())
                    || self.frames.iter().any(|root| {
                        row_roots.contains(&root.node_id()) && frame.is_descendant_of(root)
                    })
            })
            .map(Frame::node_id)
            .collect()
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

    pub(crate) fn virtual_request_for_scroll_offset(
        &self,
        target: &interaction::Target,
        offset: interaction::ScrollOffset,
    ) -> Option<crate::virtual_list::Request> {
        self.scroll_projections
            .iter()
            .filter(|projection| &projection.target == target)
            .find_map(|projection| {
                self.frames
                    .iter()
                    .find(|frame| frame.node_id() == projection.node)?
                    .virtual_list_request_for_offset(offset)
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

    pub(crate) fn virtual_list_requests(&self) -> &[crate::virtual_list::Request] {
        &self.virtual_list_requests
    }

    /// The floating panels that own independent presentation surfaces.
    ///
    /// Nested floating panels remain content of their nearest root panel; this
    /// census is shared by surface ownership and overlay scene extraction so
    /// interaction and presentation cannot disagree about the boundary.
    pub(crate) fn root_floating_panels(&self) -> impl Iterator<Item = &Frame> {
        root_floating_panels(&self.frames)
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

    pub(crate) fn text_caret_rect(&self) -> Option<Rect> {
        self.frames.iter().find_map(Frame::text_caret_rect)
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
        delta: interaction::ScrollDelta,
    ) -> Option<interaction::Target> {
        self.scroll_target_at_surface(point, delta, crate::popup::Surface::Parent)
    }

    #[cfg(any(test, feature = "renderer-debug"))]
    pub(crate) fn scroll_target_at_surface(
        &self,
        point: Point,
        delta: interaction::ScrollDelta,
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
        delta: interaction::ScrollDelta,
        surface: crate::popup::Surface,
        project: &impl Fn(composition::tree::NodeId, Point) -> Option<Point>,
        offset: &impl Fn(&interaction::Target, Viewport) -> interaction::ScrollOffset,
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

    pub(crate) fn reveal_offset_for_descendant(
        &self,
        viewport_target: &interaction::Target,
        margin: i32,
        mut accepts_descendant: impl FnMut(&Frame) -> bool,
    ) -> Option<interaction::ScrollOffset> {
        let mut found = false;
        let mut resolved = interaction::ScrollOffset::default();
        for viewport_frame in self
            .frames
            .iter()
            .filter(|frame| frame.target() == Some(viewport_target))
        {
            let Some(viewport) = viewport_frame.viewport() else {
                continue;
            };
            let Some(descendant) = self
                .frames
                .iter()
                .find(|frame| frame.is_descendant_of(viewport_frame) && accepts_descendant(frame))
            else {
                continue;
            };
            found = true;
            let candidate = viewport.reveal_rect(descendant.rect(), margin);
            let maximum = viewport.max_scroll();
            resolved = interaction::ScrollOffset::new(
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
        found.then_some(resolved)
    }

    #[cfg(test)]
    pub(crate) fn find_role(&self, role: view::Role) -> Vec<&Frame> {
        self.frames
            .iter()
            .filter(|frame| frame.role() == role)
            .collect()
    }
}

fn project_scroll_ancestries(
    frames: &[Frame],
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
    frames: &[Frame],
    table_tracks: &[table::Track],
    scroll_ancestries: &HashMap<composition::tree::NodeId, Vec<composition::tree::NodeId>>,
) -> Vec<ScrollProjection> {
    frames
        .iter()
        .filter_map(|frame| {
            let viewport = frame.property_scroll_viewport()?;
            let target = frame.target()?.clone();
            let node = frame.node_id();
            let (layer_bounds, residency) =
                scroll_layer_geometry(frames, table_tracks, scroll_ancestries, node, viewport);
            Some(ScrollProjection {
                node,
                target,
                viewport,
                layer_bounds,
                residency,
            })
        })
        .collect()
}

fn scroll_layer_geometry(
    frames: &[Frame],
    table_tracks: &[table::Track],
    scroll_ancestries: &HashMap<composition::tree::NodeId, Vec<composition::tree::NodeId>>,
    owner: composition::tree::NodeId,
    viewport: Viewport,
) -> (Rect, ScrollResidency) {
    let nearest_scroll = |node| {
        scroll_ancestries
            .get(&node)
            .and_then(|ancestry| ancestry.last())
            .copied()
    };
    let owner_frame = frames.iter().find(|frame| frame.node_id() == owner);
    let explicit_prepared_bounds =
        owner_frame.is_some_and(|frame| frame.text_area_layout().is_some());
    let mut bounds = owner_frame.and_then(|frame| frame.scroll_resident_bounds());
    for frame in frames
        .iter()
        .filter(|frame| frame.node_id() != owner && nearest_scroll(frame.node_id()) == Some(owner))
    {
        bounds = Some(union_rect(
            bounds.unwrap_or_else(|| frame.rect()),
            frame.rect(),
        ));
    }
    for track in table_tracks
        .iter()
        .filter(|track| nearest_scroll(track.owner_node()) == Some(owner))
    {
        bounds = Some(union_rect(
            bounds.unwrap_or_else(|| track.rule_rect()),
            track.rule_rect(),
        ));
    }

    let visible = viewport.visible_content();
    if visible.width() <= 0 || visible.height() <= 0 {
        return (visible, ScrollResidency::Empty);
    }
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
    let virtual_request = frames
        .iter()
        .find(|frame| frame.node_id() == owner)
        .and_then(Frame::virtual_list_request);
    let residency = match virtual_request {
        Some(request) if !request.range().is_empty() => {
            let requested = request.range();
            let expected_keys = requested
                .clone()
                .map(|index| owner_frame?.virtual_list_key_at(index))
                .collect::<Option<Vec<_>>>();
            let rows = frames
                .iter()
                .filter_map(|frame| {
                    let row = frame.provided_row()?;
                    (row.list() == request.id()
                        && requested.contains(&row.index())
                        && nearest_scroll(frame.node_id()) == Some(owner))
                    .then_some(Row {
                        node: frame.node_id(),
                        list: row.list(),
                        key: row.key(),
                        index: row.index(),
                        rect: frame.rect(),
                    })
                })
                .collect::<Vec<_>>();
            viewport
                .visible_content_coverage()
                .map_or(ScrollResidency::Empty, |required| {
                    expected_keys
                        .and_then(|expected_keys| {
                            exact_virtual_residency(
                                requested,
                                &expected_keys,
                                &rows,
                                required,
                                layer_bounds,
                            )
                        })
                        .and_then(|rows| {
                            Proof::new(
                                owner,
                                frames
                                    .iter()
                                    .find(|frame| frame.node_id() == owner)
                                    .and_then(Frame::target)
                                    .cloned()?,
                                Some(Requested {
                                    list: request.id(),
                                    range: request.range(),
                                }),
                                rows.rows,
                                viewport,
                                rows.bounds,
                            )
                        })
                        .map_or(ScrollResidency::Incomplete, ScrollResidency::Complete)
                })
        }
        Some(_) if viewport.visible_content_coverage().is_none() => ScrollResidency::Empty,
        Some(_) => ScrollResidency::Incomplete,
        None => frames
            .iter()
            .find(|frame| frame.node_id() == owner)
            .and_then(Frame::target)
            .cloned()
            .and_then(|target| Proof::new(owner, target, None, Vec::new(), viewport, layer_bounds))
            .map_or(ScrollResidency::Incomplete, ScrollResidency::Complete),
    };

    (layer_bounds, residency)
}

fn exact_virtual_residency(
    requested: Range<usize>,
    expected_keys: &[crate::virtual_list::Key],
    rows: &[Row],
    required: Rect,
    layer_bounds: Rect,
) -> Option<Rows> {
    if expected_keys.len() != requested.len() {
        return None;
    }
    let mut expected = requested.start;
    let mut previous = None::<Rect>;
    let mut bounds = None::<Rect>;
    let mut keys = HashSet::with_capacity(rows.len());
    let mut nodes = HashSet::with_capacity(rows.len());

    for row in rows.iter().copied() {
        let key = expected_keys.get(expected.saturating_sub(requested.start));
        if row.index != expected
            || key != Some(&row.key)
            || !keys.insert(row.key)
            || !nodes.insert(row.node)
            || row.rect.width() <= 0
            || row.rect.height() <= 0
            || row.rect.x() > required.x()
            || row.rect.right() < required.right()
            || previous.is_some_and(|previous| previous.bottom() != row.rect.y())
        {
            return None;
        }
        expected = expected.saturating_add(1);
        previous = Some(row.rect);
        bounds = Some(bounds.map_or(row.rect, |bounds| union_rect(bounds, row.rect)));
    }

    if expected != requested.end {
        return None;
    }
    let resident = intersect_rect(bounds?, layer_bounds)?;
    contains_rect(resident, required).then(|| Rows {
        rows: rows.to_vec(),
        bounds: resident,
    })
}

fn root_floating_panels(frames: &[Frame]) -> impl Iterator<Item = &Frame> {
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
            key: crate::virtual_list::Key::new(index as u64),
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
            .map(|residency| residency.bounds),
            Some(layer)
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
                stale[2].key = crate::virtual_list::Key::new(99);
                stale
            },
        ] {
            assert_eq!(
                exact_virtual_residency(
                    10..14,
                    &complete.iter().map(|row| row.key).collect::<Vec<_>>(),
                    &incomplete,
                    visible,
                    layer,
                ),
                None,
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
            .map(|residency| residency.bounds),
            Some(required_content),
            "pixels below a short content extent are intentionally blank, not missing rows"
        );
    }
}
