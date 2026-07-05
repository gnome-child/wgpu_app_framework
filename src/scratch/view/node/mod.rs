use super::super::interaction;
use super::{binding::Binding, control::Control, style::Style};

mod access;
mod action;
mod axis;
mod builder;
mod role;
mod traversal;

pub use axis::Axis;
pub use role::Role;

#[derive(Clone)]
pub struct Node {
    role: Role,
    id: Option<interaction::Id>,
    axis: Option<Axis>,
    style: Style,
    label: Option<String>,
    binding: Option<Binding>,
    control: Option<Control>,
    children: Vec<Node>,
}
