use super::*;

macro_rules! palette_test_command {
    ($name:ident, $command:literal) => {
        struct $name;

        impl Command for $name {
            type Args = ();
            type Output = ();

            const NAME: &'static str = $command;
        }

        impl Target<$name> for SourceState {
            fn state(&self, _: &(), _: &Context) -> command::State {
                command::State::enabled()
            }

            fn invoke(&mut self, _: (), cx: &mut Context) -> Response<()> {
                self.sources.push(cx.source());
                Response::changed(())
            }
        }
    };
}

palette_test_command!(PaletteOne, "palette.one");
palette_test_command!(PaletteTwo, "palette.two");
palette_test_command!(PaletteThree, "palette.three");
palette_test_command!(PaletteFour, "palette.four");
palette_test_command!(PaletteFive, "palette.five");
palette_test_command!(PaletteSix, "palette.six");
palette_test_command!(PaletteSeven, "palette.seven");
palette_test_command!(PaletteEight, "palette.eight");
palette_test_command!(PaletteNine, "palette.nine");
palette_test_command!(PaletteTen, "palette.ten");
palette_test_command!(PaletteEleven, "palette.eleven");
palette_test_command!(PaletteTwelve, "palette.twelve");

#[test]
fn text_editor_view_composes_to_layout_without_runtime_mutation() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let revision = app.revision();
    let mut layout_engine = layout::engine::Engine::new();
    let _: &Layout = &layout::Layout::compose(
        &projected,
        geometry::Size::new(800, 600),
        &mut layout_engine,
    );
    let layout = layout::Layout::compose(
        &projected,
        geometry::Size::new(800, 600),
        &mut layout_engine,
    );

    assert_eq!(layout.size(), geometry::Size::new(800, 600));
    assert_eq!(app.revision(), revision);
    assert_eq!(app.session().file_dialog(window), None);

    let menus = layout.find_role(view::node::Role::Menu);
    assert_eq!(menus.len(), 3);
    assert_eq!(menus[0].label_text(), Some("File"));
    assert_eq!(menus[1].label_text(), Some("Edit"));
    assert_eq!(menus[2].label_text(), Some("View"));

    let text_areas = layout.find_role(view::node::Role::TextArea);
    assert_eq!(text_areas.len(), 1);
    assert_eq!(text_areas[0].rect().y(), Theme::default().menu().bar_height);
    assert!(text_areas[0].rect().height() > 0);
    assert!(
        text_areas[0]
            .target()
            .expect("text area should expose a target")
            .captures()
    );

    let menu_hit = layout
        .hit_test(geometry::Point::new(10, 10))
        .expect("file menu should be hit");
    assert_eq!(menu_hit.frame().role(), view::node::Role::Menu);
    assert_eq!(menu_hit.frame().label_text(), Some("File"));
    assert!(matches!(
        menu_hit.action(),
        Some(view::Action::ToggleMenu(menu)) if menu.label() == "File"
    ));

    let text_hit = layout
        .hit_test(geometry::Point::new(10, 80))
        .expect("text area should be hit");
    assert_eq!(text_hit.frame().role(), view::node::Role::TextArea);
    assert!(matches!(text_hit.action(), Some(view::Action::Focus(_))));
}

#[test]
fn menu_bar_buttons_share_largest_label_width() {
    let view = View::new(
        view::Node::root().child(
            view::Node::menu_bar()
                .child(view::Node::menu("menu.file", "File"))
                .child(view::Node::menu("menu.selection", "Selection"))
                .child(view::Node::menu("menu.view", "V")),
        ),
    );
    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(400, 120), &mut layout_engine);
    let menus = layout.find_role(view::node::Role::Menu);

    assert_eq!(menus.len(), 3);
    assert_eq!(menus[0].rect().width(), menus[1].rect().width());
    assert_eq!(menus[1].rect().width(), menus[2].rect().width());
    assert!(menus[0].rect().width() > Theme::default().menu().bar_height);
}

#[test]
fn single_character_menu_titles_are_square_from_control_padding() {
    let view = View::new(
        view::Node::root().child(
            view::Node::menu_bar()
                .child(view::Node::menu("menu.a", "A"))
                .child(view::Node::menu("menu.b", "B")),
        ),
    );
    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(160, 80), &mut layout_engine);
    let menus = layout.find_role(view::node::Role::Menu);

    assert_eq!(menus.len(), 2);
    for menu in menus {
        assert_eq!(menu.rect().width(), menu.rect().height());
        assert_eq!(menu.rect().height(), Theme::default().menu().bar_height);
    }
}

#[test]
fn menu_bar_defaults_match_system_menu_scale() {
    let theme = Theme::default();

    assert_eq!(theme.typography().interface().size(), 12.0);
    assert_eq!(
        theme.typography().interface().weight(),
        text::document::Weight::Normal
    );
    assert_eq!(theme.control().height, 22);
    assert_eq!(theme.menu().bar_height, 22);
    assert_eq!(theme.menu().row_height, 22);
}

#[test]
fn menu_bar_intrinsic_height_matches_bar_content_height() {
    let theme = Theme::default();
    let view = View::new(
        view::Node::root().child(
            view::Node::stack(view::node::Axis::Vertical)
                .child(view::Node::menu_bar().child(view::Node::menu("menu.file", "File")))
                .child(view::Node::label("Below")),
        ),
    );
    let mut engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(240, 120),
        &mut engine,
        &theme,
    );
    let menu = layout
        .find_role(view::node::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("File"))
        .expect("menu title should be laid out");
    let below = layout
        .find_role(view::node::Role::Label)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Below"))
        .expect("following label should be laid out");

    assert_eq!(menu.rect().height(), theme.menu().bar_height);
    assert_eq!(below.rect().y(), menu.rect().bottom());
}

#[test]
fn menu_bar_title_typography_uses_interface_domain() {
    let view = View::new(
        view::Node::root().child(
            view::Node::menu_bar()
                .child(view::Node::menu("menu.file", "File"))
                .child(view::Node::menu("menu.selection", "Selection")),
        ),
    );
    let body_large = Theme::from_toml_str(
        r##"
        [typography]
        body-size = 28.0
        "##,
    )
    .expect("body-large theme should parse");
    let interface_large = Theme::from_toml_str(
        r##"
        [typography]
        body-size = 28.0
        interface-size = 18.0
        interface-weight = "bold"
        "##,
    )
    .expect("interface-large theme should parse");
    let mut engine = layout::engine::Engine::new();
    let default_layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(400, 120),
        &mut engine,
        &Theme::default(),
    );
    let body_large_layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(400, 120),
        &mut engine,
        &body_large,
    );
    let interface_large_layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(400, 120),
        &mut engine,
        &interface_large,
    );
    let default_selection = default_layout
        .find_role(view::node::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Selection"))
        .expect("default selection menu should be laid out")
        .rect();
    let body_large_selection = body_large_layout
        .find_role(view::node::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Selection"))
        .expect("body-large selection menu should be laid out")
        .rect();
    let interface_large_selection = interface_large_layout
        .find_role(view::node::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Selection"))
        .expect("interface-large selection menu should be laid out")
        .rect();
    let scene = scene::Scene::paint_with_theme(&interface_large_layout, &interface_large);
    let selection_text = scene_text(&scene, "Selection");

    assert_eq!(body_large_selection.width(), default_selection.width());
    assert!(interface_large_selection.width() > default_selection.width());
    assert_eq!(selection_text.style().size(), 18.0);
    assert_eq!(
        selection_text.style().weight(),
        text::document::Weight::Bold
    );
}

#[test]
fn generic_scroll_measures_content_clips_children_and_paints_scrollbar() {
    let view = widget::view(|ui| {
        ui.column(|ui| {
            ui.add(
                widget::Scroll::new()
                    .id("scroll.generic")
                    .height(view::style::Dimension::fixed(72))
                    .children(|ui| {
                        for index in 0..8 {
                            ui.label(format!("Row {index}"));
                        }
                    }),
            );
        });
    });
    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(220, 120), &mut layout_engine);
    let scroll = layout
        .find_role(view::node::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out");
    let viewport = scroll
        .viewport()
        .expect("scroll should resolve viewport geometry");

    assert_eq!(scroll.rect().height(), 72);
    assert_eq!(viewport.rect(), scroll.rect());
    assert!(viewport.content().height() > viewport.rect().height());
    assert_eq!(
        viewport.max_scroll().y(),
        viewport
            .content()
            .height()
            .saturating_sub(viewport.rect().height())
    );

    let mut theme = Theme::dark();
    theme.scrollbar_mut().metrics.policy = crate::scratch::theme::ScrollbarPolicy::GutterAlways;
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(220, 120),
        &mut layout_engine,
        &theme,
    );
    let scroll = layout
        .find_role(view::node::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out with gutter theme");
    let viewport = scroll
        .viewport()
        .expect("scroll should resolve gutter viewport geometry");
    let scene = scene::Scene::paint_with_theme(&layout, &theme);
    assert!(
        scene
            .primitives()
            .iter()
            .any(|primitive| matches!(primitive, scene::Primitive::Clip(clip) if clip.rect() == viewport.rect())),
        "scroll children should be clipped to the viewport"
    );
    let thumb = theme.scrollbar().appearance.thumb;
    assert!(
        scene
            .quads()
            .iter()
            .any(|quad| quad.fill() == thumb && rect_contains(scroll.rect(), quad.rect())),
        "scrollbar thumb should be projected from viewport geometry"
    );
}

#[test]
fn viewport_content_extent_equals_placed_child_bounds() {
    let padding = view::style::Padding::edges(5, 7, 3, 11);
    let scroll = view::Node::scroll()
        .with_interaction_id("scroll.placed")
        .with_style(
            view::Style::new()
                .with_height(view::style::Dimension::fixed(72))
                .with_padding(padding)
                .with_gap(3),
        )
        .child(view::Node::section_header("Application"))
        .child(
            view::Node::label("Fixed row")
                .with_style(view::Style::new().with_height(view::style::Dimension::fixed(31))),
        )
        .child(view::Node::label("Body row"));
    let view = View::new(view::Node::root().child(scroll));
    let mut engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(240, 120), &mut engine);
    let scroll = layout
        .find_role(view::node::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out");
    let viewport = scroll
        .viewport()
        .expect("scroll should resolve viewport geometry");
    let children = immediate_scroll_child_frames(&layout, scroll);
    let child_right = children
        .iter()
        .map(|frame| frame.rect().right())
        .max()
        .expect("scroll should have children");
    let child_bottom = children
        .iter()
        .map(|frame| frame.rect().bottom())
        .max()
        .expect("scroll should have children");
    let expected = geometry::Size::new(
        viewport.rect().width().max(
            child_right
                .saturating_sub(viewport.rect().x())
                .saturating_add(padding.right()),
        ),
        viewport.rect().height().max(
            child_bottom
                .saturating_sub(viewport.rect().y())
                .saturating_add(padding.bottom()),
        ),
    );

    assert_eq!(viewport.content(), expected);
}

#[test]
fn viewport_max_scroll_reaches_last_placed_descendant() {
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Scroll To Last"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.add(
                    widget::Scroll::new()
                        .id("scroll.last")
                        .height(view::style::Dimension::fixed(72))
                        .children(|ui| {
                            for index in 0..12 {
                                ui.label(format!("Row {index}"));
                            }
                        }),
                );
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 140);
    let initial = app
        .render_scene(window, size)
        .expect("scroll scene should render");
    let scroll = initial
        .layout()
        .find_role(view::node::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out");
    app.scroll_at(
        window,
        size,
        frame_point_at(
            scroll
                .viewport()
                .expect("scroll should expose viewport")
                .rect(),
        ),
        interaction::ScrollDelta::vertical(10_000),
    )
    .expect("scroll input should be handled");

    let rendered = app
        .render_scene(window, size)
        .expect("scroll scene should render after scroll");
    let scroll = rendered
        .layout()
        .find_role(view::node::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out");
    let viewport = scroll
        .viewport()
        .expect("scroll should expose viewport")
        .rect();
    let last = rendered
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.label_text() == Some("Row 11"))
        .expect("last row should be laid out");

    assert!(last.rect().bottom() <= viewport.bottom());
}

#[test]
fn grow_children_collapse_to_intrinsic_inside_scroll_axis() {
    let view = widget::view(|ui| {
        ui.add(
            widget::Scroll::new()
                .height(view::style::Dimension::fixed(80))
                .children(|ui| {
                    ui.add(
                        widget::Element::new()
                            .label("Grow")
                            .height(view::style::Dimension::grow()),
                    );
                    ui.label("After");
                }),
        );
    });
    let mut engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(220, 120), &mut engine);
    let scroll = layout
        .find_role(view::node::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out");
    let grow = layout
        .find_role(view::node::Role::Panel)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Grow"))
        .expect("grow child should be laid out");
    let after = layout
        .find_role(view::node::Role::Label)
        .into_iter()
        .find(|frame| frame.label_text() == Some("After"))
        .expect("following label should be laid out");

    assert!(grow.rect().height() < scroll.rect().height());
    assert_eq!(after.rect().y(), grow.rect().bottom());
}

#[test]
fn justify_content_is_start_when_scroll_content_exceeds_viewport() {
    let view = widget::view(|ui| {
        ui.column(|ui| {
            ui.add(
                widget::Scroll::new()
                    .height(view::style::Dimension::fixed(40))
                    .layout(|layout| layout.justify_content(view::style::Align::End))
                    .children(|ui| {
                        for index in 0..4 {
                            ui.add(
                                widget::Element::new()
                                    .label(format!("Row {index}"))
                                    .height(view::style::Dimension::fixed(24)),
                            );
                        }
                    }),
            );
        });
    });
    let mut engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(220, 100), &mut engine);
    let scroll = layout
        .find_role(view::node::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out");
    let first = layout
        .find_role(view::node::Role::Panel)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Row 0"))
        .expect("first row should be laid out");

    assert!(
        scroll
            .viewport()
            .expect("scroll should expose viewport")
            .is_scrollable()
    );
    assert_eq!(first.rect().y(), scroll.rect().y());
}

#[test]
fn generic_scroll_feedback_clamps_session_offset_after_present() {
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Generic Scroll"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.add(
                        widget::Scroll::new()
                            .id("scroll.feedback")
                            .height(view::style::Dimension::fixed(72))
                            .children(|ui| {
                                for index in 0..8 {
                                    ui.label(format!("Row {index}"));
                                }
                            }),
                    );
                });
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(220, 120);
    let initial = app
        .render_scene(window, size)
        .expect("scroll view should render");
    let scroll = initial
        .layout()
        .find_role(view::node::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out");
    let expected_max_scroll = scroll
        .viewport()
        .expect("scroll should resolve viewport geometry")
        .max_scroll();
    let target = scroll
        .target()
        .expect("scroll should expose a target")
        .clone();

    app.scroll_at(
        window,
        size,
        frame_point_at(scroll.rect()),
        interaction::ScrollDelta::vertical(400),
    )
    .expect("scroll input should be handled");
    app.render_scene(window, size)
        .expect("scroll feedback should render");

    let offset = app
        .session()
        .interaction(window)
        .expect("window should have interaction")
        .scroll()
        .offset(&target);

    assert_eq!(offset, expected_max_scroll);
}

