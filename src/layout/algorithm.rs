use super::super::{
    composition,
    geometry::{Rect, Size},
    keymap, theme, view,
};
use super::{
    engine, flow,
    frame::{Clip, Frame},
    measure, path,
    viewport::Viewport,
};
use crate::animation;
use measure::{
    cross_axis_height_for_width, cross_axis_offset, cross_axis_width,
    floating_panel_height_for_width, floating_panel_max_envelope_height_for_width,
    floating_panel_width, grows_vertical_space, inset_rect, intrinsic_height,
    intrinsic_height_for_width, intrinsic_width, layout_gap, menu_shortcut_width, menu_title_width,
    resolved_height, resolved_height_for_width, resolved_row_width, resolved_width, size_hint,
};

const SCROLL_AXIS_LIMIT: i32 = i32::MAX / 4;

pub(super) fn compose_frames(
    root: &view::Node,
    retained_root: &composition::Node,
    size: Size,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    animation_frame: animation::Frame,
    keymap: keymap::Profile,
) -> Vec<Frame> {
    let mut frames = Vec::new();
    layout_node(
        root,
        retained_root,
        path::Path::root(),
        Rect::from_size(size),
        engine,
        theme,
        false,
        None,
        animation_frame,
        keymap,
        &mut frames,
    );
    frames
}

fn layout_node(
    node: &view::Node,
    retained: &composition::Node,
    path: path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    floating_layer: bool,
    clip: Option<Clip>,
    animation_frame: animation::Frame,
    keymap: keymap::Profile,
    frames: &mut Vec<Frame>,
) {
    let floating_layer = floating_layer || is_floating_panel_role(node.role());
    if node.role() == view::node::Role::Scroll {
        layout_scroll(
            node,
            retained,
            path,
            rect,
            engine,
            theme,
            floating_layer,
            clip,
            animation_frame,
            keymap,
            frames,
        );
        return;
    }

    frames.push(Frame::new(
        node,
        retained.node_id(),
        path.clone(),
        rect,
        engine,
        theme,
        floating_layer,
        clip,
        animation_frame,
        keymap,
    ));

    match node.role() {
        view::node::Role::Root => layout_root(
            node,
            retained,
            &path,
            rect,
            engine,
            theme,
            clip,
            animation_frame,
            keymap,
            frames,
        ),
        view::node::Role::Stack => match node.axis() {
            Some(view::node::Axis::Horizontal) => layout_horizontal_stack(
                node,
                retained,
                &path,
                rect,
                engine,
                theme,
                floating_layer,
                clip,
                animation_frame,
                keymap,
                frames,
            ),
            Some(view::node::Axis::Vertical) | None => layout_vertical_stack(
                node,
                retained,
                &path,
                rect,
                engine,
                theme,
                floating_layer,
                clip,
                animation_frame,
                keymap,
                frames,
            ),
            Some(view::node::Axis::Overlay) => layout_overlay_stack(
                node,
                retained,
                &path,
                rect,
                engine,
                theme,
                floating_layer,
                clip,
                animation_frame,
                keymap,
                frames,
            ),
        },
        view::node::Role::MenuBar => layout_menu_bar(
            node,
            retained,
            &path,
            rect,
            engine,
            theme,
            floating_layer,
            clip,
            animation_frame,
            keymap,
            frames,
        ),
        view::node::Role::FloatingPanel => layout_floating_panel(
            node,
            retained,
            &path,
            rect,
            engine,
            theme,
            None,
            animation_frame,
            keymap,
            frames,
        ),
        view::node::Role::Panel => layout_vertical_stack(
            node,
            retained,
            &path,
            rect,
            engine,
            theme,
            floating_layer,
            clip,
            animation_frame,
            keymap,
            frames,
        ),
        view::node::Role::Menu
        | view::node::Role::Binding
        | view::node::Role::Separator
        | view::node::Role::TextArea
        | view::node::Role::Button
        | view::node::Role::Checkbox
        | view::node::Role::Radio
        | view::node::Role::Slider
        | view::node::Role::TextBox
        | view::node::Role::Scroll
        | view::node::Role::SectionHeader
        | view::node::Role::Label => {}
    }
}

