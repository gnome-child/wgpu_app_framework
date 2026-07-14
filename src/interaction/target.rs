use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use super::super::{composition, context::Source};
use super::{CommandPalette, Id, Menu};

#[derive(Debug, Clone)]
pub struct Target {
    kind: Kind,
    identity: Identity,
    label: String,
    source: Option<Source>,
    captures: bool,
}

impl PartialEq for Target {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.identity == other.identity && self.source == other.source
    }
}

impl Eq for Target {}

impl Hash for Target {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.identity.hash(state);
        self.source.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Identity {
    Element(Id),
    TableCell(crate::table::Cell),
    Node {
        id: composition::NodeId,
        element: Option<Id>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Menu,
    Command,
    TextArea,
    Scroll,
    Scrollbar,
    FloatingPanel,
    Label,
    TableDivider,
    Indicator,
}

impl Target {
    pub fn menu(id: impl Into<Id>, label: impl Into<String>) -> Self {
        Self::new(Kind::Menu, id, label)
    }

    pub fn command_element(id: impl Into<Id>, command_name: &'static str, source: Source) -> Self {
        Self {
            kind: Kind::Command,
            identity: Identity::Element(id.into()),
            label: command_name.to_owned(),
            source: Some(source),
            captures: false,
        }
    }

    pub(crate) fn command_node(
        node: composition::NodeId,
        element: Option<Id>,
        command_name: &'static str,
        source: Source,
    ) -> Self {
        Self {
            kind: Kind::Command,
            identity: Identity::Node { id: node, element },
            label: command_name.to_owned(),
            source: Some(source),
            captures: false,
        }
    }

    pub(crate) fn table_cell_editor(cell: crate::table::Cell) -> Self {
        Self {
            kind: Kind::TextArea,
            identity: Identity::TableCell(cell),
            label: "Editable table cell".to_owned(),
            source: None,
            captures: true,
        }
    }

    pub fn text_area_id(id: impl Into<Id>) -> Self {
        let id = id.into();
        Self {
            kind: Kind::TextArea,
            identity: Identity::Element(id),
            label: id.as_str().to_owned(),
            source: None,
            captures: true,
        }
    }

    pub fn scroll(id: impl Into<Id>, label: impl Into<String>) -> Self {
        Self::new(Kind::Scroll, id, label).with_capture()
    }

    pub(crate) fn scroll_node(
        node: composition::NodeId,
        element: Option<Id>,
        label: impl Into<String>,
    ) -> Self {
        Self::node(Kind::Scroll, node, element, label).with_capture()
    }

    pub(crate) fn scrollbar_node(
        node: composition::NodeId,
        element: Option<Id>,
        label: impl Into<String>,
    ) -> Self {
        Self::node(Kind::Scrollbar, node, element, label).with_capture()
    }

    pub fn floating_panel(id: impl Into<Id>, label: impl Into<String>) -> Self {
        Self::new(Kind::FloatingPanel, id, label)
    }

    pub fn label(id: impl Into<Id>, label: impl Into<String>) -> Self {
        Self::new(Kind::Label, id, label)
    }

    pub(crate) fn table_divider_node(node: composition::NodeId, label: impl Into<String>) -> Self {
        Self::node(Kind::TableDivider, node, None, label).with_capture()
    }

    pub(crate) fn indicator(owner: &Self, label: impl Into<String>) -> Self {
        Self {
            kind: Kind::Indicator,
            identity: owner.identity.clone(),
            label: label.into(),
            source: None,
            captures: false,
        }
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn element_id(&self) -> Option<Id> {
        match self.identity {
            Identity::Element(id) => Some(id),
            Identity::TableCell(_) => None,
            Identity::Node { element, .. } => element,
        }
    }

    pub(crate) fn node_id(&self) -> Option<composition::NodeId> {
        match self.identity {
            Identity::Element(_) | Identity::TableCell(_) => None,
            Identity::Node { id, .. } => Some(id),
        }
    }

    pub(crate) fn matches_removed_identity(
        &self,
        removed_nodes: &[composition::NodeId],
        removed_elements: &[Id],
        removed_table_cells: &[crate::table::Cell],
    ) -> bool {
        self.node_id().is_some_and(|id| removed_nodes.contains(&id))
            || self
                .element_id()
                .is_some_and(|id| removed_elements.contains(&id))
            || matches!(self.identity, Identity::TableCell(cell) if removed_table_cells.contains(&cell))
    }

    pub(crate) fn table_cell(&self) -> Option<crate::table::Cell> {
        match self.identity {
            Identity::TableCell(cell) => Some(cell),
            Identity::Element(_) | Identity::Node { .. } => None,
        }
    }

    pub fn focus_key(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn label_text(&self) -> &str {
        &self.label
    }

    pub fn as_menu(&self) -> Option<Menu> {
        if self.kind != Kind::Menu {
            return None;
        }

        Some(Menu::new(self.element_id()?, self.label.clone()))
    }

    pub fn is_menu_surface(&self) -> bool {
        matches!(self.kind, Kind::Menu | Kind::FloatingPanel) || self.source == Some(Source::Menu)
    }

    pub fn is_command_palette_surface(&self) -> bool {
        self.source == Some(Source::Palette)
            || self.element_id() == Some(CommandPalette::panel_id())
            || self.element_id() == CommandPalette::query_focus().target_id()
    }

    pub fn captures(&self) -> bool {
        self.captures
    }

    pub fn with_capture(mut self) -> Self {
        self.captures = true;
        self
    }

    fn new(kind: Kind, id: impl Into<Id>, label: impl Into<String>) -> Self {
        Self {
            kind,
            identity: Identity::Element(id.into()),
            label: label.into(),
            source: None,
            captures: false,
        }
    }

    fn node(
        kind: Kind,
        id: composition::NodeId,
        element: Option<Id>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            identity: Identity::Node { id, element },
            label: label.into(),
            source: None,
            captures: false,
        }
    }
}
