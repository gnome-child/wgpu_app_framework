use super::super::interaction;
use super::{binding::Binding, control::Control, style::Style};
use crate::virtual_list;
use crate::{subject, text};

mod access;
mod action;
mod axis;
mod builder;
mod role;
mod standard_menu;
mod traversal;

pub use axis::Axis;
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

/// Behavioral policy applied to content that shares the floating-panel path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PanelPolicy {
    Interactive,
    HoverTip,
    AnchoredFeedback,
    WindowFeedback,
}

impl PanelPolicy {
    pub(crate) const fn accepts_input(self) -> bool {
        matches!(self, Self::Interactive)
    }
}

/// Visual treatment for auxiliary-panel content.
///
/// This is deliberately separate from retained feedback severity: descriptive
/// and overflow content also use the panel recipe without becoming feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuxiliaryChrome {
    Plain,
    Info,
    Warning,
    Error,
}

impl AuxiliaryChrome {
    pub(crate) const fn has_icon(self) -> bool {
        !matches!(self, Self::Plain)
    }
}

impl From<crate::feedback::Severity> for AuxiliaryChrome {
    fn from(severity: crate::feedback::Severity) -> Self {
        match severity {
            crate::feedback::Severity::Info => Self::Info,
            crate::feedback::Severity::Warning => Self::Warning,
            crate::feedback::Severity::Error => Self::Error,
        }
    }
}

#[derive(Clone)]
pub struct Node {
    role: Role,
    id: Option<interaction::Id>,
    axis: Option<Axis>,
    style: Style,
    floating_placement: FloatingPlacement,
    placement_anchor: Option<crate::geometry::PlacementAnchor>,
    placement_available: Option<crate::geometry::Rect>,
    popup_context: Option<crate::popup::ContextFingerprint>,
    panel_policy: PanelPolicy,
    auxiliary_chrome: Option<AuxiliaryChrome>,
    table_panel_anchor: Option<crate::table::Cell>,
    panel_anchor_target: Option<interaction::Target>,
    force_overlay_group: bool,
    native_popup_material_preference: NativePopupMaterialPreference,
    subject: Option<subject::Segment>,
    label: Option<String>,
    text_kind: TextKind,
    binding: Option<Binding>,
    context_binding: Option<Binding>,
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
    standard_menu_bar: bool,
    standard_menu_extensions: Vec<StandardMenuExtension>,
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
