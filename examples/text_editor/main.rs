use std::path::{Path, PathBuf};

use wgpu_l3::{
    Action, Event, Task, Theme, action, app, geometry::area, layout, paint, text, ui, widget,
    window,
};

const NEW_FILE: action::Id = action::Id::new("new_file");
const OPEN_FILE: action::Id = action::Id::new("open_file");
const SAVE_FILE: action::Id = action::Id::new("save_file");
const SAVE_AS_FILE: action::Id = action::Id::new("save_as_file");
const EXIT_APP: action::Id = action::Id::new("exit_app");
const DELETE_TEXT: action::Id = action::Id::new("delete_text");
const TOGGLE_WRAP_TEXT: action::Id = action::Id::new("toggle_wrap_text");
const TOGGLE_DEBUG_PANEL: action::Id = action::Id::new("toggle_debug_panel");
const LOAD_STRESS_TEXT: action::Id = action::Id::new("load_stress_text");

const ROOT: ui::Id = ui::Id::new("root");
const APP_SHELL: ui::Id = ui::Id::new("app_shell");
const MENU_BAR: ui::Id = ui::Id::new("menu_bar");
const EDITOR_CANVAS: ui::Id = ui::Id::new("editor_canvas");
const EDITOR: ui::Id = ui::Id::new("editor");
const DEBUG_OVERLAY: ui::Id = ui::Id::new("debug_overlay");
const STATUS: ui::Id = ui::Id::new("status");
const STATUS_TEXT: ui::Id = ui::Id::new("status_text");

const FILE_MENU: widget::menu::Id = widget::menu::Id::new("file");
const EDIT_MENU: widget::menu::Id = widget::menu::Id::new("edit");
const VIEW_MENU: widget::menu::Id = widget::menu::Id::new("view");

fn main() -> app::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    app::run(Editor::default())
}

struct Editor {
    window: Option<window::Id>,
    buffer: text::Buffer,
    path: Option<PathBuf>,
    dirty: bool,
    wrap_text: bool,
    show_debug_panel: bool,
    edit_count: u32,
    last_status: String,
}

enum AppEvent {
    ApplyEdit {
        target: action::Context,
        edit: text::edit::Edit,
        label: &'static str,
    },
    ApplyCommand {
        target: action::Context,
        command: text::edit::Command,
        label: &'static str,
    },
    NewFile,
    OpenPathSelected(Option<PathBuf>),
    SaveRequested,
    SaveAsRequested,
    SaveAsPathSelected(Option<PathBuf>),
    FileSaved {
        path: PathBuf,
        result: Result<(), String>,
    },
    LoadStressText,
    ToggleWrapText,
    ToggleDebugPanel,
    Exit,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            window: None,
            buffer: text::Buffer::new_multiline(),
            path: None,
            dirty: false,
            wrap_text: true,
            show_debug_panel: true,
            edit_count: 0,
            last_status: "ready".to_owned(),
        }
    }
}

impl app::Application for Editor {
    type Event = AppEvent;

