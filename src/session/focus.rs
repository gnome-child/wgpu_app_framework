use std::time::Instant;

use super::super::{interaction, window as app_window};
use super::{Session, Window};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Focus {
    kind: Kind,
    reason: Reason,
    visibility: Visibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Kind {
    Text(interaction::Id),
    TableCell(crate::table::Cell),
    Control(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reason {
    Programmatic,
    Keyboard,
    Pointer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Visibility {
    Visible,
    Hidden,
}

impl Focus {
    pub fn text(target: impl Into<interaction::Id>) -> Self {
        Self {
            kind: Kind::Text(target.into()),
            reason: Reason::Programmatic,
            visibility: Visibility::Visible,
        }
    }

    pub fn control(target: &interaction::Target) -> Self {
        Self {
            kind: Kind::Control(target.focus_key()),
            reason: Reason::Programmatic,
            visibility: Visibility::Visible,
        }
    }

    pub fn table_cell(cell: crate::table::Cell) -> Self {
        Self {
            kind: Kind::TableCell(cell),
            reason: Reason::Programmatic,
            visibility: Visibility::Visible,
        }
    }

    pub fn with_reason(mut self, reason: Reason) -> Self {
        self.reason = reason;
        self
    }

    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn keyboard(self) -> Self {
        self.with_reason(Reason::Keyboard)
            .with_visibility(Visibility::Visible)
    }

    pub fn pointer(self) -> Self {
        self.with_reason(Reason::Pointer)
            .with_visibility(Visibility::Hidden)
    }

    pub fn reason(self) -> Reason {
        self.reason
    }

    pub fn visibility(self) -> Visibility {
        self.visibility
    }

    pub fn is_visible(self) -> bool {
        self.visibility == Visibility::Visible
    }

    pub fn shows_focus_indicator(self) -> bool {
        self.is_visible()
    }

    pub fn target(self) -> interaction::Id {
        match self.kind {
            Kind::Text(target) => target,
            Kind::TableCell(_) | Kind::Control(_) => {
                panic!("control focus does not have a text target id")
            }
        }
    }

    pub fn into_target(self) -> interaction::Target {
        self.text_target()
            .expect("control focus does not have a text target")
    }

    pub fn target_id(&self) -> Option<interaction::Id> {
        match self.kind {
            Kind::Text(target) => Some(target),
            Kind::TableCell(_) | Kind::Control(_) => None,
        }
    }

    pub(crate) fn table_cell_identity(self) -> Option<crate::table::Cell> {
        match self.kind {
            Kind::TableCell(cell) => Some(cell),
            Kind::Text(_) | Kind::Control(_) => None,
        }
    }

    pub fn text_target(self) -> Option<interaction::Target> {
        match self.kind {
            Kind::Text(target) => Some(interaction::Target::text_area_id(target)),
            Kind::TableCell(cell) => Some(interaction::Target::table_cell_editor(cell)),
            Kind::Control(_) => None,
        }
    }

    pub(crate) fn from_text_target(target: &interaction::Target) -> Option<Self> {
        if target.kind() == interaction::Kind::TextArea {
            return target
                .table_cell()
                .map(Self::table_cell)
                .or_else(|| target.element_id().map(Self::text));
        }

        None
    }

    pub fn matches_target(self, target: &interaction::Target) -> bool {
        match self.kind {
            Kind::Text(id) => {
                target.kind() == interaction::Kind::TextArea && target.element_id() == Some(id)
            }
            Kind::TableCell(cell) => target.table_cell() == Some(cell),
            Kind::Control(key) => target.focus_key() == key,
        }
    }

    pub fn same_target(self, other: &Self) -> bool {
        match (self.kind, other.kind) {
            (Kind::Text(left), Kind::Text(right)) => left == right,
            (Kind::TableCell(left), Kind::TableCell(right)) => left == right,
            (Kind::Control(left), Kind::Control(right)) => left == right,
            _ => false,
        }
    }
}

impl Session {
    pub fn focus(&mut self, id: app_window::Id, focus: Focus) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = window.focus.as_ref() != Some(&focus);
        let input_changed = if let Some(target) = focus.text_target() {
            let input_changed = window.interaction.clear_text_input_unless(&target);
            let blink_changed = window
                .interaction
                .reset_text_caret_blink(target, Instant::now());
            input_changed || blink_changed
        } else {
            window.interaction.clear_text_preedit()
        };
        window.focus = Some(focus);
        changed || input_changed
    }

    pub fn clear_focus(&mut self, id: app_window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = window.focus.is_some();
        let input_changed = window.interaction.clear_text_preedit();
        window.focus = None;
        changed || input_changed
    }

    pub fn focused(&self, id: app_window::Id) -> Option<Focus> {
        self.window(id).and_then(Window::focus)
    }

    pub(crate) fn command_focus(&self, id: app_window::Id) -> Option<Focus> {
        let window = self.window(id)?;
        if let Some(palette) = window.interaction.command_palette() {
            return palette.captured_focus();
        }

        window.menu_restore_focus.or(window.focus).or_else(|| {
            window
                .interaction
                .text_input()
                .target()
                .and_then(Focus::from_text_target)
        })
    }
}
