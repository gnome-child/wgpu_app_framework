use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::*;
use crate::geometry::{Rect, area, point, rect};
use crate::widget::menu;
use crate::{action, icon, layout, paint, pointer, text, theme, widget, window};

const ROOT: Id = Id::new("root");
const A: Id = Id::new("a");
const B: Id = Id::new("b");
const C: Id = Id::new("c");
const D: Id = Id::new("d");
const E: Id = Id::new("e");
const F: Id = Id::new("f");
const G: Id = Id::new("g");
const CLICK: action::Id = action::Id::new("click");
const OTHER_CLICK: action::Id = action::Id::new("other_click");

fn layout(tree: &Tree) -> Frame {
    layout_area(tree, area::logical(100.0, 80.0))
}

fn layout_area(tree: &Tree, area: area::Logical) -> Frame {
    let mut measurer = text::Engine::new();

    tree.layout(area, &mut measurer)
        .expect("tree should have root")
}

fn path(id: Id) -> Path {
    Path::from(id)
}

fn quad(scene: &paint::Scene, index: usize) -> paint::Quad {
    match scene.items().get(index) {
        Some(paint::Item::Quad(quad)) => *quad,
        item => panic!("expected quad item at {index}, got {item:?}"),
    }
}

fn text(scene: &paint::Scene, index: usize) -> &paint::Text {
    match scene.items().get(index) {
        Some(paint::Item::Text(text)) => text,
        item => panic!("expected text item at {index}, got {item:?}"),
    }
}

fn caret_quad_count(scene: &paint::Scene) -> usize {
    scene
        .items()
        .iter()
        .filter(|item| {
            matches!(
                item,
                paint::Item::Quad(quad)
                    if (quad.rect.area.width() - 1.0).abs() <= f32::EPSILON
            )
        })
        .count()
}

fn selection_quad_count(scene: &paint::Scene) -> usize {
    let fill = Some(paint::Fill::Brush(
        paint::Color::rgba(0.18, 0.42, 0.86, 0.48).into(),
    ));

    scene
        .items()
        .iter()
        .filter(|item| matches!(item, paint::Item::Quad(quad) if quad.style.fill == fill))
        .count()
}

fn icon_item(scene: &paint::Scene, index: usize) -> paint::Icon {
    match scene.items().get(index) {
        Some(paint::Item::Icon(icon)) => *icon,
        item => panic!("expected icon item at {index}, got {item:?}"),
    }
}

fn tint(scene: &paint::Scene, index: usize) -> paint::Tint {
    match scene.items().get(index) {
        Some(paint::Item::Tint(tint)) => *tint,
        item => panic!("expected tint item at {index}, got {item:?}"),
    }
}

fn outline(scene: &paint::Scene, index: usize) -> paint::Outline {
    match scene.items().get(index) {
        Some(paint::Item::Outline(outline)) => *outline,
        item => panic!("expected outline item at {index}, got {item:?}"),
    }
}

fn shadow(scene: &paint::Scene, index: usize) -> paint::Shadow {
    match scene.items().get(index) {
        Some(paint::Item::Shadow(shadow)) => *shadow,
        item => panic!("expected shadow item at {index}, got {item:?}"),
    }
}

fn backdrop(scene: &paint::Scene, index: usize) -> paint::Backdrop {
    match scene.items().get(index) {
        Some(paint::Item::Backdrop(backdrop)) => *backdrop,
        item => panic!("expected backdrop item at {index}, got {item:?}"),
    }
}

fn clip(scene: &paint::Scene, index: usize) -> paint::Clip {
    match scene.items().get(index) {
        Some(paint::Item::Clip(clip)) => *clip,
        item => panic!("expected clip item at {index}, got {item:?}"),
    }
}

fn assert_same_bounds(actual: Rect, expected: Rect) {
    assert_eq!(actual.origin, expected.origin);
    assert_eq!(actual.area, expected.area);
}

fn check_icon() -> icon::Icon {
    icon::Icon::phosphor(icon::Id::new("check"))
}

fn gradient_brush() -> paint::Brush {
    paint::Brush::linear_gradient(
        paint::Color::rgba(0.1, 0.2, 0.8, 0.35),
        paint::Color::rgba(0.8, 0.2, 0.5, 0.75),
    )
}

#[test]
fn fixed_and_fill_vertical_layout() {
    let mut root = Node::container(ROOT, layout::Axis::Vertical);
    root.push_child(Node::leaf(A).with_size(layout::Size::Fill, layout::Size::Fixed(20.0)));
    root.push_child(Node::leaf(B));

    let mut tree = Tree::new();
    tree.set_root(root);
    let layout = layout(&tree);

    assert_eq!(layout.children()[0].rect().area, area::logical(100.0, 20.0));
    assert_eq!(layout.children()[1].rect().area, area::logical(100.0, 60.0));
    assert_eq!(
        layout.children()[1].rect().origin,
        point::logical(0.0, 20.0)
    );
}

#[test]
fn padding_offsets_children() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_padding(layout::Insets::splat(10.0))
        .with_child(Node::leaf(A));

    let mut tree = Tree::new();
    tree.set_root(root);
    let layout = layout(&tree);

    assert_eq!(
        layout.children()[0].rect().origin,
        point::logical(10.0, 10.0)
    );
    assert_eq!(layout.children()[0].rect().area, area::logical(80.0, 60.0));
}

#[test]
fn horizontal_layout_distributes_fill_width() {
    let root = Node::container(ROOT, layout::Axis::Horizontal)
        .with_child(Node::leaf(A))
        .with_child(Node::leaf(B));

    let mut tree = Tree::new();
    tree.set_root(root);
    let layout = layout(&tree);

    assert_eq!(layout.children()[0].rect().area, area::logical(50.0, 80.0));
    assert_eq!(
        layout.children()[1].rect().origin,
        point::logical(50.0, 0.0)
    );
}

#[test]
fn fit_parent_sizes_to_children_padding_and_gap() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_size(layout::Size::Fit, layout::Size::Fit)
        .with_padding(layout::Insets::splat(5.0))
        .with_gap(4.0)
        .with_child(Node::leaf(A).with_size(layout::Size::Fixed(30.0), layout::Size::Fixed(10.0)))
        .with_child(Node::leaf(B).with_size(layout::Size::Fixed(20.0), layout::Size::Fixed(12.0)));
    let mut tree = Tree::new();

    tree.set_root(root);
    let layout = layout(&tree);

    assert_eq!(layout.rect().area, area::logical(40.0, 36.0));
    assert_eq!(layout.children()[0].rect().origin, point::logical(5.0, 5.0));
    assert_eq!(
        layout.children()[1].rect().origin,
        point::logical(5.0, 19.0)
    );
}

#[test]
fn gap_reduces_remaining_fill_space_without_outer_offsets() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_gap(5.0)
        .with_child(Node::leaf(A).with_size(layout::Size::Fill, layout::Size::Fixed(20.0)))
        .with_child(Node::leaf(B));
    let mut tree = Tree::new();

    tree.set_root(root);
    let layout = layout(&tree);

    assert_eq!(layout.children()[0].rect().origin, point::logical(0.0, 0.0));
    assert_eq!(layout.children()[0].rect().area, area::logical(100.0, 20.0));
    assert_eq!(
        layout.children()[1].rect().origin,
        point::logical(0.0, 25.0)
    );
    assert_eq!(layout.children()[1].rect().area, area::logical(100.0, 55.0));
}

#[test]
fn fixed_sizes_clamp_to_non_negative_and_available_space() {
    let root = Node::container(ROOT, layout::Axis::Horizontal)
        .with_child(Node::leaf(A).with_size(layout::Size::Fixed(-10.0), layout::Size::Fixed(-4.0)))
        .with_child(
            Node::leaf(B).with_size(layout::Size::Fixed(200.0), layout::Size::Fixed(120.0)),
        );
    let mut tree = Tree::new();

    tree.set_root(root);
    let layout = layout(&tree);

    assert_eq!(layout.children()[0].rect().area, area::logical(0.0, 0.0));
    assert_eq!(layout.children()[1].rect().area, area::logical(100.0, 80.0));
}

#[test]
fn main_axis_alignment_offsets_stack_children() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_align(layout::Align::Center)
        .with_child(Node::leaf(A).with_size(layout::Size::Fill, layout::Size::Fixed(20.0)));
    let mut tree = Tree::new();

    tree.set_root(root);
    let layout = layout(&tree);

    assert_eq!(
        layout.children()[0].rect().origin,
        point::logical(0.0, 30.0)
    );
}

#[test]
fn cross_axis_alignment_positions_and_stretches_children() {
    let centered = Node::container(ROOT, layout::Axis::Vertical)
        .with_cross_align(layout::Align::Center)
        .with_child(Node::leaf(A).with_size(layout::Size::Fixed(20.0), layout::Size::Fixed(10.0)));
    let ended = Node::container(ROOT, layout::Axis::Vertical)
        .with_cross_align(layout::Align::End)
        .with_child(Node::leaf(A).with_size(layout::Size::Fixed(20.0), layout::Size::Fixed(10.0)));
    let stretched = Node::container(ROOT, layout::Axis::Vertical)
        .with_cross_align(layout::Align::Stretch)
        .with_child(Node::leaf(A).with_size(layout::Size::Fit, layout::Size::Fixed(10.0)));
    let mut tree = Tree::new();

    tree.set_root(centered);
    let centered_layout = layout(&tree);
    assert_eq!(
        centered_layout.children()[0].rect().origin,
        point::logical(40.0, 0.0)
    );
    assert_eq!(
        centered_layout.children()[0].rect().area,
        area::logical(20.0, 10.0)
    );

    tree.set_root(ended);
    let ended_layout = layout(&tree);
    assert_eq!(
        ended_layout.children()[0].rect().origin,
        point::logical(80.0, 0.0)
    );

    tree.set_root(stretched);
    let stretched_layout = layout(&tree);
    assert_eq!(
        stretched_layout.children()[0].rect().area,
        area::logical(100.0, 10.0)
    );
}

