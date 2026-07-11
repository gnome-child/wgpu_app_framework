use std::path::PathBuf;

use crate::{clipboard, context::Context, text};
use unicode_segmentation::UnicodeSegmentation;

use super::Outcome;
use crate::command::{self, Command};

pub struct ApplyEdit;

impl Command for ApplyEdit {
    type Args = text::edit::Edit;
    type Output = Outcome;

    const NAME: &'static str = "document.apply_edit";

    fn history_group(args: &Self::Args) -> Option<command::HistoryGroup> {
        is_typing_edit(args).then_some(
            command::HistoryGroup::new("text.typing")
                .with_coalesce_window(text::edit::TYPING_UNDO_COALESCE_WINDOW),
        )
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

impl Paste {
    pub(crate) fn availability(cx: &Context) -> command::State {
        if cx
            .clipboard()
            .is_some_and(|clipboard| clipboard.contains::<clipboard::Text>().unwrap_or(true))
        {
            command::State::enabled()
        } else {
            command::State::disabled()
        }
    }
}

impl Command for Paste {
    type Args = ();
    type Output = Outcome;

    const NAME: &'static str = "edit.paste";
}

pub struct Editing;

impl Editing {
    pub fn standard() -> command::Set {
        command::Set::new()
            .include::<ApplyEdit>(command::Spec::new("Edit"))
            .include::<Cut>(
                command::Spec::new("Cut")
                    .key_chord(command::KeyChord::standard(command::Standard::Cut)),
            )
            .include::<Copy>(
                command::Spec::new("Copy")
                    .key_chord(command::KeyChord::standard(command::Standard::Copy)),
            )
            .include::<Paste>(
                command::Spec::new("Paste")
                    .key_chord(command::KeyChord::standard(command::Standard::Paste)),
            )
            .include::<Delete>(command::Spec::new("Delete"))
            .include::<SelectAll>(
                command::Spec::new("Select All")
                    .key_chord(command::KeyChord::standard(command::Standard::SelectAll)),
            )
    }
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
