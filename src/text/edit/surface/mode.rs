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

    pub(crate) fn allows_edit(self, edit: &super::super::Edit) -> bool {
        match self {
            Self::Editable => true,
            Self::ReadOnly => !edit.mutates_text(),
            Self::Disabled => false,
        }
    }
}