#[test]
fn text_and_icon_nodes_provide_fit_content_measurements() {
    let label = Node::leaf(A)
        .with_size(layout::Size::Fit, layout::Size::Fit)
        .with_label(text::Document::plain("Hi"));
    let icon = Node::leaf(B)
        .with_size(layout::Size::Fit, layout::Size::Fit)
        .with_icon(check_icon())
        .with_icon_size(18.0);
    let mut tree = Tree::new();

    tree.set_root(label);
    let label_layout = layout(&tree);
    assert!(label_layout.rect().area.width() > 0.0);
    assert!(label_layout.rect().area.height() > 0.0);

    tree.set_root(icon);
    let icon_layout = layout(&tree);
    assert_eq!(icon_layout.rect().area, area::logical(18.0, 18.0));
}

#[test]
fn menu_bar_layout_remains_compact() {
    let theme = theme::Theme::default_dark();
    let bar = menu::Bar::new().menu(menu::Menu::new(menu::Id::new("file"), "File"));
    let mut tree = Tree::new();

    tree.set_root(widget::menu_bar(ROOT, bar));
    let layout = layout(&tree);

    assert_eq!(
        layout.rect().area.height(),
        theme.density().menu_bar_height()
    );
    assert!(layout.children()[0].rect().area.width() > 0.0);
}

#[test]
fn scroll_offset_shifts_child_layout_and_paint_positions() {
    let root = widget::scroll_view(ROOT)
        .with_scroll_offset(point::logical(0.0, 12.0))
        .with_child(
            Node::leaf(A)
                .with_background(paint::Color::RED)
                .with_size(layout::Size::Fill, layout::Size::Fixed(70.0)),
        )
        .with_child(Node::leaf(B).with_size(layout::Size::Fill, layout::Size::Fixed(70.0)));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window::Id::new(1),
        Interaction::default(),
        &mut scene,
    );

    assert_eq!(
        layout.children()[0].rect().origin,
        point::logical(0.0, -12.0)
    );
    assert_eq!(quad(&scene, 1).rect.origin, point::logical(0.0, -12.0));
}

#[test]
fn cursor_overlay_text_paints_after_tree_content_and_follows_pointer() {
    let root = Node::leaf(ROOT)
        .with_background(paint::Color::RED)
        .with_size(layout::Size::Fixed(100.0), layout::Size::Fixed(80.0));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let mut text_engine = text::Engine::new();

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint_with_text_engine(
        &layout,
        &registry,
        window::Id::new(1),
        Interaction::default()
            .with_pointer_position(Some(point::logical(20.0, 10.0)))
            .with_cursor_overlay(Some(CursorOverlay::text("drag"))),
        &HashMap::new(),
        &mut text_engine,
        &mut scene,
    );

    assert!(matches!(scene.items()[0], paint::Item::Quad(_)));
    let overlay = text(&scene, scene.items().len() - 1);
    assert_eq!(overlay.wrap, paint::TextWrap::None);
    assert_eq!(overlay.rect.origin, point::logical(32.0, 26.0));
    assert_eq!(overlay.document.blocks()[0].runs()[0].text(), "drag");
    assert_eq!(
        overlay.document.blocks()[0].runs()[0].style().size(),
        theme::Theme::default_dark().text().control_size()
    );
    assert!(
        (overlay.document.blocks()[0].runs()[0].style().color().a - 0.65).abs() <= f32::EPSILON
    );
}

#[test]
fn cursor_overlay_clamps_to_root_rect() {
    let root = Node::leaf(ROOT)
        .with_background(paint::Color::RED)
        .with_size(layout::Size::Fixed(100.0), layout::Size::Fixed(80.0));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let mut text_engine = text::Engine::new();

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint_with_text_engine(
        &layout,
        &registry,
        window::Id::new(1),
        Interaction::default()
            .with_pointer_position(Some(point::logical(98.0, 78.0)))
            .with_cursor_overlay(Some(CursorOverlay::text("drag"))),
        &HashMap::new(),
        &mut text_engine,
        &mut scene,
    );

    let overlay = text(&scene, scene.items().len() - 1);
    assert!(overlay.rect.origin.x() + overlay.rect.area.width() <= 100.0);
    assert!(overlay.rect.origin.y() + overlay.rect.area.height() <= 80.0);
}

#[test]
fn no_cursor_overlay_paints_no_extra_text() {
    let root = Node::leaf(ROOT)
        .with_background(paint::Color::RED)
        .with_size(layout::Size::Fixed(100.0), layout::Size::Fixed(80.0));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window::Id::new(1),
        Interaction::default().with_pointer_position(Some(point::logical(20.0, 10.0))),
        &mut scene,
    );

    assert!(
        !scene
            .items()
            .iter()
            .any(|item| matches!(item, paint::Item::Text(_)))
    );
}

#[test]
fn vertical_scrollbar_auto_reserves_right_gutter_only_with_overflow() {
    let root = widget::scroll_view(ROOT)
        .with_child(Node::leaf(A).with_size(layout::Size::Fill, layout::Size::Fixed(20.0)));
    let mut tree = Tree::new();

    tree.set_root(root);
    let layout = layout_area(&tree, area::logical(80.0, 60.0));

    assert_eq!(layout.children()[0].rect().area.width(), 80.0);

    let root = widget::scroll_view(ROOT)
        .with_child(Node::leaf(A).with_size(layout::Size::Fill, layout::Size::Fixed(40.0)))
        .with_child(Node::leaf(B).with_size(layout::Size::Fill, layout::Size::Fixed(40.0)));
    let mut tree = Tree::new();

    tree.set_root(root);
    let layout = layout_area(&tree, area::logical(80.0, 60.0));

    assert_eq!(layout.children()[0].rect().area.width(), 70.0);
}

#[test]
fn disabled_scrollbar_axis_reserves_no_gutter() {
    let root = widget::scroll_view(ROOT)
        .with_scroll_bars(widget::scroll::Bars::none())
        .with_child(Node::leaf(A).with_size(layout::Size::Fill, layout::Size::Fixed(20.0)));
    let mut tree = Tree::new();

    tree.set_root(root);
    let layout = layout_area(&tree, area::logical(80.0, 60.0));

    assert_eq!(layout.children()[0].rect().area.width(), 80.0);
}

#[test]
fn horizontal_scrollbar_reserves_bottom_gutter() {
    let root = widget::scroll_view(ROOT)
        .with_scroll_bars(widget::scroll::Bars::horizontal())
        .with_child(
            Node::container(A, layout::Axis::Horizontal)
                .with_size(layout::Size::Fill, layout::Size::Fill)
                .with_child(Node::leaf(B).with_size(layout::Size::Fixed(50.0), layout::Size::Fill))
                .with_child(Node::leaf(C).with_size(layout::Size::Fixed(50.0), layout::Size::Fill)),
        );
    let mut tree = Tree::new();

    tree.set_root(root);
    let layout = layout_area(&tree, area::logical(80.0, 60.0));

    assert_eq!(layout.children()[0].rect().area.height(), 50.0);
}

#[test]
fn both_scrollbar_axes_leave_corner_cell_and_trim_tracks() {
    let root = widget::scroll_view(ROOT)
        .with_scroll_bars(widget::scroll::Bars::both())
        .with_child(
            Node::container(A, layout::Axis::Vertical)
                .with_size(layout::Size::Fill, layout::Size::Fill)
                .with_child(
                    Node::container(B, layout::Axis::Horizontal)
                        .with_size(layout::Size::Fill, layout::Size::Fixed(40.0))
                        .with_child(
                            Node::leaf(D).with_size(layout::Size::Fixed(50.0), layout::Size::Fill),
                        )
                        .with_child(
                            Node::leaf(E).with_size(layout::Size::Fixed(50.0), layout::Size::Fill),
                        ),
                )
                .with_child(
                    Node::container(C, layout::Axis::Horizontal)
                        .with_size(layout::Size::Fill, layout::Size::Fixed(40.0))
                        .with_child(
                            Node::leaf(F).with_size(layout::Size::Fixed(50.0), layout::Size::Fill),
                        )
                        .with_child(
                            Node::leaf(G).with_size(layout::Size::Fixed(50.0), layout::Size::Fill),
                        ),
                ),
        );
    let mut tree = Tree::new();

    tree.set_root(root);
    let layout = layout_area(&tree, area::logical(80.0, 60.0));
    let widget_metrics = tree.widget_metrics(&layout);
    let metrics = widget_metrics
        .get(&path(ROOT))
        .and_then(|metrics| metrics.scroll())
        .expect("scroll metrics");

    assert_eq!(metrics.viewport().area, area::logical(70.0, 50.0));
    assert_eq!(
        metrics.vertical_track().map(|rect| rect.area),
        Some(area::logical(10.0, 50.0))
    );
    assert_eq!(
        metrics.horizontal_track().map(|rect| rect.area),
        Some(area::logical(70.0, 10.0))
    );
    assert_eq!(
        metrics.corner().map(|rect| (rect.origin, rect.area)),
        Some((point::logical(70.0, 50.0), area::logical(10.0, 10.0)))
    );
}

#[test]
fn scrollbar_thumb_size_and_position_derive_from_metrics() {
    let root = widget::scroll_view(ROOT)
        .with_scroll_offset(point::logical(0.0, 25.0))
        .with_child(Node::leaf(A).with_size(layout::Size::Fill, layout::Size::Fixed(30.0)))
        .with_child(Node::leaf(B).with_size(layout::Size::Fill, layout::Size::Fixed(30.0)))
        .with_child(Node::leaf(C).with_size(layout::Size::Fill, layout::Size::Fixed(30.0)));
    let mut tree = Tree::new();

    tree.set_root(root);
    let layout = layout_area(&tree, area::logical(40.0, 40.0));
    let widget_metrics = tree.widget_metrics(&layout);
    let metrics = widget_metrics
        .get(&path(ROOT))
        .and_then(|metrics| metrics.scroll())
        .expect("scroll metrics");
    let thumb = metrics.vertical_thumb().expect("vertical thumb");

    assert_eq!(metrics.max_offset(), point::logical(0.0, 50.0));
    assert_eq!(thumb.area.height(), 18.0);
    assert_eq!(thumb.origin.y(), 11.0);
}

