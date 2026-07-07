use super::super::{Runtime, Shell, View, command, document, platform, window};
use super::{
    State,
    command::{
        IncrementClicks, ResetControls, SelectMode, SetLevel, SubmitQuery, ToggleAdvanced,
        ToggleGrid, ToggleWrap,
    },
    view,
    view::{CANVAS_COLOR, WINDOW_TITLE, window_size},
};

pub fn app(state: State) -> Runtime<State, (), View> {
    Runtime::new(state)
        .commands(|commands| {
            commands
                .register::<IncrementClicks>(command::Spec::new("Click").shortcut("Primary+K"))
                .register::<ToggleWrap>(command::Spec::new("Wrap text"))
                .register::<ToggleGrid>(command::Spec::new("Show grid"))
                .register::<SelectMode>(command::Spec::new("Select mode"))
                .register::<SetLevel>(command::Spec::new("Set level"))
                .register::<SubmitQuery>(command::Spec::new("Submit query"))
                .register::<ToggleAdvanced>(command::Spec::new("Advanced"))
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
                .register::<ResetControls>(command::Spec::new("Reset").shortcut("Primary+R"));
        })
        .responders(|responders| {
            responders
                .app()
                .target::<IncrementClicks>()
                .target::<ToggleWrap>()
                .target::<ToggleGrid>()
                .target::<SelectMode>()
                .target::<SetLevel>()
                .target::<SubmitQuery>()
                .target::<ToggleAdvanced>()
                .target::<ResetControls>();
        })
        .started(|cx| {
            cx.open_window(
                window::Options::new(WINDOW_TITLE)
                    .with_inner_size(window_size())
                    .with_canvas_color(CANVAS_COLOR),
            );
        })
        .view(view::view)
}

pub fn shell(state: State) -> Shell<State> {
    Shell::new(app(state))
}

pub fn runner(state: State) -> platform::Runner<State> {
    platform::Runner::new(shell(state))
}

pub fn run(state: State) -> Result<(), platform::RunError<platform::NativeError>> {
    runner(state).run()
}
