use std::{
    io,
    path::{Path, PathBuf},
};

use crate::text;
use unicode_segmentation::UnicodeSegmentation;

use super::{
    clipboard,
    command::{self, Command},
    context::Context,
    response::{Effect, Response},
    target::Target,
};

#[derive(Clone)]
pub struct Document {
    buffer: text::Buffer,
    edit_state: text::edit::State,
    path: Option<PathBuf>,
    saved_buffer_revision: u64,
    edit_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    text_changed: bool,
    selection_changed: bool,
    clipboard_changed: bool,
    unavailable: bool,
}

pub struct ApplyEdit;

impl Command for ApplyEdit {
    type Args = text::edit::Edit;
    type Output = Outcome;

    const NAME: &'static str = "document.apply_edit";

    fn history_group(args: &Self::Args) -> Option<command::HistoryGroup> {
        is_typing_edit(args).then_some(command::HistoryGroup::new("text.typing"))
    }
}

pub struct SelectAll;

impl Command for SelectAll {
    type Args = ();
    type Output = Outcome;

    const NAME: &'static str = "edit.select_all";
}

pub struct Copy;

impl Command for Copy {
    type Args = ();
    type Output = Outcome;

    const NAME: &'static str = "edit.copy";
}

pub struct Cut;

impl Command for Cut {
    type Args = ();
    type Output = Outcome;

    const NAME: &'static str = "edit.cut";
}

pub struct Delete;

impl Command for Delete {
    type Args = ();
    type Output = Outcome;

    const NAME: &'static str = "edit.delete";
}

pub struct Paste;

impl Command for Paste {
    type Args = ();
    type Output = Outcome;

    const NAME: &'static str = "edit.paste";
}

pub struct NewFile;

impl Command for NewFile {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "document.new_file";
}

pub struct OpenFile;

impl Command for OpenFile {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "document.open_file";
}

pub struct OpenPath;

impl Command for OpenPath {
    type Args = PathBuf;
    type Output = Result<(), String>;

    const NAME: &'static str = "document.open_path";
}

pub struct OpenCanceled;

impl Command for OpenCanceled {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "document.open_canceled";
}

pub struct SaveFile;

impl Command for SaveFile {
    type Args = ();
    type Output = Result<(), String>;

    const NAME: &'static str = "document.save_file";
}

pub struct SaveAsFile;

impl Command for SaveAsFile {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "document.save_as_file";
}

pub struct SaveToPath;

impl Command for SaveToPath {
    type Args = PathBuf;
    type Output = Result<(), String>;

    const NAME: &'static str = "document.save_to_path";
}

pub struct SaveCanceled;

impl Command for SaveCanceled {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "document.save_canceled";
}

impl Document {
    pub fn new_multiline() -> Self {
        Self::from_buffer(text::Buffer::new_multiline())
    }

    pub fn from_text(text: impl Into<String>) -> Self {
        Self::from_buffer(text::Buffer::from_text(text))
    }

    pub fn from_multiline_text(text: impl Into<String>) -> Self {
        Self::from_buffer(text::Buffer::from_multiline_text(text))
    }

    fn from_buffer(buffer: text::Buffer) -> Self {
        let saved_buffer_revision = buffer.revision();
        let edit_state = buffer.edit_state();

        Self {
            buffer,
            edit_state,
            path: None,
            saved_buffer_revision,
            edit_count: 0,
        }
    }

    pub fn new_file(&mut self) {
        *self = Self::new_multiline();
    }

    pub fn replace_unsaved_text(&mut self, text: impl Into<String>) {
        self.buffer = text::Buffer::from_multiline_text(text);
        self.edit_state = self.buffer.edit_state();
        self.path = None;
        self.saved_buffer_revision = self.buffer.revision().wrapping_sub(1);
        self.edit_count = 0;
    }

    pub fn open_path(&mut self, path: impl Into<PathBuf>) -> io::Result<()> {
        let path = path.into();
        let buffer = text::Buffer::from_multiline_text(std::fs::read_to_string(&path)?);
        let saved_buffer_revision = buffer.revision();

        self.buffer = buffer;
        self.edit_state = self.buffer.edit_state();
        self.path = Some(path);
        self.saved_buffer_revision = saved_buffer_revision;
        self.edit_count = 0;

        Ok(())
    }

