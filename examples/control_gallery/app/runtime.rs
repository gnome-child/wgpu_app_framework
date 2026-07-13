use super::{
    State,
    command::{
        EditRecordCount, EditRecordNote, IncrementClicks, OpenRecord, ResetControls, SelectMode,
        SetLevel, SetRecordEnabled, SubmitQuery, ToggleAdvanced, ToggleExpandedRows, ToggleGrid,
        ToggleWrap,
    },
    view,
    view::{CANVAS_COLOR, WINDOW_TITLE, window_size},
};
use wgpu_l3::{Runtime, View, command, document, window};

struct ControlsMenu;

pub fn app(state: State) -> Runtime<State, (), View> {
    Runtime::new(state)
        .commands(|commands| {
            commands
                .install(document::Editing::standard())
                .menu_category(command::menu::Category::new::<ControlsMenu>("Controls"))
                .register::<IncrementClicks>(
                    command::Spec::new("Click").shortcut("Primary+K").placement(
                        command::menu::Placement::category(command::menu::Category::of::<
                            ControlsMenu,
                        >()),
                    ),
                )
                .register::<ToggleWrap>(command::Spec::new("Wrap text").placement(
                    command::menu::Placement::category(command::menu::Category::VIEW),
                ))
                .register::<ToggleGrid>(command::Spec::new("Show grid").placement(
                    command::menu::Placement::category(command::menu::Category::VIEW),
                ))
                .register::<SelectMode>(command::Spec::new("Select mode"))
                .register::<SetLevel>(command::Spec::new("Set level"))
                .register::<SubmitQuery>(command::Spec::new("Submit query"))
                .register::<ToggleAdvanced>(command::Spec::new("Advanced").placement(
                    command::menu::Placement::category(command::menu::Category::VIEW),
                ))
                .register::<ResetControls>(
                    command::Spec::new("Reset").shortcut("Primary+R").placement(
                        command::menu::Placement::category(command::menu::Category::of::<
                            ControlsMenu,
                        >()),
                    ),
                )
                .register::<wgpu_l3::table::SortBy>(command::Spec::new("Sort table"))
                .register::<EditRecordNote>(command::Spec::new("Edit record note"))
                .register::<EditRecordCount>(command::Spec::new("Edit record count"))
                .register::<SetRecordEnabled>(command::Spec::new("Set record enabled"))
                .register::<ToggleExpandedRows>(command::Spec::new("Expanded rows"))
                .register::<OpenRecord>(command::Spec::new("Open record"));
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
                .target::<wgpu_l3::table::SortBy>()
                .target::<EditRecordNote>()
                .target::<EditRecordCount>()
                .target::<SetRecordEnabled>()
                .target::<ToggleExpandedRows>()
                .target::<OpenRecord>();
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
