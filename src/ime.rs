use crate::{composition, geometry, interaction, window};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Update {
    parent: window::Id,
    target: Option<Target>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Projection {
    parent: window::Id,
    target: Option<ProjectionTarget>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectionTarget {
    Parent {
        node: composition::tree::NodeId,
        area: geometry::Rect,
    },
    Popup {
        id: interaction::Id,
        area: geometry::Rect,
        bounds: geometry::Rect,
    },
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

impl Projection {
    pub(crate) fn new(parent: window::Id, target: Option<ProjectionTarget>) -> Self {
        Self { parent, target }
    }

    pub(crate) fn resolve(
        self,
        mut translated_rect: impl FnMut(
            composition::tree::NodeId,
            geometry::Rect,
        ) -> Option<geometry::Rect>,
    ) -> Update {
        let target = self.target.and_then(|target| match target {
            ProjectionTarget::Parent { node, area } => {
                translated_rect(node, area).map(Target::parent)
            }
            ProjectionTarget::Popup { id, area, bounds } => Some(Target::popup(id, area, bounds)),
        });
        Update::new(self.parent, target)
    }

    pub(crate) fn disabled(self) -> Update {
        Update::new(self.parent, None)
    }

    pub(crate) fn targets_popup(self) -> bool {
        matches!(self.target, Some(ProjectionTarget::Popup { .. }))
    }
}

impl ProjectionTarget {
    pub(crate) fn parent(node: composition::tree::NodeId, area: geometry::Rect) -> Self {
        Self::Parent { node, area }
    }

    pub(crate) fn popup(id: interaction::Id, area: geometry::Rect, bounds: geometry::Rect) -> Self {
        Self::Popup { id, area, bounds }
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

    #[test]
    fn projection_resolves_parent_and_popup_geometry_through_one_presented_transform() {
        let parent = window::Id::new(7);
        let mut next_node = 11;
        let node = composition::tree::NodeId::layout(&mut next_node);
        let area = geometry::Rect::new(140, 90, 1, 18);
        let translate = |candidate: composition::tree::NodeId, rect: geometry::Rect| {
            (candidate == node).then(|| {
                geometry::Rect::new(
                    rect.x().saturating_sub(20),
                    rect.y().saturating_sub(12),
                    rect.width(),
                    rect.height(),
                )
            })
        };

        assert_eq!(
            Projection::new(parent, Some(ProjectionTarget::parent(node, area))).resolve(translate),
            Update::new(
                parent,
                Some(Target::parent(geometry::Rect::new(120, 78, 1, 18)))
            )
        );
        assert_eq!(
            Projection::new(
                parent,
                Some(ProjectionTarget::popup(
                    interaction::Id::new("palette"),
                    geometry::Rect::new(120, 78, 1, 18),
                    geometry::Rect::new(100, 50, 300, 200),
                )),
            )
            .resolve(translate),
            Update::new(
                parent,
                Some(Target::Popup {
                    id: interaction::Id::new("palette"),
                    area: geometry::Rect::new(20, 28, 1, 18),
                }),
            )
        );
    }
}
