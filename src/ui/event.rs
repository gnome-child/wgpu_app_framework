use crate::geometry::{area, point};
use crate::input::{Key, Modifiers};
use crate::pointer;
use crate::text;

use super::{Path, drag_drop};

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
        delta: point::Logical,
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
        delta: point::Logical,
        target: Option<Path>,
        button: pointer::Button,
    },
    PointerUp {
        position: point::Logical,
        delta: point::Logical,
        target: Option<Path>,
        button: pointer::Button,
    },
    ScrollWheel {
        position: point::Logical,
        delta: point::Logical,
        target: Option<Path>,
    },
    ScrollRequested {
        target: Path,
        offset: point::Logical,
    },
    TextEditRequested {
        target: Path,
        edit: text::edit::Edit,
    },
    TextDropRequested {
        source_cleanup: Option<(Path, text::edit::Edit)>,
        target: Path,
        edit: text::edit::Edit,
        operation: drag_drop::Operation,
    },
    KeyDown {
        key: Key,
        modifiers: Modifiers,
        target: Option<Path>,
        repeat: bool,
    },
    KeyUp {
        key: Key,
        modifiers: Modifiers,
        target: Option<Path>,
    },
    Ignored,
}
