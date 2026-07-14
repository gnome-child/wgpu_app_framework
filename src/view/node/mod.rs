use super::super::interaction;
use super::{binding::Binding, style::Style};
use crate::virtual_list;
use crate::{subject, text};

mod access;
mod action;
mod axis;
mod builder;
mod content;
mod role;
mod standard_menu;
mod traversal;

pub use axis::Axis;
use content::Content;
pub(crate) use role::Role;
pub(crate) use standard_menu::Extension as StandardMenuExtension;

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
    AuxiliaryText,
    Table(TablePart),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TablePart {
    HeaderBand,
    Header,
    HeaderControl,
    Cell,
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

/// Behavioral policy applied to content that shares the floating-panel path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PanelPolicy {
    Interactive,
    HoverTip,
    WindowFeedback,
}

impl PanelPolicy {
    pub(crate) const fn accepts_input(self) -> bool {
        matches!(self, Self::Interactive)
    }
}

/// The independent geometry source used to attach a floating panel.
///
/// Semantic ownership remains on the target/cell/session that produced the
/// panel; this value answers only where the shared placement request begins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PanelAttachment {
    Geometry(crate::geometry::PlacementAnchor),
    Pointer(crate::geometry::Point),
    Element(interaction::Id),
}

#[derive(Clone)]
pub struct Node {
    content: Content,
    id: Option<interaction::Id>,
    axis: Option<Axis>,
    style: Style,
    subject: Option<subject::Segment>,
    label: Option<String>,
    text_kind: TextKind,
    binding: Option<Binding>,
    context_binding: Option<Binding>,
    focused: bool,
    focus_visible: bool,
    selected: bool,
    active_item: bool,
    provided_row: Option<ProvidedRow>,
    table_row: Option<crate::table::Row>,
    table_cell: Option<crate::table::Cell>,
    table_header_cell: Option<crate::table::HeaderCell>,
    table_header_presentation: Option<crate::table::HeaderPresentation>,
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
