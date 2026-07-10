#[derive(Debug, Clone, PartialEq)]
pub struct Slider {
    label: String,
    value: f64,
    start: f64,
    end: f64,
}

impl Slider {
    pub fn new(label: impl Into<String>, value: f64, start: f64, end: f64) -> Self {
        let (start, end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        let value = value.clamp(start, end);

        Self {
            label: label.into(),
            value,
            start,
            end,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn start(&self) -> f64 {
        self.start
    }

    pub fn end(&self) -> f64 {
        self.end
    }

    pub(crate) fn fraction(&self) -> f64 {
        let span = self.end - self.start;
        if span.abs() <= f64::EPSILON {
            return 0.0;
        }

        (self.value - self.start) / span
    }

    pub fn value_at_fraction(&self, fraction: f64) -> f64 {
        let fraction = if fraction.is_finite() {
            fraction.clamp(0.0, 1.0)
        } else {
            0.0
        };

        self.start + (self.end - self.start) * fraction
    }

    pub(in crate::view) fn display_label(&self) -> String {
        format!("{}: {:.2}", self.label, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::Slider;

    #[test]
    fn slider_owns_its_normalized_fraction() {
        assert_eq!(Slider::new("Level", 5.0, 0.0, 10.0).fraction(), 0.5);
        assert_eq!(Slider::new("Level", 5.0, 10.0, 0.0).fraction(), 0.5);
        assert_eq!(Slider::new("Level", 20.0, 0.0, 10.0).fraction(), 1.0);
        assert_eq!(Slider::new("Level", 5.0, 5.0, 5.0).fraction(), 0.0);
    }
}