#[test]
fn gutter_scrollbar_metrics_reduce_viewport_width() {
    let mut theme = Theme::dark();
    theme.scrollbar_mut().metrics.policy = crate::scratch::theme::ScrollbarPolicy::GutterAlways;
    let view = widget::view(|ui| {
        ui.column(|ui| {
            ui.add(
                widget::Scroll::new()
                    .id("scroll.gutter")
                    .height(view::style::Dimension::fixed(72))
                    .children(|ui| {
                        for index in 0..8 {
                            ui.label(format!("Row {index}"));
                        }
                    }),
            );
        });
    });
    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(220, 120),
        &mut layout_engine,
        &theme,
    );
    let scroll = layout
        .find_role(view::node::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out");
    let viewport = scroll
        .viewport()
        .expect("scroll should resolve viewport geometry");

    assert!(viewport.rect().width() < scroll.rect().width());
    assert_eq!(
        scroll
            .rect()
            .width()
            .saturating_sub(viewport.rect().width()),
        theme.scrollbar().metrics.thickness
    );
}

#[test]
fn generic_scroll_pointer_drag_updates_viewport_offset() {
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Generic Scroll Drag"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.add(
                        widget::Scroll::new()
                            .id("scroll.drag")
                            .height(view::style::Dimension::fixed(72))
                            .children(|ui| {
                                for index in 0..8 {
                                    ui.label(format!("Row {index}"));
                                }
                            }),
                    );
                });
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(220, 120);
    let initial = app
        .render_scene(window, size)
        .expect("scroll view should render");
    let scroll = initial
        .layout()
        .find_role(view::node::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out");
    let target = scroll
        .target()
        .expect("scroll should expose a target")
        .clone();
    let expected_max_scroll = scroll
        .viewport()
        .expect("scroll should resolve viewport geometry")
        .max_scroll();
    let track = initial
        .layout()
        .chrome()
        .iter()
        .find_map(|chrome| match chrome.kind() {
            layout::chrome::Kind::Scrollbar(scrollbar) => Some(scrollbar.track()),
        })
        .expect("scrollbar chrome should be projected");
    let press = geometry::Point::new(track.x().saturating_add(track.width() / 2), track.y() + 1);
    let drag = geometry::Point::new(
        track.x().saturating_add(track.width() / 2),
        track.bottom().saturating_sub(1),
    );

    app.pointer_down_at(window, size, press)
        .expect("scroll pointer down should be handled");
    app.pointer_drag_at(window, size, drag)
        .expect("scroll pointer drag should be handled");
    app.render_scene(window, size)
        .expect("scroll feedback should render");

    let offset = app
        .session()
        .interaction(window)
        .expect("window should have interaction")
        .scroll()
        .offset(&target);

    assert_eq!(offset, expected_max_scroll);
}

#[test]
fn viewport_intrinsics_ignore_content_extent() {
    let mut theme = Theme::dark();
    theme.viewport_mut().min_viewport_extent = 64;
    let view = widget::view(|ui| {
        ui.column(|ui| {
            ui.add(
                widget::Scroll::new()
                    .height(view::style::Dimension::fit())
                    .children(|ui| {
                        for index in 0..12 {
                            ui.label(format!("Tall row {index}"));
                        }
                    }),
            );
        });
    });
    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(240, 400),
        &mut layout_engine,
        &theme,
    );
    let scroll = layout
        .find_role(view::node::Role::Scroll)
        .into_iter()
        .next()
        .expect("fit scroll should be laid out");
    let viewport = scroll.viewport().expect("scroll should expose viewport");

    assert_eq!(scroll.rect().height(), 64);
    assert!(viewport.content().height() > scroll.rect().height());
}

#[test]
fn scrollbar_thumb_wins_hit_test_over_content() {
    let view = widget::view(|ui| {
        ui.add(
            widget::Scroll::new()
                .id("scroll.hit")
                .height(view::style::Dimension::fixed(72))
                .children(|ui| {
                    for index in 0..8 {
                        ui.button(widget::Button::new(format!("Row {index}")).trigger::<Ping>(()));
                    }
                }),
        );
    });
    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(220, 120), &mut layout_engine);
    let track = first_scrollbar_track(&layout);
    let hit = layout
        .hit_test(geometry::Point::new(
            track.x().saturating_add(track.width() / 2),
            track.y().saturating_add(track.height() / 2),
        ))
        .expect("scrollbar chrome should be hit");

    assert!(hit.is_chrome());
    assert_eq!(
        hit.target().expect("chrome should expose target").kind(),
        interaction::target::Kind::Scrollbar
    );
}

#[test]
fn overlay_auto_hides_idle_appears_after_activity_and_fades_out() {
    let mut app = scroll_app();
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(220, 120);
    let theme = Theme::dark();
    let now = std::time::Instant::now();

    let idle = app
        .render_scene_at(window, size, now)
        .expect("idle scroll view should render");
    let scroll = first_scroll_frame(&idle);
    assert!(
        !scene_has_scrollbar_thumb(idle.scene(), &theme, scroll.rect()),
        "overlay scrollbar should be hidden before activity"
    );

    app.scroll_at(
        window,
        size,
        frame_point_at(scroll.rect()),
        interaction::ScrollDelta::vertical(80),
    )
    .expect("scroll should be handled");

    let activity_at = now + std::time::Duration::from_millis(10);
    app.render_scene_at(window, size, activity_at)
        .expect("activity frame should start scrollbar fade-in");
    let visible_at = activity_at + std::time::Duration::from_millis(260);
    let visible = app
        .render_scene_at(window, size, visible_at)
        .expect("active scroll view should render");
    let scroll = first_scroll_frame(&visible);
    assert!(
        scene_has_scrollbar_thumb(visible.scene(), &theme, scroll.rect()),
        "overlay scrollbar should appear after scroll activity"
    );

    let fade_start =
        activity_at + std::time::Duration::from_millis(theme.scrollbar().appearance.fade_delay_ms);
    app.render_scene_at(window, size, fade_start)
        .expect("fade deadline should render");
    let faded = app
        .render_scene_at(
            window,
            size,
            fade_start
                + std::time::Duration::from_millis(
                    theme.scrollbar().appearance.fade_duration_ms + 20,
                ),
        )
        .expect("faded scroll view should render");
    let scroll = first_scroll_frame(&faded);
    assert!(
        !scene_has_scrollbar_thumb(faded.scene(), &theme, scroll.rect()),
        "overlay scrollbar should fade out after inactivity"
    );
}

#[test]
fn gutter_always_reserves_base_gutter_and_remains_visible() {
    let mut theme = Theme::dark();
    theme.scrollbar_mut().metrics.policy = crate::scratch::theme::ScrollbarPolicy::GutterAlways;
    let mut app = scroll_app().theme(move |_| theme.clone());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(220, 120);
    let rendered = app
        .render_scene(window, size)
        .expect("gutter scroll view should render");
    let scroll = first_scroll_frame(&rendered);
    let viewport = scroll.viewport().expect("scroll should expose viewport");
    let theme = Theme::dark();

    assert_eq!(
        scroll
            .rect()
            .width()
            .saturating_sub(viewport.rect().width()),
        theme.scrollbar().metrics.thickness
    );
    assert!(
        scene_has_scrollbar_thumb(rendered.scene(), &theme, scroll.rect()),
        "gutter scrollbar should paint while idle"
    );
}

#[test]
fn hover_thickness_does_not_change_scroll_layout_rects() {
    let mut app = scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(220, 120);
    let now = std::time::Instant::now();
    let initial = app
        .render_scene_at(window, size, now)
        .expect("scroll view should render");
    let initial_scroll = first_scroll_frame(&initial).rect();
    let initial_viewport = first_scroll_frame(&initial)
        .viewport()
        .expect("scroll should expose viewport")
        .rect();
    let track = first_scrollbar_track(initial.layout());

    app.pointer_move_at(
        window,
        size,
        geometry::Point::new(
            track.x().saturating_add(track.width() / 2),
            track.y().saturating_add(track.height() / 2),
        ),
    )
    .expect("hovering scrollbar should be handled");
    let hovered = app
        .render_scene_at(window, size, now + std::time::Duration::from_millis(260))
        .expect("hovered scroll view should render");
    let hovered_scroll = first_scroll_frame(&hovered);

    assert_eq!(hovered_scroll.rect(), initial_scroll);
    assert_eq!(
        hovered_scroll
            .viewport()
            .expect("scroll should expose hovered viewport")
            .rect(),
        initial_viewport
    );
}

#[test]
fn text_area_projects_scrollbars_like_generic_viewports() {
    let document = (0..120)
        .map(|line| format!("text area line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text(document),
        ..text_editor::State::default()
    });
    app.start();

    let window = app.session().windows()[0].id();
    let rendered = app
        .render_scene(window, geometry::Size::new(520, 180))
        .expect("text editor should render");
    let text_area = rendered
        .layout()
        .find_role(view::node::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let viewport = text_area
        .viewport()
        .expect("text area should expose shared viewport geometry");
    let target = text_area
        .target()
        .expect("text area should expose scroll target");

    assert!(viewport.max_scroll().y() > 0);
    assert!(
        rendered
            .layout()
            .chrome()
            .iter()
            .any(|chrome| chrome.scroll_target() == target),
        "text-area viewport should project scrollbar chrome"
    );
}

#[test]
fn text_area_scrollbar_hit_does_not_route_to_text_editing() {
    let document = (0..120)
        .map(|line| format!("drag line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text(document),
        ..text_editor::State::default()
    });
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(520, 180);
    let rendered = app
        .render_scene(window, size)
        .expect("text editor should render");
    let track = first_scrollbar_track(rendered.layout());
    let point = geometry::Point::new(
        track.x().saturating_add(track.width() / 2),
        track.y().saturating_add(track.height() / 2),
    );
    let before_position = app.state().document.position();
    let before_selection = app.state().document.selected_text();
    let hit = app
        .hit_test(window, size, point)
        .expect("scrollbar should hit");

    assert!(hit.is_chrome());
    assert_eq!(
        hit.target().expect("scrollbar should expose target").kind(),
        interaction::target::Kind::Scrollbar
    );

    app.pointer_down_at(window, size, point)
        .expect("scrollbar pointer down should be handled");
    app.pointer_drag_at(
        window,
        size,
        geometry::Point::new(
            track.x().saturating_add(track.width() / 2),
            track.bottom().saturating_sub(1),
        ),
    )
    .expect("scrollbar pointer drag should be handled");

    assert_eq!(app.state().document.position(), before_position);
    assert_eq!(app.state().document.selected_text(), before_selection);
}

#[test]
fn viewport_clip_applies_inside_floating_panel() {
    let view = widget::view(|ui| {
        ui.add(
            widget::panel::Floating::new("tests.floating.scroll")
                .width(view::style::Dimension::fixed(180))
                .height(view::style::Dimension::fixed(120))
                .children(|ui| {
                    ui.add(
                        widget::Scroll::new()
                            .id("scroll.floating")
                            .height(view::style::Dimension::fixed(48))
                            .children(|ui| {
                                for index in 0..6 {
                                    ui.label(format!("Floating row {index}"));
                                }
                            }),
                    );
                }),
        );
    });
    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(240, 180), &mut layout_engine);
    let scroll = layout
        .find_role(view::node::Role::Scroll)
        .into_iter()
        .next()
        .expect("floating scroll should be laid out");
    let viewport = scroll.viewport().expect("scroll should expose viewport");
    let scene = scene::Scene::paint_with_theme(&layout, &Theme::dark());
    let clip_index = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(primitive, scene::Primitive::Clip(clip) if clip.rect() == viewport.rect())
        })
        .expect("floating scroll should push a viewport clip");
    let text_index = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Text(text) if text.value() == "Floating row 0"
            )
        })
        .expect("floating scroll content should paint");
    let pop_index = scene
        .primitives()
        .iter()
        .skip(text_index)
        .position(|primitive| matches!(primitive, scene::Primitive::PopClip))
        .map(|offset| text_index + offset)
        .expect("floating scroll should pop the viewport clip after content");

    assert!(clip_index < text_index);
    assert!(text_index < pop_index);
}

#[test]
fn scrolled_out_content_is_not_interactive() {
    let focus = session::Focus::text("clip.search");
    let mut app = Runtime::new(SourceState::default())
        .commands(|commands| {
            commands.register::<PaletteOne>(command::Spec::new("Result"));
        })
        .responders(|responders| {
            responders.app().target::<PaletteOne>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Clip Hit"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.text_box(widget::TextBox::new("").focus(focus));
                    ui.add(
                        widget::Scroll::new()
                            .id("clip.results")
                            .height(view::style::Dimension::fixed(72))
                            .children(|ui| {
                                for index in 0..8 {
                                    ui.button(
                                        widget::Button::new(format!("Result {index}"))
                                            .trigger::<PaletteOne>(()),
                                    );
                                }
                            }),
                    );
                });
            })
        });
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 160);
    let initial = app
        .render_scene(window, size)
        .expect("initial clipped view should render");
    let scroll = first_scroll_frame(&initial);
    app.scroll_at(
        window,
        size,
        frame_point_at(
            scroll
                .viewport()
                .expect("scroll should expose viewport")
                .rect(),
        ),
        interaction::ScrollDelta::vertical(56),
    )
    .expect("scroll should be handled");
    let rendered = app
        .render_scene(window, size)
        .expect("scrolled view should render");
    let search = rendered
        .layout()
        .find_role(view::node::Role::TextBox)
        .into_iter()
        .next()
        .expect("search box should be laid out");
    let point = rect_bottom_point(search.rect());

    assert!(
        rendered.layout().frames().iter().any(|frame| {
            frame.target().is_some() && frame.rect().contains(point) && !frame.clip_contains(point)
        }),
        "a scrolled-out result should geometrically overlap the search box"
    );

    let hit = rendered
        .layout()
        .hit_test(point)
        .expect("visible search box should be hit");

    assert_eq!(hit.frame().role(), view::node::Role::TextBox);
}