    pub fn save_to(&mut self, path: impl Into<PathBuf>) -> io::Result<()> {
        let path = path.into();

        std::fs::write(&path, self.buffer.text())?;
        self.path = Some(path);
        self.mark_saved();

        Ok(())
    }

    pub fn buffer(&self) -> &text::Buffer {
        &self.buffer
    }

    pub fn edit_state(&self) -> text::edit::State {
        self.edit_state
    }

    pub fn position(&self) -> text::TextPosition {
        self.buffer.position_for_state(self.edit_state)
    }

    pub fn selected_text(&self) -> Option<String> {
        self.buffer.selected_text_for_state(self.edit_state)
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn text(&self) -> String {
        self.buffer.text()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn line_count(&self) -> usize {
        self.buffer.logical_line_count()
    }

    pub fn is_multiline(&self) -> bool {
        self.buffer.is_multiline()
    }

    pub fn edit_count(&self) -> usize {
        self.edit_count
    }

    pub fn buffer_revision(&self) -> u64 {
        self.buffer.revision()
    }

    pub fn saved_buffer_revision(&self) -> u64 {
        self.saved_buffer_revision
    }

    pub fn is_dirty(&self) -> bool {
        self.buffer.revision() != self.saved_buffer_revision
    }

    pub fn mark_saved(&mut self) {
        self.saved_buffer_revision = self.buffer.revision();
    }

    pub fn mark_saved_at(&mut self, path: impl Into<PathBuf>) {
        self.path = Some(path.into());
        self.mark_saved();
    }

    pub fn apply_edit(&mut self, edit: text::edit::Edit) -> Outcome {
        let mut editor = text::edit::Editor::new();
        let result = editor.apply_edit(&mut self.buffer, &mut self.edit_state, edit);
        self.outcome_from_edit_result(result)
    }

    fn apply_edit_with_caret_map(
        &mut self,
        edit: text::edit::Edit,
        caret_map: &mut dyn text::edit::CaretMap,
    ) -> Outcome {
        let mut editor = text::edit::Editor::new();
        let result = editor.apply_edit_with_caret_map(
            &mut self.buffer,
            &mut self.edit_state,
            edit,
            caret_map,
        );
        self.outcome_from_edit_result(result)
    }

    fn outcome_from_edit_result(&mut self, result: text::edit::Outcome) -> Outcome {
        if result.text_changed {
            self.edit_count += 1;
        }

        Outcome {
            text_changed: result.text_changed,
            selection_changed: result.selection_changed,
            clipboard_changed: false,
            unavailable: false,
        }
    }

    pub fn apply_command(
        &mut self,
        command: text::edit::Command,
        clipboard: &mut dyn text::edit::Clipboard,
    ) -> Outcome {
        let mut editor = text::edit::Editor::new();
        let result =
            editor.apply_command(&mut self.buffer, &mut self.edit_state, command, clipboard);
        if result.result.text_changed {
            self.edit_count += 1;
        }

        Self::outcome_from_command_result(result.result)
    }

    fn outcome_from_command_result(result: text::edit::CommandResult) -> Outcome {
        Outcome {
            text_changed: result.text_changed,
            selection_changed: result.selection_changed,
            clipboard_changed: result.clipboard_changed,
            unavailable: result.unavailable,
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new_multiline()
    }
}

impl Outcome {
    pub fn text_changed(self) -> bool {
        self.text_changed
    }

    pub fn selection_changed(self) -> bool {
        self.selection_changed
    }

    pub fn clipboard_changed(self) -> bool {
        self.clipboard_changed
    }

    pub fn unavailable(self) -> bool {
        self.unavailable
    }

    pub fn buffer_changed(self) -> bool {
        self.text_changed || self.selection_changed
    }
}

impl Target<ApplyEdit> for Document {
    fn state(&self, _: &text::edit::Edit, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, edit: text::edit::Edit, cx: &mut Context) -> Response<Outcome> {
        let outcome = if let Some(mut text) = cx.text_service() {
            self.apply_edit_with_caret_map(edit, &mut text)
        } else {
            self.apply_edit(edit)
        };
        if outcome.buffer_changed() {
            Response::changed(outcome)
        } else {
            Response::output(outcome)
        }
    }
}

impl Target<SelectAll> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        if self.buffer.is_empty() {
            command::State::disabled()
        } else {
            command::State::enabled()
        }
    }

    fn invoke(&mut self, _: (), cx: &mut Context) -> Response<Outcome> {
        invoke_text_command(self, text::edit::Command::SelectAll, cx)
    }
}

impl Target<Copy> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        if self.buffer.has_selection_for_state(self.edit_state) {
            command::State::enabled()
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), cx: &mut Context) -> Response<Outcome> {
        invoke_text_command(self, text::edit::Command::Copy, cx)
    }
}

