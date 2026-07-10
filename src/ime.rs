use crate::{geometry, interaction, window};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Update {
    parent: window::Id,
    target: Option<Target>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Target {
    Parent {
        area: geometry::Rect,
    },
    Popup {
        id: interaction::Id,
        area: geometry::Rect,
    },
}

impl Update {
    pub(crate) fn new(parent: window::Id, target: Option<Target>) -> Self {
        Self { parent, target }
    }

    pub(crate) fn parent(self) -> window::Id {
        self.parent
    }

    pub(crate) fn target(self) -> Option<Target> {
        self.target
    }
}

impl Target {
    pub(crate) fn parent(area: geometry::Rect) -> Self {
        Self::Parent { area }
    }

    pub(crate) fn popup(
        id: interaction::Id,
        area: geometry::Rect,
        popup_bounds: geometry::Rect,
    ) -> Self {
        Self::Popup {
            id,
            area: geometry::Rect::new(
                area.x().saturating_sub(popup_bounds.x()),
                area.y().saturating_sub(popup_bounds.y()),
                area.width(),
                area.height(),
            ),
        }
    }

    pub(crate) fn area(self) -> geometry::Rect {
        match self {
            Self::Parent { area } | Self::Popup { area, .. } => area,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_target_owns_popup_local_caret_geometry() {
        let id = interaction::Id::new("palette");
        let target = Target::popup(
            id,
            geometry::Rect::new(140, 90, 1, 18),
            geometry::Rect::new(100, 50, 300, 200),
        );

        assert_eq!(
            target,
            Target::Popup {
                id,
                area: geometry::Rect::new(40, 40, 1, 18),
            }
        );
    }
}