#[test]
fn editable_text_surfaces_use_active_text_input_foreground() {
    let theme = Theme::from_toml_str(
        r##"
        [text-input]
        foreground = "#ff0000"
        "##,
    )
    .expect("theme should parse");
    let document = TextDocument::from_multiline_text("area");
    let view = widget::view(|ui| {
        ui.column(|ui| {
            ui.text_box(widget::TextBox::new("field"));
            ui.text_area(widget::TextArea::from_buffer(
                document.buffer().clone(),
                document.text_state(),
            ));
        });
    });
    let mut engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(260, 120),
        &mut engine,
        &theme,
    );
    let scene = scene::Scene::paint_with_theme(&layout, &theme);
    let expected = text_color_channels(scene::Color::rgb(255, 0, 0));
    let surface_colors = scene
        .text_viewports()
        .iter()
        .flat_map(|viewport| viewport.surfaces().iter())
        .map(|surface| surface.default_color().channels())
        .collect::<Vec<_>>();

    assert!(surface_colors.len() >= 2);
    assert!(
        surface_colors
            .iter()
            .all(|channels| text_color_channels_equal(*channels, expected)),
        "all editable text surfaces should use text-input foreground"
    );
}

#[test]
fn text_box_placeholder_uses_text_input_placeholder_color() {
    let theme = Theme::from_toml_str(
        r##"
        [text-input]
        placeholder = "#00ff00"
        "##,
    )
    .expect("theme should parse");
    let view = widget::view(|ui| {
        ui.text_box(widget::TextBox::new("").placeholder("Find"));
    });
    let mut engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(220, 64),
        &mut engine,
        &theme,
    );
    let scene = scene::Scene::paint_with_theme(&layout, &theme);

    assert_eq!(
        scene_text(&scene, "Find").color(),
        scene::Color::rgb(0, 255, 0)
    );
}

#[test]
fn editable_caret_uses_text_input_caret_color() {
    let focus = session::Focus::text("caret.theme");
    let theme = Theme::from_toml_str(
        r##"
        [text-input]
        caret = "#00ff00"
        "##,
    )
    .expect("theme should parse");
    let mut app = Runtime::new(SourceState::default())
        .theme(move |_| theme.clone())
        .started(|cx| {
            cx.open_window(window::Options::new("Caret Theme"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(widget::TextBox::new("abcd").focus(focus));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 80);
    app.render_scene(window, size)
        .expect("initial render should install composition");
    app.handle_input(window, Input::focus(focus))
        .expect("focus should be handled");
    let rendered = app
        .render_scene(window, size)
        .expect("focused text box should render");

    assert!(rendered.scene().quads().iter().any(|quad| {
        quad.fill() == scene::Color::rgb(0, 255, 0)
            && quad.rasterization().snapping() == (scene::Snapping::FixedWidth { width_px: 2 })
            && quad.rasterization().edge_mode() == scene::EdgeMode::Hard
    }));
}

#[test]
fn command_palette_search_box_wins_over_clipped_results() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.render_scene(window, size)
        .expect("initial palette app should render");
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let initial = app
        .render_scene(window, size)
        .expect("open palette should render");
    let results = command_palette_results_frame(&initial);
    app.scroll_at(
        window,
        size,
        frame_point_at(
            results
                .viewport()
                .expect("results should expose viewport")
                .rect(),
        ),
        interaction::ScrollDelta::vertical(180),
    )
    .expect("palette results should scroll");
    let rendered = app
        .render_scene(window, size)
        .expect("scrolled palette should render");
    let query = rendered
        .layout()
        .find_role(view::node::Role::TextBox)
        .into_iter()
        .next()
        .expect("palette query should be laid out");
    let point = rect_bottom_point(query.rect());

    assert!(
        rendered.layout().frames().iter().any(|frame| {
            frame.target().is_some() && frame.rect().contains(point) && !frame.clip_contains(point)
        }),
        "a clipped palette result should geometrically overlap the query box"
    );

    let hit = rendered
        .layout()
        .hit_test(point)
        .expect("palette query should be hit");

    assert_eq!(hit.frame().role(), view::node::Role::TextBox);

    app.pointer_down_at(window, size, point)
        .expect("query pointer down should be handled");

    assert_eq!(
        app.session()
            .focused(window)
            .is_some_and(|focus| focus.same_target(&interaction::CommandPalette::query_focus())),
        true
    );
}

#[test]
fn palette_results_scroll_id_is_not_painted() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let rendered = app
        .render_scene(window, size)
        .expect("open palette should render");
    let results = command_palette_results_frame(&rendered);

    assert_eq!(results.label_text(), None);
    assert_eq!(
        results.target().and_then(interaction::Target::element_id),
        Some(interaction::CommandPalette::results_id())
    );
    assert!(
        rendered
            .scene()
            .texts()
            .iter()
            .all(|text| text.value() != "Command Results"),
        "the results viewport id must not paint as visible text"
    );
}

#[test]
fn command_palette_section_headers_use_bold_caption_uppercase_presentation() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let rendered = app
        .render_scene(window, size)
        .expect("open palette should render");
    let theme = Theme::dark();
    let header = scene_text(rendered.scene(), "APPLICATION");

    assert_eq!(header.style().size(), theme.typography().caption().size());
    assert_eq!(header.style().weight(), text::document::Weight::Bold);
    assert_eq!(header.color(), theme.text().muted);
    assert_eq!(header.align(), theme.command_palette().section_alignment());
    assert!(
        rendered
            .scene()
            .texts()
            .iter()
            .all(|text| text.value() != "Application"),
        "section source labels stay mixed-case data; only presentation uppercases"
    );
}

#[test]
fn command_palette_rows_use_interface_shortcut_typography() {
    let theme = Theme::from_toml_str(
        r##"
        [typography]
        interface-size = 13.0
        interface-weight = "medium"
        body-size = 19.0
        body-weight = "bold"
        caption-size = 8.0
        caption-weight = "medium"
        hint-size = 48.0
        hint-weight = "normal"

        [command-palette]
        section-alignment = "center"
        "##,
    )
    .expect("theme should parse");
    let expected = theme.clone();
    let mut app = command_palette_scroll_app().theme(move |_| theme.clone());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let rendered = app
        .render_scene(window, size)
        .expect("open palette should render");
    let command = scene_text(rendered.scene(), "Palette One");
    let row = rendered
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.label_text() == Some("Palette One"))
        .expect("palette row should be laid out");
    let slots = layout::control::palette_row_slots(row.rect(), row.shortcut_width(), &expected);
    let shortcut = scene_text_in_rect(rendered.scene(), "R", slots.shortcut);
    let shortcut_icon = scene_icon_in_rect(rendered.scene(), "caret-up", slots.shortcut);

    assert_eq!(
        command.style().size(),
        expected.typography().interface().size()
    );
    assert_eq!(command.style().weight(), text::document::Weight::Medium);
    assert_eq!(command.color(), expected.text().primary);
    assert_eq!(
        shortcut.style().size(),
        expected.typography().interface().size()
    );
    assert_eq!(
        shortcut.style().weight(),
        expected.typography().interface().weight()
    );
    assert_ne!(shortcut.style().size(), expected.typography().body().size());
    assert_ne!(shortcut.style().size(), expected.typography().hint().size());
    assert_eq!(shortcut.color(), expected.text().muted);
    assert_eq!(shortcut.align(), scene::TextAlign::Start);
    assert_eq!(shortcut_icon.color(), expected.text().muted);
    assert!(
        row.shortcut_width() > 0 && row.shortcut_width() < 120,
        "palette shortcut slots should be measured from interface typography, not body or hint"
    );
}

#[test]
fn command_palette_formats_shortcuts_with_active_keymap_profile() {
    let mut app = command_palette_scroll_app().keymap(keymap::Profile::mac());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Primary+Shift+P"))
        .expect("mac palette shortcut should open");
    let rendered = app
        .render_scene(window, size)
        .expect("open palette should render");
    let row = rendered
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.label_text() == Some("Palette One"))
        .expect("palette row should be laid out");
    let slots =
        layout::control::palette_row_slots(row.rect(), row.shortcut_width(), &Theme::dark());

    assert!(scene_icon_in_rect(rendered.scene(), "command", slots.shortcut).size() > 0.0);
    assert_eq!(
        scene_text_in_rect(rendered.scene(), "R", slots.shortcut).align(),
        scene::TextAlign::Start
    );
    assert!(
        rendered
            .scene()
            .texts()
            .iter()
            .all(|text| text.value() != "Primary+R"),
        "semantic shortcuts must not paint directly"
    );
}

#[test]
fn shortcut_display_style_changes_paint_and_measure_together() {
    let platform_theme = Theme::dark();
    let text_theme = Theme::from_toml_str(
        r##"
        [shortcuts]
        display = "text"
        "##,
    )
    .expect("text shortcut display theme should parse");
    let mut platform_app = command_palette_scroll_app().theme(move |_| platform_theme.clone());
    let mut text_app = command_palette_scroll_app().theme(move |_| text_theme.clone());
    platform_app.start();
    text_app.start();

    let platform_window = platform_app.session().windows()[0].id();
    let text_window = text_app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    platform_app
        .handle_input(platform_window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette should open");
    text_app
        .handle_input(text_window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette should open");
    let platform = platform_app
        .render_scene(platform_window, size)
        .expect("platform palette should render");
    let text = text_app
        .render_scene(text_window, size)
        .expect("text palette should render");
    let platform_row = platform
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.label_text() == Some("Palette One"))
        .expect("platform palette row should be laid out");
    let text_row = text
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.label_text() == Some("Palette One"))
        .expect("text palette row should be laid out");
    let text_shortcut = scene_text(text.scene(), "Ctrl+R");

    assert_ne!(platform_row.shortcut_width(), text_row.shortcut_width());
    assert_eq!(text_shortcut.align(), scene::TextAlign::Start);
}

#[test]
fn command_palette_uses_panel_padding_and_content_gap() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let rendered = app
        .render_scene(window, size)
        .expect("open palette should render");
    let theme = Theme::dark();
    let panel = command_palette_panel_frame(&rendered);
    let query = rendered
        .layout()
        .find_role(view::node::Role::TextBox)
        .into_iter()
        .next()
        .expect("palette query should be laid out");
    let results = command_palette_results_frame(&rendered);

    assert_eq!(
        query.rect().y().saturating_sub(panel.rect().y()),
        theme.floating_panel().padding
    );
    assert_eq!(
        results.rect().y().saturating_sub(query.rect().bottom()),
        theme.floating_panel().content_gap
    );
}

#[test]
fn explicit_zero_floating_panel_gap_disables_default_content_gap() {
    let mut theme = Theme::dark();
    theme.floating_panel_mut().content_gap = 9;
    let view = widget::view(|ui| {
        ui.add(
            widget::panel::Floating::new("tests.gap.zero")
                .layout(|layout| layout.gap(0))
                .children(|ui| {
                    ui.label("Alpha");
                    ui.label("Beta");
                }),
        );
    });
    let mut engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(240, 160),
        &mut engine,
        &theme,
    );
    let labels = layout.find_role(view::node::Role::Label);

    assert_eq!(labels.len(), 2);
    assert_eq!(labels[1].rect().y(), labels[0].rect().bottom());
}

#[test]
fn command_palette_results_shrink_until_themed_cap_then_scroll() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let many = app
        .render_scene(window, size)
        .expect("open palette should render");
    let theme = Theme::dark();
    let many_results = command_palette_results_frame(&many);
    let many_viewport = many_results
        .viewport()
        .expect("results should expose viewport");

    assert_eq!(
        many_results.rect().height(),
        theme.command_palette().max_results_height()
    );
    assert!(many_viewport.content().height() > many_viewport.rect().height());

    app.handle_input(window, Input::text_commit("twelve"))
        .expect("typing should filter palette query");
    let few = app
        .render_scene(window, size)
        .expect("filtered palette should render");
    let few_results = command_palette_results_frame(&few);
    let few_viewport = few_results
        .viewport()
        .expect("results should expose viewport");

    assert!(few_results.rect().height() < theme.command_palette().max_results_height());
    assert_eq!(
        few_viewport.content().height(),
        few_viewport.rect().height()
    );
}

#[test]
fn command_palette_uses_centered_max_envelope_placement() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let expanded = app
        .render_scene(window, size)
        .expect("open palette should render");
    let expanded_panel = command_palette_panel_frame(&expanded);
    let expanded_top = expanded_panel.rect().y();

    assert_eq!(
        expanded_panel
            .rect()
            .x()
            .saturating_add(expanded_panel.rect().width() / 2),
        size.width() / 2
    );
    assert_eq!(
        expanded_panel.rect().y(),
        size.height().saturating_sub(expanded_panel.rect().height()) / 2
    );

    app.handle_input(window, Input::text_commit("twelve"))
        .expect("typing should filter palette query");
    let short = app
        .render_scene(window, size)
        .expect("filtered palette should render");
    let short_panel = command_palette_panel_frame(&short);

    assert_eq!(short_panel.rect().y(), expanded_top);
    assert!(short_panel.rect().height() < expanded_panel.rect().height());
}

#[test]
fn arrow_selection_scrolls_palette_result_into_view() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    for _ in 0..10 {
        app.handle_input(
            window,
            Input::key_down(input::Key::ArrowDown, input::Modifiers::default()),
        )
        .expect("palette arrow navigation should be handled");
    }

    let rendered = app
        .render_scene(window, size)
        .expect("palette should render after keyboard navigation");
    let results = command_palette_results_frame(&rendered);
    let selected = selected_palette_result_frame(&rendered);
    let viewport = results
        .viewport()
        .expect("results should expose viewport")
        .rect();

    assert!(selected.rect().y() >= viewport.y());
    assert!(selected.rect().bottom() <= viewport.bottom());
    assert_tint_quad(
        rendered.scene(),
        selected.rect(),
        Theme::default().menu().row_hover_tint,
    );
    assert!(
        app.session()
            .interaction(window)
            .expect("window should have interaction")
            .scroll()
            .offset(results.target().expect("results should expose a target"))
            .y()
            > 0,
        "arrow navigation should scroll the results viewport"
    );
}

