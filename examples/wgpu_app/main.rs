use std::time::Duration;

use wgpu_l3::{
    Action, Event, Icon, Task, Theme, action, app,
    geometry::{Rect, area, point},
    icon, layout, paint, text, ui, widget, window,
};

const PING: action::Id = action::Id::new("ping");
const RUN_TASK: action::Id = action::Id::new("run_task");
const TOGGLE_PREVIEW: action::Id = action::Id::new("toggle_preview");
const DISABLED_ACTION: action::Id = action::Id::new("disabled_action");

const ROOT: ui::Id = ui::Id::new("root");
const MENU_BAR: ui::Id = ui::Id::new("menu_bar");
const CONTENT: ui::Id = ui::Id::new("content");
const LEFT_COLUMN: ui::Id = ui::Id::new("left_column");
const CENTER_COLUMN: ui::Id = ui::Id::new("center_column");
const RIGHT_COLUMN: ui::Id = ui::Id::new("right_column");

const STATUS_SECTION: ui::Id = ui::Id::new("status_section");
const STATUS_TEXT: ui::Id = ui::Id::new("status_text");
const ACTION_SECTION: ui::Id = ui::Id::new("action_section");
const ACTION_HEADER: ui::Id = ui::Id::new("action_header");
const DEFAULT_BUTTON: ui::Id = ui::Id::new("default_button");
const DISABLED_BUTTON: ui::Id = ui::Id::new("disabled_button");
const RUN_BUTTON: ui::Id = ui::Id::new("run_button");
const PREVIEW_BUTTON: ui::Id = ui::Id::new("preview_button");
const ICON_PREVIEW_BUTTON: ui::Id = ui::Id::new("icon_preview_button");
const POPUP_SECTION: ui::Id = ui::Id::new("popup_section");
const POPUP_HEADER: ui::Id = ui::Id::new("popup_header");
const POPUP_NOTE: ui::Id = ui::Id::new("popup_note");

const COMMAND_SECTION: ui::Id = ui::Id::new("command_section");
const COMMAND_HEADER: ui::Id = ui::Id::new("command_header");
const DOCUMENT_PANEL: ui::Id = ui::Id::new("document_panel");
const COMMAND_ROW: ui::Id = ui::Id::new("command_row");
const SELECT_BUTTON: ui::Id = ui::Id::new("select_button");
const WINDOW_SELECT_BUTTON: ui::Id = ui::Id::new("window_select_button");
const LAYOUT_SECTION: ui::Id = ui::Id::new("layout_section");
const LAYOUT_HEADER: ui::Id = ui::Id::new("layout_header");
const FIT_BUTTON: ui::Id = ui::Id::new("fit_button");
const FILL_BUTTON: ui::Id = ui::Id::new("fill_button");
const FIXED_BUTTON: ui::Id = ui::Id::new("fixed_button");
const ALIGN_ROW: ui::Id = ui::Id::new("align_row");
const ALIGN_START: ui::Id = ui::Id::new("align_start");
const ALIGN_CENTER: ui::Id = ui::Id::new("align_center");
const ALIGN_END: ui::Id = ui::Id::new("align_end");
const SCROLL_SECTION: ui::Id = ui::Id::new("scroll_section");
const SCROLL_HEADER: ui::Id = ui::Id::new("scroll_header");
const DOCUMENT_SCROLL: ui::Id = ui::Id::new("document_scroll");
const BOTH_AXIS_SCROLL: ui::Id = ui::Id::new("both_axis_scroll");
const BOTH_AXIS_CONTENT: ui::Id = ui::Id::new("both_axis_content");

const STATE_SECTION: ui::Id = ui::Id::new("state_section");
const STATE_HEADER: ui::Id = ui::Id::new("state_header");
const STATE_READY: ui::Id = ui::Id::new("state_ready");
const STATE_SUBJECT: ui::Id = ui::Id::new("state_subject");
const STATE_LAST: ui::Id = ui::Id::new("state_last");
const STATE_FOOTER: ui::Id = ui::Id::new("state_footer");
const TEXT_SECTION: ui::Id = ui::Id::new("text_section");
const TEXT_HEADER: ui::Id = ui::Id::new("text_header");
const TEXT_LINE: ui::Id = ui::Id::new("text_line");
const TEXT_PARAGRAPH: ui::Id = ui::Id::new("text_paragraph");

