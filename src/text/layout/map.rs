use std::rc::Rc;

use super::super::buffer::{Affinity, Position};
use super::constants::TEXT_LAYOUT_VISUAL_LINE_EPSILON;
use super::glyph::buffer_text_len;

pub(in crate::text) struct TextLayoutMap {
    pub(in crate::text) line_starts: Rc<Vec<usize>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::text) struct VisualLineGroup {
    pub(in crate::text) start: usize,
    pub(in crate::text) end: usize,
    pub(in crate::text) top: f32,
    pub(in crate::text) bottom: f32,
}

impl TextLayoutMap {
    #[cfg(test)]
    pub(in crate::text) fn new(buffer: &glyphon::Buffer) -> Self {
        Self {
            line_starts: Rc::new(super::glyph::line_start_offsets_for_buffer(buffer)),
        }
    }

    pub(in crate::text) fn from_line_starts(line_starts: Rc<Vec<usize>>) -> Self {
        Self { line_starts }
    }

    #[cfg(test)]
    pub(in crate::text) fn hit(
        &self,
        buffer: &glyphon::Buffer,
        x: f32,
        y: f32,
    ) -> Option<Position> {
        self.hit_with_observer(buffer, x, y, |_| {})
    }

    pub(super) fn hit_with_observer(
        &self,
        buffer: &glyphon::Buffer,
        x: f32,
        y: f32,
        mut observe_run_scans: impl FnMut(usize),
    ) -> Option<Position> {
        let runs: Vec<_> = buffer.layout_runs().collect();
        observe_run_scans(runs.len());

        if runs.is_empty() {
            return Some(Position::new(buffer_text_len(buffer)));
        }

        let groups = Self::visual_line_groups(&runs);
        let mut nearest_line = None::<(f32, VisualLineGroup)>;
        for group in groups {
            if y >= group.top - TEXT_LAYOUT_VISUAL_LINE_EPSILON
                && y <= group.bottom + TEXT_LAYOUT_VISUAL_LINE_EPSILON
            {
                return self.hit_line(&runs[group.start..group.end], x);
            }

            let distance = if y < group.top {
                group.top - y
            } else {
                y - group.bottom
            };
            if nearest_line
                .as_ref()
                .is_none_or(|(best_distance, _)| distance < *best_distance)
            {
                nearest_line = Some((distance, group));
            }
        }

        nearest_line
            .and_then(|(_, group)| self.hit_line(&runs[group.start..group.end], x))
            .or_else(|| Some(Position::new(buffer_text_len(buffer))))
    }

    pub(in crate::text) fn visual_line_groups(
        runs: &[glyphon::cosmic_text::LayoutRun<'_>],
    ) -> Vec<VisualLineGroup> {
        let mut groups = Vec::new();
        for (index, run) in runs.iter().enumerate() {
            let top = run.line_top;
            let bottom = run.line_top + run.line_height;
            let same_visual_line = groups.last().is_some_and(|group: &VisualLineGroup| {
                let first = &runs[group.start];
                first.line_i == run.line_i
                    && (group.top - top).abs() <= TEXT_LAYOUT_VISUAL_LINE_EPSILON
                    && (group.bottom - bottom).abs() <= TEXT_LAYOUT_VISUAL_LINE_EPSILON
            });

            if same_visual_line {
                let group = groups.last_mut().expect("last group should exist");
                group.end = index + 1;
                group.top = group.top.min(top);
                group.bottom = group.bottom.max(bottom);
            } else {
                groups.push(VisualLineGroup {
                    start: index,
                    end: index + 1,
                    top,
                    bottom,
                });
            }
        }
        groups
    }

    fn hit_line(&self, runs: &[glyphon::cosmic_text::LayoutRun<'_>], x: f32) -> Option<Position> {
        let mut nearest = None::<(f32, Position)>;
        let fallback_line = runs
            .first()
            .and_then(|run| self.line_starts.get(run.line_i).copied())
            .unwrap_or(0);

        for run in runs {
            let Some((left, right)) = Self::run_visual_bounds(run) else {
                continue;
            };

            if x >= left
                && x <= right
                && let Some(position) = self.hit_run(run, x)
            {
                return Some(position);
            }

            for (edge_x, visual_left) in [(left, true), (right, false)] {
                let Some(position) = self.run_edge_position(run, visual_left) else {
                    continue;
                };
                let distance = (x - edge_x).abs();
                if nearest
                    .as_ref()
                    .is_none_or(|(best_distance, _)| distance < *best_distance)
                {
                    nearest = Some((distance, position));
                }
            }
        }

        nearest
            .map(|(_, position)| position)
            .or_else(|| Some(Position::new(fallback_line)))
    }

    fn hit_run(&self, run: &glyphon::cosmic_text::LayoutRun<'_>, x: f32) -> Option<Position> {
        let line_start = self.line_starts.get(run.line_i).copied().unwrap_or(0);
        if run.glyphs.is_empty() {
            return Some(Position::new(line_start));
        }

        let (left, right) = Self::run_visual_bounds(run)?;
        if x <= left {
            return self.run_edge_position(run, true);
        }
        if x >= right {
            return self.run_edge_position(run, false);
        }
        for glyph in run.glyphs {
            let left = glyph.x;
            let right = glyph.x + glyph.w;
            if x >= left && x <= right {
                return Some(self.visual_position(line_start, glyph, x <= left + glyph.w * 0.5));
            }
        }

        None
    }

    pub(in crate::text) fn run_visual_bounds(
        run: &glyphon::cosmic_text::LayoutRun<'_>,
    ) -> Option<(f32, f32)> {
        let mut glyphs = run.glyphs.iter();
        let first = glyphs.next()?;
        let mut left = first.x;
        let mut right = first.x + first.w;

        for glyph in glyphs {
            left = left.min(glyph.x);
            right = right.max(glyph.x + glyph.w);
        }

        Some((left, right))
    }

    pub(in crate::text) fn run_edge_position(
        &self,
        run: &glyphon::cosmic_text::LayoutRun<'_>,
        visual_left: bool,
    ) -> Option<Position> {
        let line_start = self.line_starts.get(run.line_i).copied().unwrap_or(0);
        let mut glyph = run.glyphs.first()?;

        for candidate in run.glyphs {
            if visual_left {
                if candidate.x < glyph.x {
                    glyph = candidate;
                }
            } else if candidate.x + candidate.w > glyph.x + glyph.w {
                glyph = candidate;
            }
        }

        Some(self.visual_position(line_start, glyph, visual_left))
    }

    fn visual_position(
        &self,
        line_start: usize,
        glyph: &glyphon::LayoutGlyph,
        visual_left: bool,
    ) -> Position {
        let glyph_rtl = glyph.level.is_rtl();
        match (glyph_rtl, visual_left) {
            (false, true) => {
                Position::with_affinity(line_start + glyph.start, Affinity::Downstream)
            }
            (false, false) => Position::with_affinity(line_start + glyph.end, Affinity::Upstream),
            (true, true) => Position::with_affinity(line_start + glyph.end, Affinity::Upstream),
            (true, false) => {
                Position::with_affinity(line_start + glyph.start, Affinity::Downstream)
            }
        }
    }
}
