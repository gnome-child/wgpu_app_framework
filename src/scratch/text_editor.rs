use std::path::{Path, PathBuf};

use super::{
    Clipboard, Document as TextDocument, Response, Runtime, Shell, Target, Task, View, command,
    context::Context, document, geometry, interaction, platform, response, scene, session, state,
    timeline, view, widget, window,
};

pub struct ToggleWrapText;

impl command::Command for ToggleWrapText {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "view.toggle_wrap_text";
}

pub struct ToggleDebugPanel;

impl command::Command for ToggleDebugPanel {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "view.toggle_debug_panel";
}

pub struct LoadStressText;

impl command::Command for LoadStressText {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "document.load_stress_text";
}

const FILE_MENU: interaction::Id = interaction::Id::new("menu.file");
const EDIT_MENU: interaction::Id = interaction::Id::new("menu.edit");
const VIEW_MENU: interaction::Id = interaction::Id::new("menu.view");
const DOCUMENT_FOCUS: interaction::Id = interaction::Id::new("document");

pub enum Event {
    FileSaved {
        path: PathBuf,
        result: Result<(), String>,
    },
}

pub const STRESS_TEXT: &str =
    include_str!("../../examples/text_editor/fixtures/unicode_stress_dump.txt");
pub const WINDOW_TITLE: &str = "wgpu_l3 Notepad";
pub const CANVAS_COLOR: scene::Color = scene::Color::rgb(17, 17, 19);

pub fn window_size() -> geometry::Size {
    geometry::Size::new(920, 680)
}

#[derive(Clone)]
pub struct State {
    pub document: TextDocument,
    pub wrap_text: bool,
    pub show_debug_panel: bool,
    pub last_status: String,
}

impl super::State for State {}

impl Default for State {
    fn default() -> Self {
        Self {
            document: TextDocument::default(),
            wrap_text: true,
            show_debug_panel: false,
            last_status: "ready".to_owned(),
        }
    }
}

impl Target<ToggleWrapText> for State {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled().checked(self.wrap_text)
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.wrap_text = !self.wrap_text;
        self.last_status = if self.wrap_text {
            "wrap text enabled".to_owned()
        } else {
            "wrap text disabled".to_owned()
        };
        Response::changed(())
    }
}

impl Target<ToggleDebugPanel> for State {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled().checked(self.show_debug_panel)
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.show_debug_panel = !self.show_debug_panel;
        self.last_status = if self.show_debug_panel {
            "debug panel shown".to_owned()
        } else {
            "debug panel hidden".to_owned()
        };
        Response::changed(())
    }
}

impl Target<LoadStressText> for State {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.document.replace_unsaved_text(STRESS_TEXT);
        self.last_status = format!(
            "loaded Unicode stress fixture ({} lines)",
            STRESS_TEXT.lines().count()
        );
        Response::changed(())
    }
}

impl Target<document::NewFile> for State {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.document.new_file();
        self.last_status = "new file".to_owned();
        Response::changed(())
    }
}

impl Target<document::OpenFile> for State {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.last_status = "choosing file".to_owned();
        Response::changed(()).with_effect(response::Effect::OpenFileDialog)
    }
}

impl Target<document::OpenPath> for State {
    fn state(&self, _: &PathBuf, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, path: PathBuf, _: &mut Context) -> Response<Result<(), String>> {
        match self.document.open_path(path.clone()) {
            Ok(()) => {
                self.last_status = format!("opened {}", compact_path(&path));
                Response::changed(Ok(()))
            }
            Err(error) => {
                self.last_status = format!("open failed: {error}");
                Response::changed(Err(error.to_string()))
            }
        }
    }
}

impl Target<document::OpenCanceled> for State {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.last_status = "open canceled".to_owned();
        Response::changed(())
    }
}

impl Target<document::SaveFile> for State {
    fn state(&self, _: &(), _: &Context) -> command::State {
        if self.document.is_dirty() {
            command::State::enabled()
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), cx: &mut Context) -> Response<Result<(), String>> {
        let Some(path) = self.document.path().map(Path::to_path_buf) else {
            self.last_status = "choosing save location".to_owned();
            return Response::changed(Ok(())).with_effect(response::Effect::SaveFileDialog);
        };

        queue_save(self, path, cx)
    }
}

impl Target<document::SaveAsFile> for State {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.last_status = "choosing save location".to_owned();
        Response::changed(()).with_effect(response::Effect::SaveFileDialog)
    }
}

impl Target<document::SaveToPath> for State {
    fn state(&self, _: &PathBuf, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, path: PathBuf, cx: &mut Context) -> Response<Result<(), String>> {
        queue_save(self, path, cx)
    }
}

impl Target<document::SaveCanceled> for State {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.last_status = "save canceled".to_owned();
        Response::changed(())
    }
}

