use crate::command;
use crate::text::edit::{self, Surface};

pub trait TextTarget: 'static {}

impl command::target::Category for dyn TextTarget {}
impl TextTarget for Surface {}
impl command::Output for edit::ActionResult {}

pub fn text_target_kind() -> command::target::Kind {
    <dyn TextTarget as command::target::Category>::kind()
}

macro_rules! for_each_edit_command {
    ($macro:ident $(, $arg:expr)* $(,)?) => {
        $macro!(
            $crate::widget::text_command::Undo,
            $crate::text::edit::Action::Undo
            $(, $arg)*
        );
        $macro!(
            $crate::widget::text_command::Redo,
            $crate::text::edit::Action::Redo
            $(, $arg)*
        );
        $macro!(
            $crate::widget::text_command::SelectAll,
            $crate::text::edit::Action::SelectAll
            $(, $arg)*
        );
        $macro!(
            $crate::widget::text_command::Cut,
            $crate::text::edit::Action::Cut
            $(, $arg)*
        );
        $macro!(
            $crate::widget::text_command::Delete,
            $crate::text::edit::Action::Delete
            $(, $arg)*
        );
        $macro!(
            $crate::widget::text_command::Copy,
            $crate::text::edit::Action::Copy
            $(, $arg)*
        );
        $macro!(
            $crate::widget::text_command::Paste,
            $crate::text::edit::Action::Paste
            $(, $arg)*
        );
    };
}

pub(crate) use for_each_edit_command;

#[cfg(test)]
pub fn define<C>(
    commands: &mut command::registry::Commands,
    configure: impl FnOnce(command::definition::Definition) -> command::definition::Definition,
) -> &mut command::registry::Commands
where
    C: EditCommand,
{
    commands.define_with_target::<C>(text_target_kind(), configure)
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

pub(crate) fn edit_action_label(command: edit::Action) -> &'static str {
    match command {
        edit::Action::Copy => "copy",
        edit::Action::Cut => "cut",
        edit::Action::Delete => "delete",
        edit::Action::Paste => "paste",
        edit::Action::SelectAll => "select all",
        edit::Action::Undo => "undo",
        edit::Action::Redo => "redo",
    }
}

pub trait EditCommand: command::Command<Args = (), Output = edit::ActionResult> {
    fn edit_action() -> edit::Action;
}

crate::command!(pub SelectAll {
    name: "select_all",
    display: "Select All",
    output: edit::ActionResult,
    target: dyn TextTarget,
});
crate::command!(pub Copy {
    name: "copy",
    display: "Copy",
    output: edit::ActionResult,
    target: dyn TextTarget,
});
crate::command!(pub Cut {
    name: "cut",
    display: "Cut",
    output: edit::ActionResult,
    target: dyn TextTarget,
});
crate::command!(pub Delete {
    name: "delete",
    display: "Delete",
    output: edit::ActionResult,
    target: dyn TextTarget,
});
crate::command!(pub Paste {
    name: "paste",
    display: "Paste",
    output: edit::ActionResult,
    repeatable: true,
    target: dyn TextTarget,
});
crate::command!(pub Undo {
    name: "undo",
    display: "Undo",
    output: edit::ActionResult,
    repeatable: true,
    target: dyn TextTarget,
});
crate::command!(pub Redo {
    name: "redo",
    display: "Redo",
    output: edit::ActionResult,
    repeatable: true,
    target: dyn TextTarget,
});
crate::command!(pub InsertText {
    name: "insert_text",
    display: "Insert Text",
    args: String,
    output: edit::ActionResult,
    repeatable: true,
    target: dyn TextTarget,
});

impl EditCommand for SelectAll {
    fn edit_action() -> edit::Action {
        edit::Action::SelectAll
    }
}

impl EditCommand for Copy {
    fn edit_action() -> edit::Action {
        edit::Action::Copy
    }
}

impl EditCommand for Cut {
    fn edit_action() -> edit::Action {
        edit::Action::Cut
    }
}

impl EditCommand for Delete {
    fn edit_action() -> edit::Action {
        edit::Action::Delete
    }
}

impl EditCommand for Paste {
    fn edit_action() -> edit::Action {
        edit::Action::Paste
    }
}

impl EditCommand for Undo {
    fn edit_action() -> edit::Action {
        edit::Action::Undo
    }
}

impl EditCommand for Redo {
    fn edit_action() -> edit::Action {
        edit::Action::Redo
    }
}
