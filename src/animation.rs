use std::time::{Duration, Instant};

use winit::event_loop::ControlFlow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Frame {
    now: Instant,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Schedule {
    #[default]
    Idle,
    At(Instant),
    NextFrame,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Easing {
    Linear,
    EaseOutCubic,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Transition<T> {
    from: T,
    to: T,
    started_at: Instant,
    duration: Duration,
    easing: Easing,
}

impl Frame {
    pub(crate) fn new(now: Instant) -> Self {
        Self { now }
    }

    pub(crate) fn now(self) -> Instant {
        self.now
    }
}

impl Schedule {
    pub(crate) fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::NextFrame, _) | (_, Self::NextFrame) => Self::NextFrame,
            (Self::Idle, schedule) | (schedule, Self::Idle) => schedule,
            (Self::At(a), Self::At(b)) => Self::At(a.min(b)),
        }
    }

    pub(crate) fn is_due(self, now: Instant) -> bool {
        match self {
            Self::Idle => false,
            Self::At(deadline) => deadline <= now,
            Self::NextFrame => true,
        }
    }

    pub(crate) fn control_flow(self, now: Instant) -> ControlFlow {
        match self {
            Self::Idle => ControlFlow::Wait,
            Self::At(deadline) if deadline > now => ControlFlow::WaitUntil(deadline),
            Self::At(_) | Self::NextFrame => ControlFlow::Poll,
        }
    }
}

#[allow(dead_code)]
impl Easing {
    pub(crate) fn sample(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match self {
            Self::Linear => t,
            Self::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
        }
    }
}

#[allow(dead_code)]
impl Transition<f32> {
    pub(crate) fn new(
        from: f32,
        to: f32,
        started_at: Instant,
        duration: Duration,
        easing: Easing,
    ) -> Self {
        Self {
            from,
            to,
            started_at,
            duration,
            easing,
        }
    }

    pub(crate) fn settled(value: f32, now: Instant, duration: Duration, easing: Easing) -> Self {
        Self::new(value, value, now, duration, easing)
    }

    pub(crate) fn target(self) -> f32 {
        self.to
    }

    pub(crate) fn value_at(self, now: Instant) -> f32 {
        let progress = self.progress_at(now);
        let eased = self.easing.sample(progress);

        self.from + (self.to - self.from) * eased
    }

    pub(crate) fn retarget(&mut self, to: f32, now: Instant) {
        if self.to == to {
            return;
        }

        self.from = self.value_at(now);
        self.to = to;
        self.started_at = now;
    }

    pub(crate) fn is_animating_at(self, now: Instant) -> bool {
        self.from != self.to && self.progress_at(now) < 1.0
    }

    fn progress_at(self, now: Instant) -> f32 {
        if self.duration.is_zero() {
            return 1.0;
        }

        now.saturating_duration_since(self.started_at).as_secs_f32() / self.duration.as_secs_f32()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_schedule_stays_idle_when_merged_with_idle() {
        assert_eq!(Schedule::Idle.merge(Schedule::Idle), Schedule::Idle);
    }

    #[test]
    fn earliest_deadline_wins_when_merging_timers() {
        let now = Instant::now();
        let earlier = now + Duration::from_millis(10);
        let later = now + Duration::from_millis(20);

        assert_eq!(
            Schedule::At(later).merge(Schedule::At(earlier)),
            Schedule::At(earlier)
        );
    }

    #[test]
    fn next_frame_takes_precedence_over_timers() {
        let timer = Schedule::At(Instant::now() + Duration::from_secs(1));

        assert_eq!(Schedule::NextFrame.merge(timer), Schedule::NextFrame);
        assert_eq!(timer.merge(Schedule::NextFrame), Schedule::NextFrame);
    }

    #[test]
    fn schedule_maps_to_event_loop_control_flow() {
        let now = Instant::now();
        let deadline = now + Duration::from_millis(10);

        assert_eq!(Schedule::Idle.control_flow(now), ControlFlow::Wait);
        assert_eq!(
            Schedule::At(deadline).control_flow(now),
            ControlFlow::WaitUntil(deadline)
        );
        assert_eq!(Schedule::At(now).control_flow(now), ControlFlow::Poll);
        assert_eq!(Schedule::NextFrame.control_flow(now), ControlFlow::Poll);
    }

    #[test]
    fn ease_out_cubic_reaches_endpoints() {
        assert_eq!(Easing::EaseOutCubic.sample(0.0), 0.0);
        assert_eq!(Easing::EaseOutCubic.sample(1.0), 1.0);
        assert!(Easing::EaseOutCubic.sample(0.5) > 0.5);
    }

    #[test]
    fn transition_interpolates_and_retargets_from_current_value() {
        let now = Instant::now();
        let duration = Duration::from_millis(100);
        let mut transition = Transition::new(1.0, 2.0, now, duration, Easing::Linear);

        assert_eq!(transition.value_at(now), 1.0);
        assert_eq!(transition.value_at(now + duration), 2.0);

        transition.retarget(1.0, now + Duration::from_millis(50));

        assert_eq!(transition.target(), 1.0);
        assert_eq!(transition.value_at(now + Duration::from_millis(50)), 1.5);
        assert!(transition.is_animating_at(now + Duration::from_millis(75)));
        assert!(!transition.is_animating_at(now + Duration::from_millis(200)));
    }
}
