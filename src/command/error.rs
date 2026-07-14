use thiserror::Error as ThisError;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("command is not registered: {command}")]
    UnknownCommand { command: &'static str },

    #[error("no responder-chain target can handle command: {command}")]
    MissingTarget { command: &'static str },

    #[error("target type mismatch while invoking command: {command}")]
    TargetMismatch { command: &'static str },

    #[error("argument type mismatch while invoking command: {command}")]
    ArgsMismatch { command: &'static str },

    #[error("output type mismatch while invoking command: {command}")]
    OutputMismatch { command: &'static str },

    #[error("shortcut {shortcut} is bound to multiple commands: {commands:?}")]
    AmbiguousShortcut {
        shortcut: &'static str,
        commands: Vec<&'static str>,
    },

    #[error("shortcut {shortcut} cannot invoke command with arguments: {command}")]
    ShortcutRequiresArgs {
        shortcut: &'static str,
        command: &'static str,
    },

    #[error("command is disabled: {command}")]
    Disabled { command: &'static str },

    #[error("multiple targets claim command {command} in responder {responder}")]
    AmbiguousTarget {
        command: &'static str,
        responder: &'static str,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_facade_paths_preserve_one_command_failure_type() {
        let owned = Error::Disabled {
            command: "test.command",
        };
        let root: crate::Error = owned;
        let namespaced: crate::error::Error = root;

        assert!(matches!(
            namespaced,
            Error::Disabled {
                command: "test.command"
            }
        ));
    }
}
