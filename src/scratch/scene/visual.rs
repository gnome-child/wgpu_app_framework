use std::collections::HashMap;

use super::super::interaction;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Visuals {
    targets: HashMap<interaction::Target, Target>,
    slider_track_scale_y: HashMap<interaction::Target, f32>,
    scrollbars: HashMap<interaction::Target, Scrollbar>,
    carets: HashMap<interaction::Target, bool>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in crate::scratch) struct Target {
    hovered: bool,
    pressed: bool,
    active: bool,
    selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::scratch) struct Scrollbar {
    opacity: f32,
    thickness: i32,
    hovered: bool,
    pressed: bool,
}

impl Visuals {
    pub(in crate::scratch) fn set_target(&mut self, target: interaction::Target, state: Target) {
        if state != Target::default() {
            self.targets.insert(target, state);
        }
    }

    pub(in crate::scratch) fn target(&self, target: &interaction::Target) -> Target {
        self.targets.get(target).copied().unwrap_or_default()
    }

    pub(in crate::scratch) fn set_caret_visible(
        &mut self,
        target: interaction::Target,
        visible: bool,
    ) {
        self.carets.insert(target, visible);
    }

    pub(in crate::scratch) fn caret_visible(&self, target: &interaction::Target) -> bool {
        self.carets.get(target).copied().unwrap_or(true)
    }

    pub(in crate::scratch) fn set_slider_track_scale_y(
        &mut self,
        target: interaction::Target,
        scale_y: f32,
    ) {
        self.slider_track_scale_y
            .insert(target, sanitize_scale(scale_y));
    }

    pub(in crate::scratch) fn slider_track_scale_y(&self, target: &interaction::Target) -> f32 {
        self.slider_track_scale_y
            .get(target)
            .copied()
            .unwrap_or(1.0)
    }

    pub(in crate::scratch) fn set_scrollbar(
        &mut self,
        target: interaction::Target,
        opacity: f32,
        thickness: i32,
        hovered: bool,
        pressed: bool,
    ) {
        self.scrollbars.insert(
            target,
            Scrollbar {
                opacity: sanitize_opacity(opacity),
                thickness: thickness.max(1),
                hovered,
                pressed,
            },
        );
    }

    pub(in crate::scratch) fn scrollbar(&self, target: &interaction::Target) -> Scrollbar {
        self.scrollbars.get(target).copied().unwrap_or_default()
    }

    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
            && self.slider_track_scale_y.is_empty()
            && self.scrollbars.is_empty()
            && self.carets.is_empty()
    }
}

impl Target {
    pub(in crate::scratch) fn new(
        hovered: bool,
        pressed: bool,
        active: bool,
        selected: bool,
    ) -> Self {
        Self {
            hovered,
            pressed,
            active,
            selected,
        }
    }

    pub(in crate::scratch) fn hovered(self) -> bool {
        self.hovered
    }

    pub(in crate::scratch) fn pressed(self) -> bool {
        self.pressed
    }

    pub(in crate::scratch) fn active(self) -> bool {
        self.active
    }

    pub(in crate::scratch) fn selected(self) -> bool {
        self.selected
    }
}

impl Scrollbar {
    pub(in crate::scratch) fn opacity(self) -> f32 {
        self.opacity
    }

    pub(in crate::scratch) fn thickness(self) -> i32 {
        self.thickness
    }

    pub(in crate::scratch) fn hovered(self) -> bool {
        self.hovered
    }

    pub(in crate::scratch) fn pressed(self) -> bool {
        self.pressed
    }
}

impl Default for Scrollbar {
    fn default() -> Self {
        Self {
            opacity: 0.0,
            thickness: 1,
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

fn sanitize_opacity(opacity: f32) -> f32 {
    if opacity.is_finite() {
        opacity.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
