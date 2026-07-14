use std::time::{Duration, Instant};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Easing {
    EaseOutCubic,
}

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
}

impl Easing {
    pub(crate) fn sample(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match self {
            Self::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
        }
    }
}

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

    pub(crate) fn from(self) -> f32 {
        self.from
    }

    pub(crate) fn value_at(self, now: Instant) -> f32 {
        let eased = self.eased_progress_at(now);

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

    pub(crate) fn eased_progress_at(self, now: Instant) -> f32 {
        self.easing.sample(self.progress_at(now))
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
    fn ease_out_cubic_reaches_endpoints() {
        assert_eq!(Easing::EaseOutCubic.sample(0.0), 0.0);
        assert_eq!(Easing::EaseOutCubic.sample(1.0), 1.0);
        assert!(Easing::EaseOutCubic.sample(0.5) > 0.5);
    }

    #[test]
    fn transition_interpolates_and_retargets_from_current_value() {
        let now = Instant::now();
        let duration = Duration::from_millis(100);
        let mut transition = Transition::new(1.0, 2.0, now, duration, Easing::EaseOutCubic);

        assert_eq!(transition.value_at(now), 1.0);
        assert_eq!(transition.value_at(now + duration), 2.0);

        transition.retarget(1.0, now + Duration::from_millis(50));

        assert_eq!(transition.target(), 1.0);
        assert_eq!(transition.value_at(now + Duration::from_millis(50)), 1.875);
        assert!(transition.is_animating_at(now + Duration::from_millis(75)));
        assert!(!transition.is_animating_at(now + Duration::from_millis(200)));
    }

    #[test]
    fn schedule_and_transition_laws_hold_through_10000_deterministic_cases() {
        let epoch = Instant::now();
        let mut random = 0x6a09_e667_f3bc_c909_u64;

        for case in 0..10_000_u64 {
            random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
            let deadline_a = epoch + Duration::from_millis(random % 1_000);
            random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
            let deadline_b = epoch + Duration::from_millis(random % 1_000);
            let a = match case % 3 {
                0 => Schedule::Idle,
                1 => Schedule::At(deadline_a),
                _ => Schedule::NextFrame,
            };
            let b = match (case / 3) % 3 {
                0 => Schedule::Idle,
                1 => Schedule::At(deadline_b),
                _ => Schedule::NextFrame,
            };
            let c = match (case / 9) % 3 {
                0 => Schedule::Idle,
                1 => Schedule::At(epoch + Duration::from_millis(case % 997)),
                _ => Schedule::NextFrame,
            };

            assert_eq!(a.merge(Schedule::Idle), a, "schedule identity case {case}");
            assert_eq!(a.merge(b), b.merge(a), "schedule symmetry case {case}");
            assert_eq!(
                a.merge(b).merge(c),
                a.merge(b.merge(c)),
                "schedule associativity case {case}"
            );

            random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
            let from = (random % 20_001) as f32 / 100.0 - 100.0;
            random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
            let to = (random % 20_001) as f32 / 100.0 - 100.0;
            let duration = Duration::from_millis(case % 1_001);
            let transition = Transition::new(from, to, epoch, duration, Easing::EaseOutCubic);
            let end = epoch + duration;
            let start_value = transition.value_at(epoch);
            let end_value = transition.value_at(end);

            if duration.is_zero() {
                assert!(
                    (start_value - to).abs() <= 0.0001,
                    "zero-duration case {case}: {start_value} != {to}"
                );
            } else {
                assert_eq!(start_value, from, "transition start case {case}");
            }
            assert!(
                (end_value - to).abs() <= 0.0001,
                "transition endpoint case {case}: {end_value} != {to}"
            );
            assert!(!transition.is_animating_at(end), "settled case {case}");

            let sample = transition.value_at(epoch + duration / 2);
            let low = from.min(to);
            let high = from.max(to);
            assert!(
                sample.is_finite() && sample >= low && sample <= high,
                "bounded interpolation case {case}: {sample} outside {low}..={high}"
            );
        }
    }
}
