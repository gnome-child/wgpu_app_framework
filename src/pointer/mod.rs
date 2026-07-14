use super::window;
use std::time::Duration;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cursor {
    #[default]
    Default,
    Text,
    ResizeHorizontal,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    #[default]
    Primary,
    Secondary,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Update {
    window: window::Id,
    cursor: Cursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultiClickSettings {
    interval: Duration,
    distance_x: i32,
    distance_y: i32,
}

impl MultiClickSettings {
    pub fn new(interval: Duration, distance_x: i32, distance_y: i32) -> Self {
        Self {
            interval,
            distance_x: distance_x.max(1),
            distance_y: distance_y.max(1),
        }
    }

    pub(crate) fn accepts(self, elapsed: Duration, dx: i32, dy: i32) -> bool {
        elapsed <= self.interval && dx <= self.distance_x.max(1) && dy <= self.distance_y.max(1)
    }
}

impl Default for MultiClickSettings {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(500),
            distance_x: 4,
            distance_y: 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MultiClickSettings;
    use std::time::Duration;

    #[test]
    fn multi_click_settings_own_threshold_admission_without_platform_policy() {
        let settings = MultiClickSettings::new(Duration::from_millis(250), 3, 5);

        assert!(settings.accepts(Duration::from_millis(250), 3, 5));
        assert!(!settings.accepts(Duration::from_millis(251), 3, 5));
        assert!(!settings.accepts(Duration::from_millis(250), 4, 5));
        assert!(!settings.accepts(Duration::from_millis(250), 3, 6));
    }
}

impl Update {
    pub(crate) fn new(window: window::Id, cursor: Cursor) -> Self {
        Self { window, cursor }
    }

    pub fn window(self) -> window::Id {
        self.window
    }

    pub fn cursor(self) -> Cursor {
        self.cursor
    }
}
