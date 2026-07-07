use std::path::{Path, PathBuf};

use super::{
    State,
    command::{LoadStressText, ToggleDebugPanel, ToggleWrapText},
    event::Event,
    state::STRESS_TEXT,
    view::compact_path,
};
use wgpu_l3::{
    Response, Target, Task, command,
    context::Context,
    document,
    response::{self},
};

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

pub(super) fn finish_save(state: &mut State, path: PathBuf, result: Result<(), String>) {
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

pub(super) fn record_apply_edit_status(
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

pub(super) fn record_text_command_status(
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
