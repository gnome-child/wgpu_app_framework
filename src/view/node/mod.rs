use super::super::interaction;
use super::{binding::Binding, control::Control, style::Style};
use crate::{overlay, subject};

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
    Offset { x: i32, y: i32 },
}

#[derive(Clone)]
pub struct Node {
    role: Role,
    id: Option<interaction::Id>,
    axis: Option<Axis>,
    style: Style,
    floating_placement: FloatingPlacement,
    overlay_realization: overlay::MaterialRealization,
    force_overlay_group: bool,
    subject: Option<subject::Segment>,
    label: Option<String>,
    binding: Option<Binding>,
    control: Option<Control>,
    focused: bool,
    focus_visible: bool,
    selected: bool,
    scroll_offset: interaction::ScrollOffset,
    children: Vec<Node>,
}
