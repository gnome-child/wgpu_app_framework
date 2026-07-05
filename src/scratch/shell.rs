use std::path::PathBuf;

use crate::paint;
use crate::text;

use super::{
    Error, geometry, input, interaction, layout, runtime, scene, session, state::State, task, view,
    window,
};

pub struct Shell<M: State, E: Send + 'static = ()> {
    runtime: runtime::Runtime<M, E, view::View>,
    windows: Vec<Window>,
}

pub struct Work {
    opened_windows: Vec<Window>,
    closed_windows: Vec<window::Id>,
    presentations: Vec<Presentation>,
    requests: Vec<session::Request>,
    pending_tasks: usize,
    task_completions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    id: window::Id,
    title: String,
    size: geometry::Size,
    canvas_color: scene::Color,
}

#[derive(Clone)]
pub struct Presentation {
    window: window::Id,
    layout: layout::Layout,
    scene: paint::Scene,
}

pub enum Event {
    Started,
    WindowResized {
        window: window::Id,
        size: geometry::Size,
    },
    RedrawRequested {
        window: window::Id,
    },
    CloseRequested {
        window: window::Id,
    },
    PointerMoved {
        window: window::Id,
        point: geometry::Point,
    },
    PointerDown {
        window: window::Id,
        point: geometry::Point,
    },
    PointerUp {
        window: window::Id,
        point: geometry::Point,
    },
    PointerLeft {
        window: window::Id,
    },
    Scrolled {
        window: window::Id,
        point: geometry::Point,
        delta: interaction::ScrollDelta,
    },
    KeyDown {
        window: window::Id,
        key: input::Key,
        modifiers: input::Modifiers,
        text: Option<String>,
    },
    TextCommitted {
        window: window::Id,
        text: String,
    },
    TextPreedit {
        window: window::Id,
        preedit: text::Preedit,
    },
    FilePathSelected {
        window: window::Id,
        path: Option<PathBuf>,
    },
    Poll,
}

#[derive(Default)]
struct WindowChanges {
    opened: Vec<Window>,
    closed: Vec<window::Id>,
}

impl<M: State, E: Send + 'static> Shell<M, E> {
    pub fn new(runtime: runtime::Runtime<M, E, view::View>) -> Self {
        Self {
            runtime,
            windows: Vec::new(),
        }
    }

    pub fn runtime(&self) -> &runtime::Runtime<M, E, view::View> {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut runtime::Runtime<M, E, view::View> {
        &mut self.runtime
    }

    pub fn into_runtime(self) -> runtime::Runtime<M, E, view::View> {
        self.runtime
    }

    pub fn start(&mut self) {
        self.runtime.start();
    }

    pub fn set_window_size(&mut self, window: window::Id, size: geometry::Size) -> bool {
        self.sync_windows();
        let Some(window) = self.windows.iter_mut().find(|entry| entry.id == window) else {
            return false;
        };

        let changed = window.size != size;
        window.size = size;
        if changed {
            self.runtime.request_redraw(window.id);
        }
        true
    }

    pub fn window_size(&self, window: window::Id) -> Option<geometry::Size> {
        self.windows
            .iter()
            .find(|entry| entry.id == window)
            .map(|entry| entry.size)
    }

    pub fn drain(&mut self) -> Work {
        let changes = self.sync_windows();
        let windows = self.windows.clone();
        let work = self.runtime.drain_scenes(|window| {
            windows
                .iter()
                .find(|entry| entry.id == window)
                .map(|entry| entry.size)
                .unwrap_or_else(default_size)
        });

        Work::from_render_work(work, changes)
    }

    pub fn step(&mut self) -> Work {
        if self.runtime.pending_task_completions() > 0 {
            self.runtime.dispatch_next_task_completion();
        } else if self.runtime.pending_tasks() > 0 {
            self.runtime.run_next_task();
        }

        self.drain()
    }

    pub fn handle_event(&mut self, event: Event) -> Result<Work, Error> {
        match event {
            Event::Started => {
                self.start();
                Ok(self.drain())
            }
            Event::WindowResized { window, size } => {
                self.set_window_size(window, size);
                Ok(self.drain())
            }
            Event::RedrawRequested { window } => {
                self.runtime.request_redraw(window);
                Ok(self.drain())
            }
            Event::CloseRequested { window } => {
                let trigger = self.runtime.trigger::<session::CloseWindow>(());
                self.runtime.invoke_focused(window, trigger);
                Ok(self.drain())
            }
            Event::PointerMoved { window, point } => {
                self.pointer_move(window, point)?;
                Ok(self.drain())
            }
            Event::PointerDown { window, point } => {
                self.pointer_down(window, point)?;
                Ok(self.drain())
            }
            Event::PointerUp { window, point } => {
                self.pointer_up(window, point)?;
                Ok(self.drain())
            }
            Event::PointerLeft { window } => {
                self.pointer_left(window)?;
                Ok(self.drain())
            }
            Event::Scrolled {
                window,
                point,
                delta,
            } => {
                self.scroll(window, point, delta)?;
                Ok(self.drain())
            }
            Event::KeyDown {
                window,
                key,
                modifiers,
                text,
            } => {
                self.handle_input(
                    window,
                    input::Input::key_down_with_text(key, modifiers, text),
                )?;
                Ok(self.drain())
            }
            Event::TextCommitted { window, text } => {
                self.handle_input(window, input::Input::text_commit(text))?;
                Ok(self.drain())
            }
            Event::TextPreedit { window, preedit } => {
                self.handle_input(window, input::Input::text_preedit(preedit))?;
                Ok(self.drain())
            }
            Event::FilePathSelected { window, path } => {
                self.file_path_selected(window, path)?;
                Ok(self.drain())
            }
            Event::Poll => Ok(self.step()),
        }
    }

    pub fn handle_input(
        &mut self,
        window: window::Id,
        input: input::Input,
    ) -> Result<input::Outcome, Error> {
        self.runtime.handle_input(window, input)
    }

    pub fn file_path_selected(
        &mut self,
        window: window::Id,
        path: Option<PathBuf>,
    ) -> Result<input::Outcome, Error> {
        self.handle_input(window, input::Input::file_path_selected(path))
    }

    pub fn pointer_move(
        &mut self,
        window: window::Id,
        point: geometry::Point,
    ) -> Result<input::Outcome, Error> {
        let Some(size) = self.window_size(window) else {
            return Ok(input::Outcome::ignored());
        };

        self.runtime.pointer_move_at(window, size, point)
    }

    pub fn pointer_down(
        &mut self,
        window: window::Id,
        point: geometry::Point,
    ) -> Result<input::Outcome, Error> {
        let Some(size) = self.window_size(window) else {
            return Ok(input::Outcome::ignored());
        };

        self.runtime.pointer_down_at(window, size, point)
    }

    pub fn pointer_up(
        &mut self,
        window: window::Id,
        point: geometry::Point,
    ) -> Result<input::Outcome, Error> {
        let Some(size) = self.window_size(window) else {
            return Ok(input::Outcome::ignored());
        };

        self.runtime.pointer_up_at(window, size, point)
    }

    pub fn pointer_left(&mut self, window: window::Id) -> Result<input::Outcome, Error> {
        self.runtime
            .handle_view(window, view::Action::pointer_left())
    }

    pub fn scroll(
        &mut self,
        window: window::Id,
        point: geometry::Point,
        delta: interaction::ScrollDelta,
    ) -> Result<input::Outcome, Error> {
        let Some(size) = self.window_size(window) else {
            return Ok(input::Outcome::ignored());
        };

        self.runtime.scroll_at(window, size, point, delta)
    }

    pub fn run_next_task(&mut self) -> Option<task::Outcome> {
        self.runtime.run_next_task()
    }

    pub fn complete_next_task(&mut self) -> Option<task::Id> {
        self.runtime.complete_next_task()
    }

    pub fn dispatch_next_task_completion(&mut self) -> Option<task::Outcome> {
        self.runtime.dispatch_next_task_completion()
    }

    fn sync_windows(&mut self) -> WindowChanges {
        let windows = self
            .runtime
            .session()
            .windows()
            .iter()
            .map(|window| {
                (
                    window.id(),
                    window.title().to_owned(),
                    window.inner_size(),
                    window.canvas_color(),
                )
            })
            .collect::<Vec<_>>();

        let mut changes = WindowChanges::default();
        self.windows.retain(|entry| {
            let retained = windows.iter().any(|(window, _, _, _)| *window == entry.id);
            if !retained {
                changes.closed.push(entry.id);
            }
            retained
        });

        for (window, title, inner_size, canvas_color) in windows {
            if let Some(entry) = self.windows.iter_mut().find(|entry| entry.id == window) {
                entry.title = title;
                entry.canvas_color = canvas_color;
                continue;
            }

            let entry = Window {
                id: window,
                title,
                size: inner_size,
                canvas_color,
            };
            changes.opened.push(entry.clone());
            self.windows.push(entry);
        }

        changes
    }
}

