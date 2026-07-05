use super::super::{
    context,
    geometry::{Point, Rect},
    interaction,
    theme::Theme,
    view,
};
use super::{control, engine, path, text};

#[derive(Clone)]
pub struct Frame {
    path: path::Path,
    role: view::node::Role,
    rect: Rect,
    label: Option<String>,
    text: Option<String>,
    text_wrap: Option<view::control::Wrap>,
    focused: bool,
    text_area_layout: Option<text::Area>,
    text_box_layout: Option<text::Field>,
    checkbox: Option<view::control::Checkbox>,
    radio: Option<view::control::Radio>,
    text_area: Option<view::control::TextArea>,
    text_box: Option<view::control::TextBox>,
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
        let checkbox = node.checkbox_model().cloned();
        let radio = node.radio_model().cloned();
        let text_box = node.text_box_model().cloned();
        let text_box_layout = text_box
            .as_ref()
            .map(|text_box| engine.text_field_layout(text_box, text_box_text_rect_for(rect)));
        Self {
            path,
            role: node.role(),
            rect,
            label: label_for(node).map(str::to_owned),
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
            text_box_layout,
            checkbox,
            radio,
            text_area: text_area.cloned(),
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

    pub fn is_enabled(&self) -> bool {
        self.binding.as_ref().is_none_or(view::Binding::is_enabled)
    }

    pub(in crate::scratch) fn checkbox(&self) -> Option<&view::control::Checkbox> {
        self.checkbox.as_ref()
    }

    pub(in crate::scratch) fn radio(&self) -> Option<&view::control::Radio> {
        self.radio.as_ref()
    }

    pub(in crate::scratch) fn slider(&self) -> Option<&view::control::Slider> {
        self.slider.as_ref()
    }

    pub(in crate::scratch) fn text_box(&self) -> Option<&view::control::TextBox> {
        self.text_box.as_ref()
    }

    pub fn text_area_layout(&self) -> Option<&text::Area> {
        self.text_area_layout.as_ref()
    }

    pub fn text_box_layout(&self) -> Option<&text::Field> {
        self.text_box_layout.as_ref()
    }

    pub(in crate::scratch) fn text_box_text_rect(&self) -> Rect {
        text_box_text_rect_for(self.rect)
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

        if self.role == view::node::Role::TextBox {
            let text_box = self.text_box.as_ref()?;
            let layout = self.text_box_layout.as_ref()?;
            let text_rect = self.text_box_text_rect();
            let position = engine.text_field_position_at(text_box, layout, text_rect, point)?;

            return text_box.click_action(position);
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
        let theme = Theme::default();
        let fraction = control::slider_fraction_at(self.rect, theme.metrics(), point);

        Some(slider.value_at_fraction(fraction))
    }
}

fn text_box_text_rect_for(rect: Rect) -> Rect {
    let padding_x = Theme::default().metrics().text_box_padding_x;
    Rect::new(
        rect.x().saturating_add(padding_x),
        rect.y(),
        rect.width().saturating_sub(padding_x.saturating_mul(2)),
        rect.height(),
    )
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
        | view::node::Role::Panel
        | view::node::Role::Popup
        | view::node::Role::Label => None,
    }
}

fn target_for(node: &view::Node, path: &path::Path) -> Option<interaction::Target> {
    node.pointer_target_at_path(path.indexes())
}
