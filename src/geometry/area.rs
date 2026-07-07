use super::Size;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogicalArea {
    width: f32,
    height: f32,
}

impl LogicalArea {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width: width.max(0.0),
            height: height.max(0.0),
        }
    }

    pub fn width(self) -> f32 {
        self.width
    }

    pub fn height(self) -> f32 {
        self.height
    }

    pub(crate) fn from_size(size: Size) -> Self {
        Self::new(size.width().max(1) as f32, size.height().max(1) as f32)
    }
}
