use crate::{composition, geometry, interaction, window};

/// The presentation surface that owns a pointer event and the frames it may
/// address. Surface identity is retained until hit testing; popup input is
/// never disguised as parent-window input.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Surface {
    #[default]
    Parent,
    Native(interaction::Id),
}

/// Monotonic identity for one complete popup presentation. Checkpoint 2 owns
/// serial allocation; checkpoint 1 carries the identity through realization.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Generation(u64);

/// Semantic identity of the captured responder path that produced a
/// contextual popup. The retained owner changes when that path is retargeted;
/// it is never inferred from pixels or parent-window presentation clocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ContextFingerprint(composition::tree::NodeId);

/// The selected host's resolved popup geometry.
///
/// `local_bounds` remains in retained parent-layout coordinates so the same
/// laid-out frames can be addressed without re-layout. `host_bounds` is the
/// final placement selected against the chosen host's available bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Realization {
    popup: interaction::Id,
    parent: window::Id,
    generation: Generation,
    geometry: Geometry,
}

/// Geometry resolved by the selected native host for one popup realization.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Geometry {
    local_bounds: geometry::Rect,
    host_bounds: geometry::Rect,
    visible_clip: geometry::Rect,
    visual_bounds: geometry::Rect,
    panel_offset: geometry::Point,
    scale: f64,
}

impl Generation {
    pub(crate) const fn new(serial: u64) -> Self {
        Self(serial)
    }

    #[cfg(test)]
    pub(crate) const fn initial() -> Self {
        Self(0)
    }

    pub(crate) const fn serial(self) -> u64 {
        self.0
    }
}

impl ContextFingerprint {
    pub(crate) const fn from_owner(owner: composition::tree::NodeId) -> Self {
        Self(owner)
    }
}

impl Geometry {
    pub(crate) fn new(
        local_bounds: geometry::Rect,
        host_bounds: geometry::Rect,
        visible_clip: geometry::Rect,
        visual_bounds: geometry::Rect,
        panel_offset: geometry::Point,
        scale: f64,
    ) -> Self {
        Self {
            local_bounds,
            host_bounds,
            visible_clip,
            visual_bounds,
            panel_offset,
            scale,
        }
    }
}

impl Realization {
    pub(crate) fn native(
        popup: interaction::Id,
        parent: window::Id,
        generation: Generation,
        geometry: Geometry,
    ) -> Self {
        Self {
            popup,
            parent,
            generation,
            geometry,
        }
    }

    pub(crate) fn popup(self) -> interaction::Id {
        self.popup
    }

    pub(crate) fn parent(self) -> window::Id {
        self.parent
    }

    pub(crate) fn generation(self) -> Generation {
        self.generation
    }

    #[cfg(test)]
    pub(crate) fn surface(self) -> Surface {
        Surface::Native(self.popup)
    }

    pub(crate) fn local_bounds(self) -> geometry::Rect {
        self.geometry.local_bounds
    }

    pub(crate) fn host_bounds(self) -> geometry::Rect {
        self.geometry.host_bounds
    }

    pub(crate) fn visible_clip(self) -> geometry::Rect {
        self.geometry.visible_clip
    }

    pub(crate) fn visual_bounds(self) -> geometry::Rect {
        self.geometry.visual_bounds
    }

    pub(crate) fn panel_offset(self) -> geometry::Point {
        self.geometry.panel_offset
    }

    pub(crate) fn panel_offset_physical(self) -> (i32, i32) {
        (
            (f64::from(self.panel_offset().x()) * self.scale()).round() as i32,
            (f64::from(self.panel_offset().y()) * self.scale()).round() as i32,
        )
    }

    pub(crate) fn scale(self) -> f64 {
        self.geometry.scale
    }

    pub(crate) fn with_generation(mut self, generation: Generation) -> Self {
        self.generation = generation;
        self
    }

    #[cfg(test)]
    pub(crate) fn local_to_host(self) -> geometry::Point {
        geometry::Point::new(
            self.host_bounds()
                .x()
                .saturating_sub(self.local_bounds().x()),
            self.host_bounds()
                .y()
                .saturating_sub(self.local_bounds().y()),
        )
    }

    pub(crate) fn visual_offset(self) -> geometry::Point {
        geometry::Point::new(
            self.visual_bounds()
                .x()
                .saturating_sub(self.host_bounds().x()),
            self.visual_bounds()
                .y()
                .saturating_sub(self.host_bounds().y()),
        )
    }

    /// Maps a point in the popup surface's panel-local coordinates back into
    /// the retained layout that produced the visible popup content.
    pub(crate) fn retained_point(self, surface_point: geometry::Point) -> geometry::Point {
        geometry::Point::new(
            self.local_bounds().x().saturating_add(surface_point.x()),
            self.local_bounds().y().saturating_add(surface_point.y()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_realization_keeps_local_and_host_geometry_distinct() {
        let realization = Realization::native(
            interaction::Id::new("menu"),
            window::Id::new(1),
            Generation::initial(),
            Geometry::new(
                geometry::Rect::new(700, 20, 120, 80),
                geometry::Rect::new(580, 20, 120, 80),
                geometry::Rect::new(580, 20, 120, 80),
                geometry::Rect::new(570, 10, 140, 100),
                geometry::Point::new(10, 10),
                1.25,
            ),
        );

        assert_eq!(realization.local_to_host(), geometry::Point::new(-120, 0));
        assert_eq!(
            realization.retained_point(geometry::Point::new(15, 12)),
            geometry::Point::new(715, 32)
        );
        assert_eq!(realization.surface(), Surface::Native(realization.popup()));
    }
}
