use std::collections::HashMap;

use crate::geometry::{Rect, area, point};
use crate::{layout, text, ui, widget};

pub fn tree(
    root: &ui::Node,
    area: area::Logical,
    measurer: &mut text::layout::Engine,
) -> layout::Frame<ui::Path> {
    let mut nodes = HashMap::new();
    collect_nodes(root, &ui::Path::root(root.path_id(0)), &mut nodes);

    let item = project_node(root, ui::Path::root(root.path_id(0)));
    let mut adapter = UiMeasurer { nodes, measurer };
    let frame = layout::Engine::new().layout(&item, area, &mut adapter);

    apply_widget_layout(root, frame, &mut adapter)
}

pub fn subtree_at(
    root: &ui::Node,
    path: ui::Path,
    rect: Rect,
    measurer: &mut text::layout::Engine,
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
    measurer: &'m mut text::layout::Engine,
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
                    .measure(document, text::layout::Measure::bounded(max))
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

    for (index, child) in node.children().iter().enumerate() {
        collect_nodes(child, &path.child(child.path_id(index)), nodes);
    }
}

fn project_node(node: &ui::Node, path: ui::Path) -> layout::Item<ui::Path> {
    layout::Item::new(path.clone())
        .with_box_model(node.layout().box_model(node.style().padding()))
        .with_layout(node.layout().strategy())
        .with_children(
            node.children()
                .iter()
                .enumerate()
                .map(|(index, child)| project_node(child, path.child(child.path_id(index))))
                .collect(),
        )
}

fn project_children(node: &ui::Node, path: &ui::Path) -> Vec<layout::Item<ui::Path>> {
    node.children()
        .iter()
        .enumerate()
        .map(|(index, child)| project_node(child, path.child(child.path_id(index))))
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
    let Some(scroll) = node.scroll() else {
        return frame;
    };
    let viewport_base = widget::scroll::viewport_rect(node, frame.rect());
    let mut axes = widget::scroll::ActiveAxes::new(
        scroll.axes().vertical_enabled()
            && matches!(
                scroll.bars().vertical_policy(),
                widget::scroll::Policy::Always
            ),
        scroll.axes().horizontal_enabled()
            && matches!(
                scroll.bars().horizontal_policy(),
                widget::scroll::Policy::Always
            ),
    );
    let mut content_size = viewport_base.area;

    for _ in 0..4 {
        let (_, viewport, measured) =
            layout_scroll_children(node, &frame, axes, scroll.offset(), adapter);
        let next =
            scroll
                .bars()
                .active_axes(scroll.axes(), viewport_base, scroll.style(), measured);
        content_size = measured;

        if next == axes {
            let metrics = widget::scroll::Metrics::resolve(
                frame.rect(),
                viewport_base,
                content_size,
                scroll.offset(),
                scroll.axes(),
                scroll.bars(),
                scroll.style(),
            );
            let (children, _, _) = layout_scroll_children(
                node,
                &frame,
                metrics.active_axes(),
                metrics.offset(),
                adapter,
            );
            return frame.with_children(children);
        }

        axes = next;
        let _ = viewport;
    }

    let metrics = widget::scroll::Metrics::resolve(
        frame.rect(),
        viewport_base,
        content_size,
        scroll.offset(),
        scroll.axes(),
        scroll.bars(),
        scroll.style(),
    );
    let (children, _, _) = layout_scroll_children(
        node,
        &frame,
        metrics.active_axes(),
        metrics.offset(),
        adapter,
    );

    frame.with_children(children)
}

fn layout_scroll_children(
    node: &ui::Node,
    frame: &layout::Frame<ui::Path>,
    axes: widget::scroll::ActiveAxes,
    offset: point::Logical,
    adapter: &mut UiMeasurer<'_, '_>,
) -> (Vec<layout::Frame<ui::Path>>, Rect, area::Logical) {
    let Some(scroll) = node.scroll() else {
        return (frame.children().to_vec(), frame.rect(), frame.rect().area);
    };
    let viewport_base = widget::scroll::viewport_rect(node, frame.rect());
    let viewport = widget::scroll::viewport_rect_for_axes(viewport_base, scroll.style(), axes);
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
    let children: Vec<_> = node
        .children()
        .iter()
        .zip(inner.children())
        .map(|(child, child_frame)| {
            expand_to_descendants(apply_widget_layout(
                child,
                translate_frame(child_frame.clone(), dx, dy),
                adapter,
            ))
        })
        .collect();
    let content_size = widget::scroll::content_size_from_children(&children, viewport, offset);

    (children, viewport, content_size)
}

fn expand_to_descendants(frame: layout::Frame<ui::Path>) -> layout::Frame<ui::Path> {
    let children: Vec<_> = frame
        .children()
        .iter()
        .cloned()
        .map(expand_to_descendants)
        .collect();
    let rect = expand_rect_to_children(frame.rect(), &children);
    let content_rect = expand_rect_to_children(frame.content_rect(), &children);

    layout::Frame::with_content_rect(frame.path().clone(), rect, content_rect, children)
}

fn expand_rect_to_children(rect: Rect, children: &[layout::Frame<ui::Path>]) -> Rect {
    let right = children
        .iter()
        .fold(rect.origin.x() + rect.area.width(), |right, child| {
            right.max(child.rect().origin.x() + child.rect().area.width())
        });
    let bottom = children
        .iter()
        .fold(rect.origin.y() + rect.area.height(), |bottom, child| {
            bottom.max(child.rect().origin.y() + child.rect().area.height())
        });

    Rect::rounded(
        rect.origin,
        area::logical(
            (right - rect.origin.x()).max(rect.area.width()),
            (bottom - rect.origin.y()).max(rect.area.height()),
        ),
        rect.rounding,
    )
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
