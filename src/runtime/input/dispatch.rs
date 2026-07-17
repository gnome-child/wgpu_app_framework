use super::super::Runtime;
use crate::{
    command::Error, context as command_context, input, interaction, response, session, state,
    window,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ScrollTransition {
    Unchanged,
    PropertyTick(interaction::ScrollOffset),
    NeedsResidency {
        desired: interaction::ScrollOffset,
        resident_accepted: interaction::ScrollOffset,
        schedule_candidate: bool,
    },
}

impl ScrollTransition {
    fn offset(self) -> Option<interaction::ScrollOffset> {
        match self {
            Self::Unchanged => None,
            Self::PropertyTick(offset)
            | Self::NeedsResidency {
                desired: offset, ..
            } => Some(offset),
        }
    }

    fn effect(self) -> response::Effect {
        match self {
            Self::Unchanged | Self::PropertyTick(_) => response::Effect::None,
            Self::NeedsResidency {
                schedule_candidate: true,
                ..
            } => response::Effect::Rebuild,
            Self::NeedsResidency {
                schedule_candidate: false,
                ..
            } => response::Effect::None,
        }
    }

    pub(super) fn requests_redraw(self) -> bool {
        matches!(
            self,
            Self::PropertyTick(_)
                | Self::NeedsResidency {
                    schedule_candidate: true,
                    ..
                }
        )
    }

    fn then(self, next: Self) -> Self {
        match (self, next) {
            (
                Self::NeedsResidency {
                    schedule_candidate: left_schedule,
                    ..
                },
                Self::NeedsResidency {
                    desired,
                    resident_accepted,
                    schedule_candidate,
                },
            ) => Self::NeedsResidency {
                desired,
                resident_accepted,
                schedule_candidate: left_schedule || schedule_candidate,
            },
            (Self::NeedsResidency { .. }, Self::Unchanged | Self::PropertyTick(_)) => self,
            (
                Self::Unchanged | Self::PropertyTick(_),
                Self::NeedsResidency {
                    desired,
                    resident_accepted,
                    schedule_candidate,
                },
            ) => Self::NeedsResidency {
                desired,
                resident_accepted,
                schedule_candidate,
            },
            (Self::PropertyTick(_), Self::Unchanged) => self,
            (Self::Unchanged | Self::PropertyTick(_), Self::PropertyTick(offset)) => {
                Self::PropertyTick(offset)
            }
            (Self::Unchanged, Self::Unchanged) => Self::Unchanged,
        }
    }
}

pub(in crate::runtime) struct ScrollDispatch {
    input: input::Outcome,
    scroll: interaction::ScrollOutcome,
}

impl ScrollDispatch {
    pub(in crate::runtime) fn into_input(self) -> input::Outcome {
        let Self { input, scroll: _ } = self;
        input
    }

    pub(in crate::runtime) fn scroll_outcome(&self) -> interaction::ScrollOutcome {
        self.scroll
    }
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub fn handle_input(
        &mut self,
        window: window::Id,
        input: input::Input,
    ) -> std::result::Result<input::Outcome, Error> {
        if !self.session.contains(window) {
            return Ok(input::Outcome::ignored());
        }

        match input {
            input::Input::Cancel => {
                if self.session.close_command_palette(window) {
                    return Ok(self.window_outcome(window, false, response::Effect::Rebuild));
                }

                if self.session.close_menu(window) {
                    return Ok(self.window_outcome(window, false, response::Effect::Rebuild));
                }

                let active_text_focus = self
                    .session
                    .interaction(window)
                    .and_then(|interaction| interaction.text_input().target())
                    .and_then(session::Focus::from_text_target);
                if let Some(focus) = active_text_focus {
                    self.session.clear_text_draft(window, focus);
                    self.session
                        .request_invalidation(window, response::effect::Invalidation::Rebuild);
                    return Ok(self.window_outcome(window, false, response::Effect::Rebuild));
                }

                if self.session.clear_text_input(window) {
                    return Ok(self.window_outcome(window, false, response::Effect::Rebuild));
                }

                if self.session.cancel_pointer(window) {
                    self.finish_pointer_gesture();
                    return Ok(self.window_outcome(window, false, response::Effect::Paint));
                }

                self.clear_focus_committing_text_box(window)
            }
            input::Input::Focus(focus) => self.focus_committing_text_box(window, focus),
            input::Input::PointerMove(target) => {
                let hover_tip_was_visible = self.session.hover_tip_visible(window);
                let menu_switch = target
                    .as_ref()
                    .and_then(interaction::Target::as_menu)
                    .is_some_and(|menu| {
                        self.session
                            .interaction(window)
                            .and_then(|interaction| interaction.open_menu())
                            .is_some_and(|open| *open != menu)
                    });
                let effect = if self.session.pointer_move(window, target) {
                    if menu_switch || hover_tip_was_visible {
                        response::Effect::Rebuild
                    } else {
                        response::Effect::Paint
                    }
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::PointerDown(target) => self.handle_pointer_down_input(
                window,
                target,
                interaction::pointer::PressIntent::Activate,
                crate::pointer::Cursor::Default,
            ),
            input::Input::PointerManipulate(target) => self.handle_pointer_down_input(
                window,
                target,
                interaction::pointer::PressIntent::Manipulate,
                crate::pointer::Cursor::Default,
            ),
            input::Input::PointerDrag(hovered) => {
                let hover_tip_was_visible = self.session.hover_tip_visible(window);
                let effect = if self.session.pointer_move(window, hovered) {
                    if hover_tip_was_visible {
                        response::Effect::Rebuild
                    } else {
                        response::Effect::Paint
                    }
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::PointerUp(target) => self.handle_pointer_up_input(window, target, true),
            input::Input::PointerLeft => {
                let hover_tip_was_visible = self.session.hover_tip_visible(window);
                let effect = if self.session.pointer_left(window) {
                    if hover_tip_was_visible {
                        response::Effect::Rebuild
                    } else {
                        response::Effect::Paint
                    }
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::Scroll { target, delta } => {
                let event = delta.session_event(interaction::ScrollSource::Programmatic);
                Ok(self
                    .dispatch_scroll_event(window, vec![target], event)
                    .into_input())
            }
            input::Input::ScrollTo { target, offset } => Ok(self.scroll_to_with_source(
                window,
                target,
                offset,
                interaction::ScrollSource::Programmatic,
            )),
            input::Input::ToggleMenu(menu) => {
                let effect = if self.session.toggle_menu(window, menu) {
                    response::Effect::Rebuild
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::FilePathSelected(path) => self.handle_file_path_selected(window, path),
            input::Input::Shortcut(shortcut) => self.handle_shortcut(window, shortcut),
            input::Input::KeyDown {
                key,
                modifiers,
                text,
            } => self.handle_key_down(window, key, modifiers, text),
            input::Input::TextSelection(operation) => {
                self.handle_text_selection(window, operation, command_context::Source::Keyboard)
            }
            input::Input::TextEdit(edit) => {
                self.handle_text_edit(window, edit, command_context::Source::Keyboard)
            }
            input::Input::TextCommit(text) => self.handle_text_commit(window, text),
            input::Input::TextPreedit(preedit) => {
                let Some(focus) = self.session.focused(window) else {
                    return Ok(input::Outcome::ignored());
                };
                let Some(target) = self.text_input_target(window, focus) else {
                    return Ok(input::Outcome::ignored());
                };
                let changed = self.session.set_text_preedit_for(window, target, preedit);
                let effect = if changed {
                    response::Effect::Layout
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::TextDrop(drop) => self.handle_text_drop(window, drop),
        }
    }

    pub(in crate::runtime) fn scroll_to_with_source(
        &mut self,
        window: window::Id,
        target: interaction::Target,
        offset: interaction::ScrollOffset,
        source: interaction::ScrollSource,
    ) -> input::Outcome {
        let started = std::time::Instant::now();
        self.kinetic_scrolls.remove(&window);
        self.session.handle_scroll_session(
            window,
            &target,
            interaction::ScrollEvent::new(
                source,
                interaction::ScrollUnit::Pixel,
                started,
                interaction::ScrollPhase::Update,
                interaction::ScrollDelta::default(),
            ),
        );
        let transition = self.apply_scroll_transition(
            window,
            target,
            interaction::ScrollUpdate::Absolute(offset),
        );
        let scrolled = transition.offset().is_some();
        self.record_scroll_input(window, transition, scrolled, started.elapsed());

        self.window_outcome(window, false, transition.effect())
    }

    pub(in crate::runtime) fn apply_scroll_operation(
        &mut self,
        window: window::Id,
        target: interaction::Target,
        axis: interaction::ScrollbarAxis,
        operation: interaction::ScrollOperation,
        reversed: bool,
        source: interaction::ScrollSource,
    ) -> Option<input::Outcome> {
        let current = self
            .session
            .interaction(window)
            .map(|state| state.scroll().desired_offset(&target))?;
        let offset = self
            .session
            .scroll_operation_offset(window, &target, axis, operation, reversed)?;
        (offset != current).then(|| self.scroll_to_with_source(window, target, offset, source))
    }

    pub(in crate::runtime) fn dispatch_scroll_event(
        &mut self,
        window: window::Id,
        targets: Vec<interaction::Target>,
        event: interaction::ScrollEvent,
    ) -> ScrollDispatch {
        let started = std::time::Instant::now();
        let phase = event.phase();
        let source = event.source();
        if matches!(
            phase,
            interaction::ScrollPhase::Begin
                | interaction::ScrollPhase::Update
                | interaction::ScrollPhase::Cancel
        ) {
            self.kinetic_scrolls.remove(&window);
        }
        let original = event.delta();
        let mut outcome = interaction::ScrollOutcome::unconsumed(original);
        let mut transition = ScrollTransition::Unchanged;
        let mut handled = false;

        for target in &targets {
            let target_event = event.with_delta(outcome.remaining());
            match self
                .session
                .handle_scroll_session(window, target, target_event)
            {
                interaction::ScrollSessionDisposition::Ignored => continue,
                interaction::ScrollSessionDisposition::Tracked => {
                    handled = true;
                }
                interaction::ScrollSessionDisposition::Apply(delta) => {
                    handled = true;
                    let before = self
                        .session
                        .interaction(window)
                        .map(|state| state.scroll().desired_offset(target))
                        .unwrap_or_default();
                    let next = self.apply_scroll_transition(
                        window,
                        target.clone(),
                        interaction::ScrollUpdate::Relative(delta),
                    );
                    let after = self
                        .session
                        .interaction(window)
                        .map(|state| state.scroll().desired_offset(target))
                        .unwrap_or(before);
                    outcome = outcome.then(interaction::ScrollOutcome::from_offsets(
                        delta, before, after,
                    ));
                    transition = transition.then(next);
                }
            }
        }

        if let Some(outermost) = targets.last()
            && handled
        {
            outcome = self.session.resolve_scroll_edge(window, outermost, outcome);
        }

        if phase == interaction::ScrollPhase::End
            && matches!(
                source,
                interaction::ScrollSource::Touchpad | interaction::ScrollSource::Touchscreen
            )
            && let Some(terminal) = targets.first().and_then(|target| {
                self.session
                    .interaction(window)
                    .and_then(|state| state.scroll().kinetic_velocity(target))
            })
            && kinetic_velocity_is_significant(terminal)
        {
            let terminal = bounded_kinetic_velocity(terminal);
            for target in &targets {
                self.session.handle_scroll_session(
                    window,
                    target,
                    interaction::ScrollEvent::new(
                        source,
                        event.unit(),
                        event.timestamp(),
                        interaction::ScrollPhase::End,
                        interaction::ScrollDelta::default(),
                    )
                    .with_velocity(terminal),
                );
            }
            self.kinetic_scrolls.insert(
                window,
                super::super::KineticScroll {
                    targets: targets.clone(),
                    source,
                    velocity: terminal,
                    last_tick: event.timestamp(),
                },
            );
        }

        let scrolled = !outcome.applied().is_zero();
        if phase != interaction::ScrollPhase::Deceleration {
            self.record_scroll_input(window, transition, scrolled, started.elapsed());
        }
        let input = if handled {
            self.window_outcome(window, false, transition.effect())
        } else {
            input::Outcome::ignored()
        };
        ScrollDispatch {
            input,
            scroll: outcome,
        }
    }

    pub(in crate::runtime) fn advance_kinetic_scrolls(&mut self, now: std::time::Instant) {
        const DRAG_PER_SECOND: f64 = 8.0;
        const STOP_VELOCITY: f64 = 4.0;
        const MAX_STEP_SECONDS: f64 = 0.05;

        let active = self
            .kinetic_scrolls
            .iter()
            .map(|(window, kinetic)| (*window, kinetic.clone()))
            .collect::<Vec<_>>();
        for (window, kinetic) in active {
            if !self.session.contains(window) {
                self.kinetic_scrolls.remove(&window);
                continue;
            }
            if now < kinetic.last_tick + super::super::KINETIC_FRAME_INTERVAL {
                continue;
            }
            let seconds = now
                .saturating_duration_since(kinetic.last_tick)
                .as_secs_f64()
                .min(MAX_STEP_SECONDS);
            if seconds <= 0.0 {
                continue;
            }

            let decay = (-DRAG_PER_SECOND * seconds).exp();
            let mut next_velocity = interaction::ScrollDelta::from_logical_pixels(
                kinetic.velocity.x() * decay,
                kinetic.velocity.y() * decay,
            );
            let delta = interaction::ScrollDelta::from_logical_pixels(
                (kinetic.velocity.x() - next_velocity.x()) / DRAG_PER_SECOND,
                (kinetic.velocity.y() - next_velocity.y()) / DRAG_PER_SECOND,
            );
            let dispatch = self.dispatch_scroll_event(
                window,
                kinetic.targets.clone(),
                interaction::ScrollEvent::new(
                    kinetic.source,
                    interaction::ScrollUnit::Pixel,
                    now,
                    interaction::ScrollPhase::Deceleration,
                    delta,
                )
                .with_velocity(next_velocity),
            );
            let remainder = dispatch.scroll_outcome().remaining();
            let _ = dispatch.into_input();
            next_velocity = interaction::ScrollDelta::from_logical_pixels(
                if remainder.x() == 0.0 && next_velocity.x().abs() >= STOP_VELOCITY {
                    next_velocity.x()
                } else {
                    0.0
                },
                if remainder.y() == 0.0 && next_velocity.y().abs() >= STOP_VELOCITY {
                    next_velocity.y()
                } else {
                    0.0
                },
            );

            if next_velocity.is_zero() {
                self.kinetic_scrolls.remove(&window);
                let finished = self.dispatch_scroll_event(
                    window,
                    kinetic.targets,
                    interaction::ScrollEvent::new(
                        kinetic.source,
                        interaction::ScrollUnit::Pixel,
                        now,
                        interaction::ScrollPhase::Deceleration,
                        interaction::ScrollDelta::default(),
                    ),
                );
                let _ = finished.into_input();
            } else if let Some(active) = self.kinetic_scrolls.get_mut(&window) {
                active.velocity = next_velocity;
                active.last_tick = now;
            }
        }
    }

    fn apply_scroll_transition(
        &mut self,
        window: window::Id,
        target: interaction::Target,
        update: interaction::ScrollUpdate,
    ) -> ScrollTransition {
        let target_key = target.focus_key();
        let presented = self.presented_layout(window);
        if let Some((maximum, page)) = presented
            .as_ref()
            .and_then(|layout| layout.scroll_adjustment_geometry(&target))
        {
            self.session
                .configure_scroll(window, target.clone(), maximum, page);
        }
        let before = self
            .session
            .interaction(window)
            .map(|interaction| interaction.scroll().desired_offset(&target))
            .unwrap_or_default();
        let Some(requested) = self.session.request_scroll(window, target.clone(), update) else {
            let resident_offset = self
                .session
                .interaction(window)
                .map(|interaction| interaction.scroll().resident_offset(&target))
                .unwrap_or_default();
            self.record_scroll_trace(
                window,
                target_key,
                before,
                before,
                resident_offset,
                false,
                "unchanged",
            );
            return ScrollTransition::Unchanged;
        };
        let offset = presented.as_ref().map_or(requested, |layout| {
            layout.resolve_scroll_offset(&target, requested)
        });
        if offset != requested {
            self.session.request_scroll(
                window,
                target.clone(),
                interaction::ScrollUpdate::Geometry(offset),
            );
        }
        let resident_acceptance = presented
            .as_ref()
            .and_then(|layout| layout.scroll_property_acceptance(&target, before, offset));
        let resident_accepted = resident_acceptance.is_some();
        if offset == before {
            let resident_offset = self
                .session
                .interaction(window)
                .map(|interaction| interaction.scroll().resident_offset(&target))
                .unwrap_or_default();
            self.record_scroll_trace(
                window,
                target_key,
                requested,
                offset,
                resident_offset,
                resident_accepted,
                "unchanged",
            );
            return ScrollTransition::Unchanged;
        }
        if resident_accepted {
            self.session
                .accept_resident_scroll(window, target.clone(), offset);
            self.session.request_property_tick(window);
            if let Some(preparation) =
                resident_acceptance.and_then(|acceptance| acceptance.replenishment())
            {
                if let Some(request) = self
                    .presented_layout(window)
                    .and_then(|layout| layout.residency_replenishment(&target, offset, preparation))
                    .filter(|demand| demand.prepares_proactively())
                {
                    self.install_residency_demand(window, request);
                    self.schedule_residency_candidate(window, true);
                }
            }
            self.record_scroll_trace(
                window,
                target_key,
                requested,
                offset,
                offset,
                true,
                "property-tick",
            );
            ScrollTransition::PropertyTick(offset)
        } else {
            let resident_accepted = self
                .session
                .interaction(window)
                .map(|interaction| interaction.scroll().resident_offset(&target))
                .unwrap_or_default();
            let request = self
                .presented_layout(window)
                .and_then(|layout| layout.residency_demand(&target, offset));
            if let Some(request) = request {
                self.install_residency_demand(window, request);
            }
            let schedule_candidate = self.schedule_residency_candidate(window, false);
            self.record_scroll_trace(
                window,
                target_key,
                requested,
                offset,
                resident_accepted,
                false,
                "needs-residency",
            );
            ScrollTransition::NeedsResidency {
                desired: offset,
                resident_accepted,
                schedule_candidate,
            }
        }
    }

    fn record_scroll_trace(
        &mut self,
        window: window::Id,
        target_key: u64,
        requested: interaction::ScrollOffset,
        clamped: interaction::ScrollOffset,
        resident_offset: interaction::ScrollOffset,
        resident_accepted: bool,
        outcome: &'static str,
    ) {
        let Some(epoch) = self
            .session
            .window(window)
            .map(session::Window::requested_presentation_epoch)
        else {
            return;
        };
        self.diagnostics.get_mut(window).scroll.record_transition(
            epoch,
            target_key,
            requested,
            clamped,
            resident_offset,
            resident_accepted,
            outcome,
        );
    }
}

fn kinetic_velocity_is_significant(velocity: interaction::ScrollDelta) -> bool {
    velocity.x().abs().max(velocity.y().abs()) >= 4.0
}

fn bounded_kinetic_velocity(velocity: interaction::ScrollDelta) -> interaction::ScrollDelta {
    const MAX_VELOCITY: f64 = 20_000.0;
    interaction::ScrollDelta::from_logical_pixels(
        velocity.x().clamp(-MAX_VELOCITY, MAX_VELOCITY),
        velocity.y().clamp(-MAX_VELOCITY, MAX_VELOCITY),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Default)]
    struct NestedHandoffState;

    impl state::State for NestedHandoffState {}

    #[test]
    fn nested_dispatch_hands_fractional_diagonal_remainders_to_each_ancestor() {
        let mut runtime = Runtime::new(NestedHandoffState)
            .started(|cx| {
                cx.open_window(crate::window::Options::new("Nested handoff"));
            })
            .view(|_, _| crate::view::View::new(crate::view::Node::root()));
        runtime.start();
        let window = runtime.session.windows()[0].id();
        let inner = interaction::Target::scroll("handoff.inner", "Inner");
        let middle = interaction::Target::scroll("handoff.middle", "Middle");
        let outer = interaction::Target::scroll("handoff.outer", "Outer");

        runtime.session.configure_scroll(
            window,
            inner.clone(),
            interaction::ScrollOffset::new(10, 0),
            interaction::ScrollOffset::new(10, 10),
        );
        runtime.session.configure_scroll(
            window,
            middle.clone(),
            interaction::ScrollOffset::new(0, 40),
            interaction::ScrollOffset::new(10, 10),
        );
        runtime.session.configure_scroll(
            window,
            outer.clone(),
            interaction::ScrollOffset::new(100, 10),
            interaction::ScrollOffset::new(10, 10),
        );

        let started = std::time::Instant::now();
        let input = interaction::ScrollDelta::from_logical_pixels(30.75, 40.5);
        let dispatch = runtime.dispatch_scroll_event(
            window,
            vec![inner.clone(), middle.clone(), outer.clone()],
            interaction::ScrollEvent::new(
                interaction::ScrollSource::Touchpad,
                interaction::ScrollUnit::Pixel,
                started,
                interaction::ScrollPhase::Begin,
                input,
            ),
        );
        assert!(dispatch.input.is_handled());
        assert_eq!(dispatch.scroll_outcome().applied(), input);
        assert_eq!(
            dispatch.scroll_outcome().remaining(),
            interaction::ScrollDelta::default()
        );

        let scroll = runtime.session.interaction(window).unwrap().scroll();
        assert_eq!(
            scroll.desired_offset(&inner).precise_components_for_test(),
            [10.0, 0.0]
        );
        assert_eq!(
            scroll.desired_offset(&middle).precise_components_for_test(),
            [0.0, 40.0]
        );
        assert_eq!(
            scroll.desired_offset(&outer).precise_components_for_test(),
            [20.75, 0.5]
        );

        let reverse = interaction::ScrollDelta::from_logical_pixels(-15.25, -50.75);
        let dispatch = runtime.dispatch_scroll_event(
            window,
            vec![inner.clone(), middle.clone(), outer.clone()],
            interaction::ScrollEvent::new(
                interaction::ScrollSource::Touchpad,
                interaction::ScrollUnit::Pixel,
                started + std::time::Duration::from_millis(8),
                interaction::ScrollPhase::Update,
                reverse,
            ),
        );
        assert_eq!(
            dispatch.scroll_outcome().applied(),
            interaction::ScrollDelta::from_logical_pixels(-15.25, -40.5)
        );
        assert_eq!(
            dispatch.scroll_outcome().remaining(),
            interaction::ScrollDelta::from_logical_pixels(0.0, -10.25)
        );
        let scroll = runtime.session.interaction(window).unwrap().scroll();
        assert_eq!(
            scroll.desired_offset(&inner).precise_components_for_test(),
            [0.0, 0.0]
        );
        assert_eq!(
            scroll.desired_offset(&middle).precise_components_for_test(),
            [0.0, 0.0]
        );
        assert_eq!(
            scroll.desired_offset(&outer).precise_components_for_test(),
            [15.5, 0.0]
        );
    }

    #[test]
    fn terminal_velocity_drives_runtime_deceleration_until_direct_input_interrupts_it() {
        let mut runtime = Runtime::new(NestedHandoffState)
            .started(|cx| {
                cx.open_window(crate::window::Options::new("Kinetic scroll"));
            })
            .view(|_, _| crate::view::View::new(crate::view::Node::root()));
        runtime.start();
        let window = runtime.session.windows()[0].id();
        let target = interaction::Target::scroll("kinetic.target", "Kinetic");
        runtime.session.configure_scroll(
            window,
            target.clone(),
            interaction::ScrollOffset::new(0, 1_000),
            interaction::ScrollOffset::new(10, 100),
        );

        let started = std::time::Instant::now();
        let begin = runtime.dispatch_scroll_event(
            window,
            vec![target.clone()],
            interaction::ScrollEvent::new(
                interaction::ScrollSource::Touchpad,
                interaction::ScrollUnit::Pixel,
                started,
                interaction::ScrollPhase::Begin,
                interaction::ScrollDelta::default(),
            ),
        );
        assert!(begin.input.is_handled());
        let update = runtime.dispatch_scroll_event(
            window,
            vec![target.clone()],
            interaction::ScrollEvent::new(
                interaction::ScrollSource::Touchpad,
                interaction::ScrollUnit::Pixel,
                started + std::time::Duration::from_millis(10),
                interaction::ScrollPhase::Update,
                interaction::ScrollDelta::from_logical_pixels(0.0, 12.5),
            ),
        );
        assert_eq!(
            update.scroll_outcome().applied(),
            interaction::ScrollDelta::from_logical_pixels(0.0, 12.5)
        );

        let ended_at = started + std::time::Duration::from_millis(20);
        let end = runtime.dispatch_scroll_event(
            window,
            vec![target.clone()],
            interaction::ScrollEvent::new(
                interaction::ScrollSource::Touchpad,
                interaction::ScrollUnit::Pixel,
                ended_at,
                interaction::ScrollPhase::End,
                interaction::ScrollDelta::default(),
            )
            .with_velocity(interaction::ScrollDelta::from_logical_pixels(0.0, 800.0)),
        );
        assert!(end.input.is_handled());
        assert!(runtime.kinetic_scrolls.contains_key(&window));
        assert_eq!(
            runtime.animation_schedule(),
            crate::animation::Schedule::At(ended_at + super::super::super::KINETIC_FRAME_INTERVAL)
        );

        runtime.advance_kinetic_scrolls(ended_at + std::time::Duration::from_millis(16));
        let kinetic_offset = runtime
            .session
            .interaction(window)
            .unwrap()
            .scroll()
            .desired_offset(&target)
            .precise_components_for_test();
        assert!(kinetic_offset[1] > 12.5);
        assert!(runtime.kinetic_scrolls.contains_key(&window));

        let interrupted = runtime.dispatch_scroll_event(
            window,
            vec![target.clone()],
            interaction::ScrollEvent::new(
                interaction::ScrollSource::Touchpad,
                interaction::ScrollUnit::Pixel,
                ended_at + std::time::Duration::from_millis(17),
                interaction::ScrollPhase::Begin,
                interaction::ScrollDelta::from_logical_pixels(0.0, -1.0),
            ),
        );
        assert_eq!(
            interrupted.scroll_outcome().applied(),
            interaction::ScrollDelta::from_logical_pixels(0.0, -1.0)
        );
        assert!(!runtime.kinetic_scrolls.contains_key(&window));
        assert_eq!(
            runtime.animation_schedule(),
            crate::animation::Schedule::Idle
        );
    }

    #[test]
    fn kinetic_boundary_stops_only_the_blocked_axis() {
        let mut runtime = Runtime::new(NestedHandoffState)
            .started(|cx| {
                cx.open_window(crate::window::Options::new("Kinetic boundary"));
            })
            .view(|_, _| crate::view::View::new(crate::view::Node::root()));
        runtime.start();
        let window = runtime.session.windows()[0].id();
        let target = interaction::Target::scroll("kinetic.boundary", "Kinetic Boundary");
        runtime.session.configure_scroll(
            window,
            target.clone(),
            interaction::ScrollOffset::new(1, 1_000),
            interaction::ScrollOffset::new(10, 100),
        );

        let started = std::time::Instant::now();
        let _ = runtime.dispatch_scroll_event(
            window,
            vec![target.clone()],
            interaction::ScrollEvent::new(
                interaction::ScrollSource::Touchpad,
                interaction::ScrollUnit::Pixel,
                started,
                interaction::ScrollPhase::Begin,
                interaction::ScrollDelta::default(),
            ),
        );
        let ended_at = started + std::time::Duration::from_millis(1);
        let _ = runtime.dispatch_scroll_event(
            window,
            vec![target.clone()],
            interaction::ScrollEvent::new(
                interaction::ScrollSource::Touchpad,
                interaction::ScrollUnit::Pixel,
                ended_at,
                interaction::ScrollPhase::End,
                interaction::ScrollDelta::default(),
            )
            .with_velocity(interaction::ScrollDelta::from_logical_pixels(800.0, 800.0)),
        );

        runtime.advance_kinetic_scrolls(ended_at + std::time::Duration::from_millis(16));
        let offset = runtime
            .session
            .interaction(window)
            .unwrap()
            .scroll()
            .desired_offset(&target)
            .precise_components_for_test();
        assert_eq!(offset[0], 1.0);
        assert!(offset[1] > 1.0);
        let kinetic = runtime.kinetic_scrolls.get(&window).unwrap();
        assert_eq!(kinetic.velocity.x(), 0.0);
        assert!(kinetic.velocity.y() > 0.0);
    }
}
