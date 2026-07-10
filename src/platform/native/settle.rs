use std::time::{Duration, Instant};

#[derive(Debug)]
pub(super) struct SysApplicator<T> {
    desired: Option<T>,
    applied: Option<T>,
    desired_changed_at: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ApplyDue {
    Initial,
    Immediate,
    Settled,
}

impl<T> Default for SysApplicator<T> {
    fn default() -> Self {
        Self {
            desired: None,
            applied: None,
            desired_changed_at: None,
        }
    }
}

impl<T: Copy + PartialEq> SysApplicator<T> {
    pub(super) fn set_desired(&mut self, desired: T, now: Instant) -> bool {
        if self.desired == Some(desired) {
            return false;
        }

        self.desired = Some(desired);
        self.desired_changed_at = (self.applied != Some(desired)).then_some(now);
        true
    }

    pub(super) fn due(
        &self,
        now: Instant,
        settle_delay: Duration,
        immediate_change: impl FnOnce(T, T) -> bool,
    ) -> Option<ApplyDue> {
        let desired = self.desired?;
        if self.applied == Some(desired) {
            return None;
        }

        let Some(applied) = self.applied else {
            return Some(ApplyDue::Initial);
        };
        if immediate_change(applied, desired) {
            return Some(ApplyDue::Immediate);
        }

        let changed_at = self.desired_changed_at.unwrap_or(now);
        (now.saturating_duration_since(changed_at) >= settle_delay).then_some(ApplyDue::Settled)
    }

    pub(super) fn desired(&self) -> Option<T> {
        self.desired
    }

    pub(super) fn applied(&self) -> Option<T> {
        self.applied
    }

    pub(super) fn mark_applied(&mut self, value: T) {
        self.applied = Some(value);
        self.desired = Some(value);
        self.desired_changed_at = None;
    }

    pub(super) fn pending(&self) -> bool {
        self.desired.is_some() && self.desired != self.applied
    }

    pub(super) fn changed_instant(&self) -> Option<Instant> {
        self.desired_changed_at
    }
}

#[cfg(test)]
mod tests {
    use super::{ApplyDue, SysApplicator};
    use std::time::{Duration, Instant};

    #[test]
    fn initial_value_is_due_immediately() {
        let now = Instant::now();
        let mut state = SysApplicator::default();
        state.set_desired(1_u8, now);

        assert_eq!(
            state.due(now, Duration::from_millis(150), |_, _| false),
            Some(ApplyDue::Initial)
        );
        state.mark_applied(1);
        assert_eq!(
            state.due(now, Duration::from_millis(150), |_, _| false),
            None
        );
    }

    #[test]
    fn immediate_policy_bypasses_settle_delay() {
        let now = Instant::now();
        let mut state = SysApplicator::default();
        state.set_desired(1_u8, now);
        state.mark_applied(1);
        state.set_desired(2, now);

        assert_eq!(
            state.due(now, Duration::from_millis(150), |_, _| true),
            Some(ApplyDue::Immediate)
        );
    }

    #[test]
    fn settled_policy_coalesces_latest_without_extending_repeated_desire() {
        let now = Instant::now();
        let delay = Duration::from_millis(150);
        let mut state = SysApplicator::default();
        state.set_desired(1_u8, now);
        state.mark_applied(1);
        state.set_desired(2, now);
        state.set_desired(3, now + Duration::from_millis(20));
        let changed_at = state.changed_instant();

        assert!(!state.set_desired(3, now + Duration::from_millis(100)));
        assert_eq!(state.changed_instant(), changed_at);
        assert_eq!(state.due(now + delay, delay, |_, _| false), None);
        assert_eq!(
            state.due(now + Duration::from_millis(20) + delay, delay, |_, _| false),
            Some(ApplyDue::Settled)
        );
        assert_eq!(state.desired(), Some(3));
    }

    #[test]
    fn reverting_to_applied_value_clears_pending() {
        let now = Instant::now();
        let mut state = SysApplicator::default();
        state.set_desired(1_u8, now);
        state.mark_applied(1);
        state.set_desired(2, now);
        state.set_desired(1, now + Duration::from_millis(1));

        assert!(!state.pending());
        assert_eq!(state.changed_instant(), None);
    }
}
