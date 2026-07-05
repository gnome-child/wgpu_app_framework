use std::collections::HashMap;

use crate::geometry::{Rect, area, point};
use crate::widget::menu;
use crate::{command, ui, widget};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    surfaces: Vec<ui::floating::Surface>,
    command_contexts: HashMap<ui::Id, command::call::Context>,
    sources: HashMap<ui::Id, command::call::Source>,
    pending_keyboard_focus: bool,
}

impl State {
    pub fn surfaces(&self) -> &[ui::floating::Surface] {
        &self.surfaces
    }

    pub fn command_context(
        &self,
        surface: &ui::floating::Surface,
    ) -> Option<&command::call::Context> {
        self.command_contexts.get(&surface.root_id())
    }

    #[cfg(test)]
    pub fn source(&self, surface: &ui::floating::Surface) -> Option<command::call::Source> {
        self.sources.get(&surface.root_id()).copied()
    }

    pub fn open_menu(&self) -> Option<menu::Id> {
        self.surfaces
            .iter()
            .find_map(|surface| match surface.kind() {
                ui::floating::Kind::Menu(menu) => Some(*menu),
                ui::floating::Kind::Submenu(_) | ui::floating::Kind::ContextMenu { .. } => None,
            })
    }

    pub fn open_submenu(&self) -> Option<menu::Id> {
        self.surfaces
            .iter()
            .rev()
            .find_map(|surface| match surface.kind() {
                ui::floating::Kind::Submenu(menu) => Some(*menu),
                ui::floating::Kind::Menu(_) | ui::floating::Kind::ContextMenu { .. } => None,
            })
    }

    pub fn context_menu(&self) -> Option<&ui::floating::Surface> {
        self.surfaces
            .iter()
            .rev()
            .find(|surface| matches!(surface.kind(), ui::floating::Kind::ContextMenu { .. }))
    }

    pub fn open_top_menu(
        &mut self,
        menu: menu::Id,
        command_context: command::call::Context,
        source: command::call::Source,
        focus_policy: ui::floating::FocusPolicy,
    ) -> bool {
        let changed = self.open_menu() != Some(menu)
            || self.open_submenu().is_some()
            || self.context_menu().is_some();
        let pending_keyboard_focus =
            focus_policy == ui::floating::FocusPolicy::FocusFirstEnabledRow;
        self.clear_surfaces();
        self.push_surface(
            ui::floating::Kind::Menu(menu),
            widget::MENU_POPUP,
            zero_rect_anchor(),
            command_context,
            source,
            focus_policy,
        );
        self.pending_keyboard_focus = pending_keyboard_focus;
        changed
    }

    pub fn show_submenu(
        &mut self,
        menu: menu::Id,
        fallback_command_context: command::call::Context,
        source: command::call::Source,
    ) -> bool {
        if self.open_menu().is_none() {
            return false;
        }

        let changed = self.open_submenu() != Some(menu);
        let command_context = self
            .surfaces
            .iter()
            .rev()
            .find(|surface| {
                matches!(
                    surface.kind(),
                    ui::floating::Kind::Menu(_) | ui::floating::Kind::Submenu(_)
                )
            })
            .and_then(|surface| self.command_context(surface).cloned())
            .unwrap_or(fallback_command_context);
        self.close_kind(|kind| matches!(kind, ui::floating::Kind::Submenu(_)));
        self.push_surface(
            ui::floating::Kind::Submenu(menu),
            widget::MENU_SUBMENU_POPUP,
            zero_rect_anchor(),
            command_context,
            source,
            ui::floating::FocusPolicy::PreserveCurrentFocus,
        );
        self.pending_keyboard_focus = false;
        changed
    }

    pub fn open_context_menu(
        &mut self,
        target: ui::Path,
        anchor: point::Logical,
        command_context: command::call::Context,
        source: command::call::Source,
    ) -> bool {
        let changed = self.context_menu().and_then(|surface| {
            (surface.context_menu_target() == Some(&target)
                && surface.anchor() == ui::floating::Anchor::Point(anchor))
            .then_some(())
        }) != Some(());

        let focus_policy = match source {
            command::call::Source::Keyboard | command::call::Source::Shortcut => {
                ui::floating::FocusPolicy::FocusFirstEnabledRow
            }
            command::call::Source::Pointer | command::call::Source::Programmatic => {
                ui::floating::FocusPolicy::PreserveCurrentFocus
            }
        };
        let pending_keyboard_focus =
            focus_policy == ui::floating::FocusPolicy::FocusFirstEnabledRow;
        self.clear_surfaces();
        self.push_surface(
            ui::floating::Kind::ContextMenu { target },
            widget::TEXT_CONTEXT_MENU_POPUP,
            ui::floating::Anchor::Point(anchor),
            command_context,
            source,
            focus_policy,
        );
        self.pending_keyboard_focus = pending_keyboard_focus;
        changed
    }

    pub fn close_submenu(&mut self) -> bool {
        self.close_kind(|kind| matches!(kind, ui::floating::Kind::Submenu(_)))
    }

    pub fn close_all(&mut self) -> bool {
        let changed = !self.surfaces.is_empty() || self.pending_keyboard_focus;
        self.clear_surfaces();
        self.pending_keyboard_focus = false;
        changed
    }