#[test]
fn scroll_metrics_include_nested_descendant_overflow() {
    let wide_row = Node::container(B, layout::Axis::Horizontal)
        .with_size(layout::Size::Fill, layout::Size::Fixed(32.0))
        .with_child(Node::leaf(D).with_size(layout::Size::Fixed(80.0), layout::Size::Fill))
        .with_child(Node::leaf(E).with_size(layout::Size::Fixed(80.0), layout::Size::Fill));
    let middle_row = Node::container(C, layout::Axis::Horizontal)
        .with_size(layout::Size::Fill, layout::Size::Fixed(32.0))
        .with_child(Node::leaf(F).with_size(layout::Size::Fixed(80.0), layout::Size::Fill))
        .with_child(Node::leaf(G).with_size(layout::Size::Fixed(80.0), layout::Size::Fill));
    let content = Node::container(A, layout::Axis::Vertical)
        .with_size(layout::Size::Fill, layout::Size::Fill)
        .with_child(wide_row)
        .with_child(middle_row)
        .with_child(
            Node::leaf(Id::new("nested_tail"))
                .with_size(layout::Size::Fill, layout::Size::Fixed(32.0)),
        );
    let root = widget::scroll_view(ROOT)
        .with_scroll_bars(widget::scroll::Bars::both())
        .with_child(content);
    let mut tree = Tree::new();

    tree.set_root(root);
    let layout = layout_area(&tree, area::logical(100.0, 80.0));
    let widget_metrics = tree.widget_metrics(&layout);
    let metrics = widget_metrics
        .get(&path(ROOT))
        .and_then(|metrics| metrics.scroll())
        .expect("scroll metrics");

    assert!(metrics.content_size().width() > metrics.viewport().area.width());
    assert!(metrics.content_size().height() > metrics.viewport().area.height());
    assert!(metrics.max_offset().x() > 0.0);
    assert!(metrics.max_offset().y() > 0.0);
}

#[test]
fn scroll_view_paints_track_corner_and_thumb_chrome() {
    let root = widget::scroll_view(ROOT)
        .with_scroll_bars(widget::scroll::Bars::both())
        .with_background(paint::Color::BLACK)
        .with_child(
            Node::container(A, layout::Axis::Vertical)
                .with_size(layout::Size::Fill, layout::Size::Fill)
                .with_child(
                    Node::container(B, layout::Axis::Horizontal)
                        .with_size(layout::Size::Fill, layout::Size::Fixed(40.0))
                        .with_child(
                            Node::leaf(D).with_size(layout::Size::Fixed(50.0), layout::Size::Fill),
                        )
                        .with_child(
                            Node::leaf(E).with_size(layout::Size::Fixed(50.0), layout::Size::Fill),
                        ),
                )
                .with_child(
                    Node::container(C, layout::Axis::Horizontal)
                        .with_size(layout::Size::Fill, layout::Size::Fixed(40.0))
                        .with_child(
                            Node::leaf(F).with_size(layout::Size::Fixed(50.0), layout::Size::Fill),
                        )
                        .with_child(
                            Node::leaf(G).with_size(layout::Size::Fixed(50.0), layout::Size::Fill),
                        ),
                ),
        );
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();

    tree.set_root(root);
    let layout = layout_area(&tree, area::logical(80.0, 60.0));
    tree.paint(
        &layout,
        &registry,
        window::Id::new(1),
        Interaction::default(),
        &mut scene,
    );

    assert!(scene.items().iter().any(|item| {
        matches!(
            item,
            paint::Item::Quad(quad) if quad.rect.origin == point::logical(70.0, 0.0)
                && quad.rect.area == area::logical(10.0, 50.0)
        )
    }));
    assert!(scene.items().iter().any(|item| {
        matches!(
            item,
            paint::Item::Quad(quad) if quad.rect.origin == point::logical(70.0, 50.0)
                && quad.rect.area == area::logical(10.0, 10.0)
        )
    }));
}

#[test]
fn captured_scrollbar_thumb_stays_visually_pressed_off_thumb() {
    let root = widget::scroll_view(ROOT)
        .with_child(Node::leaf(A).with_size(layout::Size::Fill, layout::Size::Fixed(30.0)))
        .with_child(Node::leaf(B).with_size(layout::Size::Fill, layout::Size::Fixed(30.0)))
        .with_child(Node::leaf(C).with_size(layout::Size::Fill, layout::Size::Fixed(30.0)));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();

    tree.set_root(root);
    let layout = layout_area(&tree, area::logical(40.0, 40.0));
    let metrics = tree
        .widget_metrics(&layout)
        .get(&path(ROOT))
        .and_then(|metrics| metrics.scroll())
        .expect("scroll metrics");
    let thumb = metrics.vertical_thumb().expect("vertical thumb");
    let capture = pointer::Capture::new(
        path(ROOT),
        widget::Part::Scroll(widget::scroll::Part::VerticalThumb),
        pointer::Button::Primary,
        point::logical(35.0, 5.0),
        point::logical(0.0, 3.0),
    );

    tree.paint(
        &layout,
        &registry,
        window::Id::new(1),
        Interaction::default()
            .with_pointer_position(Some(point::logical(1.0, 1.0)))
            .with_pointer_capture(Some(capture)),
        &mut scene,
    );

    assert!(scene.items().iter().any(|item| {
        matches!(
            item,
            paint::Item::Tint(tint)
                if tint.rect == thumb && tint.brush == metrics.style().thumb_pressed_tint()
        )
    }));
}

#[test]
fn scrollbar_thumb_hover_tint_requires_pointer_hit() {
    let root = widget::scroll_view(ROOT)
        .with_child(Node::leaf(A).with_size(layout::Size::Fill, layout::Size::Fixed(30.0)))
        .with_child(Node::leaf(B).with_size(layout::Size::Fill, layout::Size::Fixed(30.0)))
        .with_child(Node::leaf(C).with_size(layout::Size::Fill, layout::Size::Fixed(30.0)));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();

    tree.set_root(root);
    let layout = layout_area(&tree, area::logical(40.0, 40.0));
    let metrics = tree
        .widget_metrics(&layout)
        .get(&path(ROOT))
        .and_then(|metrics| metrics.scroll())
        .expect("scroll metrics");
    let thumb = metrics.vertical_thumb().expect("vertical thumb");

    tree.paint(
        &layout,
        &registry,
        window::Id::new(1),
        Interaction::default().with_pointer_position(Some(point::logical(
            thumb.origin.x() + 1.0,
            thumb.origin.y() + 1.0,
        ))),
        &mut scene,
    );

    assert!(scene.items().iter().any(|item| {
        matches!(
            item,
            paint::Item::Tint(tint)
                if tint.rect == thumb && tint.brush == metrics.style().thumb_hover_tint()
        )
    }));
}

#[test]
fn scroll_view_clip_uses_viewport_minus_enabled_gutters() {
    let root = widget::scroll_view(ROOT)
        .with_scroll_bars(widget::scroll::Bars::both())
        .with_child(
            Node::container(A, layout::Axis::Vertical)
                .with_size(layout::Size::Fill, layout::Size::Fill)
                .with_child(
                    Node::container(B, layout::Axis::Horizontal)
                        .with_size(layout::Size::Fill, layout::Size::Fixed(40.0))
                        .with_child(
                            Node::leaf(D).with_size(layout::Size::Fixed(50.0), layout::Size::Fill),
                        )
                        .with_child(
                            Node::leaf(E).with_size(layout::Size::Fixed(50.0), layout::Size::Fill),
                        ),
                )
                .with_child(
                    Node::container(C, layout::Axis::Horizontal)
                        .with_size(layout::Size::Fill, layout::Size::Fixed(40.0))
                        .with_child(
                            Node::leaf(F).with_size(layout::Size::Fixed(50.0), layout::Size::Fill),
                        )
                        .with_child(
                            Node::leaf(G).with_size(layout::Size::Fixed(50.0), layout::Size::Fill),
                        ),
                ),
        );
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();

    tree.set_root(root);
    let layout = layout_area(&tree, area::logical(80.0, 60.0));
    tree.paint(
        &layout,
        &registry,
        window::Id::new(1),
        Interaction::default(),
        &mut scene,
    );

    assert!(scene.items().iter().any(|item| {
        matches!(
            item,
            paint::Item::Clip(clip) if clip.rect.origin == point::logical(0.0, 0.0)
                && clip.rect.area == area::logical(70.0, 50.0)
        )
    }));
}

#[test]
fn layout_assigns_stable_paths() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_child(Node::container(A, layout::Axis::Vertical).with_child(Node::leaf(B)));
    let mut tree = Tree::new();

    tree.set_root(root);
    let layout = layout(&tree);

    assert_eq!(layout.path(), &Path::new(vec![ROOT]));
    assert_eq!(layout.children()[0].path(), &Path::new(vec![ROOT, A]));
    assert_eq!(
        layout.children()[0].children()[0].path(),
        &Path::new(vec![ROOT, A, B])
    );
}

#[test]
fn popup_layout_is_topmost_for_hit_testing() {
    let mut tree = Tree::new();
    tree.set_root(
        Node::container(ROOT, layout::Axis::Vertical)
            .with_child(Node::leaf(A).with_interactivity(Interactivity::CONTROL)),
    );
    tree.push_popup(widget::Popup::new(
        Rect::new(point::logical(0.0, 0.0), area::logical(40.0, 40.0)),
        Node::leaf(B).with_interactivity(Interactivity::CONTROL),
    ));
    let layout = layout(&tree);

    assert_eq!(
        layout.hit_test_where(point::logical(10.0, 10.0), |_| true),
        Some(Path::new([ROOT, B]))
    );
}

