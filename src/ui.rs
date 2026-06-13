use crate::geometry::{area, point};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Resized {
        area: area::Physical,
        scale_factor: f32,
    },
    ScaleFactorChanged {
        scale_factor: f32,
    },
    CloseRequested,
    Focused(bool),
    CursorMoved {
        position: point::Logical,
    },
    Ignored,
}
