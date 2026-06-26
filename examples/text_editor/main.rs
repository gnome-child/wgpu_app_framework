use std::path::{Path, PathBuf};

use wgpu_l3::{
    Event, Task, Theme, app, command, geometry::area, layout, paint, text, ui, widget, window,
};

const UNICODE_STRESS_TEXT: &str = include_str!("fixtures/unicode_stress_dump.txt");

wgpu_l3::command!(NewFile {
    name: "new_file",
    display: "New",
});
wgpu_l3::command!(OpenFile {
    name: "open_file",
    display: "Open...",
});
wgpu_l3::command!(SaveFile {
    name: "save_file",
    display: "Save",
});
wgpu_l3::command!(SaveAsFile {
    name: "save_as_file",
    display: "Save As...",
});
wgpu_l3::command!(ExitApp {
    name: "exit_app",
    display: "Exit",
});
wgpu_l3::command!(ToggleWrapText {
    name: "toggle_wrap_text",
    display: "Wrap text",
});
wgpu_l3::command!(ToggleDebugPanel {
    name: "toggle_debug_panel",
    display: "Debug panel",
});
wgpu_l3::command!(LoadStressText {
    name: "load_stress_text",
    display: "Load Stress Text",
});

fn main() -> app::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    app::run(Editor::default())
}

struct Editor {
    window: Option<window::Id>,
    sender: Option<app::Sender<AppEvent>>,
    buffer: text::Buffer,
    path: Option<PathBuf>,
    dirty: bool,
    wrap_text: bool,
    show_debug_panel: bool,
    edit_count: u32,
    last_status: String,
}

enum AppEvent {
    OpenPathSelected(Option<PathBuf>),
    SaveAsPathSelected(Option<PathBuf>),
    ExitRequested,
    FileSaved {
        path: PathBuf,
        result: Result<(), String>,
    },
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            window: None,
            sender: None,
            buffer: text::Buffer::new_multiline(),
            path: None,
            dirty: false,
            wrap_text: true,
            show_debug_panel: false,
            edit_count: 0,
            last_status: "ready".to_owned(),
        }
    }
}

impl app::Application for Editor {
    type Event = AppEvent;

    fn started(&mut self, cx: &mut app::Context<'_, Self::Event>) {
        self.sender = Some(cx.sender());
        configure_commands(cx);

        let theme = Theme::default_dark();
        self.window = Some(cx.open_window(window::Options {
            title: "wgpu_l3 Notepad".to_owned(),
            inner_area: area::physical(920, 680),
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
                event: ui::Event::TextEditRequested { edit, .. },
            } if event_window == window => {
                if self.apply_edit(cx, edit, "edit") {
                    cx.request_redraw(window);
                }
            }
            Event::Ui {
                window: event_window,
                event:
                    ui::Event::TextDropRequested {
                        source_cleanup,
                        edit,
                        ..
                    },
            } if event_window == window => {
                let mut changed = self.apply_edit(cx, edit, "drag/drop");
                if changed && let Some((_, edit)) = source_cleanup {
                    changed |= self.apply_edit(cx, edit, "drag/drop cleanup");
                }

                if changed {
                    cx.request_redraw(window);
                }
            }
            Event::Ui { .. } => {}
            Event::App(AppEvent::OpenPathSelected(path)) => {
                self.open_path(path);
                cx.request_redraw(window);
            }
            Event::App(AppEvent::SaveAsPathSelected(path)) => {
                self.save_as(cx, path);
                cx.request_redraw(window);
            }
            Event::App(AppEvent::ExitRequested) => {
                cx.close_window(window);
            }
            Event::App(AppEvent::FileSaved { path, result }) => {
                self.finish_save(path, result);
                cx.request_redraw(window);
            }
        }
    }

    fn command_targets(&mut self, commands: &mut app::CommandDispatch<'_, Self::Event>) {
        if let Some(outcome) = commands.text_buffer(&mut self.buffer) {
            self.record_text_command_outcome(outcome);
        }

        commands.target(self);
    }

    fn command_states(&mut self, states: &mut app::CommandStates<'_, Self::Event>) {
        states.target(self);
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
        let diagnostics = cx.diagnostics(window);
        tree.clear_popups();
        tree.set_root(root_view(&theme, self, diagnostics));
    }
}

impl Editor {
    fn apply_edit(
        &mut self,
        cx: &mut app::Context<'_, AppEvent>,
        edit: text::edit::Edit,
        label: &'static str,
    ) -> bool {
        let mutates_text = edit_mutates_text(&edit);
        if !cx.apply_text_edit(&mut self.buffer, edit) {
            return false;
        }

        if mutates_text {
            self.dirty = true;
            self.edit_count += 1;
        }
        self.last_status = label.to_owned();
        true
    }

