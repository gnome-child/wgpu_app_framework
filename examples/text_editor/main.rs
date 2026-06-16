use wgpu_l3::{
    Action, Event, Theme, action, app, geometry::area, layout, paint, text, ui, widget, window,
};

const INSERT_SAMPLE: action::Id = action::Id::new("insert_sample_text");

const ROOT: ui::Id = ui::Id::new("root");
const MENU_BAR: ui::Id = ui::Id::new("menu_bar");
const CONTENT: ui::Id = ui::Id::new("content");
const EDITOR_PANEL: ui::Id = ui::Id::new("editor_panel");
const TITLE: ui::Id = ui::Id::new("title");
const FIELD: ui::Id = ui::Id::new("field");
const STATUS: ui::Id = ui::Id::new("status");
const COMMAND_ROW: ui::Id = ui::Id::new("command_row");
const SELECT_BUTTON: ui::Id = ui::Id::new("select_button");
const SAMPLE_BUTTON: ui::Id = ui::Id::new("sample_button");

const FILE_MENU: widget::menu::Id = widget::menu::Id::new("file");
const EDIT_MENU: widget::menu::Id = widget::menu::Id::new("edit");

fn main() -> app::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    app::run(Editor::default())
}

struct Editor {
    window: Option<window::Id>,
    buffer: text::Buffer,
    edit_count: u32,
    last_action: String,
}

enum AppEvent {
    ApplyEdit {
        target: action::Context,
        edit: text::Edit,
        label: &'static str,
    },
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            window: None,
            buffer: text::Buffer::from_text(
                "Click the field, type text, then use Edit > Select All or Ctrl+A.",
            ),
            edit_count: 0,
            last_action: "none".to_owned(),
        }
    }
}

impl app::Application for Editor {
    type Event = AppEvent;

    fn started(&mut self, cx: &mut app::Context<'_, Self::Event>) {
        cx.register_action(
            Action::new(action::SELECT_ALL, "Select All")
                .with_shortcut(action::Shortcut::control('a'))
                .emit(|invocation| AppEvent::ApplyEdit {
                    target: invocation.context().clone(),
                    edit: text::Edit::SelectAll,
                    label: "select all",
                }),
        );
        cx.register_action(
            Action::new(action::CUT, "Cut").with_shortcut(action::Shortcut::control('x')),
        );
        cx.register_action(
            Action::new(action::COPY, "Copy").with_shortcut(action::Shortcut::control('c')),
        );
        cx.register_action(
            Action::new(action::PASTE, "Paste").with_shortcut(action::Shortcut::control('v')),
        );
        cx.register_action(
            Action::new(INSERT_SAMPLE, "Insert Sample Text").emit(|invocation| {
                AppEvent::ApplyEdit {
                    target: invocation.context().clone(),
                    edit: text::Edit::insert(" sample"),
                    label: "insert sample",
                }
            }),
        );

        let theme = Theme::default_dark();
        self.window = Some(cx.open_window(window::Options {
            title: "wgpu_l3 text editor".to_owned(),
            inner_area: area::physical(720, 360),
            canvas_color: theme.surfaces().canvas(),
        }));
    }

