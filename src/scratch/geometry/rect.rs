use super::{Point, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub(in crate::scratch) x: i32,
    pub(in crate::scratch) y: i32,
    pub(in crate::scratch) width: i32,
    pub(in crate::scratch) height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width: width.max(0),
            height: height.max(0),
        }
    }

    pub fn x(self) -> i32 {
        self.x
    }

    pub fn y(self) -> i32 {
        self.y
    }

    pub fn width(self) -> i32 {
        self.width
    }

    pub fn height(self) -> i32 {
        self.height
    }

    pub(in crate::scratch) fn from_size(size: Size) -> Self {
        Self::new(0, 0, size.width, size.height)
    }

    pub(in crate::scratch) fn right(self) -> i32 {
        self.x.saturating_add(self.width)
    }

    pub(in crate::scratch) fn bottom(self) -> i32 {
        self.y.saturating_add(self.height)
    }

    pub(in crate::scratch) fn contains(self, point: Point) -> bool {
        point.x >= self.x && point.y >= self.y && point.x < self.right() && point.y < self.bottom()
    }
}
