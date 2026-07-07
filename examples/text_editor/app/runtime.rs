use super::super::{Clipboard, Runtime, Shell, View, command, document, platform, state, window};
use super::{
    State,
    command::{LoadStressText, ToggleDebugPanel, ToggleWrapText},
    event::Event,
    target, view,
    view::{CANVAS_COLOR, WINDOW_TITLE, window_size},
};

pub fn runtime(state: State) -> Runtime<State, Event> {
    Runtime::new(state)
        .commands(|commands| {
            commands
                .register::<document::ApplyEdit>(command::Spec::new("Edit"))
                .register::<document::NewFile>(command::Spec::new("New").shortcut("Primary+N"))
                .register::<document::OpenFile>(command::Spec::new("Open").shortcut("Primary+O"))
                .register::<document::OpenPath>(command::Spec::new("Open Path"))
                .register::<document::OpenCanceled>(command::Spec::new("Open Canceled"))
                .register::<document::SaveFile>(command::Spec::new("Save").shortcut("Primary+S"))
                .register::<document::SaveAsFile>(
                    command::Spec::new("Save As").shortcut("Primary+Shift+S"),
                )
                .register::<document::SaveToPath>(command::Spec::new("Save To Path"))
                .register::<document::SaveCanceled>(command::Spec::new("Save Canceled"))
                .register::<document::Cut>(
                    command::Spec::new("Cut")
                        .key_chord(command::KeyChord::standard(command::Standard::Cut)),
                )
                .register::<document::Copy>(
                    command::Spec::new("Copy")
                        .key_chord(command::KeyChord::standard(command::Standard::Copy)),
                )
                .register::<document::Paste>(
                    command::Spec::new("Paste")
                        .key_chord(command::KeyChord::standard(command::Standard::Paste)),
                )
                .register::<document::Delete>(command::Spec::new("Delete"))
                .register::<document::SelectAll>(
                    command::Spec::new("Select All")
                        .key_chord(command::KeyChord::standard(command::Standard::SelectAll)),
                )
                .register::<LoadStressText>(command::Spec::new("Load Stress Text"))
                .register::<ToggleWrapText>(command::Spec::new("Wrap text"))
                .register::<ToggleDebugPanel>(command::Spec::new("Debug panel"));
        })
        .responders(|responders| {
            responders
                .app()
                .target::<document::NewFile>()
                .target::<document::OpenFile>()
                .target::<document::OpenPath>()
                .target::<document::OpenCanceled>()
                .target::<document::SaveFile>()
                .target::<document::SaveAsFile>()
                .target::<document::SaveToPath>()
                .target::<document::SaveCanceled>()
                .target::<LoadStressText>()
                .target::<ToggleWrapText>()
                .target::<ToggleDebugPanel>();
            responders
                .object("document", |state: &mut State| &mut state.document)
                .target::<document::ApplyEdit>()
                .target::<document::Cut>()
                .target::<document::Copy>()
                .target::<document::Paste>()
                .target::<document::Delete>()
                .target::<document::SelectAll>();
        })
        .observe::<document::ApplyEdit>(target::record_apply_edit_status)
        .observe::<document::SelectAll>(|state, result, observation| {
            target::record_text_command_status(state, result, "select all", observation);
        })
        .observe::<document::Copy>(|state, result, observation| {
            target::record_text_command_status(state, result, "copy", observation);
        })
        .observe::<document::Cut>(|state, result, observation| {
            target::record_text_command_status(state, result, "cut", observation);
        })
        .observe::<document::Delete>(|state, result, observation| {
            target::record_text_command_status(state, result, "delete", observation);
        })
        .observe::<document::Paste>(|state, result, observation| {
            target::record_text_command_status(state, result, "paste", observation);
        })
        .event(|cx, event| match event {
            Event::FileSaved { path, result } => {
                cx.change(state::Reason::event("file_saved"), |state| {
                    target::finish_save(state, path, result);
                });
            }
        })
}

pub fn app(state: State) -> Runtime<State, Event, View> {
    runtime(state)
        .started(|cx| {
            cx.open_window(
                window::Options::new(WINDOW_TITLE)
                    .with_inner_size(window_size())
                    .with_canvas_color(CANVAS_COLOR),
            );
        })
        .view(view::view)
}

#[cfg(test)]
pub fn shell(state: State) -> Shell<State, Event> {
    Shell::new(app(state))
}

pub fn native_shell(state: State) -> Shell<State, Event> {
    Shell::new(app(state).with_clipboard(Clipboard::system()))
}

pub fn runner(state: State) -> platform::Runner<State, Event> {
    platform::Runner::new(native_shell(state))
}

pub fn run(state: State) -> Result<(), platform::RunError<platform::NativeError>> {
    runner(state).run()
}