#[test]
fn floating_panel_root_blocks_hits_without_taking_focus_or_action() {
    let mut tree = Tree::new();
    tree.set_root(Node::container(ROOT, layout::Axis::Vertical).with_child(
        widget::button(A, CLICK).with_size(layout::Size::Fixed(40.0), layout::Size::Fixed(40.0)),
    ));
    tree.push_popup(widget::Popup::new(
        Rect::new(point::logical(0.0, 0.0), area::logical(40.0, 40.0)),
        widget::floating_panel(B),
    ));
    let layout = layout(&tree);
    let interactivity = tree.interactivity();

    assert_eq!(
        layout.hit_test_where(point::logical(10.0, 10.0), |path| interactivity
            .get(path)
            .is_some_and(|interactivity| interactivity.hit_test())),
        Some(Path::new([ROOT, B]))
    );

    let popup_interactivity = interactivity
        .get(&Path::new([ROOT, B]))
        .expect("popup root interactivity");
    assert!(popup_interactivity.hit_test());
    assert!(!popup_interactivity.focusable());
    assert!(!popup_interactivity.actionable());
}

#[test]
fn tree_collects_popup_interactivity_with_root_prefixed_path() {
    let mut tree = Tree::new();
    tree.set_root(Node::leaf(ROOT));
    tree.push_popup(widget::Popup::new(
        Rect::new(point::logical(0.0, 0.0), area::logical(40.0, 40.0)),
        Node::leaf(B).with_interactivity(Interactivity::CONTROL),
    ));

    assert_eq!(
        tree.interactivity().get(&Path::new([ROOT, B])),
        Some(&Interactivity::CONTROL)
    );
}

#[test]
fn tree_collects_widget_scroll_metrics_with_root_prefixed_path() {
    let mut tree = Tree::new();
    tree.set_root(Node::leaf(ROOT));
    tree.push_popup(widget::Popup::new(
        Rect::new(point::logical(0.0, 0.0), area::logical(40.0, 40.0)),
        widget::scroll_view(B)
            .with_scroll_offset(point::logical(0.0, 12.0))
            .with_child(Node::leaf(A).with_size(layout::Size::Fill, layout::Size::Fixed(80.0))),
    ));
    let layout = layout_area(&tree, area::logical(100.0, 100.0));
    let widget_metrics = tree.widget_metrics(&layout);

    assert_eq!(
        widget_metrics
            .get(&Path::new([ROOT, B]))
            .and_then(|metrics| metrics.scroll())
            .map(|metrics| metrics.offset()),
        Some(point::logical(0.0, 12.0))
    );
    assert!(
        widget_metrics
            .get(&Path::new([ROOT, B]))
            .and_then(|metrics| metrics.scroll())
            .and_then(|metrics| metrics.vertical_track())
            .is_some()
    );
}

#[test]
fn duplicate_child_ids_under_different_parents_have_distinct_paths() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_child(Node::container(A, layout::Axis::Vertical).with_child(Node::leaf(C)))
        .with_child(Node::container(B, layout::Axis::Vertical).with_child(Node::leaf(C)));
    let mut tree = Tree::new();

    tree.set_root(root);
    let layout = layout(&tree);

    assert_ne!(
        layout.children()[0].children()[0].path(),
        layout.children()[1].children()[0].path()
    );
    assert_eq!(
        layout.children()[0].children()[0].path(),
        &Path::new(vec![ROOT, A, C])
    );
    assert_eq!(
        layout.children()[1].children()[0].path(),
        &Path::new(vec![ROOT, B, C])
    );
}

#[test]
fn deepest_hit_test_target_is_returned() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_child(
            Node::leaf(A)
                .with_size(layout::Size::Fill, layout::Size::Fixed(20.0))
                .hit_testable(true),
        )
        .with_child(Node::leaf(B).hit_testable(true));

    let mut tree = Tree::new();
    tree.set_root(root);
    let layout = layout(&tree);

    let interactivity = tree.interactivity();

    assert_eq!(
        layout.hit_test_where(point::logical(5.0, 25.0), |path| interactivity
            .get(path)
            .is_some_and(|interactivity| interactivity.hit_test())),
        Some(Path::new(vec![ROOT, B]))
    );
}

#[test]
fn passive_parent_does_not_become_hit_target() {
    let root = Node::container(ROOT, layout::Axis::Vertical).with_child(
        widget::button(A, CLICK).with_size(layout::Size::Fill, layout::Size::Fixed(20.0)),
    );
    let mut tree = Tree::new();

    tree.set_root(root);
    let layout = layout(&tree);
    let interactivity = tree.interactivity();

    assert_eq!(
        layout.hit_test_where(point::logical(90.0, 70.0), |path| interactivity
            .get(path)
            .is_some_and(|interactivity| interactivity.hit_test())),
        None
    );
    assert_eq!(
        layout.hit_test_where(point::logical(5.0, 5.0), |path| interactivity
            .get(path)
            .is_some_and(|interactivity| interactivity.hit_test())),
        Some(Path::new(vec![ROOT, A]))
    );
}

#[test]
fn clipped_parent_suppresses_overflow_child_hit_testing() {
    let root = widget::scroll_view(ROOT)
        .with_size(layout::Size::Fixed(100.0), layout::Size::Fixed(20.0))
        .with_child(
            widget::button(A, CLICK).with_size(layout::Size::Fill, layout::Size::Fixed(40.0)),
        );
    let mut tree = Tree::new();

    tree.set_root(root);
    let layout = layout(&tree);
    let interactivity = tree.interactivity();
    let accepts = |path: &Path| {
        interactivity
            .get(path)
            .is_some_and(|interactivity| interactivity.hit_test())
    };

    assert_eq!(
        layout.hit_test_where(point::logical(5.0, 10.0), accepts),
        Some(Path::new(vec![ROOT, A]))
    );
    assert_eq!(
        layout.hit_test_where(point::logical(5.0, 30.0), accepts),
        None
    );
}

#[test]
fn tree_renders_background_quads_in_layout_order() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_background(paint::Color::BLACK)
        .with_child(Node::leaf(A).with_background(paint::Color::RED));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window::Id::new(1),
        Interaction::default(),
        &mut scene,
    );

    assert_eq!(scene.items().len(), 2);
    assert_eq!(quad(&scene, 0).rect, layout.rect());
    assert_eq!(quad(&scene, 1).rect, layout.children()[0].rect());
}

#[test]
fn clipped_node_wraps_children_in_clip_commands() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .clipped()
        .with_background(paint::Color::BLACK)
        .with_child(Node::leaf(A).with_background(paint::Color::RED));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window::Id::new(1),
        Interaction::default(),
        &mut scene,
    );

    assert!(matches!(scene.items()[0], paint::Item::Quad(_)));
    assert_eq!(clip(&scene, 1).rect, layout.rect());
    assert!(matches!(scene.items()[2], paint::Item::Quad(_)));
    assert!(matches!(scene.items()[3], paint::Item::PopClip));
}

#[test]
fn labeled_button_stores_label_document() {
    let button = widget::labeled_button(A, CLICK, "Activate");

    let label = button.label().expect("button should have a label");
    assert_eq!(label.blocks()[0].align(), text::Align::Center);
    assert_eq!(label.blocks()[0].runs()[0].text(), "Activate");
}

#[test]
fn node_with_icon_stores_icon_data() {
    let node = Node::leaf(A).with_icon(check_icon()).with_icon_size(18.0);

    assert_eq!(node.icon(), Some(check_icon()));
    assert_eq!(node.icon_size(), Some(18.0));
}

#[test]
fn node_defaults_to_default_cursor_and_stores_cursor_intent() {
    assert_eq!(Node::leaf(A).cursor(), Cursor::Default);
    assert_eq!(
        Node::leaf(A).with_cursor(Cursor::Text).cursor(),
        Cursor::Text
    );
}

#[test]
fn node_with_backdrop_stores_backdrop_data() {
    let backdrop = Backdrop::new()
        .with_fill(paint::Color::rgba(0.1, 0.2, 0.3, 0.4))
        .with_blur(0.5);
    let node = Node::leaf(A).with_backdrop(backdrop);

    assert_eq!(node.style().backdrop(), Some(backdrop));
}

#[test]
fn icon_button_is_action_bound_control() {
    let button = widget::icon_button(A, CLICK, check_icon());

    assert_eq!(button.action(), Some(CLICK));
    assert_eq!(button.command_subject(), CommandSubject::Origin);
    assert_eq!(button.icon(), Some(check_icon()));
    assert!(button.interactivity().hit_test());
    assert!(button.interactivity().focusable());
    assert!(button.interactivity().actionable());
}

#[test]
fn default_control_builders_match_default_dark_theme_builders() {
    let theme = theme::Theme::default_dark();

    assert_eq!(widget::panel(A), widget::panel_with_theme(A, &theme));
    assert_eq!(
        widget::floating_panel(A),
        widget::floating_panel_with_theme(A, &theme)
    );
    assert_eq!(
        widget::button(A, CLICK),
        widget::button_with_theme(A, CLICK, &theme)
    );
    assert_eq!(
        widget::labeled_button(A, CLICK, "Activate"),
        widget::labeled_button_with_theme(A, CLICK, "Activate", &theme)
    );
    assert_eq!(
        widget::icon_button(A, CLICK, check_icon()),
        widget::icon_button_with_theme(A, CLICK, check_icon(), &theme)
    );
}

#[test]
fn default_button_uses_compact_theme_density() {
    let button = widget::button(A, CLICK);

    assert_eq!(
        button.layout().height(),
        layout::Size::Fixed(theme::Theme::default_dark().density().control_height())
    );
    assert_ne!(button.layout().height(), layout::Size::Fixed(160.0));
}

#[test]
fn floating_panel_uses_default_glass_material_tokens() {
    let theme = theme::Theme::default_dark();
    let floating = theme.floating_panel();
    let panel = widget::floating_panel(A);
    let style = panel.style();

    assert_eq!(
        style.backdrop(),
        Some(Backdrop::glass(floating.backdrop_fill()).with_blur(floating.backdrop_blur()))
    );
    assert_eq!(style.stroke(), Some(floating.stroke()));
    let shadow = style.shadow().expect("floating panel has shadow");
    assert_eq!(shadow.brush(), floating.shadow().brush());
    assert_eq!(shadow.blur(), floating.shadow().blur());
    assert_eq!(shadow.spread(), floating.shadow().spread());
    assert_eq!(shadow.offset(), floating.shadow().offset());
    assert_eq!(style.rounding(), floating.rounding());
    assert_eq!(style.padding(), layout::Insets::splat(floating.padding()));
    assert!(panel.interactivity().hit_test());
    assert!(!panel.interactivity().focusable());
    assert!(!panel.interactivity().actionable());
}

