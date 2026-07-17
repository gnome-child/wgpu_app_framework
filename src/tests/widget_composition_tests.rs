use super::*;

struct LabeledField {
    label: String,
    text: String,
    focus: session::Focus,
}

impl LabeledField {
    fn new(label: impl Into<String>, text: impl Into<String>, focus: session::Focus) -> Self {
        Self {
            label: label.into(),
            text: text.into(),
            focus,
        }
    }
}

impl widget::Widget for LabeledField {
    fn into_node(self) -> view::Node {
        widget::Widget::into_node(
            widget::Element::new()
                .column()
                .layout(|layout| layout.gap(4))
                .child(widget::Label::new(self.label))
                .child(widget::TextBox::new(self.text).focus(self.focus)),
        )
    }
}

struct Progress {
    fraction: f32,
}

impl Progress {
    fn new(fraction: f32) -> Self {
        Self {
            fraction: fraction.clamp(0.0, 1.0),
        }
    }
}

impl widget::Widget for Progress {
    fn into_node(self) -> view::Node {
        widget::Widget::into_node(
            widget::Element::new()
                .row()
                .width(view::Dimension::fixed(200))
                .height(view::Dimension::fixed(8))
                .background(scene::Brush::solid(scene::Color::rgb(20, 20, 20)))
                .child(
                    widget::Element::new()
                        .width(view::Dimension::percent(self.fraction))
                        .height(view::Dimension::grow())
                        .background(scene::Brush::solid(scene::Color::rgb(80, 160, 240))),
                ),
        )
    }
}

#[test]
fn application_widgets_can_compose_labeled_fields_and_progress_from_public_pieces() {
    let focus = session::Focus::text("profile.name");
    let view = widget::view(|ui| {
        ui.add(
            widget::Element::new()
                .column()
                .layout(|layout| layout.align_items(view::Align::Start))
                .children(|ui| {
                    ui.add(LabeledField::new("Name", "Ada", focus));
                    ui.add(Progress::new(0.65));
                }),
        );
    });

    assert_eq!(view.text_boxes().len(), 1);
    assert_eq!(view.text_boxes()[0].text(), "Ada");
    assert!(view.labels().contains(&"Name"));

    let mut engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(240, 100), &mut engine);
    let stacks = layout.find_role(view::Role::Stack);
    let labels = layout.find_role(view::Role::Label);
    let text_boxes = layout.find_role(view::Role::TextBox);
    let panels = layout.find_role(view::Role::Panel);

    assert_eq!(labels.len(), 1);
    assert_eq!(text_boxes.len(), 1);
    assert!(text_boxes[0].rect().y() >= labels[0].rect().bottom() + 4);

    let stack_rects = stacks.iter().map(|frame| frame.rect()).collect::<Vec<_>>();
    let panel_rects = panels.iter().map(|frame| frame.rect()).collect::<Vec<_>>();
    let progress = stacks
        .into_iter()
        .find(|frame| frame.rect().width() == 200)
        .unwrap_or_else(|| panic!("progress track missing from {stack_rects:?}"));
    let fill = panels
        .into_iter()
        .find(|frame| frame.rect().width() == 130)
        .unwrap_or_else(|| panic!("progress fill missing from {panel_rects:?}"));

    assert_eq!(progress.rect().width(), 200);
    assert_eq!(fill.rect().width(), 130);

    let scene = scene::Scene::paint(&layout);
    assert!(scene.texts().iter().any(|text| text.value() == "Name"));
}

#[test]
fn scroll_builder_preserves_the_shared_element_recipe_and_scroll_identity() {
    let node = widget::Widget::into_node(
        crate::Scroll::new()
            .id("audit.scroll")
            .label("Audit scroll")
            .row()
            .layout(|layout| {
                layout
                    .gap(7)
                    .padding(view::Padding::symmetric(5, 3))
                    .align_items(view::Align::Center)
                    .justify_content(view::Align::End)
            })
            .width(view::Dimension::fixed(180))
            .height(view::Dimension::fixed(60))
            .max_height(72)
            .background(scene::Brush::solid(scene::Color::rgb(20, 30, 40)))
            .child(widget::Label::new("First"))
            .children(|ui| {
                ui.label("Second");
            }),
    );

    assert_eq!(node.role(), view::Role::Scroll);
    assert_eq!(node.axis(), Some(view::Axis::Horizontal));
    assert_eq!(node.id(), Some(interaction::Id::new("audit.scroll")));
    assert_eq!(node.label_text(), Some("Audit scroll"));
    assert_eq!(node.style().gap(), 7);
    assert_eq!(node.style().padding(), view::Padding::symmetric(5, 3));
    assert_eq!(node.style().align_items(), view::Align::Center);
    assert_eq!(node.style().justify_content(), view::Align::End);
    assert_eq!(node.style().width(), Some(view::Dimension::Fixed(180)));
    assert_eq!(node.style().height(), Some(view::Dimension::Fixed(60)));
    assert_eq!(node.style().max_height(), Some(72));
    assert!(node.style().background().is_some());
    assert_eq!(node.children().len(), 2);
    assert_eq!(node.children()[0].label_text(), Some("First"));
    assert_eq!(node.children()[1].label_text(), Some("Second"));
}

#[test]
fn fixed_trigger_controls_share_button_source_binding_semantics() {
    let view = widget::view(|ui| {
        ui.button(widget::Button::new("Button").trigger::<RecordSource>(()));
        ui.checkbox(widget::Checkbox::new("Checkbox", true).trigger::<RecordSource>(()));
        ui.radio(widget::Radio::new("Radio", false).trigger::<RecordSource>(()));
        ui.slider(widget::Slider::new("Slider", 0.5, 0.0..=1.0).trigger::<RecordSource>(()));
    });

    let bindings = view.bindings();
    assert_eq!(bindings.len(), 4);
    assert!(
        bindings
            .iter()
            .all(|binding| binding.source() == context::Source::Button)
    );
}
