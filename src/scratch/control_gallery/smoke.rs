use std::{error::Error, io};

use super::super::{Theme, geometry, input, interaction, layout, session, view};
use super::{
    Mode, State,
    command::{ResetControls, SetLevel},
    view::window_size,
};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

pub fn smoke() -> Result {
    let mut app = super::app(State::default());
    app.start();

    let window = app
        .session()
        .windows()
        .first()
        .ok_or_else(|| io::Error::other("control gallery did not open a window"))?
        .id();
    let size = window_size();

    let initial = app
        .render_scene(window, size)
        .ok_or_else(|| io::Error::other("control gallery did not render"))?;
    if initial.scene().is_empty() {
        return Err(io::Error::other("control gallery scene is empty").into());
    }
    if !initial
        .scene()
        .texts()
        .iter()
        .any(|text| text.value() == "Interactive Controls")
    {
        return Err(io::Error::other("control gallery heading did not paint").into());
    }
    if !initial
        .scene()
        .icons()
        .iter()
        .any(|icon| icon.icon().id().as_str() == "check")
    {
        return Err(io::Error::other("control gallery did not paint checkbox icons").into());
    }

    let cold_undo = app.handle_input(window, input::Input::shortcut("Ctrl+Z"))?;
    if cold_undo.is_handled() || cold_undo.changed_state() {
        return Err(io::Error::other("cold undo shortcut should be ignored").into());
    }

    click_role_with_label(&mut app, window, size, view::node::Role::Button, "Click")?;
    if app.state().clicks != 1 {
        return Err(io::Error::other("button click did not invoke command").into());
    }

    click_role_with_label(
        &mut app,
        window,
        size,
        view::node::Role::Checkbox,
        "Show grid",
    )?;
    if !app.state().grid {
        return Err(io::Error::other("checkbox click did not toggle grid").into());
    }

    click_role_with_label(&mut app, window, size, view::node::Role::Radio, "Preview")?;
    if app.state().mode != Mode::Preview {
        return Err(io::Error::other("radio click did not select preview mode").into());
    }

    drag_slider_to_fraction(&mut app, window, size, 0.75)?;
    if (app.state().level - 75.0).abs() > 1.0 {
        return Err(io::Error::other("slider drag did not update level").into());
    }

    click_first_role(&mut app, window, size, view::node::Role::TextBox)?;
    for character in "query".chars() {
        let outcome = app.handle_input(
            window,
            input::Input::key_down_with_text(
                input::Key::Character(character),
                input::Modifiers::default(),
                Some(character.to_string()),
            ),
        )?;
        if !outcome.is_handled() {
            return Err(io::Error::other("text box key input was not handled").into());
        }
    }
    if app.state().query != "query" {
        let target = interaction::Target::text_area(session::Focus::text(super::view::QUERY_FOCUS));
        let draft = app
            .session()
            .interaction(window)
            .and_then(|interaction| interaction.text_input().draft_for(&target))
            .ok_or_else(|| io::Error::other("text box input did not create a draft"))?;
        if draft.text() != "query" || !app.state().query.is_empty() {
            return Err(io::Error::other("text box input did not update focused draft").into());
        }
    }
    let backspace = app.handle_input(
        window,
        input::Input::key_down(input::Key::Backspace, input::Modifiers::default()),
    )?;
    if !backspace.is_handled()
        || backspace.changed_state()
        || focused_query_draft_text(&app, window)? != "quer"
    {
        return Err(io::Error::other("text box backspace did not update query draft").into());
    }
    let field_undo = app.handle_input(window, input::Input::shortcut("Ctrl+Z"))?;
    if !field_undo.is_handled()
        || field_undo.changed_state()
        || focused_query_draft_text(&app, window)? != "query"
    {
        return Err(io::Error::other("text box undo did not restore query draft").into());
    }
    let field_redo = app.handle_input(window, input::Input::shortcut("Ctrl+Shift+Z"))?;
    if !field_redo.is_handled()
        || field_redo.changed_state()
        || focused_query_draft_text(&app, window)? != "quer"
    {
        return Err(io::Error::other("text box redo did not reapply query draft edit").into());
    }
    let field_restore = app.handle_input(window, input::Input::shortcut("Ctrl+Z"))?;
    if !field_restore.is_handled()
        || field_restore.changed_state()
        || focused_query_draft_text(&app, window)? != "query"
    {
        return Err(
            io::Error::other("text box undo did not restore query draft before drag").into(),
        );
    }
    drag_text_box_selection(&mut app, window, size)?;

    click_role_with_label(
        &mut app,
        window,
        size,
        view::node::Role::Checkbox,
        "Wrap text",
    )?;
    if app.state().query != "query" {
        return Err(io::Error::other("text box blur did not commit query draft").into());
    }

    let reset = app.handle_input(window, input::Input::shortcut("Ctrl+R"))?;
    if !reset.is_handled() || !reset.changed_state() {
        return Err(io::Error::other("reset shortcut did not invoke command").into());
    }
    if app.state().clicks != 0
        || app.state().grid
        || app.state().mode != Mode::Design
        || !app.state().query.is_empty()
        || (app.state().level - 42.0).abs() > f64::EPSILON
    {
        return Err(io::Error::other("reset shortcut did not restore default state").into());
    }

    app.invoke(app.trigger::<SetLevel>(33.0))
        .output
        .map_err(|error| io::Error::other(format!("set level command failed: {error:?}")))?;
    app.invoke(app.trigger::<ResetControls>(()))
        .output
        .map_err(|error| io::Error::other(format!("reset command failed: {error:?}")))?;

    Ok(())
}

