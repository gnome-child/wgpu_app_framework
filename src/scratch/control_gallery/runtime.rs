use super::super::{Runtime as FrameworkRuntime, Shell, View, command, document, platform, window};
use super::{
    State,
    command::{
        IncrementClicks, ResetControls, SelectMode, SetLevel, SubmitQuery, ToggleAdvanced,
        ToggleGrid, ToggleWrap,
    },
    view,
    view::{CANVAS_COLOR, WINDOW_TITLE, window_size},
};

pub fn app(state: State) -> FrameworkRuntime<State, (), View> {
    FrameworkRuntime::new(state)
        .commands(|commands| {
            commands
                .register::<IncrementClicks>(command::Spec::new("Click").shortcut("Ctrl+K"))
                .register::<ToggleWrap>(command::Spec::new("Wrap text"))
                .register::<ToggleGrid>(command::Spec::new("Show grid"))
                .register::<SelectMode>(command::Spec::new("Select mode"))
                .register::<SetLevel>(command::Spec::new("Set level"))
                .register::<SubmitQuery>(command::Spec::new("Submit query"))
                .register::<ToggleAdvanced>(command::Spec::new("Advanced"))
                .register::<document::Cut>(command::Spec::new("Cut").shortcut("Ctrl+X"))
                .register::<document::Copy>(command::Spec::new("Copy").shortcut("Ctrl+C"))
                .register::<document::Paste>(command::Spec::new("Paste").shortcut("Ctrl+V"))
                .register::<document::Delete>(command::Spec::new("Delete"))
                .register::<document::SelectAll>(
                    command::Spec::new("Select All").shortcut("Ctrl+A"),
                )
                .register::<ResetControls>(command::Spec::new("Reset").shortcut("Ctrl+R"));
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