#[test]
fn node_with_command_subject_stores_policy() {
    let node = Node::leaf(A)
        .with_action(CLICK)
        .with_command_subject(CommandSubject::Current);

    assert_eq!(node.command_subject(), CommandSubject::Current);
}

#[test]
fn node_with_intent_stores_open_menu_intent() {
    let file = menu::Id::new("file");
    let node = Node::leaf(A).with_intent(Intent::OpenMenu(file));

    assert_eq!(node.intent(), Some(Intent::OpenMenu(file)));
    assert_eq!(node.action(), None);
}

#[test]
fn menu_bar_widget_creates_interactive_menu_title_intents() {
    let file = menu::Id::new("file");
    let edit = menu::Id::new("edit");
    let bar = menu::Bar::new()
        .menu(menu::Menu::new(file, "File"))
        .menu(menu::Menu::new(edit, "Edit"));
    let node = widget::menu_bar(A, bar.clone());

    assert_eq!(node.menu_bar(), Some(&bar));
    assert_eq!(node.children().len(), 2);
    assert_eq!(node.children()[0].intent(), Some(Intent::OpenMenu(file)));
    assert_eq!(node.children()[1].intent(), Some(Intent::OpenMenu(edit)));
    assert!(node.children()[0].interactivity().hit_test());
    assert!(node.children()[0].interactivity().focusable());
    assert!(node.children()[0].interactivity().actionable());
}

#[test]
fn default_widget_builders_match_default_dark_theme_builders() {
    let theme = theme::Theme::default_dark();
    let bar = menu::Bar::new().menu(menu::Menu::new(menu::Id::new("file"), "File"));

    assert_eq!(
        widget::label(A, "Status"),
        widget::label_with_theme(A, "Status", &theme)
    );
    assert_eq!(
        widget::separator(A),
        widget::separator_with_theme(A, &theme)
    );
    assert_eq!(
        widget::menu_bar(A, bar.clone()),
        widget::menu_bar_with_theme(A, bar, &theme)
    );
}

#[test]
fn tree_collects_intents_and_menu_definitions() {
    let file = menu::Id::new("file");
    let bar = menu::Bar::new().menu(menu::Menu::new(file, "File"));
    let mut tree = Tree::new();

    tree.set_root(
        Node::container(ROOT, layout::Axis::Vertical).with_child(widget::menu_bar(A, bar)),
    );

    assert_eq!(
        tree.intents().get(&Path::new([ROOT, A, Id::new("file")])),
        Some(&Intent::OpenMenu(file))
    );
    assert_eq!(tree.menus().get(&file).map(menu::Menu::label), Some("File"));
}

#[test]
fn node_with_responder_stores_handled_action() {
    let node = Node::leaf(A).with_responder(action::SELECT_ALL);

    assert_eq!(node.responders(), &[action::SELECT_ALL]);
    assert_eq!(
        node.responder_bindings(),
        &[action::Binding::new(action::SELECT_ALL)]
    );
}

#[test]
fn node_with_responder_binding_stores_projected_state() {
    let binding = action::Binding::new(action::SELECT_ALL)
        .enabled(false)
        .busy(true);
    let node = Node::leaf(A).with_responder_binding(binding);

    assert_eq!(node.responders(), &[action::SELECT_ALL]);
    assert_eq!(node.responder_bindings(), &[binding]);
}

#[test]
fn tree_collects_command_subject_policies() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_child(widget::button(A, CLICK).with_command_subject(CommandSubject::Current));
    let mut tree = Tree::new();

    tree.set_root(root);

    assert_eq!(
        tree.command_subjects().get(&Path::new([ROOT, A])),
        Some(&CommandSubject::Current)
    );
}

#[test]
fn composition_indexes_node_cursors_by_path() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_child(Node::leaf(A).with_cursor(Cursor::Text))
        .with_child(Node::leaf(B));
    let mut tree = Tree::new();
    let mut registry = action::Registry::<()>::new();
    let mut measurer = text::Engine::new();
    let window = window::Id::new(1);

    tree.set_root(root);
    let composition = tree
        .compose(
            window,
            area::logical(100.0, 80.0),
            &mut registry,
            &[],
            &mut measurer,
        )
        .expect("tree should compose");

    assert_eq!(composition.cursor(&Path::new([ROOT, A])), Cursor::Text);
    assert_eq!(composition.cursor(&Path::new([ROOT, B])), Cursor::Default);
}

#[test]
fn tree_collects_responder_actions_by_path() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_child(Node::leaf(A).with_responder(action::SELECT_ALL));
    let mut tree = Tree::new();

    tree.set_root(root);

    assert_eq!(
        tree.responders().get(&Path::new([ROOT, A])),
        Some(&vec![action::SELECT_ALL])
    );
}

#[test]
fn tree_collects_responder_bindings_by_path() {
    let binding = action::Binding::new(action::SELECT_ALL).enabled(false);
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_child(Node::leaf(A).with_responder_binding(binding));
    let mut tree = Tree::new();

    tree.set_root(root);

    assert_eq!(
        tree.responder_bindings().get(&Path::new([ROOT, A])),
        Some(&vec![binding])
    );
}

#[test]
fn node_with_command_scope_marks_scope_boundary() {
    let node = Node::leaf(A).with_command_scope();

    assert!(node.is_command_scope());
}

#[test]
fn tree_collects_command_scope_paths() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_child(Node::leaf(A).with_command_scope());
    let mut tree = Tree::new();

    tree.set_root(root);

    assert_eq!(tree.command_scopes(), vec![Path::new([ROOT, A])]);
}

#[test]
fn node_rounding_is_emitted_on_paint_quad() {
    let root = Node::leaf(A)
        .with_background(paint::Color::RED)
        .with_rounding(rect::Rounding::relative(1.0));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(None, None, None),
        &mut scene,
    );

    assert_eq!(quad(&scene, 0).rect.rounding, rect::Rounding::relative(1.0));
}

#[test]
fn tree_paint_emits_label_after_node_background() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_background(paint::Color::BLACK)
        .with_child(widget::labeled_button(A, CLICK, "Activate"));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default(),
        &mut scene,
    );

    assert_eq!(scene.items().len(), 3);
    assert_eq!(quad(&scene, 0).rect, layout.rect());
    assert_same_bounds(quad(&scene, 1).rect, layout.children()[0].rect());
    assert_eq!(text(&scene, 2).rect, layout.children()[0].rect());
    assert_eq!(
        text(&scene, 2).document.blocks()[0].runs()[0].text(),
        "Activate"
    );
}

#[test]
fn text_field_paint_clips_content_and_offsets_scrolled_text() {
    let root = widget::text_field(A, text::Buffer::from_text("hello world"))
        .with_size(layout::Size::Fixed(100.0), layout::Size::Fixed(30.0));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let window = window::Id::new(1);
    let mut text_engine = text::Engine::new();
    let states = HashMap::from([(path(A), text::TextFieldState::new(12.0))]);

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint_with_text_engine(
        &layout,
        &registry,
        window,
        Interaction::new(None, Some(path(A)), None),
        &states,
        &mut text_engine,
        &mut scene,
    );

    let content = {
        let rect = layout.rect();
        let padding = tree.root().unwrap().style().padding();
        Rect::new(
            point::logical(rect.origin.x() + padding.left, rect.origin.y()),
            area::logical(
                rect.area.width() - padding.left - padding.right,
                rect.area.height(),
            ),
        )
    };

    assert_eq!(clip(&scene, 1).rect, content);
    let text = scene
        .items()
        .iter()
        .find_map(|item| match item {
            paint::Item::Text(text) => Some(text),
            _ => None,
        })
        .expect("text field should paint text");
    assert_eq!(text.wrap, paint::TextWrap::None);
    assert_eq!(text.rect.origin.x(), content.origin.x() - 12.0);
    assert_eq!(text.rect.area.width(), content.area.width() + 12.0);
}

#[test]
fn text_field_paint_emits_caret_during_visible_blink_phase() {
    let epoch = Instant::now();
    let root = widget::text_field(A, text::Buffer::from_text("hello"))
        .with_size(layout::Size::Fixed(100.0), layout::Size::Fixed(30.0));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let window = window::Id::new(1);
    let mut text_engine = text::Engine::new();
    let states = HashMap::from([(path(A), text::TextFieldState::new_at(0.0, epoch))]);

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint_with_text_engine_at(
        &layout,
        &registry,
        window,
        Interaction::new(None, Some(path(A)), None),
        &states,
        &mut text_engine,
        crate::animation::Frame::new(epoch, None),
        &mut scene,
    );

    assert_eq!(caret_quad_count(&scene), 1);
    let caret = scene
        .items()
        .iter()
        .find_map(|item| match item {
            paint::Item::Quad(quad) if (quad.rect.area.width() - 1.0).abs() <= f32::EPSILON => {
                Some(quad)
            }
            _ => None,
        })
        .expect("caret quad should be painted");
    assert_eq!(
        caret.rasterization,
        paint::Rasterization {
            snapping: paint::Snapping::FixedWidth { width_px: 2 },
            edge_mode: paint::EdgeMode::Hard,
        }
    );
}

#[test]
fn text_field_paint_omits_caret_during_hidden_blink_phase() {
    let epoch = Instant::now();
    let root = widget::text_field(A, text::Buffer::from_text("hello"))
        .with_size(layout::Size::Fixed(100.0), layout::Size::Fixed(30.0));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let window = window::Id::new(1);
    let mut text_engine = text::Engine::new();
    let states = HashMap::from([(path(A), text::TextFieldState::new_at(0.0, epoch))]);
    let hidden = epoch + Duration::from_millis(500);

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint_with_text_engine_at(
        &layout,
        &registry,
        window,
        Interaction::new(None, Some(path(A)), None),
        &states,
        &mut text_engine,
        crate::animation::Frame::new(hidden, None),
        &mut scene,
    );

    assert_eq!(caret_quad_count(&scene), 0);
}

