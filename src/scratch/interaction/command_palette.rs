use super::{Id, Target};
use crate::scratch::session;

const PANEL_ID: &str = "command_palette";
const QUERY_ID: &str = "command_palette.query";
const RESULTS_ID: &str = "command_palette.results";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPalette {
    captured_focus: Option<session::Focus>,
    selected: usize,
}

impl CommandPalette {
    pub(in crate::scratch) fn open(captured_focus: Option<session::Focus>) -> Self {
        Self {
            captured_focus,
            selected: 0,
        }
    }

    pub(in crate::scratch) fn panel_id() -> Id {
        Id::new(PANEL_ID)
    }

    pub(in crate::scratch) fn query_focus() -> session::Focus {
        session::Focus::text(QUERY_ID).keyboard()
    }

    pub(in crate::scratch) fn query_target() -> Target {
        Target::text_area_id(QUERY_ID)
    }

    pub(in crate::scratch) fn results_id() -> Id {
        Id::new(RESULTS_ID)
    }

    pub(in crate::scratch) fn results_target() -> Target {
        Target::scroll(Self::results_id(), "Command Results")
    }

    pub(in crate::scratch) fn captured_focus(&self) -> Option<session::Focus> {
        self.captured_focus
    }

    pub(in crate::scratch) fn selected(&self) -> usize {
        self.selected
    }

    pub(in crate::scratch) fn reset_selection(&mut self) -> bool {
        let changed = self.selected != 0;
        self.selected = 0;
        changed
    }

    pub(in crate::scratch) fn select_next(&mut self, len: usize) -> bool {
        self.select_offset(len, 1)
    }

    pub(in crate::scratch) fn select_previous(&mut self, len: usize) -> bool {
        self.select_offset(len, -1)
    }

    pub(in crate::scratch) fn select_page_next(&mut self, len: usize, page: usize) -> bool {
        self.select_offset(len, page.max(1) as isize)
    }

    pub(in crate::scratch) fn select_page_previous(&mut self, len: usize, page: usize) -> bool {
        self.select_offset(len, -(page.max(1) as isize))
    }

    pub(in crate::scratch) fn select_first(&mut self, len: usize) -> bool {
        self.select_index(len, 0)
    }

    pub(in crate::scratch) fn select_last(&mut self, len: usize) -> bool {
        self.select_index(len, len.saturating_sub(1))
    }

    fn select_offset(&mut self, len: usize, offset: isize) -> bool {
        if len == 0 {
            return self.select_index(0, 0);
        }

        let selected = (self.selected as isize + offset).clamp(0, len.saturating_sub(1) as isize);
        self.select_index(len, selected as usize)
    }

    fn select_index(&mut self, len: usize, index: usize) -> bool {
        let selected = if len == 0 {
            0
        } else {
            index.min(len.saturating_sub(1))
        };
        let changed = self.selected != selected;
        self.selected = selected;
        changed
    }
}
