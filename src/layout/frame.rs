use super::super::{
    composition, context,
    geometry::{Point, Rect},
    interaction, keymap, scene,
    theme::Theme,
    view,
};
use super::{control, engine, measure, path, text, typography, viewport};
use crate::animation;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Clip {
    rect: Rect,
    rounding: scene::Rounding,
}

#[derive(Clone)]
pub(crate) struct ShortcutPart {
    run: keymap::ShortcutRun,
    width: i32,
}

impl ShortcutPart {
    pub(crate) fn run(&self) -> &keymap::ShortcutRun {
        &self.run
    }

    pub(crate) fn width(&self) -> i32 {
        self.width
    }
}

pub(super) struct Input<'a> {
    pub(super) node: &'a view::Node,
    pub(super) node_id: composition::NodeId,
    pub(super) path: path::Path,
    pub(super) rect: Rect,
    pub(super) floating_layer: bool,
    pub(super) clip: Option<Clip>,
    pub(super) animation_frame: animation::Frame,
    pub(super) keymap: keymap::Profile,
}

#[derive(Clone)]
pub(crate) struct Frame {
    node_id: composition::NodeId,
    path: path::Path,
    role: view::node::Role,
    rect: Rect,
    active_rect: Rect,
    label: Option<String>,
    label_width: i32,
    text: Option<String>,
    text_wrap: Option<view::control::Wrap>,
    focused: bool,
    focus_visible: bool,
    #[cfg(test)]
    pressed: bool,
    #[cfg(test)]
    active: bool,
    selected: bool,
    floating_layer: bool,
    background: Option<scene::Brush>,
    clip: Option<Clip>,
    viewport: Option<viewport::Viewport>,
    text_area_layout: Option<text::Area>,
    text_box_layout: Option<text::Field>,
    text_box_text_rect: Rect,
    slider_track_rect: Option<Rect>,
    checkbox: Option<view::control::Checkbox>,
    radio: Option<view::control::Radio>,
    text_area: Option<view::control::TextArea>,
    text_box: Option<view::control::TextBox>,
    slider: Option<view::control::Slider>,
    target: Option<interaction::Target>,
    binding: Option<view::Binding>,
    action: Option<view::Action>,
    shortcut_width: Option<i32>,
    shortcut_content_width: i32,
    shortcut_display: Option<Vec<ShortcutPart>>,
}

