use crate::paint;

pub enum ItemBatch<'a> {
    Shapes(Vec<Shape<'a>>),
    Texts(Vec<&'a paint::Text>),
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
            paint::Item::Text(text) => match batches.last_mut() {
                Some(ItemBatch::Texts(texts)) => texts.push(text),
                _ => batches.push(ItemBatch::Texts(vec![text])),
            },
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

#[cfg(test)]
mod tests {
    use crate::geometry::{Rect, area, point};
    use crate::{paint, text};

    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    enum Kind {
        Shapes(usize),
        Texts(usize),
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
                ItemBatch::Texts(texts) => Kind::Texts(texts.len()),
            })
            .collect()
    }

    #[test]
    fn item_batches_preserve_mixed_render_order() {
        let items = vec![
            paint::Item::Quad(solid_quad(0.0)),
            paint::Item::Tint(tint(1.0)),
            paint::Item::Text(label(2.0)),
            paint::Item::Quad(solid_quad(3.0)),
        ];

        assert_eq!(
            kinds(&item_batches(&items)),
            vec![Kind::Shapes(2), Kind::Texts(1), Kind::Shapes(1)]
        );
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
