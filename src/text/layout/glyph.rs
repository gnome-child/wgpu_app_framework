use super::super::buffer::{Affinity, Cursor, CursorSelection, Position};
use super::super::document::Style;
use super::super::edit::AreaWrap;
use super::super::edit::Motion;
use super::super::unicode::{
    floor_grapheme_boundary, grapheme_range_in_text, next_grapheme_boundary, next_word_boundary,
    normalize_multiline, paragraph_end_boundary, paragraph_start_boundary,
    previous_grapheme_boundary, previous_word_boundary, word_range_at,
};
use unicode_segmentation::UnicodeSegmentation;

pub(super) fn text_area_shaping_for_text(_style: Style, _text: &str) -> glyphon::Shaping {
    glyphon::Shaping::Advanced
}

pub(super) fn glyph_wrap(wrap: AreaWrap) -> glyphon::Wrap {
    match wrap {
        AreaWrap::None => glyphon::Wrap::None,
        AreaWrap::WordOrGlyph => glyphon::Wrap::WordOrGlyph,
    }
}

fn default_text_field_metrics() -> glyphon::Metrics {
    glyphon::Metrics::relative(16.0, 1.25)
}

pub(crate) fn cosmic_buffer_from_text(text: &str) -> glyphon::Buffer {
    let mut buffer = glyphon::Buffer::new_empty(default_text_field_metrics());
    let attrs = glyphon::AttrsList::new(&glyphon::Attrs::new());

    set_cosmic_buffer_text(&mut buffer, text, attrs, glyphon::Shaping::Advanced);

    buffer
}

pub(super) fn set_cosmic_buffer_text(
    buffer: &mut glyphon::Buffer,
    text: &str,
    attrs: glyphon::AttrsList,
    shaping: glyphon::Shaping,
) {
    buffer.lines.clear();

    if text.is_empty() {
        buffer.lines.push(glyphon::BufferLine::new(
            "",
            glyphon::cosmic_text::LineEnding::None,
            attrs,
            shaping,
        ));
        return;
    }

    let mut start = 0;
    for (index, _) in text.match_indices('\n') {
        buffer.lines.push(glyphon::BufferLine::new(
            &text[start..index],
            glyphon::cosmic_text::LineEnding::Lf,
            attrs.clone(),
            shaping,
        ));
        start = index + 1;
    }

    buffer.lines.push(glyphon::BufferLine::new(
        &text[start..],
        glyphon::cosmic_text::LineEnding::None,
        attrs,
        shaping,
    ));
}

fn cosmic_buffer_text(buffer: &glyphon::Buffer) -> String {
    let mut text = String::new();

    for line in &buffer.lines {
        text.push_str(line.text());
        text.push_str(line.ending().as_str());
    }

    normalize_multiline(&text)
}

fn glyph_affinity(affinity: Affinity) -> glyphon::cosmic_text::Affinity {
    match affinity {
        Affinity::Upstream => glyphon::cosmic_text::Affinity::Before,
        Affinity::Downstream => glyphon::cosmic_text::Affinity::After,
    }
}

fn text_affinity(affinity: glyphon::cosmic_text::Affinity) -> Affinity {
    match affinity {
        glyphon::cosmic_text::Affinity::Before => Affinity::Upstream,
        glyphon::cosmic_text::Affinity::After => Affinity::Downstream,
    }
}

pub(super) fn glyph_cursor(cursor: Cursor) -> glyphon::Cursor {
    glyphon::Cursor::new_with_affinity(cursor.line, cursor.index, glyph_affinity(cursor.affinity))
}

pub(super) fn text_cursor(cursor: glyphon::Cursor) -> Cursor {
    Cursor::new_with_affinity(cursor.line, cursor.index, text_affinity(cursor.affinity))
}

pub(super) fn glyph_selection(selection: CursorSelection) -> glyphon::cosmic_text::Selection {
    match selection {
        CursorSelection::None => glyphon::cosmic_text::Selection::None,
        CursorSelection::Normal(cursor) => {
            glyphon::cosmic_text::Selection::Normal(glyph_cursor(cursor))
        }
        CursorSelection::Line(cursor) => {
            glyphon::cosmic_text::Selection::Line(glyph_cursor(cursor))
        }
        CursorSelection::Word(cursor) => {
            glyphon::cosmic_text::Selection::Word(glyph_cursor(cursor))
        }
    }
}

