use crate::action;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(&'static str);

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
    action: action::Id,
    label: Option<String>,
}

impl Id {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
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
    pub fn new(id: Id, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            sections: Vec::new(),
        }
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

    pub fn actions(&self) -> impl Iterator<Item = action::Id> + '_ {
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

    fn collect_actions(&self, actions: &mut Vec<action::Id>) {
        for section in self.sections() {
            section.collect_actions(actions);
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

    pub fn action(self, action: action::Id) -> Self {
        self.item(Item::new(action))
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

    pub fn actions(&self) -> impl Iterator<Item = action::Id> + '_ {
        let mut actions = Vec::new();
        self.collect_actions(&mut actions);

        actions.into_iter()
    }

    fn collect_actions(&self, actions: &mut Vec<action::Id>) {
        for node in &self.nodes {
            match node {
                Node::Item(item) => actions.push(item.action()),
                Node::Submenu(menu) => menu.collect_actions(actions),
                Node::Separator => {}
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
    pub fn new(action: action::Id) -> Self {
        Self {
            action,
            label: None,
        }
    }

    pub fn action(&self) -> action::Id {
        self.action
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: Id = Id::new("file");
    const EDIT: Id = Id::new("edit");
    const VIEW: Id = Id::new("view");
    const OPEN: action::Id = action::Id::new("open");
    const SAVE: action::Id = action::Id::new("save");

    #[test]
    fn menu_builders_preserve_order_sections_items_and_separators() {
        let bar = Bar::new()
            .menu(
                Menu::new(FILE, "File").section(
                    Section::new()
                        .item(Item::new(OPEN).with_label("Open File"))
                        .separator()
                        .action(SAVE),
                ),
            )
            .menu(Menu::new(EDIT, "Edit"));

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
            bar.menus()[0].actions().collect::<Vec<_>>(),
            vec![OPEN, SAVE]
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
            Menu::new(VIEW, "View").section(
                Section::new().submenu(
                    Menu::new(panels, "Panels").section(
                        Section::new()
                            .action(OPEN)
                            .separator()
                            .item(Item::new(SAVE).with_label("Save Layout")),
                    ),
                ),
            ),
        );

        assert_eq!(bar.find(panels).expect("submenu").label(), "Panels");
        assert_eq!(
            bar.find(VIEW)
                .expect("view menu")
                .actions()
                .collect::<Vec<_>>(),
            vec![OPEN, SAVE]
        );
    }
}
