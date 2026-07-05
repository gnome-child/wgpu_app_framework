use super::super::{Context, Response, Target, command};
use super::{
    State,
    command::{
        IncrementClicks, ResetControls, SelectMode, SetLevel, SubmitQuery, ToggleAdvanced,
        ToggleGrid, ToggleWrap,
    },
};

impl Target<IncrementClicks> for State {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.clicks = self.clicks.saturating_add(1);
        self.last_status = format!("clicked {}", self.clicks);
        Response::changed(())
    }
}

impl Target<ToggleWrap> for State {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled().checked(self.wrap)
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.wrap = !self.wrap;
        self.last_status = format!("wrap {}", on_off(self.wrap));
        Response::changed(())
    }
}

impl Target<ToggleGrid> for State {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled().checked(self.grid)
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.grid = !self.grid;
        self.last_status = format!("grid {}", on_off(self.grid));
        Response::changed(())
    }
}

impl Target<SelectMode> for State {
    fn state(&self, mode: &super::Mode, _: &Context) -> command::State {
        command::State::enabled().checked(self.mode == *mode)
    }

    fn invoke(&mut self, mode: super::Mode, _: &mut Context) -> Response<()> {
        self.mode = mode;
        self.last_status = format!("mode: {}", mode.label());
        Response::changed(())
    }
}

impl Target<SetLevel> for State {
    fn state(&self, _: &f64, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, level: f64, _: &mut Context) -> Response<()> {
        self.level = level.clamp(0.0, 100.0);
        self.last_status = format!("level {:.0}", self.level);
        Response::changed(())
    }
}

impl Target<SubmitQuery> for State {
    fn state(&self, _: &String, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, query: String, _: &mut Context) -> Response<()> {
        self.query = query;
        self.last_status = if self.query.is_empty() {
            "query cleared".to_owned()
        } else {
            format!("query: {}", self.query)
        };
        Response::changed(())
    }
}

impl Target<ToggleAdvanced> for State {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled().checked(self.show_advanced)
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.show_advanced = !self.show_advanced;
        self.last_status = format!("advanced {}", on_off(self.show_advanced));
        Response::changed(())
    }
}

impl Target<ResetControls> for State {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.reset();
        Response::changed(())
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}
