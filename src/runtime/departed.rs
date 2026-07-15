use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::{context as command_context, notification, state, window};

use super::Runtime;

#[derive(Debug)]
pub(super) struct WindowMap<T>(HashMap<window::Id, T>);

impl<T> Default for WindowMap<T> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<T> Deref for WindowMap<T> {
    type Target = HashMap<window::Id, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for WindowMap<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> notification::Listener<window::Departed> for WindowMap<T> {
    fn notify(&mut self, window: &window::Id) -> notification::Reaction {
        self.remove(window);
        notification::Reaction::ignored()
    }
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(super) fn deliver_departed(&mut self) {
        for window in self.session.take_departed() {
            let listeners: [&mut dyn notification::Listener<window::Departed>; 11] = [
                &mut self.layout_cache,
                &mut self.scene,
                &mut self.presented_geometry,
                &mut self.virtual_materializations,
                &mut self.virtual_measurements,
                &mut self.overlays,
                &mut self.animation_schedules,
                &mut self.visual_animations,
                &mut self.composition,
                &mut self.diagnostics,
                &mut self.gesture,
            ];

            for listener in listeners {
                listener.notify(&window);
            }

            let reaction = self.transact_notification::<window::Departed>(
                None,
                Some(window),
                window,
                command_context::Source::Programmatic,
            );
            if reaction.changed_state() {
                self.request_all_redraws();
            }
        }
    }
}

#[cfg(test)]
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct WindowResidues {
    pub(crate) layout_cache: usize,
    pub(crate) scene: usize,
    pub(crate) presented_geometry: usize,
    pub(crate) virtual_materializations: usize,
    pub(crate) virtual_measurements: usize,
    pub(crate) overlays: usize,
    pub(crate) animation_schedules: usize,
    pub(crate) visual_animations: usize,
    pub(crate) composition: usize,
    pub(crate) diagnostics: usize,
    pub(crate) gesture: usize,
}

#[cfg(test)]
impl WindowResidues {
    pub(crate) fn total(&self) -> usize {
        self.layout_cache
            + self.scene
            + self.presented_geometry
            + self.virtual_materializations
            + self.virtual_measurements
            + self.overlays
            + self.animation_schedules
            + self.visual_animations
            + self.composition
            + self.diagnostics
            + self.gesture
    }
}

#[cfg(test)]
impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(crate) fn window_residues(&self, window: window::Id) -> WindowResidues {
        WindowResidues {
            layout_cache: usize::from(self.layout_cache.contains_key(&window)),
            scene: self.scene.residue_count(window),
            presented_geometry: usize::from(self.presented_geometry.contains_key(&window)),
            virtual_materializations: usize::from(
                self.virtual_materializations.contains_key(&window),
            ),
            virtual_measurements: usize::from(self.virtual_measurements.contains_key(&window)),
            overlays: self.overlays.residue_count(window),
            animation_schedules: usize::from(self.animation_schedules.contains_key(&window)),
            visual_animations: self.visual_animations.residue_count(window),
            composition: usize::from(self.composition.get(window).is_some()),
            diagnostics: usize::from(self.diagnostics.get(window).is_some()),
            gesture: self.gesture_residue_count(window),
        }
    }
}
