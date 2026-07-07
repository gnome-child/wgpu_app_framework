use super::super::buffer::Cursor;
use super::diagnostics::HighlightStats;

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct HighlightSpans {
    pub(super) selection: Vec<SelectionSpan>,
    pub(super) preedit_underline: Vec<SelectionSpan>,
    pub(super) preedit_selection: Vec<SelectionSpan>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionSpan {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) width: f32,
    pub(super) height: f32,
}

impl SelectionSpan {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn x(self) -> f32 {
        self.x
    }

    pub fn y(self) -> f32 {
        self.y
    }

    pub fn width(self) -> f32 {
        self.width
    }

    pub fn height(self) -> f32 {
        self.height
    }
}

pub(super) fn spans_for_ranges(
    buffer: &glyphon::Buffer,
    selection: Option<(Cursor, Cursor)>,
    preedit_underline: Option<(Cursor, Cursor)>,
    preedit_selection: Option<(Cursor, Cursor)>,
    vertical_offset: f32,
    scroll_x: f32,
    scroll_y: f32,
) -> (HighlightSpans, HighlightStats) {
    let mut spans = HighlightSpans::default();
    let mut stats = HighlightStats::default();

    for run in buffer.layout_runs() {
        stats.record_run_scan();
        let ranges = [
            line_highlight_range_for_run(&run, selection),
            line_highlight_range_for_run(&run, preedit_underline),
            line_highlight_range_for_run(&run, preedit_selection),
        ];
        if ranges.iter().all(Option::is_none) {
            continue;
        }

        let mut bounds = [(f32::INFINITY, f32::NEG_INFINITY); 3];
        for glyph in run.glyphs {
            let glyph_start = glyph.start.min(glyph.end);
            let glyph_end = glyph.start.max(glyph.end);
            for (role, range) in ranges.iter().enumerate() {
                let Some((line_start, line_end)) = range else {
                    continue;
                };
                if glyph_end <= *line_start || glyph_start >= *line_end {
                    continue;
                }
                bounds[role].0 = bounds[role].0.min(glyph.x);
                bounds[role].1 = bounds[role].1.max(glyph.x + glyph.w);
            }
        }

        push_highlight_span_from_bounds(
            &mut spans.selection,
            &mut stats,
            ranges[0],
            bounds[0],
            &run,
            vertical_offset,
            scroll_x,
            scroll_y,
        );
        push_highlight_span_from_bounds(
            &mut spans.preedit_underline,
            &mut stats,
            ranges[1],
            bounds[1],
            &run,
            vertical_offset,
            scroll_x,
            scroll_y,
        );
        push_highlight_span_from_bounds(
            &mut spans.preedit_selection,
            &mut stats,
            ranges[2],
            bounds[2],
            &run,
            vertical_offset,
            scroll_x,
            scroll_y,
        );
    }

    (spans, stats)
}

fn line_highlight_range_for_run(
    run: &glyphon::cosmic_text::LayoutRun<'_>,
    range: Option<(Cursor, Cursor)>,
) -> Option<(usize, usize)> {
    let (start, end) = range?;
    let (start, end) = if (end.line, end.index) < (start.line, start.index) {
        (end, start)
    } else {
        (start, end)
    };
    if run.line_i < start.line || run.line_i > end.line || run.glyphs.is_empty() {
        return None;
    }
    let line_start = if run.line_i == start.line {
        start.index
    } else {
        0
    };
    let line_end = if run.line_i == end.line {
        end.index
    } else {
        usize::MAX
    };
    (line_start < line_end).then_some((line_start, line_end))
}

fn push_highlight_span_from_bounds(
    spans: &mut Vec<SelectionSpan>,
    stats: &mut HighlightStats,
    range: Option<(usize, usize)>,
    bounds: (f32, f32),
    run: &glyphon::cosmic_text::LayoutRun<'_>,
    vertical_offset: f32,
    scroll_x: f32,
    scroll_y: f32,
) {
    if range.is_none() {
        return;
    }
    let (left, right) = bounds;
    if !left.is_finite() || !right.is_finite() || right <= left {
        stats.record_skip();
        return;
    }

    spans.push(SelectionSpan {
        x: left - scroll_x,
        y: vertical_offset + run.line_top - scroll_y,
        width: right - left,
        height: run.line_height,
    });
    stats.record_span();
}
