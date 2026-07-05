use super::super::{
    Clipboard, Runtime as FrameworkRuntime, Shell, View, command, document, platform, state, window,
};
use super::{
    Event, LoadStressText, State, ToggleDebugPanel, ToggleWrapText, target, view,
    view::{CANVAS_COLOR, WINDOW_TITLE, window_size},
};

pub fn runtime(state: State) -> FrameworkRuntime<State, Event> {
    FrameworkRuntime::new(state)
        .commands(|commands| {
            commands
                .register::<document::ApplyEdit>(command::Spec::new("Edit"))
                .register::<document::NewFile>(command::Spec::new("New").shortcut("Ctrl+N"))
                .register::<document::OpenFile>(command::Spec::new("Open").shortcut("Ctrl+O"))
                .register::<document::OpenPath>(command::Spec::new("Open Path"))
                .register::<document::OpenCanceled>(command::Spec::new("Open Canceled"))
                .register::<document::SaveFile>(command::Spec::new("Save").shortcut("Ctrl+S"))
                .register::<document::SaveAsFile>(
                    command::Spec::new("Save As").shortcut("Ctrl+Shift+S"),
                )
                .register::<document::SaveToPath>(command::Spec::new("Save To Path"))
                .register::<document::SaveCanceled>(command::Spec::new("Save Canceled"))
                .register::<document::Cut>(command::Spec::new("Cut").shortcut("Ctrl+X"))
                .register::<document::Copy>(command::Spec::new("Copy").shortcut("Ctrl+C"))
                .register::<document::Paste>(command::Spec::new("Paste").shortcut("Ctrl+V"))
                .register::<document::Delete>(command::Spec::new("Delete"))
                .register::<document::SelectAll>(
                    command::Spec::new("Select All").shortcut("Ctrl+A"),
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

pub fn app(state: State) -> FrameworkRuntime<State, Event, View> {
    runtime(state)
        .started(|cx| {
            cx.open_window(
                window::Options::new(WINDOW_TITLE)
                    .with_inner_size(window_size())
                    .with_canvas_color(CANVAS_COLOR),
            );
        })
        .view(view)
}

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
