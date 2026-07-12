use super::{
    Clipboard, Command, Context, Diagnostics, Document as TextDocument, Error, Host, Input,
    Platform, Response, Runtime, Scene, Session, Shell, State, Target, Task, Theme, Timeline, View,
    clipboard, command, composition, context, control_gallery, diagnostics, document, draft,
    geometry, glass_tuner, host, ime, input, interaction, keymap, layout, notification, overlay,
    platform, pointer, responder, response, runtime, scene, session, shell, state, subject, task,
    text_editor, timeline, view, widget, window,
};
use crate::interaction::Interaction;
use crate::text;
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

struct Save;

impl Command for Save {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "app.save";
}

struct Ping;

impl Command for Ping {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "app.ping";
}

struct IgnoredPing;

impl Command for IgnoredPing {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "app.ignored_ping";
    const HISTORY: command::History = command::History::Ignored;
}

struct IgnoredMutation;

impl Command for IgnoredMutation {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "app.ignored_mutation";
    const HISTORY: command::History = command::History::Ignored;
}

struct RecordSource;

impl Command for RecordSource {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "app.record_source";
}

struct HiddenRecordSource;

impl Command for HiddenRecordSource {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "app.hidden_record_source";
}

struct DisabledRecordSource;

impl Command for DisabledRecordSource {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "app.disabled_record_source";
}

struct SpawnEditorEvent;

impl Command for SpawnEditorEvent {
    type Args = EditorEvent;
    type Output = Option<task::Id>;

    const NAME: &'static str = "app.spawn_editor_event";
}

struct OpenNamed;

impl Command for OpenNamed {
    type Args = String;
    type Output = usize;

    const NAME: &'static str = "app.open_named";
}

struct SetLevel;

impl Command for SetLevel {
    type Args = f64;
    type Output = f64;

    const NAME: &'static str = "app.set_level";
}

struct SetMappedLevel;

impl Command for SetMappedLevel {
    type Args = LevelArgs;
    type Output = ();

    const NAME: &'static str = "app.set_mapped_level";
}

struct SubmitText;

impl Command for SubmitText {
    type Args = String;
    type Output = String;

    const NAME: &'static str = "app.submit_text";
    const HISTORY: command::History = command::History::Ignored;
}

struct SubmitMappedText;

impl Command for SubmitMappedText {
    type Args = TextSubmitArgs;
    type Output = ();

    const NAME: &'static str = "app.submit_mapped_text";
}

#[derive(Clone)]
struct LevelArgs {
    raw: f64,
    snapped: i32,
}

#[derive(Clone)]
struct TextSubmitArgs {
    raw: String,
    normalized: String,
}

#[derive(Clone, Copy)]
enum EditorEvent {
    Edited,
    Saved,
}

#[derive(Clone, Default)]
struct EditorState {
    document: SaveDocument,
    project: Project,
    wrap_text: bool,
    event_count: usize,
}

impl State for EditorState {}

#[derive(Clone, Default)]
struct MultiDocumentState {
    first: SaveDocument,
    second: SaveDocument,
}

impl State for MultiDocumentState {}

#[derive(Clone, Default)]
struct SourceState {
    sources: Vec<context::Source>,
}

impl State for SourceState {}

impl Target<crate::table::SortBy> for SourceState {
    fn state(&self, _: &crate::table::SortIntent, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: crate::table::SortIntent, _: &mut Context) -> Response<()> {
        Response::output(())
    }
}

#[derive(Default)]
struct CloneCountState {
    clone_count: Rc<Cell<usize>>,
    value: usize,
}

impl CloneCountState {
    fn count(&self) -> Rc<Cell<usize>> {
        Rc::clone(&self.clone_count)
    }
}

impl Clone for CloneCountState {
    fn clone(&self) -> Self {
        self.clone_count.set(self.clone_count.get() + 1);
        Self {
            clone_count: Rc::clone(&self.clone_count),
            value: self.value,
        }
    }
}

impl State for CloneCountState {}

#[derive(Clone, Default)]
struct SliderValueState {
    value: f64,
    invocations: usize,
}

impl State for SliderValueState {}

#[derive(Clone, Default)]
struct MappedSliderState {
    raw: f64,
    snapped: i32,
}

impl State for MappedSliderState {}

