use super::Mode;
use wgpu_l3::command::{Command, HistoryGroup};

pub struct IncrementClicks;
pub struct ToggleWrap;
pub struct ToggleGrid;
pub struct SelectMode;
pub struct SetLevel;
pub struct SubmitQuery;
pub struct ToggleAdvanced;
pub struct ResetControls;
pub struct EditRecordNote;
pub struct EditRecordCount;
pub struct SetRecordEnabled;
pub struct ToggleExpandedRows;

#[derive(Clone)]
pub struct EditRecordNoteArgs {
    pub cell: wgpu_l3::table::Cell,
    pub value: String,
}

#[derive(Clone)]
pub struct EditRecordCountArgs {
    pub cell: wgpu_l3::table::Cell,
    pub value: i64,
}

#[derive(Clone)]
pub struct SetRecordEnabledArgs {
    pub cell: wgpu_l3::table::Cell,
    pub value: bool,
}

impl Command for IncrementClicks {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "control_gallery.increment_clicks";
}

impl Command for ToggleWrap {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "control_gallery.toggle_wrap";
}

impl Command for ToggleGrid {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "control_gallery.toggle_grid";
}

impl Command for SelectMode {
    type Args = Mode;
    type Output = ();

    const NAME: &'static str = "control_gallery.select_mode";
}

impl Command for SetLevel {
    type Args = f64;
    type Output = ();

    const NAME: &'static str = "control_gallery.set_level";

    fn history_group(_: &Self::Args) -> Option<HistoryGroup> {
        Some(HistoryGroup::new("control_gallery.level"))
    }
}

impl Command for SubmitQuery {
    type Args = String;
    type Output = ();

    const NAME: &'static str = "control_gallery.submit_query";
}

impl Command for ToggleAdvanced {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "control_gallery.toggle_advanced";
}

impl Command for ResetControls {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "control_gallery.reset_controls";
}

impl Command for EditRecordNote {
    type Args = EditRecordNoteArgs;
    type Output = ();

    const NAME: &'static str = "control_gallery.edit_record_note";
}

impl Command for EditRecordCount {
    type Args = EditRecordCountArgs;
    type Output = ();

    const NAME: &'static str = "control_gallery.edit_record_count";
}

impl Command for SetRecordEnabled {
    type Args = SetRecordEnabledArgs;
    type Output = ();

    const NAME: &'static str = "control_gallery.set_record_enabled";
}

impl Command for ToggleExpandedRows {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "control_gallery.toggle_expanded_rows";
}
