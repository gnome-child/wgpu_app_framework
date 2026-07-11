use super::{
    State,
    command::{
        CycleForegroundMode, ForegroundDisabledItem, ForegroundEnabledItem, SetToken,
        ToggleComparison, ToggleForcePromoted, TogglePanel,
    },
    view,
    view::{CANVAS_COLOR, WINDOW_TITLE, window_size},
};
use wgpu_l3::{Runtime, View, command, window};

pub fn app(state: State) -> Runtime<State, (), View> {
    Runtime::new(state)
        .commands(|commands| {
            commands
                .register::<TogglePanel>(command::Spec::new("Toggle panel"))
                .register::<ToggleComparison>(command::Spec::new("Toggle comparison"))
                .register::<ToggleForcePromoted>(command::Spec::new("Toggle forced promotion"))
                .register::<CycleForegroundMode>(command::Spec::new("Cycle foreground mode"))
                .register::<ForegroundEnabledItem>(
                    command::Spec::new("Enabled menu text").shortcut("Primary+E"),
                )
                .register::<ForegroundDisabledItem>(
                    command::Spec::new("Disabled menu text").shortcut("Primary+D"),
                )
                .register::<SetToken>(command::Spec::new("Set acrylic token"));
        })
        .responders(|responders| {
            responders
                .app()
                .target::<TogglePanel>()
                .target::<ToggleComparison>()
                .target::<ToggleForcePromoted>()
                .target::<CycleForegroundMode>()
                .target::<ForegroundEnabledItem>()
                .target::<ForegroundDisabledItem>()
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
