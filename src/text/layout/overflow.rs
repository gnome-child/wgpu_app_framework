use unicode_segmentation::UnicodeSegmentation;

use super::{Engine, Measure};
use crate::text::{self, Overflow, buffer::Position, edit::PositionMap};

const ELLIPSIS: &str = "…";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OverflowProjection {
    source: String,
    visible: String,
    position_map: Option<PositionMap>,
}

impl Engine {
    /// Resolves overflow in logical source order. Bidi shaping happens after
    /// resolution, so `End` means the end of the source string in v1.
    pub(crate) fn resolve_overflow(
        &mut self,
        value: &str,
        style: text::document::Style,
        width: f32,
        overflow: Overflow,
    ) -> String {
        self.resolve_overflow_projection(value, style, width, overflow)
            .visible
    }

    pub(crate) fn resolve_overflow_projection(
        &mut self,
        value: &str,
        style: text::document::Style,
        width: f32,
        overflow: Overflow,
    ) -> OverflowProjection {
        let width = finite_width(width);
        if overflow == Overflow::Clip || self.fits(value, style, width) {
            return OverflowProjection::identity(value);
        }
        if !self.fits(ELLIPSIS, style, width) {
            return OverflowProjection::empty(value);
        }

        let graphemes = UnicodeSegmentation::graphemes(value, true).collect::<Vec<_>>();
        match overflow {
            Overflow::Clip => OverflowProjection::identity(value),
            Overflow::EllipsisEnd => {
                let keep = self.ellipsis_end(&graphemes, style, width);
                OverflowProjection::ellipsis_end(value, &graphemes, keep)
            }
            Overflow::EllipsisMiddle => {
                let (head, tail) = self.ellipsis_middle(&graphemes, style, width);
                OverflowProjection::ellipsis_middle(value, &graphemes, head, tail)
            }
        }
    }

    pub(crate) fn resolve_single_line_overflow_projection(
        &mut self,
        value: &str,
        style: text::document::Style,
        width: f32,
        overflow: Overflow,
    ) -> OverflowProjection {
        let Some(line_end) = value.find(|character| matches!(character, '\r' | '\n')) else {
            return self.resolve_overflow_projection(value, style, width, overflow);
        };
        let first_line = &value[..line_end];
        if overflow == Overflow::Clip {
            return OverflowProjection {
                source: value.to_owned(),
                visible: first_line.to_owned(),
                position_map: Some(PositionMap::new(
                    value.len(),
                    first_line.len(),
                    [(0, 0), (line_end, line_end)],
                )),
            };
        }
        let width = finite_width(width);
        if !self.fits(ELLIPSIS, style, width) {
            return OverflowProjection::empty(value);
        }
        let graphemes = UnicodeSegmentation::graphemes(first_line, true).collect::<Vec<_>>();
        let keep = self.ellipsis_end(&graphemes, style, width);
        OverflowProjection::ellipsis_end(value, &graphemes, keep)
    }

    fn ellipsis_end(
        &mut self,
        graphemes: &[&str],
        style: text::document::Style,
        width: f32,
    ) -> usize {
        let mut low = 0;
        let mut high = graphemes.len();
        while low < high {
            let mid = low + (high - low).div_ceil(2);
            let candidate = end_candidate(graphemes, mid);
            if self.fits(&candidate, style, width) {
                low = mid;
            } else {
                high = mid - 1;
            }
        }
        low
    }

    fn ellipsis_middle(
        &mut self,
        graphemes: &[&str],
        style: text::document::Style,
        width: f32,
    ) -> (usize, usize) {
        if graphemes.is_empty() {
            return (0, 0);
        }

        let mut head = 0;
        let mut tail = 0;
        if graphemes.len() >= 2 {
            let both = middle_candidate(graphemes, 1, 1);
            if self.fits(&both, style, width) {
                head = 1;
                tail = 1;
            }
        }

        if head == 0 && tail == 0 {
            let with_head = middle_candidate(graphemes, 1, 0);
            let with_tail = middle_candidate(graphemes, 0, 1);
            if self.fits(&with_head, style, width) {
                head = 1;
            } else if self.fits(&with_tail, style, width) {
                tail = 1;
            }
        }

        let mut prefer_head = head <= tail;
        loop {
            let mut advanced = false;
            for add_head in [prefer_head, !prefer_head] {
                let (next_head, next_tail) = if add_head {
                    (head + 1, tail)
                } else {
                    (head, tail + 1)
                };
                if next_head + next_tail >= graphemes.len() {
                    continue;
                }
                let candidate = middle_candidate(graphemes, next_head, next_tail);
                if self.fits(&candidate, style, width) {
                    head = next_head;
                    tail = next_tail;
                    prefer_head = !add_head;
                    advanced = true;
                    break;
                }
            }
            if !advanced {
                break;
            }
        }

        (head, tail)
    }

