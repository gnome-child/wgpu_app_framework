use std::collections::HashMap;

use super::super::interaction;
use super::Motion;

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Visuals {
    targets: HashMap<interaction::Target, Target>,
    slider_track_scale_y: HashMap<interaction::Target, Scalar>,
    scrollbars: HashMap<interaction::Target, Scrollbar>,
    carets: HashMap<interaction::Target, bool>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Target {
    hovered: bool,
    pressed: bool,
    active: bool,
    selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Scalar {
    Moving {
        value: f32,
        from: f32,
        target: f32,
        progress: f32,
    },
    Resting {
        value: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Scrollbar {
    opacity: f32,
    thickness: i32,
    thickness_motion: Motion,
    hovered: bool,
    pressed: bool,
}

impl Visuals {
    pub(crate) fn set_target(&mut self, target: interaction::Target, state: Target) {
        if state != Target::default() {
            self.targets.insert(target, state);
        }
    }

    pub(crate) fn target(&self, target: &interaction::Target) -> Target {
        self.targets.get(target).copied().unwrap_or_default()
    }

    pub(crate) fn set_caret_visible(&mut self, target: interaction::Target, visible: bool) {
        self.carets.insert(target, visible);
    }

    pub(crate) fn caret_visible(&self, target: &interaction::Target) -> bool {
        self.carets.get(target).copied().unwrap_or(true)
    }

    pub(crate) fn set_slider_track_scale_y(
        &mut self,
        target: interaction::Target,
        scale_y: Scalar,
    ) {
        self.slider_track_scale_y
            .insert(target, scale_y.sanitized_scale());
    }

    pub(crate) fn slider_track_scale_y(&self, target: &interaction::Target) -> Scalar {
        self.slider_track_scale_y
            .get(target)
            .copied()
            .unwrap_or_else(|| Scalar::resting(1.0))
    }

    pub(crate) fn set_scrollbar(
        &mut self,
        target: interaction::Target,
        opacity: f32,
        thickness: i32,
        thickness_motion: Motion,
        hovered: bool,
        pressed: bool,
    ) {
        self.scrollbars.insert(
            target,
            Scrollbar {
                opacity: sanitize_opacity(opacity),
                thickness: thickness.max(1),
                thickness_motion,
                hovered,
                pressed,
            },
        );
    }

    pub(crate) fn scrollbar(&self, target: &interaction::Target) -> Scrollbar {
        self.scrollbars.get(target).copied().unwrap_or_default()
    }
}

impl Target {
    pub(crate) fn new(hovered: bool, pressed: bool, active: bool, selected: bool) -> Self {
        Self {
            hovered,
            pressed,
            active,
            selected,
        }
    }

    pub(crate) fn hovered(self) -> bool {
        self.hovered
    }

    pub(crate) fn pressed(self) -> bool {
        self.pressed
    }

    pub(crate) fn active(self) -> bool {
        self.active
    }

    pub(crate) fn selected(self) -> bool {
        self.selected
    }
}

impl Scalar {
    pub(crate) fn moving(value: f32, from: f32, target: f32, progress: f32) -> Self {
        Self::Moving {
            value,
            from,
            target,
            progress,
        }
    }

    pub(crate) fn resting(value: f32) -> Self {
        Self::Resting { value }
    }

    pub(crate) fn value(self) -> f32 {
        match self {
            Self::Moving { value, .. } | Self::Resting { value } => value,
        }
    }

    pub(crate) fn from(self) -> f32 {
        match self {
            Self::Moving { from, .. } => from,
            Self::Resting { value } => value,
        }
    }

    pub(crate) fn target(self) -> f32 {
        match self {
            Self::Moving { target, .. } => target,
            Self::Resting { value } => value,
        }
    }

    pub(crate) fn progress(self) -> f32 {
        match self {
            Self::Moving { progress, .. } => progress,
            Self::Resting { .. } => 1.0,
        }
    }

    pub(crate) fn motion(self) -> Motion {
        match self {
            Self::Moving { .. } => Motion::Moving,
            Self::Resting { .. } => Motion::Resting,
        }
    }

    fn sanitized_scale(self) -> Self {
        match self {
            Self::Moving {
                value,
                from,
                target,
                progress,
            } => Self::Moving {
                value: sanitize_scale(value),
                from: sanitize_scale(from),
                target: sanitize_scale(target),
                progress: sanitize_progress(progress),
            },
            Self::Resting { value } => Self::Resting {
                value: sanitize_scale(value),
            },
        }
    }
}

impl Scrollbar {
    pub(crate) fn opacity(self) -> f32 {
        self.opacity
    }

    pub(crate) fn thickness(self) -> i32 {
        self.thickness
    }

    pub(crate) fn thickness_motion(self) -> Motion {
        self.thickness_motion
    }

    pub(crate) fn hovered(self) -> bool {
        self.hovered
    }

    pub(crate) fn pressed(self) -> bool {
        self.pressed
    }
}

impl Default for Scrollbar {
    fn default() -> Self {
        Self {
            opacity: 0.0,
            thickness: 1,
            thickness_motion: Motion::Resting,
            hovered: false,
            pressed: false,
        }
    }
}

fn sanitize_scale(scale: f32) -> f32 {
    if scale.is_finite() {
        scale.max(0.0)
    } else {
        1.0
    }
}

fn sanitize_progress(progress: f32) -> f32 {
    if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

fn sanitize_opacity(opacity: f32) -> f32 {
    if opacity.is_finite() {
        opacity.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