fn focused_query_draft_text(
    app: &super::super::Runtime<State, (), super::super::View>,
    window: super::super::window::Id,
) -> Result<String> {
    let target = interaction::Target::text_area(session::Focus::text(super::view::QUERY_FOCUS));
    app.session()
        .interaction(window)
        .and_then(|interaction| interaction.text_input().draft_for(&target))
        .map(|draft| draft.text().to_owned())
        .ok_or_else(|| io::Error::other("query text box draft is missing").into())
}

fn click_role_with_label(
    app: &mut super::super::Runtime<State, (), super::super::View>,
    window: super::super::window::Id,
    size: geometry::Size,
    role: view::node::Role,
    label: &str,
) -> Result {
    let rendered = app
        .render_scene(window, size)
        .ok_or_else(|| io::Error::other("control gallery did not render before click"))?;
    let rect = rendered
        .layout()
        .find_role(role)
        .into_iter()
        .find(|frame| frame.label_text().unwrap_or_default() == label)
        .ok_or_else(|| io::Error::other(format!("missing {role:?} with label {label:?}")))?
        .active_rect();
    let point = center(rect);

    app.pointer_down_at(window, size, point)?;
    app.render_scene(window, size)
        .ok_or_else(|| io::Error::other("control gallery did not present after pointer down"))?;
    app.pointer_up_at(window, size, point)?;

    Ok(())
}

fn click_first_role(
    app: &mut super::super::Runtime<State, (), super::super::View>,
    window: super::super::window::Id,
    size: geometry::Size,
    role: view::node::Role,
) -> Result {
    let rendered = app
        .render_scene(window, size)
        .ok_or_else(|| io::Error::other("control gallery did not render before click"))?;
    let rect = rendered
        .layout()
        .find_role(role)
        .into_iter()
        .next()
        .ok_or_else(|| io::Error::other(format!("missing {role:?}")))?
        .rect();
    let point = center(rect);

    app.pointer_down_at(window, size, point)?;
    app.render_scene(window, size)
        .ok_or_else(|| io::Error::other("control gallery did not present after pointer down"))?;
    app.pointer_up_at(window, size, point)?;

    Ok(())
}