impl Work {
    fn from_render_work(work: runtime::RenderWork, changes: WindowChanges) -> Self {
        Self {
            opened_windows: changes.opened,
            closed_windows: changes.closed,
            presentations: work
                .presentations()
                .iter()
                .cloned()
                .map(Presentation::from_scene_presentation)
                .collect(),
            requests: work.requests().to_vec(),
            pending_tasks: work.pending_tasks(),
            task_completions: work.task_completions(),
        }
    }

    pub fn opened_windows(&self) -> &[Window] {
        &self.opened_windows
    }

    pub fn closed_windows(&self) -> &[window::Id] {
        &self.closed_windows
    }

    pub fn presentations(&self) -> &[Presentation] {
        &self.presentations
    }

    pub fn requests(&self) -> &[session::Request] {
        &self.requests
    }

    pub fn pending_tasks(&self) -> usize {
        self.pending_tasks
    }

    pub fn task_completions(&self) -> usize {
        self.task_completions
    }

    pub fn needs_poll(&self) -> bool {
        self.pending_tasks > 0 || self.task_completions > 0
    }

    pub fn is_empty(&self) -> bool {
        self.opened_windows.is_empty()
            && self.closed_windows.is_empty()
            && self.presentations.is_empty()
            && self.requests.is_empty()
            && self.pending_tasks == 0
            && self.task_completions == 0
    }
}

impl Window {
    pub fn id(&self) -> window::Id {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn size(&self) -> geometry::Size {
        self.size
    }

    pub fn canvas_color(&self) -> scene::Color {
        self.canvas_color
    }
}

impl Presentation {
    fn from_scene_presentation(presentation: scene::Presentation) -> Self {
        Self {
            window: presentation.window(),
            layout: presentation.layout().clone(),
            scene: presentation.scene().to_paint_scene(),
        }
    }

    pub fn window(&self) -> window::Id {
        self.window
    }

    pub fn layout(&self) -> &layout::Layout {
        &self.layout
    }

    pub fn scene(&self) -> &paint::Scene {
        &self.scene
    }
}

fn default_size() -> geometry::Size {
    window::Options::default_inner_size()
}