impl Target<Cut> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        if self.buffer.has_selection_for_state(self.edit_state) {
            command::State::enabled()
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), cx: &mut Context) -> Response<Outcome> {
        invoke_text_command(self, text::edit::Command::Cut, cx)
    }
}

impl Target<Delete> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        if self.buffer.has_selection_for_state(self.edit_state) || !self.buffer.is_empty() {
            command::State::enabled()
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), cx: &mut Context) -> Response<Outcome> {
        invoke_text_command(self, text::edit::Command::Delete, cx)
    }
}

impl Target<Paste> for Document {
    fn state(&self, _: &(), cx: &Context) -> command::State {
        if cx
            .clipboard()
            .is_some_and(|clipboard| clipboard.contains::<clipboard::Text>())
        {
            command::State::enabled()
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), cx: &mut Context) -> Response<Outcome> {
        invoke_text_command(self, text::edit::Command::Paste, cx)
    }
}

fn invoke_text_command(
    document: &mut Document,
    command: text::edit::Command,
    cx: &mut Context,
) -> Response<Outcome> {
    let Some(mut clipboard) = cx.clipboard_mut() else {
        return Response::output(Outcome {
            unavailable: true,
            text_changed: false,
            selection_changed: false,
            clipboard_changed: false,
        });
    };
    let result = document.apply_command(command, &mut clipboard);

    if result.buffer_changed() {
        Response::changed(result)
    } else {
        Response::output(result)
    }
}

fn is_typing_edit(edit: &text::edit::Edit) -> bool {
    let text = match edit {
        text::edit::Edit::Insert(text) | text::edit::Edit::ImeCommit(text) => text,
        _ => return false,
    };
    let mut graphemes = text.graphemes(true);
    let Some(first) = graphemes.next() else {
        return false;
    };
    graphemes.next().is_none()
        && !first
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_ascii_punctuation())
}

impl Target<NewFile> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.new_file();
        Response::changed(())
    }
}

impl Target<OpenFile> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        Response::output(()).with_effect(Effect::OpenFileDialog)
    }
}

impl Target<OpenPath> for Document {
    fn state(&self, _: &PathBuf, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, path: PathBuf, _: &mut Context) -> Response<Result<(), String>> {
        match self.open_path(path) {
            Ok(()) => Response::changed(Ok(())),
            Err(error) => Response::output(Err(error.to_string())),
        }
    }
}

impl Target<OpenCanceled> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        Response::output(())
    }
}

impl Target<SaveFile> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        if self.is_dirty() {
            command::State::enabled()
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<Result<(), String>> {
        let Some(path) = self.path.clone() else {
            return Response::output(Ok(())).with_effect(Effect::SaveFileDialog);
        };

        match self.save_to(path) {
            Ok(()) => Response::changed(Ok(())),
            Err(error) => Response::output(Err(error.to_string())),
        }
    }
}

impl Target<SaveAsFile> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        Response::output(()).with_effect(Effect::SaveFileDialog)
    }
}

impl Target<SaveToPath> for Document {
    fn state(&self, _: &PathBuf, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, path: PathBuf, _: &mut Context) -> Response<Result<(), String>> {
        match self.save_to(path) {
            Ok(()) => Response::changed(Ok(())),
            Err(error) => Response::output(Err(error.to_string())),
        }
    }
}

impl Target<SaveCanceled> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        Response::output(())
    }
}