    fn started(&mut self, cx: &mut app::Context<'_, Self::Event>) {
        register_file_actions(cx);
        register_edit_actions(cx);
        register_view_actions(cx);

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
                event: ui::Event::TextEditRequested { target, edit },
            } if event_window == window => {
                if self.apply_edit(cx, &target, edit, "edit") {
                    cx.request_redraw(window);
                }
            }
            Event::Ui {
                window: event_window,
                event:
                    ui::Event::TextDropRequested {
                        source_cleanup,
                        target,
                        edit,
                        ..
                    },
            } if event_window == window => {
                let mut changed = self.apply_edit(cx, &target, edit, "drag/drop");
                if changed && let Some((source, edit)) = source_cleanup {
                    changed |= self.apply_edit(cx, &source, edit, "drag/drop cleanup");
                }

                if changed {
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
            Event::App(AppEvent::NewFile) => {
                self.new_file();
                cx.request_redraw(window);
            }
            Event::App(AppEvent::OpenPathSelected(path)) => {
                self.open_path(path);
                cx.request_redraw(window);
            }
            Event::App(AppEvent::SaveRequested) => {
                self.save(cx);
                cx.request_redraw(window);
            }
            Event::App(AppEvent::SaveAsRequested) => {
                self.last_status = "choosing save location".to_owned();
                cx.spawn(Task::future(async {
                    AppEvent::SaveAsPathSelected(save_path_dialog().await)
                }));
                cx.request_redraw(window);
            }
            Event::App(AppEvent::SaveAsPathSelected(path)) => {
                self.save_as(cx, path);
                cx.request_redraw(window);
            }
            Event::App(AppEvent::FileSaved { path, result }) => {
                self.finish_save(path, result);
                cx.request_redraw(window);
            }
            Event::App(AppEvent::LoadStressText) => {
                self.buffer = text::Buffer::from_multiline_text(stress_text(20_000));
                self.path = None;
                self.dirty = true;
                self.edit_count = 0;
                self.last_status = "loaded generated 20k-line stress document".to_owned();
                cx.request_redraw(window);
            }
            Event::App(AppEvent::ToggleWrapText) => {
                self.wrap_text = !self.wrap_text;
                self.last_status = if self.wrap_text {
                    "wrap text enabled".to_owned()
                } else {
                    "wrap text disabled".to_owned()
                };
                cx.request_redraw(window);
            }
            Event::App(AppEvent::ToggleDebugPanel) => {
                self.show_debug_panel = !self.show_debug_panel;
                self.last_status = if self.show_debug_panel {
                    "debug panel shown".to_owned()
                } else {
                    "debug panel hidden".to_owned()
                };
                cx.request_redraw(window);
            }
            Event::App(AppEvent::Exit) => {
                cx.close_window(window);
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

        cx.action(window, NEW_FILE).enabled(true).active(false);
        cx.action(window, OPEN_FILE).enabled(true).active(false);
        cx.action(window, SAVE_FILE).enabled(true).active(false);
        cx.action(window, SAVE_AS_FILE).enabled(true).active(false);
        cx.action(window, EXIT_APP).enabled(true).active(false);
        cx.action(window, LOAD_STRESS_TEXT)
            .enabled(true)
            .active(false);

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
        cx.action(window, DELETE_TEXT).enabled(false).active(false);
        cx.action(window, TOGGLE_WRAP_TEXT)
            .enabled(true)
            .active(self.wrap_text);
        cx.action(window, TOGGLE_DEBUG_PANEL)
            .enabled(true)
            .active(self.show_debug_panel);

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
        target: &ui::Path,
        edit: text::edit::Edit,
        label: &'static str,
    ) -> bool {
        if target != &editor_path() {
            return false;
        }

        let mutates_text = edit_mutates_text(&edit);
        if !cx.apply_text_edit_for(target, &mut self.buffer, edit) {
            return false;
        }

        if mutates_text {
            self.dirty = true;
            self.edit_count += 1;
        }
        self.last_status = label.to_owned();
        true
    }

    fn apply_command(
        &mut self,
        cx: &mut app::Context<'_, AppEvent>,
        target: &ui::Path,
        command: text::edit::Command,
        label: &'static str,
    ) {
        if target != &editor_path() {
            return;
        }

        let result = cx.apply_text_command_for(target, &mut self.buffer, command);

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

    fn save(&mut self, cx: &app::Context<'_, AppEvent>) {
        let Some(path) = self.path.clone() else {
            self.last_status = "choosing save location".to_owned();
            cx.spawn(Task::future(async {
                AppEvent::SaveAsPathSelected(save_path_dialog().await)
            }));
            return;
        };

        self.spawn_save(cx, path);
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
        cx.spawn(Task::future(async move {
            let result = std::fs::write(&path, text).map_err(|error| error.to_string());
            AppEvent::FileSaved { path, result }
        }));
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

fn register_file_actions(cx: &mut app::Context<'_, AppEvent>) {
    cx.register_action(
        Action::new(NEW_FILE, "New")
            .with_shortcut(action::Shortcut::control('n'))
            .emit(|_| AppEvent::NewFile),
    );
    cx.register_action(
        Action::new(OPEN_FILE, "Open...")
            .with_shortcut(action::Shortcut::control('o'))
            .task(|_| Task::future(async { AppEvent::OpenPathSelected(open_path_dialog().await) })),
    );
    cx.register_action(
        Action::new(SAVE_FILE, "Save")
            .with_shortcut(action::Shortcut::control('s'))
            .emit(|_| AppEvent::SaveRequested),
    );
    cx.register_action(
        Action::new(SAVE_AS_FILE, "Save As...")
            .with_shortcut(action::Shortcut::control_shift('s'))
            .emit(|_| AppEvent::SaveAsRequested),
    );
    cx.register_action(Action::new(EXIT_APP, "Exit").emit(|_| AppEvent::Exit));
    cx.register_action(
        Action::new(LOAD_STRESS_TEXT, "Load Stress Text").emit(|_| AppEvent::LoadStressText),
    );
}

fn register_edit_actions(cx: &mut app::Context<'_, AppEvent>) {
    cx.register_action(
        Action::new(action::UNDO, "Undo")
            .with_shortcut(action::Shortcut::control('z'))
            .emit(|invocation| AppEvent::ApplyCommand {
                target: invocation.context().clone(),
                command: text::edit::Command::Undo,
                label: "undo",
            }),
    );
    cx.register_action(
        Action::new(action::REDO, "Redo")
            .with_shortcut(action::Shortcut::control_shift('z'))
            .with_shortcut(action::Shortcut::control('y'))
            .emit(|invocation| AppEvent::ApplyCommand {
                target: invocation.context().clone(),
                command: text::edit::Command::Redo,
                label: "redo",
            }),
    );
    cx.register_action(
        Action::new(action::SELECT_ALL, "Select All")
            .with_shortcut(action::Shortcut::control('a'))
            .emit(|invocation| AppEvent::ApplyCommand {
                target: invocation.context().clone(),
                command: text::edit::Command::SelectAll,
                label: "select all",
            }),
    );
    cx.register_action(
        Action::new(action::CUT, "Cut")
            .with_shortcut(action::Shortcut::control('x'))
            .emit(|invocation| AppEvent::ApplyCommand {
                target: invocation.context().clone(),
                command: text::edit::Command::Cut,
                label: "cut",
            }),
    );
    cx.register_action(
        Action::new(action::COPY, "Copy")
            .with_shortcut(action::Shortcut::control('c'))
            .emit(|invocation| AppEvent::ApplyCommand {
                target: invocation.context().clone(),
                command: text::edit::Command::Copy,
                label: "copy",
            }),
    );
    cx.register_action(
        Action::new(action::PASTE, "Paste")
            .with_shortcut(action::Shortcut::control('v'))
            .emit(|invocation| AppEvent::ApplyCommand {
                target: invocation.context().clone(),
                command: text::edit::Command::Paste,
                label: "paste",
            }),
    );
    cx.register_action(
        Action::new(DELETE_TEXT, "Delete").emit(|invocation| AppEvent::ApplyEdit {
            target: invocation.context().clone(),
            edit: text::edit::Edit::delete(),
            label: "delete",
        }),
    );
}

fn register_view_actions(cx: &mut app::Context<'_, AppEvent>) {
    cx.register_action(
        Action::new(TOGGLE_WRAP_TEXT, "Wrap text").emit(|_| AppEvent::ToggleWrapText),
    );
    cx.register_action(
        Action::new(TOGGLE_DEBUG_PANEL, "Debug panel").emit(|_| AppEvent::ToggleDebugPanel),
    );
}

fn root_view(theme: &Theme, editor: &Editor, diagnostics: app::Diagnostics) -> ui::Node {
    let app_shell = ui::Node::container(APP_SHELL, layout::Axis::Vertical)
        .with_size(layout::Size::Fill, layout::Size::Fill)
        .with_child(widget::menu_bar_with_theme(MENU_BAR, notepad_menu(), theme))
        .with_child(editor_area(theme, editor));

    let root = ui::Node::new(ROOT)
        .with_background(theme.surfaces().app())
        .with_child(app_shell);

    if editor.show_debug_panel {
        root.with_child(debug_overlay(theme, editor, diagnostics))
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

    ui::Node::container(EDITOR_CANVAS, layout::Axis::Vertical)
        .with_background(theme.surfaces().canvas())
        .with_size(layout::Size::Fill, layout::Size::Fill)
        .with_child(
            widget::text_area_surface_with_theme(EDITOR, area, theme)
                .with_responder_binding(action::Binding::new(DELETE_TEXT).enabled(true))
                .with_size(layout::Size::Fill, layout::Size::Fill),
        )
}

fn debug_overlay(theme: &Theme, editor: &Editor, diagnostics: app::Diagnostics) -> ui::Node {
    ui::Node::container(DEBUG_OVERLAY, layout::Axis::Vertical)
        .with_size(layout::Size::Fill, layout::Size::Fill)
        .with_padding(layout::Insets::splat(theme.density().app_padding() * 2.0))
        .with_align(layout::Align::End)
        .with_cross_align(layout::Align::Stretch)
        .with_child(debug_panel(theme, editor, diagnostics))
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
    let storage = if editor.path.is_some() {
        "mapped original + owned edits"
    } else {
        "owned buffer"
    };
    let wrap = if editor.wrap_text { "on" } else { "off" };

    widget::floating_panel_with_theme(STATUS, theme)
        .with_size(layout::Size::Fill, layout::Size::Fit)
        .with_child(
            ui::Node::leaf(STATUS_TEXT)
                .with_size(layout::Size::Fill, layout::Size::Fit)
                .with_label(document(
            format!(
                "File: {path} ({dirty}) | Storage: {storage} | Wrap: {wrap}\nDocument: {} lines, {} bytes | Text edits: {} | Status: {}\nLast frame text: paint {}, metrics {}, line cache {}/{}, shaped lines {}, visible {}, layout {}, overscan {}, paint surfaces {}, highlight scans {}\nLast frame scroll: wheel {}, thumb {}, offsets {}, queued {}, redraws {}, commits {}, pending {}/{}, async {}/{}, projections {}, text resolves/reuses {}/{}, shifts {}/{}, cold {}, idle {}/{}, layers hits {}, items {}, text skips {}, chrome {}, fallbacks {}, miss {}/{}/{}/{}/{}, rebuilds {}\nLast scroll gesture: wheel {}, thumb {}, offsets {}, layer hits {}, text skips {}, fallbacks {}, miss {}/{}/{}/{}/{}, rebuilds {}\nLast scroll frame: text {}us (scene {}, updates {}), render {}us, total {}us, surfaces {}, glyph batches {}, layers {}/{}\nFrames: full {}, scroll-only {} | invalidations {}/{}, native {}, clean skips {}, no-dirty {}, fallbacks {}, render skips {}\nFrame us latest/avg: compose {}/{}, commit {}/{}, sync {}/{}, paint {}/{}, render {}/{}, total {}/{}, input-present {}/{}, dirty-present {}/{}\nRender us latest/avg: acquire {}/{}, batch {}/{}, quads {}/{}, text {}/{} (scene {}/{}, updates {}/{}), backdrop {}/{}, submit {}/{}\nRender stats: scene {}, batches {}, glyph batches {}, text surfaces {}, quad vertices {}, clips {}, backdrops {}, layers {}/{}",
                editor.buffer.logical_line_count(),
                editor.buffer.len(),
                editor.edit_count,
                editor.last_status,
                text.text_area_paint_layout_calls,
                text.text_area_metrics_layout_calls,
                text.text_area_line_cache_hits,
                text.text_area_line_cache_misses,
                text.text_area_shaped_logical_lines,
                text.text_area_visible_logical_lines,
                text.text_area_layout_segments,
                text.text_area_overscan_segments,
                text.text_area_paint_surfaces,
                text.highlight_run_scans,
                scroll.wheel_events,
                scroll.thumb_drag_moves,
                scroll.scroll_offset_changes,
                scroll.queued_scroll_updates,
                scroll.scroll_redraw_requests,
                scroll.frame_scroll_commits,
                scroll.pending_scroll_applications,
                scroll.pending_scroll_updates,
                scroll.async_scroll_projection_sync_skips,
                scroll.async_scroll_reconciles,
                scroll.projection_count,
                scroll.text_area_resolves,
                scroll.text_area_projection_reuses,
                scroll.text_area_projection_shifts,
                scroll.text_area_projection_shift_misses,
                scroll.text_area_projection_cold_jumps,
                scroll.text_area_idle_refinements,
                scroll.text_area_idle_refinements_suppressed,
                scroll.retained_scroll_layer_hits,
                scroll.retained_scroll_layer_replaced_items,
                scroll.retained_scroll_layer_text_prepare_skips,
                scroll.retained_scroll_chrome_repaints,
                scroll.retained_scroll_target_repaint_fallbacks,
                scroll.retained_scroll_layer_missing,
                scroll.retained_scroll_layer_metrics_misses,
                scroll.retained_scroll_layer_coverage_misses,
                scroll.retained_scroll_layer_geometry_misses,
                scroll.retained_scroll_layer_projection_misses,
                scroll.retained_scroll_layer_rebuilds,
                scroll.last_scroll.wheel_events,
                scroll.last_scroll.thumb_drag_moves,
                scroll.last_scroll.scroll_offset_changes,
                scroll.last_scroll.retained_scroll_layer_hits,
                scroll.last_scroll.retained_scroll_layer_text_prepare_skips,
                scroll.last_scroll.retained_scroll_target_repaint_fallbacks,
                scroll.last_scroll.retained_scroll_layer_missing,
                scroll.last_scroll.retained_scroll_layer_metrics_misses,
                scroll.last_scroll.retained_scroll_layer_coverage_misses,
                scroll.last_scroll.retained_scroll_layer_geometry_misses,
                scroll.last_scroll.retained_scroll_layer_projection_misses,
                scroll.last_scroll.retained_scroll_layer_rebuilds,
                frame.last_scroll_frame.render_text_prepare_us,
                frame.last_scroll_frame.render_scene_text_prepare_us,
                frame.last_scroll_frame.render_layer_update_text_prepare_us,
                frame.last_scroll_frame.render_total_us,
                frame.last_scroll_frame.total_us,
                frame.last_scroll_frame.text_surfaces,
                frame.last_scroll_frame.glyph_batches,
                frame.last_scroll_frame.layer_items,
                frame.last_scroll_frame.layer_updates,
                frame.full_redraws,
                frame.scroll_only_redraws,
                frame.invalidations,
                frame.coalesced_invalidations,
                frame.native_redraw_requests,
                frame.clean_redraw_skips,
                frame.redraw_events_without_dirty,
                frame.scroll_only_fallbacks_to_full,
                frame.render_skips,
                frame.compose.latest_us,
                frame.compose.average_us,
                frame.scroll_commit.latest_us,
                frame.scroll_commit.average_us,
                frame.scroll_projection_sync.latest_us,
                frame.scroll_projection_sync.average_us,
                frame.paint.latest_us,
                frame.paint.average_us,
                frame.render.latest_us,
                frame.render.average_us,
                frame.total.latest_us,
                frame.total.average_us,
                frame.scroll_input_to_present.latest_us,
                frame.scroll_input_to_present.average_us,
                frame.dirty_to_present.latest_us,
                frame.dirty_to_present.average_us,
                frame.render_acquire.latest_us,
                frame.render_acquire.average_us,
                frame.render_batching.latest_us,
                frame.render_batching.average_us,
                frame.render_quad_prepare.latest_us,
                frame.render_quad_prepare.average_us,
                frame.render_text_prepare.latest_us,
                frame.render_text_prepare.average_us,
                frame.render_scene_text_prepare.latest_us,
                frame.render_scene_text_prepare.average_us,
                frame.render_layer_update_text_prepare.latest_us,
                frame.render_layer_update_text_prepare.average_us,
                frame.render_backdrop_prepare.latest_us,
                frame.render_backdrop_prepare.average_us,
                frame.render_encode_submit.latest_us,
                frame.render_encode_submit.average_us,
                frame.scene_items.latest,
                frame.render_batches.latest,
                frame.glyph_batches.latest,
                frame.text_surfaces.latest,
                frame.quad_vertices.latest,
                frame.clip_batches.latest,
                frame.backdrops.latest,
                frame.layer_items.latest,
                frame.layer_updates.latest,
            ),
            text::document::Align::Center,
            theme.text().body_size(),
            theme.text().secondary(),
                )),
        )
}

fn notepad_menu() -> widget::menu::Bar {
    widget::menu::Bar::new()
        .menu(
            widget::Menu::new(FILE_MENU, "File").section(
                widget::menu::Section::new()
                    .item(widget::menu::Item::new(NEW_FILE))
                    .item(widget::menu::Item::new(OPEN_FILE))
                    .item(widget::menu::Item::new(SAVE_FILE))
                    .item(widget::menu::Item::new(SAVE_AS_FILE))
                    .separator()
                    .item(widget::menu::Item::new(LOAD_STRESS_TEXT))
                    .separator()
                    .item(widget::menu::Item::new(EXIT_APP)),
            ),
        )
        .menu(
            widget::Menu::new(EDIT_MENU, "Edit").section(
                widget::menu::Section::new()
                    .item(widget::menu::Item::new(action::UNDO))
                    .item(widget::menu::Item::new(action::REDO))
                    .separator()
                    .item(widget::menu::Item::new(action::CUT))
                    .item(widget::menu::Item::new(action::COPY))
                    .item(widget::menu::Item::new(action::PASTE))
                    .item(widget::menu::Item::new(DELETE_TEXT))
                    .separator()
                    .item(widget::menu::Item::new(action::SELECT_ALL)),
            ),
        )
        .menu(
            widget::Menu::new(VIEW_MENU, "View").section(
                widget::menu::Section::new()
                    .item(widget::menu::Item::new(TOGGLE_WRAP_TEXT))
                    .item(widget::menu::Item::new(TOGGLE_DEBUG_PANEL)),
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

fn stress_text(lines: usize) -> String {
    (0..lines)
        .map(|line| {
            format!(
                "line {line:05}: stress text for scrolling, editing, wrapping, selection, and undo/redo responsiveness"
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
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

fn editor_path() -> ui::Path {
    ui::Path::new([ROOT, APP_SHELL, EDITOR_CANVAS, EDITOR])
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
