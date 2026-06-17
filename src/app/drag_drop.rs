use std::ops::Range;

use crate::geometry::{Rect, point};
use crate::{text, ui};

const TEXT_DRAG_THRESHOLD: f32 = 4.0;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    pending: Option<TextSource>,
    active: Option<TextSource>,
    target: Option<TextTarget>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSource {
    path: ui::Path,
    start_position: point::Logical,
    selected_range: Range<usize>,
    selected_text: String,
    source_editable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextTarget {
    path: ui::Path,
    cursor: text::Cursor,
    operation: ui::drag_drop::Operation,
    caret_rect: Rect,
}

impl State {
    pub fn begin_text(
        &mut self,
        path: ui::Path,
        start_position: point::Logical,
        selected_range: Range<usize>,
        selected_text: String,
        source_editable: bool,
    ) {
        self.pending = Some(TextSource {
            path,
            start_position,
            selected_range,
            selected_text,
            source_editable,
        });
        self.active = None;
        self.target = None;
    }

    #[cfg(test)]
    pub fn pending_text(&self) -> Option<&TextSource> {
        self.pending.as_ref()
    }

    pub fn active_text(&self) -> Option<&TextSource> {
        self.active.as_ref()
    }

    pub fn text_target(&self) -> Option<&TextTarget> {
        self.target.as_ref()
    }

    pub fn try_start_text_drag(&mut self, position: point::Logical) -> bool {
        if self.active.is_some() {
            return true;
        }

        let Some(pending) = self.pending.as_ref() else {
            return false;
        };

        if point_distance(pending.start_position, position) < TEXT_DRAG_THRESHOLD {
            return false;
        }

        self.active = self.pending.take();
        true
    }

    pub fn set_text_target(&mut self, target: Option<TextTarget>) -> bool {
        if self.target == target {
            return false;
        }

        self.target = target;
        true
    }

    pub fn clear_text_target(&mut self) -> bool {
        self.set_text_target(None)
    }

    pub fn clear(&mut self) -> bool {
        let changed = self.pending.is_some() || self.active.is_some() || self.target.is_some();
        self.pending = None;
        self.active = None;
        self.target = None;
        changed
    }
}

impl TextSource {
    pub fn path(&self) -> &ui::Path {
        &self.path
    }

    pub fn selected_range(&self) -> Range<usize> {
        self.selected_range.clone()
    }

    pub fn selected_text(&self) -> &str {
        &self.selected_text
    }

    pub fn source_editable(&self) -> bool {
        self.source_editable
    }
}

impl TextTarget {
    pub fn new(
        path: ui::Path,
        cursor: text::Cursor,
        operation: ui::drag_drop::Operation,
        caret_rect: Rect,
    ) -> Self {
        Self {
            path,
            cursor,
            operation,
            caret_rect,
        }
    }

    pub fn path(&self) -> &ui::Path {
        &self.path
    }

    pub fn cursor(&self) -> text::Cursor {
        self.cursor
    }

    pub fn operation(&self) -> ui::drag_drop::Operation {
        self.operation
    }

    pub fn caret_rect(&self) -> Rect {
        self.caret_rect
    }
}

pub fn text_operation(
    source: &TextSource,
    target_path: &ui::Path,
    target_editable: bool,
    modifiers: ui::Modifiers,
) -> ui::drag_drop::Operation {
    if !target_editable {
        return ui::drag_drop::Operation::None;
    }

    if text_copy_modifier(modifiers) || !source.source_editable() {
        return ui::drag_drop::Operation::Copy;
    }

    if target_path == source.path() {
        ui::drag_drop::Operation::Move
    } else {
        ui::drag_drop::Operation::Copy
    }
}

fn text_copy_modifier(modifiers: ui::Modifiers) -> bool {
    if cfg!(target_os = "macos") {
        modifiers.alt()
    } else {
        modifiers.control()
    }
}

fn point_distance(a: point::Logical, b: point::Logical) -> f32 {
    let dx = a.x() - b.x();
    let dy = a.y() - b.y();

    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_drag_starts_after_threshold() {
        let path = ui::Path::from(ui::Id::new("field"));
        let mut state = State::default();

        state.begin_text(path, point::logical(0.0, 0.0), 0..3, "abc".to_owned(), true);

        assert!(!state.try_start_text_drag(point::logical(2.0, 0.0)));
        assert!(state.pending_text().is_some());
        assert!(state.active_text().is_none());

        assert!(state.try_start_text_drag(point::logical(5.0, 0.0)));
        assert!(state.pending_text().is_none());
        assert!(state.active_text().is_some());
    }

    #[test]
    fn copy_modifier_changes_text_operation() {
        let path = ui::Path::from(ui::Id::new("field"));
        let other = ui::Path::from(ui::Id::new("other"));
        let mut state = State::default();
        state.begin_text(
            path.clone(),
            point::logical(0.0, 0.0),
            0..3,
            "abc".to_owned(),
            true,
        );
        state.try_start_text_drag(point::logical(5.0, 0.0));
        let source = state.active_text().expect("source");

        assert_eq!(
            text_operation(
                source,
                &path,
                true,
                ui::Modifiers::new(false, false, false, false)
            ),
            ui::drag_drop::Operation::Move
        );
        assert_eq!(
            text_operation(
                source,
                &path,
                true,
                ui::Modifiers::new(
                    false,
                    !cfg!(target_os = "macos"),
                    cfg!(target_os = "macos"),
                    false
                )
            ),
            ui::drag_drop::Operation::Copy
        );
        assert_eq!(
            text_operation(
                source,
                &other,
                true,
                ui::Modifiers::new(false, false, false, false)
            ),
            ui::drag_drop::Operation::Copy
        );
        assert_eq!(
            text_operation(
                source,
                &other,
                false,
                ui::Modifiers::new(false, false, false, false)
            ),
            ui::drag_drop::Operation::None
        );
    }
}
