use super::super::state;
use super::{Context, Runtime};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub fn start(&mut self) {
        if self.started_ran {
            return;
        }

        self.started_ran = true;

        let Some(started) = self.started.as_mut() else {
            return;
        };
        let task_sink = self.tasks.sink();
        let mut cx = Context::new(
            &mut self.store,
            &mut self.timeline,
            &mut self.session,
            &mut self.composition,
            &mut self.diagnostics,
            task_sink,
        );

        started(&mut cx);
    }

    pub fn emit(&mut self, event: E) {
        let before = self.revision();
        let Some(handler) = self.event.as_mut() else {
            return;
        };
        let task_sink = self.tasks.sink();
        let mut cx = Context::new(
            &mut self.store,
            &mut self.timeline,
            &mut self.session,
            &mut self.composition,
            &mut self.diagnostics,
            task_sink,
        );

        handler(&mut cx, event);

        if self.revision() != before {
            self.request_all_redraws();
        }
    }
}