fn is_floating_panel_role(role: view::node::Role) -> bool {
    role == view::node::Role::FloatingPanel
}

fn layout_scroll(
    node: &view::Node,
    retained: &composition::Node,
    path: path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    floating_layer: bool,
    clip: Option<Clip>,
    animation_frame: animation::Frame,
    keymap: keymap::Profile,
    frames: &mut Vec<Frame>,
) {
    let viewport_rect = scroll_viewport_rect(node, rect, theme);
    let placement = scroll_stack_placement(node, viewport_rect, engine, theme);
    let viewport = Viewport::new(viewport_rect, placement.content, node.scroll_offset());
    debug_assert_scroll_content_contains(&placement, viewport_rect);
    let child_clip_rect =
        intersect_clip(clip, viewport_rect).unwrap_or_else(|| Clip::new(viewport_rect));

    frames.push(
        Frame::new(
            node,
            retained.node_id(),
            path.clone(),
            rect,
            engine,
            theme,
            floating_layer,
            clip,
            animation_frame,
            keymap,
        )
        .with_viewport(viewport),
    );

    let offset = viewport.resolved_scroll();
    let child_clip = Some(child_clip_rect);
    let dx = -offset.x();
    let dy = -offset.y();
    let child_rects = placement
        .child_rects
        .into_iter()
        .map(|rect| translate_rect(rect, dx, dy))
        .collect::<Vec<_>>();

    emit_stack_children(
        node,
        retained,
        &path,
        child_rects,
        engine,
        theme,
        floating_layer,
        child_clip,
        animation_frame,
        keymap,
        frames,
    );
}

fn scroll_viewport_rect(node: &view::Node, rect: Rect, theme: &theme::Theme) -> Rect {
    let metrics = theme.scrollbar().metrics;
    if metrics.policy != theme::ScrollbarPolicy::GutterAlways {
        return rect;
    }

    let gutter = metrics.thickness.max(1);
    match node.axis() {
        Some(view::node::Axis::Horizontal) => Rect::new(
            rect.x(),
            rect.y(),
            rect.width(),
            rect.height().saturating_sub(gutter),
        ),
        Some(view::node::Axis::Overlay) => rect,
        Some(view::node::Axis::Vertical) | None => Rect::new(
            rect.x(),
            rect.y(),
            rect.width().saturating_sub(gutter),
            rect.height(),
        ),
    }
}

fn intersect_clip(inherited: Option<Clip>, rect: Rect) -> Option<Clip> {
    let Some(inherited) = inherited else {
        return Some(Clip::new(rect));
    };
    let inherited = inherited.rect();
    let x = inherited.x().max(rect.x());
    let y = inherited.y().max(rect.y());
    let right = inherited.right().min(rect.right());
    let bottom = inherited.bottom().min(rect.bottom());

    (right > x && bottom > y).then(|| {
        Clip::new(Rect::new(
            x,
            y,
            right.saturating_sub(x),
            bottom.saturating_sub(y),
        ))
    })
}

fn child_clip(child: &view::Node, clip: Option<Clip>) -> Option<Clip> {
    (!is_floating_panel_role(child.role()))
        .then_some(clip)
        .flatten()
}

fn layout_root(
    node: &view::Node,
    retained: &composition::Node,
    path: &path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    clip: Option<Clip>,
    animation_frame: animation::Frame,
    keymap: keymap::Profile,
    frames: &mut Vec<Frame>,
) {
    for (index, child) in node.children().iter().enumerate() {
        let retained_child = retained_child(retained, index);
        let child_path = path.child(index);
        if child.role() == view::node::Role::FloatingPanel {
            let popup_rect = root_floating_panel_rect(child, frames, rect, engine, theme);
            layout_node(
                child,
                retained_child,
                child_path,
                popup_rect,
                engine,
                theme,
                true,
                None,
                animation_frame,
                keymap,
                frames,
            );
        } else {
            layout_node(
                child,
                retained_child,
                child_path,
                rect,
                engine,
                theme,
                is_floating_panel_role(child.role()),
                clip,
                animation_frame,
                keymap,
                frames,
            );
        }
    }
}

