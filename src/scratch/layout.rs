use std::{cell::RefCell, fmt, rc::Rc, time::Instant};

use crate::{
    geometry::{area, point},
    paint, text,
};

use super::{
    diagnostics,
    geometry::{Point, Rect, Size},
    interaction, view,
};

const MENU_BAR_HEIGHT: i32 = 28;
const ROW_HEIGHT: i32 = 28;
const SEPARATOR_HEIGHT: i32 = 1;
const MIN_MENU_WIDTH: i32 = 48;
const LABEL_PADDING: i32 = 24;
const POPUP_WIDTH: i32 = 220;

#[derive(Clone)]
pub struct Layout {
    size: Size,
    frames: Vec<Frame>,
}

pub struct Engine {
    text: TextService,
}

#[derive(Clone)]
pub(super) struct TextService {
    inner: Rc<RefCell<text::layout::Engine>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path(Vec<usize>);

#[derive(Clone)]
pub struct Frame {
    path: Path,
    role: view::Role,
    rect: Rect,
    label: Option<String>,
    text: Option<String>,
    text_wrap: Option<view::Wrap>,
    focused: bool,
    text_area_layout: Option<TextAreaLayout>,
    text_area: Option<view::TextArea>,
    text_box: Option<view::TextBox>,
    text_hit_map: Option<TextHitMap>,
    slider: Option<view::Slider>,
    target: Option<interaction::Target>,
    binding: Option<view::Binding>,
    action: Option<view::Action>,
}

#[derive(Clone)]
pub struct TextAreaLayout {
    layout: text::layout::TextFieldLayout,
    interaction_surfaces: Vec<text::layout::TextAreaSurface>,
    render_surfaces: Vec<text::layout::TextAreaSurface>,
    resolved_scroll: Option<interaction::ScrollOffset>,
}

#[derive(Clone)]
struct TextHitMap {
    boundaries: Vec<TextBoundary>,
}

#[derive(Clone)]
struct TextBoundary {
    index: usize,
    x: i32,
}

#[derive(Clone)]
pub struct Hit {
    frame: Frame,
}

impl Layout {
    pub fn compose(view: &view::View, size: Size, engine: &mut Engine) -> Self {
        let size = size.sanitized();
        let mut frames = Vec::new();
        layout_node(
            view.root(),
            Path::root(),
            Rect::from_size(size),
            engine,
            &mut frames,
        );

        Self { size, frames }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }

    pub fn hit_test(&self, point: Point) -> Option<Hit> {
        self.frames
            .iter()
            .rev()
            .find(|frame| frame.target.is_some() && frame.rect.contains(point))
            .cloned()
            .map(Hit::new)
    }

    pub fn find_role(&self, role: view::Role) -> Vec<&Frame> {
        self.frames
            .iter()
            .filter(|frame| frame.role == role)
            .collect()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            text: TextService::new(),
        }
    }

    pub(super) fn text_service(&self) -> TextService {
        self.text.clone()
    }

    fn label_width(&self, label: &str) -> i32 {
        self.text.label_width(label)
    }

