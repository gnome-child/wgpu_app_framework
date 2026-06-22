use std::collections::HashMap;
use std::time::Instant;

use crate::animation::Frame as AnimationFrame;
use crate::app::scroll;
use crate::geometry::{Rect, area, point};
use crate::widget::menu;
use crate::{action, paint, text, widget, window};

use super::{
    CommandSubject, Cursor, Frame, Intent, Interaction, Interactivity, Node, Path, floating,
    layout_engine, painting,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Tree {
    root: Option<Node>,
    popups: Vec<widget::Popup>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Composition {
    tree: Tree,
    layout: Frame,
    open_menu: Option<menu::Id>,
    open_submenu: Option<menu::Id>,
    menus: HashMap<menu::Id, menu::Menu>,
    actions: HashMap<Path, action::Id>,
    command_subjects: HashMap<Path, CommandSubject>,
    intents: HashMap<Path, Intent>,
    responders: HashMap<Path, Vec<action::Id>>,
    responder_bindings: HashMap<Path, Vec<action::Binding>>,
    command_scopes: Vec<Path>,
    command_scope_contexts: HashMap<Path, action::Context>,
    text_fields: HashMap<Path, text::Field>,
    text_surfaces: HashMap<Path, text::Surface>,
    interactivity: HashMap<Path, Interactivity>,
    path_states: HashMap<Path, action::State>,
    cursors: HashMap<Path, Cursor>,
    widget_metrics: HashMap<Path, widget::Metrics>,
    focus_order: Vec<Path>,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            root: None,
            popups: Vec::new(),
        }
    }

    pub fn set_root(&mut self, root: Node) {
        self.root = Some(root);
    }

    pub fn root(&self) -> Option<&Node> {
        self.root.as_ref()
    }

    pub fn root_mut(&mut self) -> Option<&mut Node> {
        self.root.as_mut()
    }

    pub fn push_popup(&mut self, popup: widget::Popup) {
        self.popups.push(popup);
    }

    pub fn clear_popups(&mut self) {
        self.popups.clear();
    }

    pub fn popups(&self) -> &[widget::Popup] {
        &self.popups
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn layout(
        &self,
        area: area::Logical,
        measurer: &mut text::layout::Engine,
    ) -> Option<Frame> {
        let root = self.root.as_ref()?;
        let root_layout = layout_engine::tree(root, area, measurer);
        if self.popups.is_empty() {
            return Some(root_layout);
        }

        let mut children = root_layout.children().to_vec();
        let root_path = root_layout.path().clone();
        for popup in &self.popups {
            children.push(layout_engine::subtree_at(
                popup.root(),
                root_path.child(popup.root().id()),
                popup.rect(),
                measurer,
            ));
        }

        Some(root_layout.with_children(children))
    }

    pub fn compose<T>(
        &self,
        window: window::Id,
        area: area::Logical,
        actions: &mut action::Registry<T>,
        floating_surfaces: &[floating::Surface],
        measurer: &mut text::layout::Engine,
    ) -> Option<Composition> {
        let mut tree = self.clone();
        let menus = tree.menus();
        let open_menu_surface = floating_surfaces.iter().find(|surface| {
            matches!(surface.kind(), floating::Kind::Menu(menu) if menus.contains_key(menu))
        });
        let open_menu = open_menu_surface.and_then(|surface| surface.menu_id());
        let open_submenu_surface = floating_surfaces.iter().rev().find(|surface| {
            matches!(
                surface.kind(),
                floating::Kind::Submenu(menu) if open_menu.is_some() && menus.contains_key(menu)
            )
        });
        let open_submenu = open_submenu_surface.and_then(|surface| surface.menu_id());
        let context_menu = floating_surfaces
            .iter()
            .rev()
            .find(|surface| matches!(surface.kind(), floating::Kind::ContextMenu { .. }));
        let mut command_scope_contexts = HashMap::new();

        tree.publish_responder_binding_states(actions, window);

        let mut menu_popup_inserted = false;
        if let Some(surface) = open_menu_surface
            && let Some(open_menu) = open_menu
            && let Some(menu) = menus.get(&open_menu)
            && let Some(base_layout) = tree.layout(area, measurer)
            && let Some(popup) =
                widget::menu_popup(&tree, &base_layout, surface, menu, actions, measurer)
        {
            if let Some(scope) = popup_scope_path(&tree, popup.root().id()) {
                command_scope_contexts.insert(scope, surface.command_context().clone());
            }
            tree.push_popup(popup);
            menu_popup_inserted = true;
        }

        if menu_popup_inserted
            && let Some(surface) = open_submenu_surface
            && let Some(open_submenu) = open_submenu
            && let Some(menu) = menus.get(&open_submenu)
            && let Some(menu_layout) = tree.layout(area, measurer)
            && let Some(popup) =
                widget::submenu_popup(&tree, &menu_layout, surface, menu, actions, measurer)
        {
            if let Some(scope) = popup_scope_path(&tree, popup.root().id()) {
                command_scope_contexts.insert(scope, surface.command_context().clone());
            }
            tree.push_popup(popup);
        }

        if let Some(surface) = context_menu
            && let Some(base_layout) = tree.layout(area, measurer)
            && let Some(popup) =
                widget::text_context_menu_popup(surface, actions, measurer, base_layout.rect())
        {
            if let Some(scope) = popup_scope_path(&tree, popup.root().id()) {
                command_scope_contexts.insert(scope, surface.command_context().clone());
            }
            tree.push_popup(popup);
        }

        tree.publish_responder_binding_states(actions, window);
        let layout = tree.layout(area, measurer)?;

        Some(Composition::new(
            tree,
            layout,
            open_menu,
            open_submenu,
            menus,
            command_scope_contexts,
        ))
    }

    fn index(&self) -> TreeIndex {
        let mut index = TreeIndex::default();

        if let Some(root) = self.root.as_ref() {
            index.collect_node(root, &Path::root(root.id()));
            for popup in &self.popups {
                index.collect_node(
                    popup.root(),
                    &Path::root(root.id()).child(popup.root().id()),
                );
            }
        }

        index
    }

    pub fn actions(&self) -> HashMap<Path, action::Id> {
        self.index().actions
    }

    pub fn command_subjects(&self) -> HashMap<Path, CommandSubject> {
        self.index().command_subjects
    }

    pub fn intents(&self) -> HashMap<Path, Intent> {
        self.index().intents
    }

    pub fn menus(&self) -> HashMap<menu::Id, menu::Menu> {
        let mut menus = HashMap::new();

        if let Some(root) = self.root.as_ref() {
            collect_menus(root, &mut menus);
        }

        menus
    }

    pub fn responders(&self) -> HashMap<Path, Vec<action::Id>> {
        self.index().responders
    }

    pub fn responder_bindings(&self) -> HashMap<Path, Vec<action::Binding>> {
        self.index().responder_bindings
    }

    pub fn command_scopes(&self) -> Vec<Path> {
        self.index().command_scopes
    }

    pub fn interactivity(&self) -> HashMap<Path, Interactivity> {
        self.index().interactivity
    }

    pub fn widget_metrics(&self, layout: &Frame) -> HashMap<Path, widget::Metrics> {
        let mut metrics = HashMap::new();

        if let Some(root) = self.root.as_ref() {
            collect_widget_metrics(root, layout, &mut metrics);
            for popup in &self.popups {
                let path = Path::root(root.id()).child(popup.root().id());
                if let Some(popup_layout) = layout.find_path(&path) {
                    collect_widget_metrics(popup.root(), popup_layout, &mut metrics);
                }
            }
        }

        metrics
    }

    pub fn paint<T>(
        &self,
        layout: &Frame,
        actions: &action::Registry<T>,
        window: window::Id,
        interaction: Interaction,
        scene: &mut paint::Scene,
    ) {
        let mut text_engine = text::layout::Engine::new();
        let text_field_states = HashMap::new();
        self.paint_with_text_engine(
            layout,
            actions,
            window,
            interaction,
            &text_field_states,
            &mut text_engine,
            scene,
        );
    }

    pub fn paint_with_text_engine<T>(
        &self,
        layout: &Frame,
        actions: &action::Registry<T>,
        window: window::Id,
        interaction: Interaction,
        text_field_states: &HashMap<Path, text::view::TextViewState>,
        text_engine: &mut text::layout::Engine,
        scene: &mut paint::Scene,
    ) {
        self.paint_with_text_engine_at(
            layout,
            actions,
            window,
            interaction,
            text_field_states,
            text_engine,
            AnimationFrame::new(Instant::now(), None),
            scene,
        );
    }

    pub(crate) fn paint_with_text_engine_at<T>(
        &self,
        layout: &Frame,
        actions: &action::Registry<T>,
        window: window::Id,
        interaction: Interaction,
        text_field_states: &HashMap<Path, text::view::TextViewState>,
        text_engine: &mut text::layout::Engine,
        frame: AnimationFrame,
        scene: &mut paint::Scene,
    ) {
        let path_states = HashMap::new();

        self.paint_with_scroll_projections_at(
            layout,
            actions,
            window,
            interaction,
            text_field_states,
            text_engine,
            frame,
            None,
            &path_states,
            scene,
        );
    }

    pub(crate) fn paint_with_scroll_projections_at<T>(
        &self,
        layout: &Frame,
        actions: &action::Registry<T>,
        window: window::Id,
        interaction: Interaction,
        text_field_states: &HashMap<Path, text::view::TextViewState>,
        text_engine: &mut text::layout::Engine,
        frame: AnimationFrame,
        scroll_projections: Option<&scroll::Driver>,
        path_states: &HashMap<Path, action::State>,
        scene: &mut paint::Scene,
    ) {
        self.paint_with_scroll_projections_recording_at(
            layout,
            actions,
            window,
            interaction,
            text_field_states,
            text_engine,
            frame,
            scroll_projections,
            path_states,
            scene,
            None,
        );
    }

    pub(crate) fn paint_with_scroll_projections_recording_at<T>(
        &self,
        layout: &Frame,
        actions: &action::Registry<T>,
        window: window::Id,
        interaction: Interaction,
        text_field_states: &HashMap<Path, text::view::TextViewState>,
        text_engine: &mut text::layout::Engine,
        frame: AnimationFrame,
        scroll_projections: Option<&scroll::Driver>,
        path_states: &HashMap<Path, action::State>,
        scene: &mut paint::Scene,
        mut scroll_ranges: Option<&mut painting::ScrollPaintRecords>,
    ) {
        if let Some(root) = self.root.as_ref() {
            painting::tree(
                root,
                layout,
                actions,
                window,
                &interaction,
                text_field_states,
                text_engine,
                frame,
                scroll_projections,
                path_states,
                scene,
                scroll_ranges.as_deref_mut(),
            );
            for popup in &self.popups {
                let path = layout.path().child(popup.root().id());
                if let Some(popup_layout) = layout.find_path(&path) {
                    painting::tree(
                        popup.root(),
                        popup_layout,
                        actions,
                        window,
                        &interaction,
                        text_field_states,
                        text_engine,
                        frame,
                        scroll_projections,
                        path_states,
                        scene,
                        scroll_ranges.as_deref_mut(),
                    );
                }
            }

            painting::cursor_overlay(layout, &interaction, text_engine, scene);
        }
    }

    fn publish_responder_binding_states<T>(
        &self,
        actions: &mut action::Registry<T>,
        window: window::Id,
    ) -> bool {
        let mut changed = false;

        for (path, bindings) in self.responder_bindings() {
            for binding in bindings {
                let Some(state) = binding.state() else {
                    continue;
                };

                changed |= actions.set_state(
                    binding.action(),
                    action::Context::path(window, path.clone()),
                    state,
                );
            }
        }

        changed
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

fn focus_order(layout: &Frame, interactivity: &HashMap<Path, Interactivity>) -> Vec<Path> {
    let mut order = Vec::new();
    collect_focus_order(layout, interactivity, &mut order);
    order
}

fn collect_focus_order(
    layout: &Frame,
    interactivity: &HashMap<Path, Interactivity>,
    order: &mut Vec<Path>,
) {
    if interactivity
        .get(layout.path())
        .is_some_and(|interactivity| interactivity.focusable())
    {
        order.push(layout.path().clone());
    }

    for child in layout.children() {
        collect_focus_order(child, interactivity, order);
    }
}

fn node_at_path<'a>(node: &'a Node, ids: &[super::Id]) -> Option<&'a Node> {
    if ids.first().copied() != Some(node.id()) {
        return None;
    }

    let Some((_, rest)) = ids.split_first() else {
        return Some(node);
    };

    if rest.is_empty() {
        return Some(node);
    }

    node.children()
        .iter()
        .find_map(|child| node_at_path(child, rest))
}

fn text_content_rect(node: &Node, layout: &Frame) -> crate::geometry::Rect {
    let rect = layout.rect();
    let padding = node.style().padding();
    let x = rect.origin.x() + padding.left;
    let y = rect.origin.y() + padding.top;
    let width = (rect.area.width() - padding.left - padding.right).max(0.0);
    let height = (rect.area.height() - padding.top - padding.bottom).max(0.0);

    crate::geometry::Rect::new(point::logical(x, y), area::logical(width, height))
}

fn popup_scope_path(tree: &Tree, popup_root: super::Id) -> Option<Path> {
    tree.root()
        .map(|root| Path::root(root.id()).child(popup_root))
}

impl Composition {
    fn new(
        tree: Tree,
        layout: Frame,
        open_menu: Option<menu::Id>,
        open_submenu: Option<menu::Id>,
        menus: HashMap<menu::Id, menu::Menu>,
        command_scope_contexts: HashMap<Path, action::Context>,
    ) -> Self {
        let index = tree.index();
        let widget_metrics = tree.widget_metrics(&layout);
        let focus_order = focus_order(&layout, &index.interactivity);

        Self {
            tree,
            layout,
            open_menu,
            open_submenu,
            menus,
            actions: index.actions,
            command_subjects: index.command_subjects,
            intents: index.intents,
            responders: index.responders,
            responder_bindings: index.responder_bindings,
            command_scopes: index.command_scopes,
            command_scope_contexts,
            text_fields: index.text_fields,
            text_surfaces: index.text_surfaces,
            interactivity: index.interactivity,
            path_states: HashMap::new(),
            cursors: index.cursors,
            widget_metrics,
            focus_order,
        }
    }

    pub fn layout(&self) -> &Frame {
        &self.layout
    }

    pub fn open_menu(&self) -> Option<menu::Id> {
        self.open_menu
    }

    pub fn open_submenu(&self) -> Option<menu::Id> {
        self.open_submenu
    }

    pub fn menus(&self) -> &HashMap<menu::Id, menu::Menu> {
        &self.menus
    }

    pub fn menu(&self, id: menu::Id) -> Option<&menu::Menu> {
        self.menus.get(&id)
    }

    pub fn action(&self, path: &Path) -> Option<action::Id> {
        self.actions.get(path).copied()
    }

    pub fn actions(&self) -> &HashMap<Path, action::Id> {
        &self.actions
    }

    pub fn command_subject(&self, path: &Path) -> CommandSubject {
        self.command_subjects.get(path).copied().unwrap_or_default()
    }

    pub fn command_subjects(&self) -> &HashMap<Path, CommandSubject> {
        &self.command_subjects
    }

    pub fn intent(&self, path: &Path) -> Option<Intent> {
        self.intents.get(path).copied()
    }

    pub fn intents(&self) -> &HashMap<Path, Intent> {
        &self.intents
    }

    pub fn responders(&self, path: &Path) -> Option<&[action::Id]> {
        self.responders.get(path).map(Vec::as_slice)
    }

    pub fn responder_map(&self) -> &HashMap<Path, Vec<action::Id>> {
        &self.responders
    }

    pub fn responder_bindings(&self, path: &Path) -> Option<&[action::Binding]> {
        self.responder_bindings.get(path).map(Vec::as_slice)
    }

    pub fn responder_binding_map(&self) -> &HashMap<Path, Vec<action::Binding>> {
        &self.responder_bindings
    }

    pub fn has_responder(&self, path: &Path) -> bool {
        self.responders
            .get(path)
            .is_some_and(|actions| !actions.is_empty())
    }

    pub fn command_scopes(&self) -> &[Path] {
        &self.command_scopes
    }

    pub fn command_scope_context(&self, scope: &Path) -> Option<&action::Context> {
        self.command_scope_contexts.get(scope)
    }

    pub fn command_scope_contexts(&self) -> &HashMap<Path, action::Context> {
        &self.command_scope_contexts
    }

    pub fn text_field(&self, path: &Path) -> Option<&text::Field> {
        self.text_surfaces
            .get(path)
            .and_then(text::Surface::as_field)
    }

    pub fn text_area(&self, path: &Path) -> Option<&text::Area> {
        self.text_surfaces
            .get(path)
            .and_then(text::Surface::as_area)
    }

    pub fn text_surface(&self, path: &Path) -> Option<&text::Surface> {
        self.text_surfaces.get(path)
    }

    pub fn text_fields(&self) -> &HashMap<Path, text::Field> {
        &self.text_fields
    }

    pub fn text_surfaces(&self) -> &HashMap<Path, text::Surface> {
        &self.text_surfaces
    }

    pub fn text_field_edit_at(
        &self,
        path: &Path,
        position: point::Logical,
        kind: text::edit::PointerEditKind,
        state: text::view::TextViewState,
        text_engine: &mut text::layout::Engine,
    ) -> Option<text::edit::Edit> {
        let position =
            self.text_position_at_for_text_surface(path, position, state, text_engine)?;
        Some(text::edit::Edit::pointer(kind, position))
    }

    pub fn text_field_position_at(
        &self,
        path: &Path,
        position: point::Logical,
        state: text::view::TextViewState,
        text_engine: &mut text::layout::Engine,
    ) -> Option<text::TextPosition> {
        self.text_position_at_for_text_surface(path, position, state, text_engine)
    }

    fn text_position_at_for_text_surface(
        &self,
        path: &Path,
        position: point::Logical,
        state: text::view::TextViewState,
        text_engine: &mut text::layout::Engine,
    ) -> Option<text::TextPosition> {
        let node = self.node(path)?;
        let surface = node.text_surface()?;
        let style = node
            .label()
            .and_then(text::document::Document::first_style)
            .unwrap_or_default();

        if let text::Surface::Area(area_model) = surface {
            let layout = self.layout.find_path(path)?;
            if let Some((metrics, paint_layout)) = self.text_area_scroll_paint_layout_for_node(
                node,
                layout,
                state.clone(),
                text_engine,
                Instant::now(),
            ) {
                let rect = metrics.viewport();
                let local = point::logical(
                    position.x() - rect.origin.x(),
                    position.y() - rect.origin.y(),
                );
                return text_engine.text_area_position_at_for_paint_layout(
                    area_model,
                    local,
                    state,
                    &paint_layout,
                );
            }

            let rect = text_content_rect(node, layout);
            let local = point::logical(
                position.x() - rect.origin.x(),
                position.y() - rect.origin.y(),
            );
            let paint_layout = text_engine.text_area_paint_layout_for_area_at(
                area_model,
                style,
                rect.area,
                state.clone(),
                Instant::now(),
            );
            return text_engine.text_area_position_at_for_paint_layout(
                area_model,
                local,
                state,
                &paint_layout,
            );
        }

        let rect = self.text_content_rect_for_state(path, state.clone(), text_engine)?;
        let position = point::logical(
            position.x() - rect.origin.x(),
            position.y() - rect.origin.y(),
        );
        text_engine.text_position_at_for_surface(surface, style, rect.area, position, state)
    }
    pub fn text_field_caret_rect(
        &self,
        path: &Path,
        state: text::view::TextViewState,
        text_engine: &mut text::layout::Engine,
    ) -> Option<Rect> {
        let node = self.node(path)?;
        let surface = node.text_surface()?;
        let style = node
            .label()
            .and_then(text::document::Document::first_style)
            .unwrap_or_default();
        let rect = self.text_content_rect_for_state(path, state.clone(), text_engine)?;
        let caret = match surface {
            text::Surface::Field(field) => {
                text_engine.text_field_caret_for_field(field, style, rect.area, state)
            }
            text::Surface::Area(area) => {
                text_engine.text_area_caret_for_area(area, style, rect.area, state)
            }
        }?;

        Some(Rect::new(
            point::logical(rect.origin.x() + caret.x(), rect.origin.y() + caret.y()),
            area::logical(1.0, caret.height().max(1.0)),
        ))
    }

    pub fn text_field_caret_rect_at_position(
        &self,
        path: &Path,
        position: text::TextPosition,
        state: text::view::TextViewState,
        text_engine: &mut text::layout::Engine,
    ) -> Option<Rect> {
        let node = self.node(path)?;
        let surface = node.text_surface()?;
        let style = node
            .label()
            .and_then(text::document::Document::first_style)
            .unwrap_or_default();
        let rect = self.text_content_rect_for_state(path, state.clone(), text_engine)?;
        let mut buffer = surface.buffer().clone();
        let mut text_editor = text::edit::Editor::new();
        text_editor.apply_text_edit(&mut buffer, text::edit::Edit::set_position(position));
        let caret = match surface {
            text::Surface::Field(_) => {
                text_engine.text_field_caret(&buffer, style, rect.area, state)
            }
            text::Surface::Area(area) => {
                let area = text::Area::new(buffer)
                    .with_mode(area.mode())
                    .with_wrap(area.wrap());
                text_engine.text_area_caret_for_area(&area, style, rect.area, state)
            }
        }?;

        Some(Rect::new(
            point::logical(rect.origin.x() + caret.x(), rect.origin.y() + caret.y()),
            area::logical(1.0, caret.height().max(1.0)),
        ))
    }

    pub fn sync_text_field_states(
        &self,
        states: &mut HashMap<Path, text::view::TextViewState>,
        focused: Option<&Path>,
        text_engine: &mut text::layout::Engine,
    ) -> bool {
        let mut changed = false;
        let old_len = states.len();
        states.retain(|path, _| self.text_surfaces.contains_key(path));
        changed |= old_len != states.len();

        for path in self.text_surfaces.keys() {
            if !states.contains_key(path) {
                states.insert(path.clone(), text::view::TextViewState::default());
                changed = true;
            }
        }

        let Some(focused) = focused.filter(|path| self.text_surfaces.contains_key(*path)) else {
            return changed;
        };
        let Some(node) = self.node(focused) else {
            return changed;
        };
        let Some(surface) = node.text_surface() else {
            return changed;
        };
        let current = states.get(focused).cloned().unwrap_or_default();
        if surface.is_area() && !current.caret_visibility_pending() {
            return changed;
        }

        let style = node
            .label()
            .and_then(text::document::Document::first_style)
            .unwrap_or_default();
        let Some(rect) = self.text_content_rect_for_state(focused, current.clone(), text_engine)
        else {
            return changed;
        };
        let next = if surface.is_area() && current.caret_visibility_pending() {
            self.ensure_caret_visible_for_text_surface(focused, current.clone(), text_engine)
                .unwrap_or_else(|| current.clone())
                .clear_caret_visibility_pending()
        } else if surface.is_field() {
            text_engine
                .ensure_caret_visible_for_surface(surface, style, rect.area, current.clone())
                .clear_caret_visibility_pending()
        } else {
            current.clone()
        };

        if next != current {
            states.insert(focused.clone(), next);
            changed = true;
        }

        changed
    }

    pub fn text_area_wheel_scroll(
        &self,
        path: &Path,
        delta: point::Logical,
        horizontal_from_vertical: bool,
        state: text::view::TextViewState,
        text_engine: &mut text::layout::Engine,
    ) -> Option<text::view::TextViewState> {
        let node = self.node(path)?;
        node.text_area()?;
        let layout = self.layout.find_path(path)?;
        let (delta_x, delta_y) = if horizontal_from_vertical {
            (delta.y(), delta.x())
        } else {
            (delta.x(), delta.y())
        };
        let metrics = self.text_area_scroll_metrics_for_node(
            node,
            layout,
            state.clone(),
            text_engine,
            Instant::now(),
        )?;
        let offset = metrics.wheel_offset(point::logical(delta_x, delta_y));

        Some(state.with_scroll(offset.x(), offset.y()))
    }

    pub fn text_area_scroll_metrics(
        &self,
        path: &Path,
        state: text::view::TextViewState,
        text_engine: &mut text::layout::Engine,
    ) -> Option<widget::scroll::Metrics> {
        let node = self.node(path)?;
        let layout = self.layout.find_path(path)?;

        self.text_area_scroll_metrics_for_node(node, layout, state, text_engine, Instant::now())
    }

    pub(crate) fn text_area_scroll_paint_layout_with_content_hint(
        &self,
        path: &Path,
        state: text::view::TextViewState,
        text_engine: &mut text::layout::Engine,
        now: Instant,
        content_hint: Option<(text::layout::AreaScrollKey, area::Logical)>,
    ) -> Option<(
        widget::scroll::Metrics,
        text::layout::TextAreaPaintLayout,
        text::layout::AreaScrollKey,
        area::Logical,
    )> {
        let node = self.node(path)?;
        let layout = self.layout.find_path(path)?;

        self.text_area_scroll_paint_layout_for_node_with_content_hint(
            node,
            layout,
            state,
            text_engine,
            now,
            content_hint,
        )
    }

    pub(crate) fn text_area_scroll_metrics_with_content_hint(
        &self,
        path: &Path,
        state: text::view::TextViewState,
        text_engine: &mut text::layout::Engine,
        now: Instant,
        content_hint: Option<(text::layout::AreaScrollKey, area::Logical)>,
    ) -> Option<(
        widget::scroll::Metrics,
        text::layout::AreaScrollKey,
        area::Logical,
    )> {
        let node = self.node(path)?;
        let layout = self.layout.find_path(path)?;

        self.text_area_scroll_metrics_for_node_with_content_hint(
            node,
            layout,
            state,
            text_engine,
            now,
            content_hint,
        )
    }

    pub(crate) fn text_area_scroll_key(&self, path: &Path) -> Option<text::layout::AreaScrollKey> {
        let node = self.node(path)?;
        let layout = self.layout.find_path(path)?;
        let area_model = node.text_area()?;
        let style = node
            .label()
            .and_then(text::document::Document::first_style)
            .unwrap_or_default();
        let viewport_base = text_content_rect(node, layout);
        let (key, _) =
            text::layout::text_area_scroll_base_content_area(area_model, style, viewport_base.area);

        Some(key)
    }

    pub fn text_area_scroll_y_for_anchor(
        &self,
        path: &Path,
        state: text::view::TextViewState,
        anchor: text::view::ScrollAnchor,
        text_engine: &mut text::layout::Engine,
    ) -> Option<f32> {
        let node = self.node(path)?;
        let surface = node.text_surface()?;
        let text::Surface::Area(area_model) = surface else {
            return None;
        };
        let style = node
            .label()
            .and_then(text::document::Document::first_style)
            .unwrap_or_default();
        let layout = self.layout.find_path(path)?;
        let (metrics, _, _) = self.text_area_scroll_metrics_for_node_with_content_hint(
            node,
            layout,
            state.clone(),
            text_engine,
            Instant::now(),
            None,
        )?;
        let resolved_state = state.with_scroll(metrics.offset().x(), metrics.offset().y());
        let scroll_y = text_engine.text_area_scroll_y_for_anchor(
            area_model,
            style,
            metrics.viewport().area,
            resolved_state.clone(),
            anchor,
        )?;
        Some(
            metrics
                .clamp_offset(point::logical(resolved_state.scroll_x(), scroll_y))
                .y(),
        )
    }

    pub fn ensure_caret_visible_for_text_surface(
        &self,
        path: &Path,
        state: text::view::TextViewState,
        text_engine: &mut text::layout::Engine,
    ) -> Option<text::view::TextViewState> {
        let node = self.node(path)?;
        let surface = node.text_surface()?;
        let style = node
            .label()
            .and_then(text::document::Document::first_style)
            .unwrap_or_default();

        if let text::Surface::Area(area_model) = surface {
            let layout = self.layout.find_path(path)?;
            let (metrics, paint_layout) = self.text_area_scroll_paint_layout_for_node(
                node,
                layout,
                state.clone(),
                text_engine,
                Instant::now(),
            )?;
            let resolved_state = state.with_scroll(metrics.offset().x(), metrics.offset().y());
            return Some(text_engine.ensure_caret_visible_for_area(
                area_model,
                style,
                metrics.viewport().area,
                resolved_state,
                Some(paint_layout.layout()),
            ));
        }

        let rect = self.text_content_rect_for_state(path, state.clone(), text_engine)?;
        Some(text_engine.ensure_caret_visible_for_surface(surface, style, rect.area, state))
    }

    fn text_content_rect_for_state(
        &self,
        path: &Path,
        state: text::view::TextViewState,
        text_engine: &mut text::layout::Engine,
    ) -> Option<Rect> {
        let node = self.node(path)?;
        let layout = self.layout.find_path(path)?;
        let base = text_content_rect(node, layout);

        if node.text_area().is_none() {
            return Some(base);
        }

        self.text_area_scroll_metrics_for_node(node, layout, state, text_engine, Instant::now())
            .map(|metrics| metrics.viewport())
            .or(Some(base))
    }

    fn text_area_scroll_metrics_for_node(
        &self,
        node: &Node,
        layout: &Frame,
        state: text::view::TextViewState,
        text_engine: &mut text::layout::Engine,
        now: Instant,
    ) -> Option<widget::scroll::Metrics> {
        self.text_area_scroll_metrics_for_node_with_content_hint(
            node,
            layout,
            state,
            text_engine,
            now,
            None,
        )
        .map(|(metrics, _, _)| metrics)
    }

    fn text_area_scroll_paint_layout_for_node(
        &self,
        node: &Node,
        layout: &Frame,
        state: text::view::TextViewState,
        text_engine: &mut text::layout::Engine,
        now: Instant,
    ) -> Option<(widget::scroll::Metrics, text::layout::TextAreaPaintLayout)> {
        self.text_area_scroll_paint_layout_for_node_with_content_hint(
            node,
            layout,
            state,
            text_engine,
            now,
            None,
        )
        .map(|(metrics, paint_layout, _, _)| (metrics, paint_layout))
    }

    fn text_area_scroll_paint_layout_for_node_with_content_hint(
        &self,
        node: &Node,
        layout: &Frame,
        state: text::view::TextViewState,
        text_engine: &mut text::layout::Engine,
        now: Instant,
        content_hint: Option<(text::layout::AreaScrollKey, area::Logical)>,
    ) -> Option<(
        widget::scroll::Metrics,
        text::layout::TextAreaPaintLayout,
        text::layout::AreaScrollKey,
        area::Logical,
    )> {
        let (metrics, key, content_area) = self
            .text_area_scroll_metrics_for_node_with_content_hint(
                node,
                layout,
                state.clone(),
                text_engine,
                now,
                content_hint,
            )?;
        let area_model = node.text_area()?;
        let style = node
            .label()
            .and_then(text::document::Document::first_style)
            .unwrap_or_default();
        let viewport = widget::scroll::viewport_rect_for_axes(
            text_content_rect(node, layout),
            node.scroll()?.style(),
            metrics.active_axes(),
        );
        let resolved_state = state.with_scroll(metrics.offset().x(), metrics.offset().y());
        let paint_layout = text_engine.text_area_paint_layout_for_area_at(
            area_model,
            style,
            viewport.area,
            resolved_state,
            now,
        );

        Some((metrics, paint_layout, key, content_area))
    }

    fn text_area_scroll_metrics_for_node_with_content_hint(
        &self,
        node: &Node,
        layout: &Frame,
        state: text::view::TextViewState,
        text_engine: &mut text::layout::Engine,
        now: Instant,
        content_hint: Option<(text::layout::AreaScrollKey, area::Logical)>,
    ) -> Option<(
        widget::scroll::Metrics,
        text::layout::AreaScrollKey,
        area::Logical,
    )> {
        let area_model = node.text_area()?;
        let scroll = node.scroll()?;
        if !scroll.axes().is_enabled() {
            return None;
        }

        let viewport_base = text_content_rect(node, layout);
        let style = node
            .label()
            .and_then(text::document::Document::first_style)
            .unwrap_or_default();
        let (key, base_content) =
            text::layout::text_area_scroll_base_content_area(area_model, style, viewport_base.area);
        let hint = content_hint
            .filter(|(hint_key, _)| *hint_key == key)
            .map(|(_, content)| content);
        let stable_hint =
            hint.is_some() && state.preedit().is_none() && !state.caret_visibility_pending();
        let mut content_area = text::layout::stable_text_area_content_area(
            area_model.wrap(),
            base_content,
            hint,
            area::logical(0.0, 0.0),
            viewport_base.area,
        );
        let mut axes =
            scroll
                .bars()
                .active_axes(scroll.axes(), viewport_base, scroll.style(), content_area);
        if !stable_hint {
            for _ in 0..3 {
                let viewport =
                    widget::scroll::viewport_rect_for_axes(viewport_base, scroll.style(), axes);
                let candidate = text_engine.text_area_metrics_layout_for_area_at(
                    area_model,
                    style,
                    viewport.area,
                    state.clone(),
                    now,
                );
                let next_content = text::layout::stable_text_area_content_area(
                    area_model.wrap(),
                    base_content,
                    Some(content_area),
                    candidate.content_area(),
                    viewport.area,
                );
                let next_axes = scroll.bars().active_axes(
                    scroll.axes(),
                    viewport_base,
                    scroll.style(),
                    next_content,
                );

                content_area = next_content;

                if next_axes == axes {
                    break;
                }

                axes = next_axes;
            }
        }

        if state.caret_visibility_pending() {
            let viewport =
                widget::scroll::viewport_rect_for_axes(viewport_base, scroll.style(), axes);
            let width = match area_model.wrap() {
                text::AreaWrap::None => content_area
                    .width()
                    .max(state.scroll_x() + viewport.area.width()),
                text::AreaWrap::WordOrGlyph => viewport.area.width().max(0.0),
            };
            content_area = area::logical(
                width,
                content_area
                    .height()
                    .max(state.scroll_y() + viewport.area.height()),
            );
        }

        let metrics = widget::scroll::Metrics::resolve(
            layout.rect(),
            viewport_base,
            content_area,
            point::logical(state.scroll_x(), state.scroll_y()),
            scroll.axes(),
            scroll.bars(),
            scroll.style(),
        );

        Some((metrics, key, content_area))
    }

    pub fn interactivity(&self, path: &Path) -> Option<Interactivity> {
        let interactivity = self.interactivity.get(path).copied()?;
        Some(match self.path_states.get(path).copied() {
            Some(state) if !state.is_enabled() || state.is_busy() => Interactivity::NONE,
            Some(_) | None => interactivity,
        })
    }

    pub fn interactivity_map(&self) -> &HashMap<Path, Interactivity> {
        &self.interactivity
    }

    pub fn path_state(&self, path: &Path) -> Option<action::State> {
        self.path_states.get(path).copied()
    }

    pub(crate) fn set_path_states(&mut self, path_states: HashMap<Path, action::State>) -> bool {
        if self.path_states == path_states {
            return false;
        }

        self.path_states = path_states;
        self.focus_order = focus_order(&self.layout, &self.effective_interactivity());
        true
    }

    fn effective_interactivity(&self) -> HashMap<Path, Interactivity> {
        self.interactivity
            .iter()
            .map(|(path, interactivity)| {
                let interactivity = match self.path_states.get(path).copied() {
                    Some(state) if !state.is_enabled() || state.is_busy() => Interactivity::NONE,
                    Some(_) | None => *interactivity,
                };

                (path.clone(), interactivity)
            })
            .collect()
    }

    pub fn cursor(&self, path: &Path) -> Cursor {
        self.cursors.get(path).copied().unwrap_or_default()
    }

    pub fn widget_metrics(&self, path: &Path) -> Option<widget::Metrics> {
        self.widget_metrics.get(path).copied()
    }

    pub fn widget_metrics_iter(&self) -> impl Iterator<Item = (&Path, &widget::Metrics)> {
        self.widget_metrics.iter()
    }

    pub fn focus_order(&self) -> &[Path] {
        &self.focus_order
    }

    pub fn paint<T>(
        &self,
        actions: &action::Registry<T>,
        window: window::Id,
        interaction: Interaction,
        text_field_states: &HashMap<Path, text::view::TextViewState>,
        text_engine: &mut text::layout::Engine,
        scene: &mut paint::Scene,
    ) {
        self.paint_at(
            actions,
            window,
            interaction,
            text_field_states,
            text_engine,
            AnimationFrame::new(Instant::now(), None),
            None,
            scene,
        );
    }

    pub(crate) fn paint_at<T>(
        &self,
        actions: &action::Registry<T>,
        window: window::Id,
        interaction: Interaction,
        text_field_states: &HashMap<Path, text::view::TextViewState>,
        text_engine: &mut text::layout::Engine,
        frame: AnimationFrame,
        scroll_projections: Option<&scroll::Driver>,
        scene: &mut paint::Scene,
    ) {
        self.tree.paint_with_scroll_projections_at(
            &self.layout,
            actions,
            window,
            interaction,
            text_field_states,
            text_engine,
            frame,
            scroll_projections,
            &self.path_states,
            scene,
        );
    }

    pub(crate) fn paint_at_recording_scroll_ranges<T>(
        &self,
        actions: &action::Registry<T>,
        window: window::Id,
        interaction: Interaction,
        text_field_states: &HashMap<Path, text::view::TextViewState>,
        text_engine: &mut text::layout::Engine,
        frame: AnimationFrame,
        scroll_projections: Option<&scroll::Driver>,
        scene: &mut paint::Scene,
    ) -> painting::ScrollPaintRecords {
        let mut ranges = painting::ScrollPaintRecords::default();
        self.tree.paint_with_scroll_projections_recording_at(
            &self.layout,
            actions,
            window,
            interaction,
            text_field_states,
            text_engine,
            frame,
            scroll_projections,
            &self.path_states,
            scene,
            Some(&mut ranges),
        );
        ranges
    }

    pub(crate) fn paint_scroll_target_recording_at<T>(
        &self,
        target: &Path,
        actions: &action::Registry<T>,
        window: window::Id,
        interaction: Interaction,
        text_field_states: &HashMap<Path, text::view::TextViewState>,
        text_engine: &mut text::layout::Engine,
        frame: AnimationFrame,
        scroll_projections: Option<&scroll::Driver>,
        scene: &mut paint::Scene,
    ) -> Option<painting::ScrollPaintRecords> {
        let node = self.node(target)?;
        let layout = self.layout.find_path(target)?;

        Some(painting::scroll_subtree_recording(
            node,
            layout,
            actions,
            window,
            &interaction,
            text_field_states,
            text_engine,
            frame,
            scroll_projections,
            &self.path_states,
            scene,
        ))
    }

    fn node(&self, path: &Path) -> Option<&Node> {
        let root = self.tree.root.as_ref()?;

        node_at_path(root, path.ids())
    }

    #[cfg(test)]
    pub fn for_test(
        layout: Frame,
        menus: HashMap<menu::Id, menu::Menu>,
        actions: HashMap<Path, action::Id>,
        command_subjects: HashMap<Path, CommandSubject>,
        intents: HashMap<Path, Intent>,
        responders: HashMap<Path, Vec<action::Id>>,
        command_scopes: Vec<Path>,
        interactivity: HashMap<Path, Interactivity>,
        widget_metrics: HashMap<Path, widget::Metrics>,
        focus_order: Vec<Path>,
    ) -> Self {
        let responder_bindings = responders
            .iter()
            .map(|(path, responders)| {
                (
                    path.clone(),
                    responders
                        .iter()
                        .copied()
                        .map(action::Binding::new)
                        .collect(),
                )
            })
            .collect();

        Self {
            tree: Tree::new(),
            layout,
            open_menu: None,
            open_submenu: None,
            menus,
            actions,
            command_subjects,
            intents,
            responders,
            responder_bindings,
            command_scopes,
            command_scope_contexts: HashMap::new(),
            text_fields: HashMap::new(),
            text_surfaces: HashMap::new(),
            interactivity,
            path_states: HashMap::new(),
            cursors: HashMap::new(),
            widget_metrics,
            focus_order,
        }
    }
}

#[derive(Default)]
struct TreeIndex {
    actions: HashMap<Path, action::Id>,
    command_subjects: HashMap<Path, CommandSubject>,
    intents: HashMap<Path, Intent>,
    responders: HashMap<Path, Vec<action::Id>>,
    responder_bindings: HashMap<Path, Vec<action::Binding>>,
    command_scopes: Vec<Path>,
    text_fields: HashMap<Path, text::Field>,
    text_surfaces: HashMap<Path, text::Surface>,
    interactivity: HashMap<Path, Interactivity>,
    cursors: HashMap<Path, Cursor>,
}

impl TreeIndex {
    fn collect_node(&mut self, node: &Node, path: &Path) {
        if let Some(action) = node.action() {
            self.actions.insert(path.clone(), action);
            self.command_subjects
                .insert(path.clone(), node.command_subject());
        }

        if let Some(intent) = node.intent() {
            self.intents.insert(path.clone(), intent);
        }

        if !node.responders().is_empty() {
            self.responders
                .insert(path.clone(), node.responders().to_vec());
        }

        if !node.responder_bindings().is_empty() {
            self.responder_bindings
                .insert(path.clone(), node.responder_bindings().to_vec());
        }

        if node.is_command_scope() {
            self.command_scopes.push(path.clone());
        }

        if let Some(surface) = node.text_surface() {
            if let Some(field) = surface.as_field() {
                self.text_fields.insert(path.clone(), field.clone());
            }
            self.text_surfaces.insert(path.clone(), surface.clone());
        }

        self.interactivity
            .insert(path.clone(), node.interactivity());
        self.cursors.insert(path.clone(), node.cursor());

        for child in node.children() {
            self.collect_node(child, &path.child(child.id()));
        }
    }
}

fn collect_menus(node: &Node, menus: &mut HashMap<menu::Id, menu::Menu>) {
    if let Some(bar) = node.menu_bar() {
        for menu in bar.menus() {
            collect_menu(menu, menus);
        }
    }

    for child in node.children() {
        collect_menus(child, menus);
    }
}

fn collect_menu(menu: &menu::Menu, menus: &mut HashMap<menu::Id, menu::Menu>) {
    menus.insert(menu.id(), menu.clone());

    for node in menu.sections().iter().flat_map(menu::Section::nodes) {
        if let menu::Node::Submenu(submenu) = node {
            collect_menu(submenu, menus);
        }
    }
}

fn collect_widget_metrics(
    node: &Node,
    layout: &Frame,
    metrics: &mut HashMap<Path, widget::Metrics>,
) {
    if node.text_area().is_none()
        && let Some(scroll_metrics) = widget::scroll::metrics(node, layout)
    {
        metrics.insert(
            layout.path().clone(),
            widget::Metrics::Scroll(scroll_metrics),
        );
    }

    for (child, child_layout) in node.children().iter().zip(layout.children()) {
        collect_widget_metrics(child, child_layout, metrics);
    }
}