fn root_floating_panel_rect(
    node: &view::Node,
    frames: &[Frame],
    root: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
) -> Rect {
    let width = match node.style().width() {
        Some(_) => resolved_width(node, root.width, engine, theme),
        None => floating_panel_width(node, engine, theme).min(root.width.max(0)),
    };
    let height = match node.style().height() {
        Some(_) => resolved_height(node, root.height, theme),
        None => floating_panel_height_for_width(node, width, engine, theme).min(root.height.max(0)),
    };

    if let Some(anchor) = floating_panel_anchor(node, frames) {
        return Rect::new(anchor.x(), anchor.bottom(), width, height);
    }

    match node.floating_placement() {
        view::node::FloatingPlacement::Default => Rect::new(root.x(), root.y(), width, height),
        view::node::FloatingPlacement::CenteredMaxEnvelope => {
            let envelope_height =
                floating_panel_max_envelope_height_for_width(node, width, engine, theme)
                    .min(root.height().max(0));
            let x = root
                .x()
                .saturating_add(root.width().saturating_sub(width) / 2);
            let y = root
                .y()
                .saturating_add(root.height().saturating_sub(envelope_height) / 2);
            Rect::new(x, y, width, height)
        }
    }
}

fn floating_panel_anchor(node: &view::Node, frames: &[Frame]) -> Option<Rect> {
    let anchor_id = node
        .pointer_target()
        .and_then(|target| target.element_id())?;

    frames.iter().find_map(|frame| {
        let target_id = frame.target().and_then(|target| target.element_id());
        (frame.role() == view::node::Role::Menu && target_id == Some(anchor_id))
            .then(|| frame.rect())
    })
}

#[derive(Debug, Clone)]
struct StackPlacement {
    child_rects: Vec<Rect>,
    content: Size,
}

fn scroll_stack_placement(
    node: &view::Node,
    rect: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
) -> StackPlacement {
    match node.axis() {
        Some(view::node::Axis::Horizontal) => {
            horizontal_stack_placement(node, rect, engine, theme, true)
        }
        Some(view::node::Axis::Overlay) => overlay_stack_placement(node, rect, engine, theme),
        Some(view::node::Axis::Vertical) | None => {
            vertical_stack_placement(node, rect, engine, theme, true)
        }
    }
}

fn vertical_stack_placement(
    node: &view::Node,
    rect: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    scroll_axis: bool,
) -> StackPlacement {
    let children = node.children();
    if children.is_empty() {
        return StackPlacement {
            child_rects: Vec::new(),
            content: Size::new(rect.width(), rect.height()),
        };
    }

    let padding = node.style().padding();
    let content_width = rect.width().saturating_sub(padding.horizontal());
    let content_height = rect.height().saturating_sub(padding.vertical());
    let mut column = flow::Column::new()
        .gap(layout_gap(node, theme))
        .padding(padding)
        .align_items(node.style().align_items());

    for child in children {
        let width = cross_axis_width(
            child,
            content_width,
            engine,
            node.style().align_items(),
            theme,
        );
        let height = if scroll_axis {
            scroll_axis_child_height(child, width, content_height, engine, theme)
        } else {
            let grows = grows_vertical_space(child);
            let measured = size_hint(
                child,
                flow::Constraints::new(Size::new(width, 0), Size::new(width, content_height)),
                engine,
                theme,
            );
            if grows {
                0
            } else {
                measured.preferred().height()
            }
        };
        let hint = flow::SizeHint::new(Size::new(width, 0), Size::new(width, height));
        let item = if !scroll_axis && grows_vertical_space(child) {
            flow::Item::grow(hint)
        } else {
            flow::Item::fixed(hint)
        };
        column = column.item(item);
    }

    let preferred = column.size_hint().preferred();
    let layout_rect = if scroll_axis {
        Rect::new(
            rect.x(),
            rect.y(),
            rect.width(),
            preferred.height().max(rect.height()),
        )
    } else {
        rect
    };
    let justify_content = if scroll_axis && preferred.height() > rect.height() {
        view::style::Align::Start
    } else {
        node.style().justify_content()
    };
    column = column.justify_content(justify_content);
    let child_rects = column.layout(layout_rect);
    let content = placed_content_size(layout_rect, padding, &child_rects);

    StackPlacement {
        child_rects,
        content,
    }
}

