use crate::geometry::area;
use crate::paint;

pub type Id = winit::window::WindowId;

#[derive(Debug, Clone)]
pub struct Options {
    pub title: String,
    pub inner_area: area::Physical,
    pub canvas_color: paint::Color,
}
