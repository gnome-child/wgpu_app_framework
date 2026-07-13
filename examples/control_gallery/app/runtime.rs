use super::{
    State,
    command::{
        ClearFeedback, EditRecordCount, EditRecordNote, IncrementClicks, OpenRecord, ResetControls,
        SelectMode, SetLevel, SetRecordEnabled, ShowInfoFeedback, ShowWarningFeedback, SubmitQuery,
        ToggleAdvanced, ToggleExpandedRows, ToggleGrid, ToggleWrap,
    },
    view,
    view::{CANVAS_COLOR, WINDOW_TITLE, window_size},
};
use wgpu_l3::{Runtime, View, command, document, feedback, window};

struct ControlsMenu;

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Report(feedback::Severity),
    ClearFeedback,
}

pub fn app(state: State) -> Runtime<State, Event, View> {
    Runtime::new(state)
        .commands(|commands| {
            commands
                .install(document::Editing::standard())
                .menu_category(command::menu::Category::new::<ControlsMenu>("Controls"))
                .register::<IncrementClicks>(
                    command::Spec::new("Click")
                        .description("Increment the gallery click counter")
                        .shortcut("Primary+K")
                        .placement(command::menu::Placement::category(
                            command::menu::Category::of::<ControlsMenu>(),
                        )),
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
                .register::<ToggleExpandedRows>(
                    command::Spec::new("Expanded rows")
                        .description("Switch between compact and wrapped table rows"),
                )
                .register::<OpenRecord>(
                    command::Spec::new("Open record").description("Open the row under inspection"),
                )
                .register::<ShowInfoFeedback>(
                    command::Spec::new("Show information")
                        .description("Show a nonblocking informational runtime fact")
                        .placement(command::menu::Placement::category(
                            command::menu::Category::of::<ControlsMenu>(),
                        )),
                )
                .register::<ShowWarningFeedback>(
                    command::Spec::new("Show warning")
                        .description("Show a warning without trapping focus")
                        .placement(command::menu::Placement::category(
                            command::menu::Category::of::<ControlsMenu>(),
                        )),
                )
                .register::<ClearFeedback>(
                    command::Spec::new("Clear feedback")
                        .description("Clear current runtime feedback")
                        .placement(command::menu::Placement::category(
                            command::menu::Category::of::<ControlsMenu>(),
                        )),
                );
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
                .target::<OpenRecord>()
                .target::<ShowInfoFeedback>()
                .target::<ShowWarningFeedback>()
                .target::<ClearFeedback>();
        })
        .started(|cx| {
            cx.open_window(
                window::Options::new(WINDOW_TITLE)
                    .with_inner_size(window_size())
                    .with_canvas_color(CANVAS_COLOR),
            );
        })
        .event(|cx, event| {
            let Some(window) = cx.windows().first().map(wgpu_l3::session::Window::id) else {
                return;
            };
            match event {
                Event::Report(feedback::Severity::Info) => {
                    cx.report_feedback(window, feedback::Severity::Info, "The gallery is ready");
                }
                Event::Report(feedback::Severity::Warning) => {
                    cx.report_feedback(
                        window,
                        feedback::Severity::Warning,
                        "This is a nonblocking warning; focus remains free to move",
                    );
                }
                Event::Report(feedback::Severity::Error) => {
                    unreachable!("table validation demonstrates errors without a gallery command")
                }
                Event::ClearFeedback => {
                    cx.clear_all_feedback(window);
                }
            }
        })
        .view(view::view)
}
