use super::{
    geometry::{Point, Size},
    view,
};

mod algorithm;
pub mod engine;
pub mod frame;
pub mod hit;
mod measure;
pub mod path;
pub(in crate::scratch) mod text;

use frame::Frame;
use hit::Hit;
pub(super) use text::TextService;

#[derive(Clone)]
pub struct Layout {
    size: Size,
    frames: Vec<Frame>,
}

impl Layout {
    pub fn compose(view: &view::View, size: Size, engine: &mut engine::Engine) -> Self {
        let size = size.sanitized();
        let frames = algorithm::compose_frames(view.root(), size, engine);

        Self { size, frames }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }

    pub fn hit_test(&self, point: Point) -> Option<Hit> {
        self.frames
            .iter()
            .rev()
            .find(|frame| frame.target_is_some() && frame.rect().contains(point))
            .cloned()
            .map(Hit::new)
    }

    pub fn find_role(&self, role: view::node::Role) -> Vec<&Frame> {
        self.frames
            .iter()
            .filter(|frame| frame.role() == role)
            .collect()
    }
}
