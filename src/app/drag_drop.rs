use std::ops::Range;

use crate::geometry::Rect;
use crate::{text, ui};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    active_text: Option<TextSource>,
    session: Option<Session>,
    text_target: Option<TextTarget>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    source: ui::drag_drop::Source,
    allowed_operations: ui::drag_drop::Operations,
    boundary: ui::drag_drop::Boundary,
    proposed_operation: ui::drag_drop::Operation,
    target: Option<ui::drag_drop::Target>,
    resolved_operation: ui::drag_drop::Operation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSource {
    path: ui::Path,
    selected_range: Range<usize>,
    selected_text: String,
    allowed_operations: ui::drag_drop::Operations,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextTarget {
    path: ui::Path,
    position: text::TextPosition,
    insert_index: usize,
    caret_rect: Rect,
}

impl State {
    pub fn start_text(
        &mut self,
        path: ui::Path,
        selected_range: Range<usize>,
        selected_text: String,
        source_editable: bool,
    ) {
        let source = TextSource {
            path,
            selected_range,
            selected_text,
            allowed_operations: if source_editable {
                ui::drag_drop::Operations::COPY_MOVE
            } else {
                ui::drag_drop::Operations::COPY
            },
        };
        let session = Session::new(
            ui::drag_drop::Source::text(source.path.clone(), source.selected_text.clone()),
            source.allowed_operations,
            ui::drag_drop::Boundary::Internal,
        );

        self.active_text = Some(source);
        self.session = Some(session);
        self.text_target = None;
    }

    pub fn active_text(&self) -> Option<&TextSource> {
        self.active_text.as_ref()
    }

    pub(crate) fn cursor_overlay(&self) -> Option<ui::CursorOverlay> {
        self.active_text
            .as_ref()
            .map(|source| ui::CursorOverlay::text(source.selected_text.clone()))
    }

    #[cfg(test)]
    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    pub fn text_target(&self) -> Option<&TextTarget> {
        self.text_target.as_ref()
    }

    pub fn resolved_operation(&self) -> ui::drag_drop::Operation {
        self.session
            .as_ref()
            .map_or(ui::drag_drop::Operation::None, Session::resolved_operation)
    }

    pub fn operation_for_target(
        &self,
        target_operations: ui::drag_drop::Operations,
        modifiers: ui::Modifiers,
    ) -> ui::drag_drop::Operation {
        self.session
            .as_ref()
            .map_or(ui::drag_drop::Operation::None, |session| {
                session.resolve_operation(target_operations, modifiers)
            })
    }

    pub fn set_text_target(
        &mut self,
        target: Option<TextTarget>,
        target_operations: ui::drag_drop::Operations,
        modifiers: ui::Modifiers,
    ) -> bool {
        let old_target = self.text_target.clone();
        let old_session_target = self
            .session
            .as_ref()
            .and_then(|session| session.target.clone());
        let old_operation = self.resolved_operation();

        match (self.session.as_mut(), target) {
            (Some(session), Some(target)) => {
                let operation =
                    session.set_target(target.path.clone(), target_operations, modifiers);
                self.text_target = (operation != ui::drag_drop::Operation::None).then_some(target);
            }
            (Some(session), None) => {
                session.clear_target(modifiers);
                self.text_target = None;
            }
            (None, _) => {
                self.text_target = None;
            }
        }

        old_target != self.text_target
            || old_operation != self.resolved_operation()
            || old_session_target
                != self
                    .session
                    .as_ref()
                    .and_then(|session| session.target.clone())
    }

    pub fn clear_text_target(&mut self) -> bool {
        self.set_text_target(
            None,
            ui::drag_drop::Operations::NONE,
            ui::Modifiers::default(),
        )
    }

    pub fn complete(&mut self) -> ui::drag_drop::DropResult {
        let operation = self.resolved_operation();
        self.clear();

        if operation == ui::drag_drop::Operation::None {
            ui::drag_drop::DropResult::Rejected
        } else {
            ui::drag_drop::DropResult::Completed { operation }
        }
    }

    pub fn reject(&mut self) -> ui::drag_drop::DropResult {
        self.clear();
        ui::drag_drop::DropResult::Rejected
    }

    pub fn clear(&mut self) -> bool {
        let changed =
            self.active_text.is_some() || self.session.is_some() || self.text_target.is_some();
        self.active_text = None;
        self.session = None;
        self.text_target = None;
        changed
    }
}

impl Session {
    pub fn new(
        source: ui::drag_drop::Source,
        allowed_operations: ui::drag_drop::Operations,
        boundary: ui::drag_drop::Boundary,
    ) -> Self {
        Self {
            source,
            allowed_operations,
            boundary,
            proposed_operation: ui::drag_drop::Operation::None,
            target: None,
            resolved_operation: ui::drag_drop::Operation::None,
        }
    }

    pub fn resolved_operation(&self) -> ui::drag_drop::Operation {
        self.resolved_operation
    }

    fn set_target(
        &mut self,
        path: ui::Path,
        target_operations: ui::drag_drop::Operations,
        modifiers: ui::Modifiers,
    ) -> ui::drag_drop::Operation {
        let operation = self.resolve_operation(target_operations, modifiers);
        self.proposed_operation = proposed_operation(self.boundary, modifiers);
        self.resolved_operation = operation;
        self.target = (operation != ui::drag_drop::Operation::None)
            .then(|| ui::drag_drop::Target::new(path, operation));
        operation
    }

    fn clear_target(&mut self, modifiers: ui::Modifiers) {
        self.proposed_operation = proposed_operation(self.boundary, modifiers);
        self.target = None;
        self.resolved_operation = ui::drag_drop::Operation::None;
    }

    fn resolve_operation(
        &self,
        target_operations: ui::drag_drop::Operations,
        modifiers: ui::Modifiers,
    ) -> ui::drag_drop::Operation {
        resolve_operation(
            self.allowed_operations,
            target_operations,
            self.boundary,
            modifiers,
        )
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

    pub fn can_move(&self) -> bool {
        self.allowed_operations
            .contains(ui::drag_drop::Operation::Move)
    }
}

impl TextTarget {
    pub fn new(
        path: ui::Path,
        position: text::TextPosition,
        insert_index: usize,
        caret_rect: Rect,
    ) -> Self {
        Self {
            path,
            position,
            insert_index,
            caret_rect,
        }
    }

    pub fn path(&self) -> &ui::Path {
        &self.path
    }

    pub fn insert_index(&self) -> usize {
        self.insert_index
    }

    pub fn caret_rect(&self) -> Rect {
        self.caret_rect
    }
}

pub fn resolve_operation(
    source_operations: ui::drag_drop::Operations,
    target_operations: ui::drag_drop::Operations,
    boundary: ui::drag_drop::Boundary,
    modifiers: ui::Modifiers,
) -> ui::drag_drop::Operation {
    let allowed = source_operations.intersection(target_operations);
    if allowed.is_empty() {
        return ui::drag_drop::Operation::None;
    }

    let proposed = proposed_operation(boundary, modifiers);
    if allowed.contains(proposed) {
        return proposed;
    }

    match boundary {
        ui::drag_drop::Boundary::External if allowed.contains(ui::drag_drop::Operation::Copy) => {
            ui::drag_drop::Operation::Copy
        }
        _ if allowed.contains(ui::drag_drop::Operation::Move) => ui::drag_drop::Operation::Move,
        _ if allowed.contains(ui::drag_drop::Operation::Copy) => ui::drag_drop::Operation::Copy,
        _ if allowed.contains(ui::drag_drop::Operation::Link) => ui::drag_drop::Operation::Link,
        _ => ui::drag_drop::Operation::None,
    }
}

fn proposed_operation(
    boundary: ui::drag_drop::Boundary,
    modifiers: ui::Modifiers,
) -> ui::drag_drop::Operation {
    if copy_modifier(modifiers) {
        return ui::drag_drop::Operation::Copy;
    }

    match boundary {
        ui::drag_drop::Boundary::Internal => ui::drag_drop::Operation::Move,
        ui::drag_drop::Boundary::External => ui::drag_drop::Operation::Copy,
    }
}

fn copy_modifier(modifiers: ui::Modifiers) -> bool {
    if cfg!(target_os = "macos") {
        modifiers.alt()
    } else {
        modifiers.control()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn copy_modifier() -> ui::Modifiers {
        ui::Modifiers::new(
            false,
            !cfg!(target_os = "macos"),
            cfg!(target_os = "macos"),
            false,
        )
    }

    #[test]
    fn text_drag_starts_active_session() {
        let path = ui::Path::from(ui::Id::new("field"));
        let mut state = State::default();

        state.start_text(path, 0..3, "abc".to_owned(), true);

        assert!(state.active_text().is_some());
        assert!(state.session().is_some());
    }

    #[test]
    fn active_text_drag_projects_cursor_overlay() {
        let path = ui::Path::from(ui::Id::new("field"));
        let mut state = State::default();

        assert_eq!(state.cursor_overlay(), None);

        state.start_text(path, 0..3, "abc".to_owned(), true);

        let Some(ui::CursorOverlay::Text(overlay)) = state.cursor_overlay() else {
            panic!("active text drag should project a text cursor overlay");
        };
        assert_eq!(overlay.text(), "abc");
    }

    #[test]
    fn editable_internal_text_defaults_to_move() {
        assert_eq!(
            resolve_operation(
                ui::drag_drop::Operations::COPY_MOVE,
                ui::drag_drop::Operations::COPY_MOVE,
                ui::drag_drop::Boundary::Internal,
                ui::Modifiers::default(),
            ),
            ui::drag_drop::Operation::Move
        );
    }

    #[test]
    fn copy_modifier_requests_copy_when_supported() {
        assert_eq!(
            resolve_operation(
                ui::drag_drop::Operations::COPY_MOVE,
                ui::drag_drop::Operations::COPY_MOVE,
                ui::drag_drop::Boundary::Internal,
                copy_modifier(),
            ),
            ui::drag_drop::Operation::Copy
        );
    }

    #[test]
    fn read_only_source_can_only_copy() {
        assert_eq!(
            resolve_operation(
                ui::drag_drop::Operations::COPY,
                ui::drag_drop::Operations::COPY_MOVE,
                ui::drag_drop::Boundary::Internal,
                ui::Modifiers::default(),
            ),
            ui::drag_drop::Operation::Copy
        );
    }

    #[test]
    fn invalid_target_resolves_to_none() {
        assert_eq!(
            resolve_operation(
                ui::drag_drop::Operations::COPY_MOVE,
                ui::drag_drop::Operations::NONE,
                ui::drag_drop::Boundary::Internal,
                ui::Modifiers::default(),
            ),
            ui::drag_drop::Operation::None
        );
    }

    #[test]
    fn external_boundary_defaults_to_copy() {
        assert_eq!(
            resolve_operation(
                ui::drag_drop::Operations::COPY_MOVE,
                ui::drag_drop::Operations::COPY_MOVE,
                ui::drag_drop::Boundary::External,
                ui::Modifiers::default(),
            ),
            ui::drag_drop::Operation::Copy
        );
    }
}
