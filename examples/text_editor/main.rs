use wgpu_l3::{
    Action, Event, Theme, action, app, geometry::area, layout, paint, text, ui, widget, window,
};

const INSERT_SAMPLE: action::Id = action::Id::new("insert_sample_text");

const ROOT: ui::Id = ui::Id::new("root");
const MENU_BAR: ui::Id = ui::Id::new("menu_bar");
const CONTENT: ui::Id = ui::Id::new("content");
const EDITOR_PANEL: ui::Id = ui::Id::new("editor_panel");
const TITLE: ui::Id = ui::Id::new("title");
const EDITABLE_LABEL: ui::Id = ui::Id::new("editable_label");
const FIELD: ui::Id = ui::Id::new("field");
const PLACEHOLDER_LABEL: ui::Id = ui::Id::new("placeholder_label");
const PLACEHOLDER_FIELD: ui::Id = ui::Id::new("placeholder_field");
const READ_ONLY_LABEL: ui::Id = ui::Id::new("read_only_label");
const READ_ONLY_FIELD: ui::Id = ui::Id::new("read_only_field");
const OBSCURED_LABEL: ui::Id = ui::Id::new("obscured_label");
const OBSCURED_FIELD: ui::Id = ui::Id::new("obscured_field");
const DISABLED_LABEL: ui::Id = ui::Id::new("disabled_label");
const DISABLED_FIELD: ui::Id = ui::Id::new("disabled_field");
const STATUS: ui::Id = ui::Id::new("status");
const COMMAND_ROW: ui::Id = ui::Id::new("command_row");

const FILE_MENU: widget::menu::Id = widget::menu::Id::new("file");
const EDIT_MENU: widget::menu::Id = widget::menu::Id::new("edit");

fn main() -> app::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    app::run(Editor::default())
}

struct Editor {
    window: Option<window::Id>,
    buffer: text::Buffer,
    placeholder_buffer: text::Buffer,
    read_only_buffer: text::Buffer,
    obscured_buffer: text::Buffer,
    edit_count: u32,
    last_action: String,
}

enum AppEvent {
    ApplyEdit {
        target: action::Context,
        edit: text::Edit,
        label: &'static str,
    },
    ApplyCommand {
        target: action::Context,
        command: text::Command,
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
            placeholder_buffer: text::Buffer::new(),
            read_only_buffer: text::Buffer::from_text("Selectable read-only value"),
            obscured_buffer: text::Buffer::from_text("secret"),
            edit_count: 0,
            last_action: "none".to_owned(),
        }
    }
}

impl app::Application for Editor {
    type Event = AppEvent;

