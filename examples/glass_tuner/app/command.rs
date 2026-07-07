use super::super::command::Command;
use super::state::AcrylicToken;

pub struct TogglePanel;
pub struct SetToken;

impl Command for TogglePanel {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "glass_tuner.toggle_panel";
}

impl Command for SetToken {
    type Args = (AcrylicToken, f64);
    type Output = ();

    const NAME: &'static str = "glass_tuner.set_acrylic_token";
}
