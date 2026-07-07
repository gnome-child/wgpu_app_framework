use super::Target;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Pointer {
    pub(super) hovered: Option<Target>,
    pub(super) pressed: Option<Target>,
    pub(super) capture: Option<Capture>,
    pub(super) press_intent: Option<PressIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    target: Target,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressIntent {
    Activate,
    Manipulate,
}

impl Pointer {
    pub fn hovered(&self) -> Option<&Target> {
        self.hovered.as_ref()
    }

    pub fn pressed(&self) -> Option<&Target> {
        self.pressed.as_ref()
    }

    pub fn capture(&self) -> Option<&Capture> {
        self.capture.as_ref()
    }

    pub fn press_intent(&self) -> Option<PressIntent> {
        self.press_intent
    }

    pub fn activation_target(&self) -> Option<&Target> {
        (self.press_intent == Some(PressIntent::Activate))
            .then_some(self.pressed.as_ref())
            .flatten()
    }
}

impl Capture {
    pub(super) fn new(target: Target) -> Self {
        Self { target }
    }

    pub fn target(&self) -> &Target {
        &self.target
    }
}
