use super::{
    State,
    command::{SetToken, ToggleComparison, TogglePanel},
    view,
    view::{CANVAS_COLOR, WINDOW_TITLE, window_size},
};
use wgpu_l3::{Runtime, View, command, window};
#[cfg(not(test))]
use wgpu_l3::{Shell, platform};

pub fn app(state: State) -> Runtime<State, (), View> {
    Runtime::new(state)
        .commands(|commands| {
            commands
                .register::<TogglePanel>(command::Spec::new("Toggle panel"))
                .register::<ToggleComparison>(command::Spec::new("Toggle comparison"))
                .register::<SetToken>(command::Spec::new("Set acrylic token"));
        })
        .responders(|responders| {
            responders
                .app()
                .target::<TogglePanel>()
                .target::<ToggleComparison>()
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

#[cfg(not(test))]
pub fn shell(state: State) -> Shell<State> {
    Shell::new(app(state))
}

#[cfg(not(test))]
pub fn runner(state: State) -> platform::Runner<State> {
    platform::Runner::new(shell(state))
}

#[cfg(not(test))]
pub fn run(state: State) -> Result<(), platform::RunError<platform::NativeError>> {
    runner(state).run()
}
