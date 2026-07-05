use super::super::Runtime;
use crate::scratch::{interaction, state};

pub(in crate::scratch::runtime) struct Gesture<M: state::State> {
    initial: M,
    changed_automatic: bool,
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::scratch::runtime) fn begin_pointer_gesture(
        &mut self,
        target: &interaction::Target,
    ) {
        if !coalesces_pointer_gesture(target) || self.gesture.is_some() {
            return;
        }

        self.gesture = Some(Gesture {
            initial: self.store.model().clone(),
            changed_automatic: false,
        });
    }

    pub(in crate::scratch::runtime) fn finish_pointer_gesture(&mut self) {
        let Some(gesture) = self.gesture.take() else {
            return;
        };

        if gesture.changed_automatic {
            self.timeline.record(gesture.initial);
        }
    }

    pub(in crate::scratch::runtime::transaction) fn active_automatic_gesture(&self) -> bool {
        self.gesture.is_some()
    }

    pub(in crate::scratch::runtime::transaction) fn mark_automatic_gesture_changed(&mut self) {
        if let Some(gesture) = &mut self.gesture {
            gesture.changed_automatic = true;
        }
    }
}

fn coalesces_pointer_gesture(target: &interaction::Target) -> bool {
    target.captures() && target.kind() == interaction::target::Kind::Command
}
