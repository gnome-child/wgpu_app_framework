use super::Target;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Pointer {
    pub(super) hovered: Option<Target>,
    pub(super) pressed: Option<Target>,
    pub(super) capture: Option<Capture>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    target: Target,
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
}

impl Capture {
    pub(super) fn new(target: Target) -> Self {
        Self { target }
    }

    pub fn target(&self) -> &Target {
        &self.target
    }
}