fn drag_slider_to_fraction(
    app: &mut super::super::Runtime<State, (), super::super::View>,
    window: super::super::window::Id,
    size: geometry::Size,
    fraction: f64,
) -> Result {
    let rendered = app
        .render_scene(window, size)
        .ok_or_else(|| io::Error::other("control gallery did not render before slider drag"))?;
    let frame = rendered
        .layout()
        .find_role(view::node::Role::Slider)
        .into_iter()
        .next()
        .ok_or_else(|| io::Error::other("missing slider"))?;
    let track = slider_track_rect(frame);
    let x = track
        .x()
        .saturating_add(((track.width() as f64) * fraction).round() as i32);
    let point = geometry::Point::new(x, track.y().saturating_add(track.height() / 2));

    app.pointer_down_at(window, size, point)?;
    app.pointer_move_at(window, size, point)?;
    app.pointer_up_at(window, size, point)?;

    Ok(())
}

fn drag_text_box_selection(
    app: &mut super::super::Runtime<State, (), super::super::View>,
    window: super::super::window::Id,
    size: geometry::Size,
) -> Result {
    let rendered = app
        .render_scene(window, size)
        .ok_or_else(|| io::Error::other("control gallery did not render before text drag"))?;
    let frame = rendered
        .layout()
        .find_role(view::node::Role::TextBox)
        .into_iter()
        .next()
        .ok_or_else(|| io::Error::other("missing text box"))?;
    let rect = frame.rect();
    let text_rect = frame.text_box_text_rect();
    let target = frame
        .target()
        .cloned()
        .ok_or_else(|| io::Error::other("text box should have an interaction target"))?;
    let press = geometry::Point::new(rect.x() + 1, rect.y() + rect.height() / 2);
    let wobble = geometry::Point::new(rect.right() - 1, rect.y() + 2);

    app.pointer_down_at(window, size, press)?;
    app.render_scene(window, size).ok_or_else(|| {
        io::Error::other("control gallery did not present after text pointer down")
    })?;
    app.pointer_move_at(window, size, wobble)?;
    let selected = app.render_scene(window, size).ok_or_else(|| {
        io::Error::other("control gallery did not present after text pointer move")
    })?;
    app.pointer_up_at(window, size, wobble)?;
    app.render_scene(window, size)
        .ok_or_else(|| io::Error::other("control gallery did not present after text pointer up"))?;

    let expected = interaction::Target::text_area(session::Focus::text(super::view::QUERY_FOCUS));
    if target != expected {
        return Err(io::Error::other(format!(
            "text box target did not match query focus: got {target:?}, expected {expected:?}"
        ))
        .into());
    }

    let draft = app
        .session()
        .interaction(window)
        .and_then(|interaction| interaction.text_input().draft_for(&target))
        .ok_or_else(|| io::Error::other("text box drag did not retain a draft"))?;
    if draft.selection().is_none() {
        return Err(io::Error::other("text box drag did not create a selection").into());
    }

    let selection_color = Theme::default().text().selection;
    let highlight_visible = selected
        .scene()
        .quads()
        .iter()
        .any(|quad| quad.fill() == selection_color && rect_contains(text_rect, quad.rect()));
    if !highlight_visible {
        return Err(io::Error::other("text box selection highlight did not paint").into());
    }

    Ok(())
}

fn center(rect: geometry::Rect) -> geometry::Point {
    geometry::Point::new(
        rect.x().saturating_add(rect.width() / 2),
        rect.y().saturating_add(rect.height() / 2),
    )
}

fn slider_track_rect(frame: &layout::frame::Frame) -> geometry::Rect {
    let theme = Theme::default();
    layout::control::slider_track_rect(frame.rect(), frame.label_width(), &theme)
}

fn rect_contains(bounds: geometry::Rect, rect: geometry::Rect) -> bool {
    rect.x() >= bounds.x()
        && rect.y() >= bounds.y()
        && rect.x().saturating_add(rect.width()) <= bounds.x().saturating_add(bounds.width())
        && rect.y().saturating_add(rect.height()) <= bounds.y().saturating_add(bounds.height())
}