pub fn runtime(state: State) -> Runtime<State, Event> {
    Runtime::new(state)
        .commands(|commands| {
            commands
                .register::<document::ApplyEdit>(command::Spec::new("Edit"))
                .register::<document::NewFile>(command::Spec::new("New").shortcut("Ctrl+N"))
                .register::<document::OpenFile>(command::Spec::new("Open").shortcut("Ctrl+O"))
                .register::<document::OpenPath>(command::Spec::new("Open Path"))
                .register::<document::OpenCanceled>(command::Spec::new("Open Canceled"))
                .register::<document::SaveFile>(command::Spec::new("Save").shortcut("Ctrl+S"))
                .register::<document::SaveAsFile>(
                    command::Spec::new("Save As").shortcut("Ctrl+Shift+S"),
                )
                .register::<document::SaveToPath>(command::Spec::new("Save To Path"))
                .register::<document::SaveCanceled>(command::Spec::new("Save Canceled"))
                .register::<document::Cut>(command::Spec::new("Cut").shortcut("Ctrl+X"))
                .register::<document::Copy>(command::Spec::new("Copy").shortcut("Ctrl+C"))
                .register::<document::Paste>(command::Spec::new("Paste").shortcut("Ctrl+V"))
                .register::<document::Delete>(command::Spec::new("Delete"))
                .register::<document::SelectAll>(
                    command::Spec::new("Select All").shortcut("Ctrl+A"),
                )
                .register::<LoadStressText>(command::Spec::new("Load Stress Text"))
                .register::<ToggleWrapText>(command::Spec::new("Wrap text"))
                .register::<ToggleDebugPanel>(command::Spec::new("Debug panel"));
        })
        .responders(|responders| {
            responders
                .app()
                .target::<document::NewFile>()
                .target::<document::OpenFile>()
                .target::<document::OpenPath>()
                .target::<document::OpenCanceled>()
                .target::<document::SaveFile>()
                .target::<document::SaveAsFile>()
                .target::<document::SaveToPath>()
                .target::<document::SaveCanceled>()
                .target::<LoadStressText>()
                .target::<ToggleWrapText>()
                .target::<ToggleDebugPanel>();
            responders
                .object("document", |state: &mut State| &mut state.document)
                .target::<document::ApplyEdit>()
                .target::<document::Cut>()
                .target::<document::Copy>()
                .target::<document::Paste>()
                .target::<document::Delete>()
                .target::<document::SelectAll>();
        })
        .observe::<document::ApplyEdit>(record_apply_edit_status)
        .observe::<document::SelectAll>(|state, result, observation| {
            record_text_command_status(state, result, "select all", observation);
        })
        .observe::<document::Copy>(|state, result, observation| {
            record_text_command_status(state, result, "copy", observation);
        })
        .observe::<document::Cut>(|state, result, observation| {
            record_text_command_status(state, result, "cut", observation);
        })
        .observe::<document::Delete>(|state, result, observation| {
            record_text_command_status(state, result, "delete", observation);
        })
        .observe::<document::Paste>(|state, result, observation| {
            record_text_command_status(state, result, "paste", observation);
        })
        .event(|cx, event| match event {
            Event::FileSaved { path, result } => {
                cx.change(state::Reason::event("file_saved"), |state| {
                    finish_save(state, path, result);
                });
            }
        })
}

pub fn app(state: State) -> Runtime<State, Event, View> {
    runtime(state)
        .started(|cx| {
            cx.open_window(
                window::Options::new(WINDOW_TITLE)
                    .with_inner_size(window_size())
                    .with_canvas_color(CANVAS_COLOR),
            );
        })
        .view(view)
}

pub fn shell(state: State) -> Shell<State, Event> {
    Shell::new(app(state))
}

pub fn native_shell(state: State) -> Shell<State, Event> {
    Shell::new(app(state).with_clipboard(Clipboard::system()))
}

pub fn runner(state: State) -> platform::Runner<State, Event> {
    platform::Runner::new(native_shell(state))
}

pub fn run(state: State) -> Result<(), platform::RunError<platform::NativeError>> {
    runner(state).run()
}

