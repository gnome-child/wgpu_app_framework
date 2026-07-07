use super::super::{
    composition,
    geometry::{Rect, Size},
    keymap, theme, view,
};
use super::{
    Viewport, control, engine, flow,
    frame::{Clip, Frame, Input as FrameInput},
    measure, path,
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
    let mut ctx = LayoutContext::new(engine, theme, animation_frame, keymap);
    layout_node(
        root,
        retained_root,
        path::Path::root(),
        Rect::from_size(size),
        false,
        None,
        &mut ctx,
    );
    ctx.finish()
}

struct LayoutContext<'a> {
    engine: &'a mut engine::Engine,
    theme: &'a theme::Theme,
    animation_frame: animation::Frame,
    keymap: keymap::Profile,
    frames: Vec<Frame>,
}

impl<'a> LayoutContext<'a> {
    fn new(
        engine: &'a mut engine::Engine,
        theme: &'a theme::Theme,
        animation_frame: animation::Frame,
        keymap: keymap::Profile,
    ) -> Self {
        Self {
            engine,
            theme,
            animation_frame,
            keymap,
            frames: Vec::new(),
        }
    }

    fn finish(self) -> Vec<Frame> {
        self.frames
    }

    fn frame(
        &mut self,
        node: &view::Node,
        node_id: composition::NodeId,
        path: path::Path,
        rect: Rect,
        floating_layer: bool,
        clip: Option<Clip>,
    ) -> Frame {
        Frame::new(
            FrameInput {
                node,
                node_id,
                path,
                rect,
                floating_layer,
                clip,
                animation_frame: self.animation_frame,
                keymap: self.keymap,
            },
            self.engine,
            self.theme,
        )
    }
}

fn layout_node(
    node: &view::Node,
    retained: &composition::Node,
    path: path::Path,
    rect: Rect,
    floating_layer: bool,
    clip: Option<Clip>,
    ctx: &mut LayoutContext<'_>,
) {
    let floating_layer = floating_layer || is_floating_panel_role(node.role());
    if node.role() == view::Role::Scroll {
        layout_scroll(node, retained, path, rect, floating_layer, clip, ctx);
        return;
    }

    let frame = ctx.frame(
        node,
        retained.node_id(),
        path.clone(),
        rect,
        floating_layer,
        clip,
    );
    ctx.frames.push(frame);

    match node.role() {
        view::Role::Root => layout_root(node, retained, &path, rect, clip, ctx),
        view::Role::Stack => match node.axis() {
            Some(view::Axis::Horizontal) => {
                layout_horizontal_stack(node, retained, &path, rect, floating_layer, clip, ctx)
            }
            Some(view::Axis::Vertical) | None => {
                layout_vertical_stack(node, retained, &path, rect, floating_layer, clip, ctx)
            }
            Some(view::Axis::Overlay) => {
                layout_overlay_stack(node, retained, &path, rect, floating_layer, clip, ctx)
            }
        },
        view::Role::MenuBar => {
            layout_menu_bar(node, retained, &path, rect, floating_layer, clip, ctx)
        }
        view::Role::FloatingPanel => layout_floating_panel(node, retained, &path, rect, None, ctx),
        view::Role::Panel => {
            layout_vertical_stack(node, retained, &path, rect, floating_layer, clip, ctx)
        }
        view::Role::Menu
        | view::Role::Binding
        | view::Role::Separator
        | view::Role::TextArea
        | view::Role::Button
        | view::Role::Checkbox
        | view::Role::Radio
        | view::Role::Slider
        | view::Role::TextBox
        | view::Role::Scroll
        | view::Role::SectionHeader
        | view::Role::Label => {}
    }
}

fn is_floating_panel_role(role: view::Role) -> bool {
    role == view::Role::FloatingPanel
}

fn layout_scroll(
    node: &view::Node,
    retained: &composition::Node,
    path: path::Path,
    rect: Rect,
    floating_layer: bool,
    clip: Option<Clip>,
    ctx: &mut LayoutContext<'_>,
) {
    let viewport_rect = scroll_viewport_rect(node, rect, ctx.theme);
    let placement = scroll_stack_placement(node, viewport_rect, ctx.engine, ctx.theme, ctx.keymap);
    let viewport = Viewport::new(viewport_rect, placement.content, node.scroll_offset());
    debug_assert_scroll_content_contains(&placement, viewport_rect);
    let child_clip_rect =
        intersect_clip(clip, viewport_rect).unwrap_or_else(|| Clip::new(viewport_rect));

    let frame = ctx
        .frame(
            node,
            retained.node_id(),
            path.clone(),
            rect,
            floating_layer,
            clip,
        )
        .with_viewport(viewport);
    ctx.frames.push(frame);

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
        floating_layer,
        child_clip,
        ctx,
    );
}

