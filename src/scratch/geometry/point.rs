#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub(in crate::scratch) x: i32,
    pub(in crate::scratch) y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn x(self) -> i32 {
        self.x
    }

    pub fn y(self) -> i32 {
        self.y
    }
}