pub fn view(state: &State, cx: view::Context) -> View {
    let wrap = if state.wrap_text {
        view::Wrap::Word
    } else {
        view::Wrap::None
    };
    let debug_panel = if state.show_debug_panel {
        let dirty = if state.document.is_dirty() {
            "modified"
        } else {
            "saved"
        };
        let path = state
            .document
            .path()
            .map(compact_path)
            .unwrap_or_else(|| "Untitled".to_owned());
        let wrap = if state.wrap_text { "on" } else { "off" };
        let diagnostics = cx.diagnostics();
        let text_diagnostics = &diagnostics.text;
        let scroll = &diagnostics.scroll;
        let frame = &diagnostics.frame;
        let status = format!(
            "File: {path} ({dirty}) | Wrap: {wrap}\nDocument: {} lines, {} bytes | Edits: {} | Status: {}\nText layout: paint {}, metrics {}, visible {}, shaped {}, segments {}+{}, overlays {}, highlight scans {}\nText caches: line {}/{}, render surfaces {}, render cache {}/{}, render source {} lines / {} bytes\nScroll: wheel {}, offsets {}, queued {}, redraws {}, commits {}, pending {}/{}\nText scroll: projections {}, resolve/reuse/shift {}/{}/{}, shift misses {}, cold jumps {}, async reconcile/skips {}/{}\nRetained scroll: layer hits {}, text skips {}, target fallbacks {}, layer rebuilds {}\nFrames: full {}, scroll-only {}, scroll fallbacks {}, render skips {}\nFrame us latest/avg: paint {}/{}, render {}/{}, text {}/{}, total {}/{}\nLast scroll frame: text {}us, render {}us, total {}us, surfaces {}, glyph batches {}",
            state.document.line_count(),
            state.document.len(),
            state.document.edit_count(),
            state.last_status,
            text_diagnostics.text_area_paint_layout_calls,
            text_diagnostics.text_area_metrics_layout_calls,
            text_diagnostics.text_area_visible_logical_lines,
            text_diagnostics.text_area_shaped_logical_lines,
            text_diagnostics.text_area_layout_segments,
            text_diagnostics.text_area_overscan_segments,
            text_diagnostics.text_area_interaction_surfaces,
            text_diagnostics.highlight_run_scans,
            text_diagnostics.text_area_line_cache_hits,
            text_diagnostics.text_area_line_cache_misses,
            text_diagnostics.text_area_render_surface_calls,
            text_diagnostics.text_area_render_surface_cache_hits,
            text_diagnostics.text_area_render_surface_cache_misses,
            text_diagnostics.text_area_render_surface_source_lines,
            text_diagnostics.text_area_render_surface_source_bytes,
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
        );
        Some(
            widget::Panel::new()
                .child(widget::Label::new("Debug"))
                .child(widget::Label::new(status)),
        )
    } else {
        None
    };

    widget::view(|ui| {
        ui.column(|ui| {
            ui.menu_bar(|ui| {
                ui.menu(FILE_MENU, "File", |ui| {
                    ui.add(widget::Command::<document::NewFile>::menu());
                    ui.add(widget::Command::<document::OpenFile>::menu());
                    ui.add(widget::Command::<document::SaveFile>::menu());
                    ui.add(widget::Command::<document::SaveAsFile>::menu());
                    ui.separator();
                    ui.add(widget::Command::<LoadStressText>::menu());
                    ui.separator();
                    ui.add(widget::Command::<session::CloseWindow>::menu());
                });
                ui.menu(EDIT_MENU, "Edit", |ui| {
                    ui.add(widget::Command::<timeline::Undo>::menu());
                    ui.add(widget::Command::<timeline::Redo>::menu());
                    ui.separator();
                    ui.add(widget::Command::<document::Cut>::menu());
                    ui.add(widget::Command::<document::Copy>::menu());
                    ui.add(widget::Command::<document::Paste>::menu());
                    ui.add(widget::Command::<document::Delete>::menu());
                    ui.separator();
                    ui.add(widget::Command::<document::SelectAll>::menu());
                });
                ui.menu(VIEW_MENU, "View", |ui| {
                    ui.add(widget::Command::<ToggleWrapText>::menu());
                    ui.add(widget::Command::<ToggleDebugPanel>::menu());
                });
            });
            ui.text_area(
                widget::TextArea::from_buffer(
                    state.document.buffer().clone(),
                    state.document.edit_state(),
                )
                .wrap(wrap)
                .focus(session::Focus::text(DOCUMENT_FOCUS)),
            );
        });

        if let Some(debug_panel) = debug_panel {
            ui.add(debug_panel);
        }
    })
}

pub fn compact_path(path: &Path) -> String {
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

fn queue_save(state: &mut State, path: PathBuf, cx: &mut Context) -> Response<Result<(), String>> {
    let text = state.document.text();
    state.last_status = format!("saving {}", compact_path(&path));
    let scheduled = cx.spawn(Task::new(move || {
        let result = std::fs::write(&path, text).map_err(|error| error.to_string());
        Event::FileSaved { path, result }
    }));

    if scheduled.is_some() {
        Response::changed(Ok(()))
    } else {
        state.last_status = "save failed: task queue unavailable".to_owned();
        Response::changed(Err("task queue unavailable".to_owned()))
    }
}

fn finish_save(state: &mut State, path: PathBuf, result: Result<(), String>) {
    match result {
        Ok(()) => {
            state.document.mark_saved_at(path.clone());
            state.last_status = format!("saved {}", compact_path(&path));
        }
        Err(error) => {
            state.last_status = format!("save failed: {error}");
        }
    }
}

fn set_status(
    state: &mut State,
    status: impl Into<String>,
    observation: &mut command::Observation,
) {
    let status = status.into();
    if state.last_status != status {
        state.last_status = status;
        observation.mark_changed();
    }
}

fn record_apply_edit_status(
    state: &mut State,
    outcome: &document::Outcome,
    observation: &mut command::Observation,
) {
    let status = if outcome.text_changed() {
        "edit"
    } else if outcome.selection_changed() {
        "select all"
    } else {
        "edit"
    };

    set_status(state, status, observation);
}

fn record_text_command_status(
    state: &mut State,
    outcome: &document::Outcome,
    label: &'static str,
    observation: &mut command::Observation,
) {
    let status = if outcome.unavailable() {
        format!("{label} unavailable")
    } else {
        label.to_owned()
    };

    set_status(state, status, observation);
}
