use unicode_segmentation::UnicodeSegmentation;

use super::{Engine, Measure};
use crate::text::{self, Overflow};

const ELLIPSIS: &str = "…";

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
        let width = finite_width(width);
        if overflow == Overflow::Clip || self.fits(value, style, width) {
            return value.to_owned();
        }
        if !self.fits(ELLIPSIS, style, width) {
            return String::new();
        }

        let graphemes = UnicodeSegmentation::graphemes(value, true).collect::<Vec<_>>();
        match overflow {
            Overflow::Clip => value.to_owned(),
            Overflow::EllipsisEnd => self.ellipsis_end(&graphemes, style, width),
            Overflow::EllipsisMiddle => self.ellipsis_middle(&graphemes, style, width),
        }
    }

    fn ellipsis_end(
        &mut self,
        graphemes: &[&str],
        style: text::document::Style,
        width: f32,
    ) -> String {
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
        end_candidate(graphemes, low)
    }

    fn ellipsis_middle(
        &mut self,
        graphemes: &[&str],
        style: text::document::Style,
        width: f32,
    ) -> String {
        if graphemes.is_empty() {
            return ELLIPSIS.to_owned();
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

        middle_candidate(graphemes, head, tail)
    }

    fn fits(&mut self, value: &str, style: text::document::Style, width: f32) -> bool {
        single_line_width(self, value, style) <= width
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
}
