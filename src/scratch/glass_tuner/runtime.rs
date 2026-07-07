use super::super::{Runtime, Shell, View, command, platform, window};
use super::{
    State,
    command::{SetToken, TogglePanel},
    view,
    view::{CANVAS_COLOR, WINDOW_TITLE, window_size},
};

pub fn app(state: State) -> Runtime<State, (), View> {
    Runtime::new(state)
        .commands(|commands| {
            commands
                .register::<TogglePanel>(command::Spec::new("Toggle panel"))
                .register::<SetToken>(command::Spec::new("Set acrylic token"));
        })
        .responders(|responders| {
            responders
                .app()
                .target::<TogglePanel>()
                .target::<SetToken>();
        })
        .started(|cx| {
            cx.open_window(
                window::Options::new(WINDOW_TITLE)
                    .with_inner_size(window_size())
                    .with_canvas_color(CANVAS_COLOR),
            );
        })
        .theme(State::theme)
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