    fn open_file(&mut self) {
        self.last_status = "choosing file".to_owned();
        self.spawn_app_task(Task::future(async {
            AppEvent::OpenPathSelected(open_path_dialog().await)
        }));
    }

    fn choose_save_path(&mut self) {
        self.last_status = "choosing save location".to_owned();
        self.spawn_app_task(Task::future(async {
            AppEvent::SaveAsPathSelected(save_path_dialog().await)
        }));
    }

    fn record_text_command_result(
        &mut self,
        result: text::edit::CommandResult,
        label: &'static str,
    ) {
        if result.text_changed {
            self.dirty = true;
            self.edit_count += 1;
        }
        self.last_status = if result.unavailable {
            format!("{label} unavailable")
        } else {
            label.to_owned()
        };
    }

    fn record_text_command_outcome(&mut self, outcome: app::TextCommandOutcome) {
        self.record_text_command_result(outcome.result(), text_command_label(outcome.command()));
    }

    fn new_file(&mut self) {
        self.buffer = text::Buffer::new_multiline();
        self.path = None;
        self.dirty = false;
        self.edit_count = 0;
        self.last_status = "new file".to_owned();
    }

    fn open_path(&mut self, path: Option<PathBuf>) {
        let Some(path) = path else {
            self.last_status = "open canceled".to_owned();
            return;
        };

        match text::Buffer::from_mapped_file(&path) {
            Ok(buffer) => {
                self.buffer = buffer;
                self.path = Some(path.clone());
                self.dirty = false;
                self.edit_count = 0;
                self.last_status = format!("opened {}", compact_path(&path));
            }
            Err(error) => {
                self.last_status = format!("open failed: {error}");
            }
        }
    }

    fn save(&mut self) {
        let Some(path) = self.path.clone() else {
            self.choose_save_path();
            return;
        };

        self.spawn_save_task(path);
    }

    fn save_as(&mut self, cx: &app::Context<'_, AppEvent>, path: Option<PathBuf>) {
        let Some(path) = path else {
            self.last_status = "save canceled".to_owned();
            return;
        };

        self.spawn_save(cx, path);
    }

    fn spawn_save(&mut self, cx: &app::Context<'_, AppEvent>, path: PathBuf) {
        let text = self.buffer.text();
        self.last_status = format!("saving {}", compact_path(&path));
        cx.spawn(save_task(path, text));
    }

    fn spawn_save_task(&mut self, path: PathBuf) {
        let text = self.buffer.text();
        self.last_status = format!("saving {}", compact_path(&path));
        self.spawn_app_task(save_task(path, text));
    }

    fn spawn_app_task(&self, task: Task<AppEvent>) {
        let Some(sender) = self.sender.clone() else {
            return;
        };

        std::thread::spawn(move || {
            let event = task.run();
            let _ = sender.emit(event);
        });
    }

