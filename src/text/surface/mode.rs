#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum FieldMode {
    #[default]
    Editable,
    ReadOnly,
    Disabled,
}

impl FieldMode {
    pub(crate) fn is_editable(self) -> bool {
        self == Self::Editable
    }

    pub(crate) fn is_selectable(self) -> bool {
        self != Self::Disabled
    }
}
