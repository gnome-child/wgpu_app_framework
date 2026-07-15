use super::{
    composition::{self, tree},
    geometry::{Point, Rect, Size},
    interaction, keymap, session,
    theme::Theme,
    view,
};
use crate::animation;
use std::{collections::HashMap, ops::Range};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScrollResidency {
    Complete(Rect),
    Incomplete,
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
        match self.residency {
            ScrollResidency::Complete(bounds) => Some(bounds),
            ScrollResidency::Incomplete => None,
        }
    }
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

fn translate_rect(rect: Rect, dx: i32, dy: i32) -> Rect {
    Rect::new(
        rect.x().saturating_add(dx),
        rect.y().saturating_add(dy),
        rect.width(),
        rect.height(),
    )
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
            .all(|projection| matches!(projection.residency, ScrollResidency::Complete(_)))
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

    pub(crate) fn scroll_property_accepts(
        &self,
        target: &interaction::Target,
        offset: interaction::ScrollOffset,
    ) -> bool {
        let Some(projection) = self
            .scroll_projections()
            .iter()
            .find(|projection| &projection.target == target)
        else {
            return false;
        };
        let resolved = projection.viewport.resolve(offset);
        if resolved != offset {
            return false;
        }
        let baseline = projection.viewport.resolved_scroll();
        let dx = baseline.x().saturating_sub(resolved.x());
        let dy = baseline.y().saturating_sub(resolved.y());
        let Some(resident_bounds) = projection.resident_bounds() else {
            return false;
        };
        let bounds = translate_rect(resident_bounds, dx, dy);
        contains_rect(bounds, projection.viewport.visible_content())
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

    #[cfg(test)]
    pub(crate) fn scroll_target_at(
        &self,
        point: Point,
        delta: interaction::ScrollDelta,
    ) -> Option<interaction::Target> {
        self.scroll_target_at_surface(point, delta, crate::popup::Surface::Parent)
    }

    #[cfg(test)]
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
        let viewport_frame = self
            .frames
            .iter()
            .find(|frame| frame.target() == Some(viewport_target))?;
        let viewport = viewport_frame.viewport()?;
        let descendant = self
            .frames
            .iter()
            .find(|frame| frame.is_descendant_of(viewport_frame) && accepts_descendant(frame))?;

        Some(viewport.reveal_rect(descendant.rect(), margin))
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
    let mut bounds = None;
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
    let max = viewport.max_scroll();
    let guard_x = (max.x() > 0)
        .then(|| visible.width().saturating_div(2))
        .unwrap_or(0);
    let guard_y = (max.y() > 0)
        .then(|| visible.height().saturating_div(2))
        .unwrap_or(0);
    let guard = Rect::new(
        visible.x().saturating_sub(guard_x),
        visible.y().saturating_sub(guard_y),
        visible.width().saturating_add(guard_x.saturating_mul(2)),
        visible.height().saturating_add(guard_y.saturating_mul(2)),
    );
    let realized = bounds.map_or(visible, |bounds| union_rect(bounds, visible));
    let layer_bounds = intersect_rect(realized, guard).unwrap_or(visible);
    let virtual_request = frames
        .iter()
        .find(|frame| frame.node_id() == owner)
        .and_then(Frame::virtual_list_request);
    let residency = match virtual_request {
        Some(request) if !request.range().is_empty() => {
            let requested = request.range();
            let rows = frames
                .iter()
                .filter_map(|frame| {
                    let row = frame.provided_row()?;
                    (row.list() == request.id()
                        && requested.contains(&row.index())
                        && nearest_scroll(frame.node_id()) == Some(owner))
                    .then_some((row.index(), frame.rect()))
                })
                .collect::<Vec<_>>();
            exact_virtual_residency(requested, &rows, visible, layer_bounds)
                .map_or(ScrollResidency::Incomplete, ScrollResidency::Complete)
        }
        _ => ScrollResidency::Complete(layer_bounds),
    };

    (layer_bounds, residency)
}

fn exact_virtual_residency(
    requested: Range<usize>,
    rows: &[(usize, Rect)],
    visible: Rect,
    layer_bounds: Rect,
) -> Option<Rect> {
    let mut expected = requested.start;
    let mut previous = None::<Rect>;
    let mut bounds = None::<Rect>;

    for (index, rect) in rows.iter().copied() {
        if index != expected
            || rect.width() <= 0
            || rect.height() <= 0
            || rect.x() > visible.x()
            || rect.right() < visible.right()
            || previous.is_some_and(|previous| previous.bottom() != rect.y())
        {
            return None;
        }
        expected = expected.saturating_add(1);
        previous = Some(rect);
        bounds = Some(bounds.map_or(rect, |bounds| union_rect(bounds, rect)));
    }

    if expected != requested.end {
        return None;
    }
    let resident = intersect_rect(bounds?, layer_bounds)?;
    contains_rect(resident, visible).then_some(resident)
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
            (10, Rect::new(0, -20, 100, 30)),
            (11, Rect::new(0, 10, 100, 30)),
            (12, Rect::new(0, 40, 100, 30)),
            (13, Rect::new(0, 70, 100, 30)),
        ];

        assert_eq!(
            exact_virtual_residency(10..14, &complete, visible, layer),
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
                (12, Rect::new(0, 41, 100, 29)),
                complete[3],
            ],
            vec![
                complete[0],
                complete[1],
                (12, Rect::new(1, 40, 99, 30)),
                complete[3],
            ],
        ] {
            assert_eq!(
                exact_virtual_residency(10..14, &incomplete, visible, layer),
                None,
                "holes, duplicates, reordering, and pixel gaps are not drawable residency"
            );
        }
    }
}