#[derive(Clone, Default)]
struct TextBoxSubmitState {
    submitted: String,
    normalized: String,
    source: Option<context::Source>,
}

impl State for TextBoxSubmitState {}

#[derive(Clone, Default)]
struct HiddenSaveState {
    passive: PassivePane,
    project: Project,
}

impl State for HiddenSaveState {}

#[derive(Default)]
struct EditorPersistence {
    data: Option<String>,
    fail_save: bool,
}

#[derive(Default)]
struct FakeBackend {
    events: Vec<BackendEvent>,
    popup_sync_counts: Vec<usize>,
    native_popups: bool,
    fail_open: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum BackendEvent {
    OpenWindow {
        id: window::Id,
        title: String,
        size: geometry::Size,
        canvas_color: scene::Color,
        kind: window::Kind,
    },
    CloseWindow {
        id: window::Id,
    },
    Present {
        window: window::Id,
        size: geometry::Size,
        clear_color: scene::Color,
    },
    PresentPopup {
        parent: window::Id,
        id: interaction::Id,
        size: geometry::Size,
        clear_color: scene::Color,
        framework_glass_panes: usize,
        material_regions: usize,
    },
    FileDialog {
        window: window::Id,
        kind: session::RequestKind,
    },
    SetCursor {
        window: window::Id,
        cursor: pointer::Cursor,
    },
    SetIme {
        update: ime::Update,
    },
    Poll,
}

impl FakeBackend {
    fn events(&self) -> &[BackendEvent] {
        &self.events
    }

    fn popup_sync_counts(&self) -> &[usize] {
        &self.popup_sync_counts
    }

    fn clear_popup_sync_counts(&mut self) {
        self.popup_sync_counts.clear();
    }

    fn with_native_popups(mut self) -> Self {
        self.native_popups = true;
        self
    }

    fn failing_open(mut self) -> Self {
        self.fail_open = true;
        self
    }
}

impl platform::Backend for FakeBackend {
    type Error = &'static str;
    type Context<'a> = ();

    fn open_window(
        &mut self,
        _context: &mut Self::Context<'_>,
        window: &platform::Window,
    ) -> Result<(), Self::Error> {
        if self.fail_open {
            return Err("open failed");
        }
        self.events.push(BackendEvent::OpenWindow {
            id: window.id(),
            title: window.title().to_owned(),
            size: window.size(),
            canvas_color: window.canvas_color(),
            kind: window.kind(),
        });
        Ok(())
    }

    fn close_window(
        &mut self,
        _context: &mut Self::Context<'_>,
        window: window::Id,
    ) -> Result<(), Self::Error> {
        self.events.push(BackendEvent::CloseWindow { id: window });
        Ok(())
    }

    fn present(
        &mut self,
        _context: &mut Self::Context<'_>,
        presentation: &shell::Presentation,
    ) -> Result<diagnostics::RenderReport, Self::Error> {
        self.events.push(BackendEvent::Present {
            window: presentation.window(),
            size: presentation.size(),
            clear_color: presentation.scene().clear(),
        });
        Ok(diagnostics::RenderReport::new(
            Duration::from_micros(10),
            Duration::from_micros(20),
            Instant::now(),
        ))
    }

    fn overlay_capabilities(&self) -> overlay::Capabilities {
        if self.native_popups {
            overlay::Capabilities::with_native_popups()
        } else {
            overlay::Capabilities::in_frame_only()
        }
    }

    fn present_overlay_popups(
        &mut self,
        _context: &mut Self::Context<'_>,
        _synchronized_parents: &[window::Id],
        presentations: &[overlay::PopupPresentation],
    ) -> Result<(), Self::Error> {
        self.popup_sync_counts.push(presentations.len());
        for presentation in presentations {
            self.events.push(BackendEvent::PresentPopup {
                parent: presentation.parent(),
                id: presentation.id(),
                size: presentation.scene().size(),
                clear_color: presentation.scene().clear(),
                framework_glass_panes: framework_glass_pane_count(presentation.scene()),
                material_regions: presentation.scene().material_regions().len(),
            });
        }
        Ok(())
    }

