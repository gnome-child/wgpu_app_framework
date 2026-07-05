use super::super::{
    geometry::{Rect, Size},
    view,
};
use super::{engine, frame::Frame, measure, path};
use measure::{
    MENU_BAR_HEIGHT, POPUP_WIDTH, cross_axis_height, cross_axis_offset, cross_axis_width,
    gap_total, grows_vertical_space, inset_rect, intrinsic_height, intrinsic_width,
    main_axis_offset, popup_height, resolved_height, resolved_row_width,
};

pub(super) fn compose_frames(
    root: &view::Node,
    size: Size,
    engine: &mut engine::Engine,
) -> Vec<Frame> {
    let mut frames = Vec::new();
    layout_node(
        root,
        path::Path::root(),
        Rect::from_size(size),
        engine,
        &mut frames,
    );
    frames
}

fn layout_node(
    node: &view::Node,
    path: path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    frames: &mut Vec<Frame>,
) {
    frames.push(Frame::new(node, path.clone(), rect, engine));

    match node.role() {
        view::node::Role::Root => layout_root(node, &path, rect, engine, frames),
        view::node::Role::Stack => match node.axis() {
            Some(view::node::Axis::Horizontal) => {
                layout_horizontal_stack(node, &path, rect, engine, frames)
            }
            Some(view::node::Axis::Vertical) | None => {
                layout_vertical_stack(node, &path, rect, engine, frames)
            }
        },
        view::node::Role::MenuBar => layout_menu_bar(node, &path, rect, engine, frames),
        view::node::Role::Popup => layout_popup(node, &path, rect, engine, frames),
        view::node::Role::Panel => layout_vertical_stack(node, &path, rect, engine, frames),
        view::node::Role::Menu
        | view::node::Role::Binding
        | view::node::Role::Separator
        | view::node::Role::TextArea
        | view::node::Role::Button
        | view::node::Role::Checkbox
        | view::node::Role::Radio
        | view::node::Role::Slider
        | view::node::Role::TextBox
        | view::node::Role::Label => {}
    }
}

fn layout_root(
    node: &view::Node,
    path: &path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    frames: &mut Vec<Frame>,
) {
    for (index, child) in node.children().iter().enumerate() {
        let child_path = path.child(index);
        if child.role() == view::node::Role::Popup {
            let height = popup_height(child);
            let popup_rect = Rect::new(rect.x, rect.y + MENU_BAR_HEIGHT, POPUP_WIDTH, height);
            layout_node(child, child_path, popup_rect, engine, frames);
        } else {
            layout_node(child, child_path, rect, engine, frames);
        }
    }
}

fn layout_vertical_stack(
    node: &view::Node,
    path: &path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    frames: &mut Vec<Frame>,
) {
    let children = node.children();
    if children.is_empty() {
        return;
    }

    let rect = inset_rect(rect, node.style().padding());
    let gap = node.style().gap();
    let gap_total = gap_total(gap, children.len());
    let grow_count = children
        .iter()
        .filter(|child| grows_vertical_space(child))
        .count() as i32;
    let fixed_height: i32 = children
        .iter()
        .filter(|child| !grows_vertical_space(child))
        .map(|child| resolved_height(child, rect.height))
        .sum();
    let fill_height = if grow_count > 0 {
        (rect.height - fixed_height - gap_total).max(0) / grow_count
    } else {
        0
    };
    let heights = children
        .iter()
        .map(|child| {
            if grows_vertical_space(child) {
                fill_height
            } else {
                resolved_height(child, rect.height)
            }
        })
        .collect::<Vec<_>>();
    let content_height = heights
        .iter()
        .copied()
        .sum::<i32>()
        .saturating_add(gap_total);
    let mut y = rect.y.saturating_add(main_axis_offset(
        node.style().justify_content(),
        rect.height,
        content_height,
    ));

    for (index, (child, height)) in children.iter().zip(heights).enumerate() {
        let width = cross_axis_width(child, rect.width, engine, node.style().align_items());
        let x = cross_axis_offset(rect.x, rect.width, width, node.style().align_items());
        let child_rect = Rect::new(x, y, width, height);
        layout_node(child, path.child(index), child_rect, engine, frames);
        y = y.saturating_add(height).saturating_add(gap);
    }
}

fn layout_horizontal_stack(
    node: &view::Node,
    path: &path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    frames: &mut Vec<Frame>,
) {
    let children = node.children();
    if children.is_empty() {
        return;
    }

    let rect = inset_rect(rect, node.style().padding());
    let gap = node.style().gap();
    let gap_total = gap_total(gap, children.len());
    let has_explicit_width = children.iter().any(|child| child.style().width().is_some());

    if !has_explicit_width {
        let width = (rect.width - gap_total).max(0) / children.len() as i32;
        let mut x = rect.x;

        for (index, child) in children.iter().enumerate() {
            let height = cross_axis_height(child, rect.height, node.style().align_items());
            let y = cross_axis_offset(rect.y, rect.height, height, node.style().align_items());
            let child_rect = Rect::new(x, y, width, height);
            layout_node(child, path.child(index), child_rect, engine, frames);
            x = x.saturating_add(width).saturating_add(gap);
        }

        return;
    }

    let grow_count = children
        .iter()
        .filter(|child| matches!(child.style().width(), Some(view::style::Dimension::Grow)))
        .count()
        .max(1) as i32;
    let fixed_width: i32 = children
        .iter()
        .filter(|child| !matches!(child.style().width(), Some(view::style::Dimension::Grow)))
        .map(|child| resolved_row_width(child, rect.width, engine))
        .sum();
    let grow_width = (rect.width - fixed_width - gap_total).max(0) / grow_count;
    let widths = children
        .iter()
        .map(|child| {
            if matches!(child.style().width(), Some(view::style::Dimension::Grow)) {
                grow_width
            } else {
                resolved_row_width(child, rect.width, engine)
            }
        })
        .collect::<Vec<_>>();
    let content_width = widths
        .iter()
        .copied()
        .sum::<i32>()
        .saturating_add(gap_total);
    let mut x = rect.x.saturating_add(main_axis_offset(
        node.style().justify_content(),
        rect.width,
        content_width,
    ));

    for (index, (child, width)) in children.iter().zip(widths).enumerate() {
        let height = cross_axis_height(child, rect.height, node.style().align_items());
        let y = cross_axis_offset(rect.y, rect.height, height, node.style().align_items());
        let child_rect = Rect::new(x, y, width, height);
        layout_node(child, path.child(index), child_rect, engine, frames);
        x = x.saturating_add(width).saturating_add(gap);
    }
}

fn layout_menu_bar(
    node: &view::Node,
    path: &path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    frames: &mut Vec<Frame>,
) {
    let mut x = rect.x;

    for (index, child) in node.children().iter().enumerate() {
        let width = intrinsic_width(child, engine);
        let child_rect = Rect::new(x, rect.y, width, rect.height.min(MENU_BAR_HEIGHT));
        frames.push(Frame::new(child, path.child(index), child_rect, engine));
        x = x.saturating_add(width);
    }
}

fn layout_popup(
    node: &view::Node,
    path: &path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    frames: &mut Vec<Frame>,
) {
    let mut y = rect.y;

    for (index, child) in node.children().iter().enumerate() {
        let height = intrinsic_height(child);
        let child_rect = Rect::new(rect.x, y, rect.width, height);
        layout_node(child, path.child(index), child_rect, engine, frames);
        y = y.saturating_add(height);
    }
}
