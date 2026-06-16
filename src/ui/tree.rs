use std::collections::HashMap;

use crate::geometry::area;
use crate::widget::menu;
use crate::{action, layout_old, paint, text, widget, window};

use super::{
    ActionTarget, Intent, Interaction, Interactivity, Node, Path, layout_engine, painting,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Tree {
    root: Option<Node>,
    popups: Vec<widget::Popup>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Composition {
    tree: Tree,
    layout: layout_old::Box,
    open_menu: Option<menu::Id>,
    open_submenu: Option<menu::Id>,
    menus: HashMap<menu::Id, menu::Menu>,
    actions: HashMap<Path, action::Id>,
    action_targets: HashMap<Path, ActionTarget>,
    intents: HashMap<Path, Intent>,
    responders: HashMap<Path, Vec<action::Id>>,
    command_scopes: Vec<Path>,
    interactivity: HashMap<Path, Interactivity>,
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
        measurer: &mut text::Measurer,
    ) -> Option<layout_old::Box> {
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
        area: area::Logical,
        actions: &action::Registry<T>,
        command_target: &action::Context,
        open_menu: Option<menu::Id>,
        open_submenu: Option<menu::Id>,
        measurer: &mut text::Measurer,
    ) -> Option<Composition> {
        let mut tree = self.clone();
        let menus = tree.menus();
        let open_menu = open_menu.filter(|menu| menus.contains_key(menu));
        let open_submenu =
            open_submenu.filter(|menu| open_menu.is_some() && menus.contains_key(menu));

        let mut menu_popup_inserted = false;
        if let Some(open_menu) = open_menu
            && let Some(menu) = menus.get(&open_menu)
            && let Some(base_layout) = tree.layout(area, measurer)
            && let Some(popup) =
                widget::menu_popup(&tree, &base_layout, menu, actions, command_target, measurer)
        {
            tree.push_popup(popup);
            menu_popup_inserted = true;
        }

        if menu_popup_inserted
            && let Some(open_submenu) = open_submenu
            && let Some(menu) = menus.get(&open_submenu)
            && let Some(menu_layout) = tree.layout(area, measurer)
            && let Some(popup) =
                widget::submenu_popup(&tree, &menu_layout, menu, actions, command_target, measurer)
        {
            tree.push_popup(popup);
        }

        let layout = tree.layout(area, measurer)?;

        Some(Composition::new(
            tree,
            layout,
            open_menu,
            open_submenu,
            menus,
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

    pub fn action_targets(&self) -> HashMap<Path, ActionTarget> {
        self.index().action_targets
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

    pub fn command_scopes(&self) -> Vec<Path> {
        self.index().command_scopes
    }

    pub fn interactivity(&self) -> HashMap<Path, Interactivity> {
        self.index().interactivity
    }

    pub fn widget_metrics(&self, layout: &layout_old::Box) -> HashMap<Path, widget::Metrics> {
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
        layout: &layout_old::Box,
        actions: &action::Registry<T>,
        window: window::Id,
        interaction: Interaction,
        scene: &mut paint::Scene,
    ) {
        if let Some(root) = self.root.as_ref() {
            painting::tree(root, layout, actions, window, &interaction, scene);
            for popup in &self.popups {
                let path = layout.path().child(popup.root().id());
                if let Some(popup_layout) = layout.find_path(&path) {
                    painting::tree(
                        popup.root(),
                        popup_layout,
                        actions,
                        window,
                        &interaction,
                        scene,
                    );
                }
            }
        }
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

fn focus_order(
    layout: &layout_old::Box,
    interactivity: &HashMap<Path, Interactivity>,
) -> Vec<Path> {
    let mut order = Vec::new();
    collect_focus_order(layout, interactivity, &mut order);
    order
}

fn collect_focus_order(
    layout: &layout_old::Box,
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

impl Composition {
    fn new(
        tree: Tree,
        layout: layout_old::Box,
        open_menu: Option<menu::Id>,
        open_submenu: Option<menu::Id>,
        menus: HashMap<menu::Id, menu::Menu>,
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
            action_targets: index.action_targets,
            intents: index.intents,
            responders: index.responders,
            command_scopes: index.command_scopes,
            interactivity: index.interactivity,
            widget_metrics,
            focus_order,
        }
    }

    pub fn layout(&self) -> &layout_old::Box {
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

    pub fn action_target(&self, path: &Path) -> ActionTarget {
        self.action_targets.get(path).copied().unwrap_or_default()
    }

    pub fn action_targets(&self) -> &HashMap<Path, ActionTarget> {
        &self.action_targets
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

    pub fn has_responder(&self, path: &Path) -> bool {
        self.responders
            .get(path)
            .is_some_and(|actions| !actions.is_empty())
    }

    pub fn command_scopes(&self) -> &[Path] {
        &self.command_scopes
    }

    pub fn interactivity(&self, path: &Path) -> Option<Interactivity> {
        self.interactivity.get(path).copied()
    }

    pub fn interactivity_map(&self) -> &HashMap<Path, Interactivity> {
        &self.interactivity
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
        scene: &mut paint::Scene,
    ) {
        self.tree
            .paint(&self.layout, actions, window, interaction, scene);
    }

    #[cfg(test)]
    pub fn for_test(
        layout: layout_old::Box,
        menus: HashMap<menu::Id, menu::Menu>,
        actions: HashMap<Path, action::Id>,
        action_targets: HashMap<Path, ActionTarget>,
        intents: HashMap<Path, Intent>,
        responders: HashMap<Path, Vec<action::Id>>,
        command_scopes: Vec<Path>,
        interactivity: HashMap<Path, Interactivity>,
        widget_metrics: HashMap<Path, widget::Metrics>,
        focus_order: Vec<Path>,
    ) -> Self {
        Self {
            tree: Tree::new(),
            layout,
            open_menu: None,
            open_submenu: None,
            menus,
            actions,
            action_targets,
            intents,
            responders,
            command_scopes,
            interactivity,
            widget_metrics,
            focus_order,
        }
    }
}

#[derive(Default)]
struct TreeIndex {
    actions: HashMap<Path, action::Id>,
    action_targets: HashMap<Path, ActionTarget>,
    intents: HashMap<Path, Intent>,
    responders: HashMap<Path, Vec<action::Id>>,
    command_scopes: Vec<Path>,
    interactivity: HashMap<Path, Interactivity>,
}

impl TreeIndex {
    fn collect_node(&mut self, node: &Node, path: &Path) {
        if let Some(action) = node.action() {
            self.actions.insert(path.clone(), action);
            self.action_targets
                .insert(path.clone(), node.action_target());
        }

        if let Some(intent) = node.intent() {
            self.intents.insert(path.clone(), intent);
        }

        if !node.responders().is_empty() {
            self.responders
                .insert(path.clone(), node.responders().to_vec());
        }

        if node.is_command_scope() {
            self.command_scopes.push(path.clone());
        }

        self.interactivity
            .insert(path.clone(), node.interactivity());

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
    layout: &layout_old::Box,
    metrics: &mut HashMap<Path, widget::Metrics>,
) {
    if let Some(scroll_metrics) = widget::scroll::metrics(node, layout) {
        metrics.insert(
            layout.path().clone(),
            widget::Metrics::Scroll(scroll_metrics),
        );
    }

    for (child, child_layout) in node.children().iter().zip(layout.children()) {
        collect_widget_metrics(child, child_layout, metrics);
    }
}
