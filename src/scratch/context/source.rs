#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Source {
    Keyboard,
    Menu,
    Button,
    Shortcut,
    Input,
    Programmatic,
}
