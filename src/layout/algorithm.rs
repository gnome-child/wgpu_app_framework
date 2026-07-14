use super::super::{
    composition,
    geometry::{self, Rect, Size},
    interaction, keymap, theme,
    view::{self, node},
};
use super::{
    Viewport, control, engine, flow,
    frame::{Clip, Frame, Input as FrameInput},
    measure, path, table,
};
use crate::animation;
use measure::{
    cross_axis_height_for_width, cross_axis_offset, cross_axis_width,
    floating_panel_height_for_width, floating_panel_max_envelope_height_for_width,
    floating_panel_width, grows_vertical_space, intrinsic_height, intrinsic_height_for_width,
    intrinsic_or_fixed_height_for_width, intrinsic_width, layout_gap, menu_shortcut_width,
    menu_title_width, resolved_height, resolved_height_for_width, resolved_row_width,
    resolved_width, size_hint,
};

const SCROLL_AXIS_LIMIT: i32 = i32::MAX / 4;

pub(super) fn compose_frames(
    root: &view::Node,
    retained_root: &composition::tree::Node,
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
    table_projection: Option<table::Projection>,
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
            table_projection: None,
        }
    }

    fn finish(self) -> Vec<Frame> {
        self.frames
    }

    fn frame(
        &mut self,
        node: &view::Node,
        node_id: composition::tree::NodeId,
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
    retained: &composition::tree::Node,
    path: path::Path,
    rect: Rect,
    floating_layer: bool,
    clip: Option<Clip>,
    ctx: &mut LayoutContext<'_>,
) {
    let content = node.content();
    let floating_layer = floating_layer || is_floating_panel(node);
    match content {
        node::Content::Scroll(scroll) => {
            layout_scroll(
                node,
                scroll,
                retained,
                path,
                rect,
                floating_layer,
                clip,
                ctx,
            );
            return;
        }
        node::Content::VirtualList { model, .. } => {
            layout_virtual_list(node, model, retained, path, rect, floating_layer, clip, ctx);
            return;
        }
        _ => {}
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

    match content {
        node::Content::Root => layout_root(node, retained, &path, rect, clip, ctx),
        node::Content::Stack | node::Content::Table => match node.axis() {
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
        node::Content::MenuBar(_) => {
            layout_menu_bar(node, retained, &path, rect, floating_layer, clip, ctx)
        }
        node::Content::FloatingPanel(_) => {
            layout_floating_panel(node, retained, &path, rect, None, ctx)
        }
        node::Content::Panel => {
            layout_vertical_stack(node, retained, &path, rect, floating_layer, clip, ctx)
        }
        node::Content::Menu
        | node::Content::Binding
        | node::Content::Separator
        | node::Content::TextArea(_)
        | node::Content::Button(_)
        | node::Content::Checkbox(_)
        | node::Content::Radio(_)
        | node::Content::Slider(_)
        | node::Content::TextBox { .. }
        | node::Content::Scroll(_)
        | node::Content::VirtualList { .. }
        | node::Content::SectionHeader
        | node::Content::Label => {}
    }
}

fn is_floating_panel(node: &view::Node) -> bool {
    matches!(node.content(), node::Content::FloatingPanel(_))
}

fn layout_scroll(
    node: &view::Node,
    scroll: &node::Scroll,
    retained: &composition::tree::Node,
    path: path::Path,
    rect: Rect,
    floating_layer: bool,
    clip: Option<Clip>,
    ctx: &mut LayoutContext<'_>,
) {
    match scroll {
        node::Scroll::Ordinary { .. } => {}
        node::Scroll::Table { model, .. } => {
            layout_table_scroll(node, model, retained, path, rect, floating_layer, clip, ctx);
            return;
        }
    }

    let viewport_rect = scroll_viewport_rect(node, rect, ctx.theme);
    let (visible_frame, visible_content) = visible_scroll_geometry(node, rect, clip, ctx.theme);
    let placement = scroll_stack_placement(node, viewport_rect, ctx.engine, ctx.theme, ctx.keymap);
    let viewport = Viewport::new(viewport_rect, placement.content, node.scroll_offset())
        .with_visible(visible_frame, visible_content);
    debug_assert_scroll_content_contains(&placement, viewport_rect);
    let child_clip_rect = Clip::new(visible_content);

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

fn layout_table_scroll(
    node: &view::Node,
    model: &crate::table::Model,
    retained: &composition::tree::Node,
    path: path::Path,
    rect: Rect,
    floating_layer: bool,
    clip: Option<Clip>,
    ctx: &mut LayoutContext<'_>,
) {
    let surface = node
        .children()
        .first()
        .expect("table scroll owner must carry one surface");
    let header = surface
        .children()
        .first()
        .expect("table surface must carry its header");
    let columns = model.column_dimensions();
    debug_assert_eq!(columns.len(), header.children().len());

    let viewport_rect = scroll_viewport_rect(node, rect, ctx.theme);
    let (visible_frame, visible_content) = visible_scroll_geometry(node, rect, clip, ctx.theme);
    let mut allocation = flow::Row::new().pressure(flow::Pressure::Overflow);
    for ((_, dimension, _), header_cell) in columns.iter().copied().zip(header.children()) {
        let item = match dimension {
            view::Dimension::Fit => flow::Item::fixed(flow::SizeHint::fixed(Size::new(
                intrinsic_width(header_cell, ctx.engine, ctx.theme, ctx.keymap),
                1,
            ))),
            view::Dimension::Flexible { weight, minimum } => flow::Item::weighted(
                flow::SizeHint::new(Size::new(minimum, 1), Size::new(minimum, 1)),
                weight,
            ),
            view::Dimension::Fixed(width) => {
                flow::Item::fixed(flow::SizeHint::fixed(Size::new(width, 1)))
            }
            view::Dimension::Percent(percent) => {
                let width = ((viewport_rect.width().max(0) as f32) * percent).round() as i32;
                flow::Item::fixed(flow::SizeHint::fixed(Size::new(width, 1)))
            }
        };
        allocation = allocation.item(item);
    }
    let declared = allocation.layout(Rect::new(0, 0, viewport_rect.width(), 1));
    let mut origin = 0;
    let allocated = declared
        .into_iter()
        .zip(columns.iter())
        .map(|(rect, (_, _, resize_override))| {
            let width = resize_override.unwrap_or(rect.width()).max(0);
            let resolved = Rect::new(origin, rect.y(), width, rect.height());
            origin = origin.saturating_add(width);
            resolved
        })
        .collect::<Vec<_>>();
    let track_width = allocated
        .last()
        .map_or(viewport_rect.width(), |rect| rect.right())
        .max(0);
    let scroll_width = track_width.max(viewport_rect.width());
    let viewport = Viewport::new(
        viewport_rect,
        Size::new(scroll_width, viewport_rect.height()),
        node.scroll_offset(),
    )
    .with_visible(visible_frame, visible_content);
    let offset = viewport.resolved_scroll();
    let surface_rect = Rect::new(
        viewport_rect.x().saturating_sub(offset.x()),
        viewport_rect.y(),
        scroll_width,
        viewport_rect.height(),
    );
    let track_rect = Rect::new(
        surface_rect.x(),
        surface_rect.y(),
        track_width,
        surface_rect.height(),
    );
    let projection = table::Projection::new(
        model.id(),
        visible_content,
        track_rect,
        columns
            .iter()
            .map(|(identity, _, _)| *identity)
            .zip(allocated),
    );
    let table_clip = Some(Clip::new(visible_content));
    let frame = ctx
        .frame(
            node,
            retained.node_id(),
            path.clone(),
            rect,
            floating_layer,
            clip,
        )
        .with_table_scroll(viewport, projection.clone());
    ctx.frames.push(frame);

    let previous = ctx.table_projection.replace(projection);
    layout_node(
        surface,
        retained_child(retained, 0),
        path.child(0),
        surface_rect,
        floating_layer,
        child_clip(surface, table_clip),
        ctx,
    );
    ctx.table_projection = previous;
}

fn layout_virtual_list(
    node: &view::Node,
    model: &crate::virtual_list::Model,
    retained: &composition::tree::Node,
    path: path::Path,
    rect: Rect,
    floating_layer: bool,
    clip: Option<Clip>,
    ctx: &mut LayoutContext<'_>,
) {
    let viewport_rect = scroll_viewport_rect(node, rect, ctx.theme);
    let (visible_frame, visible_content) = visible_scroll_geometry(node, rect, clip, ctx.theme);
    if let Some(measurements) = model.measurements() {
        layout_variable_virtual_list(
            node,
            retained,
            path,
            rect,
            viewport_rect,
            floating_layer,
            clip,
            model,
            measurements,
            ctx,
        );
        return;
    }
    let row_height = model.row_height().max(1);
    let content_height = (model.len() as i64)
        .saturating_mul(row_height as i64)
        .min(i32::MAX as i64) as i32;
    let viewport = Viewport::new(
        viewport_rect,
        Size::new(
            viewport_rect.width(),
            content_height.max(viewport_rect.height()),
        ),
        node.scroll_offset(),
    )
    .with_visible(visible_frame, visible_content);
    let offset = viewport.resolved_scroll();
    let visible_start = (offset.y().max(0) / row_height) as usize;
    let visible_end =
        ((offset.y().max(0) as i64 + viewport_rect.height().max(0) as i64 + row_height as i64 - 1)
            / row_height as i64) as usize;
    let range = visible_start.saturating_sub(model.overscan())
        ..visible_end
            .saturating_add(model.overscan())
            .min(model.len());
    let request = crate::virtual_list::Request::new(model.id(), range);
    let row_clip = Some(Clip::new(visible_content));
    let frame = ctx
        .frame(
            node,
            retained.node_id(),
            path.clone(),
            rect,
            floating_layer,
            clip,
        )
        .with_virtual_list(viewport, request);
    ctx.frames.push(frame);

    for (index, child) in node.children().iter().enumerate() {
        let row = child
            .provided_row()
            .expect("materialized VirtualList children must carry provider identity");
        let logical_y = (row.index() as i64).saturating_mul(row_height as i64);
        let y = (viewport_rect.y() as i64)
            .saturating_add(logical_y)
            .saturating_sub(offset.y() as i64)
            .clamp(i32::MIN as i64, i32::MAX as i64) as i32;
        let child_rect = Rect::new(viewport_rect.x(), y, viewport_rect.width(), row_height);
        layout_node(
            child,
            retained_child(retained, index),
            path.child(index),
            child_rect,
            floating_layer,
            child_clip(child, row_clip),
            ctx,
        );
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "keeps recursive traversal facts and already-resolved variable-list facts explicit without a transport-only argument bag"
)]
fn layout_variable_virtual_list(
    node: &view::Node,
    retained: &composition::tree::Node,
    path: path::Path,
    rect: Rect,
    viewport_rect: Rect,
    floating_layer: bool,
    clip: Option<Clip>,
    model: &crate::virtual_list::Model,
    measured_sequence: crate::virtual_list::Measurements,
    ctx: &mut LayoutContext<'_>,
) {
    let (visible_frame, visible_content) = visible_scroll_geometry(node, rect, clip, ctx.theme);
    let provider = model.provider();
    let requested_offset = node.scroll_offset();
    {
        let mut region = measured_sequence.borrow_mut();
        region.set_width(viewport_rect.width(), provider);
        region.request(
            requested_offset.y(),
            viewport_rect.height(),
            model.overscan(),
            Vec::new(),
            provider,
        );
    }

    let table_projection = ctx.table_projection.clone();
    let row_height_floor = model.row_height();
    let row_measurements = node.children().iter().filter_map(|child| {
        let row = child.provided_row()?;
        let table_intrinsic = table_projection.as_ref().and_then(|projection| {
            table_stack_intrinsic_height(child, projection, ctx.engine, ctx.theme, ctx.keymap)
        });
        let height = table_intrinsic
            .map(|height| height.max(row_height_floor))
            .unwrap_or_else(|| {
                intrinsic_or_fixed_height_for_width(
                    child,
                    viewport_rect.width(),
                    ctx.engine,
                    ctx.theme,
                    ctx.keymap,
                )
            });
        Some((row.key(), height.max(1)))
    });
    let (offset_y, content_height, request) = {
        let mut region = measured_sequence.borrow_mut();
        let offset_y = region.refine(row_measurements, provider);
        let request = region.request(
            offset_y,
            viewport_rect.height(),
            model.overscan(),
            Vec::new(),
            provider,
        );
        (offset_y, region.content_height(), request)
    };
    let viewport = Viewport::new(
        viewport_rect,
        Size::new(
            viewport_rect.width(),
            content_height.max(viewport_rect.height()),
        ),
        interaction::ScrollOffset::new(requested_offset.x(), offset_y),
    )
    .with_visible(visible_frame, visible_content);
    let offset = viewport.resolved_scroll();
    let request = crate::virtual_list::Request::variable(
        model.id(),
        request.range(),
        measured_sequence.clone(),
    );
    let row_clip = Some(Clip::new(visible_content));
    let frame = ctx
        .frame(
            node,
            retained.node_id(),
            path.clone(),
            rect,
            floating_layer,
            clip,
        )
        .with_virtual_list(viewport, request);
    ctx.frames.push(frame);

    let region = measured_sequence.borrow();
    for (index, child) in node.children().iter().enumerate() {
        let row = child
            .provided_row()
            .expect("materialized VirtualList children must carry provider identity");
        let y = viewport_rect
            .y()
            .saturating_add(region.offset_for_index(row.index()))
            .saturating_sub(offset.y());
        let child_rect = Rect::new(
            viewport_rect.x(),
            y,
            viewport_rect.width(),
            region.height_for(row.key()),
        );
        layout_node(
            child,
            retained_child(retained, index),
            path.child(index),
            child_rect,
            floating_layer,
            child_clip(child, row_clip),
            ctx,
        );
    }
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

fn visible_scroll_geometry(
    node: &view::Node,
    rect: Rect,
    inherited: Option<Clip>,
    theme: &theme::Theme,
) -> (Rect, Rect) {
    let frame = intersect_rect(inherited.map(Clip::rect), rect);
    (frame, scroll_viewport_rect(node, frame, theme))
}

fn intersect_rect(inherited: Option<Rect>, rect: Rect) -> Rect {
    let Some(inherited) = inherited else {
        return rect;
    };
    let x = inherited.x().max(rect.x());
    let y = inherited.y().max(rect.y());
    let right = inherited.right().min(rect.right());
    let bottom = inherited.bottom().min(rect.bottom());

    Rect::new(x, y, right.saturating_sub(x), bottom.saturating_sub(y))
}

fn child_clip(child: &view::Node, clip: Option<Clip>) -> Option<Clip> {
    (!is_floating_panel(child)).then_some(clip).flatten()
}

fn layout_root(
    node: &view::Node,
    retained: &composition::tree::Node,
    path: &path::Path,
    rect: Rect,
    clip: Option<Clip>,
    ctx: &mut LayoutContext<'_>,
) {
    for (index, child) in node.children().iter().enumerate() {
        let retained_child = retained_child(retained, index);
        let child_path = path.child(index);
        if is_floating_panel(child) {
            let (popup_rect, popup_placement) = root_floating_panel_rect(
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
            if let Some(panel) = ctx
                .frames
                .iter_mut()
                .find(|frame| frame.node_id() == retained_child.node_id())
            {
                panel.set_popup_placement(popup_placement);
            }
        } else {
            layout_node(child, retained_child, child_path, rect, false, clip, ctx);
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
) -> (Rect, Option<geometry::placement::Request>) {
    let attachment = floating_panel_attachment(node, frames, theme);
    let anchor = attachment.map(|(anchor, _, _)| anchor);
    let width = match node.style().width() {
        Some(_) => resolved_width(node, root.width, engine, theme, profile),
        None if anchor.is_some() => floating_panel_width(node, engine, theme, profile),
        None => floating_panel_width(node, engine, theme, profile).min(root.width.max(0)),
    }
    .min(
        node.auxiliary_hint()
            .map(|_| theme.auxiliary_panel().max_width)
            .unwrap_or(i32::MAX),
    );
    let height = match node.style().height() {
        Some(_) => resolved_height(node, root.height, theme),
        None if anchor.is_some() => {
            floating_panel_height_for_width(node, width, engine, theme, profile)
        }
        None => floating_panel_height_for_width(node, width, engine, theme, profile)
            .min(root.height.max(0)),
    }
    .min(
        node.auxiliary_hint()
            .map(|_| theme.auxiliary_panel().max_height)
            .unwrap_or(i32::MAX),
    );

    if let Some(anchor) = anchor {
        let clearance = attachment.map_or(0, |(_, clearance, _)| clearance);
        let request = geometry::placement::Request::new(anchor, Size::new(width, height))
            .with_clearance(clearance);
        let available = attachment
            .and_then(|(_, _, available)| available)
            .unwrap_or(root);
        return (request.resolve(available), Some(request));
    }

    let rect = match node.floating_placement() {
        view::FloatingPlacement::Default => Rect::new(root.x(), root.y(), width, height),
        view::FloatingPlacement::Offset { x, y } => Rect::new(
            root.x().saturating_add(x),
            root.y().saturating_add(y),
            width,
            height,
        ),
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
    };
    (rect, None)
}

fn floating_panel_attachment(
    node: &view::Node,
    frames: &[Frame],
    theme: &theme::Theme,
) -> Option<(geometry::placement::Anchor, i32, Option<Rect>)> {
    match node.panel_attachment()? {
        view::PanelAttachment::Geometry { anchor, available } => Some((anchor, 0, available)),
        view::PanelAttachment::Pointer(point) => Some((
            geometry::placement::Anchor::Point(point),
            theme.auxiliary_panel().pointer_clearance,
            None,
        )),
        view::PanelAttachment::Element(id) => frames.iter().find_map(|frame| {
            (frame.target().and_then(interaction::Target::element_id) == Some(id))
                .then(|| (geometry::placement::Anchor::Rect(frame.rect()), 0, None))
        }),
    }
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
            vertical_stack_placement(node, rect, engine, theme, profile, true, None)
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
    table_projection: Option<&table::Projection>,
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
        .align_items(node.style().align_items())
        .pressure(if scroll_axis {
            flow::Pressure::Overflow
        } else {
            flow::Pressure::Fit
        });

    for child in children {
        let width = cross_axis_width(
            child,
            content_width,
            engine,
            node.style().align_items(),
            theme,
            profile,
        );
        let flexible = child.style().height().and_then(view::Dimension::flexible);
        let height = if scroll_axis {
            scroll_axis_child_height(child, width, content_height, engine, theme, profile)
        } else {
            let grows = grows_vertical_space(child);
            let measured_height = table_projection
                .and_then(|projection| {
                    table_stack_intrinsic_height(child, projection, engine, theme, profile)
                })
                .unwrap_or_else(|| {
                    size_hint(
                        child,
                        flow::Constraints::new(
                            Size::new(width, 0),
                            Size::new(width, content_height),
                        ),
                        engine,
                        theme,
                        profile,
                    )
                    .preferred()
                    .height()
                });
            if grows { 0 } else { measured_height }
        };
        let minimum = flexible.map_or(0, |(_, minimum)| minimum);
        let hint = flow::SizeHint::new(Size::new(width, minimum), Size::new(width, height));
        let item = if !scroll_axis && let Some((weight, _)) = flexible {
            flow::Item::weighted(hint, weight)
        } else if !scroll_axis && grows_vertical_space(child) {
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
        .align_items(node.style().align_items())
        .pressure(if scroll_axis {
            flow::Pressure::Overflow
        } else {
            flow::Pressure::Fit
        });

    for child in children {
        let flexible = child.style().width().and_then(view::Dimension::flexible);
        let width = if scroll_axis {
            scroll_axis_child_width(child, content_width, engine, theme, profile)
        } else if flexible.is_some() {
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
        let minimum = flexible.map_or(0, |(_, minimum)| minimum);
        let hint = flow::SizeHint::new(Size::new(minimum, height), Size::new(width, height));
        let item = if !scroll_axis && let Some((weight, _)) = flexible {
            flow::Item::weighted(hint, weight)
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
    let content = flow::inset_rect(rect, node.style().padding());
    let child_rects = node
        .children()
        .iter()
        .map(|child| {
            let width = resolved_width(child, content.width(), engine, theme, profile);
            let height =
                resolved_height_for_width(child, width, content.height(), engine, theme, profile);
            if let view::FloatingPlacement::Offset { x, y } = child.floating_placement() {
                return Rect::new(
                    content.x().saturating_add(x),
                    content.y().saturating_add(y),
                    width,
                    height,
                );
            }

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
        Some(view::Dimension::Flexible { .. }) => intrinsic_width(child, engine, theme, profile),
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
    retained: &composition::tree::Node,
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
    retained: &composition::tree::Node,
    path: &path::Path,
    rect: Rect,
    floating_layer: bool,
    clip: Option<Clip>,
    ctx: &mut LayoutContext<'_>,
) {
    let table_projection = ctx.table_projection.clone();
    let placement = vertical_stack_placement(
        node,
        rect,
        ctx.engine,
        ctx.theme,
        ctx.keymap,
        false,
        table_projection.as_ref(),
    );
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
    retained: &composition::tree::Node,
    path: &path::Path,
    rect: Rect,
    floating_layer: bool,
    clip: Option<Clip>,
    ctx: &mut LayoutContext<'_>,
) {
    let projection = ctx.table_projection.clone();
    let placement = projection
        .as_ref()
        .filter(|projection| table_stack_matches(node, projection))
        .map(|projection| table_stack_placement(node, rect, projection, ctx))
        .unwrap_or_else(|| {
            horizontal_stack_placement(node, rect, ctx.engine, ctx.theme, ctx.keymap, false)
        });
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

fn table_stack_matches(node: &view::Node, projection: &table::Projection) -> bool {
    node.children().first().is_some_and(|child| {
        child
            .table_header_cell()
            .is_some_and(|cell| cell.table() == projection.table())
            || child
                .table_cell()
                .is_some_and(|cell| cell.table() == projection.table())
    })
}

fn table_stack_intrinsic_height(
    node: &view::Node,
    projection: &table::Projection,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> Option<i32> {
    if !table_stack_matches(node, projection) {
        return None;
    }

    let content_height = node
        .children()
        .iter()
        .filter_map(|child| {
            let column = child
                .table_header_cell()
                .map(crate::table::HeaderCell::column)
                .or_else(|| child.table_cell().map(crate::table::Cell::column))?;
            let width = projection.column_width(column)?;
            Some(intrinsic_or_fixed_height_for_width(
                child, width, engine, theme, profile,
            ))
        })
        .max()?;

    Some(content_height.saturating_add(node.style().padding().vertical()))
}

fn table_stack_placement(
    node: &view::Node,
    rect: Rect,
    projection: &table::Projection,
    ctx: &mut LayoutContext<'_>,
) -> StackPlacement {
    let content_height = rect
        .height()
        .saturating_sub(node.style().padding().vertical());
    let content_y = rect.y().saturating_add(node.style().padding().top());
    let child_rects = node
        .children()
        .iter()
        .filter_map(|child| {
            let column = child
                .table_header_cell()
                .map(crate::table::HeaderCell::column)
                .or_else(|| child.table_cell().map(crate::table::Cell::column))?;
            let track_rect = projection.cell_rect(column, rect)?;
            let height = cross_axis_height_for_width(
                child,
                track_rect.width(),
                content_height,
                ctx.engine,
                node.style().align_items(),
                ctx.theme,
                ctx.keymap,
            );
            let y = cross_axis_offset(
                content_y,
                content_height,
                height,
                node.style().align_items(),
            );
            Some(Rect::new(track_rect.x(), y, track_rect.width(), height))
        })
        .collect::<Vec<_>>();

    StackPlacement {
        child_rects,
        content: Size::new(projection.content_width(), rect.height()),
    }
}

fn layout_overlay_stack(
    node: &view::Node,
    retained: &composition::tree::Node,
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
    retained: &composition::tree::Node,
    path: &path::Path,
    rect: Rect,
    floating_layer: bool,
    clip: Option<Clip>,
    ctx: &mut LayoutContext<'_>,
) {
    let mut x = rect.x;
    let height = rect.height.min(ctx.theme.menu().bar_height);

    for (index, child) in node.children().iter().enumerate() {
        let width = menu_title_width(child, ctx.engine, ctx.theme);
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
    retained: &composition::tree::Node,
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

        if node.auxiliary_hint().is_some()
            && let Some(child) = node.children().first()
        {
            let reserved = node.auxiliary_hint().map_or(0, |hint| {
                i32::from(hint.icon().is_some()).saturating_mul(
                    ctx.theme
                        .auxiliary_panel()
                        .icon_extent
                        .saturating_add(ctx.theme.auxiliary_panel().icon_gap),
                )
            });
            let text_rect = Rect::new(
                content.x().saturating_add(reserved),
                content.y(),
                content.width().saturating_sub(reserved),
                content.height(),
            );
            let child_height = intrinsic_height_for_width(
                child,
                text_rect.width(),
                ctx.engine,
                ctx.theme,
                ctx.keymap,
            )
            .min(text_rect.height());
            let child_rect = Rect::new(
                text_rect.x(),
                text_rect
                    .y()
                    .saturating_add(text_rect.height().saturating_sub(child_height) / 2),
                text_rect.width(),
                child_height,
            );
            layout_node(
                child,
                retained_child(retained, 0),
                path.child(0),
                child_rect,
                true,
                None,
                ctx,
            );
            return;
        }

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
    retained: &composition::tree::Node,
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

fn retained_child(parent: &composition::tree::Node, index: usize) -> &composition::tree::Node {
    parent
        .children()
        .get(index)
        .expect("composition tree must match view child order")
}
