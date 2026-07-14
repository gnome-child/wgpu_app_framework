use super::super::{geometry::Point, interaction, view};
use super::{chrome, engine, frame::Frame};

#[derive(Clone)]
pub(crate) struct Hit {
    frame: Frame,
    kind: Kind,
    table_cell: Option<crate::table::Cell>,
}

#[derive(Clone)]
enum Kind {
    Frame,
    Chrome(chrome::Chrome),
    Target(interaction::Target),
}

impl Hit {
    pub(super) fn new(frame: Frame) -> Self {
        Self {
            frame,
            kind: Kind::Frame,
            table_cell: None,
        }
    }

    pub(super) fn chrome(frame: Frame, chrome: chrome::Chrome) -> Self {
        Self {
            frame,
            kind: Kind::Chrome(chrome),
            table_cell: None,
        }
    }

    pub(super) fn table_divider(frame: Frame, target: interaction::Target) -> Self {
        Self {
            frame,
            kind: Kind::Target(target),
            table_cell: None,
        }
    }

    pub(super) fn indicator(frame: Frame, target: interaction::Target) -> Self {
        Self {
            frame,
            kind: Kind::Target(target),
            table_cell: None,
        }
    }

    pub(super) fn with_table_cell(mut self, cell: Option<crate::table::Cell>) -> Self {
        self.table_cell = cell;
        self
    }

    pub(crate) fn table_cell(&self) -> Option<crate::table::Cell> {
        self.table_cell
    }

    pub(crate) fn frame(&self) -> &Frame {
        &self.frame
    }

    pub(crate) fn is_chrome(&self) -> bool {
        matches!(&self.kind, Kind::Chrome(_))
    }

    pub(crate) fn target(&self) -> Option<&interaction::Target> {
        match &self.kind {
            Kind::Frame => self.frame.target(),
            Kind::Chrome(chrome) => Some(chrome.target()),
            Kind::Target(target) => Some(target),
        }
    }

    #[cfg(test)]
    pub(crate) fn action(&self) -> Option<&view::Action> {
        match &self.kind {
            Kind::Frame => self.frame.action(),
            Kind::Chrome(_) | Kind::Target(_) => None,
        }
    }

    pub(crate) fn action_at(&self, point: Point) -> Option<view::Action> {
        match &self.kind {
            Kind::Frame => self.frame.action_at(point),
            Kind::Chrome(_) | Kind::Target(_) => None,
        }
    }

    pub(crate) fn action_at_with_engine(
        &self,
        point: Point,
        engine: &mut engine::Engine,
    ) -> Option<view::Action> {
        match &self.kind {
            Kind::Frame => self.frame.action_at_with_engine(point, engine),
            Kind::Chrome(_) | Kind::Target(_) => None,
        }
    }

    pub(crate) fn text_action_at_with_engine(
        &self,
        point: Point,
        kind: crate::text::selection::PointerKind,
        engine: &mut engine::Engine,
    ) -> Option<view::Action> {
        match &self.kind {
            Kind::Frame => self.frame.text_action_at_with_engine(point, kind, engine),
            Kind::Chrome(_) | Kind::Target(_) => None,
        }
    }
}
