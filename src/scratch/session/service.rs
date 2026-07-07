use super::super::{
    command, composition,
    context::Context,
    diagnostics,
    error::Error,
    response::{self, Response},
    target::Target,
    window,
};
use super::Session;

pub struct CloseWindow;

pub struct OpenCommandPalette;

pub(in crate::scratch) struct Service<'a> {
    session: &'a mut Session,
    composition: &'a mut composition::Store,
    diagnostics: &'a mut diagnostics::Store,
    window: Option<window::Id>,
}

impl command::Command for CloseWindow {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "window.close";
    const HISTORY: command::History = command::History::Ignored;
}

impl command::Command for OpenCommandPalette {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "command_palette.open";
    const HISTORY: command::History = command::History::Ignored;
}

impl<'a> Service<'a> {
    pub(in crate::scratch) fn new(
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
            None => session.windows().first().map(super::Window::id),
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

        Response::output(()).with_effect(response::Effect::Rebuild)
    }
}

impl Target<OpenCommandPalette> for Service<'_> {
    fn state(&self, _args: &(), _cx: &Context) -> command::State {
        window_state(self.target_window().is_some())
    }

    fn invoke(&mut self, _args: (), _cx: &mut Context) -> Response<()> {
        let Some(window) = self.target_window() else {
            return Response::failed(Error::Disabled {
                command: <OpenCommandPalette as command::Command>::NAME,
            });
        };

        self.session_mut().open_command_palette(window);

        Response::output(()).with_effect(response::Effect::Rebuild)
    }
}

pub(in crate::scratch) fn register(commands: &mut command::Registry) {
    commands
        .register::<CloseWindow>(
            command::Spec::new("Exit")
                .key_chord(command::KeyChord::standard(command::Standard::CloseWindow)),
        )
        .register::<OpenCommandPalette>(command::Spec::new("Command Palette").key_chord(
            command::KeyChord::standard(command::Standard::CommandPalette),
        ));
}

fn window_state(enabled: bool) -> command::State {
    if enabled {
        command::State::enabled()
    } else {
        command::State::disabled()
    }
}
