use std::collections::{HashMap, HashSet};

use super::super::interaction;
use super::Motion;

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Visuals {
    targets: HashMap<interaction::Target, Target>,
    slider_track_scale_y: HashMap<interaction::Target, Scalar>,
    scrollbars: HashMap<interaction::Target, Scrollbar>,
    carets: HashMap<interaction::Target, bool>,
    dirty_scrollbars: HashSet<interaction::Target>,
    dirty_carets: HashSet<interaction::Target>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct NodeState {
    target: Target,
    slider_track_scale_y: Scalar,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct Target {
    hovered: bool,
    pressed: bool,
    active: bool,
    selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Scalar {
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
pub(super) struct Scrollbar {
    opacity: f32,
    thickness: i32,
    thickness_motion: Motion,
    hovered: bool,
    pressed: bool,
}

impl Visuals {
    pub(crate) fn from_property_baseline(previous: &Self) -> Self {
        Self {
            scrollbars: previous.scrollbars.clone(),
            carets: previous.carets.clone(),
            ..Self::default()
        }
    }

    pub(super) fn node_state(&self, target: Option<&interaction::Target>) -> NodeState {
        NodeState {
            target: target.map_or_else(Target::default, |target| self.target(target)),
            slider_track_scale_y: self.slider_track_scale_y(target),
        }
    }

    pub(crate) fn set_target(
        &mut self,
        target: interaction::Target,
        hovered: bool,
        pressed: bool,
        active: bool,
        selected: bool,
    ) {
        let state = Target {
            hovered,
            pressed,
            active,
            selected,
        };
        if state != Target::default() {
            self.targets.insert(target, state);
        }
    }

    pub(super) fn target(&self, target: &interaction::Target) -> Target {
        self.targets.get(target).copied().unwrap_or_default()
    }

    pub(crate) fn target_is_hovered_or_pressed(&self, target: &interaction::Target) -> bool {
        let state = self.target(target);
        state.hovered() || state.pressed()
    }

    pub(crate) fn set_caret_visible(&mut self, target: interaction::Target, visible: bool) {
        if self.carets.insert(target.clone(), visible) != Some(visible) {
            self.dirty_carets.insert(target);
        }
    }

    pub(super) fn caret_visible(&self, target: &interaction::Target) -> bool {
        self.carets.get(target).copied().unwrap_or(true)
    }

    pub(super) fn caret_changed(&self, target: &interaction::Target) -> bool {
        self.dirty_carets.contains(target)
    }

    pub(crate) fn set_moving_slider_track_scale_y(
        &mut self,
        target: interaction::Target,
        value: f32,
        from: f32,
        target_value: f32,
        progress: f32,
    ) {
        self.slider_track_scale_y.insert(
            target,
            Scalar::moving(value, from, target_value, progress).sanitized_scale(),
        );
    }

    pub(crate) fn set_resting_slider_track_scale_y(
        &mut self,
        target: interaction::Target,
        value: f32,
    ) {
        self.slider_track_scale_y
            .insert(target, Scalar::resting(value).sanitized_scale());
    }

    pub(super) fn slider_track_scale_y(&self, target: Option<&interaction::Target>) -> Scalar {
        target
            .and_then(|target| self.slider_track_scale_y.get(target))
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
        let scrollbar = Scrollbar {
            opacity: sanitize_opacity(opacity),
            thickness: thickness.max(1),
            thickness_motion,
            hovered,
            pressed,
        };
        if self.scrollbars.insert(target.clone(), scrollbar) != Some(scrollbar) {
            self.dirty_scrollbars.insert(target);
        }
    }

    pub(super) fn scrollbar(&self, target: &interaction::Target) -> Scrollbar {
        self.scrollbars.get(target).copied().unwrap_or_default()
    }

    pub(super) fn scrollbar_changed(&self, target: &interaction::Target) -> bool {
        self.dirty_scrollbars.contains(target)
    }
}

impl Target {
    pub(super) fn hovered(self) -> bool {
        self.hovered
    }

    pub(super) fn pressed(self) -> bool {
        self.pressed
    }

    pub(super) fn active(self) -> bool {
        self.active
    }

    pub(super) fn selected(self) -> bool {
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
    pub(super) fn opacity(self) -> f32 {
        self.opacity
    }

    pub(super) fn thickness(self) -> i32 {
        self.thickness
    }

    pub(super) fn hovered(self) -> bool {
        self.hovered
    }

    pub(super) fn pressed(self) -> bool {
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
