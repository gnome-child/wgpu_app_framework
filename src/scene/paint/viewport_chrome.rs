#[cfg(test)]
use super::super::Scene;
use super::super::{Clip, CommitBuilder, Content, ContentProjection, Outline, Quad};
use crate::composition;

#[derive(Clone)]
pub(super) struct Scope {
    clips: Vec<Clip>,
}

#[derive(Clone)]
pub(super) struct Projection {
    owner: composition::tree::NodeId,
    scope: Scope,
    layer: Layer,
    paint: Vec<Paint>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Layer {
    Scrollbar,
    Focus,
}

#[derive(Clone)]
enum Paint {
    Outline(Outline),
    Quad(Quad, ContentProjection),
}

impl Scope {
    pub(super) fn new(clips: impl IntoIterator<Item = Clip>) -> Self {
        let mut resolved = Vec::new();
        for clip in clips {
            if !resolved.contains(&clip) {
                resolved.push(clip);
            }
        }
        Self { clips: resolved }
    }
}

impl Projection {
    pub(super) fn layer_order(&self) -> u8 {
        match self.layer {
            Layer::Scrollbar => 0,
            Layer::Focus => 1,
        }
    }

    pub(super) fn outline(
        owner: composition::tree::NodeId,
        scope: Scope,
        outline: Outline,
    ) -> Self {
        Self {
            owner,
            scope,
            layer: Layer::Focus,
            paint: vec![Paint::Outline(outline)],
        }
    }

    pub(super) fn scrollbar(owner: composition::tree::NodeId, scope: Scope) -> Self {
        Self {
            owner,
            scope,
            layer: Layer::Scrollbar,
            paint: Vec::new(),
        }
    }

    pub(super) fn push_scrollbar_quad(&mut self, quad: Quad, projection: ContentProjection) {
        self.paint.push(Paint::Quad(quad, projection));
    }

    pub(super) fn is_empty(&self) -> bool {
        self.paint.is_empty()
    }

    #[cfg(test)]
    pub(super) fn paint_into(self, scene: &mut Scene) {
        let clip_count = self.scope.clips.len();
        for clip in self.scope.clips {
            scene.push_clip(clip);
        }
        for paint in self.paint {
            match paint {
                Paint::Outline(outline) => scene.push_outline(outline),
                Paint::Quad(quad, _) => scene.push_quad(quad),
            }
        }
        for _ in 0..clip_count {
            scene.pop_clip();
        }
    }
    pub(super) fn append_to_commit(self, commit: &mut CommitBuilder) {
        let clip_count = self.scope.clips.len();
        for clip in self.scope.clips {
            commit.push_clip(clip);
        }
        for paint in self.paint {
            match paint {
                Paint::Outline(outline) => commit.push_projected_content(
                    self.owner,
                    Content::Outline(outline),
                    ContentProjection::Normal,
                ),
                Paint::Quad(quad, projection) => {
                    commit.push_projected_content(self.owner, Content::Quad(quad), projection);
                }
            }
        }
        for _ in 0..clip_count {
            commit.pop_clip();
        }
    }
}

#[cfg(test)]
pub(super) fn paint(scene: &mut Scene, mut projections: Vec<Projection>) {
    projections.sort_by_key(|projection| projection.layer);
    for projection in projections {
        projection.paint_into(scene);
    }
}
