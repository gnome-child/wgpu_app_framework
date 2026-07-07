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
    assert!(view.labels().contains(&"Wrap"));
    assert!(view.labels().contains(&"Soft tabs"));
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

    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(300, 80), &mut layout_engine);
    let panels = layout.find_role(view::node::Role::Panel);
    let buttons = layout.find_role(view::node::Role::Button);

    assert_eq!(panels.len(), 2);
    assert_eq!(buttons.len(), 1);
    assert_eq!(panels[0].rect().x(), 5);
    assert_eq!(panels[0].rect().width(), 50);
    assert_eq!(panels[1].rect().x(), 65);
    assert!(panels[1].rect().width() > 100);
    assert_eq!(
        buttons[0].rect().x(),
        panels[1].rect().x() + panels[1].rect().width() + 10
    );
    assert_eq!(
        buttons[0].rect().height(),
        Theme::default().control().height
    );
    let content_y = 2;
    let content_height = 80 - 4;
    assert_eq!(
        panels[0].rect().y(),
        content_y + (content_height - panels[0].rect().height()) / 2
    );
    assert_eq!(
        panels[1].rect().y(),
        content_y + (content_height - panels[1].rect().height()) / 2
    );
    assert_eq!(
        buttons[0].rect().y(),
        content_y + (content_height - buttons[0].rect().height()) / 2
    );
}

#[test]
fn row_layout_fits_children_unless_width_grows() {
    let view = widget::view(|ui| {
        ui.add(
            widget::Element::new()
                .row()
                .layout(|layout| layout.gap(6))
                .children(|ui| {
                    ui.button(widget::Button::new("Tiny"));
                    ui.button(widget::Button::new("Much wider"));
                }),
        );
    });

    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(300, 80), &mut layout_engine);
    let buttons = layout.find_role(view::node::Role::Button);

    assert_eq!(buttons.len(), 2);
    assert!(buttons[0].rect().width() < buttons[1].rect().width());
    assert_eq!(buttons[1].rect().x(), buttons[0].rect().right() + 6);
    assert!(buttons[1].rect().right() < 300);
}

#[test]
fn fit_height_text_uses_height_for_allocated_width() {
    let text = "Alpha beta gamma delta epsilon zeta eta theta";
    let view = widget::view(|ui| {
        ui.add(
            widget::Element::new()
                .row()
                .layout(|layout| layout.align_items(view::style::Align::Start))
                .children(|ui| {
                    ui.add(
                        widget::Element::new()
                            .label(text)
                            .width(view::style::Dimension::fixed(90))
                            .height(view::style::Dimension::fit()),
                    );
                }),
        );
    });

    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(300, 160), &mut layout_engine);
    let label = layout
        .find_role(view::node::Role::Panel)
        .into_iter()
        .find(|frame| frame.label_text() == Some(text))
        .expect("fit label should be laid out");
    let scene = scene::Scene::paint(&layout);
    let painted = scene
        .texts()
        .into_iter()
        .find(|painted| painted.value() == text)
        .expect("fit label should paint");

    assert_eq!(label.rect().width(), 90);
    assert!(label.rect().height() > Theme::default().menu().row_height);
    assert_eq!(painted.rect(), label.rect());
    assert_eq!(painted.wrap(), scene::TextWrap::WordOrGlyph);
}

#[test]
fn button_reserved_labels_stabilize_fit_width() {
    let show_width = reserved_toggle_button_width("Show panel");
    let hide_width = reserved_toggle_button_width("Hide panel");

    assert_eq!(show_width, hide_width);
}

#[test]
fn button_text_is_center_aligned_by_default() {
    let view = widget::view(|ui| {
        ui.button(widget::Button::new("Hide panel").reserve_labels(["Show panel"]));
    });
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(240, 80), &mut layout_engine);
    let painted = scene::Scene::paint(&layout);

    assert!(
        painted
            .texts()
            .iter()
            .any(|text| text.value() == "Hide panel" && text.align() == scene::TextAlign::Center),
        "button label should use centered text alignment"
    );
}

#[test]
fn choice_and_slider_labels_are_passive_hit_regions() {
    let view = widget::view(|ui| {
        ui.column(|ui| {
            ui.checkbox(widget::Checkbox::new("Wrap text", true).trigger::<RecordSource>(()));
            ui.slider(widget::Slider::new("Level", 0.4, 0.0..=1.0).on_change::<SetLevel>());
        });
    });
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(260, 90), &mut layout_engine);
    let checkbox = layout
        .find_role(view::node::Role::Checkbox)
        .into_iter()
        .next()
        .expect("checkbox should be laid out");
    let slider = layout
        .find_role(view::node::Role::Slider)
        .into_iter()
        .next()
        .expect("slider should be laid out");

    assert_eq!(
        layout
            .hit_test(center(checkbox.active_rect()))
            .map(|hit| hit.frame().role()),
        Some(view::node::Role::Checkbox)
    );
    assert_eq!(
        layout
            .hit_test(center(slider.active_rect()))
            .map(|hit| hit.frame().role()),
        Some(view::node::Role::Slider)
    );
    assert!(
        layout
            .hit_test(geometry::Point::new(
                checkbox.active_rect().right().saturating_add(10),
                checkbox
                    .rect()
                    .y()
                    .saturating_add(checkbox.rect().height() / 2),
            ))
            .is_none(),
        "checkbox label text should not be an active hit target"
    );
    assert!(
        layout
            .hit_test(geometry::Point::new(
                slider.rect().x().saturating_add(12),
                slider.rect().y().saturating_add(slider.rect().height() / 2),
            ))
            .is_none(),
        "slider label text should not be an active hit target"
    );
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

    let mut layout_engine = layout::Engine::new();
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

    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(100, 100), &mut layout_engine);
    let panels = layout.find_role(view::node::Role::Panel);
    let labels = layout.find_role(view::node::Role::Label);

    assert_eq!(panels.len(), 2);
    assert_eq!(labels.len(), 1);
    assert_eq!(panels[0].rect().x(), 6);
    assert_eq!(panels[0].rect().y(), 6);
    assert_eq!(panels[0].rect().height(), 20);
    assert_eq!(panels[1].rect().y(), 30);
    let expected_grow_height = 100 - 12 - 8 - panels[0].rect().height() - labels[0].rect().height();
    assert_eq!(panels[1].rect().height(), expected_grow_height);
    assert_eq!(labels[0].rect().y(), panels[1].rect().bottom() + 4);
    assert!(labels[0].rect().height() > 0);
}

fn reserved_toggle_button_width(label: &str) -> i32 {
    let view = widget::view(|ui| {
        ui.button(widget::Button::new(label).reserve_labels(["Show panel", "Hide panel"]));
    });
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(240, 80), &mut layout_engine);

    layout
        .find_role(view::node::Role::Button)
        .into_iter()
        .next()
        .expect("button should be laid out")
        .rect()
        .width()
}

fn center(rect: geometry::Rect) -> geometry::Point {
    geometry::Point::new(
        rect.x().saturating_add(rect.width() / 2),
        rect.y().saturating_add(rect.height() / 2),
    )
}
