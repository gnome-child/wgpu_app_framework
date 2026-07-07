use super::*;
use std::time::{Duration, Instant};

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

#[test]
fn slider_hover_animates_track_transform_without_tint_or_layout_shift() {
    let mut app = Runtime::new(SliderValueState {
        value: 5.0,
        ..SliderValueState::default()
    })
    .commands(|commands| {
        commands.register::<SetLevel>(command::Spec::new("Set Level"));
    })
    .responders(|responders| {
        responders.app().target::<SetLevel>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Animated Slider"));
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
    let start = Instant::now();
    let initial = app
        .render_scene_at(window, size, start)
        .expect("slider should render");
    let initial_slider = only_slider(initial.layout());
    let initial_rect = initial_slider.rect();
    let initial_active_rect = initial_slider.active_rect();
    let track = slider_track_rect(initial_slider);
    let hover = geometry::Point::new(track.x() + track.width() / 2, track.y() + 1);

    assert_eq!(track_scale_y(initial.scene(), track), 1.0);
    let before_hover = app
        .diagnostics(window)
        .expect("diagnostics should exist")
        .frame
        .clone();

    let moved = app
        .pointer_move_at(window, size, hover)
        .expect("hovering the slider should be handled");
    assert_eq!(
        moved.effect().invalidation(),
        Some(response::Invalidation::Paint)
    );

    let hovered = app
        .render_scene_at(window, size, start)
        .expect("hovered slider should render");
    let hovered_slider = only_slider(hovered.layout());
    let after_hover = app
        .diagnostics(window)
        .expect("diagnostics should exist")
        .frame
        .clone();

    assert_eq!(hovered_slider.rect(), initial_rect);
    assert_eq!(hovered_slider.active_rect(), initial_active_rect);
    assert_eq!(track_scale_y(hovered.scene(), track), 1.0);
    assert_no_slider_hover_tint(hovered.scene(), initial_active_rect);
    assert_eq!(
        app.animation_schedule(),
        crate::animation::Schedule::NextFrame
    );
    assert_eq!(after_hover.view_rebuilds, before_hover.view_rebuilds);
    assert_eq!(
        after_hover.layout_recomposes,
        before_hover.layout_recomposes
    );
    assert!(after_hover.layout_reuses > before_hover.layout_reuses);

    let mid = app
        .render_scene_at(window, size, start + Duration::from_millis(90))
        .expect("mid-animation slider should render");
    let mid_scale = track_scale_y(mid.scene(), track);
    assert!(mid_scale > 1.0 && mid_scale < 1.5);
    assert_eq!(thumb_transform_scale_y(mid.scene(), hovered_slider), 1.0);

    let settled = app
        .render_scene_at(window, size, start + Duration::from_millis(180))
        .expect("settled hovered slider should render");
    assert_approx_eq_f32(track_scale_y(settled.scene(), track), 1.5);
    assert_eq!(app.animation_schedule(), crate::animation::Schedule::Idle);

    app.pointer_move_at(window, size, geometry::Point::new(239, 79))
        .expect("leaving the slider should be handled");
    let leaving = app
        .render_scene_at(window, size, start + Duration::from_millis(180))
        .expect("leaving slider should render");
    assert_approx_eq_f32(track_scale_y(leaving.scene(), track), 1.5);
    assert_eq!(
        app.animation_schedule(),
        crate::animation::Schedule::NextFrame
    );

    let idle = app
        .render_scene_at(window, size, start + Duration::from_millis(360))
        .expect("idle slider should render");
    assert_approx_eq_f32(track_scale_y(idle.scene(), track), 1.0);
    assert_eq!(app.animation_schedule(), crate::animation::Schedule::Idle);
}

#[test]
fn due_slider_hover_animation_requests_redraw_without_model_revision_change() {
    let mut app = Runtime::new(SliderValueState {
        value: 5.0,
        ..SliderValueState::default()
    })
    .commands(|commands| {
        commands.register::<SetLevel>(command::Spec::new("Set Level"));
    })
    .responders(|responders| {
        responders.app().target::<SetLevel>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Animated Slider"));
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
    let start = Instant::now();
    let initial = app
        .render_scene_at(window, size, start)
        .expect("slider should render");
    let track = slider_track_rect(only_slider(initial.layout()));
    let hover = geometry::Point::new(track.x() + track.width() / 2, track.y() + 1);

    app.pointer_move_at(window, size, hover)
        .expect("hovering the slider should be handled");
    app.render_scene_at(window, size, start)
        .expect("hovered slider should render");

    assert!(!app.session().windows()[0].redraw_requested());
    let revision = app.revision();
    app.invalidate_due_animation_frames(start + Duration::from_millis(1));

    assert_eq!(app.revision(), revision);
    assert!(app.session().windows()[0].redraw_requested());
}

fn slider_track_rect(frame: &layout::Frame) -> geometry::Rect {
    let theme = Theme::default();
    layout::slider_track_rect(frame.rect(), frame.label_width(), &theme)
}

fn only_slider(layout: &layout::Layout) -> &layout::Frame {
    layout
        .find_role(view::node::Role::Slider)
        .into_iter()
        .next()
        .expect("slider should be laid out")
}

fn track_scale_y(scene: &Scene, track: geometry::Rect) -> f32 {
    let theme = Theme::default();
    scene
        .quads()
        .into_iter()
        .find(|quad| quad.rect() == track && quad.fill() == theme.slider().track)
        .map(|quad| quad.transform().scale_y())
        .expect("slider track should paint")
}

fn thumb_transform_scale_y(scene: &Scene, slider: &layout::Frame) -> f32 {
    let theme = Theme::default();
    let thumb = layout::slider_thumb_rect(
        slider.rect(),
        slider.slider().expect("slider model should exist"),
        slider.label_width(),
        &theme,
    );

    scene
        .quads()
        .into_iter()
        .find(|quad| quad.rect() == thumb && quad.fill() == theme.slider().thumb)
        .map(|quad| quad.transform().scale_y())
        .expect("slider thumb should paint")
}

fn assert_no_slider_hover_tint(scene: &Scene, active_rect: geometry::Rect) {
    let theme = Theme::default();

    assert!(
        !scene.quads().iter().any(|quad| {
            quad.rect() == active_rect && quad.fill() == theme.control().hover_tint
        }),
        "hovered slider should not paint generic control hover tint"
    );
}

fn assert_approx_eq_f32(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.0001,
        "expected {actual} to be near {expected}"
    );
}
