use crate::geometry::{area, point};

use super::{Caret, LineId};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    area: area::Logical,
    scroll: point::Logical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Above,
    Below,
    Before,
    After,
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RevealIntent {
    #[default]
    None,
    CaretIfNeeded,
    CaretForce,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollAnchor {
    line_id: LineId,
    offset_y: f32,
}

impl Viewport {
    pub fn new(area: area::Logical, scroll: point::Logical) -> Self {
        Self {
            area,
            scroll: point::logical(scroll.x().max(0.0), scroll.y().max(0.0)),
        }
    }

    pub fn area(self) -> area::Logical {
        self.area
    }

    pub fn scroll(self) -> point::Logical {
        self.scroll
    }

    pub fn visibility_of_local_caret(self, caret: Caret, margin: f32) -> Visibility {
        let margin = margin.max(0.0);
        if caret.y() + caret.height() < -margin {
            return Visibility::Above;
        }
        if caret.y() > self.area.height() + margin {
            return Visibility::Below;
        }
        if caret.x() < -margin {
            return Visibility::Before;
        }
        if caret.x() > self.area.width() + margin {
            return Visibility::After;
        }
        Visibility::Visible
    }
}

impl Visibility {
    pub fn is_visible(self) -> bool {
        matches!(self, Self::Visible)
    }
}

impl RevealIntent {
    pub fn should_reveal(self) -> bool {
        !matches!(self, Self::None)
    }

    pub fn if_needed(self) -> bool {
        matches!(self, Self::CaretIfNeeded)
    }
}

impl ScrollAnchor {
    pub fn new(line_id: LineId, offset_y: f32) -> Self {
        Self { line_id, offset_y }
    }

    pub fn line_id(self) -> LineId {
        self.line_id
    }

    pub fn offset_y(self) -> f32 {
        self.offset_y
    }
}