    pub(super) fn take_text_diagnostics(&mut self) -> diagnostics::Text {
        self.text.take_diagnostics()
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl TextService {
    fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(text::layout::Engine::new())),
        }
    }

    fn label_width(&self, label: &str) -> i32 {
        let metrics = self.inner.borrow_mut().measure(
            &text::document::Document::plain(label),
            text::layout::Measure::unbounded(),
        );

        metrics.width().ceil().max(0.0) as i32 + LABEL_PADDING
    }

    fn text_width(&self, text: &str) -> i32 {
        let metrics = self.inner.borrow_mut().measure(
            &text::document::Document::plain(text),
            text::layout::Measure::unbounded(),
        );

        metrics.width().ceil().max(0.0) as i32
    }

    fn take_diagnostics(&self) -> diagnostics::Text {
        let mut engine = self.inner.borrow_mut();
        let mut diagnostics = diagnostics::Text::default();
        diagnostics.add_text_layout(engine.diagnostics());
        engine.reset_diagnostics();
        diagnostics
    }

    fn text_area_layout(&self, text_area: &view::TextArea, rect: Rect) -> TextAreaLayout {
        let area_model = text_area.area_model();
        let style =
            text::document::Style::default().with_color(paint::Color::rgb(0.10, 0.11, 0.13));
        let viewport = area::logical(rect.width() as f32, rect.height() as f32);
        let now = Instant::now();
        let mut state = text_area.view_state();
        let paint_layout = {
            let mut engine = self.inner.borrow_mut();
            if state.caret_visibility_pending() {
                state =
                    engine.ensure_caret_visible_for_area(&area_model, style, viewport, state, None);
            }
            let mut paint_layout = engine.text_area_paint_layout_for_area_at(
                &area_model,
                style,
                viewport,
                state.clone(),
                now,
            );
            let clamped_state =
                clamp_text_area_scroll_state(&state, paint_layout.layout(), viewport);
            if clamped_state.scroll_x() != state.scroll_x()
                || clamped_state.scroll_y() != state.scroll_y()
            {
                state = clamped_state;
                paint_layout = engine.text_area_paint_layout_for_area_at(
                    &area_model,
                    style,
                    viewport,
                    state.clone(),
                    now,
                );
            }
            paint_layout
        };
        let resolved_scroll = Some(scroll_offset_for_text_state(&state));
        let (layout, interaction_surfaces, render_surfaces) = paint_layout.into_projection_parts();

        TextAreaLayout {
            layout,
            interaction_surfaces,
            render_surfaces,
            resolved_scroll,
        }
    }

    fn text_area_position_at(
        &self,
        text_area: &view::TextArea,
        layout: &TextAreaLayout,
        rect: Rect,
        position: Point,
    ) -> Option<text::TextPosition> {
        let area_model = text_area.area_model();
        let local = point::logical(
            position.x().saturating_sub(rect.x()) as f32,
            position.y().saturating_sub(rect.y()) as f32,
        );

        self.inner
            .borrow_mut()
            .text_area_position_at_for_observed_surfaces(
                &area_model,
                local,
                text_area.view_state(),
                text_area.view_state().scroll_x(),
                layout.interaction_surfaces(),
            )
    }
}

impl fmt::Debug for TextService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextService").finish_non_exhaustive()
    }
}

impl text::edit::CaretMap for TextService {
    fn position_for_motion(
        &mut self,
        buffer: &text::Buffer,
        state: text::edit::State,
        motion: text::TextMotion,
    ) -> Option<text::TextPosition> {
        <text::layout::Engine as text::edit::CaretMap>::position_for_motion(
            &mut *self.inner.borrow_mut(),
            buffer,
            state,
            motion,
        )
    }
}

impl Path {
    fn root() -> Self {
        Self(Vec::new())
    }

    fn child(&self, index: usize) -> Self {
        let mut path = self.0.clone();
        path.push(index);
        Self(path)
    }

    pub fn indexes(&self) -> &[usize] {
        &self.0
    }
}

