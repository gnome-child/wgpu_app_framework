use crate::text;

use super::{
    command, composition,
    context::Context,
    diagnostics, draft,
    error::Error,
    geometry, interaction,
    response::{self, Response},
    scene, state,
    target::Target,
    window,
};

pub struct CloseWindow;

impl command::Command for CloseWindow {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "window.close";
    const HISTORY: command::History = command::History::Ignored;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Focus {
    target: interaction::Id,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileDialog {
    Open,
    SaveAs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Request {
    window: window::Id,
    kind: RequestKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequestKind {
    FileDialog(FileDialog),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    id: window::Id,
    title: String,
    inner_size: geometry::Size,
    canvas_color: scene::Color,
    redraw_requested: bool,
    presented_revision: Option<state::Revision>,
    focus: Option<Focus>,
    file_dialog: Option<FileDialog>,
    interaction: interaction::Interaction,
}

#[derive(Debug, Default)]
pub struct Session {
    windows: Vec<Window>,
    next_window_id: u64,
}

pub(super) struct Service<'a> {
    session: &'a mut Session,
    composition: &'a mut composition::Store,
    diagnostics: &'a mut diagnostics::Store,
    window: Option<window::Id>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    windows: Vec<WindowSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowSnapshot {
    id: window::Id,
    title: String,
    inner_size: geometry::Size,
    canvas_color: scene::Color,
    focus: Option<Focus>,
}

impl<'a> Service<'a> {
    pub(super) fn new(
        session: &'a mut Session,
        composition: &'a mut composition::Store,
        diagnostics: &'a mut diagnostics::Store,
        window: Option<window::Id>,
    ) -> Self {
        Self {
            session,
            composition,
            diagnostics,
            window,
        }
    }

    fn target_window(&self) -> Option<window::Id> {
        let session = self.session();
        match self.window {
            Some(window) => session.contains(window).then_some(window),
            None => session.windows().first().map(Window::id),
        }
    }

    fn session(&self) -> &Session {
        &*self.session
    }

    fn session_mut(&mut self) -> &mut Session {
        &mut *self.session
    }

    fn composition_mut(&mut self) -> &mut composition::Store {
        &mut *self.composition
    }

    fn diagnostics_mut(&mut self) -> &mut diagnostics::Store {
        &mut *self.diagnostics
    }
}

impl Target<CloseWindow> for Service<'_> {
    fn state(&self, _args: &(), _cx: &Context) -> command::State {
        window_state(self.target_window().is_some())
    }

    fn invoke(&mut self, _args: (), _cx: &mut Context) -> Response<()> {
        let Some(window) = self.target_window() else {
            return Response::failed(Error::Disabled {
                command: <CloseWindow as command::Command>::NAME,
            });
        };

        self.session_mut().close_window(window);
        self.composition_mut().remove_window(window);
        self.diagnostics_mut().remove_window(window);

        Response::output(()).with_effect(response::Effect::Repaint)
    }
}

impl Focus {
    pub fn text(target: impl Into<interaction::Id>) -> Self {
        Self {
            target: target.into(),
        }
    }

    pub fn target(self) -> interaction::Id {
        self.target
    }
}

impl Request {
    pub fn file_dialog(window: window::Id, dialog: FileDialog) -> Self {
        Self {
            window,
            kind: RequestKind::FileDialog(dialog),
        }
    }

    pub fn window(self) -> window::Id {
        self.window
    }

    pub fn kind(self) -> RequestKind {
        self.kind
    }
}

impl Window {
    pub fn id(&self) -> window::Id {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn inner_size(&self) -> geometry::Size {
        self.inner_size
    }

    pub fn canvas_color(&self) -> scene::Color {
        self.canvas_color
    }

    pub fn redraw_requested(&self) -> bool {
        self.redraw_requested
    }

    pub fn presented_revision(&self) -> Option<state::Revision> {
        self.presented_revision
    }

    pub fn focus(&self) -> Option<Focus> {
        self.focus
    }

    pub fn interaction(&self) -> &interaction::Interaction {
        &self.interaction
    }
}

impl Snapshot {
    fn new(windows: Vec<WindowSnapshot>) -> Self {
        Self { windows }
    }

    pub fn from_windows(windows: impl IntoIterator<Item = WindowSnapshot>) -> Self {
        Self::new(windows.into_iter().collect())
    }

    pub fn windows(&self) -> &[WindowSnapshot] {
        &self.windows
    }
}

impl WindowSnapshot {
    pub fn new(id: window::Id, title: impl Into<String>, focus: Option<Focus>) -> Self {
        Self {
            id,
            title: title.into(),
            inner_size: window::Options::default_inner_size(),
            canvas_color: window::Options::default_canvas_color(),
            focus,
        }
    }

    fn from_window(window: &Window) -> Self {
        Self {
            id: window.id,
            title: window.title.clone(),
            inner_size: window.inner_size,
            canvas_color: window.canvas_color,
            focus: window.focus,
        }
    }

    pub fn id(&self) -> window::Id {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn inner_size(&self) -> geometry::Size {
        self.inner_size
    }

    pub fn canvas_color(&self) -> scene::Color {
        self.canvas_color
    }

    pub fn focus(&self) -> Option<Focus> {
        self.focus
    }
}

impl Session {
    pub fn open_window(&mut self, options: window::Options) -> window::Id {
        let (title, inner_size, canvas_color) = options.into_parts();
        let id = window::Id::new(self.next_window_id);
        self.next_window_id += 1;
        self.windows.push(Window {
            id,
            title,
            inner_size,
            canvas_color,
            redraw_requested: true,
            presented_revision: None,
            focus: None,
            file_dialog: None,
            interaction: interaction::Interaction::default(),
        });

        id
    }

    pub fn close_window(&mut self, id: window::Id) -> bool {
        let Some(index) = self.windows.iter().position(|window| window.id == id) else {
            return false;
        };

        self.windows.remove(index);
        true
    }

    pub fn request_redraw(&mut self, id: window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = !window.redraw_requested;
        window.redraw_requested = true;
        changed
    }

    pub fn clear_redraw_request(&mut self, id: window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = window.redraw_requested;
        window.redraw_requested = false;
        changed
    }

    pub(super) fn mark_presented(&mut self, id: window::Id, revision: state::Revision) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = window.presented_revision != Some(revision);
        window.presented_revision = Some(revision);
        changed
    }

    pub fn focus(&mut self, id: window::Id, focus: Focus) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let target = interaction::Target::text_area(focus);
        let changed = window.focus != Some(focus);
        let input_changed = window.interaction.clear_text_input_unless(&target);
        window.focus = Some(focus);
        changed || input_changed
    }

    pub fn clear_focus(&mut self, id: window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = window.focus.is_some();
        let input_changed = window.interaction.clear_text_input();
        window.focus = None;
        changed || input_changed
    }

    pub fn request_file_dialog(&mut self, id: window::Id, dialog: FileDialog) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = window.file_dialog != Some(dialog);
        window.file_dialog = Some(dialog);
        changed
    }

    pub fn open_menu(&mut self, id: window::Id, menu: interaction::Menu) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.open_menu_with(menu)
    }

    pub fn toggle_menu(&mut self, id: window::Id, menu: interaction::Menu) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.toggle_menu(menu)
    }

