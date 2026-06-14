use crate::geometry::{Rect, area, point};
use crate::{layout, ui};

pub fn tree(root: &ui::Node, area: area::Logical) -> layout::Box {
    node(
        root,
        ui::Path::root(root.id()),
        point::logical(0.0, 0.0),
        area,
    )
}

fn node(
    node: &ui::Node,
    path: ui::Path,
    origin: point::Logical,
    available: area::Logical,
) -> layout::Box {
    let node_layout = node.layout();
    let width = resolve_size(node_layout.width(), available.width());
    let height = resolve_size(node_layout.height(), available.height());
    let rect = Rect::new(origin, area::logical(width, height));
    let padding = node.style().padding();
    let content_origin = point::logical(origin.x() + padding.left, origin.y() + padding.top);
    let content_area = area::logical(
        (width - padding.horizontal()).max(0.0),
        (height - padding.vertical()).max(0.0),
    );
    let children = match node_layout.direction() {
        Some(layout::Axis::Vertical) => {
            vertical_children(node, &path, content_origin, content_area)
        }
        Some(layout::Axis::Horizontal) => {
            horizontal_children(node, &path, content_origin, content_area)
        }
        None => node
            .children()
            .iter()
            .map(|child| self::node(child, path.child(child.id()), content_origin, content_area))
            .collect(),
    };

    layout::Box::with_path(path, rect, children)
}

fn resolve_size(size: layout::Size, available: f32) -> f32 {
    match size {
        layout::Size::Fit => 0.0,
        layout::Size::Fill => available.max(0.0),
        layout::Size::Fixed(value) => value.max(0.0),
    }
}

fn vertical_children(
    node: &ui::Node,
    path: &ui::Path,
    origin: point::Logical,
    available: area::Logical,
) -> Vec<layout::Box> {
    let fixed_height: f32 = node
        .children()
        .iter()
        .map(|child| match child.layout().height() {
            layout::Size::Fixed(value) => value.max(0.0),
            _ => 0.0,
        })
        .sum();
    let fill_count = node
        .children()
        .iter()
        .filter(|child| !matches!(child.layout().height(), layout::Size::Fixed(_)))
        .count();
    let fill_height = if fill_count == 0 {
        0.0
    } else {
        ((available.height() - fixed_height).max(0.0)) / fill_count as f32
    };
    let mut y = origin.y();
    let mut children = Vec::with_capacity(node.children().len());

    for child in node.children() {
        let child_layout = child.layout();
        let height = match child_layout.height() {
            layout::Size::Fixed(value) => value.max(0.0),
            _ => fill_height,
        };
        let child_area = area::logical(
            resolve_size(child_layout.width(), available.width()),
            height,
        );
        children.push(self::node(
            child,
            path.child(child.id()),
            point::logical(origin.x(), y),
            child_area,
        ));
        y += height;
    }

    children
}

fn horizontal_children(
    node: &ui::Node,
    path: &ui::Path,
    origin: point::Logical,
    available: area::Logical,
) -> Vec<layout::Box> {
    let fixed_width: f32 = node
        .children()
        .iter()
        .map(|child| match child.layout().width() {
            layout::Size::Fixed(value) => value.max(0.0),
            _ => 0.0,
        })
        .sum();
    let fill_count = node
        .children()
        .iter()
        .filter(|child| !matches!(child.layout().width(), layout::Size::Fixed(_)))
        .count();
    let fill_width = if fill_count == 0 {
        0.0
    } else {
        ((available.width() - fixed_width).max(0.0)) / fill_count as f32
    };
    let mut x = origin.x();
    let mut children = Vec::with_capacity(node.children().len());

    for child in node.children() {
        let child_layout = child.layout();
        let width = match child_layout.width() {
            layout::Size::Fixed(value) => value.max(0.0),
            _ => fill_width,
        };
        let child_area = area::logical(
            width,
            resolve_size(child_layout.height(), available.height()),
        );
        children.push(self::node(
            child,
            path.child(child.id()),
            point::logical(x, origin.y()),
            child_area,
        ));
        x += width;
    }

    children
}