#[allow(dead_code)]
fn cursor_for_text_position_in_buffer(buffer: &glyphon::Buffer, position: Position) -> Cursor {
    let cursor = cursor_for_text_index_in_buffer(buffer, position.index);
    Cursor::new_with_affinity(cursor.line, cursor.index, position.affinity)
}

pub(crate) fn text_position_for_cursor_in_buffer(
    buffer: &glyphon::Buffer,
    cursor: Cursor,
) -> Position {
    let cursor = clamp_cursor_in_buffer(buffer, cursor);
    Position::with_affinity(
        text_index_for_cursor_in_buffer(buffer, cursor),
        cursor.affinity,
    )
}

#[allow(dead_code)]
pub(crate) fn selection_anchor(
    buffer: &glyphon::Buffer,
    selection: CursorSelection,
) -> Option<Cursor> {
    match clamp_selection_in_buffer(buffer, selection) {
        CursorSelection::None => None,
        CursorSelection::Normal(cursor)
        | CursorSelection::Line(cursor)
        | CursorSelection::Word(cursor) => Some(cursor),
    }
}

#[allow(dead_code)]
pub(crate) fn fast_selection_bounds_in_buffer(
    buffer: &glyphon::Buffer,
    cursor: Cursor,
    selection: CursorSelection,
) -> Option<(Cursor, Cursor)> {
    let cursor = clamp_cursor_in_buffer(buffer, cursor);
    match clamp_selection_in_buffer(buffer, selection) {
        CursorSelection::None => None,
        CursorSelection::Normal(select) => Some(ordered_cursors(select, cursor)),
        CursorSelection::Line(select) => {
            let start_line = select.line.min(cursor.line);
            let end_line = select.line.max(cursor.line);
            let end_index = buffer.lines.get(end_line)?.text().len();
            Some((Cursor::new(start_line, 0), Cursor::new(end_line, end_index)))
        }
        CursorSelection::Word(select) => {
            let (mut start, mut end) = ordered_cursors(select, cursor);

            if let Some(line) = buffer.lines.get(start.line) {
                start.index = line
                    .text()
                    .unicode_word_indices()
                    .rev()
                    .map(|(index, _)| index)
                    .find(|index| *index < start.index)
                    .unwrap_or(0);
            }

            if let Some(line) = buffer.lines.get(end.line) {
                end.index = line
                    .text()
                    .unicode_word_indices()
                    .map(|(index, word)| index + word.len())
                    .find(|index| *index > end.index)
                    .unwrap_or_else(|| line.text().len());
            }

            Some((start, end))
        }
    }
}

#[allow(dead_code)]
pub(crate) fn has_non_empty_selection_in_buffer(
    buffer: &glyphon::Buffer,
    cursor: Cursor,
    selection: CursorSelection,
) -> bool {
    fast_selection_bounds_in_buffer(buffer, cursor, selection).is_some_and(|(start, end)| {
        text_index_for_cursor_in_buffer(buffer, start)
            < text_index_for_cursor_in_buffer(buffer, end)
    })
}

#[allow(dead_code)]
fn ordered_cursors(first: Cursor, second: Cursor) -> (Cursor, Cursor) {
    if (first.line, first.index) <= (second.line, second.index) {
        (first, second)
    } else {
        (second, first)
    }
}

pub(super) fn cosmic_motion_for_text_motion(
    motion: Motion,
) -> Option<glyphon::cosmic_text::Motion> {
    Some(match motion {
        Motion::VisualLeft => glyphon::cosmic_text::Motion::Left,
        Motion::VisualRight => glyphon::cosmic_text::Motion::Right,
        Motion::VisualUp => glyphon::cosmic_text::Motion::Up,
        Motion::VisualDown => glyphon::cosmic_text::Motion::Down,
        Motion::PageUp => glyphon::cosmic_text::Motion::PageUp,
        Motion::PageDown => glyphon::cosmic_text::Motion::PageDown,
        Motion::LineStart => glyphon::cosmic_text::Motion::Home,
        Motion::LineEnd => glyphon::cosmic_text::Motion::End,
        _ => return None,
    })
}