const COMMAND_SCOPE_PANEL: ui::Id = ui::Id::new("command_scope_panel");
const SCOPE_STATUS: ui::Id = ui::Id::new("scope_status");
const LOCAL_FIELD: ui::Id = ui::Id::new("local_field");
const LOCAL_SELECT_BUTTON: ui::Id = ui::Id::new("local_select_button");
const CAPTURED_SELECT_BUTTON: ui::Id = ui::Id::new("captured_select_button");

const FILE_MENU: widget::menu::Id = widget::menu::Id::new("file");
const EDIT_MENU: widget::menu::Id = widget::menu::Id::new("edit");
const VIEW_MENU: widget::menu::Id = widget::menu::Id::new("view");
const PANELS_MENU: widget::menu::Id = widget::menu::Id::new("panels");

const SCROLL_ROW_HEIGHT: f32 = 24.0;
const SCROLL_GAP: f32 = 3.0;
const SCROLL_PADDING: f32 = 4.0;

const SCROLL_ROWS: [ui::Id; 14] = [
    ui::Id::new("scroll_row_0"),
    ui::Id::new("scroll_row_1"),
    ui::Id::new("scroll_row_2"),
    ui::Id::new("scroll_row_3"),
    ui::Id::new("scroll_row_4"),
    ui::Id::new("scroll_row_5"),
    ui::Id::new("scroll_row_6"),
    ui::Id::new("scroll_row_7"),
    ui::Id::new("scroll_row_8"),
    ui::Id::new("scroll_row_9"),
    ui::Id::new("scroll_row_10"),
    ui::Id::new("scroll_row_11"),
    ui::Id::new("scroll_row_12"),
    ui::Id::new("scroll_row_13"),
];

const GRID_CELLS: [[ui::Id; 6]; 4] = [
    [
        ui::Id::new("grid_cell_0_0"),
        ui::Id::new("grid_cell_0_1"),
        ui::Id::new("grid_cell_0_2"),
        ui::Id::new("grid_cell_0_3"),
        ui::Id::new("grid_cell_0_4"),
        ui::Id::new("grid_cell_0_5"),
    ],
    [
        ui::Id::new("grid_cell_1_0"),
        ui::Id::new("grid_cell_1_1"),
        ui::Id::new("grid_cell_1_2"),
        ui::Id::new("grid_cell_1_3"),
        ui::Id::new("grid_cell_1_4"),
        ui::Id::new("grid_cell_1_5"),
    ],
    [
        ui::Id::new("grid_cell_2_0"),
        ui::Id::new("grid_cell_2_1"),
        ui::Id::new("grid_cell_2_2"),
        ui::Id::new("grid_cell_2_3"),
        ui::Id::new("grid_cell_2_4"),
        ui::Id::new("grid_cell_2_5"),
    ],
    [
        ui::Id::new("grid_cell_3_0"),
        ui::Id::new("grid_cell_3_1"),
        ui::Id::new("grid_cell_3_2"),
        ui::Id::new("grid_cell_3_3"),
        ui::Id::new("grid_cell_3_4"),
        ui::Id::new("grid_cell_3_5"),
    ],
];

fn main() -> app::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    app::run(App::default())
}

struct App {
    window: Option<window::Id>,
    workspace_ready: bool,
    preview_enabled: bool,
    ping_count: u32,
    run_count: u32,
    select_count: u32,
    last_select_target: Option<String>,
    last_select_origin: Option<String>,
    scope_capture_hint: Option<String>,
    document_scroll: f32,
    grid_scroll: point::Logical,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            workspace_ready: false,
            preview_enabled: false,
            ping_count: 0,
            run_count: 0,
            select_count: 0,
            last_select_target: None,
            last_select_origin: None,
            scope_capture_hint: None,
            document_scroll: 0.0,
            grid_scroll: point::logical(0.0, 0.0),
        }
    }
}

enum AppEvent {
    WorkspaceReady,
    Ping,
    RunTaskFinished,
    TogglePreview,
    SelectAll {
        target: action::Context,
        origin: Option<ui::Path>,
    },
}

impl app::Application for App {
    type Event = AppEvent;

