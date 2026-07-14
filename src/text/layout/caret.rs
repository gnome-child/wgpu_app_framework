use super::super::edit::{ViewState, Viewport, Visibility};
use super::constants::TEXT_FIELD_CARET_MARGIN;
use crate::geometry::{area, point};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Caret {
    pub(in crate::text) x: f32,
    pub(in crate::text) y: f32,
    pub(in crate::text) height: f32,
}

impl Caret {
    pub fn new(x: f32, y: f32, height: f32) -> Self {
        Self { x, y, height }
    }

    pub fn x(self) -> f32 {
        self.x
    }

    pub fn y(self) -> f32 {
        self.y
    }

    pub fn height(self) -> f32 {
        self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaretLayout {
    caret: Caret,
}

impl CaretLayout {
    pub fn new(caret: Caret) -> Self {
        Self { caret }
    }

    pub fn caret(self) -> Caret {
        self.caret
    }

    pub fn visibility_in(self, viewport: Viewport, margin: f32) -> Visibility {
        viewport.visibility_of_local_caret(self.caret, margin)
    }
}

pub(super) fn ensure_visible_from_layout(
    state: ViewState,
    viewport: area::Logical,
    caret_layout: CaretLayout,
    content_area: Option<area::Logical>,
) -> Option<ViewState> {
    let viewport_state =
        Viewport::new(viewport, point::logical(state.scroll_x(), state.scroll_y()));
    let visibility = caret_layout.visibility_in(viewport_state, TEXT_FIELD_CARET_MARGIN);
    if visibility.is_visible() {
        return Some(state);
    }
    if matches!(visibility, Visibility::Unknown) {
        return None;
    }

    let caret = caret_layout.caret();
    let mut scroll_x = state.scroll_x();
    let mut scroll_y = state.scroll_y();
    match visibility {
        Visibility::Above => {
            scroll_y = scroll_y + caret.y() - TEXT_FIELD_CARET_MARGIN;
        }
        Visibility::Below => {
            scroll_y =
                scroll_y + caret.y() + caret.height() + TEXT_FIELD_CARET_MARGIN - viewport.height();
        }
        Visibility::Before => {
            scroll_x = scroll_x + caret.x() - TEXT_FIELD_CARET_MARGIN;
        }
        Visibility::After => {
            scroll_x = scroll_x + caret.x() + 1.0 + TEXT_FIELD_CARET_MARGIN - viewport.width();
        }
        Visibility::Visible | Visibility::Unknown => {}
    }

    if let Some(content_area) = content_area {
        let max_scroll_x = (content_area.width() - viewport.width()).max(0.0);
        let max_scroll_y = (content_area.height() - viewport.height()).max(0.0);
        scroll_x = scroll_x.clamp(0.0, max_scroll_x);
        scroll_y = scroll_y.clamp(0.0, max_scroll_y);
    }

    Some(state.with_scroll(scroll_x, scroll_y))
}
