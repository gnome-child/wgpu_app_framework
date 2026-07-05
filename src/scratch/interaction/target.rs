use std::{
    any::TypeId,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use super::super::{context::Source, session};
use super::{Id, Menu};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Target {
    kind: Kind,
    identity: Identity,
    label: String,
    source: Option<Source>,
    captures: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Identity {
    Element(Id),
    CommandPath {
        command_type: TypeId,
        source: Source,
        path: Vec<usize>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Menu,
    Command,
    TextArea,
    Popup,
    Label,
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

    pub fn command_path(
        command_type: TypeId,
        command_name: &'static str,
        source: Source,
        path: impl Into<Vec<usize>>,
    ) -> Self {
        Self {
            kind: Kind::Command,
            identity: Identity::CommandPath {
                command_type,
                source,
                path: path.into(),
            },
            label: command_name.to_owned(),
            source: Some(source),
            captures: false,
        }
    }

    pub fn text_area(focus: session::Focus) -> Self {
        focus.into_target()
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

    pub fn popup(id: impl Into<Id>, label: impl Into<String>) -> Self {
        Self::new(Kind::Popup, id, label)
    }

    pub fn label(id: impl Into<Id>, label: impl Into<String>) -> Self {
        Self::new(Kind::Label, id, label)
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn element_id(&self) -> Option<Id> {
        match self.identity {
            Identity::Element(id) => Some(id),
            Identity::CommandPath { .. } => None,
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
        matches!(self.kind, Kind::Menu | Kind::Popup) || self.source == Some(Source::Menu)
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
}
