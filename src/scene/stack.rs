use std::sync::Arc;

use super::super::geometry;
use super::{Color, Commit, Content, MaterialRealizationReport, MaterialRenderer, Properties};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Stack {
    clear: Color,
    layers: Vec<Layer>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Layer {
    commit: Arc<Commit>,
    properties: Arc<Properties>,
    origin: geometry::Point,
    bounds: geometry::Rect,
    opacity: f32,
    force_group: bool,
    material: MaterialProjection,
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
    pub(crate) fn new(commit: Arc<Commit>, properties: Properties) -> Self {
        let clear = commit.clear();
        Self {
            clear,
            layers: vec![Layer::base(commit, properties)],
        }
    }

    pub(crate) fn from_layer(clear: Color, layer: Layer) -> Self {
        Self {
            clear,
            layers: vec![layer],
        }
    }

    pub(crate) fn push(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    pub(crate) fn clear(&self) -> Color {
        self.clear
    }

    pub(crate) fn layers(&self) -> &[Layer] {
        &self.layers
    }

    pub(crate) fn base(&self) -> &Layer {
        self.layers
            .first()
            .expect("a scene stack always contains its base layer")
    }

    pub(crate) fn with_base_properties(&self, properties: Properties) -> Self {
        debug_assert!(properties.require_compatible(self.base().commit()).is_ok());
        let mut stack = self.clone();
        stack.layers[0].properties = Arc::new(properties);
        stack
    }

    pub(crate) fn same_structure(&self, other: &Self) -> bool {
        self.clear == other.clear
            && self.layers.len() == other.layers.len()
            && self.layers.iter().zip(&other.layers).all(|(left, right)| {
                Arc::ptr_eq(left.commit(), right.commit())
                    && left.origin == right.origin
                    && left.bounds == right.bounds
                    && left.force_group == right.force_group
                    && left.material == right.material
            })
    }
}

impl Layer {
    fn base(commit: Arc<Commit>, properties: Properties) -> Self {
        let bounds = geometry::Rect::from_size(commit.size());
        Self {
            commit,
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
        properties: Arc<Properties>,
        origin: geometry::Point,
        bounds: geometry::Rect,
        opacity: f32,
        force_group: bool,
        material: MaterialProjection,
    ) -> Self {
        debug_assert!(properties.require_compatible(&commit).is_ok());
        Self {
            commit,
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

    pub(crate) fn properties(&self) -> &Properties {
        &self.properties
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