#[test]
fn read_only_text_field_paint_omits_caret_even_when_focused() {
    let epoch = Instant::now();
    let root = widget::text_field(A, text::Field::new("hello").read_only())
        .with_size(layout::Size::Fixed(100.0), layout::Size::Fixed(30.0));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let window = window::Id::new(1);
    let mut text_engine = text::Engine::new();
    let states = HashMap::from([(path(A), text::TextFieldState::new_at(0.0, epoch))]);

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint_with_text_engine_at(
        &layout,
        &registry,
        window,
        Interaction::new(None, Some(path(A)), None),
        &states,
        &mut text_engine,
        crate::animation::Frame::new(epoch, None),
        &mut scene,
    );

    assert_eq!(caret_quad_count(&scene), 0);
}

#[test]
fn empty_text_field_paints_placeholder_until_preedit_starts() {
    let theme = theme::Theme::default_dark();
    let root =
        widget::text_field_with_theme(A, text::Field::new("").with_placeholder("Search"), &theme)
            .with_size(layout::Size::Fixed(100.0), layout::Size::Fixed(30.0));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let window = window::Id::new(1);
    let mut text_engine = text::Engine::new();
    let states = HashMap::new();

    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint_with_text_engine(
        &layout,
        &registry,
        window,
        Interaction::new(None, Some(path(A)), None),
        &states,
        &mut text_engine,
        &mut scene,
    );

    let placeholder = scene
        .items()
        .iter()
        .find_map(|item| match item {
            paint::Item::Text(text) => Some(text),
            _ => None,
        })
        .expect("placeholder text should paint");
    assert_eq!(placeholder.document.blocks()[0].runs()[0].text(), "Search");
    assert_eq!(
        placeholder.document.blocks()[0].runs()[0].style().color(),
        theme.text().disabled()
    );
    let placeholder_size = placeholder.document.blocks()[0].runs()[0].style().size();
    assert_eq!(placeholder_size, theme.text().control_size());

    let filled_root = widget::text_field_with_theme(
        A,
        text::Field::new("Search").with_placeholder("Search"),
        &theme,
    )
    .with_size(layout::Size::Fixed(100.0), layout::Size::Fixed(30.0));
    let mut filled_tree = Tree::new();
    let mut filled_scene = paint::Scene::new();
    filled_tree.set_root(filled_root);
    let filled_layout = self::layout(&filled_tree);
    filled_tree.paint_with_text_engine(
        &filled_layout,
        &registry,
        window,
        Interaction::new(None, Some(path(A)), None),
        &states,
        &mut text_engine,
        &mut filled_scene,
    );
    let content = filled_scene
        .items()
        .iter()
        .find_map(|item| match item {
            paint::Item::Text(text) => Some(text),
            _ => None,
        })
        .expect("content text should paint");
    assert_eq!(
        content.document.blocks()[0].runs()[0].style().size(),
        placeholder_size
    );

    let mut preedit_scene = paint::Scene::new();
    let preedit_states = HashMap::from([(
        path(A),
        text::TextFieldState::default().with_preedit(Some(text::Preedit::new("s", Some((0, 1))))),
    )]);
    tree.paint_with_text_engine(
        &layout,
        &registry,
        window,
        Interaction::new(None, Some(path(A)), None),
        &preedit_states,
        &mut text_engine,
        &mut preedit_scene,
    );

    let preedit_content = preedit_scene
        .items()
        .iter()
        .find_map(|item| match item {
            paint::Item::Text(text) => Some(text),
            _ => None,
        })
        .expect("preedit text should paint");
    assert_eq!(preedit_content.document.blocks()[0].runs()[0].text(), "s");
}

#[test]
fn obscured_text_field_paints_dot_glyphs_instead_of_source_text() {
    let root = widget::text_field(A, text::Field::new("secret").obscured_dot())
        .with_size(layout::Size::Fixed(100.0), layout::Size::Fixed(30.0));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let window = window::Id::new(1);
    let mut text_engine = text::Engine::new();

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint_with_text_engine(
        &layout,
        &registry,
        window,
        Interaction::new(None, Some(path(A)), None),
        &HashMap::new(),
        &mut text_engine,
        &mut scene,
    );

    let painted = scene
        .items()
        .iter()
        .find_map(|item| match item {
            paint::Item::Text(text) => Some(text),
            _ => None,
        })
        .expect("obscured text should paint");
    assert_eq!(painted.document.blocks()[0].runs()[0].text(), "••••••");
}

#[test]
fn text_field_paint_omits_selection_without_text_editing_target() {
    let mut buffer = text::Buffer::from_text("hello");
    let mut engine = text::Engine::new();
    engine.apply_text_edit(&mut buffer, text::Edit::SelectAll);
    let root = widget::text_field(A, buffer)
        .with_size(layout::Size::Fixed(100.0), layout::Size::Fixed(30.0));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let window = window::Id::new(1);
    let states = HashMap::new();

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint_with_text_engine(
        &layout,
        &registry,
        window,
        Interaction::new(None, None, None),
        &states,
        &mut engine,
        &mut scene,
    );

    assert_eq!(selection_quad_count(&scene), 0);
}

#[test]
fn text_field_paint_emits_selection_for_text_editing_target() {
    let mut buffer = text::Buffer::from_text("hello");
    let mut engine = text::Engine::new();
    engine.apply_text_edit(&mut buffer, text::Edit::SelectAll);
    let root = widget::text_field(A, buffer)
        .with_size(layout::Size::Fixed(100.0), layout::Size::Fixed(30.0));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let window = window::Id::new(1);
    let states = HashMap::new();

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint_with_text_engine(
        &layout,
        &registry,
        window,
        Interaction::new(None, None, None).with_text_editing_target(Some(path(A))),
        &states,
        &mut engine,
        &mut scene,
    );

    assert_eq!(selection_quad_count(&scene), 1);
}

#[test]
fn later_sibling_quad_renders_after_button_label() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_child(widget::labeled_button(A, CLICK, "Activate"))
        .with_child(Node::leaf(B).with_background(paint::Color::RED));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default(),
        &mut scene,
    );

    assert_eq!(scene.items().len(), 3);
    assert_same_bounds(quad(&scene, 0).rect, layout.children()[0].rect());
    assert_eq!(text(&scene, 1).rect, layout.children()[0].rect());
    assert_eq!(quad(&scene, 2).rect, layout.children()[1].rect());
}

#[test]
fn disabled_action_node_renders_disabled_background() {
    let root = Node::leaf(A)
        .with_action(CLICK)
        .with_background(paint::Color::RED)
        .with_disabled_background(paint::Color::BLACK);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_state(
        CLICK,
        action::Context::path(window, path(A)),
        action::State::disabled(),
    );
    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default(),
        &mut scene,
    );

    assert_eq!(
        quad(&scene, 0).style.fill,
        Some(paint::Fill::Brush(paint::Brush::solid(paint::Color::BLACK)))
    );
}

#[test]
fn shape_chrome_builders_accept_gradient_brushes() {
    let brush = gradient_brush();
    let node = Node::leaf(A)
        .with_background(brush)
        .with_stroke(paint::Stroke { brush, width: 1.0 })
        .with_hover_tint(brush)
        .with_pressed_tint(brush)
        .with_active_tint(brush)
        .with_busy_tint(brush)
        .with_disabled_tint(brush)
        .with_focus_outline(brush, 2.0, 1.0)
        .with_shadow(brush, 12.0, 1.0, point::logical(0.0, 4.0))
        .with_backdrop(Backdrop::new().with_fill(brush));

    assert_eq!(node.style().background(), Some(brush));
    assert_eq!(node.style().stroke().expect("stroke").brush, brush);
    assert_eq!(node.style().hover_tint(), Some(brush));
    assert_eq!(node.style().pressed_tint(), Some(brush));
    assert_eq!(node.style().active_tint(), Some(brush));
    assert_eq!(node.style().busy_tint(), Some(brush));
    assert_eq!(node.style().disabled_tint(), Some(brush));
    assert_eq!(
        node.style().focus_outline().expect("outline").brush(),
        brush
    );
    assert_eq!(node.style().shadow().expect("shadow").brush(), brush);
    assert_eq!(
        node.style().backdrop().expect("backdrop").fill(),
        Some(brush)
    );
}

#[test]
fn ui_lowering_preserves_gradient_shape_chrome_order() {
    let background = gradient_brush();
    let stroke = paint::Brush::linear_gradient(
        paint::Color::rgba(1.0, 1.0, 1.0, 0.1),
        paint::Color::rgba(1.0, 1.0, 1.0, 0.4),
    );
    let tint_brush = paint::Brush::linear_gradient(
        paint::Color::rgba(1.0, 0.8, 0.2, 0.15),
        paint::Color::rgba(0.2, 0.8, 1.0, 0.2),
    );
    let outline_brush = paint::Brush::linear_gradient(
        paint::Color::rgba(0.2, 0.5, 1.0, 1.0),
        paint::Color::rgba(0.9, 0.3, 0.8, 1.0),
    );
    let shadow_brush = paint::Brush::linear_gradient(
        paint::Color::rgba(0.0, 0.0, 0.0, 0.2),
        paint::Color::rgba(0.0, 0.0, 0.0, 0.45),
    );
    let root = Node::leaf(A)
        .with_background(background)
        .with_stroke(paint::Stroke {
            brush: stroke,
            width: 1.0,
        })
        .with_hover_tint(tint_brush)
        .with_focus_outline(outline_brush, 2.0, 1.0)
        .with_shadow(shadow_brush, 12.0, 1.0, point::logical(0.0, 4.0));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(Some(path(A)), Some(path(A)), None),
        &mut scene,
    );

    assert!(matches!(scene.items()[0], paint::Item::Shadow(_)));
    assert!(matches!(scene.items()[1], paint::Item::Quad(_)));
    assert!(matches!(scene.items()[2], paint::Item::Tint(_)));
    assert!(matches!(scene.items()[3], paint::Item::Outline(_)));
    assert_eq!(shadow(&scene, 0).brush, shadow_brush);
    assert_eq!(
        quad(&scene, 1).style.fill,
        Some(paint::Fill::Brush(background))
    );
    assert_eq!(quad(&scene, 1).style.stroke.expect("stroke").brush, stroke);
    assert_eq!(tint(&scene, 2).brush, tint_brush);
    assert_eq!(outline(&scene, 3).brush, outline_brush);
}