    fn finish_save(&mut self, path: PathBuf, result: Result<(), String>) {
        match result {
            Ok(()) => {
                self.path = Some(path.clone());
                self.dirty = false;
                self.last_status = format!("saved {}", compact_path(&path));
            }
            Err(error) => {
                self.last_status = format!("save failed: {error}");
            }
        }
    }
}

impl command::Target<NewFile> for Editor {
    fn invoke(
        &mut self,
        _args: (),
        _invocation: command::call::Invocation<NewFile>,
    ) -> command::Response<()> {
        self.new_file();
        command::Response::none()
    }
}

impl command::Target<OpenFile> for Editor {
    fn invoke(
        &mut self,
        _args: (),
        _invocation: command::call::Invocation<OpenFile>,
    ) -> command::Response<()> {
        self.open_file();
        command::Response::none()
    }
}

impl command::Target<SaveFile> for Editor {
    fn invoke(
        &mut self,
        _args: (),
        _invocation: command::call::Invocation<SaveFile>,
    ) -> command::Response<()> {
        self.save();
        command::Response::none()
    }
}

impl command::Target<SaveAsFile> for Editor {
    fn invoke(
        &mut self,
        _args: (),
        _invocation: command::call::Invocation<SaveAsFile>,
    ) -> command::Response<()> {
        self.choose_save_path();
        command::Response::none()
    }
}

impl command::Target<ExitApp> for Editor {
    fn invoke(
        &mut self,
        _args: (),
        _invocation: command::call::Invocation<ExitApp>,
    ) -> command::Response<()> {
        if let Some(sender) = self.sender.clone() {
            let _ = sender.emit(AppEvent::ExitRequested);
        }

        command::Response::none()
    }
}

impl command::Target<LoadStressText> for Editor {
    fn invoke(
        &mut self,
        _args: (),
        _invocation: command::call::Invocation<LoadStressText>,
    ) -> command::Response<()> {
        self.buffer = text::Buffer::from_multiline_text(UNICODE_STRESS_TEXT);
        self.path = None;
        self.dirty = true;
        self.edit_count = 0;
        self.last_status = format!(
            "loaded Unicode stress fixture ({} lines)",
            UNICODE_STRESS_TEXT.lines().count()
        );
        command::Response::none()
    }
}

impl command::Target<ToggleWrapText> for Editor {
    fn state(&self, _context: &command::call::Context) -> command::State {
        command::State::active_if(self.wrap_text)
    }

    fn invoke(
        &mut self,
        _args: (),
        _invocation: command::call::Invocation<ToggleWrapText>,
    ) -> command::Response<()> {
        self.wrap_text = !self.wrap_text;
        self.last_status = if self.wrap_text {
            "wrap text enabled".to_owned()
        } else {
            "wrap text disabled".to_owned()
        };
        command::Response::none()
    }
}

impl command::Target<ToggleDebugPanel> for Editor {
    fn state(&self, _context: &command::call::Context) -> command::State {
        command::State::active_if(self.show_debug_panel)
    }

