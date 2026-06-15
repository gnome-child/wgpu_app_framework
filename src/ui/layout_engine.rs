use crate::geometry::{Rect, area, point};
use crate::{layout, text, ui, widget};

pub fn tree(root: &ui::Node, area: area::Logical) -> layout::Box {
    let constraints = layout::Constraints::loose(area);
    let measured = measure_node(root, constraints);
    let root_area = resolve_root_area(root.layout(), measured, area);
    let rect = Rect::new(point::logical(0.0, 0.0), root_area);

    arrange_node(root, ui::Path::root(root.id()), rect)
}

pub fn subtree_at(root: &ui::Node, path: ui::Path, rect: Rect) -> layout::Box {
    arrange_node(root, path, rect)
}

fn measure_node(node: &ui::Node, constraints: layout::Constraints) -> area::Logical {
    let node_layout = node.layout();
    let max = sanitize_area(constraints.max());
    let padding = node.style().padding();
    let content_max = area::logical(
        (max.width() - padding.horizontal()).max(0.0),
        (max.height() - padding.vertical()).max(0.0),
    );
    let content = measure_content(node, content_max);
    let desired = area::logical(
        content.width() + padding.horizontal(),
        content.height() + padding.vertical(),
    );

    area::logical(
        resolve_measured_axis(
            node_layout.width(),
            desired.width(),
            constraints.min().width(),
            max.width(),
        ),
        resolve_measured_axis(
            node_layout.height(),
            desired.height(),
            constraints.min().height(),
            max.height(),
        ),
    )
}

fn measure_content(node: &ui::Node, max: area::Logical) -> area::Logical {
    let child_constraints = layout::Constraints::loose(max);
    let child_sizes: Vec<_> = node
        .children()
        .iter()
        .map(|child| measure_node(child, child_constraints))
        .collect();
    let child_content = match node.layout().direction() {
        Some(layout::Axis::Vertical) => measure_vertical_stack(&child_sizes, node.layout().gap()),
        Some(layout::Axis::Horizontal) => {
            measure_horizontal_stack(&child_sizes, node.layout().gap())
        }
        None => measure_overlay(&child_sizes),
    };
    let node_content = measure_node_content(node);

    area::logical(
        child_content
            .width()
            .max(node_content.width())
            .min(max.width()),
        child_content
            .height()
            .max(node_content.height())
            .min(max.height()),
    )
}

fn measure_vertical_stack(children: &[area::Logical], gap: f32) -> area::Logical {
    let gap = gap_total(gap, children.len());
    let width = children
        .iter()
        .map(area::Logical::width)
        .fold(0.0, f32::max);
    let height = children.iter().map(area::Logical::height).sum::<f32>() + gap;

    area::logical(width, height)
}

fn measure_horizontal_stack(children: &[area::Logical], gap: f32) -> area::Logical {
    let gap = gap_total(gap, children.len());
    let width = children.iter().map(area::Logical::width).sum::<f32>() + gap;
    let height = children
        .iter()
        .map(area::Logical::height)
        .fold(0.0, f32::max);

    area::logical(width, height)
}

fn measure_overlay(children: &[area::Logical]) -> area::Logical {
    let width = children
        .iter()
        .map(area::Logical::width)
        .fold(0.0, f32::max);
    let height = children
        .iter()
        .map(area::Logical::height)
        .fold(0.0, f32::max);

    area::logical(width, height)
}

fn measure_node_content(node: &ui::Node) -> area::Logical {
    let label = node
        .label()
        .map(measure_text)
        .unwrap_or_else(|| area::logical(0.0, 0.0));
    let icon_size = node
        .icon()
        .map(|_| node.icon_size().unwrap_or(24.0).max(0.0));
    let icon = icon_size
        .map(|size| area::logical(size, size))
        .unwrap_or_else(|| area::logical(0.0, 0.0));

    area::logical(
        label.width().max(icon.width()),
        label.height().max(icon.height()),
    )
}

fn measure_text(document: &text::Document) -> area::Logical {
    let mut width = 0.0_f32;
    let mut height = 0.0_f32;

    for block in document.blocks() {
        let mut line_width = 0.0_f32;
        let mut line_height = 0.0_f32;

        for run in block.runs() {
            let size = run.style().size.max(0.0);
            line_width += run.text().chars().count() as f32 * size * 0.6;
            line_height = line_height.max(size * 1.2);
        }

        width = width.max(line_width);
        height += line_height;
    }

    area::logical(width, height)
}