    fn started(&mut self, cx: &mut app::Context<'_, Self::Event>) {
        cx.register_action(Action::new(PING, "Ping").emit(|_| AppEvent::Ping));
        cx.register_action(Action::new(DISABLED_ACTION, "Unavailable").emit(|_| AppEvent::Ping));
        cx.register_action(Action::new(RUN_TASK, "Run Task").task(|_| {
            Task::future(async {
                std::thread::sleep(Duration::from_millis(850));
                AppEvent::RunTaskFinished
            })
        }));
        cx.register_action(
            Action::new(TOGGLE_PREVIEW, "Toggle Preview").emit(|_| AppEvent::TogglePreview),
        );
        cx.register_action(
            Action::new(action::SELECT_ALL, "Select All")
                .with_shortcut(action::Shortcut::control('a'))
                .emit(|invocation| AppEvent::SelectAll {
                    target: invocation.context().clone(),
                    origin: invocation.origin().cloned(),
                }),
        );
        cx.register_action(
            Action::new(action::INSERT_TEXT, "Insert Text").emit(|invocation| {
                let action::Payload::Text(text) = invocation.payload() else {
                    return AppEvent::Ping;
                };

                log::debug!("insert text payload={text:?}");
                AppEvent::Ping
            }),
        );

        let theme = Theme::default_dark();
        let window = cx.open_window(window::Options {
            title: "wgpu_l3 showcase".to_owned(),
            inner_area: area::physical(900, 640),
            canvas_color: theme.surfaces().canvas(),
        });

        self.window = Some(window);
        cx.spawn(Task::future(async {
            std::thread::sleep(Duration::from_millis(700));
            AppEvent::WorkspaceReady
        }));
    }

    fn event(&mut self, cx: &mut app::Context<'_, Self::Event>, event: Event<Self::Event>) {
        let Some(window) = self.window else {
            return;
        };

        match event {
            Event::Ui {
                window: event_window,
                event: ui::Event::ScrollRequested { target, offset },
            } if event_window == window && target == document_scroll_path() => {
                self.document_scroll = offset.y();
                cx.request_redraw(window);
            }
            Event::Ui {
                window: event_window,
                event: ui::Event::ScrollRequested { target, offset },
            } if event_window == window && target == grid_scroll_path() => {
                self.grid_scroll = offset;
                cx.request_redraw(window);
            }
            Event::Ui { .. } => {}
            Event::App(AppEvent::WorkspaceReady) => {
                self.workspace_ready = true;
                cx.focus(window, document_path(), ui::focus::Visibility::Visible);
                cx.request_redraw(window);
            }
            Event::App(AppEvent::Ping) => {
                self.ping_count += 1;
                cx.request_redraw(window);
            }
            Event::App(AppEvent::RunTaskFinished) => {
                self.run_count += 1;
                cx.request_redraw(window);
            }
            Event::App(AppEvent::TogglePreview) => {
                self.preview_enabled = !self.preview_enabled;
                cx.request_redraw(window);
            }
            Event::App(AppEvent::SelectAll { target, origin }) => {
                log::debug!("select all target={target:?} origin={origin:?}");
                self.last_select_target = Some(context_name(&target));
                self.last_select_origin = origin.as_ref().map(path_name);
                self.select_count += 1;
                cx.request_redraw(window);
            }
        }
    }

    fn view(
        &mut self,
        cx: &mut app::Context<'_, Self::Event>,
        window: window::Id,
        tree: &mut ui::Tree,
    ) {
        if Some(window) != self.window {
            return;
        }

        cx.action(window, PING).enabled(true).active(false);
        cx.action(window, DISABLED_ACTION)
            .enabled(false)
            .active(false);
        cx.action(window, RUN_TASK)
            .enabled(self.workspace_ready)
            .active(false);
        cx.action(window, TOGGLE_PREVIEW)
            .enabled(true)
            .active(self.preview_enabled);
        cx.action(window, action::SELECT_ALL)
            .enabled(false)
            .active(false);
        // TODO(framework): command widgets currently emit Payload::None.
        // INSERT_TEXT should become enabled here only after a payload-producing
        // text field or command runner can bind the payload declaratively.
        cx.action(window, action::INSERT_TEXT)
            .enabled(false)
            .active(false);

        let theme = Theme::default_dark();
        let command_subject = cx.command_subject(window);
        let subject_name = context_name(&command_subject);
        let focused_name = cx
            .focused(window)
            .as_ref()
            .map(path_name)
            .unwrap_or_else(|| "none".to_owned());

        if is_outside_command_scope(command_subject.scope()) && subject_name != "window" {
            self.scope_capture_hint = Some(subject_name.clone());
        }

        let model = ViewModel {
            workspace_ready: self.workspace_ready,
            preview_enabled: self.preview_enabled,
            ping_count: self.ping_count,
            run_count: self.run_count,
            select_count: self.select_count,
            focused_name,
            subject_name,
            last_select: last_select_text(&self.last_select_target, &self.last_select_origin),
            capture_name: self
                .scope_capture_hint
                .as_deref()
                .unwrap_or("none")
                .to_owned(),
            document_is_subject: matches_path(command_subject.scope(), &document_path()),
            local_is_subject: matches_path(command_subject.scope(), &local_field_path()),
            document_scroll: self.document_scroll,
            grid_scroll: self.grid_scroll,
        };

        tree.set_root(root_view(&theme, &model));
        tree.clear_popups();

        // TODO(framework): Popup placement is still authored as an absolute rect.
        // A later popup API should derive this from anchors and viewport constraints.
        tree.push_popup(widget::Popup::new(
            Rect::new(point::logical(570.0, 92.0), area::logical(292.0, 186.0)),
            popup_panel(&theme, &model),
        ));
    }
}

