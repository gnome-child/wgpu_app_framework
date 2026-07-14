use super::super::buffer::{
    Affinity, Buffer, Cursor, CursorSelection, Position, normalize_for_buffer,
};
use super::super::unicode::{
    display_index, floor_grapheme_boundary, grapheme_range_in_text, source_grapheme_boundaries,
};
use super::super::{
    selection::State,
    view::{Preedit, ViewState},
};
use super::{Field, Obscuring};

type CursorRange = Option<(Cursor, Cursor)>;
type HighlightRanges = (CursorRange, CursorRange);

pub(crate) struct FieldProjection {
    pub(crate) buffer: Buffer,
    pub(crate) edit_state: State,
    position_map: Option<PositionMap>,
}
pub(crate) struct PreeditProjection {
    pub(crate) buffer: Buffer,
    pub(crate) edit_state: State,
    underline: Option<(Cursor, Cursor)>,
    selection: Option<(Cursor, Cursor)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PositionMap {
    source_len: usize,
    display_len: usize,
    points: Vec<MapPoint>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MapPoint {
    display: usize,
    source: usize,
}

#[derive(Clone, Copy)]
enum Edge {
    Start,
    End,
}

impl PositionMap {
    pub(crate) fn new(
        source_len: usize,
        display_len: usize,
        points: impl IntoIterator<Item = (usize, usize)>,
    ) -> Self {
        let mut points = points
            .into_iter()
            .map(|(display, source)| MapPoint {
                display: display.min(display_len),
                source: source.min(source_len),
            })
            .collect::<Vec<_>>();
        points.sort_by_key(|point| (point.display, point.source));
        points.dedup();
        debug_assert!(points.windows(2).all(|points| {
            points[0].display <= points[1].display && points[0].source <= points[1].source
        }));
        Self {
            source_len,
            display_len,
            points,
        }
    }

    pub(crate) fn source_position(&self, position: Position) -> Position {
        let display = position.index.min(self.display_len);
        let source = self
            .points
            .iter()
            .find(|point| point.display == display)
            .map(|point| point.source)
            .unwrap_or_else(|| {
                let left = self
                    .points
                    .iter()
                    .rev()
                    .find(|point| point.display < display);
                let right = self.points.iter().find(|point| point.display > display);
                if let (Some(left), Some(right)) = (left, right)
                    && right.display.saturating_sub(left.display)
                        == right.source.saturating_sub(left.source)
                {
                    return left
                        .source
                        .saturating_add(display.saturating_sub(left.display));
                }
                match (left, right, position.affinity) {
                    (Some(left), _, Affinity::Upstream) => left.source,
                    (_, Some(right), Affinity::Downstream) => right.source,
                    (Some(left), None, _) => left.source,
                    (None, Some(right), _) => right.source,
                    (None, None, _) => 0,
                }
            });
        Position::with_affinity(source.min(self.source_len), position.affinity)
    }

    pub(crate) fn project_state(&self, source: &Buffer, state: State, display: &Buffer) -> State {
        let source_cursor = source.position_for_state(state);
        let selection = source.selection_for_state(state);
        let (cursor, selection) = if let Some(selection) = selection {
            let forward = selection.anchor.index <= selection.focus.index;
            let anchor = self.display_position(
                selection.anchor,
                if forward { Edge::Start } else { Edge::End },
            );
            let focus = self.display_position(
                selection.focus,
                if forward { Edge::End } else { Edge::Start },
            );
            (
                display.cursor_for_position(focus),
                CursorSelection::Normal(display.cursor_for_position(anchor)),
            )
        } else {
            (
                display.cursor_for_position(self.display_position(source_cursor, Edge::End)),
                CursorSelection::None,
            )
        };
        let mut projected = display.initial_state();
        display.set_cursor_and_selection_for_state(&mut projected, cursor, selection);
        projected
    }

    fn display_position(&self, position: Position, edge: Edge) -> Position {
        let source = position.index.min(self.source_len);
        let display = self
            .points
            .iter()
            .find(|point| point.source == source)
            .map(|point| point.display)
            .unwrap_or_else(|| {
                let left = self.points.iter().rev().find(|point| point.source < source);
                let right = self.points.iter().find(|point| point.source > source);
                if let (Some(left), Some(right)) = (left, right)
                    && right.source.saturating_sub(left.source)
                        == right.display.saturating_sub(left.display)
                {
                    return left
                        .display
                        .saturating_add(source.saturating_sub(left.source));
                }
                match (left, right, edge) {
                    (Some(left), _, Edge::Start) => left.display,
                    (_, Some(right), Edge::End) => right.display,
                    (Some(left), None, _) => left.display,
                    (None, Some(right), _) => right.display,
                    (None, None, _) => 0,
                }
            });
        Position::with_affinity(display.min(self.display_len), position.affinity)
    }
}

impl FieldProjection {
    pub(crate) fn new(field: &Field) -> Self {
        match field.obscuring() {
            Obscuring::None => Self {
                buffer: field.buffer().clone(),
                edit_state: field.state(),
                position_map: None,
            },
            Obscuring::Dot => {
                let source_text = field.buffer().text();
                let source_boundaries = source_grapheme_boundaries(&source_text);
                let buffer = Buffer::from_text(obscured_dot_text(&source_text));
                let position_map = PositionMap::new(
                    source_text.len(),
                    buffer.len(),
                    source_boundaries
                        .iter()
                        .enumerate()
                        .map(|(index, source)| ("•".len() * index, *source)),
                );
                let edit_state = position_map.project_state(field.buffer(), field.state(), &buffer);

                Self {
                    buffer,
                    edit_state,
                    position_map: Some(position_map),
                }
            }
        }
    }

    pub(crate) fn source_position(&self, position: Position) -> Position {
        self.position_map
            .as_ref()
            .map_or(position, |map| map.source_position(position))
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

    pub(crate) fn highlight_ranges(&self) -> HighlightRanges {
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