    fn invoke(
        &mut self,
        _args: (),
        _invocation: command::call::Invocation<ToggleDebugPanel>,
    ) -> command::Response<()> {
        self.show_debug_panel = !self.show_debug_panel;
        self.last_status = if self.show_debug_panel {
            "debug panel shown".to_owned()
        } else {
            "debug panel hidden".to_owned()
        };
        command::Response::none()
    }
}

fn configure_commands(cx: &mut app::Context<'_, AppEvent>) {
    cx.commands(|commands| {
        register_file_commands(commands);
        register_edit_commands(commands);
        register_view_commands(commands);
    });
}

fn register_file_commands(commands: &mut command::registry::Commands) {
    commands
        .define::<NewFile, Editor>(|command| {
            command.shortcut(command::shortcut::Shortcut::ctrl('n'))
        })
        .define::<OpenFile, Editor>(|command| {
            command.shortcut(command::shortcut::Shortcut::ctrl('o'))
        })
        .define::<SaveFile, Editor>(|command| {
            command.shortcut(command::shortcut::Shortcut::ctrl('s'))
        })
        .define::<SaveAsFile, Editor>(|command| {
            command.shortcut(command::shortcut::Shortcut::ctrl_shift('s'))
        })
        .define::<ExitApp, Editor>(|command| command)
        .define::<LoadStressText, Editor>(|command| command);
}

fn register_edit_commands(commands: &mut command::registry::Commands) {
    text::command::define::<text::command::Undo>(commands, |command| {
        command
            .shortcut(command::shortcut::Shortcut::ctrl('z'))
            .repeatable()
    });
    text::command::define::<text::command::Redo>(commands, |command| {
        command
            .shortcut(command::shortcut::Shortcut::ctrl_shift('z'))
            .shortcut(command::shortcut::Shortcut::ctrl('y'))
            .repeatable()
    });
    text::command::define::<text::command::SelectAll>(commands, |command| {
        command.shortcut(command::shortcut::Shortcut::ctrl('a'))
    });
    text::command::define::<text::command::Cut>(commands, |command| {
        command.shortcut(command::shortcut::Shortcut::ctrl('x'))
    });
    text::command::define::<text::command::Delete>(commands, |command| command);
    text::command::define::<text::command::Copy>(commands, |command| {
        command.shortcut(command::shortcut::Shortcut::ctrl('c'))
    });
    text::command::define::<text::command::Paste>(commands, |command| {
        command
            .shortcut(command::shortcut::Shortcut::ctrl('v'))
            .repeatable()
    });
}

fn register_view_commands(commands: &mut command::registry::Commands) {
    commands
        .define::<ToggleWrapText, Editor>(|command| command)
        .define::<ToggleDebugPanel, Editor>(|command| command);
}

fn root_view(theme: &Theme, editor: &Editor, diagnostics: app::Diagnostics) -> ui::Node {
    let app_shell = ui::Node::container(layout::Axis::Vertical)
        .size(layout::Size::fill(), layout::Size::fill())
        .child(widget::menu_bar_with_theme(notepad_menu(), theme))
        .child(editor_area(theme, editor));

    let root = ui::Node::new()
        .with_background(theme.surfaces().app())
        .child(app_shell);

    if editor.show_debug_panel {
        root.child(debug_overlay(theme, editor, diagnostics))
    } else {
        root
    }
}

fn editor_area(theme: &Theme, editor: &Editor) -> ui::Node {
    let wrap = if editor.wrap_text {
        text::AreaWrap::WordOrGlyph
    } else {
        text::AreaWrap::None
    };
    let area = text::Area::new(editor.buffer.clone()).with_wrap(wrap);

    ui::Node::container(layout::Axis::Vertical)
        .with_background(theme.surfaces().canvas())
        .size(layout::Size::fill(), layout::Size::fill())
        .child(
            widget::text_area_surface_with_theme(area, theme)
                .size(layout::Size::fill(), layout::Size::fill()),
        )
}

fn debug_overlay(theme: &Theme, editor: &Editor, diagnostics: app::Diagnostics) -> ui::Node {
    ui::Node::container(layout::Axis::Vertical)
        .size(layout::Size::fill(), layout::Size::fill())
        .padding(theme.density().app_padding() * 2.0)
        .with_align(layout::Align::End)
        .with_cross_align(layout::Align::Stretch)
        .child(debug_panel(theme, editor, diagnostics))
}

fn debug_panel(theme: &Theme, editor: &Editor, diagnostics: app::Diagnostics) -> ui::Node {
    let text = diagnostics.text;
    let scroll = diagnostics.scroll;
    let frame = diagnostics.frame;
    let dirty = if editor.dirty { "modified" } else { "saved" };
    let path = editor
        .path
        .as_deref()
        .map(compact_path)
        .unwrap_or_else(|| "Untitled".to_owned());
    let wrap = if editor.wrap_text { "on" } else { "off" };

    widget::floating_panel_with_theme(theme)
        .with_size(layout::Size::Fill, layout::Size::Fit)
        .with_child(
            ui::Node::leaf()
                .with_size(layout::Size::Fill, layout::Size::Fit)
                .with_label(document(
            format!(
                "File: {path} ({dirty}) | Wrap: {wrap}\nDocument: {} lines, {} bytes | Edits: {} | Status: {}\nText layout: paint {}, metrics {}, visible {}, shaped {}, segments {}+{}, overlays {}, highlight scans {}\nText caches: line {}/{}, render surfaces {}, render cache {}/{}, render source {} lines / {} bytes\nScroll: wheel {}, offsets {}, queued {}, redraws {}, commits {}, pending {}/{}\nText scroll: projections {}, resolve/reuse/shift {}/{}/{}, shift misses {}, cold jumps {}, async reconcile/skips {}/{}\nRetained scroll: layer hits {}, text skips {}, target fallbacks {}, layer rebuilds {}\nFrames: full {}, scroll-only {}, scroll fallbacks {}, render skips {}\nFrame us latest/avg: paint {}/{}, render {}/{}, text {}/{}, total {}/{}\nLast scroll frame: text {}us, render {}us, total {}us, surfaces {}, glyph batches {}",
                editor.buffer.logical_line_count(),
                editor.buffer.len(),
                editor.edit_count,
                editor.last_status,
                text.text_area_paint_layout_calls,
                text.text_area_metrics_layout_calls,
                text.text_area_visible_logical_lines,
                text.text_area_shaped_logical_lines,
                text.text_area_layout_segments,
                text.text_area_overscan_segments,
                text.text_area_interaction_surfaces,
                text.highlight_run_scans,
                text.text_area_line_cache_hits,
                text.text_area_line_cache_misses,
                text.text_area_render_surface_calls,
                text.text_area_render_surface_cache_hits,
                text.text_area_render_surface_cache_misses,
                text.text_area_render_surface_source_lines,
                text.text_area_render_surface_source_bytes,
                scroll.wheel_events,
                scroll.scroll_offset_changes,
                scroll.queued_scroll_updates,
                scroll.scroll_redraw_requests,
                scroll.frame_scroll_commits,
                scroll.pending_scroll_applications,
                scroll.pending_scroll_updates,
                scroll.projection_count,
                scroll.text_area_resolves,
                scroll.text_area_projection_reuses,
                scroll.text_area_projection_shifts,
                scroll.text_area_projection_shift_misses,
                scroll.text_area_projection_cold_jumps,
                scroll.async_scroll_reconciles,
                scroll.async_scroll_projection_sync_skips,
                scroll.retained_scroll_layer_hits,
                scroll.retained_scroll_layer_text_prepare_skips,
                scroll.retained_scroll_target_repaint_fallbacks,
                scroll.retained_scroll_layer_rebuilds,
                frame.full_redraws,
                frame.scroll_only_redraws,
                frame.scroll_only_fallbacks_to_full,
                frame.render_skips,
                frame.paint.latest_us,
                frame.paint.average_us,
                frame.render.latest_us,
                frame.render.average_us,
                frame.render_text_prepare.latest_us,
                frame.render_text_prepare.average_us,
                frame.total.latest_us,
                frame.total.average_us,
                frame.last_scroll_frame.render_text_prepare_us,
                frame.last_scroll_frame.render_total_us,
                frame.last_scroll_frame.total_us,
                frame.last_scroll_frame.text_surfaces,
                frame.last_scroll_frame.glyph_batches,
            ),
            text::document::Align::Start,
            theme.text().body_size(),
            theme.text().secondary(),
                )),
        )
}

fn notepad_menu() -> widget::menu::Bar {
    widget::menu::Bar::new()
        .menu(
            widget::Menu::new("File").section(
                widget::menu::Section::new()
                    .item(widget::menu::Item::invokes::<NewFile, Editor>())
                    .item(widget::menu::Item::invokes::<OpenFile, Editor>())
                    .item(widget::menu::Item::invokes::<SaveFile, Editor>())
                    .item(widget::menu::Item::invokes::<SaveAsFile, Editor>())
                    .separator()
                    .item(widget::menu::Item::invokes::<LoadStressText, Editor>())
                    .separator()
                    .item(widget::menu::Item::invokes::<ExitApp, Editor>()),
            ),
        )
        .menu(
            widget::Menu::new("Edit").section(
                widget::menu::Section::new()
                    .item(widget::menu::Item::text::<text::command::Undo>())
                    .item(widget::menu::Item::text::<text::command::Redo>())
                    .separator()
                    .item(widget::menu::Item::text::<text::command::Cut>())
                    .item(widget::menu::Item::text::<text::command::Copy>())
                    .item(widget::menu::Item::text::<text::command::Paste>())
                    .item(widget::menu::Item::text::<text::command::Delete>())
                    .separator()
                    .item(widget::menu::Item::text::<text::command::SelectAll>()),
            ),
        )
        .menu(
            widget::Menu::new("View").section(
                widget::menu::Section::new()
                    .item(widget::menu::Item::invokes::<ToggleWrapText, Editor>())
                    .item(widget::menu::Item::invokes::<ToggleDebugPanel, Editor>()),
            ),
        )
}

async fn open_path_dialog() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .pick_file()
        .await
        .map(|file| file.path().to_path_buf())
}

