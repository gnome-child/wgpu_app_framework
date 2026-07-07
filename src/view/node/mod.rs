use super::super::interaction;
use super::{binding::Binding, control::Control, style::Style};
use crate::subject;

mod access;
mod action;
mod axis;
mod builder;
mod role;
mod traversal;

pub use axis::Axis;
pub(crate) use role::Role;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatingPlacement {
    Default,
    CenteredMaxEnvelope,
}

#[derive(Clone)]
pub struct Node {
    role: Role,
    id: Option<interaction::Id>,
    axis: Option<Axis>,
    style: Style,
    floating_placement: FloatingPlacement,
    subject: Option<subject::Segment>,
    label: Option<String>,
    binding: Option<Binding>,
    control: Option<Control>,
    focused: bool,
    focus_visible: bool,
    hovered: bool,
    pressed: bool,
    active: bool,
    selected: bool,
    scroll_offset: interaction::ScrollOffset,
    children: Vec<Node>,
}