fn arrange_node(node: &ui::Node, path: ui::Path, rect: Rect) -> layout::Box {
    let scroll_offset = node
        .scroll()
        .map_or_else(|| point::logical(0.0, 0.0), |scroll| scroll.offset());
    let viewport = widget::scroll::viewport_rect(node, rect);
    let child_origin = point::logical(
        viewport.origin.x() - scroll_offset.x(),
        viewport.origin.y() - scroll_offset.y(),
    );
    let children = match node.layout().direction() {
        Some(layout::Axis::Vertical) => {
            arrange_vertical_children(node, &path, child_origin, viewport.area)
        }
        Some(layout::Axis::Horizontal) => {
            arrange_horizontal_children(node, &path, child_origin, viewport.area)
        }
        None => arrange_overlay_children(node, &path, child_origin, viewport.area),
    };

    layout::Box::with_path(path, rect, children)
}

fn arrange_vertical_children(
    node: &ui::Node,
    path: &ui::Path,
    origin: point::Logical,
    available: area::Logical,
) -> Vec<layout::Box> {
    let measured = measure_children(node, available);
    let gap = node.layout().gap();
    let gap_total = gap_total(gap, node.children().len());
    let fixed_fit_height = node
        .children()
        .iter()
        .zip(&measured)
        .map(|(child, measured)| match child.layout().height() {
            layout::Size::Fixed(value) => resolve_fixed_axis(value, available.height()),
            layout::Size::Fit => measured.height(),
            layout::Size::Fill => 0.0,
        })
        .sum::<f32>();
    let fill_count = node
        .children()
        .iter()
        .filter(|child| matches!(child.layout().height(), layout::Size::Fill))
        .count();
    let fill_height = fill_size(available.height(), fixed_fit_height, gap_total, fill_count);
    let total_height = fixed_fit_height + fill_height * fill_count as f32 + gap_total;
    let mut y = origin.y() + align_offset(node.layout().align(), available.height(), total_height);
    let mut children = Vec::with_capacity(node.children().len());

    for (child, measured) in node.children().iter().zip(measured) {
        let child_layout = child.layout();
        let height = resolve_stack_main_axis(
            child_layout.height(),
            measured.height(),
            available.height(),
            fill_height,
        );
        let width = resolve_stack_cross_axis(
            child_layout.width(),
            measured.width(),
            available.width(),
            node.layout().cross_align(),
        );
        let x = origin.x() + align_offset(node.layout().cross_align(), available.width(), width);
        let rect = Rect::new(point::logical(x, y), area::logical(width, height));

        children.push(arrange_node(child, path.child(child.id()), rect));
        y += height + gap;
    }

    children
}

fn arrange_horizontal_children(
    node: &ui::Node,
    path: &ui::Path,
    origin: point::Logical,
    available: area::Logical,
) -> Vec<layout::Box> {
    let measured = measure_children(node, available);
    let gap = node.layout().gap();
    let gap_total = gap_total(gap, node.children().len());
    let fixed_fit_width = node
        .children()
        .iter()
        .zip(&measured)
        .map(|(child, measured)| match child.layout().width() {
            layout::Size::Fixed(value) => resolve_fixed_axis(value, available.width()),
            layout::Size::Fit => measured.width(),
            layout::Size::Fill => 0.0,
        })
        .sum::<f32>();
    let fill_count = node
        .children()
        .iter()
        .filter(|child| matches!(child.layout().width(), layout::Size::Fill))
        .count();
    let fill_width = fill_size(available.width(), fixed_fit_width, gap_total, fill_count);
    let total_width = fixed_fit_width + fill_width * fill_count as f32 + gap_total;
    let mut x = origin.x() + align_offset(node.layout().align(), available.width(), total_width);
    let mut children = Vec::with_capacity(node.children().len());

    for (child, measured) in node.children().iter().zip(measured) {
        let child_layout = child.layout();
        let width = resolve_stack_main_axis(
            child_layout.width(),
            measured.width(),
            available.width(),
            fill_width,
        );
        let height = resolve_stack_cross_axis(
            child_layout.height(),
            measured.height(),
            available.height(),
            node.layout().cross_align(),
        );
        let y = origin.y() + align_offset(node.layout().cross_align(), available.height(), height);
        let rect = Rect::new(point::logical(x, y), area::logical(width, height));

        children.push(arrange_node(child, path.child(child.id()), rect));
        x += width + gap;
    }

    children
}

