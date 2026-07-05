use super::super::{
    context,
    geometry::{Point, Rect},
    interaction, view,
};
use super::{
    engine, path,
    text::{TextAreaLayout, TextHitMap},
};

#[derive(Clone)]
pub struct Frame {
    path: path::Path,
    role: view::node::Role,
    rect: Rect,
    label: Option<String>,
    text: Option<String>,
    text_wrap: Option<view::control::Wrap>,
    focused: bool,
    text_area_layout: Option<TextAreaLayout>,
    text_area: Option<view::control::TextArea>,
    text_box: Option<view::control::TextBox>,
    text_hit_map: Option<TextHitMap>,
    slider: Option<view::control::Slider>,
    target: Option<interaction::Target>,
    binding: Option<view::Binding>,
    action: Option<view::Action>,
}

impl Frame {
    pub(super) fn new(
        node: &view::Node,
        path: path::Path,
        rect: Rect,
        engine: &mut engine::Engine,
    ) -> Self {
        let target = target_for(node, &path);
        let text_area = node.text_area_model();
        let text_area_layout = text_area.map(|text_area| engine.text_area_layout(text_area, rect));
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
                        .map(view::control::TextBox::display_text)
                        .map(str::to_owned)
                })
                .flatten(),
            text_wrap: node
                .text_area_model()
                .map(view::control::TextArea::wrap)
                .or_else(|| text_box.as_ref().map(|_| view::control::Wrap::None)),
            focused: node.is_focused(),
            text_area_layout,
            text_area: text_area.cloned(),
            text_hit_map: text_box
                .as_ref()
                .map(|text_box| engine.text_hit_map(text_box.text())),
            text_box,
            slider: node.slider_model().cloned(),
            target,
            binding: node.binding().cloned(),
            action: action_for(node),
        }
    }

    pub fn path(&self) -> &path::Path {
        &self.path
    }

    pub fn role(&self) -> view::node::Role {
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

    pub fn text_wrap(&self) -> Option<view::control::Wrap> {
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

    pub(in crate::scratch) fn binding_source(&self) -> Option<context::Source> {
        self.binding.as_ref().map(view::Binding::source)
    }

    pub(super) fn target_is_some(&self) -> bool {
        self.target.is_some()
    }

    pub fn action_at(&self, point: Point) -> Option<view::Action> {
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

        if self.role == view::node::Role::TextBox {
            let text_box = self.text_box.as_ref()?;
            let hit_map = self.text_hit_map.as_ref()?;
            let local_x = point.x().saturating_sub(self.rect.x());
            let position = hit_map.position_at_x(local_x);

            return text_box.click_action(position);
        }

        self.action.clone()
    }

    pub fn action_at_with_engine(
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

        self.action_at(point)
    }

    pub fn drag_action_at_with_engine(
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

        self.action_at_with_engine(point, engine)
    }

    fn slider_value_at(&self, point: Point) -> Option<f64> {
        let slider = self.slider.as_ref()?;
        let width = self.rect.width().max(1) as f64;
        let offset = point.x().saturating_sub(self.rect.x()) as f64;

        Some(slider.value_at_fraction(offset / width))
    }
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
        | view::node::Role::Panel
        | view::node::Role::Popup
        | view::node::Role::Label => None,
    }
}

fn target_for(node: &view::Node, path: &path::Path) -> Option<interaction::Target> {
    node.pointer_target_at_path(path.indexes())
}
