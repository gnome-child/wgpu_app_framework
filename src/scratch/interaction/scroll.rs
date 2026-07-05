use super::Target;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Scroll {
    offsets: Vec<ScrollEntry>,
    reveal_requests: Vec<Target>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScrollEntry {
    target: Target,
    offset: ScrollOffset,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ScrollOffset {
    x: i32,
    y: i32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ScrollDelta {
    x: i32,
    y: i32,
}

impl Scroll {
    pub fn offset(&self, target: &Target) -> ScrollOffset {
        self.offsets
            .iter()
            .find(|entry| &entry.target == target)
            .map(|entry| entry.offset)
            .unwrap_or_default()
    }

    pub fn should_reveal(&self, target: &Target) -> bool {
        self.reveal_requests.iter().any(|request| request == target)
    }

    pub(super) fn scroll_by(&mut self, target: Target, delta: ScrollDelta) -> bool {
        if delta.is_zero() {
            return false;
        }

        let Some(index) = self.offsets.iter().position(|entry| entry.target == target) else {
            let offset = ScrollOffset::default().scrolled_by(delta);
            if offset.is_zero() {
                return false;
            }

            self.offsets.push(ScrollEntry { target, offset });
            return true;
        };

        let before = self.offsets[index].offset;
        let offset = before.scrolled_by(delta);
        if offset.is_zero() {
            self.offsets.remove(index);
        } else {
            self.offsets[index].offset = offset;
        }

        before != offset
    }

    pub(super) fn scroll_to(&mut self, target: Target, offset: ScrollOffset) -> bool {
        let Some(index) = self.offsets.iter().position(|entry| entry.target == target) else {
            if offset.is_zero() {
                return false;
            }

            self.offsets.push(ScrollEntry { target, offset });
            return true;
        };

        let before = self.offsets[index].offset;
        if offset.is_zero() {
            self.offsets.remove(index);
        } else {
            self.offsets[index].offset = offset;
        }

        before != offset
    }

    pub(super) fn reveal(&mut self, target: Target) -> bool {
        if self.should_reveal(&target) {
            return false;
        }

        self.reveal_requests.push(target);
        true
    }

    pub(super) fn clear_reveal(&mut self, target: &Target) -> bool {
        let Some(index) = self
            .reveal_requests
            .iter()
            .position(|request| request == target)
        else {
            return false;
        };

        self.reveal_requests.remove(index);
        true
    }
}

impl ScrollOffset {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn x(self) -> i32 {
        self.x
    }

    pub fn y(self) -> i32 {
        self.y
    }

    fn scrolled_by(self, delta: ScrollDelta) -> Self {
        Self {
            x: self.x.saturating_add(delta.x),
            y: self.y.saturating_add(delta.y),
        }
    }

    fn is_zero(self) -> bool {
        self.x == 0 && self.y == 0
    }
}

impl ScrollDelta {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn horizontal(x: i32) -> Self {
        Self { x, y: 0 }
    }

    pub fn vertical(y: i32) -> Self {
        Self { x: 0, y }
    }

    pub fn x(self) -> i32 {
        self.x
    }

    pub fn y(self) -> i32 {
        self.y
    }

    fn is_zero(self) -> bool {
        self.x == 0 && self.y == 0
    }
}
