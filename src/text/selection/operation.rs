use super::super::{
    buffer::{Buffer, Cursor, CursorSelection, Position},
    unicode::word_range_at,
};
use super::{
    CaretMap, Motion, State, collapsed_cursor_for_motion, selection_mark_from_state,
    text_position_for_motion_in_document_for_state,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    MovePosition(Motion),
    ExtendPosition(Motion),
    SelectAll,
    SetPosition(Position),
    Pointer {
        kind: PointerKind,
        position: Position,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerKind {
    Click,
    DoubleClick,
    TripleClick,
    Drag,
}

struct NoCaretMap;

impl CaretMap for NoCaretMap {
    fn position_for_motion(
        &mut self,
        _buffer: &Buffer,
        _state: State,
        _motion: Motion,
    ) -> Option<Position> {
        None
    }
}

impl Operation {
    pub fn move_position(motion: Motion) -> Self {
        Self::MovePosition(motion)
    }

    pub fn extend_position(motion: Motion) -> Self {
        Self::ExtendPosition(motion)
    }

    #[cfg(test)]
    pub(crate) fn set_cursor(cursor: Cursor) -> Self {
        Self::SetPosition(cursor.into())
    }

    pub fn set_position(position: impl Into<Position>) -> Self {
        Self::SetPosition(position.into())
    }

    pub fn pointer(kind: PointerKind, position: impl Into<Position>) -> Self {
        Self::Pointer {
            kind,
            position: position.into(),
        }
    }
}

pub fn apply(buffer: &Buffer, state: &mut State, operation: Operation) -> bool {
    apply_with_caret_map(buffer, state, operation, &mut NoCaretMap)
}

pub(crate) fn apply_with_caret_map(
    buffer: &Buffer,
    state: &mut State,
    operation: Operation,
    caret_map: &mut dyn CaretMap,
) -> bool {
    let before = *state;
    match operation {
        Operation::MovePosition(motion) => move_position(buffer, state, motion, false, caret_map),
        Operation::ExtendPosition(motion) => move_position(buffer, state, motion, true, caret_map),
        Operation::SelectAll => {
            let end = buffer.len();
            let cursor = buffer.cursor_for_text_index(end);
            let selection = if end == 0 {
                CursorSelection::None
            } else {
                CursorSelection::Normal(buffer.cursor_for_text_index(0))
            };
            buffer.set_cursor_and_selection_for_state(state, cursor, selection);
        }
        Operation::SetPosition(position) => {
            buffer.set_cursor_and_selection_for_state(
                state,
                buffer.cursor_for_position(position),
                CursorSelection::None,
            );
        }
        Operation::Pointer { kind, position } => {
            let cursor = buffer.cursor_for_position(position);
            match kind {
                PointerKind::Click => {
                    buffer.set_cursor_and_selection_for_state(state, cursor, CursorSelection::None)
                }
                PointerKind::DoubleClick => {
                    let text = buffer.text();
                    let range = word_range_at(&text, position.index);
                    buffer.set_cursor_and_selection_for_state(
                        state,
                        buffer.cursor_for_text_index(range.end),
                        CursorSelection::Normal(buffer.cursor_for_text_index(range.start)),
                    );
                }
                PointerKind::TripleClick => {
                    let end = buffer.len();
                    let cursor = buffer.cursor_for_text_index(end);
                    let selection = if end == 0 {
                        CursorSelection::None
                    } else {
                        CursorSelection::Normal(buffer.cursor_for_text_index(0))
                    };
                    buffer.set_cursor_and_selection_for_state(state, cursor, selection);
                }
                PointerKind::Drag => {
                    let anchor = selection_mark_from_state(buffer, *state)
                        .unwrap_or_else(|| buffer.cursor_for_state(*state));
                    buffer.set_cursor_and_selection_for_state(
                        state,
                        cursor,
                        CursorSelection::Normal(anchor),
                    );
                }
            }
        }
    }
    if buffer.selected_range_for_state(*state).is_none() {
        let cursor = buffer.cursor_for_state(*state);
        buffer.set_cursor_and_selection_for_state(state, cursor, CursorSelection::None);
    }
    before != *state
}

fn move_position(
    buffer: &Buffer,
    state: &mut State,
    motion: Motion,
    extend: bool,
    caret_map: &mut dyn CaretMap,
) {
    let anchor = if extend {
        selection_mark_from_state(buffer, *state).unwrap_or_else(|| buffer.cursor_for_state(*state))
    } else {
        buffer.cursor_for_state(*state)
    };
    if !extend && let Some((start, end)) = buffer.selection_bounds_for_state(*state) {
        buffer.set_cursor_and_selection_for_state(
            state,
            collapsed_cursor_for_motion(motion, start, end),
            CursorSelection::None,
        );
        return;
    }
    let next = text_position_for_motion_in_document_for_state(buffer, *state, motion)
        .or_else(|| caret_map.position_for_motion(buffer, *state, motion))
        .unwrap_or_else(|| buffer.position_for_state(*state));
    let cursor = buffer.cursor_for_text_index(next.index);
    let cursor = Cursor::new_with_affinity(cursor.line, cursor.index, next.affinity);
    let selection = if extend {
        CursorSelection::Normal(anchor)
    } else {
        CursorSelection::None
    };
    buffer.set_cursor_and_selection_for_state(state, cursor, selection);
}
