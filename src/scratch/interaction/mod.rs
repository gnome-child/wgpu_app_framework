mod id;
mod menu;
mod pointer;
mod scroll;
pub mod target;

pub use id::Id;
pub use menu::Menu;
pub use pointer::{Capture, Pointer};
pub use scroll::{Scroll, ScrollDelta, ScrollOffset};
pub use target::Target;

use crate::text;

use super::draft;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Interaction {
    open_menu: Option<Menu>,
    pointer: Pointer,
    scroll: Scroll,
    text_input: draft::input::Input,
}

impl Interaction {
    pub(in crate::scratch) fn new(draft_limit: usize) -> Self {
        let mut interaction = Self::default();
        interaction.set_text_draft_limit(draft_limit);
        interaction
    }

    pub fn open_menu(&self) -> Option<&Menu> {
        self.open_menu.as_ref()
    }

    pub fn pointer(&self) -> &Pointer {
        &self.pointer
    }

    pub fn scroll(&self) -> &Scroll {
        &self.scroll
    }

    pub fn text_input(&self) -> &draft::input::Input {
        &self.text_input
    }

    pub(super) fn open_menu_with(&mut self, menu: Menu) -> bool {
        let changed = self.open_menu.as_ref() != Some(&menu);
        self.open_menu = Some(menu);
        changed
    }

    pub(super) fn toggle_menu(&mut self, menu: Menu) -> bool {
        if self.open_menu.as_ref() == Some(&menu) {
            self.open_menu = None;
        } else {
            self.open_menu = Some(menu);
        }

        true
    }

    pub(super) fn close_menu(&mut self) -> bool {
        let changed = self.open_menu.is_some();
        self.open_menu = None;
        changed
    }

    pub(super) fn pointer_move(&mut self, target: Option<Target>) -> bool {
        let changed = self.pointer.hovered != target;
        self.pointer.hovered = target;
        changed
    }

    pub(super) fn pointer_down(&mut self, target: Target) -> bool {
        let changed = self.pointer.hovered.as_ref() != Some(&target)
            || self.pointer.pressed.as_ref() != Some(&target)
            || self.pointer.capture.as_ref().map(Capture::target)
                != target.captures().then_some(&target);
        self.pointer.hovered = Some(target.clone());
        self.pointer.pressed = Some(target.clone());
        self.pointer.capture = target.captures().then(|| Capture::new(target));
        changed
    }

    pub(super) fn pointer_up(&mut self, target: Option<Target>) -> bool {
        let changed = self.pointer.pressed.is_some()
            || self.pointer.capture.is_some()
            || self.pointer.hovered != target;
        self.pointer.pressed = None;
        self.pointer.capture = None;
        self.pointer.hovered = target;
        changed
    }

    pub(super) fn pointer_left(&mut self) -> bool {
        let changed = self.pointer.hovered.is_some()
            || (self.pointer.capture.is_none() && self.pointer.pressed.is_some());
        self.pointer.hovered = None;
        if self.pointer.capture.is_none() {
            self.pointer.pressed = None;
        }
        changed
    }

    pub(super) fn cancel_pointer(&mut self) -> bool {
        let changed = self.pointer.pressed.is_some() || self.pointer.capture.is_some();
        self.pointer.pressed = None;
        self.pointer.capture = None;
        changed
    }

    pub(super) fn scroll_by(&mut self, target: Target, delta: ScrollDelta) -> bool {
        self.scroll.scroll_by(target, delta)
    }

    pub(super) fn scroll_to(&mut self, target: Target, offset: ScrollOffset) -> bool {
        self.scroll.scroll_to(target, offset)
    }

    pub(super) fn reveal_scroll(&mut self, target: Target) -> bool {
        self.scroll.reveal(target)
    }

    pub(super) fn clear_scroll_reveal(&mut self, target: &Target) -> bool {
        self.scroll.clear_reveal(target)
    }

    pub(super) fn set_text_preedit(
        &mut self,
        target: Target,
        preedit: text::edit::Preedit,
    ) -> bool {
        self.text_input.set_preedit(target, preedit)
    }

    pub(super) fn edit_text_draft(
        &mut self,
        target: Target,
        base: impl Into<String>,
        edit: text::edit::Edit,
    ) -> draft::Change {
        self.text_input.edit(target, base, edit)
    }

    pub(super) fn undo_text_draft(&mut self, target: &Target) -> Option<draft::Change> {
        self.text_input.undo(target)
    }

    pub(super) fn redo_text_draft(&mut self, target: &Target) -> Option<draft::Change> {
        self.text_input.redo(target)
    }

    pub(super) fn seal_text_draft(&mut self, target: &Target) -> bool {
        self.text_input.seal(target)
    }

    pub(super) fn clear_text_input(&mut self) -> bool {
        self.text_input.clear()
    }

    pub(super) fn clear_text_draft(&mut self, target: &Target) -> bool {
        self.text_input.clear_draft(target)
    }

    pub(super) fn deactivate_text_input(&mut self, target: &Target) -> bool {
        self.text_input.deactivate(target)
    }

    pub(super) fn clear_text_preedit(&mut self) -> bool {
        self.text_input.clear_preedit()
    }

    pub(super) fn clear_text_input_unless(&mut self, target: &Target) -> bool {
        self.text_input.clear_unless(target)
    }

    pub(super) fn set_text_draft_limit(&mut self, limit: usize) {
        self.text_input.set_draft_limit(limit);
    }
}
