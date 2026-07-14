use super::{
    FloatingPlacement, NativePopupMaterialPreference, PanelAttachment, PanelPolicy, Role,
    standard_menu,
};
use crate::{interaction, popup, table, virtual_list};

use super::super::{
    TextCommit,
    control::{Button, Checkbox, Radio, Slider, TextArea, TextBox},
};

#[derive(Clone)]
pub(crate) enum Content {
    Root,
    Stack,
    MenuBar(MenuBar),
    Menu,
    Binding,
    Separator,
    TextArea(TextArea),
    Button(Button),
    Checkbox(Checkbox),
    Radio(Radio),
    Slider(Slider),
    TextBox {
        model: TextBox,
        commit: Option<TextCommit>,
    },
    Scroll(Scroll),
    VirtualList {
        model: virtual_list::Model,
        offset: interaction::ScrollOffset,
    },
    Table,
    Panel,
    FloatingPanel(Panel),
    SectionHeader,
    Label,
}

#[derive(Clone)]
pub(crate) enum MenuBar {
    Ordinary,
    Standard(Vec<standard_menu::Extension>),
}

#[derive(Clone)]
pub(crate) enum Scroll {
    Ordinary {
        offset: interaction::ScrollOffset,
    },
    Table {
        model: table::Model,
        offset: interaction::ScrollOffset,
    },
}

#[derive(Clone)]
pub(crate) struct Panel {
    pub(super) placement: FloatingPlacement,
    pub(super) attachment: Option<PanelAttachment>,
    pub(crate) popup_context: Option<popup::ContextFingerprint>,
    pub(crate) policy: PanelPolicy,
    pub(crate) force_overlay_group: bool,
    pub(crate) native_material: NativePopupMaterialPreference,
}

impl Content {
    pub(super) fn role(&self) -> Role {
        match self {
            Self::Root => Role::Root,
            Self::Stack => Role::Stack,
            Self::MenuBar(_) => Role::MenuBar,
            Self::Menu => Role::Menu,
            Self::Binding => Role::Binding,
            Self::Separator => Role::Separator,
            Self::TextArea(_) => Role::TextArea,
            Self::Button(_) => Role::Button,
            Self::Checkbox(_) => Role::Checkbox,
            Self::Radio(_) => Role::Radio,
            Self::Slider(_) => Role::Slider,
            Self::TextBox { .. } => Role::TextBox,
            Self::Scroll(_) => Role::Scroll,
            Self::VirtualList { .. } => Role::VirtualList,
            Self::Table => Role::Table,
            Self::Panel => Role::Panel,
            Self::FloatingPanel(_) => Role::FloatingPanel,
            Self::SectionHeader => Role::SectionHeader,
            Self::Label => Role::Label,
        }
    }

    pub(super) fn standard_menu_extensions(&self) -> Option<&[standard_menu::Extension]> {
        match self {
            Self::MenuBar(MenuBar::Standard(extensions)) => Some(extensions),
            _ => None,
        }
    }

    pub(super) fn standard_menu_extensions_mut(
        &mut self,
    ) -> Option<&mut Vec<standard_menu::Extension>> {
        match self {
            Self::MenuBar(MenuBar::Standard(extensions)) => Some(extensions),
            _ => None,
        }
    }

    pub(super) fn panel(&self) -> Option<&Panel> {
        match self {
            Self::FloatingPanel(panel) => Some(panel),
            _ => None,
        }
    }

    pub(super) fn panel_mut(&mut self) -> Option<&mut Panel> {
        match self {
            Self::FloatingPanel(panel) => Some(panel),
            _ => None,
        }
    }

    pub(super) fn scroll_offset(&self) -> interaction::ScrollOffset {
        match self {
            Self::Scroll(Scroll::Ordinary { offset } | Scroll::Table { offset, .. })
            | Self::VirtualList { offset, .. } => *offset,
            _ => interaction::ScrollOffset::default(),
        }
    }

    pub(super) fn set_scroll_offset(&mut self, value: interaction::ScrollOffset) {
        match self {
            Self::Scroll(Scroll::Ordinary { offset } | Scroll::Table { offset, .. })
            | Self::VirtualList { offset, .. } => *offset = value,
            _ => debug_assert_eq!(value, interaction::ScrollOffset::default()),
        }
    }

    pub(super) fn virtual_list(&self) -> Option<&virtual_list::Model> {
        match self {
            Self::VirtualList { model, .. } => Some(model),
            _ => None,
        }
    }

    pub(super) fn virtual_list_mut(&mut self) -> Option<&mut virtual_list::Model> {
        match self {
            Self::VirtualList { model, .. } => Some(model),
            _ => None,
        }
    }

    pub(super) fn table_model(&self) -> Option<&table::Model> {
        match self {
            Self::Scroll(Scroll::Table { model, .. }) => Some(model),
            _ => None,
        }
    }

    pub(super) fn text_commit(&self) -> Option<&TextCommit> {
        match self {
            Self::TextBox { commit, .. } => commit.as_ref(),
            _ => None,
        }
    }

    pub(super) fn text_area(&self) -> Option<&TextArea> {
        match self {
            Self::TextArea(model) => Some(model),
            _ => None,
        }
    }

    pub(super) fn text_area_mut(&mut self) -> Option<&mut TextArea> {
        match self {
            Self::TextArea(model) => Some(model),
            _ => None,
        }
    }

    pub(super) fn button(&self) -> Option<&Button> {
        match self {
            Self::Button(model) => Some(model),
            _ => None,
        }
    }

    pub(super) fn checkbox(&self) -> Option<&Checkbox> {
        match self {
            Self::Checkbox(model) => Some(model),
            _ => None,
        }
    }

    pub(super) fn radio(&self) -> Option<&Radio> {
        match self {
            Self::Radio(model) => Some(model),
            _ => None,
        }
    }

    pub(super) fn slider(&self) -> Option<&Slider> {
        match self {
            Self::Slider(model) => Some(model),
            _ => None,
        }
    }

    pub(super) fn text_box(&self) -> Option<&TextBox> {
        match self {
            Self::TextBox { model, .. } => Some(model),
            _ => None,
        }
    }

    pub(super) fn text_box_mut(&mut self) -> Option<&mut TextBox> {
        match self {
            Self::TextBox { model, .. } => Some(model),
            _ => None,
        }
    }
}

impl Panel {
    pub(super) fn interactive() -> Self {
        Self {
            placement: FloatingPlacement::Default,
            attachment: None,
            popup_context: None,
            policy: PanelPolicy::Interactive,
            force_overlay_group: false,
            native_material: NativePopupMaterialPreference::System,
        }
    }
}
