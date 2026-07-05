use super::super::super::buffer::{
    Buffer, Cursor, CursorSelection, Position, normalize_for_buffer,
};
use super::super::super::unicode::{
    display_index, floor_boundary, floor_grapheme_boundary, grapheme_range_in_text,
    source_grapheme_boundaries,
};
use super::super::{Preedit, State, ViewState};
use super::{Field, Obscuring};

pub(crate) struct FieldProjection {
    pub(crate) buffer: Buffer,
    pub(crate) edit_state: State,
    source_boundaries: Option<Vec<usize>>,
}
pub(crate) struct PreeditProjection {
    pub(crate) buffer: Buffer,
    pub(crate) edit_state: State,
    underline: Option<(Cursor, Cursor)>,
    selection: Option<(Cursor, Cursor)>,
}

impl FieldProjection {
    pub(crate) fn new(field: &Field) -> Self {
        match field.obscuring() {
            Obscuring::None => Self {
                buffer: field.buffer().clone(),
                edit_state: field.state(),
                source_boundaries: None,
            },
            Obscuring::Dot => {
                let source_text = field.buffer().text();
                let source_boundaries = source_grapheme_boundaries(&source_text);
                let buffer = Buffer::from_text(obscured_dot_text(&source_text));
                let mut edit_state = buffer.initial_state();

                if let Some((start, end)) = field.buffer().selection_bounds_for_state(field.state())
                {
                    buffer.set_cursor_and_selection_for_state(
                        &mut edit_state,
                        Self::display_cursor(&source_boundaries, end),
                        CursorSelection::Normal(Self::display_cursor(&source_boundaries, start)),
                    );
                } else {
                    buffer.set_cursor_and_selection_for_state(
                        &mut edit_state,
                        Self::display_cursor(
                            &source_boundaries,
                            field.buffer().cursor_for_state(field.state()),
                        ),
                        CursorSelection::None,
                    );
                }

                Self {
                    buffer,
                    edit_state,
                    source_boundaries: Some(source_boundaries),
                }
            }
        }
    }

    pub(crate) fn source_position(&self, position: Position) -> Position {
        let Some(source_boundaries) = self.source_boundaries.as_ref() else {
            return position;
        };

        Position::with_affinity(
            self.source_index(source_boundaries, position.index),
            position.affinity,
        )
    }

    fn display_cursor(source_boundaries: &[usize], cursor: Cursor) -> Cursor {
        Cursor::new(0, display_index(source_boundaries, cursor.index))
    }

    fn source_index(&self, source_boundaries: &[usize], display_index: usize) -> usize {
        let text = self.buffer.text();
        let display_index = floor_boundary(&text, display_index);
        let character = text[..display_index].chars().count();

        source_boundaries
            .get(character.min(source_boundaries.len().saturating_sub(1)))
            .copied()
            .unwrap_or(0)
    }
}

impl PreeditProjection {
    pub(crate) fn new(buffer: &Buffer, edit_state: State, state: &ViewState) -> Self {
        let Some(preedit) = state.preedit() else {
            return Self::committed(buffer, edit_state);
        };

        let source = buffer.text();
        let range = preedit_replacement_range(buffer, edit_state, &source);
        let preedit_text = normalize_for_buffer(buffer, preedit.text());
        let preedit_start = range.start;
        let preedit_end = preedit_start + preedit_text.len();
        let mut text =
            String::with_capacity(source.len() - (range.end - range.start) + preedit_text.len());
        text.push_str(&source[..range.start]);
        text.push_str(&preedit_text);
        text.push_str(&source[range.end..]);

        let buffer = Buffer::from_text_with_mode(text, buffer.is_multiline());
        let mut edit_state = buffer.initial_state();
        let selection_range = preedit
            .selection()
            .map(|(start, end)| preedit_selection_range(&preedit_text, start, end));
        let cursor_index = selection_range
            .as_ref()
            .map(|range| preedit_start + range.end)
            .unwrap_or(preedit_end);
        let cursor = buffer.cursor_for_text_index(cursor_index);
        buffer.set_cursor_and_selection_for_state(&mut edit_state, cursor, CursorSelection::None);

        let underline = (preedit_start < preedit_end).then(|| {
            (
                buffer.cursor_for_text_index(preedit_start),
                buffer.cursor_for_text_index(preedit_end),
            )
        });
        let selection = selection_range.and_then(|range| {
            (range.start < range.end).then(|| {
                (
                    buffer.cursor_for_text_index(preedit_start + range.start),
                    buffer.cursor_for_text_index(preedit_start + range.end),
                )
            })
        });

        Self {
            buffer,
            edit_state,
            underline,
            selection,
        }
    }

    fn committed(buffer: &Buffer, edit_state: State) -> Self {
        Self {
            buffer: buffer.clone(),
            edit_state,
            underline: None,
            selection: None,
        }
    }

    pub(crate) fn has_preedit(&self) -> bool {
        self.underline.is_some()
    }

    pub(crate) fn highlight_ranges(&self) -> (Option<(Cursor, Cursor)>, Option<(Cursor, Cursor)>) {
        (self.underline, self.selection)
    }

    pub(crate) fn cursor(&self) -> Cursor {
        self.buffer.cursor_for_state(self.edit_state)
    }

    pub(crate) fn selection_bounds(&self) -> Option<(Cursor, Cursor)> {
        self.buffer.selection_bounds_for_state(self.edit_state)
    }

    pub(crate) fn has_non_empty_selection(&self) -> bool {
        self.buffer
            .has_non_empty_selection_for_state(self.edit_state)
    }
}

pub(super) fn preedit_replacement_range(
    buffer: &Buffer,
    edit_state: State,
    source: &str,
) -> std::ops::Range<usize> {
    if let Some(range) = buffer.selected_range_for_state(edit_state) {
        return grapheme_range_in_text(source, range.as_range());
    }

    let index = floor_grapheme_boundary(
        source,
        buffer.text_index_for_cursor(buffer.cursor_for_state(edit_state)),
    );
    index..index
}

fn preedit_selection_range(text: &str, start: usize, end: usize) -> std::ops::Range<usize> {
    if start == end {
        let index = floor_grapheme_boundary(text, start);
        return index..index;
    }

    grapheme_range_in_text(text, start.min(end)..start.max(end))
}

pub(crate) fn projected_state_for_field(field: &Field, state: ViewState) -> ViewState {
    if field.obscuring() != Obscuring::Dot {
        return state;
    }

    let Some(preedit) = state.preedit().cloned() else {
        return state;
    };

    state.with_preedit(Some(obscured_preedit(&preedit)))
}

fn obscured_preedit(preedit: &Preedit) -> Preedit {
    let boundaries = source_grapheme_boundaries(preedit.text());
    let text = obscured_dot_text(preedit.text());
    let selection = preedit.selection().map(|(start, end)| {
        (
            display_index(&boundaries, start),
            display_index(&boundaries, end),
        )
    });

    Preedit::new(text, selection)
}

pub(super) fn composed_presentation_text(
    source: &str,
    replace_range: std::ops::Range<usize>,
    preedit_text: &str,
) -> String {
    let mut text = String::with_capacity(
        source.len() - (replace_range.end - replace_range.start) + preedit_text.len(),
    );
    text.push_str(&source[..replace_range.start]);
    text.push_str(preedit_text);
    text.push_str(&source[replace_range.end..]);
    text
}

pub(super) fn obscured_dot_text(text: &str) -> String {
    "•".repeat(source_grapheme_boundaries(text).len().saturating_sub(1))
}