    fn event(&mut self, cx: &mut app::Context<'_, Self::Event>, event: Event<Self::Event>) {
        let Some(window) = self.window else {
            return;
        };

        match event {
            Event::Ui {
                window: event_window,
                event: ui::Event::TextEditRequested { target, edit },
            } if event_window == window && target == field_path() => {
                self.apply_edit(edit, "keyboard/pointer");
                cx.request_redraw(window);
            }
            Event::Ui { .. } => {}
            Event::App(AppEvent::ApplyEdit {
                target,
                edit,
                label,
            }) => {
                if matches!(target.scope(), action::Scope::Path(path) if path == &field_path()) {
                    self.apply_edit(edit, label);
                    cx.request_redraw(window);
                }
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

        cx.action(window, action::SELECT_ALL)
            .enabled(false)
            .active(false);
        cx.action(window, action::CUT).enabled(false).active(false);
        cx.action(window, action::COPY).enabled(false).active(false);
        cx.action(window, action::PASTE)
            .enabled(false)
            .active(false);
        cx.action(window, INSERT_SAMPLE)
            .enabled(false)
            .active(false);

        let theme = Theme::default_dark();
        tree.clear_popups();
        tree.set_root(root_view(
            &theme,
            &self.buffer,
            self.edit_count,
            &self.last_action,
        ));
    }
}

impl Editor {
    fn apply_edit(&mut self, edit: text::Edit, label: &'static str) {
        self.buffer.apply(edit);
        self.edit_count += 1;
        self.last_action = label.to_owned();
    }
}

fn root_view(theme: &Theme, buffer: &text::Buffer, edit_count: u32, last_action: &str) -> ui::Node {
    ui::Node::container(ROOT, layout::Axis::Vertical)
        .with_background(theme.surfaces().app())
        .with_child(widget::menu_bar_with_theme(MENU_BAR, edit_menu(), theme))
        .with_child(
            ui::Node::container(CONTENT, layout::Axis::Vertical)
                .with_size(layout::Size::Fill, layout::Size::Fill)
                .with_padding(layout::Insets::splat(theme.density().app_padding()))
                .with_gap(theme.density().app_padding())
                .with_child(editor_panel(theme, buffer))
                .with_child(status_panel(theme, edit_count, last_action)),
        )
}

fn editor_panel(theme: &Theme, buffer: &text::Buffer) -> ui::Node {
    widget::panel_with_theme(EDITOR_PANEL, theme)
        .with_size(layout::Size::Fill, layout::Size::Fixed(156.0))
        .with_padding(layout::Insets::splat(theme.density().panel_padding()))
        .with_gap(10.0)
        .with_child(widget::text_with_theme(
            TITLE,
            "Single-line text field",
            theme,
        ))
        .with_child(
            widget::text_field_with_theme(FIELD, buffer.clone(), theme)
                .with_responder_binding(action::Binding::new(INSERT_SAMPLE).enabled(true))
                .with_size(layout::Size::Fill, layout::Size::Fixed(36.0)),
        )
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
                        SAMPLE_BUTTON,
                        INSERT_SAMPLE,
                        "Insert sample",
                        theme,
                    )
                    .with_command_subject(ui::CommandSubject::Current)
                    .with_size(layout::Size::Fill, layout::Size::Fill),
                ),
        )
}

fn status_panel(theme: &Theme, edit_count: u32, last_action: &str) -> ui::Node {
    widget::panel_with_theme(STATUS, theme)
        .with_size(layout::Size::Fill, layout::Size::Fit)
        .with_padding(layout::Insets::splat(theme.density().panel_padding()))
        .with_label(document(
            format!(
                "Edits: {edit_count}\nLast action: {last_action}\nCut, Copy, and Paste are registered with shortcuts but disabled until clipboard support lands."
            ),
            text::Align::Start,
            theme.text().body_size(),
            theme.text().secondary(),
        ))
}

fn edit_menu() -> widget::menu::Bar {
    widget::menu::Bar::new()
        .menu(widget::Menu::new(FILE_MENU, "File").section(widget::menu::Section::new()))
        .menu(
            widget::Menu::new(EDIT_MENU, "Edit").section(
                widget::menu::Section::new()
                    .item(widget::menu::Item::new(action::SELECT_ALL))
                    .separator()
                    .item(widget::menu::Item::new(action::CUT))
                    .item(widget::menu::Item::new(action::COPY))
                    .item(widget::menu::Item::new(action::PASTE))
                    .separator()
                    .item(widget::menu::Item::new(INSERT_SAMPLE)),
            ),
        )
}

fn document(
    label: impl Into<String>,
    align: text::Align,
    size: f32,
    color: paint::Color,
) -> text::Document {
    let mut block = text::Block::new(align);
    block.push_run(text::Run::new(
        label,
        text::Style::default().with_size(size).with_color(color),
    ));

    text::Document::from_block(block)
}

fn field_path() -> ui::Path {
    ui::Path::new([ROOT, CONTENT, EDITOR_PANEL, FIELD])
}