async fn save_path_dialog() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .save_file()
        .await
        .map(|file| file.path().to_path_buf())
}

fn edit_mutates_text(edit: &text::edit::Edit) -> bool {
    matches!(
        edit,
        text::edit::Edit::Insert(_)
            | text::edit::Edit::ImeCommit(_)
            | text::edit::Edit::ReplaceRange { .. }
            | text::edit::Edit::MoveRange { .. }
            | text::edit::Edit::Backspace
            | text::edit::Edit::Delete
            | text::edit::Edit::InsertLineBreak
            | text::edit::Edit::DeleteWordBackward
            | text::edit::Edit::DeleteWordForward
    )
}

fn compact_path(path: &Path) -> String {
    let path = path.display().to_string();
    let max_chars = 120;
    if path.chars().count() <= max_chars {
        return path;
    }

    let suffix = path
        .chars()
        .rev()
        .take(max_chars - 3)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    format!("...{suffix}")
}

fn text_command_label(command: text::edit::Command) -> &'static str {
    match command {
        text::edit::Command::Copy => "copy",
        text::edit::Command::Cut => "cut",
        text::edit::Command::Delete => "delete",
        text::edit::Command::Paste => "paste",
        text::edit::Command::SelectAll => "select all",
        text::edit::Command::Undo => "undo",
        text::edit::Command::Redo => "redo",
    }
}

