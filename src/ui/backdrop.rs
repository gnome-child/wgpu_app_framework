use crate::paint;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Backdrop {
    fill: Option<paint::Color>,
    blur: Option<f32>,
}

impl Backdrop {
    pub fn new() -> Self {
        Self {
            fill: None,
            blur: None,
        }
    }

    pub fn glass(fill: paint::Color) -> Self {
        Self::new().with_fill(fill).with_blur(1.0)
    }

    pub fn with_fill(mut self, fill: paint::Color) -> Self {
        self.fill = Some(fill);
        self
    }

    pub fn with_blur(mut self, amount: f32) -> Self {
        let amount = amount.clamp(0.0, 1.0);
        self.blur = (amount > 0.0).then_some(amount);
        self
    }

    pub fn fill(self) -> Option<paint::Color> {
        self.fill
    }

    pub fn blur(self) -> Option<f32> {
        self.blur
    }
}

impl Default for Backdrop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backdrop_stores_fill_and_normalized_blur() {
        let backdrop = Backdrop::new()
            .with_fill(paint::Color::rgba(0.1, 0.2, 0.3, 0.4))
            .with_blur(0.5);

        assert_eq!(
            backdrop.fill(),
            Some(paint::Color::rgba(0.1, 0.2, 0.3, 0.4))
        );
        assert_eq!(backdrop.blur(), Some(0.5));
    }

    #[test]
    fn backdrop_blur_below_zero_clamps_to_no_blur() {
        assert_eq!(Backdrop::new().with_blur(-1.0).blur(), None);
    }

    #[test]
    fn backdrop_blur_above_one_clamps_to_full_blur() {
        assert_eq!(Backdrop::new().with_blur(2.0).blur(), Some(1.0));
    }

    #[test]
    fn glass_preset_uses_fill_and_full_blur() {
        let fill = paint::Color::rgba(0.1, 0.2, 0.3, 0.4);
        let backdrop = Backdrop::glass(fill);

        assert_eq!(backdrop.fill(), Some(fill));
        assert_eq!(backdrop.blur(), Some(1.0));
    }
}
