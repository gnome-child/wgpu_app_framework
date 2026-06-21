pub mod buffer;
pub mod document;
pub mod edit;
pub mod layout;
pub mod surface;
pub mod unicode;
pub mod view;

pub use buffer::{
    Buffer, LineId, TextAffinity, TextMotion, TextPosition, TextRange, TextSelection,
};
pub use document::{Align, Block, Document, Role, Run, Style, TextDirection, Weight};
pub use edit::{
    Clipboard, ClipboardError, ClipboardResult, Command, CommandResult, Edit, Editor,
    PointerEditKind,
};
pub use layout::{
    Caret, CaretLayout, Diagnostics, Engine, Measure, Metrics, SelectionSpan, TextAreaPaintLayout,
    TextAreaSurface, TextFieldLayout,
};
pub use surface::{Area, AreaWrap, Field, FieldMode, Obscuring, Surface};
pub use view::{Preedit, RevealIntent, ScrollAnchor, TextViewState, View, Viewport, Visibility};

#[cfg(test)]
mod tests;
