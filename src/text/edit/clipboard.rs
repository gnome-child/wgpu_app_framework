#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardError {
    Unavailable,
}

pub type ClipboardResult<T> = Result<T, ClipboardError>;

pub trait Clipboard {
    fn read_text(&mut self) -> ClipboardResult<Option<String>>;
    fn write_text(&mut self, text: &str) -> ClipboardResult<()>;
}