struct ViewModel {
    workspace_ready: bool,
    preview_enabled: bool,
    ping_count: u32,
    run_count: u32,
    select_count: u32,
    focused_name: String,
    subject_name: String,
    last_select: String,
    capture_name: String,
    document_is_subject: bool,
    local_is_subject: bool,
    document_scroll: f32,
    grid_scroll: point::Logical,
}

fn root_view(theme: &Theme, model: &ViewModel) -> ui::Node {
    ui::Node::container(ROOT, layout::Axis::Vertical)
        .with_background(theme.surfaces().app())
        .with_child(widget::menu_bar_with_theme(MENU_BAR, app_menu(), theme))
        .with_child(
            ui::Node::container(CONTENT, layout::Axis::Horizontal)
                .with_size(layout::Size::Fill, layout::Size::Fill)
                .with_padding(layout::Insets::splat(theme.density().app_padding()))
                .with_gap(theme.density().app_padding())
                .with_child(left_column(theme, model))
                .with_child(center_column(theme, model))
                .with_child(right_column(theme, model)),
        )
}

fn left_column(theme: &Theme, model: &ViewModel) -> ui::Node {
    ui::Node::container(LEFT_COLUMN, layout::Axis::Vertical)
        .with_size(layout::Size::Fixed(220.0), layout::Size::Fill)
        .with_gap(theme.density().app_padding())
        .with_child(status_section(theme, model))
        .with_child(action_section(theme, model))
        .with_child(popup_section(theme))
}

fn center_column(theme: &Theme, model: &ViewModel) -> ui::Node {
    ui::Node::container(CENTER_COLUMN, layout::Axis::Vertical)
        .with_size(layout::Size::Fill, layout::Size::Fill)
        .with_gap(theme.density().app_padding())
        .with_child(command_section(theme, model))
        .with_child(layout_section(theme))
        .with_child(scroll_section(theme, model))
}

fn right_column(theme: &Theme, model: &ViewModel) -> ui::Node {
    ui::Node::container(RIGHT_COLUMN, layout::Axis::Vertical)
        .with_size(layout::Size::Fixed(210.0), layout::Size::Fill)
        .with_gap(theme.density().app_padding())
        .with_child(text_section(theme))
        .with_child(state_section(theme, model))
}

fn status_section(theme: &Theme, model: &ViewModel) -> ui::Node {
    section_panel(STATUS_SECTION, "Runtime", theme, layout::Size::Fixed(112.0)).with_child(
        info_panel(
            STATUS_TEXT,
            if model.workspace_ready {
                format!(
                    "ready\nfocus: {}\nsubject: {}",
                    model.focused_name, model.subject_name
                )
            } else {
                "loading workspace...\nfocus: none\nsubject: window".to_owned()
            },
            theme,
        ),
    )
}

