use std::any::TypeId;

use super::super::super::Runtime;
use super::field::CommitAttempt;
use crate::{error::Error, input, response, session, state, window};

pub(in crate::runtime) struct TaskTransition {
    accepted: bool,
    outcome: input::Outcome,
}

impl TaskTransition {
    fn accepted(outcome: input::Outcome) -> Self {
        Self {
            accepted: true,
            outcome,
        }
    }

    fn rejected(outcome: input::Outcome) -> Self {
        Self {
            accepted: false,
            outcome,
        }
    }

    pub(in crate::runtime) fn is_accepted(&self) -> bool {
        self.accepted
    }

    pub(in crate::runtime) fn outcome(&self) -> &input::Outcome {
        &self.outcome
    }

    pub(in crate::runtime) fn into_outcome(self) -> input::Outcome {
        self.outcome
    }

    pub(in crate::runtime) fn then(self, next: input::Outcome) -> input::Outcome {
        merge_outcomes(self.outcome, next)
    }
}

pub(in crate::runtime) struct FocusedTextCommand {
    owned_by_text: bool,
    transition: Option<TaskTransition>,
}

impl FocusedTextCommand {
    pub(in crate::runtime) fn is_owned_by_text(&self) -> bool {
        self.owned_by_text
    }

    pub(in crate::runtime) fn committed(&self) -> Option<&input::Outcome> {
        self.transition.as_ref().map(TaskTransition::outcome)
    }

    pub(in crate::runtime) fn into_committed(self) -> Option<input::Outcome> {
        self.transition.map(TaskTransition::into_outcome)
    }

    pub(in crate::runtime) fn is_accepted(&self) -> bool {
        self.transition
            .as_ref()
            .is_none_or(TaskTransition::is_accepted)
    }
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::runtime) fn prepare_focused_text_for_command(
        &mut self,
        window: window::Id,
        command_type: TypeId,
    ) -> std::result::Result<FocusedTextCommand, Error> {
        let owned_by_text = self.focused_text_owns_command(window, command_type);
        let preserves_focus = command_type == TypeId::of::<session::OpenCommandPalette>();
        let transition = if owned_by_text || preserves_focus {
            None
        } else {
            self.commit_and_deactivate_focused_text_box(window)?
        };

        Ok(FocusedTextCommand {
            owned_by_text,
            transition,
        })
    }

    pub(in crate::runtime) fn focus_committing_text_box(
        &mut self,
        window: window::Id,
        focus: session::Focus,
    ) -> std::result::Result<input::Outcome, Error> {
        self.attempt_focus_transition(window, focus)
            .map(TaskTransition::into_outcome)
    }

    pub(in crate::runtime) fn attempt_focus_transition(
        &mut self,
        window: window::Id,
        focus: session::Focus,
    ) -> std::result::Result<TaskTransition, Error> {
        let mut outcome = input::Outcome::ignored();

        if let Some(current) = self.session.focused(window)
            && !current.same_target(&focus)
            && let Some(transition) = self.commit_and_deactivate_focused_text_box(window)?
        {
            if !transition.is_accepted() {
                return Ok(transition);
            }
            outcome = merge_outcomes(outcome, transition.into_outcome());
        }

        let focus_changed = self.focus(window, focus);
        let focused = focus_changed
            .then(|| input::Outcome::handled(false, response::Effect::Layout))
            .unwrap_or_else(input::Outcome::ignored);
        Ok(TaskTransition::accepted(merge_outcomes(outcome, focused)))
    }

    pub(in crate::runtime) fn clear_focus_committing_text_box(
        &mut self,
        window: window::Id,
    ) -> std::result::Result<input::Outcome, Error> {
        self.attempt_clear_focus_transition(window)
            .map(TaskTransition::into_outcome)
    }

    pub(in crate::runtime) fn attempt_clear_focus_transition(
        &mut self,
        window: window::Id,
    ) -> std::result::Result<TaskTransition, Error> {
        let mut outcome = input::Outcome::ignored();

        if let Some(transition) = self.commit_and_deactivate_focused_text_box(window)? {
            if !transition.is_accepted() {
                return Ok(transition);
            }
            outcome = merge_outcomes(outcome, transition.into_outcome());
        }

        let focus_changed = self.clear_focus(window);
        let cleared = focus_changed
            .then(|| input::Outcome::handled(false, response::Effect::Layout))
            .unwrap_or_else(input::Outcome::ignored);
        Ok(TaskTransition::accepted(merge_outcomes(outcome, cleared)))
    }

    pub(in crate::runtime) fn commit_and_deactivate_focused_text_box(
        &mut self,
        window: window::Id,
    ) -> std::result::Result<Option<TaskTransition>, Error> {
        let Some(current) = self.session.focused(window) else {
            return Ok(None);
        };
        if self.text_draft_base(window, current).is_none() {
            return Ok(None);
        }

        let mut outcome = input::Outcome::ignored();
        match self.commit_text_box_draft(window, current)? {
            CommitAttempt::NotAttempted => {}
            CommitAttempt::Accepted(committed) => {
                outcome = merge_outcomes(outcome, committed);
            }
            CommitAttempt::Rejected(rejected) => {
                return Ok(Some(TaskTransition::rejected(rejected)));
            }
        }

        if self.session.deactivate_text_draft(window, current) {
            outcome = merge_outcomes(
                outcome,
                input::Outcome::handled(false, response::Effect::Layout),
            );
        }
        self.apply_window_update(window, outcome.changed_state(), outcome.effect());
        Ok(Some(TaskTransition::accepted(outcome)))
    }
}

fn merge_outcomes(left: input::Outcome, right: input::Outcome) -> input::Outcome {
    if !left.is_handled() && !right.is_handled() {
        return input::Outcome::ignored();
    }
    input::Outcome::handled(
        left.changed_state() || right.changed_state(),
        left.effect().clone().then(right.effect().clone()),
    )
}
