use super::edit;
use super::surface::Surface;
use crate::command;

pub trait TextTarget: 'static {}

impl command::target::Category for dyn TextTarget {}
impl TextTarget for Surface {}
impl command::Output for edit::CommandResult {}

pub fn text_target_kind() -> command::target::Kind {
    <dyn TextTarget as command::target::Category>::kind()
}

macro_rules! for_each_edit_command {
    ($macro:ident $(, $arg:expr)* $(,)?) => {
        $macro!(
            $crate::text::command::Undo,
            $crate::text::edit::Command::Undo
            $(, $arg)*
        );
        $macro!(
            $crate::text::command::Redo,
            $crate::text::edit::Command::Redo
            $(, $arg)*
        );
        $macro!(
            $crate::text::command::SelectAll,
            $crate::text::edit::Command::SelectAll
            $(, $arg)*
        );
        $macro!(
            $crate::text::command::Cut,
            $crate::text::edit::Command::Cut
            $(, $arg)*
        );
        $macro!(
            $crate::text::command::Delete,
            $crate::text::edit::Command::Delete
            $(, $arg)*
        );
        $macro!(
            $crate::text::command::Copy,
            $crate::text::edit::Command::Copy
            $(, $arg)*
        );
        $macro!(
            $crate::text::command::Paste,
            $crate::text::edit::Command::Paste
            $(, $arg)*
        );
    };
}

pub(crate) use for_each_edit_command;

pub fn define<C>(
    commands: &mut command::registry::Commands,
    configure: impl FnOnce(command::definition::Definition) -> command::definition::Definition,
) -> &mut command::registry::Commands
where
    C: EditCommand,
{
    commands.define_with_target::<C>(text_target_kind(), configure)
}

pub fn define_insert_text(
    commands: &mut command::registry::Commands,
    configure: impl FnOnce(command::definition::Definition) -> command::definition::Definition,
) -> &mut command::registry::Commands {
    commands.define_with_target::<InsertText>(text_target_kind(), configure)
}

pub fn define_defaults(
    commands: &mut command::registry::Commands,
) -> &mut command::registry::Commands {
    define::<Undo>(commands, |command| {
        command
            .shortcut(command::shortcut::Shortcut::ctrl('z'))
            .repeatable()
    });
    define::<Redo>(commands, |command| {
        command
            .shortcut(command::shortcut::Shortcut::ctrl_shift('z'))
            .shortcut(command::shortcut::Shortcut::ctrl('y'))
            .repeatable()
    });
    define::<SelectAll>(commands, |command| {
        command.shortcut(command::shortcut::Shortcut::ctrl('a'))
    });
    define::<Cut>(commands, |command| {
        command.shortcut(command::shortcut::Shortcut::ctrl('x'))
    });
    define::<Delete>(commands, |command| command);
    define::<Copy>(commands, |command| {
        command.shortcut(command::shortcut::Shortcut::ctrl('c'))
    });
    define::<Paste>(commands, |command| {
        command
            .shortcut(command::shortcut::Shortcut::ctrl('v'))
            .repeatable()
    });
    define_insert_text(commands, |command| command.repeatable())
}

pub(crate) fn editable_bindings() -> Vec<command::binding::Binding> {
    let mut bindings = Vec::new();

    macro_rules! push_binding {
        ($command:ty, $edit:expr, $bindings:expr) => {
            $bindings.push(command::binding::Binding::of::<$command>());
        };
    }

    for_each_edit_command!(push_binding, bindings);
    bindings.push(command::binding::Binding::of::<InsertText>());
    bindings
}

pub(crate) fn read_only_bindings() -> [command::binding::Binding; 2] {
    [
        command::binding::Binding::of::<SelectAll>(),
        command::binding::Binding::of::<Copy>(),
    ]
}

impl command::binding::Responder for Surface {
    fn bind_targets(&self, targets: &mut Vec<command::target::Kind>) {
        if self.is_selectable() {
            targets.push(text_target_kind());
        }
    }

    fn bind_commands(&self, bindings: &mut Vec<command::binding::Binding>) {
        if self.is_editable() {
            bindings.extend(editable_bindings());
        } else if self.is_read_only() {
            bindings.extend(read_only_bindings());
        }
    }
}

pub(crate) fn edit_command_label(command: edit::Command) -> &'static str {
    match command {
        edit::Command::Copy => "copy",
        edit::Command::Cut => "cut",
        edit::Command::Delete => "delete",
        edit::Command::Paste => "paste",
        edit::Command::SelectAll => "select all",
        edit::Command::Undo => "undo",
        edit::Command::Redo => "redo",
    }
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
    output: edit::CommandResult,
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
