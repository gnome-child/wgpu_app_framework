use std::{
    io,
    path::{Path, PathBuf},
};

use crate::text;

mod command;
mod edit;
mod file;
mod notification;
mod outcome;
mod save;

pub(crate) use command::register;
pub use command::{
    ApplyEdit, Copy, Cut, Delete, NewFile, OpenFile, OpenPath, Paste, SaveAsFile, SaveFile,
    SaveToPath, SelectAll,
};
pub use notification::{OpenDialogCanceled, SaveDialogCanceled};
pub use outcome::Outcome;
pub use save::{Identity, SaveSnapshot, Version};

#[derive(Clone)]
pub struct Document {
    identity: Identity,
    buffer: text::Buffer,
    text_state: text::edit::State,
    path: Option<PathBuf>,
    saved_buffer_revision: u64,
    edit_count: usize,
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
        let text_state = buffer.initial_state();

        Self {
            identity: Identity::next(),
            buffer,
            text_state,
            path: None,
            saved_buffer_revision,
            edit_count: 0,
        }
    }

    pub fn new_file(&mut self) {
        *self = Self::new_multiline();
    }

    pub fn replace_unsaved_text(&mut self, text: impl Into<String>) {
        self.identity = Identity::next();
        self.buffer = text::Buffer::from_multiline_text(text);
        self.text_state = self.buffer.initial_state();
        self.path = None;
        self.saved_buffer_revision = self.buffer.revision().wrapping_sub(1);
        self.edit_count = 0;
    }

    pub fn open_path(&mut self, path: impl Into<PathBuf>) -> io::Result<()> {
        let path = path.into();
        let buffer = text::Buffer::from_multiline_text(std::fs::read_to_string(&path)?);
        let saved_buffer_revision = buffer.revision();

        self.identity = Identity::next();
        self.buffer = buffer;
        self.text_state = self.buffer.initial_state();
        self.path = Some(path);
        self.saved_buffer_revision = saved_buffer_revision;
        self.edit_count = 0;

        Ok(())
    }

    pub fn save_to(&mut self, path: impl Into<PathBuf>) -> io::Result<()> {
        let path = path.into();
        let snapshot = self.save_snapshot();

        snapshot.write_to(&path)?;
        let accepted = self.record_saved_version_at(snapshot.version(), path);
        debug_assert!(accepted, "a synchronous save keeps its document identity");

        Ok(())
    }

    pub fn identity(&self) -> Identity {
        self.identity
    }

    pub fn version(&self) -> Version {
        Version::new(self.identity, self.buffer.revision())
    }

    pub fn save_snapshot(&self) -> SaveSnapshot {
        SaveSnapshot::new(self.version(), self.buffer.text())
    }

    pub fn buffer(&self) -> &text::Buffer {
        &self.buffer
    }

    pub fn text_state(&self) -> text::edit::State {
        self.text_state
    }

    pub fn position(&self) -> text::buffer::Position {
        self.buffer.position_for_state(self.text_state)
    }

    pub fn selected_text(&self) -> Option<String> {
        self.buffer.selected_text_for_state(self.text_state)
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

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
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

    pub fn record_saved_version_at(&mut self, version: Version, path: impl Into<PathBuf>) -> bool {
        if version.identity() != self.identity {
            return false;
        }

        self.path = Some(path.into());
        self.saved_buffer_revision = version.revision();
        true
    }

    pub fn apply_edit(&mut self, edit: text::edit::Edit) -> Outcome {
        let mut editor = text::edit::Editor::new();
        let result = editor.apply_edit(&mut self.buffer, &mut self.text_state, edit);
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
            &mut self.text_state,
            edit,
            caret_map,
        );
        self.outcome_from_edit_result(result)
    }

    fn outcome_from_edit_result(&mut self, result: text::edit::Outcome) -> Outcome {
        if result.text_changed {
            self.edit_count += 1;
        }

        Outcome::from_edit_result(result)
    }

    pub fn apply_action(
        &mut self,
        action: text::edit::Action,
        clipboard: &mut dyn text::edit::Clipboard,
    ) -> Outcome {
        let mut editor = text::edit::Editor::new();
        let result = editor.apply_action(&mut self.buffer, &mut self.text_state, action, clipboard);
        if result.result.text_changed {
            self.edit_count += 1;
        }

        Outcome::from_command_result(result.result)
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new_multiline()
    }
}
