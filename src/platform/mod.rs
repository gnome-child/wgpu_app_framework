use super::{host, pointer, runtime, session, shell, state::State, view};
use crate::{animation, window};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

mod backend;
mod error;
mod event;
mod native;
mod runner;

pub(crate) use backend::ResidencyCandidateRetirement;
pub use backend::{Backend, PresentResult, Presented, Window};
pub use error::{Error, RunError};
pub use event::{
    Events, key, key_text, modifiers, point_from_physical, scroll_delta, size_from_physical,
};
pub use native::{Native, NativeContext, NativeError};
#[cfg(test)]
pub(crate) use runner::RunnerEvent;
#[cfg(test)]
pub(super) use runner::file_dialog_selected;
pub use runner::{Runner, run};

pub fn launch<M: State, E: Send + 'static>(
    app: runtime::Runtime<M, E, view::View>,
) -> Result<(), RunError<NativeError>> {
    run(native_shell(app))
}

pub(crate) fn native_shell<M: State, E: Send + 'static>(
    app: runtime::Runtime<M, E, view::View>,
) -> shell::Shell<M, E> {
    shell::Shell::new(app.with_system_clipboard_default())
}

pub struct Platform<M: State, E: Send + 'static = (), B = ()> {
    host: host::Host<M, E>,
    backend: B,
    active_requests: Vec<session::Request>,
    active_cursors: Vec<pointer::Update>,
    poll_scheduled: bool,
    presentation_continuations: HashSet<window::Id>,
    presentation_continuation_deadlines: HashMap<window::Id, Instant>,
    redraw_requests: RedrawRequests,
    animation_schedule: animation::Schedule,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct RedrawRequests {
    in_flight: HashSet<window::Id>,
}

impl RedrawRequests {
    fn begin(&mut self, window: window::Id) -> bool {
        self.in_flight.insert(window)
    }

    fn delivered(&mut self, window: window::Id) {
        self.in_flight.remove(&window);
    }
}

