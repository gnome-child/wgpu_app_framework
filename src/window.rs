use crate::geometry::area;
use crate::paint;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(u64);

impl Id {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Options {
    pub title: String,
    pub inner_area: area::Physical,
    pub canvas_color: paint::Color,
}