#[test]
fn palette_arrow_navigation_reaches_last_command() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    for _ in 0..32 {
        app.handle_input(
            window,
            Input::key_down(input::Key::ArrowDown, input::Modifiers::default()),
        )
        .expect("palette arrow navigation should be handled");
    }

    let rendered = app
        .render_scene(window, size)
        .expect("palette should render after keyboard navigation");
    let results = command_palette_results_frame(&rendered);
    let selected = selected_palette_result_frame(&rendered);
    let viewport = results
        .viewport()
        .expect("results should expose viewport")
        .rect();

    assert_eq!(selected.label_text(), Some("Command Palette"));
    assert!(selected.rect().y() >= viewport.y());
    assert!(selected.rect().bottom() <= viewport.bottom());
}

#[test]
fn palette_reveal_uses_selected_frame_rect() {
    let mut theme = Theme::dark();
    theme.viewport_mut().reveal_margin = 12;
    let expected_margin = theme.viewport().reveal_margin;
    let mut app = command_palette_scroll_app().theme(move |_| theme.clone());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    for _ in 0..10 {
        app.handle_input(
            window,
            Input::key_down(input::Key::ArrowDown, input::Modifiers::default()),
        )
        .expect("palette arrow navigation should be handled");
    }

    let rendered = app
        .render_scene(window, size)
        .expect("palette should render after keyboard navigation");
    let results = command_palette_results_frame(&rendered);
    let selected = selected_palette_result_frame(&rendered);
    let viewport = results
        .viewport()
        .expect("results should expose viewport")
        .rect();

    assert_eq!(
        selected.rect().bottom(),
        viewport.bottom() - expected_margin
    );
}

#[test]
fn palette_query_keeps_focus_while_active_result_moves() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    app.handle_input(
        window,
        Input::key_down(input::Key::ArrowDown, input::Modifiers::default()),
    )
    .expect("palette arrow navigation should be handled");
    app.render_scene(window, size)
        .expect("palette should render after keyboard navigation");

    assert_eq!(
        app.session().focused(window),
        Some(interaction::CommandPalette::query_focus())
    );
}

#[test]
fn active_descendant_reveal_request_clears_after_resolution() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    app.handle_input(
        window,
        Input::key_down(input::Key::ArrowDown, input::Modifiers::default()),
    )
    .expect("palette arrow navigation should be handled");

    assert!(
        app.session()
            .interaction(window)
            .expect("window should have interaction")
            .scroll()
            .active_descendant_targets()
            .contains(&interaction::CommandPalette::results_target())
    );

    app.render_scene(window, size)
        .expect("palette should render after keyboard navigation");

    assert!(
        !app.session()
            .interaction(window)
            .expect("window should have interaction")
            .scroll()
            .active_descendant_targets()
            .contains(&interaction::CommandPalette::results_target())
    );
}

#[test]
fn label_means_visible_text() {
    let view = View::new(
        view::Node::root()
            .child(view::Node::panel().with_interaction_id("identity.only"))
            .child(view::Node::panel().with_label("Visible Panel")),
    );
    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(240, 120), &mut layout_engine);
    let scene = scene::Scene::paint(&layout);

    assert!(
        scene
            .texts()
            .iter()
            .any(|text| text.value() == "Visible Panel"),
        "labels are visible presentation text"
    );
    assert!(
        scene
            .texts()
            .iter()
            .all(|text| text.value() != "identity.only"),
        "interaction ids are invisible identity"
    );
}

#[test]
fn typography_metrics_affect_label_measurement() {
    let default = Theme::dark();
    let large = Theme::from_toml_str(
        r##"
        [typography]
        body-size = 28.0
        "##,
    )
    .expect("theme should parse");
    let view = View::new(
        view::Node::root().child(
            view::Node::panel()
                .with_style(view::Style::new().with_align_items(view::style::Align::Start))
                .child(view::Node::label("Typography")),
        ),
    );
    let mut engine = layout::engine::Engine::new();
    let default_layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(320, 120),
        &mut engine,
        &default,
    );
    let large_layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(320, 120),
        &mut engine,
        &large,
    );
    let default_label = default_layout
        .find_role(view::node::Role::Label)
        .into_iter()
        .next()
        .expect("default label should be laid out")
        .rect();
    let large_label = large_layout
        .find_role(view::node::Role::Label)
        .into_iter()
        .next()
        .expect("large label should be laid out")
        .rect();

    assert!(
        large_label.width() > default_label.width()
            || large_label.height() > default_label.height(),
        "type size is a layout-visible metric, not paint-only appearance"
    );
}

#[test]
fn interface_metrics_affect_system_widgets_without_body_metrics() {
    let default = Theme::dark();
    let body_large = Theme::from_toml_str(
        r##"
        [typography]
        body-size = 28.0
        "##,
    )
    .expect("body-large theme should parse");
    let interface_large = Theme::from_toml_str(
        r##"
        [typography]
        interface-size = 18.0
        interface-weight = "bold"
        "##,
    )
    .expect("interface-large theme should parse");
    let view = widget::view(|ui| {
        ui.column(|ui| {
            ui.add(
                widget::Element::new()
                    .row()
                    .height(view::style::Dimension::fit())
                    .children(|ui| {
                        ui.button(widget::Button::new("System Command"));
                    }),
            );
            ui.checkbox(widget::Checkbox::new("System Choice", true));
            ui.text_box(widget::TextBox::new("").placeholder("Find"));
            ui.add(
                widget::Element::new()
                    .row()
                    .height(view::style::Dimension::fit())
                    .children(|ui| {
                        ui.label("Content Label");
                    }),
            );
        });
    });
    let mut engine = layout::engine::Engine::new();
    let default_layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(420, 160),
        &mut engine,
        &default,
    );
    let body_layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(420, 160),
        &mut engine,
        &body_large,
    );
    let interface_layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(420, 160),
        &mut engine,
        &interface_large,
    );
    let default_scene = scene::Scene::paint_with_theme(&default_layout, &default);
    let body_scene = scene::Scene::paint_with_theme(&body_layout, &body_large);
    let interface_scene = scene::Scene::paint_with_theme(&interface_layout, &interface_large);
    let default_button = default_layout
        .find_role(view::node::Role::Button)
        .into_iter()
        .next()
        .expect("default button should be laid out")
        .rect();
    let body_button = body_layout
        .find_role(view::node::Role::Button)
        .into_iter()
        .next()
        .expect("body-large button should be laid out")
        .rect();
    let interface_button = interface_layout
        .find_role(view::node::Role::Button)
        .into_iter()
        .next()
        .expect("interface-large button should be laid out")
        .rect();
    let default_label = default_layout
        .find_role(view::node::Role::Label)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Content Label"))
        .expect("default content label should be laid out")
        .rect();
    let body_label = body_layout
        .find_role(view::node::Role::Label)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Content Label"))
        .expect("body-large content label should be laid out")
        .rect();
    let interface_label = interface_layout
        .find_role(view::node::Role::Label)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Content Label"))
        .expect("interface-large content label should be laid out")
        .rect();

    assert_eq!(body_button.width(), default_button.width());
    assert!(interface_button.width() > default_button.width());
    assert_eq!(body_button.height(), default.control().height);
    assert_eq!(interface_button.height(), interface_large.control().height);
    assert!(body_label.width() > default_label.width());
    assert_eq!(interface_label.width(), default_label.width());

    assert_eq!(
        scene_text(&body_scene, "System Command").style().size(),
        body_large.typography().interface().size()
    );
    assert_eq!(
        scene_text(&interface_scene, "System Command")
            .style()
            .weight(),
        text::document::Weight::Bold
    );
    assert_eq!(
        scene_text(&interface_scene, "Find").style().size(),
        interface_large.typography().interface().size()
    );
    assert_eq!(
        scene_text(&default_scene, "Content Label").style().size(),
        default.typography().body().size()
    );
}

#[test]
fn scroll_target_at_ignores_clipped_viewports() {
    let mut app = nested_clipped_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 180);
    scroll_outer_until_inner_overlaps_search(&mut app, window, size);
    let rendered = app
        .render_scene(window, size)
        .expect("nested clipped scroll should render");
    let inner = scroll_frame_with_label(&rendered, "Inner Scroll");
    let point = rect_top_point(
        inner
            .viewport()
            .expect("inner scroll should expose viewport")
            .rect(),
    );

    assert!(
        !inner.clip_contains(point),
        "inner viewport acquisition point should be clipped by the outer viewport"
    );
    assert_eq!(
        rendered
            .layout()
            .scroll_target_at(point, interaction::ScrollDelta::vertical(24)),
        None
    );
}

#[test]
fn chrome_hit_respects_owner_ancestor_clip() {
    let mut app = nested_clipped_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 180);
    scroll_outer_until_inner_overlaps_search(&mut app, window, size);
    let rendered = app
        .render_scene(window, size)
        .expect("nested clipped scroll should render");
    let inner = scroll_frame_with_label(&rendered, "Inner Scroll");
    let inner_target = inner
        .target()
        .expect("inner scroll should expose a target")
        .clone();
    let inner_chrome = rendered
        .layout()
        .chrome()
        .iter()
        .find(|chrome| chrome.scroll_target() == &inner_target)
        .expect("inner scroll should project chrome");
    let track = match inner_chrome.kind() {
        layout::chrome::Kind::Scrollbar(scrollbar) => scrollbar.track(),
    };
    let point = rect_top_point(track);

    assert!(
        !inner.clip_contains(point),
        "inner scrollbar should be outside its owner's ancestor clip"
    );
    assert_ne!(
        rendered
            .layout()
            .hit_test(point)
            .and_then(|hit| hit.target().cloned()),
        Some(inner_chrome.target().clone())
    );
}

#[test]
fn scrollbar_drag_does_not_dismiss_owning_palette() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.render_scene(window, size)
        .expect("initial palette app should render");
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let initial = app
        .render_scene(window, size)
        .expect("open palette should render");
    let results = command_palette_results_frame(&initial);
    let target = results
        .target()
        .expect("palette results should expose a scroll target")
        .clone();
    let track = initial
        .layout()
        .chrome()
        .iter()
        .find(|chrome| chrome.scroll_target() == &target)
        .and_then(|chrome| match chrome.kind() {
            layout::chrome::Kind::Scrollbar(scrollbar) => Some(scrollbar.track()),
        })
        .expect("palette results should project scrollbar chrome");
    let press = frame_point_at(track);
    let drag = geometry::Point::new(
        track.x().saturating_add(track.width() / 2),
        track.bottom().saturating_sub(1),
    );

    app.pointer_down_at(window, size, press)
        .expect("palette scrollbar pointer down should be handled");

    assert!(
        app.session()
            .interaction(window)
            .and_then(interaction::Interaction::command_palette)
            .is_some(),
        "palette should stay open on its own scrollbar press"
    );
    assert_eq!(
        app.session().focused(window),
        Some(interaction::CommandPalette::query_focus()),
        "palette scrollbar should not steal query focus"
    );

    app.pointer_drag_at(window, size, drag)
        .expect("palette scrollbar drag should be handled");

    assert!(
        app.session()
            .interaction(window)
            .and_then(interaction::Interaction::command_palette)
            .is_some(),
        "palette should stay open while dragging its own scrollbar"
    );
    assert!(
        app.session()
            .interaction(window)
            .expect("window should have interaction")
            .scroll()
            .offset(&target)
            .y()
            > 0,
        "palette scrollbar drag should update the results scroll offset"
    );
    assert_eq!(
        app.session().focused(window),
        Some(interaction::CommandPalette::query_focus())
    );
}

#[test]
fn layout_hit_testing_uses_stable_identity_and_topmost_popup_order() {
    let duplicate = View::new(
        view::Node::root().child(
            view::Node::menu_bar()
                .child(view::Node::menu("menu.first", "Same"))
                .child(view::Node::menu("menu.second", "Same")),
        ),
    );
    let mut layout_engine = layout::engine::Engine::new();
    let duplicate_layout = layout::Layout::compose(
        &duplicate,
        geometry::Size::new(320, 120),
        &mut layout_engine,
    );
    let duplicate_menus = duplicate_layout.find_role(view::node::Role::Menu);

    assert_eq!(duplicate_menus.len(), 2);
    assert_eq!(duplicate_menus[0].label_text(), Some("Same"));
    assert_eq!(duplicate_menus[1].label_text(), Some("Same"));
    assert_ne!(duplicate_menus[0].target(), duplicate_menus[1].target());

    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let file = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should be in the view");

    app.handle_view(
        window,
        file.menu_action().expect("menu should expose an action"),
    )
    .expect("menu action should be handled");

    let projected = app
        .present(window)
        .expect("open menu should project a popup");
    let layout = layout::Layout::compose(
        &projected,
        geometry::Size::new(800, 600),
        &mut layout_engine,
    );
    let popup_hit = layout
        .hit_test(geometry::Point::new(10, 34))
        .expect("popup should be hit above the text area");

    assert_eq!(popup_hit.frame().role(), view::node::Role::Binding);
    assert_eq!(popup_hit.frame().label_text(), Some("New"));
    assert_eq!(
        popup_hit
            .target()
            .expect("popup command should expose a target")
            .kind(),
        interaction::target::Kind::Command
    );
}

#[test]
fn runtime_host_pointer_coordinates_route_to_view_actions() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    app.render_scene(window, size)
        .expect("initial scene should install a composition");

    let hit = app
        .hit_test(window, size, geometry::Point::new(10, 10))
        .expect("file menu should be hit");
    assert_eq!(hit.frame().role(), view::node::Role::Menu);
    assert_eq!(hit.frame().label_text(), Some("File"));

    let moved = app
        .pointer_move_at(window, size, geometry::Point::new(10, 10))
        .expect("coordinate pointer move should be handled");
    assert!(moved.is_handled());

    let pressed = app
        .pointer_down_at(window, size, geometry::Point::new(10, 10))
        .expect("coordinate pointer down should be handled");
    assert!(pressed.is_handled());

    let released = app
        .pointer_up_at(window, size, geometry::Point::new(10, 10))
        .expect("coordinate pointer up should be handled");
    assert!(released.is_handled());
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.open_menu())
            .map(|menu| menu.label()),
        Some("File")
    );
}

