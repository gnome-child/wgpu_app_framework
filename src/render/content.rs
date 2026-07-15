use crate::paint;

#[derive(Debug, Clone, Copy)]
pub(in crate::render) enum Glyph<'a> {
    Text(&'a paint::Text),
    TextViewport(&'a paint::TextViewport),
    Icon(&'a paint::Icon),
}

pub(in crate::render) enum Shape<'a> {
    Quad(&'a paint::Quad),
    Rule(&'a paint::Rule),
    Shadow(&'a paint::Shadow),
    Outline(&'a paint::Outline),
}