    fn started(&mut self, cx: &mut app::Context<'_, Self::Event>) {
        cx.register_action(
            Action::new(action::UNDO, "Undo")
                .with_shortcut(action::Shortcut::control('z'))
                .emit(|invocation| AppEvent::ApplyCommand {
                    target: invocation.context().clone(),
                    command: text::Command::Undo,
                    label: "undo",
                }),
        );
        cx.register_action(
            Action::new(action::REDO, "Redo")
                .with_shortcut(action::Shortcut::control_shift('z'))
                .with_shortcut(action::Shortcut::control('y'))
                .emit(|invocation| AppEvent::ApplyCommand {
                    target: invocation.context().clone(),
                    command: text::Command::Redo,
                    label: "redo",
                }),
        );
        cx.register_action(
            Action::new(action::SELECT_ALL, "Select All")
                .with_shortcut(action::Shortcut::control('a'))
                .emit(|invocation| AppEvent::ApplyCommand {
                    target: invocation.context().clone(),
                    command: text::Command::SelectAll,
                    label: "select all",
                }),
        );
        cx.register_action(
            Action::new(action::CUT, "Cut")
                .with_shortcut(action::Shortcut::control('x'))
                .emit(|invocation| AppEvent::ApplyCommand {
                    target: invocation.context().clone(),
                    command: text::Command::Cut,
                    label: "cut",
                }),
        );
        cx.register_action(
            Action::new(action::COPY, "Copy")
                .with_shortcut(action::Shortcut::control('c'))
                .emit(|invocation| AppEvent::ApplyCommand {
                    target: invocation.context().clone(),
                    command: text::Command::Copy,
                    label: "copy",
                }),
        );
        cx.register_action(
            Action::new(action::PASTE, "Paste")
                .with_shortcut(action::Shortcut::control('v'))
                .emit(|invocation| AppEvent::ApplyCommand {
                    target: invocation.context().clone(),
                    command: text::Command::Paste,
                    label: "paste",
                }),
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
            inner_area: area::physical(820, 620),
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
            } if event_window == window => {
                if self.apply_edit(cx, &target, edit, "keyboard/pointer") {
                    cx.request_redraw(window);
                }
            }
            Event::Ui { .. } => {}
            Event::App(AppEvent::ApplyEdit {
                target,
                edit,
                label,
            }) => {
                if let action::Scope::Path(path) = target.scope() {
                    let path = path.clone();
                    self.apply_edit(cx, &path, edit, label);
                    cx.request_redraw(window);
                }
            }
            Event::App(AppEvent::ApplyCommand {
                target,
                command,
                label,
            }) => {
                if let action::Scope::Path(path) = target.scope() {
                    let path = path.clone();
                    self.apply_command(cx, &path, command, label);
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
        cx.action(window, action::UNDO).enabled(false).active(false);
        cx.action(window, action::REDO).enabled(false).active(false);
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
        tree.set_root(root_view(&theme, self, self.edit_count, &self.last_action));
    }
}

impl Editor {
    fn apply_edit(
        &mut self,
        cx: &mut app::Context<'_, AppEvent>,
        target: &ui::Path,
        edit: text::Edit,
        label: &'static str,
    ) -> bool {
        let Some(buffer) = self.buffer_for_path_mut(target) else {
            return false;
        };

        if !cx.apply_text_edit_for(target, buffer, edit) {
            return false;
        }

        self.edit_count += 1;
        self.last_action = label.to_owned();
        true
    }

    fn apply_command(
        &mut self,
        cx: &mut app::Context<'_, AppEvent>,
        target: &ui::Path,
        command: text::Command,
        label: &'static str,
    ) {
        let Some(buffer) = self.buffer_for_path_mut(target) else {
            return;
        };

        let result = cx.apply_text_command_for(target, buffer, command);

        if result.buffer_changed() {
            self.edit_count += 1;
        }
        self.last_action = if result.unavailable {
            format!("{label} unavailable")
        } else {
            label.to_owned()
        };
    }

    fn buffer_for_path_mut(&mut self, target: &ui::Path) -> Option<&mut text::Buffer> {
        if target == &field_path(FIELD) {
            Some(&mut self.buffer)
        } else if target == &field_path(PLACEHOLDER_FIELD) {
            Some(&mut self.placeholder_buffer)
        } else if target == &field_path(READ_ONLY_FIELD) {
            Some(&mut self.read_only_buffer)
        } else if target == &field_path(OBSCURED_FIELD) {
            Some(&mut self.obscured_buffer)
        } else {
            None
        }
    }
}

fn root_view(theme: &Theme, editor: &Editor, edit_count: u32, last_action: &str) -> ui::Node {
    ui::Node::container(ROOT, layout::Axis::Vertical)
        .with_background(theme.surfaces().app())
        .with_child(widget::menu_bar_with_theme(MENU_BAR, edit_menu(), theme))
        .with_child(
            ui::Node::container(CONTENT, layout::Axis::Vertical)
                .with_size(layout::Size::Fill, layout::Size::Fill)
                .with_padding(layout::Insets::splat(theme.density().app_padding()))
                .with_gap(theme.density().app_padding())
                .with_child(editor_panel(theme, editor))
                .with_child(status_panel(theme, edit_count, last_action)),
        )
}

fn editor_panel(theme: &Theme, editor: &Editor) -> ui::Node {
    widget::panel_with_theme(EDITOR_PANEL, theme)
        .with_size(layout::Size::Fill, layout::Size::Fit)
        .with_padding(layout::Insets::splat(theme.density().panel_padding()))
        .with_gap(10.0)
        .with_child(widget::text_with_theme(
            TITLE,
            "Text field semantics",
            theme,
        ))
        .with_child(field_label(
            EDITABLE_LABEL,
            "Editable with undo/redo",
            theme,
        ))
        .with_child(
            widget::text_field_with_theme(
                FIELD,
                text::Field::new(editor.buffer.clone()).with_placeholder("Type editable text"),
                theme,
            )
            .with_responder_binding(action::Binding::new(INSERT_SAMPLE).enabled(true))
            .with_size(layout::Size::Fill, layout::Size::Fixed(36.0)),
        )
        .with_child(field_label(
            PLACEHOLDER_LABEL,
            "Editable placeholder",
            theme,
        ))
        .with_child(
            widget::text_field_with_theme(
                PLACEHOLDER_FIELD,
                text::Field::new(editor.placeholder_buffer.clone())
                    .with_placeholder("Placeholder disappears when you type"),
                theme,
            )
            .with_size(layout::Size::Fill, layout::Size::Fixed(36.0)),
        )
        .with_child(field_label(
            READ_ONLY_LABEL,
            "Read-only, selectable, copyable",
            theme,
        ))
        .with_child(
            widget::text_field_with_theme(
                READ_ONLY_FIELD,
                text::Field::new(editor.read_only_buffer.clone()).read_only(),
                theme,
            )
            .with_size(layout::Size::Fill, layout::Size::Fixed(36.0)),
        )
        .with_child(field_label(OBSCURED_LABEL, "Obscured dot field", theme))
        .with_child(
            widget::text_field_with_theme(
                OBSCURED_FIELD,
                text::Field::new(editor.obscured_buffer.clone())
                    .obscured_dot()
                    .with_placeholder("Password"),
                theme,
            )
            .with_size(layout::Size::Fill, layout::Size::Fixed(36.0)),
        )
        .with_child(field_label(DISABLED_LABEL, "Disabled field", theme))
        .with_child(
            widget::text_field_with_theme(
                DISABLED_FIELD,
                text::Field::new("Disabled fields do not focus or edit").disabled(),
                theme,
            )
            .with_size(layout::Size::Fill, layout::Size::Fixed(36.0)),
        )
        .with_child(
            ui::Node::container(COMMAND_ROW, layout::Axis::Horizontal)
                .with_size(
                    layout::Size::Fill,
                    layout::Size::Fixed(theme.density().control_height()),
                )
                .with_gap(8.0),
        )
}

fn field_label(id: ui::Id, label: &'static str, theme: &Theme) -> ui::Node {
    widget::text_with_theme(id, label, theme)
        .with_size(layout::Size::Fill, layout::Size::Fixed(20.0))
}

fn status_panel(theme: &Theme, edit_count: u32, last_action: &str) -> ui::Node {
    widget::panel_with_theme(STATUS, theme)
        .with_size(layout::Size::Fill, layout::Size::Fit)
        .with_padding(layout::Insets::splat(theme.density().panel_padding()))
        .with_label(document(
            format!(
                "Edits: {edit_count}\nLast action: {last_action}\nCut, Copy, and Paste use the system clipboard."
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
                    .item(widget::menu::Item::new(action::UNDO))
                    .item(widget::menu::Item::new(action::REDO))
                    .separator()
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

fn field_path(field: ui::Id) -> ui::Path {
    ui::Path::new([ROOT, CONTENT, EDITOR_PANEL, field])
}