fn horizontal_stack_placement(
    node: &view::Node,
    rect: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    scroll_axis: bool,
) -> StackPlacement {
    let children = node.children();
    if children.is_empty() {
        return StackPlacement {
            child_rects: Vec::new(),
            content: Size::new(rect.width(), rect.height()),
        };
    }

    let padding = node.style().padding();
    let content_width = rect.width().saturating_sub(padding.horizontal());
    let content_height = rect.height().saturating_sub(padding.vertical());
    let content_y = rect.y().saturating_add(padding.top());
    let mut row = flow::Row::new()
        .gap(layout_gap(node, theme))
        .padding(padding)
        .align_items(node.style().align_items());

    for child in children {
        let grows = matches!(child.style().width(), Some(view::style::Dimension::Grow));
        let width = if scroll_axis {
            scroll_axis_child_width(child, content_width, engine, theme)
        } else if grows {
            0
        } else {
            resolved_row_width(child, content_width, engine, theme)
        };
        let height = cross_axis_height_for_width(
            child,
            width,
            content_height,
            engine,
            node.style().align_items(),
            theme,
        );
        let hint = flow::SizeHint::new(Size::new(0, height), Size::new(width, height));
        let item = if !scroll_axis && grows {
            flow::Item::grow(hint)
        } else {
            flow::Item::fixed(hint)
        };
        row = row.item(item);
    }

    let preferred = row.size_hint().preferred();
    let layout_rect = if scroll_axis {
        Rect::new(
            rect.x(),
            rect.y(),
            preferred.width().max(rect.width()),
            rect.height(),
        )
    } else {
        rect
    };
    let justify_content = if scroll_axis && preferred.width() > rect.width() {
        view::style::Align::Start
    } else {
        node.style().justify_content()
    };
    row = row.justify_content(justify_content);
    let placed = row.layout(layout_rect);
    let child_rects = children
        .iter()
        .zip(placed)
        .map(|(child, row_rect)| {
            let height = cross_axis_height_for_width(
                child,
                row_rect.width(),
                content_height,
                engine,
                node.style().align_items(),
                theme,
            );
            let y = cross_axis_offset(
                content_y,
                content_height,
                height,
                node.style().align_items(),
            );
            Rect::new(row_rect.x(), y, row_rect.width(), height)
        })
        .collect::<Vec<_>>();
    let content = placed_content_size(layout_rect, padding, &child_rects);

    StackPlacement {
        child_rects,
        content,
    }
}

fn overlay_stack_placement(
    node: &view::Node,
    rect: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
) -> StackPlacement {
    let content = inset_rect(rect, node.style().padding());
    let child_rects = node
        .children()
        .iter()
        .map(|child| {
            let width = resolved_width(child, content.width(), engine, theme);
            let height = resolved_height_for_width(child, width, content.height(), engine, theme);
            let x = cross_axis_offset(
                content.x(),
                content.width(),
                width,
                node.style().align_items(),
            );
            let y = cross_axis_offset(
                content.y(),
                content.height(),
                height,
                node.style().justify_content(),
            );
            Rect::new(x, y, width, height)
        })
        .collect::<Vec<_>>();

    StackPlacement {
        child_rects,
        content: Size::new(rect.width(), rect.height()),
    }
}

fn scroll_axis_child_height(
    child: &view::Node,
    width: i32,
    viewport_content_height: i32,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
) -> i32 {
    if grows_vertical_space(child) {
        return intrinsic_height_for_width(child, width, engine, theme);
    }

    match child.style().height() {
        Some(view::style::Dimension::Percent(percent)) => {
            ((viewport_content_height.max(0) as f32) * percent).round() as i32
        }
        _ => resolved_height_for_width(child, width, SCROLL_AXIS_LIMIT, engine, theme),
    }
    .max(0)
}