    pub fn close_menu(&mut self, id: window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.close_menu()
    }

    pub fn pointer_move(&mut self, id: window::Id, target: Option<interaction::Target>) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.pointer_move(target)
    }

    pub fn pointer_down(&mut self, id: window::Id, target: interaction::Target) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.pointer_down(target)
    }

    pub fn pointer_up(&mut self, id: window::Id, target: Option<interaction::Target>) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.pointer_up(target)
    }

    pub fn pointer_left(&mut self, id: window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.pointer_left()
    }

    pub fn cancel_pointer(&mut self, id: window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.cancel_pointer()
    }

    pub fn scroll_by(
        &mut self,
        id: window::Id,
        target: interaction::Target,
        delta: interaction::ScrollDelta,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.scroll_by(target, delta)
    }

    pub fn reveal_scroll(&mut self, id: window::Id, target: interaction::Target) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.reveal_scroll(target)
    }

    pub fn resolve_scroll(
        &mut self,
        id: window::Id,
        target: interaction::Target,
        offset: interaction::ScrollOffset,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        let scrolled = window.interaction.scroll_to(target.clone(), offset);
        let revealed = window.interaction.clear_scroll_reveal(&target);
        scrolled || revealed
    }

    pub fn set_text_preedit(&mut self, id: window::Id, preedit: text::Preedit) -> Option<bool> {
        let window = self.window_mut(id)?;
        let target = interaction::Target::text_area(window.focus?);

        Some(window.interaction.set_text_preedit(target, preedit))
    }

    pub fn set_text_preedit_for(
        &mut self,
        id: window::Id,
        target: interaction::Target,
        preedit: text::Preedit,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.set_text_preedit(target, preedit)
    }

    pub fn edit_text_draft(
        &mut self,
        id: window::Id,
        focus: Focus,
        base: impl Into<String>,
        edit: text::edit::Edit,
    ) -> Option<draft::Change> {
        let window = self.window_mut(id)?;
        if window.focus != Some(focus) {
            return None;
        }

        Some(
            window
                .interaction
                .edit_text_draft(interaction::Target::text_area(focus), base, edit),
        )
    }

    pub fn clear_text_input(&mut self, id: window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.clear_text_input()
    }

    pub fn take_file_dialog(&mut self, id: window::Id) -> Option<FileDialog> {
        self.window_mut(id)?.file_dialog.take()
    }

    pub fn windows(&self) -> &[Window] {
        &self.windows
    }

    pub fn window(&self, id: window::Id) -> Option<&Window> {
        self.windows.iter().find(|window| window.id == id)
    }

    pub fn contains(&self, id: window::Id) -> bool {
        self.window(id).is_some()
    }

    pub fn focused(&self, id: window::Id) -> Option<Focus> {
        self.window(id).and_then(Window::focus)
    }

    pub fn interaction(&self, id: window::Id) -> Option<&interaction::Interaction> {
        self.window(id).map(Window::interaction)
    }

    pub fn file_dialog(&self, id: window::Id) -> Option<FileDialog> {
        self.window(id).and_then(|window| window.file_dialog)
    }

    pub fn requests(&self) -> Vec<Request> {
        self.windows
            .iter()
            .filter_map(|window| {
                window
                    .file_dialog
                    .map(|dialog| Request::file_dialog(window.id, dialog))
            })
            .collect()
    }

    pub(super) fn snapshot(&self) -> Snapshot {
        Snapshot::new(
            self.windows
                .iter()
                .map(WindowSnapshot::from_window)
                .collect(),
        )
    }

    pub(super) fn restore(&mut self, snapshot: Snapshot) {
        self.windows = snapshot
            .windows
            .into_iter()
            .map(|window| Window {
                id: window.id,
                title: window.title,
                inner_size: window.inner_size,
                canvas_color: window.canvas_color,
                redraw_requested: true,
                presented_revision: None,
                focus: window.focus,
                file_dialog: None,
                interaction: interaction::Interaction::default(),
            })
            .collect();
        self.next_window_id = self
            .windows
            .iter()
            .map(|window| window.id.get() + 1)
            .max()
            .unwrap_or_default();
    }

    fn window_mut(&mut self, id: window::Id) -> Option<&mut Window> {
        self.windows.iter_mut().find(|window| window.id == id)
    }
}

pub(super) fn register(commands: &mut command::Registry) {
    commands.register::<CloseWindow>(command::Spec::new("Exit").shortcut("Alt+F4"));
}

fn window_state(enabled: bool) -> command::State {
    if enabled {
        command::State::enabled()
    } else {
        command::State::disabled()
    }
}