fn action_section(theme: &Theme, _model: &ViewModel) -> ui::Node {
    section_panel(
        ACTION_SECTION,
        "Actions and State",
        theme,
        layout::Size::Fixed(188.0),
    )
    .with_child(
        widget::labeled_button_with_theme(DEFAULT_BUTTON, PING, "Default action", theme).with_size(
            layout::Size::Fill,
            layout::Size::Fixed(theme.density().control_height()),
        ),
    )
    .with_child(
        widget::labeled_button_with_theme(
            DISABLED_BUTTON,
            DISABLED_ACTION,
            "Disabled action",
            theme,
        )
        .with_size(
            layout::Size::Fill,
            layout::Size::Fixed(theme.density().control_height()),
        ),
    )
    .with_child(
        widget::labeled_button_with_theme(RUN_BUTTON, RUN_TASK, "Async task", theme).with_size(
            layout::Size::Fill,
            layout::Size::Fixed(theme.density().control_height()),
        ),
    )
    .with_child(
        ui::Node::container(ui::Id::new("toggle_row"), layout::Axis::Horizontal)
            .with_size(
                layout::Size::Fill,
                layout::Size::Fixed(theme.density().control_height()),
            )
            .with_gap(8.0)
            .with_child(
                widget::labeled_button_with_theme(
                    PREVIEW_BUTTON,
                    TOGGLE_PREVIEW,
                    "Active toggle",
                    theme,
                )
                .with_size(layout::Size::Fill, layout::Size::Fill),
            )
            .with_child(
                widget::icon_button_with_theme(
                    ICON_PREVIEW_BUTTON,
                    TOGGLE_PREVIEW,
                    Icon::phosphor(icon::Id::new("eye")),
                    theme,
                )
                .with_size(layout::Size::Fixed(42.0), layout::Size::Fill),
            ),
    )
}

fn popup_section(theme: &Theme) -> ui::Node {
    section_panel(POPUP_SECTION, "Floating Glass", theme, layout::Size::Fill)
        .with_child(info_panel(
            POPUP_NOTE,
            "A command-scope popup is injected over the content.\nIt keeps the captured subject while local focus moves inside it.",
            theme,
        ))
}

fn command_section(theme: &Theme, model: &ViewModel) -> ui::Node {
    let mut document = widget::panel_with_theme(DOCUMENT_PANEL, theme)
        .with_size(layout::Size::Fill, layout::Size::Fixed(58.0))
        .with_interactivity(
            ui::Interactivity::NONE
                .with_hit_test(true)
                .with_focusable(true),
        )
        .with_responder_binding(
            action::Binding::new(action::SELECT_ALL).enabled(model.workspace_ready),
        )
        .with_label(document(
            format!("Document responder | {}", model.last_select),
            text::document::Align::Center,
            theme.text().body_size(),
            theme.text().primary(),
        ));

    if model.document_is_subject {
        document = document.with_stroke(subject_stroke(theme));
    }

    section_panel(
        COMMAND_SECTION,
        "Command Subject",
        theme,
        layout::Size::Fixed(146.0),
    )
    .with_child(document)
    .with_child(
        ui::Node::container(COMMAND_ROW, layout::Axis::Horizontal)
            .with_size(
                layout::Size::Fill,
                layout::Size::Fixed(theme.density().control_height()),
            )
            .with_gap(8.0)
            .with_child(
                widget::labeled_button_with_theme(
                    SELECT_BUTTON,
                    action::SELECT_ALL,
                    "Select current",
                    theme,
                )
                .with_command_subject(ui::CommandSubject::Current)
                .with_size(layout::Size::Fill, layout::Size::Fill),
            )
            .with_child(
                widget::labeled_button_with_theme(
                    WINDOW_SELECT_BUTTON,
                    action::SELECT_ALL,
                    "Window target",
                    theme,
                )
                .with_command_subject(ui::CommandSubject::Window)
                .with_size(layout::Size::Fill, layout::Size::Fill),
            ),
    )
}

fn layout_section(theme: &Theme) -> ui::Node {
    section_panel(
        LAYOUT_SECTION,
        "Layout Pressure",
        theme,
        layout::Size::Fixed(128.0),
    )
    .with_child(
        ui::Node::container(ui::Id::new("fit_fill_row"), layout::Axis::Horizontal)
            .with_size(
                layout::Size::Fill,
                layout::Size::Fixed(theme.density().control_height()),
            )
            .with_gap(8.0)
            .with_child(
                widget::labeled_button_with_theme(FIT_BUTTON, PING, "Fit", theme)
                    .with_size(layout::Size::Fit, layout::Size::Fill),
            )
            .with_child(
                widget::labeled_button_with_theme(FILL_BUTTON, PING, "Fill", theme)
                    .with_size(layout::Size::Fill, layout::Size::Fill),
            )
            .with_child(
                widget::labeled_button_with_theme(FIXED_BUTTON, PING, "Fixed", theme)
                    .with_size(layout::Size::Fixed(92.0), layout::Size::Fill),
            ),
    )
    .with_child(
        ui::Node::container(ALIGN_ROW, layout::Axis::Horizontal)
            .with_size(
                layout::Size::Fill,
                layout::Size::Fixed(theme.density().control_height()),
            )
            .with_cross_align(layout::Align::Center)
            .with_gap(8.0)
            .with_child(
                widget::labeled_button_with_theme(ALIGN_START, PING, "Start", theme)
                    .with_size(layout::Size::Fit, layout::Size::Fill),
            )
            .with_child(
                widget::labeled_button_with_theme(ALIGN_CENTER, PING, "Center", theme)
                    .with_size(layout::Size::Fill, layout::Size::Fill),
            )
            .with_child(
                widget::labeled_button_with_theme(ALIGN_END, PING, "End", theme)
                    .with_size(layout::Size::Fit, layout::Size::Fill),
            ),
    )
}

