use super::{
    State,
    command::{SetToken, TogglePanel},
    state::AcrylicToken,
};
use wgpu_l3::{Context, Response, Target, command};

impl Target<TogglePanel> for State {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled().checked(self.panel_open)
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.panel_open = !self.panel_open;
        self.last_status = if self.panel_open {
            "panel shown".to_owned()
        } else {
            "panel hidden".to_owned()
        };
        Response::changed(())
    }
}

impl Target<SetToken> for State {
    fn state(&self, _: &(AcrylicToken, f64), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, (token, value): (AcrylicToken, f64), _: &mut Context) -> Response<()> {
        self.set_token(token, value);
        Response::changed(())
    }
}