fn scroll_viewport_rect(node: &view::Node, rect: Rect, theme: &theme::Theme) -> Rect {
    let metrics = theme.scrollbar().metrics;
    if metrics.policy != theme::ScrollbarPolicy::GutterAlways {
        return rect;
    }

    let gutter = metrics.thickness.max(1);
    match node.axis() {
        Some(view::Axis::Horizontal) => Rect::new(
            rect.x(),
            rect.y(),
            rect.width(),
            rect.height().saturating_sub(gutter),
        ),
        Some(view::Axis::Overlay) => rect,
        Some(view::Axis::Vertical) | None => Rect::new(
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
    clip: Option<Clip>,
    ctx: &mut LayoutContext<'_>,
) {
    for (index, child) in node.children().iter().enumerate() {
        let retained_child = retained_child(retained, index);
        let child_path = path.child(index);
        if child.role() == view::Role::FloatingPanel {
            let popup_rect = root_floating_panel_rect(
                child,
                &ctx.frames,
                rect,
                ctx.engine,
                ctx.theme,
                ctx.keymap,
            );
            layout_node(
                child,
                retained_child,
                child_path,
                popup_rect,
                true,
                None,
                ctx,
            );
        } else {
            layout_node(
                child,
                retained_child,
                child_path,
                rect,
                is_floating_panel_role(child.role()),
                clip,
                ctx,
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
    profile: keymap::Profile,
) -> Rect {
    let width = match node.style().width() {
        Some(_) => resolved_width(node, root.width, engine, theme, profile),
        None => floating_panel_width(node, engine, theme, profile).min(root.width.max(0)),
    };
    let height = match node.style().height() {
        Some(_) => resolved_height(node, root.height, theme),
        None => floating_panel_height_for_width(node, width, engine, theme, profile)
            .min(root.height.max(0)),
    };

    if let Some(anchor) = floating_panel_anchor(node, frames) {
        return Rect::new(anchor.x(), anchor.bottom(), width, height);
    }

    match node.floating_placement() {
        view::FloatingPlacement::Default => Rect::new(root.x(), root.y(), width, height),
        view::FloatingPlacement::CenteredMaxEnvelope => {
            let envelope_height =
                floating_panel_max_envelope_height_for_width(node, width, engine, theme, profile)
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
        (frame.role() == view::Role::Menu && target_id == Some(anchor_id)).then(|| frame.rect())
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
    profile: keymap::Profile,
) -> StackPlacement {
    match node.axis() {
        Some(view::Axis::Horizontal) => {
            horizontal_stack_placement(node, rect, engine, theme, profile, true)
        }
        Some(view::Axis::Overlay) => overlay_stack_placement(node, rect, engine, theme, profile),
        Some(view::Axis::Vertical) | None => {
            vertical_stack_placement(node, rect, engine, theme, profile, true)
        }
    }
}

fn vertical_stack_placement(
    node: &view::Node,
    rect: Rect,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
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
            profile,
        );
        let height = if scroll_axis {
            scroll_axis_child_height(child, width, content_height, engine, theme, profile)
        } else {
            let grows = grows_vertical_space(child);
            let measured = size_hint(
                child,
                flow::Constraints::new(Size::new(width, 0), Size::new(width, content_height)),
                engine,
                theme,
                profile,
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
        view::Align::Start
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
    profile: keymap::Profile,
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
        let grows = matches!(child.style().width(), Some(view::Dimension::Grow));
        let width = if scroll_axis {
            scroll_axis_child_width(child, content_width, engine, theme, profile)
        } else if grows {
            0
        } else {
            resolved_row_width(child, content_width, engine, theme, profile)
        };
        let height = cross_axis_height_for_width(
            child,
            width,
            content_height,
            engine,
            node.style().align_items(),
            theme,
            profile,
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
        view::Align::Start
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
                profile,
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
    profile: keymap::Profile,
) -> StackPlacement {
    let content = inset_rect(rect, node.style().padding());
    let child_rects = node
        .children()
        .iter()
        .map(|child| {
            let width = resolved_width(child, content.width(), engine, theme, profile);
            let height =
                resolved_height_for_width(child, width, content.height(), engine, theme, profile);
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
    profile: keymap::Profile,
) -> i32 {
    if grows_vertical_space(child) {
        return intrinsic_height_for_width(child, width, engine, theme, profile);
    }

    match child.style().height() {
        Some(view::Dimension::Percent(percent)) => {
            ((viewport_content_height.max(0) as f32) * percent).round() as i32
        }
        _ => resolved_height_for_width(child, width, SCROLL_AXIS_LIMIT, engine, theme, profile),
    }
    .max(0)
}

fn scroll_axis_child_width(
    child: &view::Node,
    viewport_content_width: i32,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    match child.style().width() {
        Some(view::Dimension::Grow) => intrinsic_width(child, engine, theme, profile),
        Some(view::Dimension::Percent(percent)) => {
            ((viewport_content_width.max(0) as f32) * percent).round() as i32
        }
        _ => resolved_row_width(child, SCROLL_AXIS_LIMIT, engine, theme, profile),
    }
    .max(0)
}

fn placed_content_size(rect: Rect, padding: view::Padding, children: &[Rect]) -> Size {
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
    floating_layer: bool,
    clip: Option<Clip>,
    ctx: &mut LayoutContext<'_>,
) {
    for (index, (child, child_rect)) in node.children().iter().zip(child_rects).enumerate() {
        layout_node(
            child,
            retained_child(retained, index),
            path.child(index),
            child_rect,
            floating_layer,
            child_clip(child, clip),
            ctx,
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
    floating_layer: bool,
    clip: Option<Clip>,
    ctx: &mut LayoutContext<'_>,
) {
    let placement = vertical_stack_placement(node, rect, ctx.engine, ctx.theme, ctx.keymap, false);
    emit_stack_children(
        node,
        retained,
        path,
        placement.child_rects,
        floating_layer,
        clip,
        ctx,
    );
}

fn layout_horizontal_stack(
    node: &view::Node,
    retained: &composition::Node,
    path: &path::Path,
    rect: Rect,
    floating_layer: bool,
    clip: Option<Clip>,
    ctx: &mut LayoutContext<'_>,
) {
    let placement =
        horizontal_stack_placement(node, rect, ctx.engine, ctx.theme, ctx.keymap, false);
    emit_stack_children(
        node,
        retained,
        path,
        placement.child_rects,
        floating_layer,
        clip,
        ctx,
    );
}

fn layout_overlay_stack(
    node: &view::Node,
    retained: &composition::Node,
    path: &path::Path,
    rect: Rect,
    floating_layer: bool,
    clip: Option<Clip>,
    ctx: &mut LayoutContext<'_>,
) {
    let placement = overlay_stack_placement(node, rect, ctx.engine, ctx.theme, ctx.keymap);
    emit_stack_children(
        node,
        retained,
        path,
        placement.child_rects,
        floating_layer,
        clip,
        ctx,
    );
}

fn layout_menu_bar(
    node: &view::Node,
    retained: &composition::Node,
    path: &path::Path,
    rect: Rect,
    floating_layer: bool,
    clip: Option<Clip>,
    ctx: &mut LayoutContext<'_>,
) {
    let mut x = rect.x;
    let width = menu_title_width(node, ctx.engine, ctx.theme);
    let height = rect.height.min(ctx.theme.menu().bar_height);

    for (index, child) in node.children().iter().enumerate() {
        let child_rect = Rect::new(x, rect.y, width, height);
        let frame = ctx.frame(
            child,
            retained_child(retained, index).node_id(),
            path.child(index),
            child_rect,
            floating_layer,
            clip,
        );
        ctx.frames.push(frame);
        x = x.saturating_add(width);
    }
}

fn layout_floating_panel(
    node: &view::Node,
    retained: &composition::Node,
    path: &path::Path,
    rect: Rect,
    _clip: Option<Clip>,
    ctx: &mut LayoutContext<'_>,
) {
    if !control::is_menu_panel(node) {
        let padding = ctx.theme.floating_panel().padding.max(0);
        let content = Rect::new(
            rect.x().saturating_add(padding),
            rect.y().saturating_add(padding),
            rect.width().saturating_sub(padding.saturating_mul(2)),
            rect.height().saturating_sub(padding.saturating_mul(2)),
        );

        match node.axis() {
            Some(view::Axis::Horizontal) => {
                layout_horizontal_stack(node, retained, path, content, true, None, ctx)
            }
            Some(view::Axis::Overlay) => {
                layout_overlay_stack(node, retained, path, content, true, None, ctx)
            }
            Some(view::Axis::Vertical) | None => {
                layout_vertical_stack(node, retained, path, content, true, None, ctx)
            }
        }

        return;
    }

    let padding = ctx.theme.floating_panel().padding;
    let panel_clip = Some(Clip::rounded(rect, ctx.theme.floating_panel().rounding));
    let mut y = rect.y.saturating_add(padding);
    let row_x = rect.x.saturating_add(padding);
    let row_width = rect.width.saturating_sub(padding.saturating_mul(2));
    let shortcut_width = menu_shortcut_width(node, ctx.engine, ctx.theme, ctx.keymap);

    for (index, child) in node.children().iter().enumerate() {
        let retained_child = retained_child(retained, index);
        let height = intrinsic_height(child, ctx.theme);
        let child_rect = Rect::new(row_x, y, row_width, height);
        if control::is_menu_panel_row(child) {
            layout_menu_row(
                child,
                retained_child,
                path.child(index),
                child_rect,
                shortcut_width,
                panel_clip,
                ctx,
            );
        } else {
            layout_node(
                child,
                retained_child,
                path.child(index),
                child_rect,
                true,
                None,
                ctx,
            );
        }
        y = y.saturating_add(height);
    }
}

fn layout_menu_row(
    node: &view::Node,
    retained: &composition::Node,
    path: path::Path,
    rect: Rect,
    shortcut_width: i32,
    clip: Option<Clip>,
    ctx: &mut LayoutContext<'_>,
) {
    let frame = ctx
        .frame(node, retained.node_id(), path, rect, true, clip)
        .with_shortcut_width(shortcut_width);
    ctx.frames.push(frame);
}

fn retained_child(parent: &composition::Node, index: usize) -> &composition::Node {
    parent
        .children()
        .get(index)
        .expect("composition tree must match view child order")
}
