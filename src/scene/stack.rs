use std::sync::Arc;

use super::super::geometry;
use super::{
    Color, Commit, Content, MaterialRealizationReport, MaterialRenderer, Properties, Residency,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Stack {
    clear: Color,
    layers: Vec<Layer>,
    spatial_supplements: Vec<SpatialSupplement>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Layer {
    commit: Arc<Commit>,
    drawable: Arc<Commit>,
    residencies: Arc<[Residency]>,
    properties: Arc<Properties>,
    origin: geometry::Point,
    bounds: geometry::Rect,
    opacity: f32,
    force_group: bool,
    material: MaterialProjection,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpatialSupplement {
    commit: Arc<Commit>,
    properties: Arc<Properties>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MaterialProjection {
    Source,
    WithoutBackdropSampling,
    NativePopup {
        opaque: bool,
        reports: Arc<[MaterialRealizationReport]>,
    },
}

impl Stack {
    pub(crate) fn new(
        commit: Arc<Commit>,
        drawable: Arc<Commit>,
        residencies: Arc<[Residency]>,
        properties: Properties,
    ) -> Self {
        let clear = commit.clear();
        Self {
            clear,
            layers: vec![Layer::base(commit, drawable, residencies, properties)],
            spatial_supplements: Vec::new(),
        }
    }

    pub(crate) fn from_layer(clear: Color, layer: Layer) -> Self {
        Self {
            clear,
            layers: vec![layer],
            spatial_supplements: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    pub(crate) fn push_spatial_supplement(
        &mut self,
        commit: Arc<Commit>,
        properties: Arc<Properties>,
    ) {
        debug_assert!(properties.require_compatible(&commit).is_ok());
        self.spatial_supplements
            .push(SpatialSupplement { commit, properties });
    }

    pub(crate) fn clear(&self) -> Color {
        self.clear
    }

    pub(crate) fn layers(&self) -> &[Layer] {
        &self.layers
    }

    pub(crate) fn spatial_supplements(&self) -> &[SpatialSupplement] {
        &self.spatial_supplements
    }

    pub(crate) fn base(&self) -> &Layer {
        self.layers
            .first()
            .expect("a scene stack always contains its base layer")
    }

    pub(crate) fn scroll_offset(
        &self,
        node: super::super::composition::tree::NodeId,
    ) -> Option<super::super::interaction::ScrollOffset> {
        self.layers
            .iter()
            .find_map(|layer| layer.properties().scroll_offset(node))
    }

    pub(crate) fn with_base_properties(&self, properties: Properties) -> Self {
        debug_assert!(properties.require_compatible(self.base().commit()).is_ok());
        let mut stack = self.clone();
        stack.layers[0].properties = Arc::new(properties);
        stack
    }

    pub(crate) fn with_base_properties_and_spatial_supplements(
        &self,
        properties: Properties,
        supplements: &Self,
    ) -> Self {
        let mut stack = self.with_base_properties(properties);
        stack.spatial_supplements = supplements.spatial_supplements.clone();
        stack
    }

    pub(crate) fn with_spatial_supplements(&self, supplements: &Self) -> Self {
        let mut stack = self.clone();
        stack.spatial_supplements = supplements.spatial_supplements.clone();
        stack
    }

    pub(crate) fn project_base_properties(
        &self,
        candidate: &Properties,
    ) -> Option<(Properties, bool)> {
        let base = self.base();
        candidate
            .project_onto_with_scroll(base.drawable_commit(), base.properties(), |node, offset| {
                base.residencies()
                    .iter()
                    .find(|residency| residency.scroll() == node)
                    .map_or(offset, |residency| residency.project(offset))
            })
            .ok()
    }

    pub(crate) fn base_snapshots(&self) -> (Arc<Commit>, Arc<Commit>, Arc<[Residency]>) {
        let base = self.base();
        (
            Arc::clone(base.commit()),
            Arc::clone(base.drawable_commit()),
            Arc::clone(&base.residencies),
        )
    }

    pub(crate) fn project_base_properties_toward(
        &self,
        active: &Self,
        desired: &Properties,
    ) -> Option<Properties> {
        let (properties, _) = self.project_base_properties(desired)?;
        let mut advances = false;
        for residency in self.base().residencies() {
            let node = residency.scroll();
            let projected = properties.scroll_offset(node)?;
            match (active.scroll_offset(node), desired.scroll_offset(node)) {
                (Some(from), Some(to)) => {
                    if !scroll_offset_lies_between(from, projected, to) {
                        return None;
                    }
                    advances |= projected != from;
                }
                _ if projected == super::super::interaction::ScrollOffset::default() => {}
                _ => return None,
            }
        }
        advances.then_some(properties)
    }

    pub(crate) fn same_structure(&self, other: &Self) -> bool {
        self.clear == other.clear
            && self.layers.len() == other.layers.len()
            && self.layers.iter().zip(&other.layers).all(|(left, right)| {
                Arc::ptr_eq(left.commit(), right.commit())
                    && Arc::ptr_eq(left.drawable_commit(), right.drawable_commit())
                    && left.same_residencies(right)
                    && left.origin == right.origin
                    && left.bounds == right.bounds
                    && left.force_group == right.force_group
                    && left.material == right.material
            })
    }

    pub(crate) fn same_presented_property_state(&self, other: &Self) -> bool {
        self.same_structure(other)
            && self
                .layers
                .iter()
                .zip(&other.layers)
                .all(|(left, right)| left.properties().serial() == right.properties().serial())
            && self.spatial_supplements.len() == other.spatial_supplements.len()
            && self
                .spatial_supplements
                .iter()
                .zip(&other.spatial_supplements)
                .all(|(left, right)| {
                    Arc::ptr_eq(&left.commit, &right.commit)
                        && left.properties.serial() == right.properties.serial()
                })
    }
}

fn scroll_offset_lies_between(
    from: super::super::interaction::ScrollOffset,
    projected: super::super::interaction::ScrollOffset,
    to: super::super::interaction::ScrollOffset,
) -> bool {
    let contains =
        |from: i32, projected: i32, to: i32| (from.min(to)..=from.max(to)).contains(&projected);
    contains(from.x(), projected.x(), to.x()) && contains(from.y(), projected.y(), to.y())
}

impl Layer {
    fn base(
        commit: Arc<Commit>,
        drawable: Arc<Commit>,
        residencies: Arc<[Residency]>,
        properties: Properties,
    ) -> Self {
        debug_assert!(
            residencies
                .iter()
                .all(|residency| residency.require_compatible(&commit).is_ok())
        );
        let bounds = geometry::Rect::from_size(commit.size());
        Self {
            commit,
            drawable,
            residencies,
            properties: Arc::new(properties),
            origin: geometry::Point::new(0, 0),
            bounds,
            opacity: 1.0,
            force_group: false,
            material: MaterialProjection::Source,
        }
    }

    pub(crate) fn projected(
        commit: Arc<Commit>,
        residencies: Arc<[Residency]>,
        properties: Arc<Properties>,
        origin: geometry::Point,
        bounds: geometry::Rect,
        opacity: f32,
        force_group: bool,
        material: MaterialProjection,
    ) -> Self {
        debug_assert!(properties.require_compatible(&commit).is_ok());
        debug_assert!(
            residencies
                .iter()
                .all(|residency| residency.require_compatible(&commit).is_ok())
        );
        Self {
            drawable: Arc::clone(&commit),
            commit,
            residencies,
            properties,
            origin,
            bounds,
            opacity: if opacity.is_finite() {
                opacity.clamp(0.0, 1.0)
            } else {
                0.0
            },
            force_group,
            material,
        }
    }

    pub(crate) fn commit(&self) -> &Arc<Commit> {
        &self.commit
    }

    pub(crate) fn drawable_commit(&self) -> &Arc<Commit> {
        &self.drawable
    }

    pub(crate) fn properties(&self) -> &Properties {
        &self.properties
    }

    pub(crate) fn property_snapshot(&self) -> Arc<Properties> {
        Arc::clone(&self.properties)
    }

    pub(crate) fn residencies(&self) -> &[Residency] {
        &self.residencies
    }

    pub(crate) fn origin(&self) -> geometry::Point {
        self.origin
    }

    pub(crate) fn bounds(&self) -> geometry::Rect {
        self.bounds
    }

    pub(crate) fn opacity(&self) -> f32 {
        self.opacity
    }

    pub(crate) fn force_group(&self) -> bool {
        self.force_group
    }

    pub(crate) fn material(&self) -> &MaterialProjection {
        &self.material
    }

    fn same_residencies(&self, other: &Self) -> bool {
        self.residencies.len() == other.residencies.len()
            && self
                .residencies
                .iter()
                .zip(other.residencies.iter())
                .all(|(left, right)| {
                    left.scroll() == right.scroll() && left.revision() == right.revision()
                })
    }
}

impl SpatialSupplement {
    pub(crate) fn commit_snapshot(&self) -> Arc<Commit> {
        Arc::clone(&self.commit)
    }

    pub(crate) fn property_snapshot(&self) -> Arc<Properties> {
        Arc::clone(&self.properties)
    }
}

impl MaterialProjection {
    pub(crate) fn projected_content(&self, content: &Content) -> Option<(Content, u8)> {
        match self {
            MaterialProjection::Source => Some((content.clone(), 0)),
            MaterialProjection::WithoutBackdropSampling => Some(match content {
                Content::Pane(pane) => (Content::Pane(pane.without_backdrop_sampling()), 1),
                _ => (content.clone(), 0),
            }),
            MaterialProjection::NativePopup { opaque, reports } => match content {
                Content::Shadow(_) => None,
                Content::Pane(pane) => {
                    let mut fidelity = Vec::new();
                    let projected = super::resolve_material_pane(
                        pane,
                        MaterialRenderer::NativePopup { opaque: *opaque },
                        reports,
                        &mut fidelity,
                    )?;
                    match projected {
                        super::Primitive::Pane(pane) => Some((Content::Pane(pane), 3)),
                        super::Primitive::Quad(quad) => Some((Content::Quad(quad), 2)),
                        _ => unreachable!(
                            "native pane projection stays pane-shaped or falls back to a quad"
                        ),
                    }
                }
                _ => Some((content.clone(), 0)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::scroll_offset_lies_between;
    use crate::interaction::ScrollOffset;

    #[test]
    fn progressive_scroll_projection_is_componentwise_and_never_overshoots() {
        assert!(scroll_offset_lies_between(
            ScrollOffset::new(10, 20),
            ScrollOffset::new(20, 30),
            ScrollOffset::new(30, 40),
        ));
        assert!(scroll_offset_lies_between(
            ScrollOffset::new(30, 40),
            ScrollOffset::new(20, 30),
            ScrollOffset::new(10, 20),
        ));
        assert!(scroll_offset_lies_between(
            ScrollOffset::new(10, 40),
            ScrollOffset::new(20, 30),
            ScrollOffset::new(30, 20),
        ));
        assert!(!scroll_offset_lies_between(
            ScrollOffset::new(10, 20),
            ScrollOffset::new(31, 30),
            ScrollOffset::new(30, 40),
        ));
        assert!(!scroll_offset_lies_between(
            ScrollOffset::new(10, 40),
            ScrollOffset::new(20, 41),
            ScrollOffset::new(30, 20),
        ));
    }
}
