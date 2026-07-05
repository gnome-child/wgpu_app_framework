use super::*;

#[test]
fn slider_on_change_invokes_command_with_layout_derived_value() {
    let mut app = Runtime::new(SliderValueState::default())
        .commands(|commands| {
            commands.register::<SetLevel>(command::Spec::new("Set Level"));
        })
        .responders(|responders| {
            responders.app().target::<SetLevel>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Slider"));
        })
        .view(|state, _| {
            widget::view(|ui| {
                ui.slider(
                    widget::Slider::new("Level", state.value, 0.0..=10.0).on_change::<SetLevel>(),
                );
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 80);
    let presentation = app
        .render_scene(window, size)
        .expect("slider view should render");
    let slider = presentation
        .layout()
        .find_role(view::node::Role::Slider)
        .into_iter()
        .next()
        .expect("slider should be laid out");
    let track = slider_track_rect(slider);
    let target = slider
        .target()
        .expect("bound slider should expose a command target");

    assert!(target.captures());
    assert!(
        presentation
            .scene()
            .quads()
            .iter()
            .any(|quad| quad.rect() == track),
        "slider input geometry should match the painted track"
    );

    let middle = geometry::Point::new(track.x() + track.width() / 2, track.y() + 1);
    let end = geometry::Point::new(track.right(), track.y() + 1);

    let pressed = pointer_down_then_present(&mut app, window, size, middle);

    assert!(pressed.is_handled());
    assert!(pressed.changed_state());
    assert_near(app.state().value, 5.0);
    assert_eq!(app.state().invocations, 1);
    assert_eq!(app.revision().get(), 1);
    assert!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().capture())
            .is_some()
    );

    let dragged = pointer_move_then_present(&mut app, window, size, end);

    assert!(dragged.is_handled());
    assert!(dragged.changed_state());
    assert_near(app.state().value, 10.0);
    assert_eq!(app.state().invocations, 2);
    assert_eq!(app.revision().get(), 2);

    pointer_up_then_present(&mut app, window, size, end);
    assert!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().capture())
            .is_none()
    );
}

#[test]
fn captured_slider_drag_coalesces_into_one_undo_entry() {
    let mut app = Runtime::new(SliderValueState::default())
        .commands(|commands| {
            commands.register::<SetLevel>(command::Spec::new("Set Level"));
        })
        .responders(|responders| {
            responders.app().target::<SetLevel>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Slider History"));
        })
        .view(|state, _| {
            widget::view(|ui| {
                ui.slider(
                    widget::Slider::new("Level", state.value, 0.0..=10.0).on_change::<SetLevel>(),
                );
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 80);
    let presentation = app
        .render_scene(window, size)
        .expect("slider view should render");
    let slider = presentation
        .layout()
        .find_role(view::node::Role::Slider)
        .into_iter()
        .next()
        .expect("slider should be laid out");
    let track = slider_track_rect(slider);
    let quarter = geometry::Point::new(track.x() + track.width() / 4, track.y() + 1);
    let half = geometry::Point::new(track.x() + track.width() / 2, track.y() + 1);
    let end = geometry::Point::new(track.right(), track.y() + 1);

    pointer_down_then_present(&mut app, window, size, quarter);
    pointer_move_then_present(&mut app, window, size, half);
    pointer_move_then_present(&mut app, window, size, end);

    assert_near(app.state().value, 10.0);
    assert_eq!(app.state().invocations, 3);
    assert_eq!(
        app.timeline().undo_depth(),
        0,
        "captured gesture should not publish undo entries until release"
    );

    pointer_up_then_present(&mut app, window, size, end);

    assert_near(app.state().value, 10.0);
    assert_eq!(app.timeline().undo_depth(), 1);
    assert_eq!(app.revision().get(), 3);

    assert!(app.undo(), "coalesced slider gesture should undo");

    assert_near(app.state().value, 0.0);
    assert_eq!(app.timeline().undo_depth(), 0);
    assert_eq!(app.timeline().redo_depth(), 1);
}

#[test]
fn slider_trigger_with_maps_layout_value_into_custom_command_args() {
    let mut app = Runtime::new(MappedSliderState::default())
        .commands(|commands| {
            commands.register::<SetMappedLevel>(command::Spec::new("Set Mapped Level"));
        })
        .responders(|responders| {
            responders.app().target::<SetMappedLevel>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Mapped Slider"));
        })
        .view(|state, _| {
            widget::view(|ui| {
                ui.slider(
                    widget::Slider::new("Mapped", state.raw, 0.0..=10.0)
                        .trigger_with::<SetMappedLevel, _>(|value| LevelArgs {
                            raw: value * 2.0,
                            snapped: value.round() as i32,
                        }),
                );
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 80);
    let presentation = app
        .render_scene(window, size)
        .expect("mapped slider view should render");
    let slider = presentation
        .layout()
        .find_role(view::node::Role::Slider)
        .into_iter()
        .next()
        .expect("mapped slider should be laid out");
    let track = slider_track_rect(slider);
    let middle = geometry::Point::new(track.x() + track.width() / 2, track.y() + 1);

    let pressed = app
        .pointer_down_at(window, size, middle)
        .expect("mapped slider pointer down should be handled");

    assert!(pressed.is_handled());
    assert!(pressed.changed_state());
    assert_near(app.state().raw, 10.0);
    assert_eq!(app.state().snapped, 5);
}

fn slider_track_rect(frame: &layout::frame::Frame) -> geometry::Rect {
    let theme = Theme::default();
    layout::control::slider_track_rect(frame.rect(), theme.metrics())
}
