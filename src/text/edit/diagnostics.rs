#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Diagnostics {
    pub text_edit_calls: usize,
    pub text_edit_changed_calls: usize,
    pub text_edit_apply_nanos: u128,
    pub text_edit_deleted_bytes: usize,
    pub text_edit_inserted_bytes: usize,
    pub text_edit_impacted_logical_lines: usize,
}