impl<M: State, E: Send + 'static, B: Backend> Platform<M, E, B> {
    pub fn new(shell: shell::Shell<M, E>, backend: B) -> Self {
        Self::with_host(host::Host::new(shell), backend)
    }

    pub fn with_host(host: host::Host<M, E>, backend: B) -> Self {
        Self {
            host,
            backend,
            active_requests: Vec::new(),
            active_cursors: Vec::new(),
            poll_scheduled: false,
            presentation_continuations: HashSet::new(),
            presentation_continuation_deadlines: HashMap::new(),
            redraw_requests: RedrawRequests::default(),
            animation_schedule: animation::Schedule::Idle,
        }
    }

    pub fn host(&self) -> &host::Host<M, E> {
        &self.host
    }

    pub fn host_mut(&mut self) -> &mut host::Host<M, E> {
        &mut self.host
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    pub fn into_parts(self) -> (host::Host<M, E>, B) {
        (self.host, self.backend)
    }

    pub(crate) fn animation_schedule(&self) -> animation::Schedule {
        let continuation = self
            .presentation_continuation_deadlines
            .values()
            .copied()
            .min()
            .map_or(animation::Schedule::Idle, animation::Schedule::At);
        self.animation_schedule.merge(continuation)
    }

    pub(crate) fn runtime_poll_scheduled(&self) -> bool {
        self.poll_scheduled
    }

    pub fn start(&mut self) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        let mut context: B::Context<'_> = Default::default();
        self.handle_event_with(&mut context, host::Event::Started)
    }

    pub fn poll(&mut self) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        let mut context: B::Context<'_> = Default::default();
        self.handle_event_with(&mut context, host::Event::Poll)
    }

    pub fn drain(&mut self) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        let mut context: B::Context<'_> = Default::default();
        self.drain_with(&mut context)
    }

    pub fn handle_event(&mut self, event: host::Event) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        let mut context: B::Context<'_> = Default::default();
        self.handle_event_with(&mut context, event)
    }

    pub fn start_with(&mut self, context: &mut B::Context<'_>) -> Result<(), Error<B::Error>> {
        self.handle_event_with(context, host::Event::Started)
    }

    pub fn poll_with(&mut self, context: &mut B::Context<'_>) -> Result<(), Error<B::Error>> {
        self.handle_event_with(context, host::Event::Poll)
    }

    #[cfg(test)]
    pub(crate) fn continue_presentations(&mut self) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        let mut context: B::Context<'_> = Default::default();
        self.continue_presentations_with(&mut context)
    }

    #[cfg(test)]
    pub(crate) fn continue_presentations_with(
        &mut self,
        context: &mut B::Context<'_>,
    ) -> Result<(), Error<B::Error>> {
        let windows = self
            .presentation_continuations
            .iter()
            .copied()
            .collect::<Vec<_>>();
        for window in windows {
            self.continue_presentation_with(context, window)?;
        }
        Ok(())
    }

    fn continue_presentation_with(
        &mut self,
        context: &mut B::Context<'_>,
        window: window::Id,
    ) -> Result<(), Error<B::Error>> {
        self.cancel_presentation_continuation(window);
        let result = self
            .backend
            .resume_presentation(context, window)
            .map_err(Error::Backend)?;
        if let Some(result) = result {
            self.apply_present_result(context, result)
                .map_err(Error::Backend)?;
        }
        let cursor_updates = self.host.shell_mut().runtime_mut().take_cursor_updates();
        self.sync_cursors(context, &cursor_updates)
            .map_err(Error::Backend)?;
        Ok(())
    }

    pub fn drain_with(&mut self, context: &mut B::Context<'_>) -> Result<(), Error<B::Error>> {
        self.sync_overlay_capabilities();
        let work = self.host.drain();
        self.apply_work(context, &work).map_err(Error::Backend)
    }

    pub fn handle_event_with(
        &mut self,
        context: &mut B::Context<'_>,
        event: host::Event,
    ) -> Result<(), Error<B::Error>> {
        let redraw = match &event {
            host::Event::Window {
                window,
                event: host::WindowEvent::RedrawRequested,
            } => Some(*window),
            _ => None,
        };
        if let Some(window) = redraw {
            self.redraw_requests.delivered(window);
            let progress_expected = self.presentation_continuations.contains(&window)
                || self
                    .host
                    .shell()
                    .runtime()
                    .session()
                    .window(window)
                    .is_some_and(crate::session::Window::redraw_requested);
            self.host.shell_mut().runtime_mut().record_redraw_delivered(
                window,
                Instant::now(),
                progress_expected,
            );
        }
        if let Some(window) = redraw
            && self.presentation_continuations.contains(&window)
            && !self
                .host
                .shell()
                .runtime()
                .session()
                .window(window)
                .is_some_and(crate::session::Window::redraw_requested)
        {
            return self.continue_presentation_with(context, window);
        }
        if matches!(&event, host::Event::Poll) {
            self.poll_scheduled = false;
            let now = Instant::now();
            let due = self
                .presentation_continuation_deadlines
                .iter()
                .filter_map(|(window, deadline)| (*deadline <= now).then_some(*window))
                .collect::<Vec<_>>();
            for window in due {
                self.continue_presentation_with(context, window)?;
            }
        }

        self.sync_overlay_capabilities();
        let work = self.host.handle_event(event).map_err(Error::Framework)?;
        self.apply_work(context, &work).map_err(Error::Backend)?;
        if let Some(window) = redraw
            && self.presentation_continuations.contains(&window)
            && !self
                .presentation_continuation_deadlines
                .contains_key(&window)
        {
            self.schedule_presentation_continuation(context, window)
                .map_err(Error::Backend)?;
        }
        Ok(())
    }

    fn apply_work(
        &mut self,
        context: &mut B::Context<'_>,
        work: &shell::Work,
    ) -> Result<(), B::Error> {
        for window in work.closed_windows() {
            self.cancel_presentation_continuation(*window);
            self.redraw_requests.delivered(*window);
            log::debug!("closing backend window: {window:?}");
            self.backend.close_window(context, *window)?;
        }
        self.active_cursors
            .retain(|update| !work.closed_windows().contains(&update.window()));

        for window in work.opened_windows() {
            log::debug!(
                "opening backend window {:?}: title={:?}, size={:?}",
                window.id(),
                window.title(),
                window.size()
            );
            self.backend
                .open_window(context, &Window::from_shell(window))?;
        }

        for window in work.redraw_windows() {
            self.request_backend_redraw(context, *window)?;
        }

        let synchronized_popup_parents = work
            .presentations()
            .iter()
            .map(shell::Presentation::window)
            .collect::<Vec<_>>();
        if let Some(popup_presentations) = work.popup_presentations() {
            self.backend.present_overlay_popups(
                context,
                &synchronized_popup_parents,
                popup_presentations,
            )?;
        }
        let mut cursor_updates = work.cursor_updates().to_vec();
        for presentation in work.presentations() {
            self.cancel_presentation_continuation(presentation.window());
            let result = self.backend.present(context, presentation)?;
            self.apply_present_result(context, result)?;
        }
        cursor_updates.extend(self.host.shell_mut().runtime_mut().take_cursor_updates());

        self.sync_cursors(context, &cursor_updates)?;
        self.sync_requests(context, work.requests())?;
        self.sync_poll(context, work.needs_poll())?;
        self.animation_schedule = work.animation_schedule();
        self.backend.maintain(context)?;

        Ok(())
    }

    fn apply_present_result(
        &mut self,
        context: &mut B::Context<'_>,
        result: PresentResult,
    ) -> Result<(), B::Error> {
        match result {
            PresentResult::Presented(presented) => {
                let window = presented.window();
                let epoch = presented.epoch();
                let present_submitted = presented.present_submitted();
                let property_serial = presented.property_serial();
                let ime_projection = presented.ime_projection();
                self.cancel_presentation_continuation(window);
                let retry = self.finish_presented(presented, false);
                self.apply_present_submitted_ime(
                    context,
                    window,
                    epoch,
                    property_serial,
                    ime_projection,
                    present_submitted,
                )?;
                if retry {
                    self.request_backend_redraw(context, window)?;
                }
            }
            PresentResult::PresentedAndDeferred(presented) => {
                let window = presented.window();
                let epoch = presented.epoch();
                let present_submitted = presented.present_submitted();
                let property_serial = presented.property_serial();
                let ime_projection = presented.ime_projection();
                self.finish_presented(presented, false);
                self.apply_present_submitted_ime(
                    context,
                    window,
                    epoch,
                    property_serial,
                    ime_projection,
                    present_submitted,
                )?;
                self.schedule_presentation_continuation(context, window)?;
            }
            PresentResult::ActiveRefreshedAndDeferred(presented) => {
                let window = presented.window();
                let epoch = presented.epoch();
                let present_submitted = presented.present_submitted();
                let property_serial = presented.property_serial();
                let ime_projection = presented.ime_projection();
                self.finish_presented(presented, true);
                self.apply_present_submitted_ime(
                    context,
                    window,
                    epoch,
                    property_serial,
                    ime_projection,
                    present_submitted,
                )?;
                self.schedule_presentation_continuation(context, window)?;
            }
            PresentResult::Deferred { window, retry_at } => {
                self.schedule_presentation_continuation_at(window, retry_at);
            }
        }
        Ok(())
    }

    fn apply_present_submitted_ime(
        &mut self,
        context: &mut B::Context<'_>,
        window: window::Id,
        epoch: window::PresentationEpoch,
        property_serial: crate::scene::PropertySerial,
        ime_projection: crate::ime::Projection,
        present_submitted: bool,
    ) -> Result<(), B::Error> {
        if !present_submitted
            || self
                .host
                .shell()
                .runtime()
                .session()
                .window(window)
                .is_none_or(|window| !window.present_submitted_matches(epoch, property_serial))
        {
            return Ok(());
        }
        let update = self.host.shell().runtime().presented_ime_update(
            window,
            property_serial,
            ime_projection,
        );
        self.backend.set_ime(context, update)?;
        Ok(())
    }

    fn finish_presented(&mut self, presented: Presented, refreshes_active: bool) -> bool {
        let residency_retirement = presented.residency_retirement();
        let (presented, report) = presented.into_parts();
        let window = presented.window();
        let retry = if refreshes_active {
            self.host.shell_mut().runtime_mut().finish_active_refresh(
                window,
                presented.epoch(),
                presented.invalidation(),
                presented.layout(),
                presented.stack(),
                report,
            )
        } else {
            self.host.shell_mut().runtime_mut().finish_render_report(
                window,
                presented.epoch(),
                presented.invalidation(),
                presented.layout(),
                presented.stack(),
                presented.property_only(),
                report,
            )
        };
        let retirement_retry = match residency_retirement {
            Some(backend::ResidencyCandidateRetirement::SupersedeFront(epoch)) => self
                .host
                .shell_mut()
                .runtime_mut()
                .supersede_residency_candidate(window, epoch),
            Some(backend::ResidencyCandidateRetirement::PreemptProactive(epoch)) => self
                .host
                .shell_mut()
                .runtime_mut()
                .preempt_proactive_residency_candidate(window, epoch),
            Some(backend::ResidencyCandidateRetirement::CancelPipeline(epoch)) => {
                self.host
                    .shell_mut()
                    .runtime_mut()
                    .cancel_residency_pipeline(window, epoch);
                false
            }
            None => false,
        };
        retry || retirement_retry
    }

    fn schedule_presentation_continuation(
        &mut self,
        context: &mut B::Context<'_>,
        window: window::Id,
    ) -> Result<(), B::Error> {
        let converted_from_deadline = self
            .presentation_continuation_deadlines
            .remove(&window)
            .is_some();
        if !self.presentation_continuations.insert(window) && !converted_from_deadline {
            return Ok(());
        }

        self.request_backend_redraw(context, window)
    }

    fn schedule_presentation_continuation_at(&mut self, window: window::Id, retry_at: Instant) {
        self.presentation_continuations.insert(window);
        self.presentation_continuation_deadlines
            .entry(window)
            .and_modify(|deadline| *deadline = (*deadline).min(retry_at))
            .or_insert(retry_at);
    }

    fn cancel_presentation_continuation(&mut self, window: window::Id) {
        self.presentation_continuations.remove(&window);
        self.presentation_continuation_deadlines.remove(&window);
    }

    fn request_backend_redraw(
        &mut self,
        context: &mut B::Context<'_>,
        window: window::Id,
    ) -> Result<(), B::Error> {
        if !self.redraw_requests.begin(window) {
            return Ok(());
        }
        self.host
            .shell_mut()
            .runtime_mut()
            .record_redraw_requested(window, Instant::now());
        if let Err(error) = self.backend.request_redraw(context, window) {
            self.redraw_requests.delivered(window);
            return Err(error);
        }
        Ok(())
    }

    fn sync_overlay_capabilities(&mut self) {
        let capabilities = self.backend.overlay_capabilities();
        self.host
            .shell_mut()
            .runtime_mut()
            .set_overlay_capabilities(capabilities);
    }

    fn sync_requests(
        &mut self,
        context: &mut B::Context<'_>,
        requests: &[session::Request],
    ) -> Result<(), B::Error> {
        self.active_requests
            .retain(|request| requests.contains(request));

        for request in requests {
            if self.active_requests.contains(request) {
                continue;
            }

            log::debug!("submitting backend request: {request:?}");
            self.backend.request(context, *request)?;
            self.active_requests.push(*request);
        }

        Ok(())
    }

    fn sync_cursors(
        &mut self,
        context: &mut B::Context<'_>,
        updates: &[pointer::Update],
    ) -> Result<(), B::Error> {
        for update in updates {
            if self.active_cursors.iter().any(|active| {
                active.window() == update.window() && active.cursor() == update.cursor()
            }) {
                continue;
            }

            log::debug!(
                "setting backend cursor for window {:?}: {:?}",
                update.window(),
                update.cursor()
            );
            self.backend
                .set_cursor(context, update.window(), update.cursor())?;
            self.active_cursors
                .retain(|active| active.window() != update.window());
            self.active_cursors.push(*update);
        }

        Ok(())
    }

    fn sync_poll(
        &mut self,
        context: &mut B::Context<'_>,
        needs_poll: bool,
    ) -> Result<(), B::Error> {
        if !needs_poll {
            self.poll_scheduled = false;
            return Ok(());
        }

        if self.poll_scheduled {
            return Ok(());
        }

        log::debug!("scheduling backend poll");
        self.backend.schedule_poll(context)?;
        self.poll_scheduled = true;
        Ok(())
    }
}
