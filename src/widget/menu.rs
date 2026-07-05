use std::fmt;

use crate::{Command, action, command};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(Repr);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Repr {
    Named(&'static str),
    Structural(u64),
    Auto,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bar {
    menus: Vec<Menu>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Menu {
    id: Id,
    label: String,
    sections: Vec<Section>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    nodes: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Item(Item),
    Submenu(Menu),
    Separator,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    route: action::Route,
    label: Option<String>,
}

impl Id {
    pub const fn new(value: &'static str) -> Self {
        Self(Repr::Named(value))
    }

    const fn auto() -> Self {
        Self(Repr::Auto)
    }

    const fn structural(value: u64) -> Self {
        Self(Repr::Structural(value))
    }

    pub const fn as_str(self) -> &'static str {
        match self.0 {
            Repr::Named(value) => value,
            Repr::Structural(_) | Repr::Auto => "__menu",
        }
    }

    fn is_auto(self) -> bool {
        matches!(self.0, Repr::Auto)
    }
}

impl Bar {
    pub fn new() -> Self {
        Self { menus: Vec::new() }
    }

    pub fn menu(mut self, menu: Menu) -> Self {
        self.menus.push(menu);
        self
    }

    pub(crate) fn with_structural_ids(mut self) -> Self {
        for (index, menu) in self.menus.iter_mut().enumerate() {
            menu.resolve_structural_ids(structural_child(MENU_ROOT, index));
        }

        self
    }

    pub fn menus(&self) -> &[Menu] {
        &self.menus
    }

    pub fn find(&self, id: Id) -> Option<&Menu> {
        self.menus.iter().find_map(|menu| menu.find(id))
    }
}

impl Default for Bar {
    fn default() -> Self {
        Self::new()
    }
}

impl Menu {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: Id::auto(),
            label: label.into(),
            sections: Vec::new(),
        }
    }

    pub fn key(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn section(mut self, section: Section) -> Self {
        self.sections.push(section);
        self
    }

    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    pub(crate) fn actions(&self) -> impl Iterator<Item = action::Route> + '_ {
        let mut actions = Vec::new();
        self.collect_actions(&mut actions);

        actions.into_iter()
    }

    pub fn find(&self, id: Id) -> Option<&Menu> {
        if self.id == id {
            return Some(self);
        }

        self.sections()
            .iter()
            .flat_map(Section::nodes)
            .find_map(|node| match node {
                Node::Submenu(menu) => menu.find(id),
                Node::Item(_) | Node::Separator => None,
            })
    }

    fn collect_actions(&self, actions: &mut Vec<action::Route>) {
        for section in self.sections() {
            section.collect_actions(actions);
        }
    }

    fn resolve_structural_ids(&mut self, structural_id: u64) {
        if self.id.is_auto() {
            self.id = Id::structural(structural_id);
        }

        for (section_index, section) in self.sections.iter_mut().enumerate() {
            section.resolve_structural_ids(structural_child(structural_id, section_index));
        }
    }
}

impl Section {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn item(mut self, item: Item) -> Self {
        self.nodes.push(Node::Item(item));
        self
    }

    pub fn invokes<C, TTarget>(self) -> Self
    where
        C: Command,
        TTarget: command::Target<C> + 'static,
    {
        self.item(Item::invokes::<C, TTarget>())
    }

    pub fn text<C>(self) -> Self
    where
        C: crate::text::command::EditCommand,
    {
        self.item(Item::text::<C>())
    }

    #[cfg(test)]
    pub(crate) fn command_key(self, command: command::Key) -> Self {
        self.item(Item::key(command))
    }

    pub fn submenu(mut self, menu: Menu) -> Self {
        self.nodes.push(Node::Submenu(menu));
        self
    }

    pub fn separator(mut self) -> Self {
        self.nodes.push(Node::Separator);
        self
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    fn collect_actions(&self, actions: &mut Vec<action::Route>) {
        for node in &self.nodes {
            match node {
                Node::Item(item) => actions.push(item.route()),
                Node::Submenu(menu) => menu.collect_actions(actions),
                Node::Separator => {}
            }
        }
    }

    fn resolve_structural_ids(&mut self, structural_id: u64) {
        for (node_index, node) in self.nodes.iter_mut().enumerate() {
            if let Node::Submenu(menu) = node {
                menu.resolve_structural_ids(structural_child(structural_id, node_index));
            }
        }
    }
}

impl Default for Section {
    fn default() -> Self {
        Self::new()
    }
}

impl Item {
    pub fn invokes<C, TTarget>() -> Self
    where
        C: Command,
        TTarget: command::Target<C> + 'static,
    {
        Self::from_action(command::binding::Route::invokes::<C, TTarget>().action())
    }

