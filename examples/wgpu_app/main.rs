use std::time::Duration;

use wgpu_l3::{
    Event, Icon, Task, Theme, app, command,
    geometry::{Rect, area, point},
    icon, paint, text,
    ui::{self, layout},
    widget, window,
};

wgpu_l3::command!(Ping {
    name: "ping",
    display: "Ping",
});
wgpu_l3::command!(RunTask {
    name: "run_task",
    display: "Run Task",
});
wgpu_l3::command!(TogglePreview {
    name: "toggle_preview",
    display: "Toggle Preview",
});
wgpu_l3::command!(SelectTarget {
    name: "select_target",
    display: "Select Target",
});
wgpu_l3::command!(DisabledCommand {
    name: "disabled_command",
    display: "Unavailable",
});

const SCROLL_ROW_HEIGHT: f32 = 24.0;
const SCROLL_GAP: f32 = 3.0;
const SCROLL_PADDING: f32 = 4.0;

fn main() -> app::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    app::run(App::default())
}

struct App {
    window: Option<window::Id>,
    sender: Option<app::Sender<AppEvent>>,
    workspace_ready: bool,
    preview_enabled: bool,
    ping_count: u32,
    run_count: u32,
    select_count: u32,
    last_select_target: Option<String>,
    last_select_origin: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            sender: None,
            workspace_ready: false,
            preview_enabled: false,
            ping_count: 0,
            run_count: 0,
            select_count: 0,
            last_select_target: None,
            last_select_origin: None,
        }
    }
}

enum AppEvent {
    WorkspaceReady,
    RunTaskFinished,
}

impl command::Target<Ping> for App {
    fn invoke(
        &mut self,
        _args: (),
        _invocation: command::call::Invocation<Ping>,
    ) -> command::Response<()> {
        self.ping_count += 1;
        command::Response::none()
    }
}

impl command::Target<DisabledCommand> for App {
    fn state(&self, _context: &command::call::Context) -> command::State {
        command::State::unavailable()
    }

    fn invoke(
        &mut self,
        _args: (),
        _invocation: command::call::Invocation<DisabledCommand>,
    ) -> command::Response<()> {
        command::Response::none()
    }
}

impl command::Target<RunTask> for App {
    fn state(&self, _context: &command::call::Context) -> command::State {
        command::State::available_if(self.workspace_ready)
    }

    fn invoke(
        &mut self,
        _args: (),
        _invocation: command::call::Invocation<RunTask>,
    ) -> command::Response<()> {
        let Some(sender) = self.sender.clone() else {
            return command::Response::none();
        };

        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(850));
            let _ = sender.emit(AppEvent::RunTaskFinished);
        });

        command::Response::none()
    }
}

impl command::Target<TogglePreview> for App {
    fn state(&self, _context: &command::call::Context) -> command::State {
        command::State::active_if(self.preview_enabled)
    }

    fn invoke(
        &mut self,
        _args: (),
        _invocation: command::call::Invocation<TogglePreview>,
    ) -> command::Response<()> {
        self.preview_enabled = !self.preview_enabled;
        command::Response::none()
    }
}

impl command::Target<SelectTarget> for App {
    fn state(&self, _context: &command::call::Context) -> command::State {
        command::State::unavailable()
    }

    fn invoke(
        &mut self,
        _args: (),
        invocation: command::call::Invocation<SelectTarget>,
    ) -> command::Response<()> {
        let target = invocation.context().clone();
        let origin = invocation.origin().cloned();

        log::debug!("select target target={target:?} origin={origin:?}");
        self.last_select_target = Some(context_name(&target));
        self.last_select_origin = origin.as_ref().map(path_name);
        self.select_count += 1;

        command::Response::none()
    }
}

impl app::Application for App {
    type Event = AppEvent;