fn scroll_section(theme: &Theme, model: &ViewModel) -> ui::Node {
    section_panel(SCROLL_SECTION, "Scroll Widgets", theme, layout::Size::Fill)
        .with_child(document_scroll(theme, model.document_scroll))
        .with_child(both_axis_scroll(theme, model.grid_scroll))
}

fn state_section(theme: &Theme, model: &ViewModel) -> ui::Node {
    section_panel(STATE_SECTION, "State Projection", theme, layout::Size::Fill)
        .with_child(info_panel(
            STATE_READY,
            format!(
                "workspace: {}\npings: {}",
                if model.workspace_ready {
                    "ready"
                } else {
                    "loading"
                },
                model.ping_count
            ),
            theme,
        ))
        .with_child(info_panel(
            STATE_SUBJECT,
            format!(
                "preview: {}\nsubject: {}",
                if model.preview_enabled { "on" } else { "off" },
                model.subject_name
            ),
            theme,
        ))
        .with_child(info_panel(
            STATE_LAST,
            format!(
                "runs: {}\nselections: {}",
                model.run_count, model.select_count
            ),
            theme,
        ))
        .with_child(info_panel(STATE_FOOTER, model.last_select.clone(), theme))
}

fn text_section(theme: &Theme) -> ui::Node {
    section_panel(
        TEXT_SECTION,
        "Text Widgets",
        theme,
        layout::Size::Fixed(128.0),
    )
    .with_child(
        widget::text_with_theme(TEXT_LINE, "fit-sized text()", theme)
            .with_label_color(theme.text().secondary()),
    )
    .with_child(
        widget::paragraph_with_theme(
            TEXT_PARAGRAPH,
            "paragraph() measures through text::layout::Engine and wraps as layout constraints change.",
            theme,
        )
        .with_label_color(theme.text().secondary()),
    )
}

fn document_scroll(theme: &Theme, offset: f32) -> ui::Node {
    let mut scroll = widget::scroll_view_with_theme(DOCUMENT_SCROLL, theme)
        .with_scroll_offset(point::logical(0.0, offset))
        .with_background(theme.surfaces().panel())
        .with_rounding(theme.roundings().panel())
        .with_gap(SCROLL_GAP)
        .with_padding(layout::Insets::splat(SCROLL_PADDING))
        .with_size(layout::Size::Fill, layout::Size::Fixed(166.0));

    for (index, id) in SCROLL_ROWS.iter().copied().enumerate() {
        scroll.push_child(
            widget::panel_with_theme(id, theme)
                .with_size(layout::Size::Fill, layout::Size::Fixed(SCROLL_ROW_HEIGHT))
                .with_label(document(
                    format!("Scrollable row {}", index + 1),
                    text::document::Align::Center,
                    theme.text().body_size(),
                    theme.text().primary(),
                )),
        );
    }

    scroll
}

fn both_axis_scroll(theme: &Theme, offset: point::Logical) -> ui::Node {
    widget::scroll_view_with_theme(BOTH_AXIS_SCROLL, theme)
        .with_scroll_bars(widget::scroll::Bars::both())
        .with_scroll_offset(offset)
        .with_background(theme.surfaces().panel())
        .with_rounding(theme.roundings().panel())
        .with_gap(SCROLL_GAP)
        .with_padding(layout::Insets::splat(SCROLL_PADDING))
        .with_size(layout::Size::Fill, layout::Size::Fixed(112.0))
        .with_child(both_axis_content(theme))
}

