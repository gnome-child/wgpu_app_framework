use super::super::buffer::{Buffer, Cursor, Position};
use super::State;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Motion {
    VisualLeft,
    VisualRight,
    VisualUp,
    VisualDown,
    PageUp,
    PageDown,
    LogicalPrevious,
    LogicalNext,
    WordPrevious,
    WordNext,
    LineStart,
    LineEnd,
    ParagraphStart,
    ParagraphEnd,
    DocumentStart,
    DocumentEnd,
}

pub(crate) fn text_position_for_motion_in_document_for_state(
    buffer: &Buffer,
    state: State,
    motion: Motion,
) -> Option<Position> {
    let inner = &buffer.inner;
    let index = inner
        .document
        .position_for_mark(state.cursor)
        .unwrap_or_else(|| Position::new(inner.document.text_len()))
        .index;
    let next = match motion {
        Motion::VisualLeft => inner.document.previous_grapheme_boundary_index(index),
        Motion::VisualRight => inner.document.next_grapheme_boundary_index(index),
        Motion::LogicalPrevious => inner.document.previous_grapheme_boundary_index(index),
        Motion::LogicalNext => inner.document.next_grapheme_boundary_index(index),
        Motion::WordPrevious => inner.document.previous_word_boundary_index(index),
        Motion::WordNext => inner.document.next_word_boundary_index(index),
        Motion::LineStart => {
            let (line, _) = inner.document.line_and_local_for_index(index);
            inner.document.line_start(line)
        }
        Motion::LineEnd => {
            let (line, _) = inner.document.line_and_local_for_index(index);
            inner.document.line_start(line) + inner.document.line_text_len(line)
        }
        Motion::ParagraphStart => {
            let (line, _) = inner.document.line_and_local_for_index(index);
            inner.document.line_start(line)
        }
        Motion::ParagraphEnd => {
            let (line, _) = inner.document.line_and_local_for_index(index);
            inner.document.line_start(line) + inner.document.line_text_len(line)
        }
        Motion::DocumentStart => 0,
        Motion::DocumentEnd => inner.document.text_len(),
        _ => return None,
    };

    Some(Position::new(next))
}

pub(crate) fn collapsed_cursor_for_motion(motion: Motion, start: Cursor, end: Cursor) -> Cursor {
    match motion {
        Motion::VisualLeft
        | Motion::LogicalPrevious
        | Motion::WordPrevious
        | Motion::LineStart
        | Motion::ParagraphStart
        | Motion::DocumentStart => start,
        Motion::VisualRight
        | Motion::LogicalNext
        | Motion::WordNext
        | Motion::LineEnd
        | Motion::ParagraphEnd
        | Motion::DocumentEnd => end,
        _ => end,
    }
}
