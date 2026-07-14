use super::Id;
use crate::{composition, geometry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Origin {
    Authored,
    Context {
        owner: composition::tree::NodeId,
        anchor: geometry::Point,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Menu {
    id: Id,
    label: String,
    origin: Origin,
}

impl Menu {
    pub fn new(id: impl Into<Id>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            origin: Origin::Authored,
        }
    }

    pub(crate) fn context(owner: composition::tree::NodeId, anchor: geometry::Point) -> Self {
        Self {
            id: Self::context_id(),
            label: "Context menu".to_owned(),
            origin: Origin::Context { owner, anchor },
        }
    }

    pub(crate) fn context_id() -> Id {
        Id::new("context_menu")
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub(crate) fn context_owner(&self) -> Option<composition::tree::NodeId> {
        match self.origin {
            Origin::Authored => None,
            Origin::Context { owner, .. } => Some(owner),
        }
    }

    pub(crate) fn context_anchor(&self) -> Option<geometry::Point> {
        match self.origin {
            Origin::Authored => None,
            Origin::Context { anchor, .. } => Some(anchor),
        }
    }

    pub(crate) fn is_context(&self) -> bool {
        matches!(self.origin, Origin::Context { .. })
    }
}