#[test]
fn runtime_host_scroll_coordinates_route_to_scroll_target() {
    let document = (0..120)
        .map(|line| format!("scroll line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text(document),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let presentation = app
        .render_scene(window, size)
        .expect("initial scene should install a composition");
    let text_area = presentation
        .layout()
        .find_role(view::node::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let target = text_area
        .target()
        .expect("text area should expose a scroll target")
        .clone();
    let point = geometry::Point::new(text_area.rect().x() + 4, text_area.rect().y() + 4);

    let outcome = app
        .scroll_at(window, size, point, interaction::ScrollDelta::vertical(96))
        .expect("coordinate scroll should be handled");

    assert!(outcome.is_handled());
    assert!(outcome.effect().contains_invalidation());
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .scroll()
            .offset(&target),
        interaction::ScrollOffset::new(0, 96)
    );

    let scrolled = app
        .render_scene(window, size)
        .expect("scrolled scene should render");
    let text_area = scrolled
        .layout()
        .find_role(view::node::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out after scrolling");
    assert_eq!(
        text_area
            .text_area_layout()
            .expect("text area should use text area layout")
            .layout()
            .scroll_y(),
        96.0
    );
}

#[test]
fn platform_wheel_down_scroll_moves_text_area_content_up() {
    use winit::event::MouseScrollDelta;

    let document = (0..160)
        .map(|line| format!("direction line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text(document),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(520, 180);
    let initial = app
        .render_scene(window, size)
        .expect("initial scene should render");
    let initial_y = first_visible_text_area_surface_y(&initial);
    let text_area = initial
        .layout()
        .find_role(view::node::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let target = text_area
        .target()
        .expect("text area should expose a scroll target")
        .clone();
    let point = geometry::Point::new(text_area.rect().x() + 8, text_area.rect().y() + 8);
    let wheel_down = platform::scroll_delta(MouseScrollDelta::LineDelta(0.0, -16.0), 1.0);

    let outcome = app
        .scroll_at(window, size, point, wheel_down)
        .expect("wheel scroll should route to text area");
    assert!(outcome.is_handled());
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should retain interaction state")
            .scroll()
            .offset(&target),
        interaction::ScrollOffset::new(0, 448)
    );

    let scrolled = app
        .render_scene(window, size)
        .expect("scrolled scene should render");
    let scrolled_y = first_visible_text_area_surface_y(&scrolled);
    let scroll_y = scrolled
        .layout()
        .find_role(view::node::Role::TextArea)
        .first()
        .and_then(|frame| frame.text_area_layout())
        .map(|text_area| text_area.layout().scroll_y())
        .expect("text area should have a text layout");

    assert!(scroll_y > 0.0);
    assert!(scrolled_y < initial_y);
}

#[test]
fn text_area_render_writes_back_clamped_scroll_offset() {
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text("short\ntext"),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let presentation = app
        .render_scene(window, size)
        .expect("initial scene should install a composition");
    let text_area = presentation
        .layout()
        .find_role(view::node::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let target = text_area
        .target()
        .expect("text area should expose a scroll target")
        .clone();
    let point = geometry::Point::new(text_area.rect().x() + 4, text_area.rect().y() + 4);

    app.scroll_at(
        window,
        size,
        point,
        interaction::ScrollDelta::vertical(4_000),
    )
    .expect("coordinate scroll should be handled");
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .scroll()
            .offset(&target),
        interaction::ScrollOffset::new(0, 4_000)
    );

    let clamped = app
        .render_scene(window, size)
        .expect("clamped scene should render");

    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .scroll()
            .offset(&target),
        interaction::ScrollOffset::default()
    );
    assert_eq!(
        app.diagnostics(window)
            .expect("window should have diagnostics after clamping")
            .scroll
            .frame_scroll_commits,
        1
    );
    let text_area = clamped
        .layout()
        .find_role(view::node::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out after clamping");
    assert_eq!(
        text_area
            .text_area_layout()
            .expect("text area should use text area layout")
            .layout()
            .scroll_y(),
        0.0
    );
}

#[test]
fn text_area_caret_reveal_resolves_framework_owned_scroll_after_edit() {
    let document = (0..120)
        .map(|line| format!("reveal line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text(document),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let focus = app
        .present(window)
        .expect("initial view should present")
        .text_areas()[0]
        .focus()
        .expect("text area should declare focus");
    let presentation = app
        .render_scene(window, size)
        .expect("initial scene should install a composition");
    let text_area = presentation
        .layout()
        .find_role(view::node::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let target = text_area
        .target()
        .expect("text area should expose a scroll target")
        .clone();
    let point = geometry::Point::new(text_area.rect().x() + 4, text_area.rect().y() + 4);

    app.scroll_at(window, size, point, interaction::ScrollDelta::vertical(240))
        .expect("coordinate scroll should be handled");
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .scroll()
            .offset(&target),
        interaction::ScrollOffset::new(0, 240)
    );

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    let moved = app
        .handle_input(
            window,
            Input::text_edit(text::edit::Edit::set_position(text::buffer::Position::new(
                0,
            ))),
        )
        .expect("caret move should be handled");

    assert!(moved.is_handled());
    assert!(moved.changed_state());
    assert!(moved.effect().contains_invalidation());

    let revealed = app
        .render_scene(window, size)
        .expect("revealed scene should render");

    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .scroll()
            .offset(&target),
        interaction::ScrollOffset::default()
    );
    let text_area = revealed
        .layout()
        .find_role(view::node::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out after reveal");
    assert_eq!(
        text_area
            .text_area_layout()
            .expect("text area should use text area layout")
            .layout()
            .scroll_y(),
        0.0
    );
}

#[test]
fn text_editor_layout_paints_to_renderer_neutral_scene() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    app.invoke(app.trigger::<text_editor::ToggleDebugPanel>(()))
        .output
        .expect("debug panel toggle should resolve");
    app.invoke(app.trigger::<text_editor::LoadStressText>(()))
        .output
        .expect("stress text command should resolve");
    let projected = app.present(window).expect("window should have a view");
    let revision = app.revision();
    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(
        &projected,
        geometry::Size::new(800, 600),
        &mut layout_engine,
    );
    let _: &Scene = &scene::Scene::paint(&layout);
    let scene = scene::Scene::paint(&layout);
    assert_eq!(scene.size(), layout.size());
    assert!(!scene.is_empty());
    assert!(
        scene
            .texts()
            .iter()
            .any(|text| text.value().contains("File"))
    );
    assert!(
        scene
            .texts()
            .iter()
            .any(|text| text.value().contains("loaded Unicode stress fixture"))
    );
    assert!(
        scene
            .text_viewports()
            .iter()
            .any(|viewport| !viewport.surfaces().is_empty())
    );
    assert!(scene.quads().iter().any(|quad| {
        quad.fill().channels() == (28, 28, 30, 255)
            && layout
                .find_role(view::node::Role::TextArea)
                .iter()
                .any(|frame| frame.rect() == quad.rect())
    }));
    assert_eq!(scene.clear().channels(), (17, 18, 20, 255));
    assert_eq!(app.revision(), revision);
}

#[test]
fn text_area_selection_highlight_is_clipped_to_text_area_viewport() {
    let text = (0..180)
        .map(|line| format!("highlight line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut document = TextDocument::from_multiline_text(text);
    let mut clipboard = Clipboard::default();
    let selected = document.apply_action(text::edit::Action::SelectAll, &mut clipboard);

    assert!(selected.selection_changed());

    let mut app = text_editor::app(text_editor::State {
        document,
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(520, 180);
    let initial = app
        .render_scene(window, size)
        .expect("initial scene should render");
    let text_area = initial
        .layout()
        .find_role(view::node::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let target = text_area
        .target()
        .expect("text area should expose a scroll target")
        .clone();
    let point = geometry::Point::new(text_area.rect().x() + 8, text_area.rect().y() + 8);

    app.scroll_at(window, size, point, interaction::ScrollDelta::vertical(220))
        .expect("scroll should route to text area");

    let scrolled = app
        .render_scene(window, size)
        .expect("scrolled scene should render");
    let text_area_rect = scrolled
        .layout()
        .find_role(view::node::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out after scrolling")
        .rect();
    let highlights = scrolled
        .scene()
        .quads()
        .into_iter()
        .filter(|quad| quad.fill().channels() == (10, 132, 255, 96))
        .collect::<Vec<_>>();

    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should retain interaction state")
            .scroll()
            .offset(&target),
        interaction::ScrollOffset::new(0, 220)
    );
    assert!(!highlights.is_empty());

    for highlight in highlights {
        assert!(
            rect_contains(text_area_rect, highlight.rect()),
            "selection highlight should stay inside text area: bounds {:?}, highlight {:?}",
            text_area_rect,
            highlight.rect()
        );
    }
}

#[test]
fn text_area_selection_highlight_paints_below_menu_bar_chrome() {
    let text = (0..24)
        .map(|line| format!("selected line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut document = TextDocument::from_multiline_text(text);
    let mut clipboard = Clipboard::default();
    document.apply_action(text::edit::Action::SelectAll, &mut clipboard);

    let mut app = text_editor::app(text_editor::State {
        document,
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let rendered = app
        .render_scene(window, geometry::Size::new(520, 180))
        .expect("selected text area should render");
    let primitives = rendered.scene().primitives();
    let menu_bar_rect = rendered
        .layout()
        .find_role(view::node::Role::MenuBar)
        .into_iter()
        .next()
        .expect("menu bar should be laid out")
        .rect();
    let highlight = primitives
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Quad(quad)
                    if quad.fill().channels() == (10, 132, 255, 96)
            )
        })
        .expect("selection highlight should be painted");
    let menu_bar_chrome = primitives
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Quad(quad)
                    if quad.fill().channels() == (28, 28, 30, 255)
                        && quad.rect() == menu_bar_rect
            )
        })
        .expect("menu bar chrome should be painted");
    let file_menu_text = primitives
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Text(text) if text.value() == "File"
            )
        })
        .expect("menu bar file text should be painted");

    assert!(
        highlight < menu_bar_chrome,
        "selection highlight should paint below menu bar background"
    );
    assert!(
        highlight < file_menu_text,
        "selection highlight should paint below menu bar text"
    );
}

#[test]
fn text_editor_wrap_command_changes_text_area_paint_wrap() {
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text("alpha beta gamma"),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let wrapped = app
        .render_scene(window, geometry::Size::new(320, 180))
        .expect("wrapped scene should render");

    assert_eq!(
        wrapped
            .layout()
            .find_role(view::node::Role::TextArea)
            .first()
            .and_then(|frame| frame.text_wrap()),
        Some(view::control::Wrap::Word)
    );
    assert!(!wrapped.scene().text_viewports().is_empty());

    app.invoke(app.trigger::<text_editor::ToggleWrapText>(()))
        .output
        .expect("wrap toggle should resolve");

    let unwrapped = app
        .render_scene(window, geometry::Size::new(320, 180))
        .expect("unwrapped scene should render");

    assert_eq!(
        unwrapped
            .layout()
            .find_role(view::node::Role::TextArea)
            .first()
            .and_then(|frame| frame.text_wrap()),
        Some(view::control::Wrap::None)
    );
    assert!(!unwrapped.scene().text_viewports().is_empty());
}

#[test]
fn scene_paints_controls_from_semantic_state() {
    let view = widget::view(|ui| {
        ui.column(|ui| {
            ui.checkbox(widget::Checkbox::new("Wrap", true));
            ui.radio(widget::Radio::new("Soft tabs", true));
            ui.slider(widget::Slider::new("Zoom", 5.0, 0.0..=10.0));
        });
    });
    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(320, 120), &mut layout_engine);
    let scene = scene::Scene::paint(&layout);
    let checkbox = layout
        .find_role(view::node::Role::Checkbox)
        .into_iter()
        .next()
        .expect("checkbox should be laid out");
    let radio = layout
        .find_role(view::node::Role::Radio)
        .into_iter()
        .next()
        .expect("radio should be laid out");
    let slider = layout
        .find_role(view::node::Role::Slider)
        .into_iter()
        .next()
        .expect("slider should be laid out");

    assert!(scene.texts().iter().any(|text| text.value() == "Wrap"));
    assert!(scene.texts().iter().any(|text| text.value() == "Soft tabs"));
    assert!(
        scene
            .texts()
            .iter()
            .any(|text| text.value().starts_with("Zoom: 5.00"))
    );
    assert!(
        scene
            .texts()
            .iter()
            .all(|text| !text.value().contains("..")),
        "slider range should not wrap into clipped control text"
    );
    assert!(
        scene
            .texts()
            .iter()
            .all(|text| !text.value().starts_with("[") && !text.value().starts_with("(")),
        "control state should be painted, not encoded in labels"
    );
    assert!(scene.icons().iter().any(|icon| {
        icon.icon().id().as_str() == "check" && rect_contains(checkbox.rect(), icon.rect())
    }));
    assert!(scene.quads().iter().any(|quad| {
        quad.fill().channels() == (245, 245, 247, 255)
            && rect_contains(checkbox.rect(), quad.rect())
            && quad.rounding() == scene::Rounding::fixed(4.0)
    }));
    assert!(scene.quads().iter().any(|quad| {
        quad.fill().channels() == (10, 132, 255, 255)
            && rect_contains(radio.rect(), quad.rect())
            && quad.rounding() == scene::Rounding::relative(1.0)
    }));
    assert!(scene.quads().iter().any(|quad| {
        quad.fill().channels() == (58, 58, 60, 255)
            && rect_contains(slider.rect(), quad.rect())
            && quad.rect().height() == 4
    }));
    assert!(scene.quads().iter().any(|quad| {
        quad.fill().channels() == (10, 132, 255, 255)
            && rect_contains(slider.rect(), quad.rect())
            && quad.rect().height() == 4
    }));
    assert!(scene.quads().iter().any(|quad| {
        quad.fill().channels() == (245, 245, 247, 255)
            && rect_contains(slider.rect(), quad.rect())
            && quad.rounding() == scene::Rounding::relative(1.0)
    }));
}

#[test]
fn choice_marks_paint_pressed_tint_above_mark_without_label_overlay() {
    let mut app = Runtime::new(SourceState::default())
        .commands(|commands| {
            commands.register::<RecordSource>(command::Spec::new("Record"));
        })
        .responders(|responders| {
            responders.app().target::<RecordSource>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Choice Pressed Paint"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.checkbox(widget::Checkbox::new("Wrap", true).trigger::<RecordSource>(()));
                    ui.radio(widget::Radio::new("Soft tabs", true).trigger::<RecordSource>(()));
                });
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 120);
    let initial = app
        .render_scene(window, size)
        .expect("choice view should render");
    let checkbox = initial
        .layout()
        .find_role(view::node::Role::Checkbox)
        .into_iter()
        .next()
        .expect("checkbox should be laid out");
    let checkbox_point = frame_point_at(checkbox.active_rect());

    let checkbox_down = app
        .pointer_down_at(window, size, checkbox_point)
        .expect("checkbox pointer down should be handled");
    assert!(checkbox_down.is_handled());
    let checkbox_pressed = app
        .render_scene(window, size)
        .expect("checkbox pressed state should render");
    let pressed_checkbox = checkbox_pressed
        .layout()
        .find_role(view::node::Role::Checkbox)
        .into_iter()
        .next()
        .expect("pressed checkbox should be laid out");
    assert!(
        pressed_checkbox.is_pressed(),
        "checkbox frame should project pressed state"
    );
    assert!(pressed_checkbox.is_enabled());
    assert_choice_pressed_tint_above_mark(&checkbox_pressed, pressed_checkbox.active_rect());
    assert_no_choice_label_overlay(&checkbox_pressed, pressed_checkbox.rect());

    app.handle_input(window, Input::cancel())
        .expect("cancel should reset checkbox pressed state");
    let released = app
        .render_scene(window, size)
        .expect("choice view should render after checkbox release");
    let radio = released
        .layout()
        .find_role(view::node::Role::Radio)
        .into_iter()
        .next()
        .expect("radio should be laid out");
    let radio_point = frame_point_at(radio.active_rect());

    let radio_down = app
        .pointer_down_at(window, size, radio_point)
        .expect("radio pointer down should be handled");
    assert!(radio_down.is_handled());
    let radio_pressed = app
        .render_scene(window, size)
        .expect("radio pressed state should render");
    let pressed_radio = radio_pressed
        .layout()
        .find_role(view::node::Role::Radio)
        .into_iter()
        .next()
        .expect("pressed radio should be laid out");
    assert!(
        pressed_radio.is_pressed(),
        "radio frame should project pressed state"
    );
    assert!(pressed_radio.is_enabled());
    assert_choice_pressed_tint_above_mark(&radio_pressed, pressed_radio.active_rect());
    assert_no_choice_label_overlay(&radio_pressed, pressed_radio.rect());
}

#[test]
fn scene_paint_accepts_theme_data_variants() {
    let view = widget::view(|ui| {
        ui.button(widget::Button::new("Action"));
    });
    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(180, 60), &mut layout_engine);
    let dark = scene::Scene::paint(&layout);
    let light_theme = Theme::light();
    let light = scene::Scene::paint_with_theme(&layout, &light_theme);
    let root = geometry::Rect::new(0, 0, 180, 60);
    let dark_root = dark
        .quads()
        .into_iter()
        .find(|quad| quad.rect() == root)
        .expect("dark scene should paint the root");
    let light_root = light
        .quads()
        .into_iter()
        .find(|quad| quad.rect() == root)
        .expect("light scene should paint the root");

    assert_eq!(light.clear(), light_theme.surfaces().canvas);
    assert_ne!(dark.clear(), light.clear());
    assert_ne!(dark_root.fill(), light_root.fill());
}

#[test]
fn theme_toml_tokens_drive_layout_and_scene_primitives() {
    let theme = Theme::from_toml_str(
        r##"
        [palette]
        brand = "#112233"

        [surfaces]
        root = "brand"

        [text]
        primary = "#445566"

        [typography]
        interface-size = 13.5
        interface-weight = "bold"

        [control]
        button-background = "#334455"
        rounding = { fixed = 9.0 }
        height = 30

        [menu]
        bar-background = "#010203"
        bar-height = 34
        row-height = 34

        [floating-panel]
        material = { kind = "glass", recipe = "panel-dark", blur-sigma = 24.0, tint = { from = "#22334488", to = "#33445599" }, tint-opacity = 1.0, refraction-displacement = 3.0, refraction-splay = 1.5, refraction-feather = 12.0, refraction-curve = 2.5 }
        rounding = { fixed = 13.0 }
        padding = 10
        "##,
    )
    .expect("theme TOML should parse");
    let view = View::new(
        view::Node::root()
            .child(
                view::Node::stack(view::node::Axis::Vertical)
                    .child(view::Node::menu_bar().child(view::Node::menu("menu.file", "File")))
                    .child(view::Node::button("Run")),
            )
            .child(view::Node::floating_panel("panel").child(view::Node::label("Item"))),
    );
    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(220, 120),
        &mut layout_engine,
        &theme,
    );
    let scene = scene::Scene::paint_with_theme(&layout, &theme);
    let menu_bar = layout
        .find_role(view::node::Role::MenuBar)
        .into_iter()
        .next()
        .expect("menu bar should be laid out");
    let button = layout
        .find_role(view::node::Role::Button)
        .into_iter()
        .next()
        .expect("button should be laid out");
    let popup = layout
        .find_role(view::node::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("popup should be laid out");
    let item = layout
        .find_role(view::node::Role::Label)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Item"))
        .expect("popup item should be laid out");

    assert_eq!(menu_bar.rect().height(), 34);
    assert_eq!(button.rect().height(), 30);
    assert_eq!(
        popup.rect().height(),
        item.rect()
            .height()
            .saturating_add(theme.floating_panel().padding.saturating_mul(2))
    );
    assert!(scene.quads().iter().any(|quad| {
        quad.rect() == menu_bar.rect() && quad.fill() == scene::Color::rgb(1, 2, 3)
    }));
    let menu_title = scene_text(&scene, "File");
    assert_eq!(menu_title.style().size(), 13.5);
    assert_eq!(menu_title.style().weight(), text::document::Weight::Bold);
    assert!(scene.quads().iter().any(|quad| {
        quad.rect() == button.rect()
            && quad.fill() == scene::Color::rgb(51, 68, 85)
            && quad.rounding() == scene::Rounding::fixed(9.0)
    }));
    assert!(
        scene
            .texts()
            .iter()
            .any(|text| text.value() == "Run" && text.color() == scene::Color::rgb(68, 85, 102))
    );
    assert!(scene.filters().iter().any(|filter| {
        filter.rect() == popup.rect()
            && matches!(
                filter.ops(),
                [scene::FilterOp::BackdropBlur(blur)] if blur.sigma() == 24.0
            )
            && filter.rounding() == scene::Rounding::fixed(13.0)
    }));
    assert!(scene.filters().iter().any(|filter| {
        filter.rect() == popup.rect()
            && matches!(
                filter.ops(),
                [scene::FilterOp::Refraction(refraction)] if refraction.displacement() == 3.0
                    && refraction.splay() == 1.5
                    && refraction.feather() == 12.0
                    && refraction.curve() == 2.5
            )
            && filter.rounding() == scene::Rounding::fixed(13.0)
    }));
    assert!(scene.quads().iter().any(|quad| {
        quad.rect() == popup.rect()
            && quad.style().fill()
                == Some(scene::Brush::linear_gradient(
                    scene::Color::rgba(34, 51, 68, 136),
                    scene::Color::rgba(51, 68, 85, 153),
                ))
            && quad.rounding() == scene::Rounding::fixed(13.0)
    }));
}

#[test]
fn menu_bar_labels_are_center_aligned() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let rendered = app
        .render_scene(window, geometry::Size::new(520, 180))
        .expect("text editor should render");
    let file = rendered
        .scene()
        .texts()
        .into_iter()
        .find(|text| text.value() == "File")
        .expect("file menu label should paint");

    assert_eq!(file.align(), scene::TextAlign::Center);
}

#[test]
fn open_menu_projects_menu_bar_state_without_popup_title_text() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let file_menu = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should exist")
        .menu_action()
        .expect("file menu should have an action");

    app.handle_view(window, file_menu)
        .expect("menu action should be handled");

    let rendered = app
        .render_scene(window, geometry::Size::new(800, 600))
        .expect("open file menu should render");
    let menu_bar = rendered
        .layout()
        .find_role(view::node::Role::MenuBar)
        .into_iter()
        .next()
        .expect("menu bar should be laid out");
    let file = rendered
        .layout()
        .find_role(view::node::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("File"))
        .expect("file menu should be laid out");
    let edit = rendered
        .layout()
        .find_role(view::node::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Edit"))
        .expect("edit menu should be laid out");

    assert!(menu_bar.is_active());
    assert!(file.is_active());
    assert!(!edit.is_active());
    assert_eq!(
        rendered
            .scene()
            .texts()
            .into_iter()
            .filter(|text| text.value() == "File")
            .count(),
        1
    );
}

#[test]
fn control_gallery_example_renders_interactive_widget_scene() {
    let mut app = control_gallery::app(control_gallery::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let rendered = app
        .render_scene(window, geometry::Size::new(760, 520))
        .expect("control gallery should render");
    let scene = rendered.scene();

    assert!(scene.texts().iter().any(|text| text.value() == "Controls"));
    assert!(scene.texts().iter().any(|text| text.value() == "Wrap text"));
    assert!(scene.texts().iter().any(|text| text.value() == "Design"));
    assert!(
        scene
            .texts()
            .iter()
            .any(|text| text.value().starts_with("Level: 42.00"))
    );
    assert!(
        scene
            .icons()
            .iter()
            .any(|icon| icon.icon().id().as_str() == "check")
    );
    assert!(scene.quads().iter().any(|quad| {
        quad.fill().channels() == (10, 132, 255, 255)
            && quad.rounding() == scene::Rounding::relative(1.0)
    }));
}

#[test]
fn control_gallery_choice_labels_are_single_line_row_content() {
    let mut app = control_gallery::app(control_gallery::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let rendered = app
        .render_scene(window, geometry::Size::new(760, 520))
        .expect("control gallery should render");

    for (role, label) in [
        (view::node::Role::Checkbox, "Wrap text"),
        (view::node::Role::Checkbox, "Show grid"),
        (view::node::Role::Checkbox, "Advanced"),
        (view::node::Role::Radio, "Design"),
        (view::node::Role::Radio, "Inspect"),
        (view::node::Role::Radio, "Preview"),
    ] {
        let frame = rendered
            .layout()
            .find_role(role)
            .into_iter()
            .find(|frame| frame.label_text() == Some(label))
            .expect("choice frame should be laid out");
        let text = rendered
            .scene()
            .texts()
            .into_iter()
            .find(|text| text.value() == label)
            .expect("choice label should paint");

        assert_eq!(text.wrap(), scene::TextWrap::None);
        assert_eq!(text.rect().y(), frame.rect().y());
        assert_eq!(text.rect().height(), frame.rect().height());
        assert!(text.rect().x() >= frame.active_rect().right());
        assert!(text.rect().right() <= frame.rect().right());
    }
}

#[test]
fn menu_popup_rows_use_row_layout_for_labels_shortcuts_and_separators() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let file_menu = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should exist")
        .menu_action()
        .expect("file menu should have an action");

    app.handle_view(window, file_menu)
        .expect("menu action should be handled");

    let rendered = app
        .render_scene(window, geometry::Size::new(800, 600))
        .expect("open file menu should render");
    let exit = rendered
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.role() == view::node::Role::Binding && frame.label_text() == Some("Exit")
        })
        .expect("exit row should be laid out");
    let theme = Theme::default();
    let slots = layout::control::menu_row_slots(exit.rect(), exit.shortcut_width(), &theme);
    let exit_label = rendered
        .scene()
        .texts()
        .into_iter()
        .find(|text| text.value() == "Exit")
        .expect("exit label should paint");
    let exit_shortcut = rendered
        .scene()
        .texts()
        .into_iter()
        .find(|text| text.value() == "F4" && rect_contains(slots.shortcut, text.rect()))
        .expect("exit shortcut key should paint");
    let exit_shortcut_icon = scene_icon_in_rect(rendered.scene(), "option", slots.shortcut);

    assert_eq!(exit_label.rect(), slots.label);
    assert_eq!(exit_label.align(), scene::TextAlign::Start);
    assert!(rect_contains(slots.shortcut, exit_shortcut.rect()));
    assert_eq!(exit_shortcut.align(), scene::TextAlign::Start);
    assert_eq!(
        exit_shortcut.style().size(),
        theme.typography().interface().size()
    );
    assert_eq!(
        exit_shortcut.style().weight(),
        theme.typography().interface().weight()
    );
    assert_eq!(exit_shortcut.color(), theme.text().muted);
    assert_eq!(exit_shortcut_icon.color(), theme.text().muted);
    assert_eq!(slots.glyph.width(), slots.glyph.height());
    assert_eq!(slots.trailing.width(), slots.trailing.height());

    let separator = rendered
        .layout()
        .find_role(view::node::Role::Separator)
        .into_iter()
        .next()
        .expect("file menu separator should be laid out");
    let popup = rendered
        .layout()
        .find_role(view::node::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("file menu popup should be laid out");
    let separator_slots =
        layout::control::menu_row_slots(separator.rect(), separator.shortcut_width(), &theme);

    assert_eq!(separator.rect().height(), theme.menu().row_height);
    assert_eq!(
        separator.rect().x(),
        popup.rect().x() + theme.floating_panel().padding
    );
    assert_eq!(
        separator.rect().right(),
        popup.rect().right() - theme.floating_panel().padding
    );
    assert_eq!(separator_slots.separator.x(), separator.rect().x());
    assert_eq!(separator_slots.separator.width(), separator.rect().width());
    assert!(rendered.scene().quads().iter().any(|quad| {
        quad.rect() == separator_slots.separator && quad.fill() == theme.menu().separator
    }));
}

#[test]
fn menu_popup_opens_under_its_menu_title() {
    let mut app = control_gallery::app(control_gallery::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    let view_menu_action = app
        .present(window)
        .expect("control gallery should present")
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("View"))
        .expect("view menu should be projected")
        .menu_action()
        .expect("view menu should have an action");

    app.handle_view(window, view_menu_action)
        .expect("view menu action should open the menu");

    let rendered = app
        .render_scene(window, size)
        .expect("open view menu should render");
    let view_menu = rendered
        .layout()
        .find_role(view::node::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("View"))
        .expect("view menu should be laid out");
    let popup = rendered
        .layout()
        .find_role(view::node::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("view menu popup should be laid out");

    assert!(view_menu.rect().x() > 0);
    assert_eq!(popup.rect().x(), view_menu.rect().x());
    assert_eq!(popup.rect().y(), view_menu.rect().bottom());
}

#[test]
fn menu_titles_paint_hover_pressed_and_active_tints() {
    let mut app = control_gallery::app(control_gallery::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    let initial = app
        .render_scene(window, size)
        .expect("control gallery should render");
    let menu = initial
        .layout()
        .find_role(view::node::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Controls"))
        .expect("controls menu should be laid out");
    let point = frame_point(menu);

    assert_no_tint_quad(
        initial.scene(),
        menu.rect(),
        Theme::default().menu().title_background,
    );

    app.pointer_move_at(window, size, point)
        .expect("menu pointer move should be handled");
    let hovered = app
        .render_scene(window, size)
        .expect("hovered menu should render");
    assert_tint_quad(
        hovered.scene(),
        menu.rect(),
        Theme::default().menu().title_hover_tint,
    );

    app.pointer_down_at(window, size, point)
        .expect("menu pointer down should be handled");
    let pressed = app
        .render_scene(window, size)
        .expect("pressed menu should render");
    assert_tint_quad(
        pressed.scene(),
        menu.rect(),
        Theme::default().menu().title_pressed_tint,
    );

    app.pointer_up_at(window, size, point)
        .expect("menu pointer up should be handled");
    let active = app
        .render_scene(window, size)
        .expect("open menu should render active title");
    assert_tint_quad(
        active.scene(),
        menu.rect(),
        Theme::default().menu().title_active_tint,
    );
}

#[test]
fn menu_popup_rows_paint_hover_tint_from_pointer_projection() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let initial = app
        .render_scene(window, size)
        .expect("text editor should render");
    let file = initial
        .layout()
        .find_role(view::node::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("File"))
        .expect("file menu should be laid out");
    let file_point = frame_point(file);

    app.pointer_down_at(window, size, file_point)
        .expect("file menu pointer down should be handled");
    app.pointer_up_at(window, size, file_point)
        .expect("file menu pointer up should open the menu");
    let opened = app
        .render_scene(window, size)
        .expect("open file menu should render");
    let new_row = opened
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.role() == view::node::Role::Binding && frame.label_text() == Some("New")
        })
        .expect("new command row should be laid out");
    let before_hover = app
        .diagnostics(window)
        .expect("diagnostics should exist")
        .frame
        .clone();

    let moved = app
        .pointer_move_at(window, size, frame_point(new_row))
        .expect("popup row pointer move should be handled");

    assert!(moved.is_handled());
    assert_eq!(
        moved.effect().invalidation(),
        Some(response::Invalidation::Paint)
    );
    assert!(moved.effect().contains_invalidation());

    let hovered = app
        .render_scene(window, size)
        .expect("hovered popup row should render");
    let after_hover = app
        .diagnostics(window)
        .expect("diagnostics should exist")
        .frame
        .clone();
    let hovered_row = hovered
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.role() == view::node::Role::Binding && frame.label_text() == Some("New")
        })
        .expect("new command row should still be laid out");

    assert_tint_quad(
        hovered.scene(),
        hovered_row.rect(),
        Theme::default().menu().row_hover_tint,
    );
    assert_eq!(after_hover.view_rebuilds, before_hover.view_rebuilds);
    assert_eq!(
        after_hover.layout_recomposes,
        before_hover.layout_recomposes
    );
    assert!(after_hover.layout_reuses > before_hover.layout_reuses);
}