fn scroll_axis_child_width(
    child: &view::Node,
    viewport_content_width: i32,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
) -> i32 {
    match child.style().width() {
        Some(view::style::Dimension::Grow) => intrinsic_width(child, engine, theme),
        Some(view::style::Dimension::Percent(percent)) => {
            ((viewport_content_width.max(0) as f32) * percent).round() as i32
        }
        _ => resolved_row_width(child, SCROLL_AXIS_LIMIT, engine, theme),
    }
    .max(0)
}

fn placed_content_size(rect: Rect, padding: view::style::Padding, children: &[Rect]) -> Size {
    let mut width = rect.width();
    let mut height = rect.height();

    for child in children {
        width = width.max(
            child
                .right()
                .saturating_sub(rect.x())
                .saturating_add(padding.right()),
        );
        height = height.max(
            child
                .bottom()
                .saturating_sub(rect.y())
                .saturating_add(padding.bottom()),
        );
    }

    Size::new(width.max(0), height.max(0))
}

fn emit_stack_children(
    node: &view::Node,
    retained: &composition::Node,
    path: &path::Path,
    child_rects: Vec<Rect>,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    floating_layer: bool,
    clip: Option<Clip>,
    animation_frame: animation::Frame,
    keymap: keymap::Profile,
    frames: &mut Vec<Frame>,
) {
    for (index, (child, child_rect)) in node.children().iter().zip(child_rects).enumerate() {
        layout_node(
            child,
            retained_child(retained, index),
            path.child(index),
            child_rect,
            engine,
            theme,
            floating_layer,
            child_clip(child, clip),
            animation_frame,
            keymap,
            frames,
        );
    }
}

fn translate_rect(rect: Rect, dx: i32, dy: i32) -> Rect {
    Rect::new(
        rect.x().saturating_add(dx),
        rect.y().saturating_add(dy),
        rect.width(),
        rect.height(),
    )
}

fn debug_assert_scroll_content_contains(placement: &StackPlacement, viewport: Rect) {
    debug_assert!(placement.content.width() >= viewport.width());
    debug_assert!(placement.content.height() >= viewport.height());

    for child in &placement.child_rects {
        debug_assert!(
            child.right().saturating_sub(viewport.x()) <= placement.content.width()
                && child.bottom().saturating_sub(viewport.y()) <= placement.content.height(),
            "scroll viewport content must contain placed child bounds"
        );
    }
}

fn layout_vertical_stack(
    node: &view::Node,
    retained: &composition::Node,
    path: &path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    floating_layer: bool,
    clip: Option<Clip>,
    animation_frame: animation::Frame,
    keymap: keymap::Profile,
    frames: &mut Vec<Frame>,
) {
    let placement = vertical_stack_placement(node, rect, engine, theme, false);
    emit_stack_children(
        node,
        retained,
        path,
        placement.child_rects,
        engine,
        theme,
        floating_layer,
        clip,
        animation_frame,
        keymap,
        frames,
    );
}

fn layout_horizontal_stack(
    node: &view::Node,
    retained: &composition::Node,
    path: &path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    floating_layer: bool,
    clip: Option<Clip>,
    animation_frame: animation::Frame,
    keymap: keymap::Profile,
    frames: &mut Vec<Frame>,
) {
    let placement = horizontal_stack_placement(node, rect, engine, theme, false);
    emit_stack_children(
        node,
        retained,
        path,
        placement.child_rects,
        engine,
        theme,
        floating_layer,
        clip,
        animation_frame,
        keymap,
        frames,
    );
}

fn layout_overlay_stack(
    node: &view::Node,
    retained: &composition::Node,
    path: &path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    floating_layer: bool,
    clip: Option<Clip>,
    animation_frame: animation::Frame,
    keymap: keymap::Profile,
    frames: &mut Vec<Frame>,
) {
    let placement = overlay_stack_placement(node, rect, engine, theme);
    emit_stack_children(
        node,
        retained,
        path,
        placement.child_rects,
        engine,
        theme,
        floating_layer,
        clip,
        animation_frame,
        keymap,
        frames,
    );
}

