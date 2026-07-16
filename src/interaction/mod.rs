mod command_palette;
mod menu;
pub(crate) mod pointer;
mod scroll;
mod selection;
pub(crate) mod table;
mod target;

pub use crate::identity::Id;
pub(crate) use command_palette::CommandPalette;
pub use menu::Menu;
pub(crate) use pointer::Pointer;
pub(crate) use scroll::{Scroll, ScrollUpdate};
pub use scroll::{ScrollDelta, ScrollOffset};
pub(crate) use selection::Selections;
pub(crate) use table::Tables;
pub(crate) use target::ScrollbarAxis;
pub use target::{Kind, Target};

use crate::text;
use std::time::Instant;

use super::draft;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Interaction {
    surface: Option<Surface>,
    pointer: Pointer,
    scroll: Scroll,
    text_input: draft::Input,
    selections: Selections,
    tables: Tables,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Surface {
    Menu(Menu),
    CommandPalette(CommandPalette),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Pruned {
    outcome: PruneOutcome,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum PruneOutcome {
    #[default]
    Unchanged,
    Changed {
        capture_removed: bool,
        menu_removed: bool,
    },
}

impl Pruned {
    pub(crate) fn changed(self) -> bool {
        matches!(self.outcome, PruneOutcome::Changed { .. })
    }

    pub(crate) fn capture_removed(self) -> bool {
        matches!(
            self.outcome,
            PruneOutcome::Changed {
                capture_removed: true,
                ..
            }
        )
    }

    pub(crate) fn menu_removed(self) -> bool {
        matches!(
            self.outcome,
            PruneOutcome::Changed {
                menu_removed: true,
                ..
            }
        )
    }
}

impl Interaction {
    pub(crate) fn classify_click(
        &mut self,
        target: &Target,
        point: crate::geometry::Point,
        at: Instant,
        settings: crate::pointer::MultiClickSettings,
    ) -> pointer::ClickCount {
        self.pointer.classify_click(target, point, at, settings)
    }

    pub(crate) fn cancel_click_sequence(&mut self) -> bool {
        self.pointer.cancel_click_sequence()
    }

    pub(crate) fn new(draft_limit: usize) -> Self {
        let mut interaction = Self::default();
        interaction.set_text_draft_limit(draft_limit);
        interaction
    }

    pub(crate) fn open_menu(&self) -> Option<&Menu> {
        match &self.surface {
            Some(Surface::Menu(menu)) => Some(menu),
            Some(Surface::CommandPalette(_)) | None => None,
        }
    }

    pub(crate) fn command_palette(&self) -> Option<&CommandPalette> {
        match &self.surface {
            Some(Surface::CommandPalette(palette)) => Some(palette),
            Some(Surface::Menu(_)) | None => None,
        }
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
        let changed = self.open_menu() != Some(&menu);
        self.surface = Some(Surface::Menu(menu));
        changed
    }

    pub(super) fn toggle_menu(&mut self, menu: Menu) -> bool {
        if self.open_menu() == Some(&menu) {
            self.surface = None;
        } else {
            self.surface = Some(Surface::Menu(menu));
        }

        true
    }

    pub(super) fn close_menu(&mut self) -> bool {
        let changed = self.open_menu().is_some();
        if changed {
            self.surface = None;
        }
        changed
    }

    pub(super) fn open_command_palette(
        &mut self,
        captured_focus: Option<super::session::Focus>,
    ) -> bool {
        let palette = CommandPalette::open(captured_focus);
        let changed = self.command_palette() != Some(&palette);
        self.surface = Some(Surface::CommandPalette(palette));
        changed
    }

    pub(super) fn close_command_palette(&mut self) -> bool {
        let changed = self.command_palette().is_some();
        if changed {
            self.surface = None;
        }
        let query = CommandPalette::query_target();
        self.clear_text_draft(&query) || changed
    }

    pub(super) fn reset_command_palette_selection(&mut self) -> bool {
        self.command_palette_mut()
            .is_some_and(CommandPalette::reset_selection)
    }

    pub(super) fn select_command_palette_next(&mut self, len: usize) -> bool {
        self.command_palette_mut()
            .is_some_and(|palette| palette.select_next(len))
    }

    pub(super) fn select_command_palette_previous(&mut self, len: usize) -> bool {
        self.command_palette_mut()
            .is_some_and(|palette| palette.select_previous(len))
    }

    pub(super) fn select_command_palette_page_next(&mut self, len: usize, page: usize) -> bool {
        self.command_palette_mut()
            .is_some_and(|palette| palette.select_page_next(len, page))
    }

    pub(super) fn select_command_palette_page_previous(&mut self, len: usize, page: usize) -> bool {
        self.command_palette_mut()
            .is_some_and(|palette| palette.select_page_previous(len, page))
    }

    fn command_palette_mut(&mut self) -> Option<&mut CommandPalette> {
        match &mut self.surface {
            Some(Surface::CommandPalette(palette)) => Some(palette),
            Some(Surface::Menu(_)) | None => None,
        }
    }

    pub(super) fn pointer_move(&mut self, target: Option<Target>) -> bool {
        let changed = self.pointer.hovered != target;
        if changed {
            self.pointer.hovered = target;
        }
        changed | (changed && self.pointer.dismiss_hover_tip())
    }

    pub(crate) fn set_pointer_location(
        &mut self,
        point: crate::geometry::Point,
        surface: crate::popup::Surface,
    ) -> bool {
        let location = pointer::Location::new(point, surface);
        let changed = self.pointer.location != Some(location);
        self.pointer.location = Some(location);
        changed
    }

    pub(crate) fn set_pointer_modifiers(&mut self, modifiers: crate::keyboard::Modifiers) -> bool {
        let changed = self.pointer.modifiers != modifiers;
        self.pointer.modifiers = modifiers;
        changed
    }

    pub(crate) fn project_pointer_hover(
        &mut self,
        target: Option<Target>,
        tip_eligible: bool,
    ) -> bool {
        self.pointer
            .update_projected_hover(target, tip_eligible, std::time::Instant::now())
    }

    pub(super) fn pointer_down(
        &mut self,
        target: Target,
        intent: pointer::PressIntent,
        cursor: crate::pointer::Cursor,
    ) -> bool {
        let press = pointer::Press::new(target.clone(), intent, cursor);
        let changed = self.pointer.hovered.as_ref() != Some(&target)
            || self.pointer.press.as_ref() != Some(&press);
        self.pointer.hovered = Some(target);
        self.pointer.press = Some(press);
        changed | self.pointer.dismiss_hover_tip()
    }

    pub(super) fn pointer_up(&mut self, target: Option<Target>) -> bool {
        let changed = self.pointer.press.is_some() || self.pointer.hovered != target;
        self.pointer.press = None;
        self.pointer.hovered = target;
        changed | self.pointer.dismiss_hover_tip()
    }

    pub(super) fn set_pointer_press_intent(
        &mut self,
        target: &Target,
        intent: pointer::PressIntent,
    ) -> bool {
        let Some(press) = self
            .pointer
            .press
            .as_mut()
            .filter(|press| press.target() == target)
        else {
            return false;
        };

        let changed = press.intent() != intent;
        press.set_intent(intent);
        changed
    }

    pub(super) fn pointer_left(&mut self) -> bool {
        let uncaptured_press = self
            .pointer
            .press
            .as_ref()
            .is_some_and(|press| press.capture().is_none());
        let changed =
            self.pointer.location.is_some() || self.pointer.hovered.is_some() || uncaptured_press;
        self.pointer.location = None;
        self.pointer.hovered = None;
        if uncaptured_press {
            self.pointer.press = None;
        }
        changed | self.pointer.dismiss_hover_tip()
    }

    pub(super) fn cancel_pointer(&mut self) -> bool {
        let changed = self.pointer.press.take().is_some();
        changed | self.pointer.dismiss_hover_tip()
    }

    pub(crate) fn promote_hover_tip(
        &mut self,
        now: std::time::Instant,
        delay: std::time::Duration,
    ) -> bool {
        self.pointer.promote_hover_tip(now, delay)
    }

    pub(super) fn request_scroll(
        &mut self,
        target: Target,
        update: scroll::ScrollUpdate,
    ) -> Option<ScrollOffset> {
        self.scroll.request(target, update)
    }

    pub(super) fn accept_resident_scroll(
        &mut self,
        target: Target,
        offset: ScrollOffset,
    ) -> Option<ScrollOffset> {
        self.scroll.accept_resident(target, offset)
    }

    pub(crate) fn project_requested_scroll(&mut self) {
        self.scroll.project_desired();
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

    pub(super) fn set_text_preedit(&mut self, target: Target, preedit: text::Preedit) -> bool {
        self.text_input.set_preedit(target, preedit)
    }

    pub(super) fn reset_text_caret_blink(&mut self, target: Target, now: Instant) -> bool {
        self.text_input.reset_caret_blink(target, now)
    }

    pub(super) fn edit_text_draft(
        &mut self,
        target: Target,
        base: impl Into<String>,
        edit: text::Edit,
        input: text::Input,
    ) -> draft::Change {
        self.text_input.edit(target, base, edit, input)
    }

    pub(super) fn select_text_draft(
        &mut self,
        target: Target,
        base: impl Into<String>,
        operation: text::selection::Operation,
    ) -> draft::Change {
        self.text_input.select(target, base, operation)
    }

    pub(super) fn activate_text_draft(&mut self, target: Target, base: impl Into<String>) -> bool {
        self.text_input.activate(target, base)
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

    pub(super) fn report_text_feedback(
        &mut self,
        target: &Target,
        severity: crate::feedback::Severity,
        text: String,
    ) -> bool {
        self.text_input.report_feedback(target, severity, text)
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

    pub(super) fn deactivate_text_input_unless(&mut self, target: &Target) -> bool {
        self.text_input.deactivate_unless(target)
    }

    pub(super) fn set_text_draft_limit(&mut self, limit: usize) {
        self.text_input.set_draft_limit(limit);
    }

    pub(crate) fn prune_removed(
        &mut self,
        removed_nodes: &[super::composition::tree::NodeId],
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
        let press_changed = self
            .pointer
            .press
            .as_ref()
            .is_some_and(|press| removed(press.target()));
        let capture_changed = press_changed
            && self
                .pointer
                .press
                .as_ref()
                .is_some_and(|press| press.capture().is_some());
        if press_changed {
            self.pointer.press = None;
        }

        let scroll_changed = self.scroll.prune_removed(removed_nodes, removed_elements);
        let text_changed =
            self.text_input
                .prune_removed(removed_nodes, removed_elements, removed_table_cells);
        let menu_changed = self
            .open_menu()
            .and_then(Menu::context_owner)
            .is_some_and(|owner| removed_nodes.contains(&owner));
        if menu_changed {
            self.surface = None;
        }

        let changed =
            hovered_changed || press_changed || scroll_changed || text_changed || menu_changed;
        let outcome = if changed {
            PruneOutcome::Changed {
                capture_removed: capture_changed,
                menu_removed: menu_changed,
            }
        } else {
            PruneOutcome::Unchanged
        };
        Pruned { outcome }
    }
}
