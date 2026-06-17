use std::time::{Duration, Instant};

use winit::event_loop::ControlFlow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Frame {
    now: Instant,
    delta: Duration,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Schedule {
    #[default]
    Idle,
    At(Instant),
    NextFrame,
}

impl Frame {
    pub(crate) fn new(now: Instant, previous: Option<Instant>) -> Self {
        Self {
            now,
            delta: previous.map_or(Duration::ZERO, |previous| {
                now.saturating_duration_since(previous)
            }),
        }
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
}
