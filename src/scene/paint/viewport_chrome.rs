use super::super::{Clip, Outline, Quad, Scene};

#[derive(Clone)]
pub(super) struct Scope {
    clips: Vec<Clip>,
}

pub(super) struct Projection {
    scope: Scope,
    layer: Layer,
    paint: Vec<Paint>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Layer {
    Scrollbar,
    Focus,
}

enum Paint {
    Outline(Outline),
    Quad(Quad),
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
    pub(super) fn outline(scope: Scope, outline: Outline) -> Self {
        Self {
            scope,
            layer: Layer::Focus,
            paint: vec![Paint::Outline(outline)],
        }
    }

    pub(super) fn scrollbar(scope: Scope) -> Self {
        Self {
            scope,
            layer: Layer::Scrollbar,
            paint: Vec::new(),
        }
    }

    pub(super) fn push_quad(&mut self, quad: Quad) {
        self.paint.push(Paint::Quad(quad));
    }

    pub(super) fn is_empty(&self) -> bool {
        self.paint.is_empty()
    }
}

pub(super) fn paint(scene: &mut Scene, mut projections: Vec<Projection>) {
    projections.sort_by_key(|projection| projection.layer);
    for projection in projections {
        let clip_count = projection.scope.clips.len();
        for clip in projection.scope.clips {
            scene.push_clip(clip);
        }
        for paint in projection.paint {
            match paint {
                Paint::Outline(outline) => scene.push_outline(outline),
                Paint::Quad(quad) => scene.push_quad(quad),
            }
        }
        for _ in 0..clip_count {
            scene.pop_clip();
        }
    }
}
