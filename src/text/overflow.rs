/// How world-authored text behaves when its allocated width is too small.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Overflow {
    /// Preserve the source text and clip glyphs at the allocated bounds.
    #[default]
    Clip,
    /// Preserve a logical prefix and replace the omitted suffix with `…`.
    EllipsisEnd,
    /// Preserve logical head and tail segments around a `…` marker.
    EllipsisMiddle,
}
