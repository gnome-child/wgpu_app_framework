use super::super::{geometry::Rect, view};
use super::engine;

pub(in crate::scratch::layout) const MENU_BAR_HEIGHT: i32 = 28;
pub(in crate::scratch::layout) const ROW_HEIGHT: i32 = 28;
pub(in crate::scratch::layout) const SEPARATOR_HEIGHT: i32 = 1;
pub(in crate::scratch::layout) const MIN_MENU_WIDTH: i32 = 48;
pub(in crate::scratch::layout) const POPUP_WIDTH: i32 = 220;

pub(in crate::scratch::layout) fn intrinsic_width(
    node: &view::Node,
    engine: &mut engine::Engine,
) -> i32 {
    let label_width = node
        .label_text()
        .map(|label| engine.label_width(label))
        .unwrap_or_default();

    match node.role() {
        view::node::Role::Menu => label_width.max(MIN_MENU_WIDTH),
        view::node::Role::Binding | view::node::Role::Popup => label_width.max(POPUP_WIDTH),
        view::node::Role::Slider => label_width.max(160),
        view::node::Role::TextBox => label_width.max(120),
        _ => label_width.max(MIN_MENU_WIDTH),
    }
}

pub(in crate::scratch::layout) fn intrinsic_height(node: &view::Node) -> i32 {
    match node.role() {
        view::node::Role::MenuBar
        | view::node::Role::Menu
        | view::node::Role::Binding
        | view::node::Role::Popup
        | view::node::Role::Button
        | view::node::Role::Checkbox
        | view::node::Role::Radio
        | view::node::Role::Slider
        | view::node::Role::TextBox => ROW_HEIGHT,
        view::node::Role::Separator => SEPARATOR_HEIGHT,
        view::node::Role::Label => ROW_HEIGHT,
        view::node::Role::TextArea
        | view::node::Role::Panel
        | view::node::Role::Root
        | view::node::Role::Stack => ROW_HEIGHT,
    }
}

pub(in crate::scratch::layout) fn popup_height(node: &view::Node) -> i32 {
    let child_height: i32 = node.children().iter().map(intrinsic_height).sum();
    child_height.max(ROW_HEIGHT)
}

pub(in crate::scratch::layout) fn gap_total(gap: i32, child_count: usize) -> i32 {
    gap.saturating_mul(child_count.saturating_sub(1) as i32)
}

pub(in crate::scratch::layout) fn grows_vertical_space(node: &view::Node) -> bool {
    match node.style().height() {
        Some(view::style::Dimension::Grow) => true,
        Some(
            view::style::Dimension::Fit
            | view::style::Dimension::Fixed(_)
            | view::style::Dimension::Percent(_),
        ) => false,
        None => matches!(
            node.role(),
            view::node::Role::TextArea | view::node::Role::Panel | view::node::Role::Stack
        ),
    }
}

pub(in crate::scratch::layout) fn resolved_width(
    node: &view::Node,
    parent_width: i32,
    engine: &mut engine::Engine,
) -> i32 {
    match node.style().width() {
        Some(view::style::Dimension::Fit) => intrinsic_width(node, engine),
        Some(view::style::Dimension::Grow) | None => parent_width,
        Some(view::style::Dimension::Fixed(width)) => width,
        Some(view::style::Dimension::Percent(percent)) => {
            ((parent_width.max(0) as f32) * percent).round() as i32
        }
    }
    .clamp(0, parent_width.max(0))
}

pub(in crate::scratch::layout) fn resolved_row_width(
    node: &view::Node,
    parent_width: i32,
    engine: &mut engine::Engine,
) -> i32 {
    match node.style().width() {
        None | Some(view::style::Dimension::Fit) => intrinsic_width(node, engine),
        Some(view::style::Dimension::Grow) => parent_width,
        Some(view::style::Dimension::Fixed(width)) => width,
        Some(view::style::Dimension::Percent(percent)) => {
            ((parent_width.max(0) as f32) * percent).round() as i32
        }
    }
    .clamp(0, parent_width.max(0))
}

pub(in crate::scratch::layout) fn cross_axis_width(
    node: &view::Node,
    parent_width: i32,
    engine: &mut engine::Engine,
    align: view::style::Align,
) -> i32 {
    match align {
        view::style::Align::Stretch => resolved_width(node, parent_width, engine),
        view::style::Align::Start | view::style::Align::Center | view::style::Align::End => {
            match node.style().width() {
                None | Some(view::style::Dimension::Fit) => intrinsic_width(node, engine),
                Some(view::style::Dimension::Grow) => parent_width,
                Some(view::style::Dimension::Fixed(width)) => width,
                Some(view::style::Dimension::Percent(percent)) => {
                    ((parent_width.max(0) as f32) * percent).round() as i32
                }
            }
            .clamp(0, parent_width.max(0))
        }
    }
}

pub(in crate::scratch::layout) fn cross_axis_height(
    node: &view::Node,
    parent_height: i32,
    align: view::style::Align,
) -> i32 {
    match align {
        view::style::Align::Stretch => match node.style().height() {
            None | Some(view::style::Dimension::Grow) => parent_height,
            Some(view::style::Dimension::Fit) => intrinsic_height(node),
            Some(view::style::Dimension::Fixed(height)) => height,
            Some(view::style::Dimension::Percent(percent)) => {
                ((parent_height.max(0) as f32) * percent).round() as i32
            }
        },
        view::style::Align::Start | view::style::Align::Center | view::style::Align::End => {
            match node.style().height() {
                None | Some(view::style::Dimension::Fit) => intrinsic_height(node),
                Some(view::style::Dimension::Grow) => parent_height,
                Some(view::style::Dimension::Fixed(height)) => height,
                Some(view::style::Dimension::Percent(percent)) => {
                    ((parent_height.max(0) as f32) * percent).round() as i32
                }
            }
        }
    }
    .clamp(0, parent_height.max(0))
}

pub(in crate::scratch::layout) fn main_axis_offset(
    align: view::style::Align,
    available: i32,
    content: i32,
) -> i32 {
    let slack = available.saturating_sub(content);
    match align {
        view::style::Align::Start | view::style::Align::Stretch => 0,
        view::style::Align::Center => slack / 2,
        view::style::Align::End => slack,
    }
}

pub(in crate::scratch::layout) fn cross_axis_offset(
    origin: i32,
    available: i32,
    size: i32,
    align: view::style::Align,
) -> i32 {
    origin.saturating_add(main_axis_offset(align, available, size))
}

pub(in crate::scratch::layout) fn inset_rect(rect: Rect, padding: view::style::Padding) -> Rect {
    let x = rect.x.saturating_add(padding.left());
    let y = rect.y.saturating_add(padding.top());
    let width = rect.width.saturating_sub(padding.horizontal());
    let height = rect.height.saturating_sub(padding.vertical());
    Rect::new(x, y, width, height)
}

pub(in crate::scratch::layout) fn resolved_height(node: &view::Node, parent_height: i32) -> i32 {
    match node.style().height() {
        Some(view::style::Dimension::Fit) => intrinsic_height(node),
        Some(view::style::Dimension::Grow) | None => {
            if grows_vertical_space(node) {
                parent_height
            } else {
                intrinsic_height(node)
            }
        }
        Some(view::style::Dimension::Fixed(height)) => height,
        Some(view::style::Dimension::Percent(percent)) => {
            ((parent_height.max(0) as f32) * percent).round() as i32
        }
    }
    .clamp(0, parent_height.max(0))
}
