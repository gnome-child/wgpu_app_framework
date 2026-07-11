use super::super::interaction;
use super::{binding::Binding, control::Control, style::Style};
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
    children: Vec<Node>,
}
