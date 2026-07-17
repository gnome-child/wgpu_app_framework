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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum ScrollAxisPolicy {
    Always,
    Automatic,
    Never,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScrollChromePresentation {
    Overlay,
    Consuming,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScrollSizing {
    Minimum,
    Natural,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScrollDirection {
    LeftToRight,
    RightToLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScrollContainer {
    pub(crate) horizontal_policy: ScrollAxisPolicy,
    pub(crate) vertical_policy: ScrollAxisPolicy,
    pub(crate) chrome: ScrollChromePresentation,
    pub(crate) horizontal_sizing: ScrollSizing,
    pub(crate) vertical_sizing: ScrollSizing,
    pub(crate) direction: ScrollDirection,
}

impl ScrollContainer {
    pub(crate) const fn new(
        horizontal_policy: ScrollAxisPolicy,
        vertical_policy: ScrollAxisPolicy,
        chrome: ScrollChromePresentation,
        horizontal_sizing: ScrollSizing,
        vertical_sizing: ScrollSizing,
        direction: ScrollDirection,
    ) -> Self {
        Self {
            horizontal_policy,
            vertical_policy,
            chrome,
            horizontal_sizing,
            vertical_sizing,
            direction,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Panel {
    pub(super) placement: FloatingPlacement,
    pub(super) attachment: Option<PanelAttachment>,
    pub(crate) popup_context: Option<popup::ContextFingerprint>,
    pub(crate) policy: PanelPolicy,
    pub(crate) force_overlay_group: bool,
    pub(crate) native_material: NativePopupMaterialPreference,
}

impl Content {
    pub(super) fn same_scene_state(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Root, Self::Root)
            | (Self::Stack, Self::Stack)
            | (Self::MenuBar(_), Self::MenuBar(_))
            | (Self::Menu, Self::Menu)
            | (Self::Binding, Self::Binding)
            | (Self::Separator, Self::Separator)
            | (Self::Scroll(Scroll::Ordinary { .. }), Self::Scroll(Scroll::Ordinary { .. }))
            | (Self::Table, Self::Table)
            | (Self::Panel, Self::Panel)
            | (Self::SectionHeader, Self::SectionHeader)
            | (Self::Label, Self::Label) => true,
            (Self::TextArea(left), Self::TextArea(right)) => left.same_scene_state(right),
            (Self::Button(left), Self::Button(right)) => left == right,
            (Self::Checkbox(left), Self::Checkbox(right)) => left == right,
            (Self::Radio(left), Self::Radio(right)) => left == right,
            (Self::Slider(left), Self::Slider(right)) => left == right,
            (Self::TextBox { model: left, .. }, Self::TextBox { model: right, .. }) => {
                left.same_scene_state(right)
            }
            (
                Self::Scroll(Scroll::Table { model: left, .. }),
                Self::Scroll(Scroll::Table { model: right, .. }),
            ) => left.same_scene_state(right),
            (Self::VirtualList { model: left, .. }, Self::VirtualList { model: right, .. }) => {
                left.same_scene_state(right)
            }
            (Self::FloatingPanel(left), Self::FloatingPanel(right)) => left == right,
            _ => false,
        }
    }

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
            Self::Scroll(Scroll::Ordinary { offset, .. } | Scroll::Table { offset, .. })
            | Self::VirtualList { offset, .. } => *offset,
            _ => interaction::ScrollOffset::default(),
        }
    }

    pub(super) fn set_scroll_offset(&mut self, value: interaction::ScrollOffset) {
        match self {
            Self::Scroll(Scroll::Ordinary { offset, .. } | Scroll::Table { offset, .. })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_value_is_not_scene_content_state() {
        let first = Content::Scroll(Scroll::Ordinary {
            offset: interaction::ScrollOffset::new(0, 0),
        });
        let second = Content::Scroll(Scroll::Ordinary {
            offset: interaction::ScrollOffset::new(20, 40),
        });

        assert!(first.same_scene_state(&second));
    }
}
