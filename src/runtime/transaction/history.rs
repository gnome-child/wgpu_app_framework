use std::time::{Duration, Instant};

use super::super::Runtime;
use crate::{command, state};

const HISTORY_GROUP_COALESCE_WINDOW: Duration = Duration::from_millis(1000);

pub(in crate::runtime) struct ActiveGroup {
    group: command::HistoryGroup,
    recorded_at: Instant,
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::runtime) fn snapshot_before_transaction(
        &mut self,
        history: command::History,
    ) -> Option<state::PendingSnapshot<M>> {
        match history {
            command::History::Automatic => Some(self.store.prepare_snapshot()),
            command::History::Committed | command::History::Ignored => None,
        }
    }

    pub(in crate::runtime) fn finish_transaction(
        &mut self,
        before: Option<state::PendingSnapshot<M>>,
        history: command::History,
        history_group: Option<command::HistoryGroup>,
        revision_before: state::Revision,
        reason: state::Reason,
        changed: bool,
    ) {
        if !changed {
            if let Some(before) = before {
                self.store.restore_prepared_snapshot(before);
            }
            return;
        }

        match history {
            command::History::Automatic => {
                let before = before.expect("automatic history snapshots before dispatch");
                if self.active_automatic_gesture() {
                    self.mark_automatic_gesture_changed();
                    self.clear_history_group();
                    drop(before);
                } else if self.coalesces_history_group(history_group) {
                    drop(before);
                } else {
                    self.timeline.record(before.into_model());
                }
                self.store.commit_retaining_current(reason);
            }
            command::History::Committed | command::History::Ignored => {
                self.clear_history_group();
                if self.revision() == revision_before {
                    self.store.commit(reason);
                } else {
                    self.store.discard_retained_snapshot();
                }
            }
        }
    }

    pub(in crate::runtime::transaction) fn coalesces_history_group(
        &mut self,
        group: Option<command::HistoryGroup>,
    ) -> bool {
        let Some(group) = group else {
            self.clear_history_group();
            return false;
        };
        let now = Instant::now();
        let coalesces = self.history_group.as_ref().is_some_and(|active| {
            active.group == group
                && now.saturating_duration_since(active.recorded_at)
                    <= HISTORY_GROUP_COALESCE_WINDOW
        });
        self.history_group = Some(ActiveGroup {
            group,
            recorded_at: now,
        });
        coalesces
    }

    pub(in crate::runtime::transaction) fn clear_history_group(&mut self) {
        self.history_group = None;
    }
}