fn arrange_overlay_children(
    node: &ui::Node,
    path: &ui::Path,
    origin: point::Logical,
    available: area::Logical,
) -> Vec<layout::Box> {
    measure_children(node, available)
        .into_iter()
        .zip(node.children())
        .map(|(measured, child)| {
            let width =
                resolve_overlay_axis(child.layout().width(), measured.width(), available.width());
            let height = resolve_overlay_axis(
                child.layout().height(),
                measured.height(),
                available.height(),
            );
            let x = origin.x() + align_offset(node.layout().align(), available.width(), width);
            let y =
                origin.y() + align_offset(node.layout().cross_align(), available.height(), height);
            let rect = Rect::new(point::logical(x, y), area::logical(width, height));

            arrange_node(child, path.child(child.id()), rect)
        })
        .collect()
}

fn measure_children(node: &ui::Node, available: area::Logical) -> Vec<area::Logical> {
    let constraints = layout::Constraints::loose(available);

    node.children()
        .iter()
        .map(|child| measure_node(child, constraints))
        .collect()
}

fn resolve_root_area(
    layout: ui::Layout,
    measured: area::Logical,
    available: area::Logical,
) -> area::Logical {
    area::logical(
        resolve_root_axis(layout.width(), measured.width(), available.width()),
        resolve_root_axis(layout.height(), measured.height(), available.height()),
    )
}

fn resolve_root_axis(size: layout::Size, measured: f32, available: f32) -> f32 {
    match size {
        layout::Size::Fixed(value) => resolve_fixed_axis(value, available),
        layout::Size::Fill => available.max(0.0),
        layout::Size::Fit => measured.min(available.max(0.0)),
    }
}

fn resolve_measured_axis(size: layout::Size, desired: f32, min: f32, max: f32) -> f32 {
    let max = max.max(0.0);
    let value = match size {
        layout::Size::Fixed(value) => value.max(0.0),
        layout::Size::Fill | layout::Size::Fit => desired.max(0.0),
    };

    value.max(min.max(0.0)).min(max)
}

fn resolve_stack_main_axis(size: layout::Size, measured: f32, available: f32, fill: f32) -> f32 {
    match size {
        layout::Size::Fixed(value) => resolve_fixed_axis(value, available),
        layout::Size::Fill => fill,
        layout::Size::Fit => measured.min(available.max(0.0)),
    }
}

fn resolve_stack_cross_axis(
    size: layout::Size,
    measured: f32,
    available: f32,
    align: layout::Align,
) -> f32 {
    match size {
        layout::Size::Fixed(value) => resolve_fixed_axis(value, available),
        layout::Size::Fill => available.max(0.0),
        layout::Size::Fit if align == layout::Align::Stretch => available.max(0.0),
        layout::Size::Fit => measured.min(available.max(0.0)),
    }
}

fn resolve_overlay_axis(size: layout::Size, measured: f32, available: f32) -> f32 {
    match size {
        layout::Size::Fixed(value) => resolve_fixed_axis(value, available),
        layout::Size::Fill => available.max(0.0),
        layout::Size::Fit => measured.min(available.max(0.0)),
    }
}

fn resolve_fixed_axis(value: f32, max: f32) -> f32 {
    value.max(0.0).min(max.max(0.0))
}

fn fill_size(available: f32, fixed_fit: f32, gap: f32, fill_count: usize) -> f32 {
    if fill_count == 0 {
        return 0.0;
    }

    ((available - fixed_fit - gap).max(0.0)) / fill_count as f32
}

fn align_offset(align: layout::Align, available: f32, size: f32) -> f32 {
    let extra = (available - size).max(0.0);

    match align {
        layout::Align::Start | layout::Align::Stretch => 0.0,
        layout::Align::Center => extra / 2.0,
        layout::Align::End => extra,
    }
}

fn gap_total(gap: f32, count: usize) -> f32 {
    gap.max(0.0) * count.saturating_sub(1) as f32
}

fn sanitize_area(area: area::Logical) -> area::Logical {
    area::logical(area.width().max(0.0), area.height().max(0.0))
}
