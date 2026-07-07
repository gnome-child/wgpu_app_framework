use wgpu_l3::command;

pub struct ToggleWrapText;

impl command::Command for ToggleWrapText {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "view.toggle_wrap_text";
}

pub struct ToggleDebugPanel;

impl command::Command for ToggleDebugPanel {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "view.toggle_debug_panel";
}

pub struct LoadStressText;

impl command::Command for LoadStressText {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "document.load_stress_text";
}