fn save_task(path: PathBuf, text: String) -> Task<AppEvent> {
    Task::future(async move {
        let result = std::fs::write(&path, text).map_err(|error| error.to_string());
        AppEvent::FileSaved { path, result }
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_editor_buffer_is_multiline() {
        let editor = Editor::default();

        assert!(editor.buffer.is_multiline());
        assert_eq!(editor.buffer.text(), "");
    }

    #[test]
    fn debug_panel_is_hidden_by_default() {
        let editor = Editor::default();

        assert!(!editor.show_debug_panel);
    }

    #[test]
    fn new_file_resets_to_multiline_buffer() {
        let mut editor = Editor::default();
        editor.buffer = text::Buffer::from_text("single line");

        editor.new_file();

        assert!(editor.buffer.is_multiline());
        assert_eq!(editor.buffer.text(), "");
    }

    #[test]
    fn initial_empty_editor_accepts_newlines_and_select_all() {
        let mut editor = Editor::default();
        let mut text_editor = text::edit::Editor::new();

        text_editor.apply_text_edit(&mut editor.buffer, text::edit::Edit::insert("alpha"));
        text_editor.apply_text_edit(&mut editor.buffer, text::edit::Edit::insert_line_break());
        text_editor.apply_text_edit(&mut editor.buffer, text::edit::Edit::insert("beta"));
        text_editor.apply_text_edit(&mut editor.buffer, text::edit::Edit::SelectAll);

        assert_eq!(editor.buffer.text(), "alpha\nbeta");
        assert_eq!(
            editor.buffer.selected_text(),
            Some("alpha\nbeta".to_owned())
        );
    }
}