fn both_axis_content(theme: &Theme) -> ui::Node {
    const ROWS: [ui::Id; 4] = [
        ui::Id::new("grid_row_0"),
        ui::Id::new("grid_row_1"),
        ui::Id::new("grid_row_2"),
        ui::Id::new("grid_row_3"),
    ];

    let mut content = ui::Node::container(BOTH_AXIS_CONTENT, layout::Axis::Vertical)
        .with_size(layout::Size::Fixed(880.0), layout::Size::Fixed(220.0))
        .with_padding(layout::Insets::splat(8.0))
        .with_gap(6.0)
        .with_background(theme.surfaces().canvas());

    for (row_index, row_id) in ROWS.iter().copied().enumerate() {
        let mut row = ui::Node::container(row_id, layout::Axis::Horizontal)
            .with_size(layout::Size::Fill, layout::Size::Fixed(42.0))
            .with_gap(6.0);

        for column_index in 0..6 {
            row = row.with_child(
                widget::labeled_button_with_theme(
                    GRID_CELLS[row_index][column_index],
                    PING,
                    format!("{row_index}-{column_index}"),
                    theme,
                )
                .with_size(layout::Size::Fixed(132.0), layout::Size::Fill),
            );
        }

        content = content.with_child(row);
    }

    content
}

fn popup_panel(theme: &Theme, model: &ViewModel) -> ui::Node {
    let mut local_field = widget::panel_with_theme(LOCAL_FIELD, theme)
        .with_size(
            layout::Size::Fill,
            layout::Size::Fixed(theme.density().control_height()),
        )
        .with_interactivity(
            ui::Interactivity::NONE
                .with_hit_test(true)
                .with_focusable(true),
        )
        .with_responder_binding(
            action::Binding::new(action::SELECT_ALL).enabled(model.workspace_ready),
        )
        .with_label(document(
            if model.local_is_subject {
                "Local responder | current"
            } else {
                "Local responder"
            },
            text::document::Align::Center,
            theme.text().body_size(),
            theme.text().primary(),
        ));

    if model.local_is_subject {
        local_field = local_field.with_stroke(subject_stroke(theme));
    }

    widget::floating_panel_with_theme(COMMAND_SCOPE_PANEL, theme)
        .with_size(layout::Size::Fill, layout::Size::Fit)
        .with_command_scope()
        .with_child(info_panel(
            SCOPE_STATUS,
            format!("captured: {}", model.capture_name),
            theme,
        ))
        .with_child(local_field)
        .with_child(
            widget::labeled_button_with_theme(
                LOCAL_SELECT_BUTTON,
                action::SELECT_ALL,
                "Select current",
                theme,
            )
            .with_command_subject(ui::CommandSubject::Current)
            .with_size(
                layout::Size::Fill,
                layout::Size::Fixed(theme.density().control_height()),
            ),
        )
        .with_child(
            widget::labeled_button_with_theme(
                CAPTURED_SELECT_BUTTON,
                action::SELECT_ALL,
                "Select captured",
                theme,
            )
            .with_command_subject(ui::CommandSubject::Captured)
            .with_size(
                layout::Size::Fill,
                layout::Size::Fixed(theme.density().control_height()),
            ),
        )
}

fn section_panel(
    id: ui::Id,
    title: impl Into<String>,
    theme: &Theme,
    height: layout::Size,
) -> ui::Node {
    widget::panel_with_theme(id, theme)
        .with_size(layout::Size::Fill, height)
        .with_padding(layout::Insets::splat(theme.density().panel_padding()))
        .with_gap(8.0)
        .with_child(section_header(header_id(id), title, theme))
}

fn section_header(id: ui::Id, title: impl Into<String>, theme: &Theme) -> ui::Node {
    ui::Node::leaf(id)
        .with_label(document(
            title,
            text::document::Align::Start,
            theme.text().label_size(),
            theme.text().secondary(),
        ))
        .with_size(
            layout::Size::Fill,
            layout::Size::Fixed(theme.density().label_height()),
        )
}

fn info_panel(id: ui::Id, label: impl Into<String>, theme: &Theme) -> ui::Node {
    widget::panel_with_theme(id, theme)
        .with_size(layout::Size::Fill, layout::Size::Fill)
        .with_label(document(
            label,
            text::document::Align::Center,
            theme.text().body_size(),
            theme.text().secondary(),
        ))
}

