use super::super::Runtime;
use crate::{interaction, notification, state, window};

pub(in crate::runtime) struct Gesture<M: state::State> {
    window: window::Id,
    initial: M,
    changed_automatic: bool,
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::runtime) fn begin_pointer_gesture(
        &mut self,
        window: window::Id,
        target: &interaction::Target,
    ) {
        if !coalesces_pointer_gesture(target) || self.gesture.is_some() {
            return;
        }

        self.gesture = Some(Gesture {
            window,
            initial: self.store.model().clone(),
            changed_automatic: false,
        });
    }

    pub(in crate::runtime) fn finish_pointer_gesture(&mut self) {
        let Some(gesture) = self.gesture.take() else {
            return;
        };

        if gesture.changed_automatic {
            self.timeline.record(gesture.initial);
        }
    }

    pub(in crate::runtime::transaction) fn active_automatic_gesture(&self) -> bool {
        self.gesture.is_some()
    }

    pub(in crate::runtime::transaction) fn mark_automatic_gesture_changed(&mut self) {
        if let Some(gesture) = &mut self.gesture {
            gesture.changed_automatic = true;
        }
    }

    #[cfg(test)]
    pub(in crate::runtime) fn gesture_residue_count(&self, window: window::Id) -> usize {
        usize::from(
            self.gesture
                .as_ref()
                .is_some_and(|gesture| gesture.window == window),
        )
    }
}

impl<M: state::State> notification::Listener<window::Departed> for Option<Gesture<M>> {
    fn notify(&mut self, window: &window::Id) -> notification::Reaction {
        if self
            .as_ref()
            .is_some_and(|gesture| gesture.window == *window)
        {
            self.take();
        }
        notification::Reaction::ignored()
    }
}

fn coalesces_pointer_gesture(target: &interaction::Target) -> bool {
    target.captures() && target.kind() == interaction::Kind::Command
}
