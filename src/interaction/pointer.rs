use super::Target;
use crate::geometry::Point;
use std::time::Instant;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Pointer {
    pub(super) hovered: Option<Target>,
    pub(super) pressed: Option<Target>,
    pub(super) capture: Option<Capture>,
    pub(super) press_intent: Option<PressIntent>,
    last_click: Option<Click>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Capture {
    target: Target,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PressIntent {
    Activate,
    Manipulate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Click {
    target: Target,
    point: Point,
    at: Instant,
    count: ClickCount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClickCount {
    Single,
    Double,
    Triple,
}

impl Pointer {
    pub(crate) fn classify_click(
        &mut self,
        target: &Target,
        point: Point,
        at: Instant,
    ) -> ClickCount {
        let settings = crate::pointer::MultiClickSettings::system();
        let count = self
            .last_click
            .as_ref()
            .filter(|last| {
                last.target == *target
                    && settings.accepts(
                        at.saturating_duration_since(last.at),
                        point.x().abs_diff(last.point.x()) as i32,
                        point.y().abs_diff(last.point.y()) as i32,
                    )
            })
            .map_or(ClickCount::Single, |last| match last.count {
                ClickCount::Single => ClickCount::Double,
                ClickCount::Double => ClickCount::Triple,
                ClickCount::Triple => ClickCount::Single,
            });
        self.last_click = Some(Click {
            target: target.clone(),
            point,
            at,
            count,
        });
        count
    }

    pub(crate) fn cancel_click_sequence(&mut self) -> bool {
        self.last_click.take().is_some()
    }

    pub(crate) fn hovered(&self) -> Option<&Target> {
        self.hovered.as_ref()
    }

    pub(crate) fn pressed(&self) -> Option<&Target> {
        self.pressed.as_ref()
    }

    pub(crate) fn capture(&self) -> Option<&Capture> {
        self.capture.as_ref()
    }

    pub(crate) fn activation_target(&self) -> Option<&Target> {
        (self.press_intent == Some(PressIntent::Activate))
            .then_some(self.pressed.as_ref())
            .flatten()
    }
}

impl Capture {
    pub(super) fn new(target: Target) -> Self {
        Self { target }
    }

    pub(crate) fn target(&self) -> &Target {
        &self.target
    }
}
