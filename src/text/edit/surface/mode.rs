#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum FieldMode {
    #[default]
    Editable,
    ReadOnly,
    Disabled,
}