impl Frame {
    pub(super) fn new(input: Input<'_>, engine: &mut engine::Engine, theme: &Theme) -> Self {
        let Input {
            node,
            node_id,
            path,
            rect,
            floating_layer,
            clip,
            animation_frame,
            keymap,
        } = input;
        let target = target_for(node, node_id);
        let binding = node.binding().cloned();
        let text_area = node.text_area_model();
        let now = animation_frame.now();
        let text_area_layout =
            text_area.map(|text_area| engine.text_area_layout(text_area, rect, theme, now));
        let viewport = text_area_layout.as_ref().map(text::Area::viewport);
        let checkbox = node.checkbox_model().cloned();
        let radio = node.radio_model().cloned();
        let text_box = node.text_box_model().cloned();
        let text_box_text_rect = text_box_text_rect_for(rect, theme);
        let text_box_layout = text_box
            .as_ref()
            .map(|text_box| engine.text_field_layout(text_box, text_box_text_rect, theme, now));
        let label = label_for(node).map(str::to_owned);
        let label_width = label
            .as_deref()
            .map(|label| match node.role() {
                view::node::Role::Menu => {
                    engine.label_width_with_style(label, typography::interface_text_style(theme))
                }
                view::node::Role::SectionHeader => engine.label_width_with_style(
                    &typography::section_header_text(label),
                    typography::section_header_style(theme),
                ),
                view::node::Role::Binding
                | view::node::Role::Button
                | view::node::Role::Checkbox
                | view::node::Role::Radio
                | view::node::Role::Slider
                | view::node::Role::TextBox => {
                    engine.label_width_with_style(label, typography::interface_text_style(theme))
                }
                view::node::Role::Label
                    if node
                        .binding()
                        .is_some_and(|binding| binding.source() == context::Source::Palette) =>
                {
                    engine.label_width_with_style(label, typography::interface_text_style(theme))
                }
                _ => engine.label_width_with_style(label, theme.typography().body()),
            })
            .unwrap_or_default();
        let shortcut_display = binding
            .as_ref()
            .and_then(view::Binding::shortcut)
            .map(|shortcut| shortcut.display_parts(keymap, theme.shortcuts().display()));
        let (shortcut_display, shortcut_content_width) = shortcut_display
            .map(|display| {
                let mut width = 0_i32;
                let mut parts = Vec::with_capacity(display.runs().len());
                for (index, run) in display.runs().iter().cloned().enumerate() {
                    if index > 0 {
                        width = width.saturating_add(typography::shortcut_run_gap(theme));
                    }
                    let run_width = measure::shortcut_run_width(&run, engine, theme);
                    width = width.saturating_add(run_width);
                    parts.push(ShortcutPart {
                        run,
                        width: run_width,
                    });
                }

                (Some(parts), width)
            })
            .unwrap_or((None, 0));
        let shortcut_width = shortcut_display.as_ref().map(|_| shortcut_content_width);
        let slider = node.slider_model().cloned();
        let slider_track_rect = slider
            .as_ref()
            .map(|_| control::slider_track_rect(rect, label_width, theme));
        let active_rect = active_rect_for(node, rect, slider.as_ref(), label_width, theme);
        Self {
            path,
            node_id,
            role: node.role(),
            rect,
            active_rect,
            label,
            label_width,
            text: node
                .label_text()
                .is_none()
                .then(|| {
                    text_box
                        .as_ref()
                        .map(view::control::TextBox::display_text)
                        .map(str::to_owned)
                })
                .flatten(),
            text_wrap: node
                .text_area_model()
                .map(view::control::TextArea::wrap)
                .or_else(|| text_box.as_ref().map(|_| view::control::Wrap::None)),
            focused: node.is_focused(),
            focus_visible: node.focus_visible(),
            #[cfg(test)]
            pressed: node.is_pressed(),
            #[cfg(test)]
            active: node.is_active(),
            selected: node.is_selected(),
            floating_layer,
            background: node.style().background(),
            clip,
            viewport,
            text_area_layout,
            text_box_layout,
            text_box_text_rect,
            slider_track_rect,
            checkbox,
            radio,
            text_area: text_area.cloned(),
            text_box,
            slider,
            target,
            binding,
            action: action_for(node),
            shortcut_width,
            shortcut_content_width,
            shortcut_display,
        }
    }

    pub(super) fn with_viewport(mut self, viewport: viewport::Viewport) -> Self {
        self.viewport = Some(viewport);
        self
    }

    pub(super) fn with_shortcut_width(mut self, width: i32) -> Self {
        self.shortcut_width = Some(width.max(0));
        self
    }

    pub(crate) fn path(&self) -> &path::Path {
        &self.path
    }

    pub(crate) fn node_id(&self) -> composition::NodeId {
        self.node_id
    }

    pub(crate) fn role(&self) -> view::node::Role {
        self.role
    }

    pub(crate) fn rect(&self) -> Rect {
        self.rect
    }

    pub(crate) fn active_rect(&self) -> Rect {
        self.active_rect
    }

