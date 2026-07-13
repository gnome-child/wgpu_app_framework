use super::{AuxiliaryChrome, Node, PanelPolicy, View, Wrap};
use crate::{feedback, geometry, interaction};

const TABLE_FEEDBACK_PANEL: &str = "feedback.table";
const WINDOW_FEEDBACK_PANEL: &str = "feedback.window";
const HOVER_TIP_PANEL: &str = "feedback.hover";

impl View {
    pub(crate) fn hover_tip_eligible(
        &self,
        retained: &crate::composition::Tree,
        target: &interaction::Target,
        overflowed: bool,
    ) -> bool {
        self.root
            .hover_tip_text_retained(retained.root(), target, false)
            .is_some_and(|(blocked, command_text)| {
                !blocked && (command_text.is_some() || overflowed)
            })
    }

    pub(crate) fn project_feedback(
        &mut self,
        window_feedback: Option<(feedback::Severity, String)>,
    ) {
        if let Some((cell, text)) = self.root.first_table_rejection() {
            self.push_floating_panel(
                auxiliary_panel(
                    TABLE_FEEDBACK_PANEL,
                    AuxiliaryChrome::Error,
                    text,
                    PanelPolicy::AnchoredFeedback,
                )
                .with_table_panel_anchor(cell),
            );
        }

        if let Some((severity, text)) = window_feedback {
            self.push_floating_panel(
                auxiliary_panel(
                    WINDOW_FEEDBACK_PANEL,
                    severity.into(),
                    text,
                    PanelPolicy::WindowFeedback,
                )
                .with_panel_anchor(geometry::PlacementAnchor::Point(
                    geometry::Point::new(12, 12),
                )),
            );
        }
    }

    pub(crate) fn project_hover_tip(
        &mut self,
        retained: &crate::composition::Tree,
        target: &interaction::Target,
        pointer_anchor: geometry::Point,
        overflow: Option<String>,
    ) -> bool {
        let Some((blocked, command_text)) =
            self.root
                .hover_tip_text_retained(retained.root(), target, false)
        else {
            return false;
        };
        if blocked {
            return false;
        }
        let (chrome, text) = match (command_text, overflow) {
            (Some(text), _) => (AuxiliaryChrome::Info, text),
            (None, Some(text)) => (AuxiliaryChrome::Plain, text),
            (None, None) => return false,
        };

        self.push_floating_panel(
            auxiliary_panel(HOVER_TIP_PANEL, chrome, text, PanelPolicy::HoverTip)
                .with_pointer_panel_anchor(pointer_anchor),
        );
        true
    }
}

fn auxiliary_panel(
    id: impl Into<interaction::Id>,
    chrome: AuxiliaryChrome,
    text: String,
    policy: PanelPolicy,
) -> Node {
    Node::floating_panel(id)
        .with_panel_policy(policy)
        .with_auxiliary_chrome(chrome)
        .child(Node::wrapped_world_text(text, Wrap::Word).with_auxiliary_text_participation())
}
