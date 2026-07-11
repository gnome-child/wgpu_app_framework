use super::{
    State,
    command::{
        EditRecordCount, EditRecordCountArgs, EditRecordNote, EditRecordNoteArgs, IncrementClicks,
        ResetControls, SelectMode, SetLevel, SetRecordEnabled, SetRecordEnabledArgs, SubmitQuery,
        ToggleAdvanced, ToggleGrid, ToggleWrap,
    },
};
use wgpu_l3::{Context, Response, Target, command};

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

impl Target<wgpu_l3::table::SortBy> for State {
    fn state(&self, _: &wgpu_l3::table::SortIntent, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, intent: wgpu_l3::table::SortIntent, _: &mut Context) -> Response<()> {
        self.record_sort = wgpu_l3::table::SortState::new(intent.column(), intent.direction());
        self.last_status = format!(
            "{}: {}",
            intent.column().as_str(),
            match intent.direction() {
                wgpu_l3::table::SortDirection::Ascending => "ascending",
                wgpu_l3::table::SortDirection::Descending => "descending",
            }
        );
        Response::changed(())
    }
}

impl Target<EditRecordNote> for State {
    fn state(&self, _: &EditRecordNoteArgs, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, args: EditRecordNoteArgs, _: &mut Context) -> Response<()> {
        self.record_notes
            .insert(args.cell.row().value(), args.value);
        self.last_status = format!("edited note for record {}", args.cell.row().value());
        Response::changed(())
    }
}

impl Target<EditRecordCount> for State {
    fn state(&self, _: &EditRecordCountArgs, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, args: EditRecordCountArgs, _: &mut Context) -> Response<()> {
        self.record_counts
            .insert(args.cell.row().value(), args.value);
        self.last_status = format!("edited count for record {}", args.cell.row().value());
        Response::changed(())
    }
}

impl Target<SetRecordEnabled> for State {
    fn state(&self, args: &SetRecordEnabledArgs, _: &Context) -> command::State {
        let key = args.cell.row().value();
        let checked = self
            .record_enabled
            .get(&key)
            .copied()
            .unwrap_or(key % 2 == 0);
        command::State::enabled().checked(checked)
    }

    fn invoke(&mut self, args: SetRecordEnabledArgs, _: &mut Context) -> Response<()> {
        let key = args.cell.row().value();
        self.record_enabled.insert(key, args.value);
        self.last_status = format!("record {key}: enabled {}", on_off(args.value));
        Response::changed(())
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}
