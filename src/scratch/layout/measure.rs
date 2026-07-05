use super::super::{context::Source, geometry::Rect, theme, view};
use super::{control, engine};

pub(in crate::scratch::layout) fn intrinsic_width(
    node: &view::Node,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
) -> i32 {
    let label_width = node
        .label_text()
        .map(|label| engine.label_width(label))
        .unwrap_or_default();

    match node.role() {
        view::node::Role::MenuBar => {
            menu_title_width(node, engine, theme).saturating_mul(node.children().len() as i32)
        }
        view::node::Role::Menu => menu_intrinsic_width(node, engine, theme),
        view::node::Role::Popup => popup_width(node, engine, theme),
        view::node::Role::Binding
            if node
                .binding()
                .is_some_and(|binding| binding.source() == Source::Menu) =>
        {
            menu_row_width(node, engine, theme).max(theme.menu().popup_min_width)
        }
        view::node::Role::Binding => label_width.max(theme.menu().popup_min_width),
        view::node::Role::Slider => label_width.max(160),
        view::node::Role::TextBox => label_width.max(120),
        _ => control_intrinsic_width(node, label_width, theme),
    }
}

pub(in crate::scratch::layout) fn intrinsic_height(node: &view::Node, theme: &theme::Theme) -> i32 {
    let row_height = theme.menu().row_height;
    match node.role() {
        view::node::Role::MenuBar
        | view::node::Role::Menu
        | view::node::Role::Binding
        | view::node::Role::Popup
        | view::node::Role::Button
        | view::node::Role::Checkbox
        | view::node::Role::Radio
        | view::node::Role::Slider
        | view::node::Role::TextBox => row_height,
        view::node::Role::Separator => row_height,
        view::node::Role::Label => row_height,
        view::node::Role::TextArea
        | view::node::Role::Panel
        | view::node::Role::Root
        | view::node::Role::Stack => row_height,
    }
}

pub(in crate::scratch::layout) fn popup_height(node: &view::Node, theme: &theme::Theme) -> i32 {
    let child_height: i32 = node
        .children()
        .iter()
        .map(|child| intrinsic_height(child, theme))
        .sum();
    child_height
        .max(theme.menu().row_height)
        .saturating_add(theme.menu().padding.saturating_mul(2))
}

pub(in crate::scratch::layout) fn popup_width(
    node: &view::Node,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
) -> i32 {
    let content_width = node
        .children()
        .iter()
        .map(|child| menu_row_width(child, engine, theme))
        .max()
        .unwrap_or_default()
        .max(theme.menu().popup_min_width);

    content_width.saturating_add(theme.menu().padding.saturating_mul(2))
}

pub(in crate::scratch::layout) fn menu_shortcut_width(
    node: &view::Node,
    engine: &mut engine::Engine,
) -> i32 {
    node.children()
        .iter()
        .filter_map(|child| {
            child
                .binding()
                .and_then(view::Binding::shortcut)
                .map(|shortcut| engine.label_width(shortcut.as_str()))
        })
        .max()
        .unwrap_or_default()
}

pub(in crate::scratch::layout) fn menu_title_width(
    node: &view::Node,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
) -> i32 {
    node.children()
        .iter()
        .filter(|child| child.role() == view::node::Role::Menu)
        .map(|child| menu_intrinsic_width(child, engine, theme))
        .max()
        .unwrap_or_default()
}

fn menu_intrinsic_width(
    node: &view::Node,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
) -> i32 {
    let label_width = node
        .label_text()
        .map(|label| engine.label_width(label))
        .unwrap_or_default();
    let content_width = if has_single_character_label(node) {
        label_width.max(control::control_content_extent(
            theme.menu().bar_height,
            theme,
        ))
    } else {
        label_width
    };

    control::padded_control_extent(content_width, theme)
}

fn menu_row_width(node: &view::Node, engine: &mut engine::Engine, theme: &theme::Theme) -> i32 {
    if node.role() == view::node::Role::Separator {
        return 0;
    }

    let label_width = node
        .label_text()
        .map(|label| engine.label_width(label))
        .unwrap_or_default();
    let shortcut_width = node
        .binding()
        .and_then(view::Binding::shortcut)
        .map(|shortcut| engine.label_width(shortcut.as_str()))
        .unwrap_or_default();

    let side_slot_width = control::padded_control_extent(
        control::control_content_extent(theme.menu().row_height, theme),
        theme,
    );

    side_slot_width
        .saturating_add(label_width)
        .saturating_add(shortcut_width)
        .saturating_add(side_slot_width)
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
    theme: &theme::Theme,
) -> i32 {
    match node.style().width() {
        Some(view::style::Dimension::Fit) => intrinsic_width(node, engine, theme),
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
    theme: &theme::Theme,
) -> i32 {
    match node.style().width() {
        None | Some(view::style::Dimension::Fit) => intrinsic_width(node, engine, theme),
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
    theme: &theme::Theme,
) -> i32 {
    match align {
        view::style::Align::Stretch => resolved_width(node, parent_width, engine, theme),
        view::style::Align::Start | view::style::Align::Center | view::style::Align::End => {
            match node.style().width() {
                None | Some(view::style::Dimension::Fit) => intrinsic_width(node, engine, theme),
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
    theme: &theme::Theme,
) -> i32 {
    match align {
        view::style::Align::Stretch => match node.style().height() {
            None | Some(view::style::Dimension::Grow) => parent_height,
            Some(view::style::Dimension::Fit) => intrinsic_height(node, theme),
            Some(view::style::Dimension::Fixed(height)) => height,
            Some(view::style::Dimension::Percent(percent)) => {
                ((parent_height.max(0) as f32) * percent).round() as i32
            }
        },
        view::style::Align::Start | view::style::Align::Center | view::style::Align::End => {
            match node.style().height() {
                None | Some(view::style::Dimension::Fit) => intrinsic_height(node, theme),
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

pub(in crate::scratch::layout) fn resolved_height(
    node: &view::Node,
    parent_height: i32,
    theme: &theme::Theme,
) -> i32 {
    match node.style().height() {
        Some(view::style::Dimension::Fit) => intrinsic_height(node, theme),
        Some(view::style::Dimension::Grow) | None => {
            if grows_vertical_space(node) {
                parent_height
            } else {
                intrinsic_height(node, theme)
            }
        }
        Some(view::style::Dimension::Fixed(height)) => height,
        Some(view::style::Dimension::Percent(percent)) => {
            ((parent_height.max(0) as f32) * percent).round() as i32
        }
    }
    .clamp(0, parent_height.max(0))
}

fn control_intrinsic_width(node: &view::Node, label_width: i32, theme: &theme::Theme) -> i32 {
    if has_single_character_label(node) {
        let content_width = label_width.max(control::control_content_extent(
            theme.menu().row_height,
            theme,
        ));

        control::padded_control_extent(content_width, theme)
    } else {
        control::padded_control_extent(label_width, theme)
    }
}

fn has_single_character_label(node: &view::Node) -> bool {
    node.label_text()
        .is_some_and(|label| label.chars().count() == 1)
}