    fn request(
        &mut self,
        _context: &mut Self::Context<'_>,
        request: session::Request,
    ) -> Result<(), Self::Error> {
        self.events.push(BackendEvent::FileDialog {
            window: request.window(),
            kind: request.kind(),
        });
        Ok(())
    }

    fn set_cursor(
        &mut self,
        _context: &mut Self::Context<'_>,
        window: window::Id,
        cursor: pointer::Cursor,
    ) -> Result<(), Self::Error> {
        self.events.push(BackendEvent::SetCursor { window, cursor });
        Ok(())
    }

    fn set_ime(
        &mut self,
        _context: &mut Self::Context<'_>,
        update: ime::Update,
    ) -> Result<(), Self::Error> {
        self.events.push(BackendEvent::SetIme { update });
        Ok(())
    }

    fn schedule_poll(&mut self, _context: &mut Self::Context<'_>) -> Result<(), Self::Error> {
        self.events.push(BackendEvent::Poll);
        Ok(())
    }
}

#[derive(Clone, Default)]
struct SaveDocument {
    dirty: bool,
    save_count: usize,
}

impl Target<Save> for SaveDocument {
    fn state(&self, _: &(), _: &Context) -> command::State {
        if self.dirty {
            command::State::enabled()
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.dirty = false;
        self.save_count += 1;
        Response::changed(()).with_effect(response::Effect::Rebuild)
    }
}

#[derive(Clone, Default)]
struct Project {
    dirty: bool,
    save_count: usize,
}

impl Target<Save> for Project {
    fn state(&self, _: &(), _: &Context) -> command::State {
        if self.dirty {
            command::State::enabled().with_label("Save Project")
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.dirty = false;
        self.save_count += 1;
        Response::changed(()).with_effect(response::Effect::Rebuild)
    }
}

impl Target<Save> for EditorState {
    fn state(&self, _: &(), _: &Context) -> command::State {
        if self.project.dirty {
            command::State::enabled().with_label("Save App")
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.project.dirty = false;
        self.project.save_count += 1;
        Response::changed(())
    }
}

#[derive(Clone, Default)]
struct PassivePane;

impl Target<Save> for PassivePane {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::hidden()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        unreachable!("hidden targets are not invoked")
    }
}

impl Target<Save> for HiddenSaveState {
    fn state(&self, _: &(), _: &Context) -> command::State {
        if self.project.dirty {
            command::State::enabled().with_label("Save Project")
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.project.dirty = false;
        self.project.save_count += 1;
        Response::changed(())
    }
}

impl Target<text_editor::ToggleWrapText> for EditorState {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled().checked(self.wrap_text)
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.wrap_text = !self.wrap_text;
        Response::changed(())
    }
}

impl Target<Ping> for EditorState {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        Response::output(())
    }
}

impl Target<Ping> for CloneCountState {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        Response::output(())
    }
}

impl Target<IgnoredPing> for CloneCountState {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        Response::output(())
    }
}

impl Target<IgnoredMutation> for CloneCountState {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.value += 1;
        Response::changed(())
    }
}

impl Target<timeline::Undo> for CloneCountState {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.value += 1;
        Response::changed(())
    }
}

impl Target<RecordSource> for SourceState {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), cx: &mut Context) -> Response<()> {
        self.sources.push(cx.source());
        Response::changed(())
    }
}

impl Target<HiddenRecordSource> for SourceState {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::hidden()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        unreachable!("hidden command should not be invokable through presentation")
    }
}

impl Target<DisabledRecordSource> for SourceState {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::disabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.sources.push(context::Source::Button);
        Response::changed(())
    }
}

impl Target<SpawnEditorEvent> for EditorState {
    fn state(&self, _: &EditorEvent, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, event: EditorEvent, cx: &mut Context) -> Response<Option<task::Id>> {
        Response::output(cx.spawn(Task::ready(event)))
    }
}

impl Target<OpenNamed> for EditorState {
    fn state(&self, _: &String, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, name: String, _: &mut Context) -> Response<usize> {
        self.event_count += name.len();
        Response::changed(name.len())
    }
}

impl Target<OpenNamed> for CloneCountState {
    fn state(&self, _: &String, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, name: String, _: &mut Context) -> Response<usize> {
        self.value += name.len();
        Response::changed(self.value)
    }
}

