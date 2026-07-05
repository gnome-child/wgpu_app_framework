use super::super::{geometry::Point, interaction, view};
use super::{engine, frame::Frame};

#[derive(Clone)]
pub struct Hit {
    frame: Frame,
}

impl Hit {
    pub(super) fn new(frame: Frame) -> Self {
        Self { frame }
    }

    pub fn frame(&self) -> &Frame {
        &self.frame
    }

    pub fn target(&self) -> Option<&interaction::Target> {
        self.frame.target()
    }

    pub fn action(&self) -> Option<&view::Action> {
        self.frame.action()
    }

    pub fn action_at(&self, point: Point) -> Option<view::Action> {
        self.frame.action_at(point)
    }

    pub fn action_at_with_engine(
        &self,
        point: Point,
        engine: &mut engine::Engine,
    ) -> Option<view::Action> {
        self.frame.action_at_with_engine(point, engine)
    }

    pub fn drag_action_at_with_engine(
        &self,
        point: Point,
        engine: &mut engine::Engine,
    ) -> Option<view::Action> {
        self.frame.drag_action_at_with_engine(point, engine)
    }
}