#[test]
fn checked_menu_popup_rows_do_not_paint_active_tint() {
    let mut app = control_gallery::app(control_gallery::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    let view_menu = app
        .present(window)
        .expect("control gallery should present")
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("View"))
        .expect("view menu should be projected")
        .menu_action()
        .expect("view menu should have an action");

    app.handle_view(window, view_menu)
        .expect("view menu action should open the menu");

    let opened = app
        .render_scene(window, size)
        .expect("open view menu should render");
    let wrap = opened
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.role() == view::node::Role::Binding && frame.label_text() == Some("Wrap text")
        })
        .expect("checked wrap row should be laid out");
    let theme = Theme::default();
    let slots = layout::control::menu_row_slots(wrap.rect(), wrap.shortcut_width(), &theme);

    assert_eq!(wrap.checked(), Some(true));
    assert_no_tint_quad(opened.scene(), wrap.rect(), theme.menu().title_active_tint);
    assert!(
        opened
            .scene()
            .icons()
            .iter()
            .any(|icon| { icon.rect() == slots.glyph && icon.icon().id().as_str() == "check" })
    );
}

#[test]
fn focus_outline_uses_frame_rounding() {
    let mut app = control_gallery::app(control_gallery::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    app.render_scene(window, size)
        .expect("control gallery should render before focus");
    app.handle_input(
        window,
        Input::key_down(input::Key::Tab, input::Modifiers::default()),
    )
    .expect("tab should focus first menu");

    let focused = app
        .render_scene(window, size)
        .expect("focused menu should render");
    let frame = focused
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.is_focused())
        .expect("focused frame should be present");
    let outline = focused
        .scene()
        .outlines()
        .into_iter()
        .find(|outline| {
            outline.rect() == frame.rect() && outline.color() == Theme::default().focus().color
        })
        .expect("focus outline should paint");

    assert_eq!(outline.rounding(), Theme::default().control().rounding);
    assert_eq!(outline.offset(), Theme::default().focus().offset);
}

