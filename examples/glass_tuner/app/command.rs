use super::state::AcrylicToken;
use wgpu_l3::command::Command;

pub struct TogglePanel;
pub struct ToggleComparison;
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

impl Command for ToggleComparison {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "glass_tuner.toggle_promotion_comparison";
}
