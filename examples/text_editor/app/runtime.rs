use super::{
    State,
    command::{LoadStressText, ToggleDebugPanel, ToggleWrapText},
    event::Event,
    target, view,
    view::{CANVAS_COLOR, WINDOW_TITLE, window_size},
};
#[cfg(test)]
use wgpu_l3::Shell;
use wgpu_l3::{Runtime, View, command, document, state, window};

pub fn runtime(state: State) -> Runtime<State, Event> {
    Runtime::new(state)
        .commands(|commands| {
            commands
                .install(document::Editing::standard())
                .register::<document::NewFile>(command::Spec::standard(command::Standard::New))
                .register::<document::OpenFile>(command::Spec::standard(command::Standard::Open))
                .register::<document::OpenPath>(command::Spec::new("Open Path"))
                .register::<document::SaveFile>(command::Spec::standard(command::Standard::Save))
                .register::<document::SaveAsFile>(command::Spec::standard(
                    command::Standard::SaveAs,
                ))
                .register::<document::SaveToPath>(command::Spec::new("Save To Path"))
                .register::<LoadStressText>(
                    command::Spec::new("Load Stress Text")
                        .description("Replace the document with generated stress text")
                        .placement(command::menu::Placement::section_after(
                            command::Standard::SaveAs,
                        )),
                )
                .register::<ToggleWrapText>(command::Spec::new("Wrap text").placement(
                    command::menu::Placement::category(command::menu::Category::VIEW),
                ))
                .register::<ToggleDebugPanel>(command::Spec::new("Debug panel").placement(
                    command::menu::Placement::category(command::menu::Category::VIEW),
                ));
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
                .target::<document::ApplySelection>()
                .target::<document::Cut>()
                .target::<document::Copy>()
                .target::<document::Paste>()
                .target::<document::Delete>()
                .target::<document::SelectAll>();
        })
        .observe::<document::ApplyEdit>(target::record_apply_edit_status)
        .observe::<document::ApplySelection>(target::record_apply_edit_status)
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
