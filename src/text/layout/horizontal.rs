use std::ops::Range;

use super::constants::TEXT_AREA_RENDER_HORIZONTAL_OVERSCAN;

pub(super) const TEXT_AREA_HORIZONTAL_INDEX_MAX_SOURCE_SPAN: u32 = 4_096;

pub(super) fn prepared_window(viewport_width: f32, scroll_x: f32) -> (f32, f32) {
    let viewport_width = viewport_width.max(1.0);
    let overscan = TEXT_AREA_RENDER_HORIZONTAL_OVERSCAN.max(1.0);
    let width = viewport_width + overscan * 2.0;
    let desired_origin = (scroll_x - overscan).max(0.0);
    let origin = (desired_origin / overscan).floor() * overscan;
    (origin, width)
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Checkpoint {
    source_byte: u32,
    x: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Window {
    pub(super) source: Range<usize>,
    pub(super) x: f32,
    pub(super) width: f32,
}

#[derive(Debug, Clone)]
pub(super) struct LineIndex {
    checkpoints: Vec<Checkpoint>,
    source_len: usize,
    width: f32,
    height: f32,
}

impl LineIndex {
    pub(super) fn from_ascii_buffer(text: &str, buffer: &glyphon::Buffer) -> Option<Self> {
        if !text.is_ascii() || text.is_empty() {
            return None;
        }
        let source_len = u32::try_from(text.len()).ok()?;

        let mut runs = buffer.layout_runs();
        let run = runs.next()?;
        if runs.next().is_some() || run.rtl || run.text != text || run.line_w <= 0.0 {
            return None;
        }

        let mut checkpoints = vec![Checkpoint {
            source_byte: 0,
            x: 0.0,
        }];
        let mut last_x = 0.0_f32;
        let mut terminal_x = run.line_w;
        let mut pending_boundary = false;
        for glyph in run.glyphs {
            if glyph.end < glyph.start || glyph.end > text.len() || glyph.x < last_x {
                return None;
            }
            let cluster = text.get(glyph.start..glyph.end)?;
            let whitespace = cluster.chars().all(char::is_whitespace);
            if pending_boundary && !whitespace {
                let glyph_start = u32::try_from(glyph.start).ok()?;
                if checkpoints
                    .last()
                    .is_none_or(|checkpoint| checkpoint.source_byte != glyph_start)
                {
                    checkpoints.push(Checkpoint {
                        source_byte: glyph_start,
                        x: glyph.x,
                    });
                }
                pending_boundary = false;
            }
            if whitespace {
                pending_boundary = true;
            }
            last_x = glyph.x;
            terminal_x = terminal_x.max(glyph.x + glyph.w);
        }

        let terminal = Checkpoint {
            source_byte: source_len,
            x: terminal_x.max(checkpoints.last()?.x),
        };
        if checkpoints
            .last()
            .is_none_or(|last| last.source_byte != terminal.source_byte || last.x != terminal.x)
        {
            checkpoints.push(terminal);
        }
        if checkpoints.len() < 3
            || checkpoints.windows(2).any(|pair| {
                pair[0].source_byte >= pair[1].source_byte
                    || pair[0].x > pair[1].x
                    || pair[1].source_byte - pair[0].source_byte
                        > TEXT_AREA_HORIZONTAL_INDEX_MAX_SOURCE_SPAN
            })
        {
            return None;
        }

        Some(Self {
            checkpoints,
            source_len: text.len(),
            width: run.line_w,
            height: run.line_height.max(1.0),
        })
    }

    pub(super) fn width(&self) -> f32 {
        self.width
    }

    pub(super) fn height(&self) -> f32 {
        self.height
    }

    pub(super) fn checkpoint_count(&self) -> usize {
        self.checkpoints.len()
    }

    pub(super) fn resident_bytes(&self) -> usize {
        self.checkpoints
            .len()
            .saturating_mul(std::mem::size_of::<Checkpoint>())
    }

    pub(super) fn windows_for_x(&self, origin: f32, width: f32) -> Vec<Window> {
        let origin = origin.max(0.0).min(self.width);
        let end = (origin + width.max(1.0)).min(self.width);
        let start_index = self
            .checkpoints
            .partition_point(|checkpoint| checkpoint.x <= origin)
            .saturating_sub(1)
            .saturating_sub(1);
        let end_index = self
            .checkpoints
            .partition_point(|checkpoint| checkpoint.x < end)
            .saturating_add(1)
            .min(self.checkpoints.len().saturating_sub(1));
        self.windows_between(start_index, end_index.max(start_index + 1))
    }

    pub(super) fn windows_for_source_byte(&self, source_byte: usize) -> Vec<Window> {
        let source_byte = source_byte.min(self.source_len);
        let source_byte = u32::try_from(source_byte).unwrap_or(u32::MAX);
        let containing = self
            .checkpoints
            .partition_point(|checkpoint| checkpoint.source_byte <= source_byte)
            .saturating_sub(1)
            .min(self.checkpoints.len().saturating_sub(2));
        let start = containing.saturating_sub(1);
        let end = containing.saturating_add(2).min(self.checkpoints.len() - 1);
        self.windows_between(start, end.max(start + 1))
    }

    fn windows_between(&self, start: usize, end: usize) -> Vec<Window> {
        let start = start.min(self.checkpoints.len().saturating_sub(2));
        let end = end.clamp(start + 1, self.checkpoints.len() - 1);
        (start..end)
            .map(|index| self.window_between(index, index + 1))
            .collect()
    }

    fn window_between(&self, start: usize, end: usize) -> Window {
        let start = start.min(self.checkpoints.len().saturating_sub(2));
        let end = end.clamp(start + 1, self.checkpoints.len() - 1);
        let first = self.checkpoints[start];
        let last = self.checkpoints[end];
        Window {
            source: first.source_byte as usize..last.source_byte as usize,
            x: first.x,
            width: (last.x - first.x).max(0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index() -> LineIndex {
        LineIndex {
            checkpoints: vec![
                Checkpoint {
                    source_byte: 0,
                    x: 0.0,
                },
                Checkpoint {
                    source_byte: 100,
                    x: 1_000.0,
                },
                Checkpoint {
                    source_byte: 200,
                    x: 2_000.0,
                },
                Checkpoint {
                    source_byte: 300,
                    x: 3_000.0,
                },
                Checkpoint {
                    source_byte: 400,
                    x: 4_000.0,
                },
            ],
            source_len: 400,
            width: 4_000.0,
            height: 20.0,
        }
    }

    #[test]
    fn x_window_is_bounded_by_neighboring_safe_checkpoints() {
        let windows = index().windows_for_x(1_450.0, 920.0);
        assert_eq!(windows.len(), 4);
        assert_eq!(windows[0].source, 0..100);
        assert_eq!(windows[3].source, 300..400);
        assert!(windows[0].x <= 1_450.0);
        assert!(windows[3].x + windows[3].width >= 2_370.0);
    }

    #[test]
    fn source_window_includes_a_guard_checkpoint_on_each_side() {
        let windows = index().windows_for_source_byte(250);
        assert_eq!(windows.len(), 3);
        assert_eq!(windows[0].source, 100..200);
        assert_eq!(windows[2].source, 300..400);
    }
}
