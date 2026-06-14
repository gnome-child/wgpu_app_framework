use crate::geometry::{area, point};

use super::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Left,
    Right,
    Middle,
    Other(u16),
}

#[derive(Debug, Clone, PartialEq)]
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
    PointerMoved {
        position: point::Logical,
        target: Option<Path>,
    },
    PointerEntered {
        target: Path,
    },
    PointerLeft {
        target: Path,
    },
    PointerDown {
        position: point::Logical,
        target: Option<Path>,
        button: Button,
    },
    PointerUp {
        position: point::Logical,
        target: Option<Path>,
        button: Button,
    },
    Ignored,
}