#[test]
fn scene_preserves_popup_paint_order_after_base_content() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let file_menu = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should exist")
        .menu_action()
        .expect("file menu should have an action");

    app.handle_view(window, file_menu)
        .expect("menu action should be handled");

    let projected = app.present(window).expect("window should have a view");
    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(
        &projected,
        geometry::Size::new(800, 600),
        &mut layout_engine,
    );
    let scene = scene::Scene::paint(&layout);
    let popup = layout
        .find_role(view::node::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("file menu popup should be laid out");
    let theme = Theme::default();

    assert!(
        scene
            .shadows()
            .iter()
            .any(|shadow| shadow.color().channels() == (0, 0, 0, 96)),
        "popup paint should include theme-owned elevation"
    );
    assert!(
        !scene
            .outlines()
            .iter()
            .any(|outline| outline.rect() == popup.rect()),
        "menu popup should not paint a default outline"
    );

    let popup_shadow = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Shadow(shadow) if shadow.rect() == popup.rect()
            )
        })
        .expect("popup shadow should be painted");
    let popup_filter = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Filter(filter)
                    if filter.rect() == popup.rect()
                        && matches!(
                            filter.ops(),
                            [scene::FilterOp::BackdropBlur(blur)] if blur.sigma() == 44.55
                        )
                        && filter.rounding() == theme.floating_panel().rounding
            )
        })
        .expect("popup filter should be painted");
    let popup_material = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Quad(quad)
                    if quad.rect() == popup.rect()
                        && quad.style().fill()
                            == Some(scene::Brush::solid(scene::Color::rgba(28, 28, 30, 224)))
                        && quad.rounding() == theme.floating_panel().rounding
            )
        })
        .expect("popup material fill should be painted");
    let file_menu_text = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Text(text) if text.value() == "File"
            )
        })
        .expect("menu bar file text should be painted");
    let open_command_text = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Text(text) if text.value() == "Open"
            )
        })
        .expect("popup open command text should be painted");
    let open_command_clip = scene.primitives()[..open_command_text]
        .iter()
        .rposition(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Clip(clip)
                    if clip.rect() == popup.rect()
                        && clip.rounding() == theme.floating_panel().rounding
            )
        })
        .expect("popup row should be clipped to rounded panel");
    let open_command_pop_clip = scene.primitives()[open_command_text..]
        .iter()
        .position(|primitive| matches!(primitive, scene::Primitive::PopClip))
        .map(|index| index + open_command_text)
        .expect("popup row clip should be popped after row content");
    let exit_command_text = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Text(text) if text.value() == "Exit"
            )
        })
        .expect("popup exit command text should be painted");
    let exit_shortcut_text = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Text(text) if text.value() == "F4"
            )
        })
        .expect("popup exit shortcut key text should be painted");

    assert!(popup_shadow < popup_filter);
    assert!(popup_filter < popup_material);
    assert!(popup_material < open_command_text);
    assert!(popup_material < open_command_clip);
    assert!(open_command_clip < open_command_text);
    assert!(open_command_text < open_command_pop_clip);
    assert!(file_menu_text < open_command_text);
    assert!(file_menu_text < exit_command_text);
    assert!(exit_command_text < exit_shortcut_text);
}

#[test]
fn generic_floating_panel_uses_shared_chrome_before_content() {
    let view = widget::view(|ui| {
        ui.add(
            widget::Element::new()
                .overlay()
                .width(view::style::Dimension::fixed(240))
                .height(view::style::Dimension::fixed(160))
                .children(|ui| {
                    ui.add(
                        widget::panel::Floating::new("tests.floating")
                            .width(view::style::Dimension::fixed(180))
                            .height(view::style::Dimension::fixed(80))
                            .children(|ui| {
                                ui.label("Inside");
                            }),
                    );
                }),
        );
    });
    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(240, 160), &mut layout_engine);
    let scene = scene::Scene::paint(&layout);
    let panel = layout
        .find_role(view::node::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("floating panel should be laid out");

    let shadow = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Shadow(shadow) if shadow.rect() == panel.rect()
            )
        })
        .expect("floating panel shadow should paint");
    let filter = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Filter(filter) if filter.rect() == panel.rect()
            )
        })
        .expect("floating panel filter should paint");
    let material = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Quad(quad) if quad.rect() == panel.rect()
            )
        })
        .expect("floating panel material should paint");
    let content = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Text(text) if text.value() == "Inside"
            )
        })
        .expect("floating panel content should paint");

    assert!(shadow < filter);
    assert!(filter < material);
    assert!(material < content);
}

#[test]
fn generic_floating_panel_uses_stack_padding_and_gap_for_content() {
    let view = widget::view(|ui| {
        ui.add(
            widget::panel::Floating::new("tests.floating.layout")
                .column()
                .width(view::style::Dimension::fixed(220))
                .height(view::style::Dimension::fixed(140))
                .layout(|layout| layout.gap(7).padding(view::style::Padding::all(11)))
                .children(|ui| {
                    ui.label("Alpha");
                    ui.label("Beta");
                }),
        );
    });
    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(260, 180), &mut layout_engine);
    let theme = Theme::default();
    let panel = layout
        .find_role(view::node::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("floating panel should be laid out");
    let alpha = layout
        .find_role(view::node::Role::Label)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Alpha"))
        .expect("alpha label should be laid out");
    let beta = layout
        .find_role(view::node::Role::Label)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Beta"))
        .expect("beta label should be laid out");

    assert_eq!(
        alpha.rect().x(),
        panel.rect().x() + theme.floating_panel().padding + 11
    );
    assert_eq!(
        alpha.rect().y(),
        panel.rect().y() + theme.floating_panel().padding + 11
    );
    assert_eq!(beta.rect().y(), alpha.rect().bottom() + 7);
}

#[test]
fn slider_labels_are_single_line_without_default_row_fill() {
    let view = widget::view(|ui| {
        ui.add(
            widget::Element::new()
                .column()
                .height(view::style::Dimension::fixed(80))
                .children(|ui| {
                    ui.slider(widget::Slider::new("Feather", 24.0, 0.0..=64.0));
                }),
        );
    });
    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(360, 80), &mut layout_engine);
    let scene = scene::Scene::paint(&layout);
    let slider = layout
        .find_role(view::node::Role::Slider)
        .into_iter()
        .next()
        .expect("slider should be laid out");
    let label = scene
        .texts()
        .into_iter()
        .find(|text| text.value() == "Feather: 24.00")
        .expect("slider label should paint");

    assert_eq!(label.wrap(), scene::TextWrap::None);
    assert!(
        !scene
            .quads()
            .iter()
            .any(|quad| quad.rect() == slider.rect()),
        "default slider row should not paint a filled control background"
    );
}

#[test]
fn overlay_layout_paints_styled_backgrounds_under_floating_panel() {
    let bar = scene::Color::rgb(235, 73, 83);
    let view = widget::view(|ui| {
        ui.add(
            widget::Element::new()
                .overlay()
                .width(view::style::Dimension::fixed(200))
                .height(view::style::Dimension::fixed(120))
                .children(|ui| {
                    ui.add(
                        widget::Element::new()
                            .background(scene::Brush::solid(bar))
                            .width(view::style::Dimension::grow())
                            .height(view::style::Dimension::grow()),
                    );
                    ui.add(
                        widget::panel::Floating::new("tests.overlay.panel")
                            .width(view::style::Dimension::fixed(120))
                            .height(view::style::Dimension::fixed(64))
                            .children(|ui| {
                                ui.label("Panel");
                            }),
                    );
                }),
        );
    });
    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(200, 120), &mut layout_engine);
    let scene = scene::Scene::paint(&layout);
    let panel = layout
        .find_role(view::node::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("floating panel should be laid out");

    assert!(
        scene.quads().iter().any(|quad| {
            quad.fill() == bar && quad.rect() == geometry::Rect::new(0, 0, 200, 120)
        })
    );
    assert!(
        scene
            .filters()
            .iter()
            .any(|filter| filter.rect() == panel.rect())
    );
}

