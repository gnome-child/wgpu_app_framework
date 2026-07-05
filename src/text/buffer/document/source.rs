use std::{fmt, path::PathBuf, sync::Arc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TextPieceSource {
    OriginalOwned,
    OriginalMapped,
    Add,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TextPiece {
    pub(super) source: TextPieceSource,
    pub(super) start: usize,
    pub(super) len: usize,
}

#[derive(Clone)]
pub(super) struct MappedTextSource {
    pub(super) path: PathBuf,
    pub(super) mmap: Arc<memmap2::Mmap>,
}

impl fmt::Debug for MappedTextSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MappedTextSource")
            .field("path", &self.path)
            .field("len", &self.mmap.len())
            .finish()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(super) enum TextOriginal {
    Owned(Arc<str>),
    Mapped(Arc<MappedTextSource>),
}
