use super::{
    State,
    command::{LoadStressText, ToggleDebugPanel, ToggleWrapText},
    event::Event,
    target, view,
    view::{CANVAS_COLOR, WINDOW_TITLE, window_size},
};
use wgpu_l3::{Clipboard, Runtime, Shell, View, command, document, platform, state, window};

pub fn runtime(state: State) -> Runtime<State, Event> {
    Runtime::new(state)
        .commands(|commands| {
            commands
                .install(document::Editing::standard())
                .register::<document::NewFile>(command::Spec::new("New").shortcut("Primary+N"))
                .register::<document::OpenFile>(command::Spec::new("Open").shortcut("Primary+O"))
                .register::<document::OpenPath>(command::Spec::new("Open Path"))
                .register::<document::SaveFile>(command::Spec::new("Save").shortcut("Primary+S"))
                .register::<document::SaveAsFile>(
                    command::Spec::new("Save As").shortcut("Primary+Shift+S"),
                )
                .register::<document::SaveToPath>(command::Spec::new("Save To Path"))
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
                .target::<document::SaveFile>()
                .target::<document::SaveAsFile>()
                .target::<document::SaveToPath>()
                .target::<LoadStressText>()
                .target::<ToggleWrapText>()
                .target::<ToggleDebugPanel>()
                .listen::<document::OpenDialogCanceled>()
                .listen::<document::SaveDialogCanceled>();
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
            Event::FileSaved {
                version,
                generation,
                path,
                result,
            } => {
                if target::accepts_save_completion(cx.state(), version, generation) {
                    cx.change(state::Reason::event("file_saved"), |state| {
                        target::finish_save(state, version, path, result);
                    });
                }
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