#[test]
fn disabled_button_uses_disabled_label_color() {
    let root = widget::labeled_button(A, CLICK, "Disabled");
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_state(
        CLICK,
        action::Context::path(window, path(A)),
        action::State::disabled(),
    );
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default(),
        &mut scene,
    );

    assert_eq!(
        text(&scene, 2).document.blocks()[0].runs()[0]
            .style()
            .color(),
        root.style()
            .disabled_label_color()
            .expect("control has disabled label color")
    );
    assert_eq!(
        tint(&scene, 1).brush,
        root.style()
            .disabled_tint()
            .expect("control has disabled tint")
    );
}

#[test]
fn disabled_icon_button_uses_disabled_label_color() {
    let root = widget::icon_button(A, CLICK, check_icon());
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_state(
        CLICK,
        action::Context::path(window, path(A)),
        action::State::disabled(),
    );
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default(),
        &mut scene,
    );

    assert_eq!(
        icon_item(&scene, 2).color,
        root.style()
            .disabled_label_color()
            .expect("control has disabled label color")
    );
}

#[test]
fn control_hover_state_emits_hover_tint_over_base_background() {
    let root = widget::button(A, CLICK);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(Some(path(A)), None, None),
        &mut scene,
    );

    assert_eq!(
        quad(&scene, 0).style.fill,
        Some(paint::Fill::Brush(
            root.style().background().expect("control has base color")
        ))
    );
    assert_eq!(
        tint(&scene, 1).brush,
        root.style().hover_tint().expect("control has hover tint")
    );
    assert_same_bounds(tint(&scene, 1).rect, layout.rect());
    assert_eq!(tint(&scene, 1).rect.rounding, root.style().rounding());
}

#[test]
fn control_focus_state_emits_outline_without_changing_fill() {
    let root = widget::button(A, CLICK);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(Some(path(B)), Some(path(A)), None),
        &mut scene,
    );

    assert_eq!(
        quad(&scene, 0).style.fill,
        Some(paint::Fill::Brush(
            root.style().background().expect("control has base color")
        ))
    );
    assert_same_bounds(outline(&scene, 1).rect, layout.rect());
    assert_eq!(outline(&scene, 1).rect.rounding, root.style().rounding());
}

#[test]
fn focus_background_renders_when_focus_visible() {
    let focus_background = paint::Color::rgb(0.2, 0.3, 0.4);
    let root = Node::leaf(A)
        .with_background(paint::Color::BLACK)
        .with_focus_background(focus_background);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(None, Some(path(A)), None)
            .with_focus_visibility(focus::Visibility::Visible),
        &mut scene,
    );

    assert_eq!(
        quad(&scene, 0).style.fill,
        Some(paint::Fill::Brush(paint::Brush::solid(focus_background)))
    );
}

#[test]
fn open_menu_title_emits_active_tint() {
    let file = menu::Id::new("file");
    let active_tint = paint::Color::rgba(0.1, 0.2, 0.3, 0.5);
    let root = Node::leaf(A)
        .with_background(paint::Color::BLACK)
        .with_active_tint(active_tint)
        .with_intent(Intent::OpenMenu(file));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default().with_open_menu(Some(file)),
        &mut scene,
    );

    assert_eq!(tint(&scene, 1).brush, paint::Brush::solid(active_tint));
}

#[test]
fn open_submenu_row_emits_active_tint() {
    let panels = menu::Id::new("panels");
    let active_tint = paint::Color::rgba(0.1, 0.2, 0.3, 0.5);
    let root = Node::leaf(A)
        .with_background(paint::Color::BLACK)
        .with_active_tint(active_tint)
        .with_intent(Intent::OpenSubmenu(panels));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default().with_open_submenu(Some(panels)),
        &mut scene,
    );

    assert_eq!(tint(&scene, 1).brush, paint::Brush::solid(active_tint));
}

#[test]
fn hidden_focus_does_not_emit_outline() {
    let root = widget::button(A, CLICK);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(None, Some(path(A)), None)
            .with_focus_visibility(focus::Visibility::Hidden),
        &mut scene,
    );

    assert!(matches!(scene.items(), [paint::Item::Quad(_)]));
}

#[test]
fn visible_focus_on_text_field_emits_outline() {
    let root = widget::text_field(A, text::Buffer::from_text("hello"));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(None, Some(path(A)), None)
            .with_focus_visibility(focus::Visibility::Visible),
        &mut scene,
    );

    assert!(
        scene
            .items()
            .iter()
            .any(|item| matches!(item, paint::Item::Outline(_)))
    );
}

#[test]
fn active_state_renders_independently_from_focus_visibility() {
    let root = widget::button(A, CLICK);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_state(
        CLICK,
        action::Context::path(window, path(A)),
        action::State::active(),
    );
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(None, Some(path(A)), None)
            .with_focus_visibility(focus::Visibility::Hidden),
        &mut scene,
    );

    assert_eq!(
        tint(&scene, 1).brush,
        root.style().active_tint().expect("control has active tint")
    );
    assert_eq!(scene.items().len(), 2);
}

#[test]
fn command_subject_widget_visuals_derive_from_command_subject_state() {
    let root = widget::button(A, CLICK).with_command_subject(CommandSubject::Current);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_state(
        CLICK,
        action::Context::path(window, path(B)),
        action::State::active(),
    );
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default().with_command_subject(action::Context::path(window, path(B))),
        &mut scene,
    );

    assert_eq!(
        tint(&scene, 1).brush,
        root.style().active_tint().expect("control has active tint")
    );
}

#[test]
fn window_subject_widget_visuals_derive_from_window_state() {
    let root = widget::button(A, CLICK).with_command_subject(CommandSubject::Window);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_state(
        CLICK,
        action::Context::window(window),
        action::State::active(),
    );
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default(),
        &mut scene,
    );

    assert_eq!(
        tint(&scene, 1).brush,
        root.style().active_tint().expect("control has active tint")
    );
}

#[test]
fn captured_subject_widget_visuals_derive_from_scope_capture() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_command_scope()
        .with_child(widget::button(A, CLICK).with_command_subject(CommandSubject::Captured));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_state(
        CLICK,
        action::Context::path(window, path(B)),
        action::State::active(),
    );
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default().with_command_scope_captures(HashMap::from([(
            path(ROOT),
            action::Context::path(window, path(B)),
        )])),
        &mut scene,
    );

    assert_eq!(
        tint(&scene, 1).brush,
        root.children()[0]
            .style()
            .active_tint()
            .expect("control has active tint")
    );
}

#[test]
fn active_hovered_control_emits_active_then_hover_tint() {
    let root = widget::button(A, CLICK);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_state(
        CLICK,
        action::Context::path(window, path(A)),
        action::State::active(),
    );
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(Some(path(A)), Some(path(A)), None),
        &mut scene,
    );

    assert_eq!(
        quad(&scene, 0).style.fill,
        Some(paint::Fill::Brush(
            root.style().background().expect("control has base color")
        ))
    );
    assert_eq!(
        tint(&scene, 1).brush,
        root.style().active_tint().expect("control has active tint")
    );
    assert_eq!(
        tint(&scene, 2).brush,
        root.style().hover_tint().expect("control has hover tint")
    );
}

#[test]
fn active_pressed_control_emits_active_then_pressed_tint() {
    let root = widget::button(A, CLICK);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_state(
        CLICK,
        action::Context::path(window, path(A)),
        action::State::active(),
    );
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(Some(path(A)), None, Some(path(A))),
        &mut scene,
    );

    assert_eq!(
        tint(&scene, 1).brush,
        root.style().active_tint().expect("control has active tint")
    );
    assert_eq!(
        tint(&scene, 2).brush,
        root.style()
            .pressed_tint()
            .expect("control has pressed tint")
    );
    assert_eq!(scene.items().len(), 3);
}

#[test]
fn busy_control_emits_busy_tint_and_suppresses_hover_press() {
    let root = widget::button(A, CLICK);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_state(
        CLICK,
        action::Context::path(window, path(A)),
        action::State::active().with_busy(true),
    );
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(Some(path(A)), Some(path(A)), Some(path(A))),
        &mut scene,
    );

    assert_eq!(
        quad(&scene, 0).style.fill,
        Some(paint::Fill::Brush(
            root.style().background().expect("control has base color")
        ))
    );
    assert_eq!(
        tint(&scene, 1).brush,
        root.style().active_tint().expect("control has active tint")
    );
    assert_eq!(
        tint(&scene, 2).brush,
        root.style().busy_tint().expect("control has busy tint")
    );
    assert_same_bounds(outline(&scene, 3).rect, layout.rect());
    assert_eq!(outline(&scene, 3).rect.rounding, root.style().rounding());
    assert_eq!(scene.items().len(), 4);
}

#[test]
fn disabled_control_emits_disabled_tint_and_suppresses_hover_press() {
    let root = widget::button(A, CLICK);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_state(
        CLICK,
        action::Context::path(window, path(A)),
        action::State::disabled(),
    );
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(Some(path(A)), None, Some(path(A))),
        &mut scene,
    );

    assert_eq!(
        tint(&scene, 1).brush,
        root.style()
            .disabled_tint()
            .expect("control has disabled tint")
    );
    assert_eq!(scene.items().len(), 2);
}

#[test]
fn disabled_actionable_parent_paints_child_label_with_disabled_color() {
    let theme = theme::Theme::default_dark();
    let root = Node::container(A, layout::Axis::Horizontal)
        .with_action(CLICK)
        .with_child(
            Node::leaf(B)
                .with_label(text::Document::plain("Unavailable"))
                .with_label_color(theme.text().primary())
                .with_disabled_label_color(theme.text().disabled()),
        );
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_state(
        CLICK,
        action::Context::path(window, path(A)),
        action::State::disabled(),
    );
    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default(),
        &mut scene,
    );

    assert_eq!(
        text(&scene, 0).document.blocks()[0].runs()[0]
            .style()
            .color(),
        theme.text().disabled()
    );
}

