use crate::paint;

pub(in crate::render) enum ItemBatch<'a> {
    Shapes(Vec<Shape<'a>>),
    Pane(&'a paint::Pane),
    Glyphs(Vec<Glyph<'a>>),
    PushClip(&'a paint::Clip),
    PopClip,
    Group(&'a paint::Group),
}

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

pub(in crate::render) fn item_batches(items: &[paint::Item]) -> Vec<ItemBatch<'_>> {
    let mut batches = Vec::new();

    for item in items {
        match item {
            paint::Item::Quad(quad) => push_shape(&mut batches, Shape::Quad(quad)),
            paint::Item::Rule(rule) => push_shape(&mut batches, Shape::Rule(rule)),
            paint::Item::Text(text) => push_glyph(&mut batches, Glyph::Text(text)),
            paint::Item::TextViewport(text) => push_glyph(&mut batches, Glyph::TextViewport(text)),
            paint::Item::Icon(icon) => push_glyph(&mut batches, Glyph::Icon(icon)),
            paint::Item::Shadow(shadow) => push_shape(&mut batches, Shape::Shadow(shadow)),
            paint::Item::Outline(outline) => push_shape(&mut batches, Shape::Outline(outline)),
            paint::Item::Pane(pane) => batches.push(ItemBatch::Pane(pane)),
            paint::Item::Clip(clip) => batches.push(ItemBatch::PushClip(clip)),
            paint::Item::PopClip => batches.push(ItemBatch::PopClip),
            paint::Item::Group(group) => batches.push(ItemBatch::Group(group)),
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
    use crate::paint::{self, Rect};
    use crate::{icon, text};

    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    enum Kind {
        Shapes(usize),
        Pane,
        Glyphs(usize),
        PushClip,
        PopClip,
        Group,
    }

    fn solid_quad(x: f32) -> paint::Quad {
        paint::Quad::unchecked_for_test(
            Rect::new(
                paint::point::logical(x, 0.0),
                paint::area::logical(10.0, 10.0),
            ),
            paint::Style {
                fill: Some(paint::Fill::Brush(paint::Brush::solid(paint::Color::RED))),
                stroke: None,
                tint: None,
            },
            paint::Rasterization::default(),
            paint::Transform::identity(),
        )
    }

    fn label(x: f32) -> paint::Text {
        paint::Text {
            rect: Rect::new(
                paint::point::logical(x, 0.0),
                paint::area::logical(10.0, 10.0),
            ),
            document: text::document::Document::plain("Label"),
            wrap: paint::TextWrap::WordOrGlyph,
            vertical_align: paint::TextVerticalAlign::Center,
        }
    }

    fn icon(x: f32) -> paint::Icon {
        paint::Icon {
            rect: Rect::new(
                paint::point::logical(x, 0.0),
                paint::area::logical(10.0, 10.0),
            ),
            icon: icon::Icon::phosphor(icon::Id::new("check")),
            color: paint::Color::BLACK,
            size: 16.0,
        }
    }

    fn shadow(x: f32) -> paint::Shadow {
        paint::Shadow {
            rect: Rect::new(
                paint::point::logical(x, 0.0),
                paint::area::logical(10.0, 10.0),
            ),
            brush: paint::Brush::solid(paint::Color::rgba(0.0, 0.0, 0.0, 0.35)),
            blur: 16.0,
            spread: 1.0,
            offset: paint::point::logical(0.0, 4.0),
        }
    }

    fn kinds(batches: &[ItemBatch<'_>]) -> Vec<Kind> {
        batches
            .iter()
            .map(|batch| match batch {
                ItemBatch::Shapes(shapes) => Kind::Shapes(shapes.len()),
                ItemBatch::Pane(_) => Kind::Pane,
                ItemBatch::Glyphs(glyphs) => Kind::Glyphs(glyphs.len()),
                ItemBatch::PushClip(_) => Kind::PushClip,
                ItemBatch::PopClip => Kind::PopClip,
                ItemBatch::Group(_) => Kind::Group,
            })
            .collect()
    }

    #[test]
    fn item_batches_preserve_mixed_render_order() {
        let items = vec![
            paint::Item::Quad(solid_quad(0.0)),
            paint::Item::Shadow(shadow(0.5)),
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
    fn pane_batches_as_own_ordered_material_operation() {
        let pane = paint::Pane::new(
            Rect::new(
                paint::point::logical(1.0, 0.0),
                paint::area::logical(10.0, 10.0),
            ),
            paint::Material::Glass(paint::Glass {
                fallback: paint::Brush::solid(paint::Color::BLACK),
                backdrop_layers: vec![paint::BackdropLayer::Blur(paint::BackdropBlur {
                    sigma: 10.0,
                    edge_mode: paint::BackdropEdgeMode::Mirror,
                })],
                surface_layers: Vec::new(),
            }),
        );
        let items = vec![
            paint::Item::Quad(solid_quad(0.0)),
            paint::Item::Pane(pane),
            paint::Item::Quad(solid_quad(2.0)),
        ];

        assert_eq!(
            kinds(&item_batches(&items)),
            vec![Kind::Shapes(1), Kind::Pane, Kind::Shapes(1)]
        );
    }

    #[test]
    fn clip_commands_split_batches_to_preserve_order() {
        let clip = paint::Clip {
            rect: Rect::new(
                paint::point::logical(1.0, 0.0),
                paint::area::logical(10.0, 10.0),
            ),
        };
        let items = vec![
            paint::Item::Quad(solid_quad(0.0)),
            paint::Item::Clip(clip),
            paint::Item::Text(label(1.0)),
            paint::Item::PopClip,
            paint::Item::Quad(solid_quad(2.0)),
        ];

        assert_eq!(
            kinds(&item_batches(&items)),
            vec![
                Kind::Shapes(1),
                Kind::PushClip,
                Kind::Glyphs(1),
                Kind::PopClip,
                Kind::Shapes(1)
            ]
        );
    }

    #[test]
    fn group_batches_as_own_ordered_operation() {
        let group = paint::Group {
            bounds: Rect::new(
                paint::point::logical(0.0, 0.0),
                paint::area::logical(10.0, 10.0),
            ),
            opacity: 0.5,
            items: vec![paint::Item::Quad(solid_quad(0.0))],
        };
        let items = vec![
            paint::Item::Quad(solid_quad(0.0)),
            paint::Item::Group(group),
            paint::Item::Text(label(3.0)),
        ];

        assert_eq!(
            kinds(&item_batches(&items)),
            vec![Kind::Shapes(1), Kind::Group, Kind::Glyphs(1)]
        );
    }

    #[test]
    fn promoted_group_text_stays_in_separate_glyph_batch() {
        let group = paint::Group {
            bounds: Rect::new(
                paint::point::logical(20.0, 20.0),
                paint::area::logical(40.0, 20.0),
            ),
            opacity: 0.5,
            items: vec![paint::Item::Text(label(0.0)), paint::Item::Icon(icon(12.0))],
        };
        let items = vec![paint::Item::Text(label(0.0)), paint::Item::Group(group)];
        let batches = item_batches(&items);
        let group = match &batches[1] {
            ItemBatch::Group(group) => group,
            _ => panic!("second batch should be the promoted group"),
        };

        assert_eq!(kinds(&batches), vec![Kind::Glyphs(1), Kind::Group]);
        assert_eq!(kinds(&item_batches(&group.items)), vec![Kind::Glyphs(2)]);
    }
}
