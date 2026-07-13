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
    World(WorldText),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WorldText {
    wrap: super::control::Wrap,
    overflow: text::Overflow,
    align: super::style::Align,
}

impl WorldText {
    fn new(wrap: super::control::Wrap, overflow: text::Overflow) -> Self {
        Self {
            wrap,
            overflow,
            align: super::style::Align::Start,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Participation {
    MenuRow,
    PaletteRow,
    Table(TablePart),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TablePart {
    Header,
    HeaderControl,
    Cell,
    Editor,
    PassiveToggle,
    Toggle,
    Action,
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
    menu_anchor: Option<crate::geometry::PlacementAnchor>,
    menu_available: Option<crate::geometry::Rect>,
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
    active_item: bool,
    scroll_offset: interaction::ScrollOffset,
    virtual_list: Option<virtual_list::Model>,
    provided_row: Option<ProvidedRow>,
    table_row: Option<crate::table::Row>,
    table_cell: Option<crate::table::Cell>,
    table_header_cell: Option<crate::table::HeaderCell>,
    table_header_presentation: Option<crate::table::HeaderPresentation>,
    table_model: Option<crate::table::Model>,
    table_edit: Option<crate::table::Edit>,
    table_edit_error: Option<String>,
    participation: Option<Participation>,
    context_menu: bool,
    children: Vec<Node>,
}

#[derive(Debug, Clone, Copy)]
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
