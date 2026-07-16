use super::super::buffer::{Affinity, Cursor, CursorSelection, Position};
use super::super::document::Style;
use super::super::unicode::floor_grapheme_boundary;
use super::super::{selection::Motion, surface::AreaWrap};
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
    }
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

pub(crate) fn buffer_text_len(buffer: &glyphon::Buffer) -> usize {
    buffer
        .lines
        .iter()
        .map(|line| line.text().len() + line.ending().as_str().len())
        .sum()
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

pub(crate) fn clamp_cursor_in_buffer(buffer: &glyphon::Buffer, cursor: Cursor) -> Cursor {
    let line = cursor.line.min(buffer.lines.len().saturating_sub(1));
    let line_text = buffer
        .lines
        .get(line)
        .map(glyphon::BufferLine::text)
        .unwrap_or("");

    Cursor::new_with_affinity(
        line,
        floor_grapheme_boundary(line_text, cursor.index),
        cursor.affinity,
    )
}

pub(crate) fn cursor_position(buffer: &glyphon::Buffer, cursor: Cursor) -> Option<(i32, i32)> {
    cursor_position_observed(buffer, cursor).0
}

pub(super) fn cursor_position_observed(
    buffer: &glyphon::Buffer,
    cursor: Cursor,
) -> (Option<(i32, i32)>, usize, usize) {
    let mut run_scans = 0_usize;
    let mut glyph_scans = 0_usize;

    // This is the immutable query performed by cosmic-text's Editor cursor
    // projection. Keeping it over borrowed layout runs avoids cloning the
    // complete shaped buffer merely to satisfy Editor's mutable BufferRef API.
    for run in buffer.layout_runs() {
        run_scans += 1;
        if cursor.line != run.line_i {
            continue;
        }

        let mut cursor_glyph = run.glyphs.last().and_then(|glyph| {
            (cursor.index == run.text.len() && cursor.index == glyph.end)
                .then_some((run.glyphs.len(), 0.0))
        });
        if cursor_glyph.is_some() {
            glyph_scans += 1;
        } else {
            for (glyph_index, glyph) in run.glyphs.iter().enumerate() {
                glyph_scans += 1;
                if cursor.index == glyph.start {
                    cursor_glyph = Some((glyph_index, 0.0));
                    break;
                }
                if cursor.index > glyph.start && cursor.index < glyph.end {
                    let cluster = &run.text[glyph.start..glyph.end];
                    let mut before = 0_usize;
                    let mut total = 0_usize;
                    for (index, _) in cluster.grapheme_indices(true) {
                        before += usize::from(glyph.start + index < cursor.index);
                        total += 1;
                    }
                    let offset = if total == 0 {
                        0.0
                    } else {
                        glyph.w * before as f32 / total as f32
                    };
                    cursor_glyph = Some((glyph_index, offset));
                    break;
                }
            }
        }

        if cursor_glyph.is_none() {
            cursor_glyph = match run.glyphs.last() {
                Some(glyph) if cursor.index == glyph.end => Some((run.glyphs.len(), 0.0)),
                None => Some((0, 0.0)),
                _ => None,
            };
        }
        let Some((glyph_index, glyph_offset)) = cursor_glyph else {
            continue;
        };
        let x = run.glyphs.get(glyph_index).map_or_else(
            || {
                run.glyphs.last().map_or(0, |glyph| {
                    if glyph.level.is_rtl() {
                        glyph.x as i32
                    } else {
                        (glyph.x + glyph.w) as i32
                    }
                })
            },
            |glyph| {
                if glyph.level.is_rtl() {
                    (glyph.x + glyph.w - glyph_offset) as i32
                } else {
                    (glyph.x + glyph_offset) as i32
                }
            },
        );
        return (Some((x, run.line_top as i32)), run_scans, glyph_scans);
    }

    (None, run_scans, glyph_scans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed_cursor_geometry_matches_cosmic_editor_for_ltr_rtl_and_clusters() {
        let mut font_system = super::super::system::font_system();
        for text in ["hello world", "שלום עולם", "office e\u{301}lan", ""] {
            let mut buffer = glyphon::Buffer::new_empty(glyphon::Metrics::relative(13.0, 1.25));
            set_cosmic_buffer_text(
                &mut buffer,
                text,
                glyphon::AttrsList::new(&glyphon::Attrs::new()),
                glyphon::Shaping::Advanced,
            );
            buffer.set_wrap(&mut font_system, glyphon::Wrap::None);
            buffer.shape_until_scroll(&mut font_system, false);

            for index in text
                .grapheme_indices(true)
                .map(|(index, _)| index)
                .chain(std::iter::once(text.len()))
            {
                let cursor = Cursor::new(0, index);
                let expected = {
                    let mut clone = buffer.clone();
                    let mut editor = glyphon::Editor::new(&mut clone);
                    glyphon::Edit::set_cursor(&mut editor, glyph_cursor(cursor));
                    glyphon::Edit::cursor_position(&editor)
                };
                assert_eq!(
                    cursor_position(&buffer, cursor),
                    expected,
                    "cursor geometry differs for {text:?} at byte {index}"
                );
            }
        }
    }
}