#[test]
fn disabled_actionable_parent_paints_child_icon_with_disabled_color() {
    let theme = theme::Theme::default_dark();
    let root = Node::container(A, layout::Axis::Horizontal)
        .with_action(CLICK)
        .with_child(
            Node::leaf(B)
                .with_icon(check_icon())
                .with_label_color(theme.text().primary())
                .with_disabled_label_color(theme.text().disabled()),
        );
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_state(
        CLICK,
        action::Context::path(window, path(A)),
        action::State::disabled(),
    );
    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default(),
        &mut scene,
    );

    assert_eq!(icon_item(&scene, 0).color, theme.text().disabled());
}

#[test]
fn enabled_actionable_parent_paints_child_content_with_normal_color() {
    let theme = theme::Theme::default_dark();
    let root = Node::container(A, layout::Axis::Horizontal)
        .with_action(CLICK)
        .with_child(
            Node::leaf(B)
                .with_label(text::Document::plain("Available"))
                .with_label_color(theme.text().primary())
                .with_disabled_label_color(theme.text().disabled()),
        )
        .with_child(
            Node::leaf(C)
                .with_icon(check_icon())
                .with_label_color(theme.text().primary())
                .with_disabled_label_color(theme.text().disabled()),
        );
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default(),
        &mut scene,
    );

    assert_eq!(
        text(&scene, 0).document.blocks()[0].runs()[0]
            .style()
            .color(),
        theme.text().primary()
    );
    assert_eq!(icon_item(&scene, 1).color, theme.text().primary());
}

#[test]
fn child_action_uses_own_state_instead_of_inherited_disabled_state() {
    let theme = theme::Theme::default_dark();
    let root = Node::container(A, layout::Axis::Horizontal)
        .with_action(CLICK)
        .with_child(
            Node::leaf(B)
                .with_action(OTHER_CLICK)
                .with_label(text::Document::plain("Child"))
                .with_label_color(theme.text().primary())
                .with_disabled_label_color(theme.text().disabled()),
        );
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.register(action::Action::new(OTHER_CLICK, "Child"));
    registry.set_state(
        CLICK,
        action::Context::path(window, path(A)),
        action::State::disabled(),
    );
    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default(),
        &mut scene,
    );

    assert_eq!(
        text(&scene, 0).document.blocks()[0].runs()[0]
            .style()
            .color(),
        theme.text().primary()
    );
}

#[test]
fn busy_button_uses_busy_label_color() {
    let root = widget::labeled_button(A, CLICK, "Working");
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_busy(CLICK, action::Context::path(window, path(A)), true);
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default(),
        &mut scene,
    );

    assert_eq!(
        text(&scene, 2).document.blocks()[0].runs()[0]
            .style()
            .color(),
        root.style()
            .busy_label_color()
            .expect("control has busy label color")
    );
    assert_eq!(
        tint(&scene, 1).brush,
        root.style().busy_tint().expect("control has busy tint")
    );
}

#[test]
fn busy_icon_button_uses_busy_label_color() {
    let root = widget::icon_button(A, CLICK, check_icon());
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_busy(CLICK, action::Context::path(window, path(A)), true);
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default(),
        &mut scene,
    );

    assert_eq!(
        icon_item(&scene, 2).color,
        root.style()
            .busy_label_color()
            .expect("control has busy label color")
    );
}

#[test]
fn pressed_state_emits_pressed_tint_after_action_states() {
    let root = widget::button(A, CLICK);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(Some(path(A)), None, Some(path(A))),
        &mut scene,
    );

    assert_eq!(
        tint(&scene, 1).brush,
        root.style()
            .pressed_tint()
            .expect("control has pressed tint")
    );
}

#[test]
fn focused_node_emits_overlay_outline_after_tree_content() {
    let root = widget::panel(A).with_child(Node::leaf(B).with_background(paint::Color::RED));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(None, Some(path(A)), None),
        &mut scene,
    );

    assert_same_bounds(quad(&scene, 0).rect, layout.rect());
    assert_eq!(
        quad(&scene, 0).rect.rounding,
        theme::Theme::default_dark().roundings().panel()
    );
    assert_eq!(quad(&scene, 1).rect, layout.children()[0].rect());
    assert_same_bounds(outline(&scene, 2).rect, layout.rect());
    assert_eq!(
        outline(&scene, 2).rect.rounding,
        theme::Theme::default_dark().roundings().panel()
    );
}

#[test]
fn popup_shadow_renders_before_popup_panel_fill() {
    let popup_rect = Rect::new(point::logical(10.0, 10.0), area::logical(40.0, 40.0));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    tree.set_root(Node::leaf(ROOT).with_background(paint::Color::BLACK));
    tree.push_popup(widget::Popup::new(
        popup_rect,
        Node::leaf(B)
            .with_background(paint::Color::RED)
            .with_shadow(
                paint::Color::rgba(0.0, 0.0, 0.0, 0.35),
                18.0,
                1.0,
                point::logical(0.0, 6.0),
            ),
    ));
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default(),
        &mut scene,
    );

    assert!(matches!(scene.items()[0], paint::Item::Quad(_)));
    assert!(matches!(scene.items()[1], paint::Item::Shadow(_)));
    assert!(matches!(scene.items()[2], paint::Item::Quad(_)));
    assert_eq!(shadow(&scene, 1).rect, popup_rect);
    assert_eq!(quad(&scene, 2).rect, popup_rect);
}

#[test]
fn backdrop_lowers_before_node_background() {
    let root = Node::leaf(A)
        .with_backdrop(
            Backdrop::new()
                .with_fill(paint::Color::rgba(1.0, 1.0, 1.0, 0.5))
                .with_blur(0.5),
        )
        .with_rounding(rect::Rounding::relative(0.4));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default(),
        &mut scene,
    );

    assert!(matches!(scene.items()[0], paint::Item::Backdrop(_)));
    assert!(matches!(scene.items()[1], paint::Item::Quad(_)));
    assert_eq!(backdrop(&scene, 0).rect, quad(&scene, 1).rect);
    assert_eq!(backdrop(&scene, 0).rect.rounding, root.style().rounding());
    assert_eq!(
        backdrop(&scene, 0).filter,
        paint::BackdropFilter::Blur { amount: 0.5 }
    );
    assert_eq!(
        quad(&scene, 1).style.fill,
        Some(paint::Fill::Brush(paint::Brush::solid(paint::Color::rgba(
            1.0, 1.0, 1.0, 0.5
        ))))
    );
}

#[test]
fn popup_backdrop_lowers_after_shadow_before_popup_panel_fill() {
    let popup_rect = Rect::rounded(
        point::logical(10.0, 10.0),
        area::logical(40.0, 40.0),
        rect::Rounding::relative(0.5),
    );
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    tree.set_root(Node::leaf(ROOT).with_background(paint::Color::BLACK));
    tree.push_popup(widget::Popup::new(
        popup_rect,
        Node::leaf(B)
            .with_background(paint::Color::rgba(1.0, 1.0, 1.0, 0.35))
            .with_backdrop(Backdrop::new().with_blur(0.75))
            .with_rounding(rect::Rounding::relative(0.5))
            .with_shadow(
                paint::Color::rgba(0.0, 0.0, 0.0, 0.35),
                18.0,
                1.0,
                point::logical(0.0, 6.0),
            ),
    ));
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default(),
        &mut scene,
    );

    assert!(matches!(scene.items()[0], paint::Item::Quad(_)));
    assert!(matches!(scene.items()[1], paint::Item::Shadow(_)));
    assert!(matches!(scene.items()[2], paint::Item::Backdrop(_)));
    assert!(matches!(scene.items()[3], paint::Item::Quad(_)));
    assert_eq!(shadow(&scene, 1).rect, popup_rect);
    assert_eq!(backdrop(&scene, 2).rect, popup_rect);
    assert_eq!(quad(&scene, 3).rect, popup_rect);
}

#[test]
fn icon_paint_is_emitted_after_tints_before_focus_outline() {
    let root = widget::icon_button(A, CLICK, check_icon());
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_state(
        CLICK,
        action::Context::path(window, path(A)),
        action::State::active(),
    );
    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(None, Some(path(A)), None),
        &mut scene,
    );

    assert!(matches!(scene.items()[0], paint::Item::Quad(_)));
    assert!(matches!(scene.items()[1], paint::Item::Tint(_)));
    assert!(matches!(scene.items()[2], paint::Item::Icon(_)));
    assert!(matches!(scene.items()[3], paint::Item::Outline(_)));
}

#[test]
fn focused_first_button_outline_is_not_covered_by_second_button() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_child(widget::labeled_button(A, CLICK, "Active"))
        .with_child(Node::leaf(B).with_background(paint::Color::RED));
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_state(
        CLICK,
        action::Context::path(window, Path::new(vec![ROOT, A])),
        action::State::active(),
    );
    tree.set_root(root);
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(None, Some(Path::new(vec![ROOT, A])), None),
        &mut scene,
    );

    assert!(matches!(scene.items()[0], paint::Item::Quad(_)));
    assert!(matches!(scene.items()[1], paint::Item::Tint(_)));
    assert!(matches!(scene.items()[2], paint::Item::Text(_)));
    assert!(matches!(scene.items()[3], paint::Item::Quad(_)));
    assert!(matches!(scene.items()[4], paint::Item::Outline(_)));
    assert_same_bounds(outline(&scene, 4).rect, layout.children()[0].rect());
    assert_eq!(
        outline(&scene, 4).rect.rounding,
        theme::Theme::default_dark().roundings().control()
    );
}

#[test]
fn enabled_inactive_action_node_uses_base_background() {
    let root = widget::button(A, CLICK);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_state(
        CLICK,
        action::Context::path(window, path(A)),
        action::State::new(true, false),
    );
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::default(),
        &mut scene,
    );

    assert_eq!(
        quad(&scene, 0).style.fill,
        Some(paint::Fill::Brush(
            root.style().background().expect("control has base color")
        ))
    );
}
