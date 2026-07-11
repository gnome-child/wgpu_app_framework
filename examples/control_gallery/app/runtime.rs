use super::{
    State,
    command::{
        EditRecordCount, EditRecordNote, IncrementClicks, ResetControls, SelectMode, SetLevel,
        SortRecords, SubmitQuery, ToggleAdvanced, ToggleGrid, ToggleWrap,
    },
    view,
    view::{CANVAS_COLOR, WINDOW_TITLE, window_size},
};
use wgpu_l3::{Runtime, View, command, document, window};

pub fn app(state: State) -> Runtime<State, (), View> {
    Runtime::new(state)
        .commands(|commands| {
            commands
                .install(document::Editing::standard())
                .register::<IncrementClicks>(command::Spec::new("Click").shortcut("Primary+K"))
                .register::<ToggleWrap>(command::Spec::new("Wrap text"))
                .register::<ToggleGrid>(command::Spec::new("Show grid"))
                .register::<SelectMode>(command::Spec::new("Select mode"))
                .register::<SetLevel>(command::Spec::new("Set level"))
                .register::<SubmitQuery>(command::Spec::new("Submit query"))
                .register::<ToggleAdvanced>(command::Spec::new("Advanced"))
                .register::<ResetControls>(command::Spec::new("Reset").shortcut("Primary+R"))
                .register::<SortRecords>(command::Spec::new("Sort records"))
                .register::<EditRecordNote>(command::Spec::new("Edit record note"))
                .register::<EditRecordCount>(command::Spec::new("Edit record count"));
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
                .target::<ResetControls>()
                .target::<SortRecords>()
                .target::<EditRecordNote>()
                .target::<EditRecordCount>();
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