    pub fn has_open_surface(&self) -> bool {
        !self.surfaces.is_empty()
    }

    #[cfg(test)]
    pub fn needs_keyboard_focus(&self) -> bool {
        self.pending_keyboard_focus
    }

    pub fn take_keyboard_focus_request(&mut self) -> bool {
        let pending = self.pending_keyboard_focus;
        self.pending_keyboard_focus = false;
        pending
    }

    fn close_kind(&mut self, matches: impl Fn(&ui::floating::Kind) -> bool) -> bool {
        let old_len = self.surfaces.len();
        let removed_roots = self
            .surfaces
            .iter()
            .filter(|surface| matches(surface.kind()))
            .map(ui::floating::Surface::root_id)
            .collect::<Vec<_>>();

        self.surfaces.retain(|surface| !matches(surface.kind()));
        for root in removed_roots {
            self.command_contexts.remove(&root);
            self.sources.remove(&root);
        }

        let changed = self.surfaces.len() != old_len;
        if changed {
            self.pending_keyboard_focus = false;
        }
        changed
    }

    fn push_surface(
        &mut self,
        kind: ui::floating::Kind,
        root_id: ui::Id,
        anchor: ui::floating::Anchor,
        command_context: command::call::Context,
        source: command::call::Source,
        focus_policy: ui::floating::FocusPolicy,
    ) {
        self.surfaces.push(ui::floating::Surface::new(
            kind,
            root_id,
            anchor,
            focus_policy,
        ));
        self.command_contexts.insert(root_id, command_context);
        self.sources.insert(root_id, source);
    }

    fn clear_surfaces(&mut self) {
        self.surfaces.clear();
        self.command_contexts.clear();
        self.sources.clear();
    }
}

fn zero_rect_anchor() -> ui::floating::Anchor {
    ui::floating::Anchor::Rect(Rect::new(point::logical(0.0, 0.0), area::logical(0.0, 0.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: menu::Id = menu::Id::new("file");
    const PANELS: menu::Id = menu::Id::new("panels");
    const FIELD: ui::Id = ui::Id::new("field");

    fn context() -> command::call::Context {
        command::call::Context::window(crate::window::Id::new(1))
    }

    #[test]
    fn opening_menu_and_submenu_projects_ids_from_stack() {
        let mut state = State::default();

        assert!(state.open_top_menu(
            FILE,
            context(),
            command::call::Source::Pointer,
            ui::floating::FocusPolicy::PreserveCurrentFocus,
        ));
        assert!(state.show_submenu(PANELS, context(), command::call::Source::Pointer));

        assert_eq!(state.open_menu(), Some(FILE));
        assert_eq!(state.open_submenu(), Some(PANELS));
        assert_eq!(state.surfaces().len(), 2);
    }

    #[test]
    fn opening_context_menu_closes_unrelated_menu_surfaces() {
        let mut state = State::default();

        state.open_top_menu(
            FILE,
            context(),
            command::call::Source::Pointer,
            ui::floating::FocusPolicy::PreserveCurrentFocus,
        );
        state.open_context_menu(
            ui::Path::from(FIELD),
            point::logical(10.0, 20.0),
            context(),
            command::call::Source::Pointer,
        );

        assert_eq!(state.open_menu(), None);
        assert_eq!(state.open_submenu(), None);
        assert!(state.context_menu().is_some());
    }

    #[test]
    fn keyboard_context_menu_records_pending_row_focus() {
        let mut state = State::default();

        state.open_context_menu(
            ui::Path::from(FIELD),
            point::logical(10.0, 20.0),
            context(),
            command::call::Source::Keyboard,
        );

        assert!(state.needs_keyboard_focus());
        assert!(state.take_keyboard_focus_request());
        assert!(!state.needs_keyboard_focus());
    }

    #[test]
    fn keyboard_menu_records_pending_row_focus() {
        let mut state = State::default();

        state.open_top_menu(
            FILE,
            context(),
            command::call::Source::Keyboard,
            ui::floating::FocusPolicy::FocusFirstEnabledRow,
        );

        assert!(state.needs_keyboard_focus());
    }

    #[test]
    fn pointer_menu_preserves_current_focus() {
        let mut state = State::default();

        state.open_top_menu(
            FILE,
            context(),
            command::call::Source::Pointer,
            ui::floating::FocusPolicy::PreserveCurrentFocus,
        );

        assert!(!state.needs_keyboard_focus());
        assert_eq!(
            state.surfaces()[0].focus_policy(),
            ui::floating::FocusPolicy::PreserveCurrentFocus
        );
    }

    #[test]
    fn submenu_inherits_parent_command_context() {
        let mut state = State::default();
        let parent_context =
            command::call::Context::path(crate::window::Id::new(1), ui::Path::from(FIELD));

        state.open_top_menu(
            FILE,
            parent_context.clone(),
            command::call::Source::Pointer,
            ui::floating::FocusPolicy::PreserveCurrentFocus,
        );
        assert!(state.show_submenu(PANELS, context(), command::call::Source::Pointer));

        assert_eq!(
            state.command_context(&state.surfaces()[1]),
            Some(&parent_context)
        );
    }
}
