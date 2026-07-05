use super::super::{error::Error, geometry, input, interaction, layout, state, view, window};
use super::Runtime;
impl<M: state::State, E: Send + 'static> Runtime<M, E, view::View> {
    pub fn pointer_move_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
    ) -> std::result::Result<input::Outcome, Error> {
        if self
            .session
            .interaction(window)
            .and_then(|interaction| interaction.pointer().pressed())
            .is_some()
        {
            return self.pointer_drag_at(window, size, point);
        }

        let target = self
            .hit_test(window, size, point)
            .and_then(|hit| hit.target().cloned());

        self.handle_view(window, view::Action::pointer_move(target))
    }

    pub fn pointer_down_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(hit) = self.hit_test(window, size, point) else {
            return Ok(input::Outcome::ignored());
        };
        let Some(target) = hit.target().cloned() else {
            return Ok(input::Outcome::ignored());
        };

        let action = if matches!(
            hit.frame().role(),
            view::node::Role::TextArea | view::node::Role::TextBox
        ) {
            hit.action_at_with_engine(point, &mut self.layout)
                .map(|action| {
                    view::Action::sequence([view::Action::pointer_down(target.clone()), action])
                })
                .unwrap_or_else(|| view::Action::pointer_down(target))
        } else if hit.frame().role() == view::node::Role::Slider {
            hit.action_at_with_engine(point, &mut self.layout)
                .map(|action| {
                    view::Action::sequence([view::Action::pointer_down(target.clone()), action])
                })
                .unwrap_or_else(|| view::Action::pointer_down(target))
        } else {
            view::Action::pointer_down(target)
        };

        self.handle_view(window, action)
    }

    pub fn pointer_up_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
    ) -> std::result::Result<input::Outcome, Error> {
        let hit = self.hit_test(window, size, point);
        let target = hit.as_ref().and_then(|hit| hit.target().cloned());
        let action = hit.as_ref().and_then(|hit| hit.action_at(point));

        self.handle_view(window, view::Action::pointer_up(target, action))
    }

    pub fn pointer_drag_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(composition) = self.composition.get(window) else {
            return Ok(input::Outcome::ignored());
        };

        let layout = layout::Layout::compose(composition.view(), size, &mut self.layout);
        let hit = layout.hit_test(point);
        let hovered = hit.as_ref().and_then(|hit| hit.target().cloned());
        let active = self.session.interaction(window).and_then(|interaction| {
            interaction
                .pointer()
                .capture()
                .map(|capture| capture.target().clone())
                .or_else(|| interaction.pointer().pressed().cloned())
        });

        let Some(target) = active else {
            return self.handle_view(window, view::Action::pointer_move(hovered));
        };

        let action = layout
            .frames()
            .iter()
            .find(|frame| frame.target() == Some(&target))
            .and_then(|frame| frame.drag_action_at_with_engine(point, &mut self.layout));

        self.handle_view(window, view::Action::pointer_drag(hovered, target, action))
    }

    pub fn scroll_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
        delta: interaction::ScrollDelta,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(target) = self
            .hit_test(window, size, point)
            .and_then(|hit| hit.target().cloned())
        else {
            return Ok(input::Outcome::ignored());
        };

        self.handle_view(window, view::Action::scroll(target, delta))
    }
}
