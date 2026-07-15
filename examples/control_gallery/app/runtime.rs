use super::{
    State,
    command::{
        ClearFeedback, EditRecordCount, EditRecordNote, IncrementClicks, OpenRecord, ResetControls,
        SelectMode, SelectRendererViewport, SetLevel, SetRecordEnabled, SetRendererWorkload,
        ShowInfoFeedback, ShowWarningFeedback, SubmitQuery, ToggleAdvanced, ToggleExpandedRows,
        ToggleGrid, ToggleWrap, WriteRendererReceipt,
    },
    view,
    view::{CANVAS_COLOR, WINDOW_TITLE, window_size},
};
use wgpu_l3::{Runtime, View, command, document, feedback, window};

struct ControlsMenu;

#[derive(Debug, Clone)]
pub enum Event {
    Report(feedback::Severity),
    ClearFeedback,
    BeginRendererMeasurement,
    WriteRendererReceipt(String),
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
                )
                .register::<WriteRendererReceipt>(
                    command::Spec::new("Write renderer receipt")
                        .shortcut("Primary+Shift+R")
                        .description("Write local renderer telemetry beside the gallery executable")
                        .placement(command::menu::Placement::category(
                            command::menu::Category::of::<ControlsMenu>(),
                        )),
                )
                .register::<SelectRendererViewport>(command::Spec::new("Select renderer viewport"))
                .register::<SetRendererWorkload>(command::Spec::new("Set renderer workload"));
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
                .target::<ClearFeedback>()
                .target::<SelectRendererViewport>()
                .target::<SetRendererWorkload>()
                .target::<WriteRendererReceipt>();
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
                Event::BeginRendererMeasurement => {
                    if let Some(diagnostics) = cx.diagnostics_mut(window) {
                        diagnostics.begin_renderer_measurement();
                    }
                }
                Event::WriteRendererReceipt(workload) => {
                    let captured_unix_ms = std::time::SystemTime::now()
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis();
                    let result = std::env::current_exe()
                        .map_err(|error| error.to_string())
                        .and_then(|executable| {
                            executable
                                .parent()
                                .map(|parent| parent.join("renderer-receipts"))
                                .ok_or_else(|| {
                                    "gallery executable has no parent directory".to_owned()
                                })
                        })
                        .and_then(|directory| {
                            let path = directory.join(format!(
                                "{}-{captured_unix_ms}.txt",
                                receipt_file_stem(&workload)
                            ));
                            let diagnostics = cx.diagnostics(window).ok_or_else(|| {
                                "renderer diagnostics are not available yet".to_owned()
                            })?;
                            std::fs::create_dir_all(&directory)
                                .map_err(|error| error.to_string())?;
                            std::fs::write(&path, diagnostics.renderer_receipt_text(&workload))
                                .map_err(|error| error.to_string())?;
                            Ok(path)
                        });
                    match result {
                        Ok(path) => {
                            cx.report_feedback(
                                window,
                                feedback::Severity::Info,
                                format!("Renderer receipt written to {}", path.display()),
                            );
                        }
                        Err(error) => {
                            cx.report_feedback(
                                window,
                                feedback::Severity::Warning,
                                format!("Renderer receipt failed: {error}"),
                            );
                        }
                    }
                }
            }
        })
        .view(view::view)
}

fn receipt_file_stem(workload: &str) -> String {
    let stem: String = workload
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect();
    let stem = stem.trim_matches('-');
    if stem.is_empty() {
        "control-gallery-manual".to_owned()
    } else {
        stem.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::receipt_file_stem;

    #[test]
    fn receipt_filename_keeps_workload_identity_local_and_portable() {
        assert_eq!(
            receipt_file_stem("control gallery / 500px scroll"),
            "control-gallery---500px-scroll"
        );
        assert_eq!(receipt_file_stem("   "), "control-gallery-manual");
    }
}
