use crate::paint;

pub enum ItemBatch<'a> {
    Shapes(Vec<Shape<'a>>),
    Glyphs(Vec<Glyph<'a>>),
}

#[derive(Debug, Clone, Copy)]
pub enum Glyph<'a> {
    Text(&'a paint::Text),
    Icon(&'a paint::Icon),
}

pub enum Shape<'a> {
    Quad(&'a paint::Quad),
    Tint(&'a paint::Tint),
    Outline(&'a paint::Outline),
    BackdropBlur(&'a paint::Blur),
}

pub fn item_batches(items: &[paint::Item]) -> Vec<ItemBatch<'_>> {
    let mut batches = Vec::new();

    for item in items {
        match item {
            paint::Item::Quad(quad) => push_shape(&mut batches, Shape::Quad(quad)),
            paint::Item::Text(text) => push_glyph(&mut batches, Glyph::Text(text)),
            paint::Item::Icon(icon) => push_glyph(&mut batches, Glyph::Icon(icon)),
            paint::Item::Tint(tint) => push_shape(&mut batches, Shape::Tint(tint)),
            paint::Item::Outline(outline) => push_shape(&mut batches, Shape::Outline(outline)),
            paint::Item::BackdropBlur(blur) => push_shape(&mut batches, Shape::BackdropBlur(blur)),
        }
    }

    batches
}

fn push_shape<'a>(batches: &mut Vec<ItemBatch<'a>>, shape: Shape<'a>) {
    match batches.last_mut() {
        Some(ItemBatch::Shapes(shapes)) => shapes.push(shape),
        _ => batches.push(ItemBatch::Shapes(vec![shape])),
    }
}

fn push_glyph<'a>(batches: &mut Vec<ItemBatch<'a>>, glyph: Glyph<'a>) {
    match batches.last_mut() {
        Some(ItemBatch::Glyphs(glyphs)) => glyphs.push(glyph),
        _ => batches.push(ItemBatch::Glyphs(vec![glyph])),
    }
}

#[cfg(test)]
mod tests {
    use crate::geometry::{Rect, area, point};
    use crate::{icon, paint, text};

    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    enum Kind {
        Shapes(usize),
        Glyphs(usize),
    }

    fn solid_quad(x: f32) -> paint::Quad {
        paint::Quad {
            rect: Rect::new(point::logical(x, 0.0), area::logical(10.0, 10.0)),
            style: paint::Style {
                fill: Some(paint::Fill::Brush(paint::Brush::Solid(paint::Color::RED))),
                stroke: None,
                tint: None,
            },
        }
    }

    fn label(x: f32) -> paint::Text {
        paint::Text {
            rect: Rect::new(point::logical(x, 0.0), area::logical(10.0, 10.0)),
            document: text::Document::plain("Label"),
        }
    }

    fn icon(x: f32) -> paint::Icon {
        paint::Icon {
            rect: Rect::new(point::logical(x, 0.0), area::logical(10.0, 10.0)),
            icon: icon::Icon::phosphor(icon::Id::new("check")),
            color: paint::Color::BLACK,
            size: 16.0,
        }
    }

    fn tint(x: f32) -> paint::Tint {
        paint::Tint {
            rect: Rect::new(point::logical(x, 0.0), area::logical(10.0, 10.0)),
            color: paint::Color::rgba(1.0, 1.0, 1.0, 0.25),
        }
    }

    fn kinds(batches: &[ItemBatch<'_>]) -> Vec<Kind> {
        batches
            .iter()
            .map(|batch| match batch {
                ItemBatch::Shapes(shapes) => Kind::Shapes(shapes.len()),
                ItemBatch::Glyphs(glyphs) => Kind::Glyphs(glyphs.len()),
            })
            .collect()
    }

    #[test]
    fn item_batches_preserve_mixed_render_order() {
        let items = vec![
            paint::Item::Quad(solid_quad(0.0)),
            paint::Item::Tint(tint(1.0)),
            paint::Item::Text(label(2.0)),
            paint::Item::Icon(icon(2.5)),
            paint::Item::Quad(solid_quad(3.0)),
        ];

        assert_eq!(
            kinds(&item_batches(&items)),
            vec![Kind::Shapes(2), Kind::Glyphs(2), Kind::Shapes(1)]
        );
    }

    #[test]
    fn contiguous_text_and_icon_items_share_glyph_batch() {
        let items = vec![
            paint::Item::Text(label(0.0)),
            paint::Item::Icon(icon(1.0)),
            paint::Item::Text(label(2.0)),
        ];

        assert_eq!(kinds(&item_batches(&items)), vec![Kind::Glyphs(3)]);
    }

    #[test]
    fn backdrop_blur_batches_as_skipped_shape() {
        let items = vec![paint::Item::BackdropBlur(paint::Blur {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
            radius: 12.0,
        })];

        assert_eq!(kinds(&item_batches(&items)), vec![Kind::Shapes(1)]);
    }
}