    fn fits(&mut self, value: &str, style: text::document::Style, width: f32) -> bool {
        single_line_width(self, value, style) <= width
    }
}

impl OverflowProjection {
    fn identity(source: &str) -> Self {
        Self {
            source: source.to_owned(),
            visible: source.to_owned(),
            position_map: None,
        }
    }

    fn empty(source: &str) -> Self {
        Self {
            source: source.to_owned(),
            visible: String::new(),
            position_map: Some(PositionMap::new(source.len(), 0, [(0, 0)])),
        }
    }

    fn ellipsis_end(source: &str, graphemes: &[&str], keep: usize) -> Self {
        let head_end = graphemes[..keep].iter().map(|value| value.len()).sum();
        let visible = end_candidate(graphemes, keep);
        Self {
            source: source.to_owned(),
            position_map: Some(PositionMap::new(
                source.len(),
                visible.len(),
                [(0, 0), (head_end, head_end), (visible.len(), source.len())],
            )),
            visible,
        }
    }

    fn ellipsis_middle(source: &str, graphemes: &[&str], head: usize, tail: usize) -> Self {
        let head_end = graphemes[..head].iter().map(|value| value.len()).sum();
        let tail_bytes = graphemes[graphemes.len().saturating_sub(tail)..]
            .iter()
            .map(|value| value.len())
            .sum::<usize>();
        let tail_start = source.len().saturating_sub(tail_bytes);
        let visible = middle_candidate(graphemes, head, tail);
        let tail_visible_start = head_end + ELLIPSIS.len();
        Self {
            source: source.to_owned(),
            position_map: Some(PositionMap::new(
                source.len(),
                visible.len(),
                [
                    (0, 0),
                    (head_end, head_end),
                    (tail_visible_start, tail_start),
                    (visible.len(), source.len()),
                ],
            )),
            visible,
        }
    }

    pub(crate) fn visible(&self) -> &str {
        &self.visible
    }

    pub(crate) fn source_position(&self, position: Position) -> Position {
        self.position_map.as_ref().map_or_else(
            || Position::with_affinity(position.index.min(self.source.len()), position.affinity),
            |map| map.source_position(position),
        )
    }

    pub(crate) fn project_buffer_state(
        &self,
        source: &text::Buffer,
        state: text::edit::State,
    ) -> (text::Buffer, text::edit::State) {
        debug_assert_eq!(source.text(), self.source);
        let Some(position_map) = self.position_map.as_ref() else {
            return (source.clone(), state);
        };
        let buffer = text::Buffer::from_multiline_text(self.visible.clone());
        let projected = position_map.project_state(source, state, &buffer);
        (buffer, projected)
    }
}

fn finite_width(width: f32) -> f32 {
    if width.is_finite() {
        width.max(0.0)
    } else {
        0.0
    }
}

fn single_line_width(engine: &mut Engine, value: &str, style: text::document::Style) -> f32 {
    let mut block = text::document::Block::new(text::document::Align::Start);
    block.push_run(text::document::Run::new(value, style));
    engine
        .measure(
            &text::document::Document::from_block(block),
            Measure::unbounded(),
        )
        .width()
}

fn end_candidate(graphemes: &[&str], keep: usize) -> String {
    let mut candidate = graphemes[..keep].concat();
    candidate.push_str(ELLIPSIS);
    candidate
}

