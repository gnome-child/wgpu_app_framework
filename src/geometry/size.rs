#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub(crate) width: i32,
    pub(crate) height: i32,
}

impl Size {
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height }.sanitized()
    }

    pub fn width(self) -> i32 {
        self.width
    }

    pub fn height(self) -> i32 {
        self.height
    }

    pub(crate) fn sanitized(self) -> Self {
        Self {
            width: self.width.max(0),
            height: self.height.max(0),
        }
    }
}
