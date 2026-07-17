use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use crate::animation::{self, Easing, Transition};
use crate::{notification, text};

use super::super::{interaction, layout, scene, theme, view, window};

const SLIDER_TRACK_IDLE_SCALE_Y: f32 = 1.0;
const SLIDER_TRACK_HOVER_SCALE_Y: f32 = 1.5;
const SLIDER_TRACK_TRANSITION: Duration = Duration::from_millis(120);
const SLIDER_TRACK_EASING: Easing = Easing::EaseOutCubic;
const SCROLLBAR_THICKNESS_TRANSITION: Duration = Duration::from_millis(120);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Key {
    window: window::Id,
    target: interaction::Target,
    property: Property,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Property {
    SliderTrackScaleY,
    ScrollbarOpacity,
    ScrollbarThickness,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct VisualTransition {
    transition: Transition<f32>,
    base_value: f32,
}

#[derive(Debug, Default)]
pub(super) struct Animations {
    transitions: HashMap<Key, VisualTransition>,
    scrollbar_activity: HashMap<ScrollKey, Instant>,
    scrollbar_offsets: HashMap<ScrollKey, interaction::ScrollOffset>,
    visuals: HashMap<window::Id, scene::Visuals>,
}

pub(super) struct Update {
    visuals: scene::Visuals,
    schedule: animation::Schedule,
}

struct VisualPass<'a> {
    window: window::Id,
    layout: &'a layout::Layout,
    interaction: Option<&'a interaction::Interaction>,
    theme: &'a theme::Theme,
    now: Instant,
    visuals: &'a mut scene::Visuals,
    schedule: &'a mut animation::Schedule,
    seen: &'a mut HashSet<Key>,
    remove: &'a mut Vec<Key>,
}

struct TransitionStep {
    key: Key,
    desired: f32,
    base: f32,
    duration: Duration,
}

impl Animations {
    pub(super) fn clear(&mut self) {
        self.transitions.clear();
        self.scrollbar_activity.clear();
        self.scrollbar_offsets.clear();
        self.visuals.clear();
    }

    pub(super) fn update_window(
        &mut self,
        window: window::Id,
        layout: &layout::Layout,
        interaction: Option<&interaction::Interaction>,
        theme: &theme::Theme,
        now: Instant,
    ) -> Update {
        let mut visuals = self
            .visuals
            .get(&window)
            .map(scene::Visuals::from_property_baseline)
            .unwrap_or_default();
        let mut schedule = animation::Schedule::Idle;
        let mut seen = HashSet::new();
        let mut remove = Vec::new();
        self.update_target_visuals(layout, interaction, now, &mut visuals);

        for frame in layout.frames() {
            if frame.role() != view::Role::Slider {
                continue;
            }

            let Some(target) = frame.target().cloned() else {
                continue;
            };

            let key = Key {
                window,
                target: target.clone(),
                property: Property::SliderTrackScaleY,
            };
            seen.insert(key.clone());

            let desired = if frame.is_enabled() && visuals.target_is_hovered_or_pressed(&target) {
                SLIDER_TRACK_HOVER_SCALE_Y
            } else {
                SLIDER_TRACK_IDLE_SCALE_Y
            };

            let transition = self.transitions.entry(key.clone()).or_insert_with(|| {
                VisualTransition::settled(
                    SLIDER_TRACK_IDLE_SCALE_Y,
                    SLIDER_TRACK_IDLE_SCALE_Y,
                    now,
                    SLIDER_TRACK_TRANSITION,
                )
            });
            transition.retarget(desired, now);

            let value = transition.value_at(now);
            let is_animating = transition.is_animating_at(now);
            if is_animating {
                visuals.set_moving_slider_track_scale_y(
                    target,
                    value,
                    transition.from(),
                    transition.target(),
                    transition.eased_progress_at(now),
                );
            } else {
                visuals.set_resting_slider_track_scale_y(target, value);
            }

            if is_animating {
                schedule = schedule.merge(animation::Schedule::NextFrame);
            } else if transition.is_at_base(now) {
                remove.push(key);
            }
        }

        {
            let mut pass = VisualPass {
                window,
                layout,
                interaction,
                theme,
                now,
                visuals: &mut visuals,
                schedule: &mut schedule,
                seen: &mut seen,
                remove: &mut remove,
            };
            self.update_scrollbar_visuals(&mut pass);
        }

        self.transitions
            .retain(|key, _| key.window != window || seen.contains(key));
        for key in remove {
            self.transitions.remove(&key);
        }

        self.visuals.insert(window, visuals.clone());
        Update { visuals, schedule }
    }

    fn update_scrollbar_visuals(&mut self, pass: &mut VisualPass<'_>) {
        let pointer = pass.interaction.map(interaction::Interaction::pointer);
        let hovered = pointer.and_then(interaction::Pointer::hovered);
        let pressed = pointer.and_then(interaction::Pointer::pressed);
        let mut seen_scrolls = HashSet::new();

        for chrome in pass.layout.chrome() {
            let target = chrome.target().clone();
            let scroll_target = chrome.scroll_target().clone();
            let scroll_key = ScrollKey {
                window: pass.window,
                target: scroll_target.clone(),
            };
            let offset_changed = if seen_scrolls.insert(scroll_key.clone()) {
                let offset = pass
                    .interaction
                    .map(interaction::Interaction::scroll)
                    .map(|scroll| scroll.desired_offset(&scroll_target))
                    .unwrap_or_else(|| chrome.resolved_scroll());
                self.scrollbar_offsets
                    .insert(scroll_key.clone(), offset)
                    .is_some_and(|previous| previous != offset)
            } else {
                false
            };
            let is_hovered = hovered == Some(&target);
            let is_pressed = pressed == Some(&target);
            if offset_changed || is_hovered || is_pressed {
                self.scrollbar_activity.insert(scroll_key, pass.now);
            }
        }

        for chrome in pass.layout.chrome() {
            let target = chrome.target().clone();
            let scroll_key = ScrollKey {
                window: pass.window,
                target: chrome.scroll_target().clone(),
            };
            let is_hovered = hovered == Some(&target);
            let is_pressed = pressed == Some(&target);
            let (desired_opacity, fade_deadline) = self.scrollbar_opacity_target(
                &scroll_key,
                is_hovered || is_pressed,
                chrome.presentation(),
                pass.theme,
                pass.now,
            );
            let base_thickness = match chrome.presentation() {
                view::ScrollChromePresentation::Consuming => {
                    pass.theme.scrollbar().metrics.thickness
                }
                view::ScrollChromePresentation::Overlay => {
                    pass.theme.scrollbar().appearance.overlay_thickness
                }
            }
            .max(1);
            let desired_thickness = if is_hovered || is_pressed {
                pass.theme
                    .scrollbar()
                    .appearance
                    .hover_thickness
                    .max(base_thickness)
            } else {
                base_thickness
            };
            let (opacity, _) = self.transition_value(
                pass,
                TransitionStep {
                    key: Key {
                        window: pass.window,
                        target: target.clone(),
                        property: Property::ScrollbarOpacity,
                    },
                    desired: desired_opacity,
                    base: idle_opacity_for(chrome.presentation()),
                    duration: Duration::from_millis(
                        pass.theme.scrollbar().appearance.fade_duration_ms,
                    ),
                },
            );
            let (thickness, thickness_motion) = self.transition_value(
                pass,
                TransitionStep {
                    key: Key {
                        window: pass.window,
                        target: target.clone(),
                        property: Property::ScrollbarThickness,
                    },
                    desired: desired_thickness as f32,
                    base: base_thickness as f32,
                    duration: SCROLLBAR_THICKNESS_TRANSITION,
                },
            );

            if let Some(deadline) = fade_deadline {
                *pass.schedule = pass.schedule.merge(animation::Schedule::At(deadline));
            }
            pass.visuals.set_scrollbar(
                target,
                opacity,
                thickness.round() as i32,
                thickness_motion,
                is_hovered,
                is_pressed,
            );
        }

        self.scrollbar_offsets
            .retain(|key, _| key.window != pass.window || seen_scrolls.contains(key));
        self.scrollbar_activity
            .retain(|key, _| key.window != pass.window || seen_scrolls.contains(key));
    }

    fn update_target_visuals(
        &self,
        layout: &layout::Layout,
        interaction: Option<&interaction::Interaction>,
        now: Instant,
        visuals: &mut scene::Visuals,
    ) {
        let pointer = interaction.map(interaction::Interaction::pointer);
        let hovered = pointer.and_then(interaction::Pointer::hovered);
        let pressed = pointer.and_then(interaction::Pointer::pressed);
        let activation = pointer.and_then(interaction::Pointer::activation_target);
        let open_menu = interaction.and_then(interaction::Interaction::open_menu);
        let selected_palette = interaction
            .and_then(interaction::Interaction::command_palette)
            .map(interaction::CommandPalette::selected);
        let mut palette_row = 0_usize;

        for frame in layout.frames() {
            let Some(target) = frame.target().cloned() else {
                continue;
            };

            let selected = if frame.is_palette_row() {
                let selected = selected_palette == Some(palette_row);
                palette_row = palette_row.saturating_add(1);
                selected
            } else {
                frame.is_selected() || frame.is_active_item()
            };
            let active = match frame.role() {
                view::Role::MenuBar => open_menu.is_some(),
                view::Role::Menu => target
                    .as_menu()
                    .is_some_and(|menu| open_menu == Some(&menu)),
                _ => activation == Some(&target),
            };

            visuals.set_target(
                target.clone(),
                hovered == Some(&target),
                pressed == Some(&target),
                active,
                selected,
            );

            if let Some(visible) = caret_visible_for(frame, now) {
                visuals.set_caret_visible(target, visible);
            }
        }
    }

    fn scrollbar_opacity_target(
        &self,
        key: &ScrollKey,
        pointer_active: bool,
        presentation: view::ScrollChromePresentation,
        theme: &theme::Theme,
        now: Instant,
    ) -> (f32, Option<Instant>) {
        if presentation == view::ScrollChromePresentation::Consuming {
            return (1.0, None);
        }
        let Some(last_activity) = self.scrollbar_activity.get(key).copied() else {
            return (0.0, None);
        };
        if pointer_active {
            return (1.0, None);
        }

        let fade_start =
            last_activity + Duration::from_millis(theme.scrollbar().appearance.fade_delay_ms);
        if now < fade_start {
            (1.0, Some(fade_start))
        } else {
            (0.0, None)
        }
    }

    fn transition_value(
        &mut self,
        pass: &mut VisualPass<'_>,
        step: TransitionStep,
    ) -> (f32, scene::Motion) {
        pass.seen.insert(step.key.clone());
        let transition = self.transitions.entry(step.key.clone()).or_insert_with(|| {
            VisualTransition::settled(step.base, step.base, pass.now, step.duration)
        });
        transition.retarget(step.desired, pass.now);
        let value = transition.value_at(pass.now);
        let is_animating = transition.is_animating_at(pass.now);
        let motion = if is_animating {
            scene::Motion::Moving
        } else {
            scene::Motion::Resting
        };

        if is_animating {
            *pass.schedule = pass.schedule.merge(animation::Schedule::NextFrame);
        } else if transition.is_at_base(pass.now) {
            pass.remove.push(step.key);
        }

        (value, motion)
    }
}

impl notification::Listener<window::Departed> for Animations {
    fn notify(&mut self, window: &window::Id) -> notification::Reaction {
        self.transitions.retain(|key, _| key.window != *window);
        self.scrollbar_activity
            .retain(|key, _| key.window != *window);
        self.scrollbar_offsets
            .retain(|key, _| key.window != *window);
        self.visuals.remove(window);
        notification::Reaction::ignored()
    }
}

#[cfg(test)]
impl Animations {
    pub(super) fn residue_count(&self, window: window::Id) -> usize {
        self.transitions
            .keys()
            .filter(|key| key.window == window)
            .count()
            + self
                .scrollbar_activity
                .keys()
                .filter(|key| key.window == window)
                .count()
            + self
                .scrollbar_offsets
                .keys()
                .filter(|key| key.window == window)
                .count()
            + usize::from(self.visuals.contains_key(&window))
    }
}

fn caret_visible_for(frame: &layout::Frame, now: Instant) -> Option<bool> {
    if !frame.is_focused() {
        return None;
    }

    if let Some(text_area) = frame.text_area() {
        let area = text_area.area_model();
        if !area.paints_caret()
            || text_area
                .buffer()
                .has_selection_for_state(text_area.state())
        {
            return Some(false);
        }

        let epoch = text_area.caret_epoch().unwrap_or(now);
        return Some(text::view::ViewState::new_at(0.0, epoch).caret_visible(now));
    }

    if let Some(text_box) = frame.text_box() {
        if text_box.cursor().is_none()
            || text_box
                .selection()
                .is_some_and(|selection| selection.start != selection.end)
        {
            return Some(false);
        }

        let epoch = text_box.caret_epoch().unwrap_or(now);
        return Some(text::view::ViewState::new_at(0.0, epoch).caret_visible(now));
    }

    None
}

impl Update {
    pub(super) fn visuals(&self) -> &scene::Visuals {
        &self.visuals
    }

    pub(super) fn schedule(&self) -> animation::Schedule {
        self.schedule
    }
}

impl VisualTransition {
    fn settled(value: f32, base_value: f32, now: Instant, duration: Duration) -> Self {
        Self {
            transition: Transition::settled(value, now, duration, SLIDER_TRACK_EASING),
            base_value,
        }
    }

    fn retarget(&mut self, value: f32, now: Instant) {
        self.transition.retarget(value, now);
    }

    fn value_at(self, now: Instant) -> f32 {
        self.transition.value_at(now)
    }

    fn from(self) -> f32 {
        self.transition.from()
    }

    fn target(self) -> f32 {
        self.transition.target()
    }

    fn eased_progress_at(self, now: Instant) -> f32 {
        self.transition.eased_progress_at(now)
    }

    fn is_animating_at(self, now: Instant) -> bool {
        self.transition.is_animating_at(now)
    }

    fn is_at_base(self, now: Instant) -> bool {
        !self.is_animating_at(now) && self.transition.target() == self.base_value
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ScrollKey {
    window: window::Id,
    target: interaction::Target,
}

fn idle_opacity_for(presentation: view::ScrollChromePresentation) -> f32 {
    match presentation {
        view::ScrollChromePresentation::Overlay => 0.0,
        view::ScrollChromePresentation::Consuming => 1.0,
    }
}
