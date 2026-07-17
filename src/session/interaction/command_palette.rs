use crate::{interaction, window as app_window};

use super::super::{CommandScope, Session, Window};

impl Session {
    pub fn open_command_palette(&mut self, id: app_window::Id) -> bool {
        let captured_focus = self.command_focus(id);
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        let closed_menu = window.interaction.close_menu();
        window.menu_restore_focus = None;
        let opened = window.interaction.open_command_palette(captured_focus);
        let focus = interaction::CommandPalette::query_focus();
        let focus_changed = window.focus != Some(focus);
        if focus_changed {
            window.focus_reveal_pending = focus.is_visible();
        }
        window.focus = Some(focus);

        closed_menu || opened || focus_changed
    }

    pub fn close_command_palette(&mut self, id: app_window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let Some(palette) = window.interaction.command_palette().cloned() else {
            return false;
        };

        let closed = window.interaction.close_command_palette();
        let restore_focus = palette.captured_focus();
        let focus_changed = window.focus != restore_focus;
        if focus_changed {
            window.focus_reveal_pending =
                restore_focus.is_some_and(super::super::Focus::is_visible);
        }
        window.focus = restore_focus;

        closed || focus_changed
    }

    pub fn dismiss_command_palette_for_target(
        &mut self,
        id: app_window::Id,
        target: Option<&interaction::Target>,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        dismiss_command_palette_for_target(window, target)
    }

    pub(crate) fn dismiss_command_palette_for_surface(
        &mut self,
        id: app_window::Id,
        inside_surface: bool,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        dismiss_command_palette_for_surface(window, inside_surface)
    }

    pub(crate) fn command_palette_query(&self, id: app_window::Id) -> Option<String> {
        let interaction = &self.window(id)?.interaction;
        interaction.command_palette()?;
        let target = interaction::CommandPalette::query_target();
        Some(
            interaction
                .text_input()
                .draft_for(&target)
                .map(|draft| draft.text().to_owned())
                .unwrap_or_default(),
        )
    }

    pub(crate) fn command_palette_selected(&self, id: app_window::Id) -> Option<usize> {
        self.window(id)?
            .interaction
            .command_palette()
            .map(interaction::CommandPalette::selected)
    }

    pub(crate) fn command_palette_captured_focus(
        &self,
        id: app_window::Id,
    ) -> Option<Option<super::super::Focus>> {
        Some(
            self.window(id)?
                .interaction
                .command_palette()?
                .captured_focus(),
        )
    }

    pub(crate) fn command_scope(
        &self,
        id: app_window::Id,
        focus: Option<super::super::Focus>,
    ) -> CommandScope {
        let Some(_) = self
            .window(id)
            .and_then(|window| window.interaction.command_palette())
        else {
            return CommandScope::focused(focus);
        };

        if focus.is_some_and(|focus| focus.same_target(&interaction::CommandPalette::query_focus()))
        {
            CommandScope::transient(interaction::CommandPalette::query_focus())
        } else {
            CommandScope::focused(focus)
        }
    }

    pub(crate) fn command_palette_captured_scope(
        &self,
        id: app_window::Id,
    ) -> Option<CommandScope> {
        self.window(id)?
            .interaction
            .command_palette()
            .map(|palette| CommandScope::captured(palette.captured_focus()))
    }

    pub(crate) fn reset_command_palette_selection(&mut self, id: app_window::Id) -> bool {
        self.window_mut(id)
            .is_some_and(|window| window.interaction.reset_command_palette_selection())
    }

    pub(crate) fn select_command_palette_next(&mut self, id: app_window::Id, len: usize) -> bool {
        self.window_mut(id)
            .is_some_and(|window| window.interaction.select_command_palette_next(len))
    }

    pub(crate) fn select_command_palette_previous(
        &mut self,
        id: app_window::Id,
        len: usize,
    ) -> bool {
        self.window_mut(id)
            .is_some_and(|window| window.interaction.select_command_palette_previous(len))
    }

    pub(crate) fn select_command_palette_page_next(
        &mut self,
        id: app_window::Id,
        len: usize,
        page: usize,
    ) -> bool {
        self.window_mut(id).is_some_and(|window| {
            window
                .interaction
                .select_command_palette_page_next(len, page)
        })
    }

    pub(crate) fn select_command_palette_page_previous(
        &mut self,
        id: app_window::Id,
        len: usize,
        page: usize,
    ) -> bool {
        self.window_mut(id).is_some_and(|window| {
            window
                .interaction
                .select_command_palette_page_previous(len, page)
        })
    }
}

pub(super) fn dismiss_command_palette_for_target(
    window: &mut Window,
    target: Option<&interaction::Target>,
) -> bool {
    dismiss_command_palette_for_surface(
        window,
        target.is_some_and(interaction::Target::is_command_palette_surface),
    )
}

fn dismiss_command_palette_for_surface(window: &mut Window, inside_surface: bool) -> bool {
    if window.interaction.command_palette().is_none() {
        return false;
    }
    if inside_surface {
        return false;
    }

    let closed = window.interaction.close_command_palette();
    if closed {
        window.focus = None;
        window.focus_reveal_pending = false;
    }
    closed
}
