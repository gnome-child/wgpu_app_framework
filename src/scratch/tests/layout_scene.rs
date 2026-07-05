use super::*;

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
    assert_eq!(text_areas[0].rect().y(), 28);
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
fn layout_intrinsic_width_uses_text_measurement() {
    let view = View::new(
        view::Node::root().child(
            view::Node::menu_bar()
                .child(view::Node::menu("menu.wide", "WWWWWW"))
                .child(view::Node::menu("menu.narrow", "iiiiii")),
        ),
    );
    let mut layout_engine = layout::engine::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(400, 120), &mut layout_engine);
    let menus = layout.find_role(view::node::Role::Menu);

    assert_eq!(menus.len(), 2);
    assert!(
        menus[0].rect().width() > menus[1].rect().width(),
        "intrinsic menu widths should come from glyph measurement, not character count"
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
    assert_eq!(outcome.effect(), &response::Effect::Repaint);
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
    assert_eq!(moved.effect(), &response::Effect::Repaint);

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
        quad.fill().channels() == (245, 247, 250, 255)
            && layout
                .find_role(view::node::Role::TextArea)
                .iter()
                .any(|frame| frame.rect() == quad.rect())
    }));
    assert_eq!(scene.clear().channels(), (20, 22, 25, 255));
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
        .filter(|quad| quad.fill().channels() == (76, 132, 255, 96))
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
    let highlight = primitives
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Quad(quad)
                    if quad.fill().channels() == (76, 132, 255, 96)
            )
        })
        .expect("selection highlight should be painted");
    let menu_bar_chrome = primitives
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Quad(quad)
                    if quad.fill().channels() == (34, 37, 42, 255)
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
        quad.fill().channels() == (245, 247, 250, 255)
            && rect_contains(checkbox.rect(), quad.rect())
            && quad.rounding() == scene::Rounding::fixed(4.0)
    }));
    assert!(scene.quads().iter().any(|quad| {
        quad.fill().channels() == (76, 132, 255, 255)
            && rect_contains(radio.rect(), quad.rect())
            && quad.rounding() == scene::Rounding::relative(1.0)
    }));
    assert!(scene.quads().iter().any(|quad| {
        quad.fill().channels() == (75, 80, 88, 255)
            && rect_contains(slider.rect(), quad.rect())
            && quad.rect().height() == 4
    }));
    assert!(scene.quads().iter().any(|quad| {
        quad.fill().channels() == (76, 132, 255, 255)
            && rect_contains(slider.rect(), quad.rect())
            && quad.rect().height() == 4
    }));
    assert!(scene.quads().iter().any(|quad| {
        quad.fill().channels() == (238, 241, 245, 255)
            && rect_contains(slider.rect(), quad.rect())
            && quad.rounding() == scene::Rounding::relative(1.0)
    }));
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

    assert_eq!(light.clear(), light_theme.palette().canvas);
    assert_ne!(dark.clear(), light.clear());
    assert_ne!(dark_root.fill(), light_root.fill());
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
        quad.fill().channels() == (76, 132, 255, 255)
            && quad.rounding() == scene::Rounding::relative(1.0)
    }));
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

    assert!(
        scene
            .shadows()
            .iter()
            .any(|shadow| shadow.color().channels() == (0, 0, 0, 96)),
        "popup paint should include theme-owned elevation"
    );

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

    assert!(file_menu_text < open_command_text);
    assert!(matches!(
        scene.primitives().last(),
        Some(scene::Primitive::Text(text)) if text.value() == "Exit"
    ));
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
