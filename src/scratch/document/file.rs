use std::path::PathBuf;

use super::{
    Document, NewFile, OpenCanceled, OpenFile, OpenPath, SaveAsFile, SaveCanceled, SaveFile,
    SaveToPath,
};
use crate::scratch::{
    command,
    context::Context,
    response::{Effect, Response},
    target::Target,
};

impl Target<NewFile> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.new_file();
        Response::changed(())
    }
}

impl Target<OpenFile> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        Response::output(()).with_effect(Effect::OpenFileDialog)
    }
}

impl Target<OpenPath> for Document {
    fn state(&self, _: &PathBuf, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, path: PathBuf, _: &mut Context) -> Response<Result<(), String>> {
        match self.open_path(path) {
            Ok(()) => Response::changed(Ok(())),
            Err(error) => Response::output(Err(error.to_string())),
        }
    }
}

impl Target<OpenCanceled> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        Response::output(())
    }
}

impl Target<SaveFile> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        if self.is_dirty() {
            command::State::enabled()
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<Result<(), String>> {
        let Some(path) = self.path.clone() else {
            return Response::output(Ok(())).with_effect(Effect::SaveFileDialog);
        };

        match self.save_to(path) {
            Ok(()) => Response::changed(Ok(())),
            Err(error) => Response::output(Err(error.to_string())),
        }
    }
}

impl Target<SaveAsFile> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        Response::output(()).with_effect(Effect::SaveFileDialog)
    }
}

impl Target<SaveToPath> for Document {
    fn state(&self, _: &PathBuf, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, path: PathBuf, _: &mut Context) -> Response<Result<(), String>> {
        match self.save_to(path) {
            Ok(()) => Response::changed(Ok(())),
            Err(error) => Response::output(Err(error.to_string())),
        }
    }
}

impl Target<SaveCanceled> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        Response::output(())
    }
}