#[allow(dead_code)]
pub(crate) fn text_position_for_motion_in_buffer(
    buffer: &glyphon::Buffer,
    cursor: Cursor,
    motion: Motion,
) -> Option<Position> {
    let text = cosmic_buffer_text(buffer);
    let index = text_index_for_cursor_in_buffer(buffer, cursor);
    let next = match motion {
        Motion::LogicalPrevious => previous_grapheme_boundary(&text, index),
        Motion::LogicalNext => next_grapheme_boundary(&text, index),
        Motion::WordPrevious => previous_word_boundary(&text, index),
        Motion::WordNext => next_word_boundary(&text, index),
        Motion::ParagraphStart => paragraph_start_boundary(&text, index),
        Motion::ParagraphEnd => paragraph_end_boundary(&text, index),
        Motion::DocumentStart => 0,
        Motion::DocumentEnd => text.len(),
        _ => return None,
    };

    Some(Position::new(next))
}

#[allow(dead_code)]
pub(crate) fn word_selection_cursors(buffer: &glyphon::Buffer, index: usize) -> (Cursor, Cursor) {
    let text = cosmic_buffer_text(buffer);
    let range = word_range_at(&text, index);
    (
        cursor_for_text_index_in_buffer(buffer, range.start),
        cursor_for_text_index_in_buffer(buffer, range.end),
    )
}

#[allow(dead_code)]
pub(crate) fn normalized_range_in_buffer(
    buffer: &glyphon::Buffer,
    range: std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    let text = cosmic_buffer_text(buffer);
    grapheme_range_in_text(&text, range)
}

#[allow(dead_code)]
pub(crate) fn floor_text_index_in_buffer(buffer: &glyphon::Buffer, index: usize) -> usize {
    let text = cosmic_buffer_text(buffer);
    floor_grapheme_boundary(&text, index)
}

#[allow(dead_code)]
fn line_start_offsets(text: &str) -> Vec<usize> {
    let mut starts = vec![0];

    for (index, character) in text.char_indices() {
        if character == '\n' {
            starts.push(index + 1);
        }
    }

    starts
}

#[cfg(test)]
pub(crate) fn line_start_offsets_for_buffer(buffer: &glyphon::Buffer) -> Vec<usize> {
    let mut starts = Vec::with_capacity(buffer.lines.len().max(1));
    let mut offset = 0;

    for line in &buffer.lines {
        starts.push(offset);
        offset += line.text().len() + line.ending().as_str().len();
    }

    if starts.is_empty() {
        starts.push(0);
    }

    starts
}

#[allow(dead_code)]
pub(crate) fn cursor_for_text_index(text: &str, index: usize) -> Cursor {
    let index = floor_grapheme_boundary(text, index);
    let starts = line_start_offsets(text);
    let line = starts
        .partition_point(|start| *start <= index)
        .saturating_sub(1);
    let line_start = starts.get(line).copied().unwrap_or(0);
    Cursor::new(line, index.saturating_sub(line_start))
}

pub(crate) fn buffer_text_len(buffer: &glyphon::Buffer) -> usize {
    buffer
        .lines
        .iter()
        .map(|line| line.text().len() + line.ending().as_str().len())
        .sum()
}

pub(crate) fn cursor_for_text_index_in_buffer(buffer: &glyphon::Buffer, index: usize) -> Cursor {
    let mut remaining = index.min(buffer_text_len(buffer));

    for (line_index, line) in buffer.lines.iter().enumerate() {
        let text = line.text();
        if remaining <= text.len() {
            return Cursor::new(line_index, floor_grapheme_boundary(text, remaining));
        }

        remaining -= text.len();
        let ending_len = line.ending().as_str().len();
        if remaining < ending_len {
            return Cursor::new(line_index, text.len());
        }
        remaining = remaining.saturating_sub(ending_len);
    }

    let line = buffer.lines.len().saturating_sub(1);
    Cursor::new(
        line,
        buffer
            .lines
            .get(line)
            .map(glyphon::BufferLine::text)
            .map(str::len)
            .unwrap_or(0),
    )
}

