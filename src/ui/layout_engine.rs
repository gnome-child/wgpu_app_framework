use std::collections::HashMap;

use crate::geometry::{Rect, area, point};
use crate::{layout, text, ui, widget};

pub fn tree(
    root: &ui::Node,
    area: area::Logical,
    measurer: &mut text::Engine,
) -> layout::Frame<ui::Path> {
    let mut nodes = HashMap::new();
    collect_nodes(root, &ui::Path::root(root.id()), &mut nodes);

    let item = project_node(root, ui::Path::root(root.id()));
    let mut adapter = UiMeasurer { nodes, measurer };
    let frame = layout::Engine::new().layout(&item, area, &mut adapter);

    apply_widget_layout(root, frame, &mut adapter)
}

pub fn subtree_at(
    root: &ui::Node,
    path: ui::Path,
    rect: Rect,
    measurer: &mut text::Engine,
) -> layout::Frame<ui::Path> {
    let mut nodes = HashMap::new();
    collect_nodes(root, &path, &mut nodes);

    let item = project_node(root, path.clone()).with_box_model(
        root.layout().box_model(root.style().padding()).with_size(
            layout::Size::Fixed(rect.area.width()),
            layout::Size::Fixed(rect.area.height()),
        ),
    );
    let mut adapter = UiMeasurer { nodes, measurer };
    let frame = layout::Engine::new().layout(&item, rect.area, &mut adapter);
    let frame = translate_frame(frame, rect.origin.x(), rect.origin.y());

    apply_widget_layout(root, frame, &mut adapter)
}

struct UiMeasurer<'a, 'm> {
    nodes: HashMap<ui::Path, &'a ui::Node>,
    measurer: &'m mut text::Engine,
}

impl layout::Measurer<ui::Path> for UiMeasurer<'_, '_> {
    fn measure(&mut self, key: &ui::Path, limits: layout::Limits) -> layout::Intrinsic {
        let Some(node) = self.nodes.get(key) else {
            return layout::Intrinsic::zero();
        };

        let max = limits.max();
        let label = node
            .label()
            .map(|document| {
                self.measurer
                    .measure(document, text::Measure::bounded(max))
                    .area()
            })
            .unwrap_or_else(|| area::logical(0.0, 0.0));
        let icon_size = node
            .icon()
            .map(|_| node.icon_size().unwrap_or(24.0).max(0.0));
        let icon = icon_size
            .map(|size| area::logical(size, size))
            .unwrap_or_else(|| area::logical(0.0, 0.0));

        layout::Intrinsic::fixed(area::logical(
            label.width().max(icon.width()),
            label.height().max(icon.height()),
        ))
    }
}

fn collect_nodes<'a>(
    node: &'a ui::Node,
    path: &ui::Path,
    nodes: &mut HashMap<ui::Path, &'a ui::Node>,
) {
    nodes.insert(path.clone(), node);

    for child in node.children() {
        collect_nodes(child, &path.child(child.id()), nodes);
    }
}

fn project_node(node: &ui::Node, path: ui::Path) -> layout::Item<ui::Path> {
    layout::Item::new(path.clone())
        .with_box_model(node.layout().box_model(node.style().padding()))
        .with_layout(node.layout().strategy())
        .with_children(
            node.children()
                .iter()
                .map(|child| project_node(child, path.child(child.id())))
                .collect(),
        )
}

fn project_children(node: &ui::Node, path: &ui::Path) -> Vec<layout::Item<ui::Path>> {
    node.children()
        .iter()
        .map(|child| project_node(child, path.child(child.id())))
        .collect()
}

fn apply_widget_layout(
    node: &ui::Node,
    frame: layout::Frame<ui::Path>,
    adapter: &mut UiMeasurer<'_, '_>,
) -> layout::Frame<ui::Path> {
    if node.scroll().is_some() {
        return apply_scroll_layout(node, frame, adapter);
    }

    let children = node
        .children()
        .iter()
        .zip(frame.children())
        .map(|(child, child_frame)| apply_widget_layout(child, child_frame.clone(), adapter))
        .collect();

    frame.with_children(children)
}

fn apply_scroll_layout(
    node: &ui::Node,
    frame: layout::Frame<ui::Path>,
    adapter: &mut UiMeasurer<'_, '_>,
) -> layout::Frame<ui::Path> {
    let viewport = widget::scroll::viewport_rect(node, frame.rect());
    let offset = node
        .scroll()
        .map_or_else(|| point::logical(0.0, 0.0), |scroll| scroll.offset());
    let path = frame.path().clone();
    let item = layout::Item::new(path.clone())
        .with_box_model(layout::BoxModel::new(
            layout::Size::Fixed(viewport.area.width()),
            layout::Size::Fixed(viewport.area.height()),
        ))
        .with_layout(node.layout().strategy())
        .with_children(project_children(node, &path));
    let inner = layout::Engine::new().layout(&item, viewport.area, adapter);
    let dx = viewport.origin.x() - offset.x();
    let dy = viewport.origin.y() - offset.y();
    let children = node
        .children()
        .iter()
        .zip(inner.children())
        .map(|(child, child_frame)| {
            apply_widget_layout(child, translate_frame(child_frame.clone(), dx, dy), adapter)
        })
        .collect();

    frame.with_children(children)
}

fn translate_frame(frame: layout::Frame<ui::Path>, dx: f32, dy: f32) -> layout::Frame<ui::Path> {
    let rect = translate_rect(frame.rect(), dx, dy);
    let content_rect = translate_rect(frame.content_rect(), dx, dy);
    let children = frame
        .children()
        .iter()
        .cloned()
        .map(|child| translate_frame(child, dx, dy))
        .collect();

    layout::Frame::with_content_rect(frame.path().clone(), rect, content_rect, children)
}

fn translate_rect(rect: Rect, dx: f32, dy: f32) -> Rect {
    Rect::rounded(
        point::logical(rect.origin.x() + dx, rect.origin.y() + dy),
        rect.area,
        rect.rounding,
    )
}