    pub fn text<C>() -> Self
    where
        C: crate::text::command::EditCommand,
    {
        Self::from_action(
            command::binding::Route::new(
                command::Key::of::<C>(),
                crate::text::command::text_target_kind(),
            )
            .action(),
        )
    }

    #[cfg(test)]
    pub(crate) fn key(command: command::Key) -> Self {
        Self::from_action(
            command::binding::Route::new(command, command::target::Kind::command(command)).action(),
        )
    }

    pub(crate) fn from_action(route: action::Route) -> Self {
        Self { route, label: None }
    }

    pub(crate) fn action(&self) -> action::Key {
        self.route.key()
    }

    pub(crate) fn route(&self) -> action::Route {
        self.route
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

const MENU_ROOT: u64 = 0x9d5c_4f2d_1b38_7a61;

fn structural_child(parent: u64, index: usize) -> u64 {
    parent
        .wrapping_mul(0x1000_0000_01b3)
        .wrapping_add(index as u64)
        .wrapping_add(0x517c_c1b7_2722_0a95)
}

impl fmt::Display for Id {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Repr::Named(value) => formatter.write_str(value),
            Repr::Structural(value) => write!(formatter, "__menu_{value:016x}"),
            Repr::Auto => formatter.write_str("__menu_auto"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: Id = Id::new("file");
    const EDIT: Id = Id::new("edit");
    const VIEW: Id = Id::new("view");
    struct Open;
    struct Save;

    impl Command for Open {
        type Args = ();
        type Output = ();

        const NAME: &'static str = "open";
        const DISPLAY: &'static str = "Open";
    }

    impl Command for Save {
        type Args = ();
        type Output = ();

        const NAME: &'static str = "save";
        const DISPLAY: &'static str = "Save";
    }

    const OPEN: command::Key = command::Key::of::<Open>();
    const SAVE: command::Key = command::Key::of::<Save>();

    #[test]
    fn menu_builders_preserve_order_sections_items_and_separators() {
        let bar = Bar::new()
            .menu(
                Menu::new("File").key(FILE).section(
                    Section::new()
                        .item(Item::key(OPEN).with_label("Open File"))
                        .separator()
                        .command_key(SAVE),
                ),
            )
            .menu(Menu::new("Edit").key(EDIT));

        assert_eq!(bar.menus()[0].id(), FILE);
        assert_eq!(bar.menus()[1].id(), EDIT);
        assert_eq!(bar.find(FILE).expect("file menu").label(), "File");
        assert!(matches!(
            bar.menus()[0].sections()[0].nodes()[0],
            Node::Item(_)
        ));
        assert!(matches!(
            bar.menus()[0].sections()[0].nodes()[1],
            Node::Separator
        ));
        assert_eq!(
            bar.menus()[0]
                .actions()
                .map(|route| route.key())
                .collect::<Vec<_>>(),
            vec![OPEN.action(), SAVE.action()]
        );
        let Node::Item(item) = &bar.menus()[0].sections()[0].nodes()[0] else {
            panic!("first node should be an item");
        };
        assert_eq!(item.label(), Some("Open File"));
    }

    #[test]
    fn menu_builders_support_recursive_submenus() {
        let panels = Id::new("panels");
        let bar = Bar::new().menu(
            Menu::new("View").key(VIEW).section(
                Section::new().submenu(
                    Menu::new("Panels").key(panels).section(
                        Section::new()
                            .command_key(OPEN)
                            .separator()
                            .item(Item::key(SAVE).with_label("Save Layout")),
                    ),
                ),
            ),
        );

        assert_eq!(bar.find(panels).expect("submenu").label(), "Panels");
        assert_eq!(
            bar.find(VIEW)
                .expect("view menu")
                .actions()
                .map(|route| route.key())
                .collect::<Vec<_>>(),
            vec![OPEN.action(), SAVE.action()]
        );
    }
}
