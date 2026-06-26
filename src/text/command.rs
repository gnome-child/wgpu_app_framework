use super::edit;
use crate::command;

pub trait TextTarget: 'static {}

impl command::target::Category for dyn TextTarget {}

pub fn text_target_kind() -> command::target::Kind {
    <dyn TextTarget as command::target::Category>::kind()
}

pub fn define<C>(
    commands: &mut command::registry::Commands,
    configure: impl FnOnce(command::definition::Definition) -> command::definition::Definition,
) -> &mut command::registry::Commands
where
    C: EditCommand,
{
    commands.define_with_target::<C>(text_target_kind(), configure)
}

pub trait EditCommand: command::Command<Args = (), Output = edit::CommandResult> {
    fn edit_command() -> edit::Command;
}

crate::command!(pub SelectAll {
    name: "select_all",
    display: "Select All",
    output: edit::CommandResult,
    target: dyn TextTarget,
});
crate::command!(pub Copy {
    name: "copy",
    display: "Copy",
    output: edit::CommandResult,
    target: dyn TextTarget,
});
crate::command!(pub Cut {
    name: "cut",
    display: "Cut",
    output: edit::CommandResult,
    target: dyn TextTarget,
});
crate::command!(pub Delete {
    name: "delete",
    display: "Delete",
    output: edit::CommandResult,
    target: dyn TextTarget,
});
crate::command!(pub Paste {
    name: "paste",
    display: "Paste",
    output: edit::CommandResult,
    repeatable: true,
    target: dyn TextTarget,
});
crate::command!(pub Undo {
    name: "undo",
    display: "Undo",
    output: edit::CommandResult,
    repeatable: true,
    target: dyn TextTarget,
});
crate::command!(pub Redo {
    name: "redo",
    display: "Redo",
    output: edit::CommandResult,
    repeatable: true,
    target: dyn TextTarget,
});
crate::command!(pub InsertText {
    name: "insert_text",
    display: "Insert Text",
    args: String,
    repeatable: true,
    target: dyn TextTarget,
});

impl EditCommand for SelectAll {
    fn edit_command() -> edit::Command {
        edit::Command::SelectAll
    }
}

impl EditCommand for Copy {
    fn edit_command() -> edit::Command {
        edit::Command::Copy
    }
}

impl EditCommand for Cut {
    fn edit_command() -> edit::Command {
        edit::Command::Cut
    }
}

impl EditCommand for Delete {
    fn edit_command() -> edit::Command {
        edit::Command::Delete
    }
}

impl EditCommand for Paste {
    fn edit_command() -> edit::Command {
        edit::Command::Paste
    }
}

impl EditCommand for Undo {
    fn edit_command() -> edit::Command {
        edit::Command::Undo
    }
}

impl EditCommand for Redo {
    fn edit_command() -> edit::Command {
        edit::Command::Redo
    }
}
