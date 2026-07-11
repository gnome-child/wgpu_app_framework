use super::super::interaction;
use super::{binding::Binding, control::Control, style::Style};
use crate::virtual_list;
use crate::{subject, text};

mod access;
mod action;
mod axis;
mod builder;
mod role;
mod traversal;

pub use axis::Axis;
pub(crate) use role::Role;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextKind {
    Author,
    World(text::Overflow),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatingPlacement {
    Default,
    CenteredMaxEnvelope,
    Offset { x: i32, y: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativePopupMaterialPreference {
    System,
    OpaqueFallback,
    NoAccent,
}

#[derive(Clone)]
pub struct Node {
    role: Role,
    id: Option<interaction::Id>,
    axis: Option<Axis>,
    style: Style,
    floating_placement: FloatingPlacement,
    force_overlay_group: bool,
    native_popup_material_preference: NativePopupMaterialPreference,
    subject: Option<subject::Segment>,
    label: Option<String>,
    text_kind: TextKind,
    binding: Option<Binding>,
    control: Option<Control>,
    focused: bool,
    focus_visible: bool,
    selected: bool,
    scroll_offset: interaction::ScrollOffset,
    virtual_list: Option<virtual_list::Model>,
    provided_row: Option<ProvidedRow>,
    children: Vec<Node>,
}

#[derive(Clone, Copy)]
pub(crate) struct ProvidedRow {
    list: interaction::Id,
    key: virtual_list::Key,
    index: usize,
}

impl ProvidedRow {
    pub(crate) fn list(self) -> interaction::Id {
        self.list
    }

    pub(crate) fn key(self) -> virtual_list::Key {
        self.key
    }

    pub(crate) fn index(self) -> usize {
        self.index
    }
}