fn middle_candidate(graphemes: &[&str], head: usize, tail: usize) -> String {
    let mut candidate = graphemes[..head].concat();
    candidate.push_str(ELLIPSIS);
    if tail > 0 {
        candidate.push_str(&graphemes[graphemes.len() - tail..].concat());
    }
    candidate
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::buffer::CursorSelection;

    fn style() -> text::document::Style {
        text::document::Style::default().with_size(16.0)
    }

    #[test]
    fn end_ellipsis_preserves_graphemes() {
        let mut engine = Engine::new();
        let value = "A👩‍🚀BC";
        let width = single_line_width(&mut engine, "A👩‍🚀…", style()) + 0.1;
        let resolved = engine.resolve_overflow(value, style(), width, Overflow::EllipsisEnd);

        assert_eq!(resolved, "A👩‍🚀…");
        assert!(resolved.is_char_boundary(resolved.len()));
    }

    #[test]
    fn middle_ellipsis_preserves_head_and_tail() {
        let mut engine = Engine::new();
        let value = "alpha/path/report.csv";
        let width = single_line_width(&mut engine, "a…v", style()) + 0.1;
        let resolved = engine.resolve_overflow(value, style(), width, Overflow::EllipsisMiddle);

        assert!(resolved.starts_with('a'));
        assert!(resolved.ends_with('v'));
        assert_eq!(resolved.matches(ELLIPSIS).count(), 1);
    }

    #[test]
    fn middle_ellipsis_never_splits_a_grapheme_cluster() {
        let mut engine = Engine::new();
        let value = "A👩‍🚀e\u{301}Z";
        let width = single_line_width(&mut engine, "A…Z", style()) + 0.1;
        let resolved = engine.resolve_overflow(value, style(), width, Overflow::EllipsisMiddle);
        let source_graphemes = UnicodeSegmentation::graphemes(value, true).collect::<Vec<_>>();

        for grapheme in UnicodeSegmentation::graphemes(resolved.as_str(), true) {
            assert!(grapheme == ELLIPSIS || source_graphemes.contains(&grapheme));
        }
    }

    #[test]
    fn tiny_width_has_a_deterministic_empty_result() {
        let mut engine = Engine::new();
        assert_eq!(
            engine.resolve_overflow("world", style(), 0.0, Overflow::EllipsisEnd),
            ""
        );
        assert_eq!(
            engine.resolve_overflow("world", style(), f32::NAN, Overflow::EllipsisMiddle),
            ""
        );
    }

    #[test]
    fn clip_preserves_source_even_when_it_does_not_fit() {
        let mut engine = Engine::new();
        assert_eq!(
            engine.resolve_overflow("world", style(), 0.0, Overflow::Clip),
            "world"
        );
    }

    #[test]
    fn bidi_policy_operates_on_logical_source_order() {
        let mut engine = Engine::new();
        let value = "אבגדה";
        let width = single_line_width(&mut engine, "אב…", style()) + 0.1;
        let resolved = engine.resolve_overflow(value, style(), width, Overflow::EllipsisEnd);

        assert_eq!(resolved, "אב…");
    }

    #[test]
    fn end_ellipsis_projection_maps_visible_selection_back_to_the_full_source() {
        let mut engine = Engine::new();
        let value = "A👩‍🚀BC";
        let width = single_line_width(&mut engine, "A👩‍🚀…", style()) + 0.1;
        let projection =
            engine.resolve_overflow_projection(value, style(), width, Overflow::EllipsisEnd);
        let source = text::Buffer::from_text(value);
        let mut source_state = source.initial_state();
        source.set_cursor_and_selection_for_state(
            &mut source_state,
            source.cursor_for_text_index(value.len()),
            CursorSelection::Normal(source.cursor_for_text_index(0)),
        );
        let (visible, visible_state) = projection.project_buffer_state(&source, source_state);

        assert_eq!(visible.text(), "A👩‍🚀…");
        assert_eq!(
            visible.selected_text_for_state(visible_state).as_deref(),
            Some("A👩‍🚀…")
        );
        assert_eq!(
            projection
                .source_position(Position::new(projection.visible().len()))
                .index,
            value.len()
        );
        assert_eq!(
            source.selected_text_for_state(source_state).as_deref(),
            Some(value),
            "clipboard truth remains the selected source range"
        );
    }

    #[test]
    fn middle_ellipsis_projection_maps_the_visible_tail_to_its_source_tail() {
        let mut engine = Engine::new();
        let value = "alpha/path/report.csv";
        let width = single_line_width(&mut engine, "a…v", style()) + 0.1;
        let projection =
            engine.resolve_overflow_projection(value, style(), width, Overflow::EllipsisMiddle);
        let visible_tail = projection
            .visible()
            .rfind('v')
            .expect("middle projection keeps a source tail");
        let source_tail = value.rfind('v').expect("source tail");

        assert_eq!(
            projection
                .source_position(Position::new(visible_tail))
                .index,
            source_tail
        );
        assert!(projection.visible().is_char_boundary(visible_tail));
        assert!(value.is_char_boundary(source_tail));
    }

    #[test]
    fn single_line_projection_treats_following_display_lines_as_mapped_residue() {
        let mut engine = Engine::new();
        let value = "alpha\nbeta";
        let width = single_line_width(&mut engine, "alpha…", style()) + 1.0;
        let projection = engine.resolve_single_line_overflow_projection(
            value,
            style(),
            width,
            Overflow::EllipsisEnd,
        );

        assert_eq!(projection.visible(), "alpha…");
        assert!(!projection.visible().contains('\n'));
        assert_eq!(
            projection
                .source_position(Position::new(projection.visible().len()))
                .index,
            value.len()
        );
        assert_eq!(
            engine
                .resolve_overflow_projection(value, style(), width, Overflow::Clip)
                .visible(),
            value,
            "wrapped presentation preserves explicit Display line breaks"
        );
    }
}
