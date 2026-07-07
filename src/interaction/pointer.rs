use super::Target;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Pointer {
    pub(super) hovered: Option<Target>,
    pub(super) pressed: Option<Target>,
    pub(super) capture: Option<Capture>,
    pub(super) press_intent: Option<PressIntent>,
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

impl Pointer {
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
