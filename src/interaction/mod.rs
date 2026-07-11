mod command_palette;
mod id;
mod menu;
mod pointer;
mod scroll;
mod selection;
mod table;
mod target;

pub(crate) use command_palette::CommandPalette;
pub use id::Id;
pub use menu::Menu;
pub(crate) use pointer::{Capture, Pointer, PressIntent};
pub(crate) use scroll::Scroll;
pub use scroll::{ScrollDelta, ScrollOffset};
pub(crate) use selection::Selections;
pub(crate) use table::Tables;
pub use target::{Kind, Target};

use crate::text;
use std::time::Instant;

use super::draft;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Interaction {
    open_menu: Option<Menu>,
    command_palette: Option<CommandPalette>,
    pointer: Pointer,
    scroll: Scroll,
    text_input: draft::Input,
    selections: Selections,
    tables: Tables,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Pruned {
    changed: bool,
    capture_removed: bool,
}

impl Pruned {
    pub(crate) fn changed(self) -> bool {
        self.changed
    }

    pub(crate) fn capture_removed(self) -> bool {
        self.capture_removed
    }
}

impl Interaction {
    pub(crate) fn new(draft_limit: usize) -> Self {
        let mut interaction = Self::default();
        interaction.set_text_draft_limit(draft_limit);
        interaction
    }

    pub(crate) fn open_menu(&self) -> Option<&Menu> {
        self.open_menu.as_ref()
    }

    pub(crate) fn command_palette(&self) -> Option<&CommandPalette> {
        self.command_palette.as_ref()
    }

    pub(crate) fn pointer(&self) -> &Pointer {
        &self.pointer
    }

    pub(crate) fn scroll(&self) -> &Scroll {
        &self.scroll
    }

    pub(crate) fn text_input(&self) -> &draft::Input {
        &self.text_input
    }

    pub(crate) fn selections(&self) -> &Selections {
        &self.selections
    }

    pub(crate) fn selections_mut(&mut self) -> &mut Selections {
        &mut self.selections
    }

    pub(crate) fn tables(&self) -> &Tables {
        &self.tables
    }

    pub(crate) fn tables_mut(&mut self) -> &mut Tables {
        &mut self.tables
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

    pub(super) fn open_command_palette(
        &mut self,
        captured_focus: Option<super::session::Focus>,
    ) -> bool {
        let palette = CommandPalette::open(captured_focus);
        let changed = self.command_palette.as_ref() != Some(&palette);
        self.command_palette = Some(palette);
        changed
    }

    pub(super) fn close_command_palette(&mut self) -> bool {
        let changed = self.command_palette.is_some();
        self.command_palette = None;
        let query = CommandPalette::query_target();
        self.clear_text_draft(&query) || changed
    }

    pub(super) fn reset_command_palette_selection(&mut self) -> bool {
        self.command_palette
            .as_mut()
            .is_some_and(CommandPalette::reset_selection)
    }

    pub(super) fn select_command_palette_next(&mut self, len: usize) -> bool {
        self.command_palette
            .as_mut()
            .is_some_and(|palette| palette.select_next(len))
    }

    pub(super) fn select_command_palette_previous(&mut self, len: usize) -> bool {
        self.command_palette
            .as_mut()
            .is_some_and(|palette| palette.select_previous(len))
    }

    pub(super) fn select_command_palette_page_next(&mut self, len: usize, page: usize) -> bool {
        self.command_palette
            .as_mut()
            .is_some_and(|palette| palette.select_page_next(len, page))
    }

    pub(super) fn select_command_palette_page_previous(&mut self, len: usize, page: usize) -> bool {
        self.command_palette
            .as_mut()
            .is_some_and(|palette| palette.select_page_previous(len, page))
    }

    pub(super) fn pointer_move(&mut self, target: Option<Target>) -> bool {
        let changed = self.pointer.hovered != target;
        self.pointer.hovered = target;
        changed
    }

    pub(super) fn pointer_down(&mut self, target: Target, intent: PressIntent) -> bool {
        let changed = self.pointer.hovered.as_ref() != Some(&target)
            || self.pointer.pressed.as_ref() != Some(&target)
            || self.pointer.capture.as_ref().map(Capture::target)
                != target.captures().then_some(&target)
            || self.pointer.press_intent != Some(intent);
        self.pointer.hovered = Some(target.clone());
        self.pointer.pressed = Some(target.clone());
        self.pointer.capture = target.captures().then(|| Capture::new(target));
        self.pointer.press_intent = Some(intent);
        changed
    }

    pub(super) fn pointer_up(&mut self, target: Option<Target>) -> bool {
        let changed = self.pointer.pressed.is_some()
            || self.pointer.capture.is_some()
            || self.pointer.press_intent.is_some()
            || self.pointer.hovered != target;
        self.pointer.pressed = None;
        self.pointer.capture = None;
        self.pointer.press_intent = None;
        self.pointer.hovered = target;
        changed
    }

    pub(super) fn set_pointer_press_intent(
        &mut self,
        target: &Target,
        intent: PressIntent,
    ) -> bool {
        if self.pointer.pressed.as_ref() != Some(target) {
            return false;
        }

        let changed = self.pointer.press_intent != Some(intent);
        self.pointer.press_intent = Some(intent);
        changed
    }

    pub(super) fn pointer_left(&mut self) -> bool {
        let changed = self.pointer.hovered.is_some()
            || (self.pointer.capture.is_none() && self.pointer.pressed.is_some());
        self.pointer.hovered = None;
        if self.pointer.capture.is_none() {
            self.pointer.pressed = None;
            self.pointer.press_intent = None;
        }
        changed
    }

    pub(super) fn cancel_pointer(&mut self) -> bool {
        let changed = self.pointer.pressed.is_some() || self.pointer.capture.is_some();
        self.pointer.pressed = None;
        self.pointer.capture = None;
        self.pointer.press_intent = None;
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

    pub(super) fn reveal_active_descendant(&mut self, viewport: Target) -> bool {
        self.scroll.reveal_active_descendant(viewport)
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

    pub(super) fn reset_text_caret_blink(&mut self, target: Target, now: Instant) -> bool {
        self.text_input.reset_caret_blink(target, now)
    }

    pub(super) fn edit_text_draft(
        &mut self,
        target: Target,
        base: impl Into<String>,
        edit: text::edit::Edit,
        input: text::Input,
    ) -> draft::Change {
        self.text_input.edit(target, base, edit, input)
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

    pub(crate) fn prune_removed(
        &mut self,
        removed_nodes: &[super::composition::NodeId],
        removed_elements: &[Id],
        removed_table_cells: &[crate::table::Cell],
    ) -> Pruned {
        let removed = |target: &Target| {
            target.matches_removed_identity(removed_nodes, removed_elements, removed_table_cells)
        };
        let hovered_changed = self.pointer.hovered.as_ref().is_some_and(removed);
        if hovered_changed {
            self.pointer.hovered = None;
        }
        let pressed_changed = self.pointer.pressed.as_ref().is_some_and(removed);
        if pressed_changed {
            self.pointer.pressed = None;
            self.pointer.press_intent = None;
        }
        let capture_changed = self
            .pointer
            .capture
            .as_ref()
            .is_some_and(|capture| removed(capture.target()));
        if capture_changed {
            self.pointer.capture = None;
        }

        let scroll_changed = self.scroll.prune_removed(removed_nodes, removed_elements);
        let text_changed =
            self.text_input
                .prune_removed(removed_nodes, removed_elements, removed_table_cells);
        let tables_changed = self.tables.prune_removed(removed_table_cells);

        Pruned {
            changed: hovered_changed
                || pressed_changed
                || capture_changed
                || scroll_changed
                || text_changed
                || tables_changed,
            capture_removed: capture_changed,
        }
    }
}
