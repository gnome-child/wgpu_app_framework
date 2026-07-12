use unicode_segmentation::UnicodeSegmentation;

use super::{Engine, Measure};
use crate::text::{
    self, Overflow,
    buffer::{Affinity, CursorSelection, Position},
};

const ELLIPSIS: &str = "…";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OverflowProjection {
    source: String,
    visible: String,
    mapping: Mapping,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mapping {
    Identity,
    Empty,
    End {
        head_end: usize,
    },
    Middle {
        head_end: usize,
        tail_start: usize,
        tail_visible_start: usize,
    },
}

#[derive(Clone, Copy)]
enum Edge {
    Start,
    End,
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
            mapping: Mapping::Identity,
        }
    }

    fn empty(source: &str) -> Self {
        Self {
            source: source.to_owned(),
            visible: String::new(),
            mapping: Mapping::Empty,
        }
    }

    fn ellipsis_end(source: &str, graphemes: &[&str], keep: usize) -> Self {
        let head_end = graphemes[..keep].iter().map(|value| value.len()).sum();
        Self {
            source: source.to_owned(),
            visible: end_candidate(graphemes, keep),
            mapping: Mapping::End { head_end },
        }
    }

    fn ellipsis_middle(source: &str, graphemes: &[&str], head: usize, tail: usize) -> Self {
        let head_end = graphemes[..head].iter().map(|value| value.len()).sum();
        let tail_bytes = graphemes[graphemes.len().saturating_sub(tail)..]
            .iter()
            .map(|value| value.len())
            .sum::<usize>();
        let tail_start = source.len().saturating_sub(tail_bytes);
        Self {
            source: source.to_owned(),
            visible: middle_candidate(graphemes, head, tail),
            mapping: Mapping::Middle {
                head_end,
                tail_start,
                tail_visible_start: head_end + ELLIPSIS.len(),
            },
        }
    }

    pub(crate) fn visible(&self) -> &str {
        &self.visible
    }

    pub(crate) fn source_position(&self, position: Position) -> Position {
        let index = position.index.min(self.visible.len());
        let source = match self.mapping {
            Mapping::Identity => index.min(self.source.len()),
            Mapping::Empty => 0,
            Mapping::End { head_end } => {
                if index <= head_end {
                    index
                } else {
                    self.source.len()
                }
            }
            Mapping::Middle {
                head_end,
                tail_start,
                tail_visible_start,
            } => {
                if index <= head_end {
                    index
                } else if index < tail_visible_start {
                    match position.affinity {
                        Affinity::Upstream => head_end,
                        Affinity::Downstream => tail_start,
                    }
                } else {
                    tail_start.saturating_add(index.saturating_sub(tail_visible_start))
                }
            }
        };
        Position::with_affinity(source.min(self.source.len()), position.affinity)
    }

    pub(crate) fn project_buffer_state(
        &self,
        source: &text::Buffer,
        state: text::edit::State,
    ) -> (text::Buffer, text::edit::State) {
        debug_assert_eq!(source.text(), self.source);
        let buffer = text::Buffer::from_multiline_text(self.visible.clone());
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
                buffer.cursor_for_position(focus),
                CursorSelection::Normal(buffer.cursor_for_position(anchor)),
            )
        } else {
            (
                buffer.cursor_for_position(self.display_position(source_cursor, Edge::End)),
                CursorSelection::None,
            )
        };
        let mut projected = buffer.initial_state();
        buffer.set_cursor_and_selection_for_state(&mut projected, cursor, selection);
        (buffer, projected)
    }

    fn display_position(&self, position: Position, edge: Edge) -> Position {
        let index = position.index.min(self.source.len());
        let visible = match self.mapping {
            Mapping::Identity => index.min(self.visible.len()),
            Mapping::Empty => 0,
            Mapping::End { head_end } => {
                if index <= head_end {
                    index
                } else {
                    match edge {
                        Edge::Start => head_end,
                        Edge::End => self.visible.len(),
                    }
                }
            }
            Mapping::Middle {
                head_end,
                tail_start,
                tail_visible_start,
            } => {
                if index <= head_end {
                    index
                } else if index >= tail_start {
                    tail_visible_start.saturating_add(index.saturating_sub(tail_start))
                } else {
                    match edge {
                        Edge::Start => head_end,
                        Edge::End => tail_visible_start,
                    }
                }
            }
        };
        Position::with_affinity(visible.min(self.visible.len()), position.affinity)
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
}