impl Frame {
    fn new(node: &view::Node, path: Path, rect: Rect, engine: &mut Engine) -> Self {
        let target = target_for(node, &path);
        let text_area = node.text_area_model();
        let text_area_layout =
            text_area.map(|text_area| engine.text.text_area_layout(text_area, rect));
        let text_box = node.text_box_model().cloned();
        Self {
            path,
            role: node.role(),
            rect,
            label: node
                .label_text()
                .or_else(|| node.binding().and_then(view::Binding::label))
                .map(str::to_owned),
            text: node
                .label_text()
                .is_none()
                .then(|| {
                    text_box
                        .as_ref()
                        .map(view::TextBox::display_text)
                        .map(str::to_owned)
                })
                .flatten(),
            text_wrap: node
                .text_area_model()
                .map(view::TextArea::wrap)
                .or_else(|| text_box.as_ref().map(|_| view::Wrap::None)),
            focused: node.is_focused(),
            text_area_layout,
            text_area: text_area.cloned(),
            text_hit_map: text_box
                .as_ref()
                .map(|text_box| TextHitMap::new(text_box.text(), engine)),
            text_box,
            slider: node.slider_model().cloned(),
            target,
            binding: node.binding().cloned(),
            action: action_for(node),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn role(&self) -> view::Role {
        self.role
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn label_text(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    pub fn text_wrap(&self) -> Option<view::Wrap> {
        self.text_wrap
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn text_area_layout(&self) -> Option<&TextAreaLayout> {
        self.text_area_layout.as_ref()
    }

    pub fn target(&self) -> Option<&interaction::Target> {
        self.target.as_ref()
    }

    pub fn action(&self) -> Option<&view::Action> {
        self.action.as_ref()
    }

    pub fn action_at(&self, point: Point) -> Option<view::Action> {
        if self.role == view::Role::Slider {
            let value = self.slider_value_at(point)?;
            if let Some(action) = self
                .binding
                .as_ref()
                .and_then(|binding| binding.slider_action(value))
            {
                return Some(action);
            }
        }

        if self.role == view::Role::TextBox {
            let text_box = self.text_box.as_ref()?;
            let hit_map = self.text_hit_map.as_ref()?;
            let local_x = point.x().saturating_sub(self.rect.x());
            let position = text::TextPosition::new(hit_map.index_at_x(local_x));

            return text_box.click_action(position);
        }

        self.action.clone()
    }

    pub fn action_at_with_engine(&self, point: Point, engine: &mut Engine) -> Option<view::Action> {
        if self.role == view::Role::TextArea {
            let text_area = self.text_area.as_ref()?;
            let layout = self.text_area_layout.as_ref()?;
            let position = engine
                .text
                .text_area_position_at(text_area, layout, self.rect, point)?;

            return text_area.click_action(position);
        }

        self.action_at(point)
    }

    pub fn drag_action_at_with_engine(
        &self,
        point: Point,
        engine: &mut Engine,
    ) -> Option<view::Action> {
        if self.role == view::Role::TextArea {
            let text_area = self.text_area.as_ref()?;
            let layout = self.text_area_layout.as_ref()?;
            let position = engine
                .text
                .text_area_position_at(text_area, layout, self.rect, point)?;

            return Some(text_area.drag_action(position));
        }

        self.action_at_with_engine(point, engine)
    }

    fn slider_value_at(&self, point: Point) -> Option<f64> {
        let slider = self.slider.as_ref()?;
        let width = self.rect.width().max(1) as f64;
        let offset = point.x().saturating_sub(self.rect.x()) as f64;

        Some(slider.value_at_fraction(offset / width))
    }
}

impl TextAreaLayout {
    pub fn layout(&self) -> &text::layout::TextFieldLayout {
        &self.layout
    }

    pub fn interaction_surfaces(&self) -> &[text::layout::TextAreaSurface] {
        &self.interaction_surfaces
    }

    pub fn render_surfaces(&self) -> &[text::layout::TextAreaSurface] {
        &self.render_surfaces
    }

    pub fn resolved_scroll(&self) -> Option<interaction::ScrollOffset> {
        self.resolved_scroll
    }
}

impl TextHitMap {
    fn new(text: &str, engine: &mut Engine) -> Self {
        let mut boundaries = Vec::with_capacity(text.chars().count() + 1);
        boundaries.push(TextBoundary { index: 0, x: 0 });

        for (index, _) in text.char_indices().skip(1) {
            boundaries.push(TextBoundary {
                index,
                x: engine.text.text_width(&text[..index]),
            });
        }

        boundaries.push(TextBoundary {
            index: text.len(),
            x: engine.text.text_width(text),
        });

        Self { boundaries }
    }

    fn index_at_x(&self, x: i32) -> usize {
        let Some(first) = self.boundaries.first() else {
            return 0;
        };
        if x <= first.x {
            return first.index;
        }

        for pair in self.boundaries.windows(2) {
            let left = &pair[0];
            let right = &pair[1];
            let midpoint = left.x.saturating_add(right.x.saturating_sub(left.x) / 2);
            if x < midpoint {
                return left.index;
            }
        }

        self.boundaries
            .last()
            .map(|boundary| boundary.index)
            .unwrap_or(0)
    }
}

impl Hit {
    fn new(frame: Frame) -> Self {
        Self { frame }
    }

    pub fn frame(&self) -> &Frame {
        &self.frame
    }

    pub fn target(&self) -> Option<&interaction::Target> {
        self.frame.target()
    }

    pub fn action(&self) -> Option<&view::Action> {
        self.frame.action()
    }

    pub fn action_at(&self, point: Point) -> Option<view::Action> {
        self.frame.action_at(point)
    }

    pub fn action_at_with_engine(&self, point: Point, engine: &mut Engine) -> Option<view::Action> {
        self.frame.action_at_with_engine(point, engine)
    }

    pub fn drag_action_at_with_engine(
        &self,
        point: Point,
        engine: &mut Engine,
    ) -> Option<view::Action> {
        self.frame.drag_action_at_with_engine(point, engine)
    }
}

fn layout_node(
    node: &view::Node,
    path: Path,
    rect: Rect,
    engine: &mut Engine,
    frames: &mut Vec<Frame>,
) {
    frames.push(Frame::new(node, path.clone(), rect, engine));

    match node.role() {
        view::Role::Root => layout_root(node, &path, rect, engine, frames),
        view::Role::Stack => match node.axis() {
            Some(view::Axis::Horizontal) => {
                layout_horizontal_stack(node, &path, rect, engine, frames)
            }
            Some(view::Axis::Vertical) | None => {
                layout_vertical_stack(node, &path, rect, engine, frames)
            }
        },
        view::Role::MenuBar => layout_menu_bar(node, &path, rect, engine, frames),
        view::Role::Popup => layout_popup(node, &path, rect, engine, frames),
        view::Role::Panel => layout_vertical_stack(node, &path, rect, engine, frames),
        view::Role::Menu
        | view::Role::Command
        | view::Role::Separator
        | view::Role::TextArea
        | view::Role::Button
        | view::Role::Checkbox
        | view::Role::Radio
        | view::Role::Slider
        | view::Role::TextBox
        | view::Role::Label => {}
    }
}

fn layout_root(
    node: &view::Node,
    path: &Path,
    rect: Rect,
    engine: &mut Engine,
    frames: &mut Vec<Frame>,
) {
    for (index, child) in node.children().iter().enumerate() {
        let child_path = path.child(index);
        if child.role() == view::Role::Popup {
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
    path: &Path,
    rect: Rect,
    engine: &mut Engine,
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
    path: &Path,
    rect: Rect,
    engine: &mut Engine,
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
        .filter(|child| matches!(child.style().width(), Some(view::Dimension::Grow)))
        .count()
        .max(1) as i32;
    let fixed_width: i32 = children
        .iter()
        .filter(|child| !matches!(child.style().width(), Some(view::Dimension::Grow)))
        .map(|child| resolved_row_width(child, rect.width, engine))
        .sum();
    let grow_width = (rect.width - fixed_width - gap_total).max(0) / grow_count;
    let widths = children
        .iter()
        .map(|child| {
            if matches!(child.style().width(), Some(view::Dimension::Grow)) {
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
    path: &Path,
    rect: Rect,
    engine: &mut Engine,
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
    path: &Path,
    rect: Rect,
    engine: &mut Engine,
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

fn action_for(node: &view::Node) -> Option<view::Action> {
    if let Some(binding) = node.binding() {
        return binding.is_enabled().then(|| binding.action());
    }

    match node.role() {
        view::Role::Menu => node.menu_action(),
        view::Role::TextArea => node
            .text_area_model()
            .and_then(view::TextArea::focus_action),
        view::Role::TextBox => node.text_box_model().and_then(view::TextBox::focus_action),
        view::Role::Root
        | view::Role::Stack
        | view::Role::MenuBar
        | view::Role::Command
        | view::Role::Separator
        | view::Role::Button
        | view::Role::Checkbox
        | view::Role::Radio
        | view::Role::Slider
        | view::Role::Panel
        | view::Role::Popup
        | view::Role::Label => None,
    }
}

fn target_for(node: &view::Node, path: &Path) -> Option<interaction::Target> {
    node.pointer_target_at_path(path.indexes())
}

fn intrinsic_width(node: &view::Node, engine: &mut Engine) -> i32 {
    let label_width = node
        .label_text()
        .map(|label| engine.label_width(label))
        .unwrap_or_default();

    match node.role() {
        view::Role::Menu => label_width.max(MIN_MENU_WIDTH),
        view::Role::Command | view::Role::Popup => label_width.max(POPUP_WIDTH),
        view::Role::Slider => label_width.max(160),
        view::Role::TextBox => label_width.max(120),
        _ => label_width.max(MIN_MENU_WIDTH),
    }
}

fn intrinsic_height(node: &view::Node) -> i32 {
    match node.role() {
        view::Role::MenuBar
        | view::Role::Menu
        | view::Role::Command
        | view::Role::Popup
        | view::Role::Button
        | view::Role::Checkbox
        | view::Role::Radio
        | view::Role::Slider
        | view::Role::TextBox => ROW_HEIGHT,
        view::Role::Separator => SEPARATOR_HEIGHT,
        view::Role::Label => ROW_HEIGHT,
        view::Role::TextArea | view::Role::Panel | view::Role::Root | view::Role::Stack => {
            ROW_HEIGHT
        }
    }
}

fn popup_height(node: &view::Node) -> i32 {
    let child_height: i32 = node.children().iter().map(intrinsic_height).sum();
    child_height.max(ROW_HEIGHT)
}

fn gap_total(gap: i32, child_count: usize) -> i32 {
    gap.saturating_mul(child_count.saturating_sub(1) as i32)
}

fn grows_vertical_space(node: &view::Node) -> bool {
    match node.style().height() {
        Some(view::Dimension::Grow) => true,
        Some(view::Dimension::Fit | view::Dimension::Fixed(_) | view::Dimension::Percent(_)) => {
            false
        }
        None => matches!(
            node.role(),
            view::Role::TextArea | view::Role::Panel | view::Role::Stack
        ),
    }
}

fn resolved_width(node: &view::Node, parent_width: i32, engine: &mut Engine) -> i32 {
    match node.style().width() {
        Some(view::Dimension::Fit) => intrinsic_width(node, engine),
        Some(view::Dimension::Grow) | None => parent_width,
        Some(view::Dimension::Fixed(width)) => width,
        Some(view::Dimension::Percent(percent)) => {
            ((parent_width.max(0) as f32) * percent).round() as i32
        }
    }
    .clamp(0, parent_width.max(0))
}

fn resolved_row_width(node: &view::Node, parent_width: i32, engine: &mut Engine) -> i32 {
    match node.style().width() {
        None | Some(view::Dimension::Fit) => intrinsic_width(node, engine),
        Some(view::Dimension::Grow) => parent_width,
        Some(view::Dimension::Fixed(width)) => width,
        Some(view::Dimension::Percent(percent)) => {
            ((parent_width.max(0) as f32) * percent).round() as i32
        }
    }
    .clamp(0, parent_width.max(0))
}

fn cross_axis_width(
    node: &view::Node,
    parent_width: i32,
    engine: &mut Engine,
    align: view::Align,
) -> i32 {
    match align {
        view::Align::Stretch => resolved_width(node, parent_width, engine),
        view::Align::Start | view::Align::Center | view::Align::End => match node.style().width() {
            None | Some(view::Dimension::Fit) => intrinsic_width(node, engine),
            Some(view::Dimension::Grow) => parent_width,
            Some(view::Dimension::Fixed(width)) => width,
            Some(view::Dimension::Percent(percent)) => {
                ((parent_width.max(0) as f32) * percent).round() as i32
            }
        }
        .clamp(0, parent_width.max(0)),
    }
}

fn cross_axis_height(node: &view::Node, parent_height: i32, align: view::Align) -> i32 {
    match align {
        view::Align::Stretch => match node.style().height() {
            None | Some(view::Dimension::Grow) => parent_height,
            Some(view::Dimension::Fit) => intrinsic_height(node),
            Some(view::Dimension::Fixed(height)) => height,
            Some(view::Dimension::Percent(percent)) => {
                ((parent_height.max(0) as f32) * percent).round() as i32
            }
        },
        view::Align::Start | view::Align::Center | view::Align::End => {
            match node.style().height() {
                None | Some(view::Dimension::Fit) => intrinsic_height(node),
                Some(view::Dimension::Grow) => parent_height,
                Some(view::Dimension::Fixed(height)) => height,
                Some(view::Dimension::Percent(percent)) => {
                    ((parent_height.max(0) as f32) * percent).round() as i32
                }
            }
        }
    }
    .clamp(0, parent_height.max(0))
}

fn main_axis_offset(align: view::Align, available: i32, content: i32) -> i32 {
    let slack = available.saturating_sub(content);
    match align {
        view::Align::Start | view::Align::Stretch => 0,
        view::Align::Center => slack / 2,
        view::Align::End => slack,
    }
}

fn cross_axis_offset(origin: i32, available: i32, size: i32, align: view::Align) -> i32 {
    origin.saturating_add(main_axis_offset(align, available, size))
}

fn inset_rect(rect: Rect, padding: view::Padding) -> Rect {
    let x = rect.x.saturating_add(padding.left());
    let y = rect.y.saturating_add(padding.top());
    let width = rect.width.saturating_sub(padding.horizontal());
    let height = rect.height.saturating_sub(padding.vertical());
    Rect::new(x, y, width, height)
}

fn resolved_height(node: &view::Node, parent_height: i32) -> i32 {
    match node.style().height() {
        Some(view::Dimension::Fit) => intrinsic_height(node),
        Some(view::Dimension::Grow) | None => {
            if grows_vertical_space(node) {
                parent_height
            } else {
                intrinsic_height(node)
            }
        }
        Some(view::Dimension::Fixed(height)) => height,
        Some(view::Dimension::Percent(percent)) => {
            ((parent_height.max(0) as f32) * percent).round() as i32
        }
    }
    .clamp(0, parent_height.max(0))
}

fn scroll_offset_for_text_state(state: &text::TextViewState) -> interaction::ScrollOffset {
    interaction::ScrollOffset::new(
        scroll_component(state.scroll_x()),
        scroll_component(state.scroll_y()),
    )
}

fn clamp_text_area_scroll_state(
    state: &text::TextViewState,
    layout: &text::layout::TextFieldLayout,
    viewport: area::Logical,
) -> text::TextViewState {
    let content_area = layout.content_area();
    let max_scroll_x = (content_area.width() - viewport.width()).max(0.0);
    let max_scroll_y = (content_area.height() - viewport.height()).max(0.0);

    state.clone().with_scroll(
        state.scroll_x().clamp(0.0, max_scroll_x),
        state.scroll_y().clamp(0.0, max_scroll_y),
    )
}

fn scroll_component(value: f32) -> i32 {
    value.round().clamp(0.0, i32::MAX as f32) as i32
}