fn app_menu() -> widget::menu::Bar {
    widget::menu::Bar::new()
        .menu(
            widget::Menu::new(FILE_MENU, "File").section(
                widget::menu::Section::new()
                    .item(widget::menu::Item::new(PING))
                    .item(widget::menu::Item::new(RUN_TASK))
                    .separator()
                    .item(widget::menu::Item::new(DISABLED_ACTION)),
            ),
        )
        .menu(
            widget::Menu::new(EDIT_MENU, "Edit").section(
                widget::menu::Section::new()
                    .item(widget::menu::Item::new(action::SELECT_ALL))
                    .separator()
                    .item(widget::menu::Item::new(action::INSERT_TEXT)),
            ),
        )
        .menu(
            widget::Menu::new(VIEW_MENU, "View").section(
                widget::menu::Section::new()
                    .item(widget::menu::Item::new(TOGGLE_PREVIEW))
                    .separator()
                    .submenu(widget::Menu::new(PANELS_MENU, "Panels").section(
                        widget::menu::Section::new().item(
                            widget::menu::Item::new(TOGGLE_PREVIEW).with_label("Preview Panel"),
                        ),
                    )),
            ),
        )
}

fn header_id(id: ui::Id) -> ui::Id {
    if id == ACTION_SECTION {
        ACTION_HEADER
    } else if id == COMMAND_SECTION {
        COMMAND_HEADER
    } else if id == LAYOUT_SECTION {
        LAYOUT_HEADER
    } else if id == SCROLL_SECTION {
        SCROLL_HEADER
    } else if id == STATE_SECTION {
        STATE_HEADER
    } else if id == POPUP_SECTION {
        POPUP_HEADER
    } else if id == TEXT_SECTION {
        TEXT_HEADER
    } else {
        ui::Id::new("section_header")
    }
}

fn document(
    label: impl Into<String>,
    align: text::document::Align,
    size: f32,
    color: paint::Color,
) -> text::document::Document {
    let mut block = text::document::Block::new(align);
    block.push_run(text::document::Run::new(
        label,
        text::document::Style::default()
            .with_size(size)
            .with_color(color),
    ));

    text::document::Document::from_block(block)
}

fn subject_stroke(theme: &Theme) -> paint::Stroke {
    paint::Stroke {
        brush: paint::Brush::solid(theme.palette().accent()),
        width: 1.0,
    }
}

fn last_select_text(target: &Option<String>, origin: &Option<String>) -> String {
    match (target, origin) {
        (Some(target), Some(origin)) => format!("last select: {target} from {origin}"),
        (Some(target), None) => format!("last select: {target}"),
        _ => "last select: none".to_owned(),
    }
}

fn is_outside_command_scope(scope: &action::Scope) -> bool {
    match scope {
        action::Scope::Path(path) => !path.is_descendant_of(&command_scope_path()),
        action::Scope::Window => false,
    }
}

fn matches_path(scope: &action::Scope, expected: &ui::Path) -> bool {
    matches!(scope, action::Scope::Path(path) if path == expected)
}

fn context_name(context: &action::Context) -> String {
    match context.scope() {
        action::Scope::Path(path) => path_name(path),
        action::Scope::Window => "window".to_owned(),
    }
}

fn path_name(path: &ui::Path) -> String {
    if path == &document_path() {
        "document".to_owned()
    } else if path == &local_field_path() {
        "local responder".to_owned()
    } else {
        path.leaf()
            .map(ui::Id::as_str)
            .unwrap_or("unknown")
            .to_owned()
    }
}

// TODO(framework): Stable command/scroll APIs still require hand-authored tree paths.
// A later widget-reference layer should let examples avoid mirroring tree shape here.
fn document_path() -> ui::Path {
    ui::Path::new([
        ROOT,
        CONTENT,
        CENTER_COLUMN,
        COMMAND_SECTION,
        DOCUMENT_PANEL,
    ])
}

fn document_scroll_path() -> ui::Path {
    ui::Path::new([
        ROOT,
        CONTENT,
        CENTER_COLUMN,
        SCROLL_SECTION,
        DOCUMENT_SCROLL,
    ])
}

fn grid_scroll_path() -> ui::Path {
    ui::Path::new([
        ROOT,
        CONTENT,
        CENTER_COLUMN,
        SCROLL_SECTION,
        BOTH_AXIS_SCROLL,
    ])
}

fn command_scope_path() -> ui::Path {
    ui::Path::new([ROOT, COMMAND_SCOPE_PANEL])
}

fn local_field_path() -> ui::Path {
    ui::Path::new([ROOT, COMMAND_SCOPE_PANEL, LOCAL_FIELD])
}
