use super::{
    composition,
    geometry::{Point, Rect, Size},
    interaction, keymap, session,
    theme::Theme,
    view,
};
use crate::animation;
use std::collections::HashSet;

mod algorithm;
mod chrome;
mod control;
mod engine;
mod flow;
mod frame;
mod hit;
mod measure;
mod path;
mod table;
mod text;
mod typography;
mod viewport;

pub(crate) use chrome::{Chrome, Kind as ChromeKind, Scrollbar};
pub(crate) use control::{
    choice_label_rect, choice_mark_rect, control_content_extent, menu_row_parts, palette_row_parts,
    slider_label_rect, slider_thumb_rect, slider_track_rect, table_choice_label_rect,
    table_choice_mark_rect, table_content_rect, table_header_label_rect, table_sort_indicator_rect,
};
pub(crate) use engine::Engine;
pub(crate) use frame::Frame;
pub(crate) use hit::Hit;
pub(crate) use table::{Axis as TableTrackAxis, Track as TableTrack};
pub(crate) use text::{Area as TextArea, Service as TextService};
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
    table_tracks: Vec<TableTrack>,
    virtual_list_requests: Vec<crate::virtual_list::Request>,
    native_popup_surfaces: HashSet<interaction::Id>,
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
        let tree = composition::Tree::layout(view);
        Self::compose_view_tree_with_theme_at(
            view,
            &tree,
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
            composition.tree(),
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
        tree: &composition::Tree,
        size: Size,
        engine: &mut Engine,
        theme: &Theme,
        frame: animation::Frame,
        keymap: keymap::Profile,
        popup_surfaces: PopupSurfaces,
    ) -> Self {
        let size = size.sanitized();
        let frames =
            algorithm::compose_frames(view.root(), tree.root(), size, engine, theme, frame, keymap);
        let chrome = chrome::project(&frames, theme);
        let table_tracks = table::project(&frames);
        let virtual_list_requests = frames
            .iter()
            .filter_map(Frame::virtual_list_request)
            .cloned()
            .collect();
        let native_popup_surfaces = match popup_surfaces {
            PopupSurfaces::InFrame => HashSet::new(),
            PopupSurfaces::Native => root_floating_panels(&frames)
                .filter_map(|panel| panel.target()?.element_id())
                .collect(),
        };

        Self {
            size,
            frames,
            chrome,
            table_tracks,
            virtual_list_requests,
            native_popup_surfaces,
        }
    }

    pub(crate) fn size(&self) -> Size {
        self.size
    }

    pub(crate) fn frames(&self) -> &[Frame] {
        &self.frames
    }

    pub(crate) fn frame_for_node(&self, node: composition::NodeId) -> Option<&Frame> {
        self.frames.iter().find(|frame| frame.node_id() == node)
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

    pub(crate) fn table_tracks(&self) -> &[TableTrack] {
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
        let table_cell = self
            .frames
            .iter()
            .rev()
            .find(|frame| {
                self.surface_accepts_frame(surface, frame)
                    && frame.table_cell().is_some()
                    && frame.rect().contains(point)
                    && frame.clip_contains(point)
            })
            .and_then(Frame::table_cell);
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

        if let Some(track) = self
            .table_tracks
            .iter()
            .rev()
            .find(|track| track.accepts_resize_hit(point))
        {
            let header = self.frames.iter().find(|frame| {
                Some(frame.node_id()) == track.header_node()
                    && self.surface_accepts_frame(surface, frame)
            })?;
            return Some(
                Hit::table_divider(header.clone(), track.divider_target()?)
                    .with_table_cell(table_cell),
            );
        }

        self.frames
            .iter()
            .rev()
            .find(|frame| self.surface_accepts_frame(surface, frame) && frame.accepts_hit(point))
            .cloned()
            .map(Hit::new)
            .map(|hit| hit.with_table_cell(table_cell))
    }

    /// Returns the deepest laid-out node under a point, including inert
    /// display nodes that ordinary activation hit testing intentionally skips.
    #[cfg(test)]
    pub(crate) fn context_node_at(&self, point: Point) -> Option<composition::NodeId> {
        self.context_node_at_surface(point, crate::popup::Surface::Parent)
    }

    pub(crate) fn context_node_at_surface(
        &self,
        point: Point,
        surface: crate::popup::Surface,
    ) -> Option<composition::NodeId> {
        self.frames
            .iter()
            .rev()
            .find(|frame| {
                self.surface_accepts_frame(surface, frame)
                    && frame.rect().contains(point)
                    && frame.clip_contains(point)
            })
            .map(Frame::node_id)
    }

    pub(crate) fn context_available_for_node(&self, node: composition::NodeId) -> Option<Rect> {
        let frame = self.frame_for_node(node)?;
        Some(
            frame
                .clip()
                .map(|clip| clip.rect())
                .unwrap_or_else(|| Rect::from_size(self.size)),
        )
    }

    pub(crate) fn drag_action_for_target(
        &self,
        target: &interaction::Target,
        point: Point,
        engine: &mut Engine,
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
            return Some((view::Role::Label, track.resize_action_at(point)));
        }

        self.frames
            .iter()
            .find(|frame| frame.target() == Some(target))
            .map(|frame| {
                (
                    frame.role(),
                    frame.drag_action_at_with_engine(point, engine),
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

    pub(crate) fn scroll_target_at_surface(
        &self,
        point: Point,
        delta: interaction::ScrollDelta,
        surface: crate::popup::Surface,
    ) -> Option<interaction::Target> {
        self.frames
            .iter()
            .rev()
            .find(|frame| {
                self.surface_accepts_frame(surface, frame)
                    && frame.viewport().is_some_and(|viewport| {
                        viewport.rect().contains(point)
                            && frame.clip_contains(point)
                            && viewport.can_consume(delta)
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
        self.frames
            .iter()
            .filter(|candidate| candidate.role() == view::Role::FloatingPanel)
            .filter_map(|candidate| {
                let id = candidate.target()?.element_id()?;
                self.native_popup_surfaces
                    .contains(&id)
                    .then_some((id, candidate))
            })
            .filter(|(_, candidate)| {
                candidate.node_id() == frame.node_id() || frame.is_descendant_of(candidate)
            })
            .max_by_key(|(_, candidate)| candidate.path_depth())
            .map(|(id, _)| id)
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
            .with_menu_placement(
                crate::geometry::PlacementAnchor::Point(Point::new(95, 75)),
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
            .with_menu_placement(
                crate::geometry::PlacementAnchor::Point(Point::new(48, 38)),
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
}