    fn started(&mut self, cx: &mut app::Context<'_, Self::Event>) {
        self.sender = Some(cx.sender());
        cx.commands(|commands| {
            commands
                .define::<Ping, App>(|command| command)
                .define::<DisabledCommand, App>(|command| command)
                .define::<RunTask, App>(|command| command)
                .define::<TogglePreview, App>(|command| command)
                .define::<SelectTarget, App>(|command| {
                    command.shortcut(command::shortcut::Shortcut::ctrl('a'))
                });
        });

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
                event: ui::Event::ScrollRequested { .. },
            } if event_window == window => {
                cx.request_redraw(window);
            }
            Event::Ui { .. } => {}
            Event::App(AppEvent::WorkspaceReady) => {
                self.workspace_ready = true;
                cx.request_redraw(window);
            }
            Event::App(AppEvent::RunTaskFinished) => {
                self.run_count += 1;
                cx.request_redraw(window);
            }
        }
    }

    fn command_targets(&mut self, commands: &mut app::CommandDispatch<'_, Self::Event>) {
        commands.target(self);
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

        let theme = Theme::default_dark();
        let command_subject = cx.command_subject(window);
        let subject_name = context_name(&command_subject);
        let focused_name = cx
            .focused(window)
            .as_ref()
            .map(path_name)
            .unwrap_or_else(|| "none".to_owned());

        let model = ViewModel {
            workspace_ready: self.workspace_ready,
            preview_enabled: self.preview_enabled,
            ping_count: self.ping_count,
            run_count: self.run_count,
            select_count: self.select_count,
            focused_name,
            subject_name,
            last_select: last_select_text(&self.last_select_target, &self.last_select_origin),
            capture_name: context_name(&command_subject),
        };

        tree.set_root(root_view(&theme, &model));
        tree.clear_popups();

        // TODO(framework): Popup placement is still authored as an absolute rect.
        // A later popup API should derive this from anchors and viewport constraints.
        tree.push_popup(ui::Popup::new(
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
}

fn root_view(theme: &Theme, model: &ViewModel) -> ui::Node {
    ui::Node::container(layout::Axis::Vertical)
        .with_background(theme.surfaces().app())
        .child(widget::menu_bar_with_theme(app_menu(), theme))
        .child(
            ui::Node::container(layout::Axis::Horizontal)
                .size(layout::Size::fill(), layout::Size::fill())
                .padding(theme.density().app_padding())
                .gap(theme.density().app_padding())
                .with_children([
                    left_column(theme, model),
                    center_column(theme, model),
                    right_column(theme, model),
                ]),
        )
}

fn left_column(theme: &Theme, model: &ViewModel) -> ui::Node {
    ui::Node::container(layout::Axis::Vertical)
        .size(layout::Size::fixed(220.0), layout::Size::fill())
        .gap(theme.density().app_padding())
        .with_children([
            status_section(theme, model),
            invocation_section(theme, model),
            popup_section(theme),
        ])
}

fn center_column(theme: &Theme, model: &ViewModel) -> ui::Node {
    ui::Node::container(layout::Axis::Vertical)
        .size(layout::Size::fill(), layout::Size::fill())
        .gap(theme.density().app_padding())
        .with_children([
            command_section(theme, model),
            layout_section(theme),
            scroll_section(theme),
        ])
}

fn right_column(theme: &Theme, model: &ViewModel) -> ui::Node {
    ui::Node::container(layout::Axis::Vertical)
        .size(layout::Size::fixed(210.0), layout::Size::fill())
        .gap(theme.density().app_padding())
        .with_children([text_section(theme), state_section(theme, model)])
}

fn status_section(theme: &Theme, model: &ViewModel) -> ui::Node {
    section_panel("Runtime", theme, layout::Size::Fixed(112.0)).with_child(info_panel(
        if model.workspace_ready {
            format!(
                "ready\nfocus: {}\nsubject: {}",
                model.focused_name, model.subject_name
            )
        } else {
            "loading workspace...\nfocus: none\nsubject: window".to_owned()
        },
        theme,
    ))
}

fn invocation_section(theme: &Theme, _model: &ViewModel) -> ui::Node {
    section_panel("Commands and State", theme, layout::Size::Fixed(188.0))
        .with_child(
            widget::labeled_button_with_theme::<Ping, App>("Default command", theme).with_size(
                layout::Size::Fill,
                layout::Size::Fixed(theme.density().control_height()),
            ),
        )
        .with_child(
            widget::labeled_button_with_theme::<DisabledCommand, App>("Disabled command", theme)
                .with_size(
                    layout::Size::Fill,
                    layout::Size::Fixed(theme.density().control_height()),
                ),
        )
        .with_child(
            widget::labeled_button_with_theme::<RunTask, App>("Async task", theme).with_size(
                layout::Size::Fill,
                layout::Size::Fixed(theme.density().control_height()),
            ),
        )
        .with_child(
            ui::Node::container(layout::Axis::Horizontal)
                .with_size(
                    layout::Size::Fill,
                    layout::Size::Fixed(theme.density().control_height()),
                )
                .with_gap(8.0)
                .with_child(
                    widget::labeled_button_with_theme::<TogglePreview, App>("Active toggle", theme)
                        .with_size(layout::Size::Fill, layout::Size::Fill),
                )
                .with_child(
                    widget::icon_button_with_theme::<TogglePreview, App>(
                        Icon::phosphor(icon::Id::new("eye")),
                        theme,
                    )
                    .with_size(layout::Size::Fixed(42.0), layout::Size::Fill),
                ),
        )
}

fn popup_section(theme: &Theme) -> ui::Node {
    section_panel("Floating Glass", theme, layout::Size::Fill).with_child(info_panel(
        "A command-scope popup is injected over the content.\nIt keeps the captured subject while local focus moves inside it.",
        theme,
    ))
}

fn command_section(theme: &Theme, model: &ViewModel) -> ui::Node {
    let document = widget::panel_with_theme(theme)
        .with_size(layout::Size::Fill, layout::Size::Fixed(58.0))
        .with_interactivity(
            ui::Interactivity::NONE
                .with_hit_test(true)
                .with_focusable(true),
        )
        .with_responder_binding(
            command::binding::Binding::of::<SelectTarget>()
                .available(model.workspace_ready)
                .action(),
        )
        .with_label(document(
            format!("Document responder | {}", model.last_select),
            text::document::Align::Center,
            theme.text().body_size(),
            theme.text().primary(),
        ));

    section_panel("Command Subject", theme, layout::Size::Fixed(146.0))
        .with_child(document)
        .with_child(
            ui::Node::container(layout::Axis::Horizontal)
                .with_size(
                    layout::Size::Fill,
                    layout::Size::Fixed(theme.density().control_height()),
                )
                .with_gap(8.0)
                .with_child(
                    widget::labeled_button_with_theme::<SelectTarget, App>("Select current", theme)
                        .with_action_subject(ui::ActionSubject::Current)
                        .with_size(layout::Size::Fill, layout::Size::Fill),
                )
                .with_child(
                    widget::labeled_button_with_theme::<SelectTarget, App>("Window target", theme)
                        .with_action_subject(ui::ActionSubject::Window)
                        .with_size(layout::Size::Fill, layout::Size::Fill),
                ),
        )
}

fn layout_section(theme: &Theme) -> ui::Node {
    section_panel("Layout Pressure", theme, layout::Size::Fixed(128.0))
        .with_child(
            ui::Node::container(layout::Axis::Horizontal)
                .with_size(
                    layout::Size::Fill,
                    layout::Size::Fixed(theme.density().control_height()),
                )
                .with_gap(8.0)
                .with_child(
                    widget::labeled_button_with_theme::<Ping, App>("Fit", theme)
                        .with_size(layout::Size::Fit, layout::Size::Fill),
                )
                .with_child(
                    widget::labeled_button_with_theme::<Ping, App>("Fill", theme)
                        .with_size(layout::Size::Fill, layout::Size::Fill),
                )
                .with_child(
                    widget::labeled_button_with_theme::<Ping, App>("Fixed", theme)
                        .with_size(layout::Size::Fixed(92.0), layout::Size::Fill),
                ),
        )
        .with_child(
            ui::Node::container(layout::Axis::Horizontal)
                .with_size(
                    layout::Size::Fill,
                    layout::Size::Fixed(theme.density().control_height()),
                )
                .with_cross_align(layout::Align::Center)
                .with_gap(8.0)
                .with_child(
                    widget::labeled_button_with_theme::<Ping, App>("Start", theme)
                        .with_size(layout::Size::Fit, layout::Size::Fill),
                )
                .with_child(
                    widget::labeled_button_with_theme::<Ping, App>("Center", theme)
                        .with_size(layout::Size::Fill, layout::Size::Fill),
                )
                .with_child(
                    widget::labeled_button_with_theme::<Ping, App>("End", theme)
                        .with_size(layout::Size::Fit, layout::Size::Fill),
                ),
        )
}

fn scroll_section(theme: &Theme) -> ui::Node {
    section_panel("Scroll Widgets", theme, layout::Size::Fill)
        .with_child(document_scroll(theme))
        .with_child(both_axis_scroll(theme))
}

fn state_section(theme: &Theme, model: &ViewModel) -> ui::Node {
    section_panel("State Projection", theme, layout::Size::Fill)
        .with_child(info_panel(
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
            format!(
                "preview: {}\nsubject: {}",
                if model.preview_enabled { "on" } else { "off" },
                model.subject_name
            ),
            theme,
        ))
        .with_child(info_panel(
            format!(
                "runs: {}\nselections: {}",
                model.run_count, model.select_count
            ),
            theme,
        ))
        .with_child(info_panel(model.last_select.clone(), theme))
}

fn text_section(theme: &Theme) -> ui::Node {
    section_panel(
        "Text Widgets",
        theme,
        layout::Size::Fixed(128.0),
    )
    .with_child(
        widget::text_with_theme("fit-sized text()", theme)

            .with_label_color(theme.text().secondary()),
    )
    .with_child(
        widget::paragraph_with_theme(
            "paragraph() measures through text::layout::Engine and wraps as layout constraints change.",
            theme,
        )

        .with_label_color(theme.text().secondary()),
    )
}

fn document_scroll(theme: &Theme) -> ui::Node {
    let mut scroll = widget::scroll_view_with_theme(theme)
        .with_background(theme.surfaces().panel())
        .with_rounding(theme.roundings().panel())
        .with_gap(SCROLL_GAP)
        .with_padding(layout::Insets::splat(SCROLL_PADDING))
        .with_size(layout::Size::Fill, layout::Size::Fixed(166.0));

    for index in 0..14 {
        scroll.push_child(
            widget::panel_with_theme(theme)
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

fn both_axis_scroll(theme: &Theme) -> ui::Node {
    widget::scroll_view_with_theme(theme)
        .with_scroll_axes(widget::scroll::Axes::both())
        .with_scroll_bars(widget::scroll::Bars::both())
        .with_background(theme.surfaces().panel())
        .with_rounding(theme.roundings().panel())
        .with_gap(SCROLL_GAP)
        .with_padding(layout::Insets::splat(SCROLL_PADDING))
        .with_size(layout::Size::Fill, layout::Size::Fixed(112.0))
        .with_child(both_axis_content(theme))
}

fn both_axis_content(theme: &Theme) -> ui::Node {
    let mut content = ui::Node::container(layout::Axis::Vertical)
        .with_size(layout::Size::Fixed(880.0), layout::Size::Fixed(220.0))
        .with_padding(layout::Insets::splat(8.0))
        .with_gap(6.0)
        .with_background(theme.surfaces().canvas());

    for row_index in 0..4 {
        let mut row = ui::Node::container(layout::Axis::Horizontal)
            .with_size(layout::Size::Fill, layout::Size::Fixed(42.0))
            .with_gap(6.0);

        for column_index in 0..6 {
            row = row.with_child(
                widget::labeled_button_with_theme::<Ping, App>(
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
    let local_field = widget::panel_with_theme(theme)
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
            command::binding::Binding::of::<SelectTarget>()
                .available(model.workspace_ready)
                .action(),
        )
        .with_label(document(
            "Local responder",
            text::document::Align::Center,
            theme.text().body_size(),
            theme.text().primary(),
        ));

    widget::floating_panel_with_theme(theme)
        .with_size(layout::Size::Fill, layout::Size::Fit)
        .with_action_scope()
        .with_child(info_panel(
            format!("captured: {}", model.capture_name),
            theme,
        ))
        .with_child(local_field)
        .with_child(
            widget::labeled_button_with_theme::<SelectTarget, App>("Select current", theme)
                .with_action_subject(ui::ActionSubject::Current)
                .with_size(
                    layout::Size::Fill,
                    layout::Size::Fixed(theme.density().control_height()),
                ),
        )
        .with_child(
            widget::labeled_button_with_theme::<SelectTarget, App>("Select captured", theme)
                .with_action_subject(ui::ActionSubject::Captured)
                .with_size(
                    layout::Size::Fill,
                    layout::Size::Fixed(theme.density().control_height()),
                ),
        )
}

fn section_panel(title: impl Into<String>, theme: &Theme, height: layout::Size) -> ui::Node {
    widget::panel_with_theme(theme)
        .with_size(layout::Size::Fill, height)
        .with_padding(layout::Insets::splat(theme.density().panel_padding()))
        .with_gap(8.0)
        .with_child(section_header(title, theme))
}

fn section_header(title: impl Into<String>, theme: &Theme) -> ui::Node {
    ui::Node::leaf()
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

fn info_panel(label: impl Into<String>, theme: &Theme) -> ui::Node {
    widget::panel_with_theme(theme)
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
            widget::Menu::new("File").section(
                widget::menu::Section::new()
                    .item(widget::menu::Item::invokes::<Ping, App>())
                    .item(widget::menu::Item::invokes::<RunTask, App>())
                    .separator()
                    .item(widget::menu::Item::invokes::<DisabledCommand, App>()),
            ),
        )
        .menu(widget::Menu::new("Edit").section(
            widget::menu::Section::new().item(widget::menu::Item::invokes::<SelectTarget, App>()),
        ))
        .menu(
            widget::Menu::new("View").section(
                widget::menu::Section::new()
                    .item(widget::menu::Item::invokes::<TogglePreview, App>())
                    .separator()
                    .submenu(
                        widget::Menu::new("Panels").section(
                            widget::menu::Section::new().item(
                                widget::menu::Item::invokes::<TogglePreview, App>()
                                    .with_label("Preview Panel"),
                            ),
                        ),
                    ),
            ),
        )
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

fn last_select_text(target: &Option<String>, origin: &Option<String>) -> String {
    match (target, origin) {
        (Some(target), Some(origin)) => format!("last select: {target} from {origin}"),
        (Some(target), None) => format!("last select: {target}"),
        _ => "last select: none".to_owned(),
    }
}

fn context_name(context: &command::call::Context) -> String {
    match context.scope() {
        command::call::Scope::Path(path) => path_name(path),
        command::call::Scope::Window => "window".to_owned(),
        command::call::Scope::Current => "current".to_owned(),
        command::call::Scope::Focused => "focused".to_owned(),
        command::call::Scope::Captured => "captured".to_owned(),
    }
}

fn path_name(path: &ui::Path) -> String {
    path.leaf()
        .map(|id| id.to_string())
        .unwrap_or_else(|| "unknown".to_owned())
}
