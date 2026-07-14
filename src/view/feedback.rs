use super::{Hint, Node, PanelPolicy, View, Wrap};
use crate::{composition::Tree, feedback, geometry, interaction};

const WINDOW_FEEDBACK_PANEL: &str = "feedback.window";
const HOVER_TIP_PANEL: &str = "feedback.hover";

enum Auxiliary {
    HoverTip,
    WindowFeedback,
}

impl View {
    pub(crate) fn hover_tip_eligible(
        &self,
        retained: &Tree,
        target: &interaction::Target,
        overflowed: bool,
    ) -> bool {
        self.root.input_hint_for_target(target).is_some()
            || self
                .root
                .hover_tip_text_retained(retained.root(), target)
                .is_some_and(|command_text| command_text.is_some() || overflowed)
    }

    pub(crate) fn project_feedback(
        &mut self,
        window_feedback: Option<(feedback::Severity, String)>,
    ) {
        if let Some((severity, text)) = window_feedback {
            self.push_floating_panel(
                auxiliary_panel(
                    WINDOW_FEEDBACK_PANEL,
                    Hint::from_feedback(severity, text),
                    Auxiliary::WindowFeedback,
                )
                .with_panel_anchor(geometry::placement::Anchor::Point(
                    geometry::Point::new(12, 12),
                )),
            );
        }
    }

    pub(crate) fn project_hover_tip(
        &mut self,
        retained: &Tree,
        target: &interaction::Target,
        pointer_anchor: geometry::Point,
        overflow: Option<String>,
    ) -> bool {
        if let Some(hint) = self.root.input_hint_for_target(target) {
            self.push_floating_panel(
                auxiliary_panel(HOVER_TIP_PANEL, hint, Auxiliary::HoverTip)
                    .with_pointer_panel_anchor(pointer_anchor),
            );
            return true;
        }

        let Some(command_text) = self.root.hover_tip_text_retained(retained.root(), target) else {
            return false;
        };
        let hint = match (command_text, overflow) {
            (Some(text), _) => Hint::information(text),
            (None, Some(text)) => Hint::plain(text),
            (None, None) => return false,
        };

        self.push_floating_panel(
            auxiliary_panel(HOVER_TIP_PANEL, hint, Auxiliary::HoverTip)
                .with_pointer_panel_anchor(pointer_anchor),
        );
        true
    }
}

fn auxiliary_panel(id: impl Into<interaction::Id>, hint: Hint, kind: Auxiliary) -> Node {
    let description = hint.description().to_owned();
    let policy = match kind {
        Auxiliary::HoverTip => PanelPolicy::HoverTip(hint),
        Auxiliary::WindowFeedback => PanelPolicy::WindowFeedback(hint),
    };
    Node::floating_panel(id).with_panel_policy(policy).child(
        Node::wrapped_world_text(description, Wrap::Word).with_auxiliary_text_participation(),
    )
}
