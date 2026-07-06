use super::super::{
    geometry::{Rect, Size},
    theme, view,
};
use super::{engine, frame::Frame, measure, path};
use measure::{
    cross_axis_height, cross_axis_offset, cross_axis_width, gap_total, grows_vertical_space,
    inset_rect, intrinsic_height, main_axis_offset, menu_shortcut_width, menu_title_width,
    popup_height, popup_width, resolved_height, resolved_row_width,
};

pub(super) fn compose_frames(
    root: &view::Node,
    size: Size,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
) -> Vec<Frame> {
    let mut frames = Vec::new();
    layout_node(
        root,
        path::Path::root(),
        Rect::from_size(size),
        engine,
        theme,
        &mut frames,
    );
    frames
}

fn layout_node(
    node: &view::Node,
    path: path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    frames: &mut Vec<Frame>,
) {
    frames.push(Frame::new(node, path.clone(), rect, engine));

    match node.role() {
        view::node::Role::Root => layout_root(node, &path, rect, engine, theme, frames),
        view::node::Role::Stack => match node.axis() {
            Some(view::node::Axis::Horizontal) => {
                layout_horizontal_stack(node, &path, rect, engine, theme, frames)
            }
            Some(view::node::Axis::Vertical) | None => {
                layout_vertical_stack(node, &path, rect, engine, theme, frames)
            }
        },
        view::node::Role::MenuBar => layout_menu_bar(node, &path, rect, engine, theme, frames),
        view::node::Role::Popup => layout_popup(node, &path, rect, engine, theme, frames),
        view::node::Role::Panel => layout_vertical_stack(node, &path, rect, engine, theme, frames),
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
    theme: &theme::Theme,
    frames: &mut Vec<Frame>,
) {
    for (index, child) in node.children().iter().enumerate() {
        let child_path = path.child(index);
        if child.role() == view::node::Role::Popup {
            let height = popup_height(child, theme);
            let width = popup_width(child, engine, theme);
            let x = popup_anchor_x(child, frames, rect.x);
            let popup_rect = Rect::new(
                x,
                rect.y.saturating_add(theme.menu().bar_height),
                width,
                height,
            );
            layout_node(child, child_path, popup_rect, engine, theme, frames);
        } else {
            layout_node(child, child_path, rect, engine, theme, frames);
        }
    }
}

fn popup_anchor_x(node: &view::Node, frames: &[Frame], fallback: i32) -> i32 {
    let Some(anchor_id) = node.pointer_target().and_then(|target| target.element_id()) else {
        return fallback;
    };

    frames
        .iter()
        .find_map(|frame| {
            let target_id = frame.target().and_then(|target| target.element_id());
            (frame.role() == view::node::Role::Menu && target_id == Some(anchor_id))
                .then(|| frame.rect().x())
        })
        .unwrap_or(fallback)
}

fn layout_vertical_stack(
    node: &view::Node,
    path: &path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
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
        .map(|child| resolved_height(child, rect.height, theme))
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
                resolved_height(child, rect.height, theme)
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
        let width = cross_axis_width(child, rect.width, engine, node.style().align_items(), theme);
        let x = cross_axis_offset(rect.x, rect.width, width, node.style().align_items());
        let child_rect = Rect::new(x, y, width, height);
        layout_node(child, path.child(index), child_rect, engine, theme, frames);
        y = y.saturating_add(height).saturating_add(gap);
    }
}

fn layout_horizontal_stack(
    node: &view::Node,
    path: &path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
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
            let height = cross_axis_height(child, rect.height, node.style().align_items(), theme);
            let y = cross_axis_offset(rect.y, rect.height, height, node.style().align_items());
            let child_rect = Rect::new(x, y, width, height);
            layout_node(child, path.child(index), child_rect, engine, theme, frames);
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
        .map(|child| resolved_row_width(child, rect.width, engine, theme))
        .sum();
    let grow_width = (rect.width - fixed_width - gap_total).max(0) / grow_count;
    let widths = children
        .iter()
        .map(|child| {
            if matches!(child.style().width(), Some(view::style::Dimension::Grow)) {
                grow_width
            } else {
                resolved_row_width(child, rect.width, engine, theme)
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
        let height = cross_axis_height(child, rect.height, node.style().align_items(), theme);
        let y = cross_axis_offset(rect.y, rect.height, height, node.style().align_items());
        let child_rect = Rect::new(x, y, width, height);
        layout_node(child, path.child(index), child_rect, engine, theme, frames);
        x = x.saturating_add(width).saturating_add(gap);
    }
}

fn layout_menu_bar(
    node: &view::Node,
    path: &path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    frames: &mut Vec<Frame>,
) {
    let mut x = rect.x;
    let width = menu_title_width(node, engine, theme);
    let height = rect.height.min(theme.menu().bar_height);

    for (index, child) in node.children().iter().enumerate() {
        let child_rect = Rect::new(x, rect.y, width, height);
        frames.push(Frame::new(child, path.child(index), child_rect, engine));
        x = x.saturating_add(width);
    }
}

fn layout_popup(
    node: &view::Node,
    path: &path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    frames: &mut Vec<Frame>,
) {
    let padding = theme.floating_panel().padding;
    let mut y = rect.y.saturating_add(padding);
    let row_x = rect.x.saturating_add(padding);
    let row_width = rect.width.saturating_sub(padding.saturating_mul(2));
    let shortcut_width = menu_shortcut_width(node, engine);

    for (index, child) in node.children().iter().enumerate() {
        let height = intrinsic_height(child, theme);
        let child_rect = Rect::new(row_x, y, row_width, height);
        layout_menu_row(
            child,
            path.child(index),
            child_rect,
            shortcut_width,
            engine,
            frames,
        );
        y = y.saturating_add(height);
    }
}

fn layout_menu_row(
    node: &view::Node,
    path: path::Path,
    rect: Rect,
    shortcut_width: i32,
    engine: &mut engine::Engine,
    frames: &mut Vec<Frame>,
) {
    frames.push(Frame::new(node, path, rect, engine).with_menu_shortcut_width(shortcut_width));
}
