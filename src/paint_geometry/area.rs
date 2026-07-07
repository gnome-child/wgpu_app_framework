#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Physical {
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Logical {
    width: f32,
    height: f32,
}

pub fn physical(width: u32, height: u32) -> Physical {
    Physical { width, height }
}

pub fn logical(width: f32, height: f32) -> Logical {
    Logical { width, height }
}

impl Physical {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn clamp_min(self, min: u32) -> Self {
        Self {
            width: self.width.max(min),
            height: self.height.max(min),
        }
    }

    pub fn clamp_max(self, max: u32) -> Self {
        Self {
            width: self.width.min(max),
            height: self.height.min(max),
        }
    }

    pub fn to_logical(self, scale_factor: f32) -> Logical {
        debug_assert!(scale_factor > 0.0);

        Logical {
            width: self.width as f32 / scale_factor,
            height: self.height as f32 / scale_factor,
        }
    }
}

impl Logical {
    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn clamp_min(self, min: f32) -> Self {
        Self {
            width: self.width.max(min),
            height: self.height.max(min),
        }
    }

    pub fn clamp_max(self, max: f32) -> Self {
        Self {
            width: self.width.min(max),
            height: self.height.min(max),
        }
    }

    pub fn to_physical(self, scale_factor: f32) -> Physical {
        debug_assert!(scale_factor > 0.0);

        Physical {
            width: (self.width * scale_factor).round() as u32,
            height: (self.height * scale_factor).round() as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_area_converts_to_logical_area() {
        let area = physical(300, 150).to_logical(1.5);

        assert_eq!(area, logical(200.0, 100.0));
    }

    #[test]
    fn logical_area_converts_to_rounded_physical_area() {
        let area = logical(10.4, 20.6).to_physical(2.0);

        assert_eq!(area, physical(21, 41));
    }

    #[test]
    fn physical_area_clamps_to_minimum_surface_size() {
        let area = physical(0, 2).clamp_min(1);

        assert_eq!(area, physical(1, 2));
    }
}