pub(crate) fn text_index_for_cursor_in_buffer(buffer: &glyphon::Buffer, cursor: Cursor) -> usize {
    let cursor = clamp_cursor_in_buffer(buffer, cursor);
    let mut index = 0;

    for (line_index, line) in buffer.lines.iter().enumerate() {
        if line_index == cursor.line {
            return index + floor_grapheme_boundary(line.text(), cursor.index);
        }

        index += line.text().len() + line.ending().as_str().len();
    }

    index
}

#[allow(dead_code)]
pub(crate) fn text_range_for_cursors(
    buffer: &glyphon::Buffer,
    start: Cursor,
    end: Cursor,
) -> String {
    let start = clamp_cursor_in_buffer(buffer, start);
    let end = clamp_cursor_in_buffer(buffer, end);

    if start.line == end.line {
        let Some(line) = buffer.lines.get(start.line) else {
            return String::new();
        };

        return line.text()[start.index..end.index].to_owned();
    }

    let mut text = String::new();

    if let Some(line) = buffer.lines.get(start.line) {
        text.push_str(&line.text()[start.index..]);
        text.push_str(line.ending().as_str());
    }

    for line_index in start.line + 1..end.line {
        if let Some(line) = buffer.lines.get(line_index) {
            text.push_str(line.text());
            text.push_str(line.ending().as_str());
        }
    }

    if let Some(line) = buffer.lines.get(end.line) {
        text.push_str(&line.text()[..end.index]);
    }

    text
}

pub(crate) fn clamp_cursor_in_buffer(buffer: &glyphon::Buffer, cursor: Cursor) -> Cursor {
    let line = cursor.line.min(buffer.lines.len().saturating_sub(1));
    let line_text = buffer
        .lines
        .get(line)
        .map(glyphon::BufferLine::text)
        .unwrap_or("");

    Cursor::new(line, floor_grapheme_boundary(line_text, cursor.index))
}

pub(crate) fn clamp_selection_in_buffer(
    buffer: &glyphon::Buffer,
    selection: CursorSelection,
) -> CursorSelection {
    match selection {
        CursorSelection::None => CursorSelection::None,
        CursorSelection::Normal(cursor) => {
            CursorSelection::Normal(clamp_cursor_in_buffer(buffer, cursor))
        }
        CursorSelection::Line(cursor) => {
            CursorSelection::Line(clamp_cursor_in_buffer(buffer, cursor))
        }
        CursorSelection::Word(cursor) => {
            CursorSelection::Word(clamp_cursor_in_buffer(buffer, cursor))
        }
    }
}

pub(crate) fn cursor_position(buffer: &glyphon::Buffer, cursor: Cursor) -> Option<(i32, i32)> {
    let mut buffer = buffer.clone();
    let mut editor = glyphon::Editor::new(&mut buffer);
    glyphon::Edit::set_cursor(&mut editor, glyph_cursor(cursor));

    glyphon::Edit::cursor_position(&editor)
}

#[allow(dead_code)]
fn selection_bounds(
    buffer: &glyphon::Buffer,
    cursor: Cursor,
    selection: CursorSelection,
) -> Option<(Cursor, Cursor)> {
    let mut buffer = buffer.clone();
    let cursor = clamp_cursor_in_buffer(&buffer, cursor);
    let selection = clamp_selection_in_buffer(&buffer, selection);
    let mut editor = glyphon::Editor::new(&mut buffer);
    glyphon::Edit::set_cursor(&mut editor, glyph_cursor(cursor));
    glyphon::Edit::set_selection(&mut editor, glyph_selection(selection));
    let bounds = glyphon::Edit::selection_bounds(&editor);

    drop(editor);

    bounds
        .map(|(start, end)| (text_cursor(start), text_cursor(end)))
        .filter(|(start, end)| {
            text_index_for_cursor_in_buffer(&buffer, *start)
                < text_index_for_cursor_in_buffer(&buffer, *end)
        })
}