#[test]
fn glass_tuner_projects_live_theme_values_and_hit_tests_panel_controls() {
    let mut app = glass_tuner::app(glass_tuner::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = glass_tuner::window_size();
    let initial = app
        .render_scene(window, size)
        .expect("glass tuner should render");
    let panel = initial
        .layout()
        .find_role(view::node::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("glass tuner floating panel should be laid out");
    let default_blur = initial
        .scene()
        .filters()
        .into_iter()
        .find(|filter| filter.rect() == panel.rect())
        .and_then(|filter| filter.ops().first().copied())
        .expect("glass tuner panel should paint a blur filter");

    assert_eq!(
        default_blur,
        scene::FilterOp::BackdropBlur(scene::BackdropBlur::new(44.55))
    );
    assert!(initial.scene().quads().iter().any(|quad| {
        quad.rect() == panel.rect()
            && quad.style().fill() == Some(scene::Brush::solid(scene::Color::rgba(28, 28, 30, 224)))
    }));
    assert!(initial.scene().texts().iter().any(|text| {
        text.value().contains("blur-sigma = 44.55")
            || text.value().contains("noise-opacity = 0.022")
    }));
    assert!(initial.scene().texts().iter().any(|text| {
        text.value() == "Blur sigma: 44.55" && text.wrap() == scene::TextWrap::None
    }));
    assert!(
        !initial
            .scene()
            .texts()
            .iter()
            .any(|text| text.value().contains("refraction")),
        "acrylic tuner should not expose refraction TOML by default"
    );

    app.invoke(app.trigger::<glass_tuner::SetToken>((glass_tuner::AcrylicToken::TintR, 80.0)))
        .output
        .expect("set tint red should succeed");
    app.invoke(
        app.trigger::<glass_tuner::SetToken>((glass_tuner::AcrylicToken::NoiseOpacity, 0.04)),
    )
    .output
    .expect("set noise opacity should succeed");

    let rendered = app
        .render_scene(window, size)
        .expect("glass tuner should render after acrylic tuning");
    assert!(rendered.scene().quads().iter().any(|quad| {
        quad.rect() == panel.rect()
            && quad.style().fill() == Some(scene::Brush::solid(scene::Color::rgba(80, 28, 30, 224)))
    }));
    assert!(rendered.scene().texts().iter().any(|text| {
        text.value().contains("tint = \"#501c1e\"")
            || text.value().contains("noise-opacity = 0.040")
    }));
    assert!(rendered.scene().texts().iter().any(|text| {
        text.value() == "Noise opacity: 0.04" && text.wrap() == scene::TextWrap::None
    }));

    let slider = rendered
        .layout()
        .find_role(view::node::Role::Slider)
        .into_iter()
        .next()
        .expect("glass tuner should lay out sliders");
    let slider_rect = slider.active_rect();
    let hit = app
        .hit_test(
            window,
            size,
            geometry::Point::new(
                slider_rect.x().saturating_add(slider_rect.width() / 2),
                slider_rect.y().saturating_add(slider_rect.height() / 2),
            ),
        )
        .expect("slider should be hit testable");
    assert_eq!(hit.frame().role(), view::node::Role::Slider);

    app.invoke(app.trigger::<glass_tuner::TogglePanel>(()))
        .output
        .expect("toggle panel should succeed");
    let hidden = app
        .render_scene(window, size)
        .expect("hidden glass tuner should render");
    assert!(hidden.scene().filters().is_empty());
}

fn assert_tint_quad(scene: &Scene, rect: geometry::Rect, color: scene::Color) {
    assert!(
        scene.quads().iter().any(|quad| {
            quad.rect() == rect
                && quad.fill() == color
                && quad.rounding() == Theme::default().control().rounding
        }),
        "expected tint quad for rect {rect:?} and color {:?}",
        color.channels()
    );
}

fn assert_no_tint_quad(scene: &Scene, rect: geometry::Rect, color: scene::Color) {
    assert!(
        !scene
            .quads()
            .iter()
            .any(|quad| quad.rect() == rect && quad.fill() == color),
        "unexpected tint quad for rect {rect:?} and color {:?}",
        color.channels()
    );
}

fn assert_choice_pressed_tint_above_mark(presentation: &scene::Presentation, mark: geometry::Rect) {
    let primitives = presentation.scene().primitives();
    let mark_color = Theme::default().choice().mark;
    let pressed_tint = Theme::default().control().pressed_tint;
    let mark_index = primitives
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Quad(quad) if quad.rect() == mark && quad.fill() == mark_color
            )
        })
        .expect("choice mark base should be painted");
    let tint_index = primitives
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Quad(quad) if quad.rect() == mark && quad.fill() == pressed_tint
            )
        })
        .expect("choice pressed tint should be painted");

    assert!(
        mark_index < tint_index,
        "choice pressed tint should paint above the mark base"
    );
}

fn assert_no_choice_label_overlay(presentation: &scene::Presentation, rect: geometry::Rect) {
    let pressed_tint = Theme::default().control().pressed_tint;

    assert!(
        !presentation
            .scene()
            .quads()
            .iter()
            .any(|quad| quad.rect() == rect && quad.fill() == pressed_tint),
        "choice label region should not paint the pressed tint"
    );
}

fn frame_point_at(rect: geometry::Rect) -> geometry::Point {
    geometry::Point::new(
        rect.x().saturating_add(rect.width() / 2),
        rect.y().saturating_add(rect.height() / 2),
    )
}

fn rect_top_point(rect: geometry::Rect) -> geometry::Point {
    geometry::Point::new(
        rect.x().saturating_add(rect.width() / 2),
        rect.y().saturating_add(1),
    )
}

fn rect_bottom_point(rect: geometry::Rect) -> geometry::Point {
    geometry::Point::new(
        rect.x().saturating_add(rect.width() / 2),
        rect.bottom().saturating_sub(2),
    )
}

fn first_visible_text_area_surface_y(presentation: &scene::Presentation) -> f32 {
    visible_text_area_surfaces(presentation)
        .into_iter()
        .map(|(_, y, _)| y)
        .min_by(f32::total_cmp)
        .expect("text area should render at least one visible surface")
}

fn visible_text_area_surfaces(presentation: &scene::Presentation) -> Vec<(usize, f32, f32)> {
    presentation
        .layout()
        .find_role(view::node::Role::TextArea)
        .into_iter()
        .flat_map(|frame| {
            let height = frame.rect().height() as f32;
            frame
                .text_area_layout()
                .into_iter()
                .flat_map(layout::text::Area::render_surfaces)
                .filter(move |surface| surface.y() < height && surface.y() + surface.height() > 0.0)
        })
        .map(|surface| (surface.source_line(), surface.y(), surface.height()))
        .collect()
}

fn scroll_app() -> Runtime<SourceState, (), View> {
    Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Scroll"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.add(
                        widget::Scroll::new()
                            .id("scroll.test")
                            .height(view::style::Dimension::fixed(72))
                            .children(|ui| {
                                for index in 0..8 {
                                    ui.label(format!("Row {index}"));
                                }
                            }),
                    );
                });
            })
        })
}

fn command_palette_scroll_app() -> Runtime<SourceState, (), View> {
    Runtime::new(SourceState::default())
        .commands(|commands| {
            commands
                .register::<PaletteOne>(command::Spec::new("Palette One").shortcut("Primary+R"))
                .register::<PaletteTwo>(command::Spec::new("Palette Two"))
                .register::<PaletteThree>(command::Spec::new("Palette Three"))
                .register::<PaletteFour>(command::Spec::new("Palette Four"))
                .register::<PaletteFive>(command::Spec::new("Palette Five"))
                .register::<PaletteSix>(command::Spec::new("Palette Six"))
                .register::<PaletteSeven>(command::Spec::new("Palette Seven"))
                .register::<PaletteEight>(command::Spec::new("Palette Eight"))
                .register::<PaletteNine>(command::Spec::new("Palette Nine"))
                .register::<PaletteTen>(command::Spec::new("Palette Ten"))
                .register::<PaletteEleven>(command::Spec::new("Palette Eleven"))
                .register::<PaletteTwelve>(command::Spec::new("Palette Twelve"));
        })
        .responders(|responders| {
            responders
                .app()
                .target::<PaletteOne>()
                .target::<PaletteTwo>()
                .target::<PaletteThree>()
                .target::<PaletteFour>()
                .target::<PaletteFive>()
                .target::<PaletteSix>()
                .target::<PaletteSeven>()
                .target::<PaletteEight>()
                .target::<PaletteNine>()
                .target::<PaletteTen>()
                .target::<PaletteEleven>()
                .target::<PaletteTwelve>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Palette Scroll"));
        })
        .view(|_, _| View::new(view::Node::root()))
}

fn nested_clipped_scroll_app() -> Runtime<SourceState, (), View> {
    let focus = session::Focus::text("nested.search");
    Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Nested Scroll"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.text_box(widget::TextBox::new("").focus(focus));
                    ui.add(
                        widget::Scroll::new()
                            .id("scroll.outer")
                            .label("Outer Scroll")
                            .height(view::style::Dimension::fixed(64))
                            .children(|ui| {
                                ui.label("Outer row 0");
                                ui.label("Outer row 1");
                                ui.add(
                                    widget::Scroll::new()
                                        .id("scroll.inner")
                                        .label("Inner Scroll")
                                        .height(view::style::Dimension::fixed(54))
                                        .children(|ui| {
                                            for index in 0..8 {
                                                ui.label(format!("Inner row {index}"));
                                            }
                                        }),
                                );
                                for index in 2..10 {
                                    ui.label(format!("Outer row {index}"));
                                }
                            }),
                    );
                });
            })
        })
}

fn scroll_outer_until_inner_overlaps_search(
    app: &mut Runtime<SourceState, (), View>,
    window: window::Id,
    size: geometry::Size,
) {
    let initial = app
        .render_scene(window, size)
        .expect("nested scroll should render");
    let outer = scroll_frame_with_label(&initial, "Outer Scroll");
    app.scroll_at(
        window,
        size,
        frame_point_at(
            outer
                .viewport()
                .expect("outer scroll should expose viewport")
                .rect(),
        ),
        interaction::ScrollDelta::vertical(112),
    )
    .expect("outer scroll should be handled");
}

fn first_scroll_frame(presentation: &scene::Presentation) -> &layout::frame::Frame {
    presentation
        .layout()
        .find_role(view::node::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out")
}

fn scroll_frame_with_label<'a>(
    presentation: &'a scene::Presentation,
    label: &str,
) -> &'a layout::frame::Frame {
    presentation
        .layout()
        .find_role(view::node::Role::Scroll)
        .into_iter()
        .find(|frame| frame.label_text() == Some(label))
        .expect("named scroll should be laid out")
}

fn command_palette_results_frame(presentation: &scene::Presentation) -> &layout::frame::Frame {
    presentation
        .layout()
        .find_role(view::node::Role::Scroll)
        .into_iter()
        .find(|frame| {
            frame.target().and_then(interaction::Target::element_id)
                == Some(interaction::CommandPalette::results_id())
        })
        .expect("command palette results scroll should be laid out")
}

fn command_palette_panel_frame(presentation: &scene::Presentation) -> &layout::frame::Frame {
    presentation
        .layout()
        .find_role(view::node::Role::FloatingPanel)
        .into_iter()
        .find(|frame| {
            frame.target().and_then(interaction::Target::element_id)
                == Some(interaction::CommandPalette::panel_id())
        })
        .expect("command palette panel should be laid out")
}

fn immediate_scroll_child_frames<'a>(
    layout: &'a layout::Layout,
    scroll: &layout::frame::Frame,
) -> Vec<&'a layout::frame::Frame> {
    let child_depth = scroll.path().indexes().len() + 1;
    layout
        .frames()
        .iter()
        .filter(|frame| {
            frame.path().indexes().len() == child_depth
                && frame.path().is_descendant_of(scroll.path())
        })
        .collect()
}

fn selected_palette_result_frame(presentation: &scene::Presentation) -> &layout::frame::Frame {
    presentation
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.is_selected() && frame.binding_source() == Some(context::Source::Palette)
        })
        .expect("selected command palette result should be laid out")
}

fn first_scrollbar_track(layout: &layout::Layout) -> geometry::Rect {
    layout
        .chrome()
        .iter()
        .find_map(|chrome| match chrome.kind() {
            layout::chrome::Kind::Scrollbar(scrollbar) => Some(scrollbar.track()),
        })
        .expect("scrollbar chrome should be projected")
}

fn scene_has_scrollbar_thumb(scene: &Scene, theme: &Theme, bounds: geometry::Rect) -> bool {
    let (thumb_r, thumb_g, thumb_b, _) = theme.scrollbar().appearance.thumb.channels();
    scene.quads().iter().any(|quad| {
        let (r, g, b, a) = quad.fill().channels();
        (r, g, b) == (thumb_r, thumb_g, thumb_b) && a > 0 && rect_contains(bounds, quad.rect())
    })
}

fn scene_text<'a>(scene: &'a Scene, value: &str) -> &'a scene::Text {
    scene
        .texts()
        .into_iter()
        .find(|text| text.value() == value)
        .expect("scene text should exist")
}

fn scene_text_in_rect<'a>(
    scene: &'a Scene,
    value: &str,
    bounds: geometry::Rect,
) -> &'a scene::Text {
    scene
        .texts()
        .into_iter()
        .find(|text| text.value() == value && rect_contains(bounds, text.rect()))
        .expect("scene text should exist inside bounds")
}

fn scene_icon_in_rect<'a>(scene: &'a Scene, icon: &str, bounds: geometry::Rect) -> &'a scene::Icon {
    scene
        .icons()
        .into_iter()
        .find(|candidate| {
            candidate.icon().id().as_str() == icon && rect_contains(bounds, candidate.rect())
        })
        .expect("scene icon should exist inside bounds")
}

fn text_color_channels(color: scene::Color) -> (f32, f32, f32, f32) {
    let (r, g, b, a) = color.channels();

    (
        linear_channel(r),
        linear_channel(g),
        linear_channel(b),
        alpha_channel(a),
    )
}

fn text_color_channels_equal(actual: (f32, f32, f32, f32), expected: (f32, f32, f32, f32)) -> bool {
    (actual.0 - expected.0).abs() < f32::EPSILON
        && (actual.1 - expected.1).abs() < f32::EPSILON
        && (actual.2 - expected.2).abs() < f32::EPSILON
        && (actual.3 - expected.3).abs() < f32::EPSILON
}

fn linear_channel(channel: u8) -> f32 {
    let value = alpha_channel(channel);

    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn alpha_channel(channel: u8) -> f32 {
    channel as f32 / u8::MAX as f32
}
