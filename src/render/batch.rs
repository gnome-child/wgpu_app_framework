use crate::paint;

pub enum ItemBatch<'a> {
    Quads(Vec<&'a paint::Quad>),
    Texts(Vec<&'a paint::Text>),
}

pub fn item_batches(items: &[paint::Item]) -> Vec<ItemBatch<'_>> {
    let mut batches = Vec::new();

    for item in items {
        match item {
            paint::Item::Quad(quad) => match batches.last_mut() {
                Some(ItemBatch::Quads(quads)) => quads.push(quad),
                _ => batches.push(ItemBatch::Quads(vec![quad])),
            },
            paint::Item::Text(text) => match batches.last_mut() {
                Some(ItemBatch::Texts(texts)) => texts.push(text),
                _ => batches.push(ItemBatch::Texts(vec![text])),
            },
        }
    }

    batches
}

#[cfg(test)]
mod tests {
    use crate::geometry::{Rect, area, point};
    use crate::{paint, text};

    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    enum Kind {
        Quads(usize),
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

    fn kinds(batches: &[ItemBatch<'_>]) -> Vec<Kind> {
        batches
            .iter()
            .map(|batch| match batch {
                ItemBatch::Quads(quads) => Kind::Quads(quads.len()),
                ItemBatch::Texts(texts) => Kind::Texts(texts.len()),
            })
            .collect()
    }

    #[test]
    fn item_batches_preserve_mixed_render_order() {
        let items = vec![
            paint::Item::Quad(solid_quad(0.0)),
            paint::Item::Quad(solid_quad(1.0)),
            paint::Item::Text(label(2.0)),
            paint::Item::Quad(solid_quad(3.0)),
        ];

        assert_eq!(
            kinds(&item_batches(&items)),
            vec![Kind::Quads(2), Kind::Texts(1), Kind::Quads(1)]
        );
    }
}
