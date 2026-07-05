use super::*;

#[test]
fn widget_closure_api_models_layout_controls_and_trigger_bindings() {
    let element = widget::Element::new()
        .layout(|layout| {
            layout
                .row()
                .gap(12)
                .padding(view::style::Padding::symmetric(8, 4))
                .align_items(view::style::Align::Center)
                .justify_content(view::style::Align::End)
        })
        .width(view::style::Dimension::grow())
        .height(view::style::Dimension::fixed(44));

    assert_eq!(element.layout_state().direction(), widget::Direction::Row);
    assert_eq!(element.layout_state().gap_value(), 12);
    assert_eq!(
        element.layout_state().padding_value(),
        view::style::Padding::symmetric(8, 4)
    );
    assert_eq!(element.width_state(), Some(view::style::Dimension::Grow));
    assert_eq!(
        element.height_state(),
        Some(view::style::Dimension::Fixed(44))
    );

    let styled_node = widget::Widget::into_node(element);
    assert_eq!(styled_node.style().gap(), 12);
    assert_eq!(styled_node.style().padding().left(), 8);
    assert_eq!(styled_node.style().padding().top(), 4);
    assert_eq!(
        styled_node.style().width(),
        Some(view::style::Dimension::Grow)
    );
    assert_eq!(
        styled_node.style().height(),
        Some(view::style::Dimension::Fixed(44))
    );
    assert_eq!(
        styled_node.style().align_items(),
        view::style::Align::Center
    );
    assert_eq!(
        styled_node.style().justify_content(),
        view::style::Align::End
    );

    let view = widget::view(|ui| {
        ui.column(|ui| {
            ui.label("Inspector");
            ui.button(widget::Button::new("Record").trigger::<RecordSource>(()));
            ui.checkbox(widget::Checkbox::new("Wrap", true).trigger::<RecordSource>(()));
            ui.radio(widget::Radio::new("Soft tabs", false).trigger::<RecordSource>(()));
            ui.slider(widget::Slider::new("Zoom", 1.0, 0.5..=2.0).on_change::<SetLevel>());
            ui.text_box(widget::TextBox::new("").placeholder("Find"));
        });
    });

    assert!(view.labels().contains(&"Inspector"));
    assert!(view.labels().contains(&"Record"));
    assert!(view.labels().contains(&"[x] Wrap"));
    assert!(view.labels().contains(&"( ) Soft tabs"));
    assert!(
        view.labels()
            .iter()
            .any(|label| label.starts_with("Zoom: 1.00"))
    );
    assert_eq!(view.bindings().len(), 4);
    assert_eq!(view.buttons().len(), 1);
    assert_eq!(view.buttons()[0].label(), "Record");
    assert_eq!(view.checkboxes().len(), 1);
    assert!(view.checkboxes()[0].checked());
    assert_eq!(view.radios().len(), 1);
    assert!(!view.radios()[0].selected());
    assert_eq!(view.sliders().len(), 1);
    assert_eq!(view.sliders()[0].label(), "Zoom");
    assert_eq!(view.sliders()[0].value(), 1.0);
    assert_eq!(view.text_areas().len(), 0);
    assert_eq!(view.text_boxes().len(), 1);
    assert_eq!(view.text_boxes()[0].text(), "");
    assert_eq!(view.text_boxes()[0].display_text(), "Find");
}

#[test]
fn widget_element_style_affects_row_layout_frames() {
    let view = widget::view(|ui| {
        ui.add(
            widget::Element::new()
                .row()
                .layout(|layout| {
                    layout
                        .gap(10)
                        .padding(view::style::Padding::symmetric(5, 2))
                        .align_items(view::style::Align::Center)
                })
                .children(|ui| {
                    ui.add(
                        widget::Element::new()
                            .label("Fixed")
                            .width(view::style::Dimension::fixed(50)),
                    );
                    ui.add(
                        widget::Element::new()
                            .label("Grow")
                            .width(view::style::Dimension::grow()),
                    );
                    ui.button(widget::Button::new("Fit"));
                }),
        );
    });

    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(300, 80), &mut layout_engine);
    let panels = layout.find_role(view::node::Role::Panel);
    let buttons = layout.find_role(view::node::Role::Button);

    assert_eq!(panels.len(), 2);
    assert_eq!(buttons.len(), 1);
    assert_eq!(panels[0].rect().x(), 5);
    assert_eq!(panels[0].rect().y(), 26);
    assert_eq!(panels[0].rect().width(), 50);
    assert_eq!(panels[1].rect().x(), 65);
    assert_eq!(panels[1].rect().y(), 26);
    assert!(panels[1].rect().width() > 100);
    assert_eq!(
        buttons[0].rect().x(),
        panels[1].rect().x() + panels[1].rect().width() + 10
    );
    assert_eq!(buttons[0].rect().y(), 26);
    assert_eq!(buttons[0].rect().height(), 28);
}

#[test]
fn widget_element_alignment_affects_layout_frames() {
    let view = widget::view(|ui| {
        ui.add(
            widget::Element::new()
                .row()
                .layout(|layout| {
                    layout
                        .gap(10)
                        .align_items(view::style::Align::Center)
                        .justify_content(view::style::Align::End)
                })
                .children(|ui| {
                    ui.add(
                        widget::Element::new()
                            .label("One")
                            .width(view::style::Dimension::fixed(40))
                            .height(view::style::Dimension::fixed(20)),
                    );
                    ui.add(
                        widget::Element::new()
                            .label("Two")
                            .width(view::style::Dimension::fixed(60))
                            .height(view::style::Dimension::fixed(30)),
                    );
                }),
        );
    });

    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(200, 100), &mut layout_engine);
    let panels = layout.find_role(view::node::Role::Panel);

    assert_eq!(panels.len(), 2);
    assert_eq!(panels[0].rect(), geometry::Rect::new(90, 40, 40, 20));
    assert_eq!(panels[1].rect(), geometry::Rect::new(140, 35, 60, 30));
}

#[test]
fn widget_element_style_affects_column_layout_frames() {
    let view = widget::view(|ui| {
        ui.add(
            widget::Element::new()
                .column()
                .layout(|layout| layout.gap(4).padding(view::style::Padding::all(6)))
                .children(|ui| {
                    ui.add(
                        widget::Element::new()
                            .label("Fixed")
                            .height(view::style::Dimension::fixed(20)),
                    );
                    ui.add(
                        widget::Element::new()
                            .label("Grow")
                            .height(view::style::Dimension::grow()),
                    );
                    ui.label("Fit");
                }),
        );
    });

    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(100, 100), &mut layout_engine);
    let panels = layout.find_role(view::node::Role::Panel);
    let labels = layout.find_role(view::node::Role::Label);

    assert_eq!(panels.len(), 2);
    assert_eq!(labels.len(), 1);
    assert_eq!(panels[0].rect().x(), 6);
    assert_eq!(panels[0].rect().y(), 6);
    assert_eq!(panels[0].rect().height(), 20);
    assert_eq!(panels[1].rect().y(), 30);
    assert_eq!(panels[1].rect().height(), 32);
    assert_eq!(labels[0].rect().y(), 66);
    assert_eq!(labels[0].rect().height(), 28);
}