impl Target<SetLevel> for SliderValueState {
    fn state(&self, _: &f64, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, value: f64, _: &mut Context) -> Response<f64> {
        self.value = value;
        self.invocations += 1;
        Response::changed(value)
    }
}

impl Target<SetMappedLevel> for MappedSliderState {
    fn state(&self, _: &LevelArgs, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, args: LevelArgs, _: &mut Context) -> Response<()> {
        self.raw = args.raw;
        self.snapped = args.snapped;
        Response::changed(())
    }
}

impl Target<SubmitText> for TextBoxSubmitState {
    fn state(&self, _: &String, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, text: String, cx: &mut Context) -> Response<String> {
        self.source = Some(cx.source());
        self.submitted = text.clone();
        Response::changed(text)
    }
}

impl Target<SubmitMappedText> for TextBoxSubmitState {
    fn state(&self, _: &TextSubmitArgs, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, args: TextSubmitArgs, cx: &mut Context) -> Response<()> {
        self.source = Some(cx.source());
        self.submitted = args.raw;
        self.normalized = args.normalized;
        Response::changed(())
    }
}

impl runtime::Persistence<EditorState> for EditorPersistence {
    type Error = &'static str;

    fn save(&mut self, snapshot: &runtime::Snapshot<EditorState>) -> Result<(), Self::Error> {
        if self.fail_save {
            return Err("save failed");
        }

        let state = snapshot.state().model();
        let windows = snapshot
            .session()
            .windows()
            .iter()
            .map(|window| {
                format!(
                    "{}:{}:{}",
                    window.id().get(),
                    window.title(),
                    encode_focus(window.focus())
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        self.data = Some(format!(
            "{}|{}|{windows}",
            state.event_count, state.wrap_text
        ));
        Ok(())
    }

    fn load(&mut self) -> Result<runtime::Snapshot<EditorState>, Self::Error> {
        let data = self.data.as_deref().ok_or("missing snapshot")?;
        let mut fields = data.splitn(3, '|');
        let event_count = fields
            .next()
            .ok_or("missing event count")?
            .parse::<usize>()
            .map_err(|_| "invalid event count")?;
        let wrap_text = fields
            .next()
            .ok_or("missing wrap flag")?
            .parse::<bool>()
            .map_err(|_| "invalid wrap flag")?;
        let windows = fields.next().unwrap_or_default();

        let state = state::Snapshot::from_model(EditorState {
            event_count,
            wrap_text,
            ..EditorState::default()
        });
        let session = session::Snapshot::from_windows(parse_windows(windows)?);

        Ok(runtime::Snapshot::new(state, session))
    }
}
mod architecture;
mod commands;
mod composition_tests;
mod document_editor;
mod host_adapter_tests;
mod host_shell_tests;
mod interaction_tests;
mod layout_scene;
mod notifications;
mod platform_tests;
mod responder_tests;
mod runtime_tests;
mod text_input;
mod widget_binding_tests;
mod widget_composition_tests;
mod widget_focus_tests;
mod widget_identity_tests;
mod widget_layout_tests;
mod widget_slider_tests;
mod widget_text_box_tests;
fn open_view_menu_and_wrap_command_point(
    app: &mut Runtime<text_editor::State, text_editor::Event, View>,
    window: window::Id,
    size: geometry::Size,
) -> geometry::Point {
    let view_menu_point = {
        let presentation = app
            .render_scene(window, size)
            .expect("initial scene should install a composition");
        let view_menu = presentation
            .layout()
            .find_role(view::Role::Menu)
            .into_iter()
            .find(|frame| frame.label_text() == Some("View"))
            .expect("view menu should be laid out");

        frame_point(view_menu)
    };

    app.pointer_down_at(window, size, view_menu_point)
        .expect("view menu pointer down should be handled");
    app.pointer_up_at(window, size, view_menu_point)
        .expect("view menu pointer up should be handled");

    let presentation = app
        .render_scene(window, size)
        .expect("open view menu should install a composition");
    let wrap_command = presentation
        .layout()
        .find_role(view::Role::Binding)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Wrap text"))
        .expect("wrap text command should be laid out");

    frame_point(wrap_command)
}

fn frame_point(frame: &layout::Frame) -> geometry::Point {
    let rect = frame.rect();
    geometry::Point::new(rect.x() + 1, rect.y() + 1)
}

fn pointer_down_then_present<M, E>(
    app: &mut Runtime<M, E, View>,
    window: window::Id,
    size: geometry::Size,
    point: geometry::Point,
) -> input::Outcome
where
    M: State,
    E: Send + 'static,
{
    let outcome = app
        .pointer_down_at(window, size, point)
        .expect("pointer down should be handled");
    app.render_scene(window, size)
        .expect("native loop presents after pointer down");
    outcome
}

fn pointer_move_then_present<M, E>(
    app: &mut Runtime<M, E, View>,
    window: window::Id,
    size: geometry::Size,
    point: geometry::Point,
) -> input::Outcome
where
    M: State,
    E: Send + 'static,
{
    let outcome = app
        .pointer_move_at(window, size, point)
        .expect("pointer move should be handled");
    app.render_scene(window, size)
        .expect("native loop presents after pointer move");
    outcome
}

fn pointer_up_then_present<M, E>(
    app: &mut Runtime<M, E, View>,
    window: window::Id,
    size: geometry::Size,
    point: geometry::Point,
) -> input::Outcome
where
    M: State,
    E: Send + 'static,
{
    let outcome = app
        .pointer_up_at(window, size, point)
        .expect("pointer up should be handled");
    app.render_scene(window, size)
        .expect("native loop presents after pointer up");
    outcome
}

fn rect_contains(bounds: geometry::Rect, rect: geometry::Rect) -> bool {
    rect.x() >= bounds.x()
        && rect.y() >= bounds.y()
        && rect.x().saturating_add(rect.width()) <= bounds.x().saturating_add(bounds.width())
        && rect.y().saturating_add(rect.height()) <= bounds.y().saturating_add(bounds.height())
}

fn framework_glass_pane_count(scene: &scene::Scene) -> usize {
    scene
        .panes()
        .into_iter()
        .filter(|pane| match pane.material() {
            scene::Material::Glass(_) => true,
            scene::Material::Solid(_) => false,
        })
        .count()
}

fn assert_near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.0001,
        "expected {actual} to be near {expected}"
    );
}

fn text_area_node(node: &view::Node) -> Option<&view::Node> {
    if node.text_area_model().is_some() {
        return Some(node);
    }

    node.children().iter().find_map(text_area_node)
}

fn text_box_node(node: &view::Node) -> Option<&view::Node> {
    if node.text_box_model().is_some() {
        return Some(node);
    }

    node.children().iter().find_map(text_box_node)
}

fn text_draft<M, E, V>(
    app: &Runtime<M, E, V>,
    window: window::Id,
    focus: session::Focus,
) -> &draft::State
where
    M: State,
    E: Send + 'static,
{
    let target = interaction::Target::text_area(focus);
    app.session()
        .interaction(window)
        .expect("window should have interaction state")
        .text_input()
        .draft_for(&target)
        .expect("text box should have a draft")
}

fn non_text_focus(id: &'static str) -> session::Focus {
    let target = interaction::Target::label(interaction::Id::new(id), id);
    session::Focus::control(&target)
}

fn temp_text_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("wgpu_l3_{}_{}", std::process::id(), name))
}

fn encode_focus(focus: Option<session::Focus>) -> &'static str {
    match focus {
        Some(focus) => focus.target().as_str(),
        None => "-",
    }
}

fn decode_focus(focus: &str) -> Result<Option<session::Focus>, &'static str> {
    match focus {
        "-" => Ok(None),
        "document" => Ok(Some(session::Focus::text("document"))),
        _ => Err("unknown focus target"),
    }
}

fn parse_windows(windows: &str) -> Result<Vec<session::WindowSnapshot>, &'static str> {
    if windows.is_empty() {
        return Ok(Vec::new());
    }

    windows
        .split(',')
        .map(|window| {
            let mut fields = window.splitn(3, ':');
            let id = fields
                .next()
                .ok_or("missing window id")?
                .parse::<u64>()
                .map_err(|_| "invalid window id")?;
            let title = fields.next().ok_or("missing window title")?;
            let focus = decode_focus(fields.next().unwrap_or("-"))?;

            Ok(session::WindowSnapshot::new(
                window::Id::new(id),
                title,
                focus,
            ))
        })
        .collect()
}