    pub(crate) fn label_text(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub(crate) fn label_width(&self) -> i32 {
        self.label_width
    }

    pub(crate) fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    pub(crate) fn text_wrap(&self) -> Option<view::control::Wrap> {
        self.text_wrap
    }

    pub(crate) fn is_focused(&self) -> bool {
        self.focused
    }

    pub(crate) fn focus_visible(&self) -> bool {
        self.focus_visible
    }

    #[cfg(test)]
    pub(crate) fn is_pressed(&self) -> bool {
        self.pressed
    }

    #[cfg(test)]
    pub(crate) fn is_active(&self) -> bool {
        self.active
    }

    pub(crate) fn is_selected(&self) -> bool {
        self.selected
    }

    pub(crate) fn is_floating_layer(&self) -> bool {
        self.floating_layer
    }

    pub(crate) fn background(&self) -> Option<scene::Brush> {
        self.background
    }

    pub(crate) fn clip(&self) -> Option<Clip> {
        self.clip
    }

    pub(crate) fn viewport(&self) -> Option<viewport::Viewport> {
        self.viewport
    }

    pub(crate) fn resolved_scroll(&self) -> Option<interaction::ScrollOffset> {
        self.viewport.map(viewport::Viewport::resolved_scroll)
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.binding.as_ref().is_none_or(view::Binding::is_enabled)
    }

    pub(crate) fn checked(&self) -> Option<bool> {
        self.binding.as_ref().and_then(view::Binding::checked)
    }

    pub(crate) fn shortcut_display(&self) -> Option<&[ShortcutPart]> {
        self.shortcut_display.as_deref()
    }

    pub(crate) fn shortcut_width(&self) -> i32 {
        self.shortcut_width.unwrap_or_default()
    }

    pub(crate) fn shortcut_content_width(&self) -> i32 {
        self.shortcut_content_width
    }

    pub(crate) fn checkbox(&self) -> Option<&view::control::Checkbox> {
        self.checkbox.as_ref()
    }

    pub(crate) fn radio(&self) -> Option<&view::control::Radio> {
        self.radio.as_ref()
    }

    pub(crate) fn slider(&self) -> Option<&view::control::Slider> {
        self.slider.as_ref()
    }

    pub(crate) fn slider_track_rect(&self) -> Option<Rect> {
        self.slider_track_rect
    }

    pub(crate) fn text_box(&self) -> Option<&view::control::TextBox> {
        self.text_box.as_ref()
    }

    pub(crate) fn text_area(&self) -> Option<&view::control::TextArea> {
        self.text_area.as_ref()
    }

    pub(crate) fn text_area_layout(&self) -> Option<&text::Area> {
        self.text_area_layout.as_ref()
    }

    pub(crate) fn text_box_layout(&self) -> Option<&text::Field> {
        self.text_box_layout.as_ref()
    }

    pub(crate) fn text_box_text_rect(&self) -> Rect {
        self.text_box_text_rect
    }

    pub(crate) fn target(&self) -> Option<&interaction::Target> {
        self.target.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn action(&self) -> Option<&view::Action> {
        self.action.as_ref()
    }

    pub(crate) fn binding_source(&self) -> Option<context::Source> {
        self.binding.as_ref().map(view::Binding::source)
    }

    pub(crate) fn clip_contains(&self, point: Point) -> bool {
        self.clip.is_none_or(|clip| clip.contains(point))
    }

    pub(super) fn accepts_hit(&self, point: Point) -> bool {
        self.target.is_some() && self.active_rect.contains(point) && self.clip_contains(point)
    }

    pub(crate) fn action_at(&self, point: Point) -> Option<view::Action> {
        if self.role == view::node::Role::Slider {
            let value = self.slider_value_at(point)?;
            if let Some(action) = self
                .binding
                .as_ref()
                .and_then(|binding| binding.slider_action(value))
            {
                return Some(action);
            }
        }

        self.action.clone()
    }

    pub(crate) fn action_at_with_engine(
        &self,
        point: Point,
        engine: &mut engine::Engine,
    ) -> Option<view::Action> {
        if self.role == view::node::Role::TextArea {
            let text_area = self.text_area.as_ref()?;
            let layout = self.text_area_layout.as_ref()?;
            let position = engine.text_area_position_at(text_area, layout, self.rect, point)?;

            return text_area.click_action(position);
        }

        if self.role == view::node::Role::TextBox {
            let text_box = self.text_box.as_ref()?;
            let layout = self.text_box_layout.as_ref()?;
            let text_rect = self.text_box_text_rect();
            let position = engine.text_field_position_at(text_box, layout, text_rect, point)?;

            return text_box.click_action(position);
        }

        self.action_at(point)
    }

    pub(crate) fn drag_action_at_with_engine(
        &self,
        point: Point,
        engine: &mut engine::Engine,
    ) -> Option<view::Action> {
        if self.role == view::node::Role::TextArea {
            let text_area = self.text_area.as_ref()?;
            let layout = self.text_area_layout.as_ref()?;
            let position = engine.text_area_position_at(text_area, layout, self.rect, point)?;

            return Some(text_area.drag_action(position));
        }

        if self.role == view::node::Role::TextBox {
            let text_box = self.text_box.as_ref()?;
            let layout = self.text_box_layout.as_ref()?;
            let text_rect = self.text_box_text_rect();
            let position = engine.text_field_position_at(text_box, layout, text_rect, point)?;

            return Some(text_box.drag_action(position));
        }

        self.action_at_with_engine(point, engine)
    }

    fn slider_value_at(&self, point: Point) -> Option<f64> {
        let slider = self.slider.as_ref()?;
        let track = self.slider_track_rect?;
        let width = track.width().max(1) as f64;
        let offset = point.x().saturating_sub(track.x()) as f64;
        let fraction = offset / width;

        Some(slider.value_at_fraction(fraction))
    }
}

impl Clip {
    pub(super) fn new(rect: Rect) -> Self {
        Self {
            rect,
            rounding: scene::Rounding::none(),
        }
    }

    pub(super) fn rounded(rect: Rect, rounding: scene::Rounding) -> Self {
        Self { rect, rounding }
    }

    pub(crate) fn rect(self) -> Rect {
        self.rect
    }

    pub(crate) fn rounding(self) -> scene::Rounding {
        self.rounding
    }

    pub(crate) fn contains(self, point: Point) -> bool {
        self.rect.contains(point)
    }
}

fn text_box_text_rect_for(rect: Rect, theme: &Theme) -> Rect {
    let padding_x = theme.text_input().padding_x;
    Rect::new(
        rect.x().saturating_add(padding_x),
        rect.y(),
        rect.width().saturating_sub(padding_x.saturating_mul(2)),
        rect.height(),
    )
}

fn active_rect_for(
    node: &view::Node,
    rect: Rect,
    slider: Option<&view::control::Slider>,
    label_width: i32,
    theme: &Theme,
) -> Rect {
    match node.role() {
        view::node::Role::Checkbox | view::node::Role::Radio => {
            control::choice_mark_rect(rect, theme)
        }
        view::node::Role::Slider => slider
            .map(|slider| control::slider_active_rect(rect, slider, label_width, theme))
            .unwrap_or(rect),
        _ => rect,
    }
}

fn label_for(node: &view::Node) -> Option<&str> {
    node.label_text().or_else(|| {
        (node.role() == view::node::Role::Binding)
            .then(|| node.binding().and_then(view::Binding::label))
            .flatten()
    })
}

fn action_for(node: &view::Node) -> Option<view::Action> {
    if let Some(binding) = node.binding() {
        return binding.is_enabled().then(|| binding.action());
    }

    match node.role() {
        view::node::Role::Menu => node.menu_action(),
        view::node::Role::TextArea => node
            .text_area_model()
            .and_then(view::control::TextArea::focus_action),
        view::node::Role::TextBox => node
            .text_box_model()
            .and_then(view::control::TextBox::focus_action),
        view::node::Role::Root
        | view::node::Role::Stack
        | view::node::Role::MenuBar
        | view::node::Role::Binding
        | view::node::Role::Separator
        | view::node::Role::Button
        | view::node::Role::Checkbox
        | view::node::Role::Radio
        | view::node::Role::Slider
        | view::node::Role::Scroll
        | view::node::Role::Panel
        | view::node::Role::FloatingPanel
        | view::node::Role::SectionHeader
        | view::node::Role::Label => None,
    }
}

fn target_for(node: &view::Node, node_id: composition::NodeId) -> Option<interaction::Target> {
    node.node_pointer_target(node_id)
}