fn layout_menu_bar(
    node: &view::Node,
    retained: &composition::Node,
    path: &path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    floating_layer: bool,
    clip: Option<Clip>,
    animation_frame: animation::Frame,
    keymap: keymap::Profile,
    frames: &mut Vec<Frame>,
) {
    let mut x = rect.x;
    let width = menu_title_width(node, engine, theme);
    let height = rect.height.min(theme.menu().bar_height);

    for (index, child) in node.children().iter().enumerate() {
        let child_rect = Rect::new(x, rect.y, width, height);
        frames.push(Frame::new(
            child,
            retained_child(retained, index).node_id(),
            path.child(index),
            child_rect,
            engine,
            theme,
            floating_layer,
            clip,
            animation_frame,
            keymap,
        ));
        x = x.saturating_add(width);
    }
}

fn layout_floating_panel(
    node: &view::Node,
    retained: &composition::Node,
    path: &path::Path,
    rect: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    _clip: Option<Clip>,
    animation_frame: animation::Frame,
    keymap: keymap::Profile,
    frames: &mut Vec<Frame>,
) {
    if !is_menu_panel(node) {
        let padding = theme.floating_panel().padding.max(0);
        let content = Rect::new(
            rect.x().saturating_add(padding),
            rect.y().saturating_add(padding),
            rect.width().saturating_sub(padding.saturating_mul(2)),
            rect.height().saturating_sub(padding.saturating_mul(2)),
        );

        match node.axis() {
            Some(view::node::Axis::Horizontal) => layout_horizontal_stack(
                node,
                retained,
                path,
                content,
                engine,
                theme,
                true,
                None,
                animation_frame,
                keymap,
                frames,
            ),
            Some(view::node::Axis::Overlay) => layout_overlay_stack(
                node,
                retained,
                path,
                content,
                engine,
                theme,
                true,
                None,
                animation_frame,
                keymap,
                frames,
            ),
            Some(view::node::Axis::Vertical) | None => layout_vertical_stack(
                node,
                retained,
                path,
                content,
                engine,
                theme,
                true,
                None,
                animation_frame,
                keymap,
                frames,
            ),
        }

        return;
    }

    let padding = theme.floating_panel().padding;
    let panel_clip = Some(Clip::rounded(rect, theme.floating_panel().rounding));
    let mut y = rect.y.saturating_add(padding);
    let row_x = rect.x.saturating_add(padding);
    let row_width = rect.width.saturating_sub(padding.saturating_mul(2));
    let shortcut_width = menu_shortcut_width(node, engine, theme, keymap);

    for (index, child) in node.children().iter().enumerate() {
        let retained_child = retained_child(retained, index);
        let height = intrinsic_height(child, theme);
        let child_rect = Rect::new(row_x, y, row_width, height);
        if is_menu_panel_row(child) {
            layout_menu_row(
                child,
                retained_child,
                path.child(index),
                child_rect,
                shortcut_width,
                engine,
                theme,
                panel_clip,
                animation_frame,
                keymap,
                frames,
            );
        } else {
            layout_node(
                child,
                retained_child,
                path.child(index),
                child_rect,
                engine,
                theme,
                true,
                None,
                animation_frame,
                keymap,
                frames,
            );
        }
        y = y.saturating_add(height);
    }
}

fn is_menu_panel(node: &view::Node) -> bool {
    !node.children().is_empty() && node.children().iter().all(is_menu_panel_row)
}

fn is_menu_panel_row(node: &view::Node) -> bool {
    node.role() == view::node::Role::Separator
        || node
            .binding()
            .is_some_and(|binding| binding.source() == super::super::context::Source::Menu)
}

fn layout_menu_row(
    node: &view::Node,
    retained: &composition::Node,
    path: path::Path,
    rect: Rect,
    shortcut_width: i32,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    clip: Option<Clip>,
    animation_frame: animation::Frame,
    keymap: keymap::Profile,
    frames: &mut Vec<Frame>,
) {
    frames.push(
        Frame::new(
            node,
            retained.node_id(),
            path,
            rect,
            engine,
            theme,
            true,
            clip,
            animation_frame,
            keymap,
        )
        .with_shortcut_width(shortcut_width),
    );
}

fn retained_child(parent: &composition::Node, index: usize) -> &composition::Node {
    parent
        .children()
        .get(index)
        .expect("composition tree must match view child order")
}
