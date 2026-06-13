#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Physical {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Logical {
    x: f32,
    y: f32,
}

pub fn physical(x: f32, y: f32) -> Physical {
    Physical { x, y }
}

pub fn logical(x: f32, y: f32) -> Logical {
    Logical { x, y }
}

impl Physical {
    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn to_logical(self, scale_factor: f32) -> Logical {
        debug_assert!(scale_factor > 0.0);

        Logical {
            x: self.x / scale_factor,
            y: self.y / scale_factor,
        }
    }
}

impl Logical {
    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn to_physical(self, scale_factor: f32) -> Physical {
        debug_assert!(scale_factor > 0.0);

        Physical {
            x: self.x * scale_factor,
            y: self.y * scale_factor,
        }
    }
}
