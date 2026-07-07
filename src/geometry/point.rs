#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub(crate) x: i32,
    pub(crate) y: i32,
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
