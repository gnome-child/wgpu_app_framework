use std::{error::Error, fs, io};

use crate::scratch::{geometry, input, session, shell::Event as ShellEvent, view};

use super::{LoadStressText, State, app, shell as new_shell, window_size};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

pub fn smoke() -> Result {
    let mut app = app(State::default());
    app.start();

    let window = app
        .session()
        .windows()
        .first()
        .ok_or_else(|| io::Error::other("text editor did not open a window"))?
        .id();
    let size = window_size();

    let initial = app
        .render_scene(window, size)
        .ok_or_else(|| io::Error::other("text editor did not render its initial scene"))?;
    if initial.scene().is_empty() {
        return Err(io::Error::other("initial text editor scene is empty").into());
    }

    let cold_undo = app.handle_input(window, input::Input::shortcut("Ctrl+Z"))?;
    if cold_undo.is_handled() || cold_undo.changed_state() {
        return Err(io::Error::other("cold undo shortcut should be ignored").into());
    }

    let focus = session::Focus::text("document");
    let focus_outcome = app.handle_input(window, input::Input::focus(focus))?;
    if !focus_outcome.is_handled() || app.session().focused(window) != Some(focus) {
        return Err(io::Error::other("text editor did not focus its document").into());
    }

    for character in "smoke".chars() {
        let outcome = app.handle_input(
            window,
            input::Input::key_down_with_text(
                input::Key::Character(character),
                input::Modifiers::default(),
                Some(character.to_string()),
            ),
        )?;
        if !outcome.is_handled() || !outcome.changed_state() {
            return Err(io::Error::other("printable key input did not edit the document").into());
        }
    }
    if app.state().document.text() != "smoke" {
        return Err(io::Error::other("edited document text did not match key input").into());
    }

    let undo = app.handle_input(window, input::Input::shortcut("Ctrl+Z"))?;
    if !undo.is_handled() || !undo.changed_state() || !app.state().document.is_empty() {
        return Err(io::Error::other("undo shortcut did not restore the empty document").into());
    }

    let redo = app.handle_input(window, input::Input::shortcut("Ctrl+Shift+Z"))?;
    if !redo.is_handled() || !redo.changed_state() || app.state().document.text() != "smoke" {
        return Err(io::Error::other("redo shortcut did not restore typed input").into());
    }

    let edited = app
        .render_scene(window, size)
        .ok_or_else(|| io::Error::other("text editor did not render edited scene"))?;
    if !edited
        .scene()
        .text_viewports()
        .iter()
        .any(|viewport| !viewport.surfaces().is_empty())
    {
        return Err(io::Error::other("edited scene did not produce text surfaces").into());
    }

    app.invoke(app.trigger::<LoadStressText>(()));
    if !app
        .state()
        .last_status
        .contains("loaded Unicode stress fixture")
    {
        return Err(io::Error::other("stress text command did not update status").into());
    }

    let stressed = app
        .render_scene(window, size)
        .ok_or_else(|| io::Error::other("text editor did not render stress scene"))?;
    if !stressed
        .scene()
        .text_viewports()
        .iter()
        .any(|viewport| !viewport.surfaces().is_empty())
    {
        return Err(io::Error::other("stress scene did not produce text surfaces").into());
    }

    smoke_shell_file_flow()?;

    Ok(())
}

fn smoke_shell_file_flow() -> Result {
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let path = std::env::temp_dir().join(format!("wgpu_l3_text_editor_smoke_{suffix}.txt"));
    let _ = fs::remove_file(&path);

    let mut shell = new_shell(State::default());
    let started = shell.handle_event(ShellEvent::Started)?;
    let window = started
        .opened_windows()
        .first()
        .ok_or_else(|| io::Error::other("shell smoke did not open a window"))?
        .id();

    let text_area_rect = started
        .presentations()
        .iter()
        .find(|presentation| presentation.window() == window)
        .and_then(|presentation| {
            presentation
                .layout()
                .find_role(view::node::Role::TextArea)
                .into_iter()
                .next()
                .map(|frame| frame.rect())
        })
        .ok_or_else(|| io::Error::other("shell smoke did not lay out a text area"))?;
    let text_area_point = geometry::Point::new(text_area_rect.x() + 4, text_area_rect.y() + 4);
    shell.handle_event(ShellEvent::PointerDown {
        window,
        point: text_area_point,
    })?;
    let Some(focus) = shell.runtime().session().focused(window) else {
        return Err(io::Error::other("pointer down did not focus shell document").into());
    };
    if !focus.same_target(&session::Focus::text("document"))
        || focus.reason() != session::focus::Reason::Pointer
        || focus.visibility() != session::focus::Visibility::Hidden
    {
        return Err(io::Error::other("pointer down did not focus shell document").into());
    }

    for character in "smoke".chars() {
        shell.handle_event(ShellEvent::KeyDown {
            window,
            key: input::Key::Character(character),
            modifiers: input::Modifiers::default(),
            text: Some(character.to_string()),
        })?;
    }
    if shell.runtime().state().document.text() != "smoke" {
        return Err(io::Error::other("shell key input did not edit document").into());
    }

    let save = shell.handle_event(ShellEvent::KeyDown {
        window,
        key: input::Key::Character('s'),
        modifiers: input::Modifiers::new(false, true, false, false),
        text: None,
    })?;
    let save_request = save
        .requests()
        .first()
        .ok_or_else(|| io::Error::other("save shortcut did not request a file path"))?;
    if save_request.window() != window
        || save_request.kind() != session::RequestKind::FileDialog(session::FileDialog::SaveAs)
    {
        return Err(io::Error::other("save shortcut requested the wrong dialog").into());
    }

    let selected = shell.handle_event(ShellEvent::FilePathSelected {
        window,
        path: Some(path.clone()),
    })?;
    if selected.pending_tasks() != 1 || !selected.needs_poll() {
        return Err(io::Error::other("save path selection did not schedule file write").into());
    }

    let saved = shell.handle_event(ShellEvent::Poll)?;
    if saved.pending_tasks() != 0 || saved.needs_poll() {
        return Err(io::Error::other("save poll did not complete file write").into());
    }
    if fs::read_to_string(&path)? != "smoke" || shell.runtime().state().document.is_dirty() {
        return Err(io::Error::other("saved file did not match document state").into());
    }

    fs::write(&path, "opened from smoke")?;
    let open = shell.handle_event(ShellEvent::KeyDown {
        window,
        key: input::Key::Character('o'),
        modifiers: input::Modifiers::new(false, true, false, false),
        text: None,
    })?;
    let open_request = open
        .requests()
        .first()
        .ok_or_else(|| io::Error::other("open shortcut did not request a file path"))?;
    if open_request.window() != window
        || open_request.kind() != session::RequestKind::FileDialog(session::FileDialog::Open)
    {
        return Err(io::Error::other("open shortcut requested the wrong dialog").into());
    }

    shell.handle_event(ShellEvent::FilePathSelected {
        window,
        path: Some(path.clone()),
    })?;
    if shell.runtime().state().document.text() != "opened from smoke" {
        return Err(io::Error::other("open path selection did not load file contents").into());
    }

    let _ = fs::remove_file(path);

    Ok(())
}
