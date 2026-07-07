use super::Target;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Scroll {
    offsets: Vec<ScrollEntry>,
    reveal_requests: Vec<Reveal>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScrollEntry {
    target: Target,
    offset: ScrollOffset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reveal {
    Viewport(Target),
    ActiveDescendant { viewport: Target },
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
        self.reveal_requests
            .iter()
            .any(|request| matches!(request, Reveal::Viewport(viewport) if viewport == target))
    }

    pub(crate) fn active_descendant_targets(&self) -> Vec<Target> {
        self.reveal_requests
            .iter()
            .filter_map(|request| match request {
                Reveal::ActiveDescendant { viewport } => Some(viewport.clone()),
                Reveal::Viewport(_) => None,
            })
            .collect()
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
        let request = Reveal::Viewport(target);
        if self.reveal_requests.contains(&request) {
            return false;
        }

        self.reveal_requests.push(request);
        true
    }

    pub(super) fn reveal_active_descendant(&mut self, viewport: Target) -> bool {
        let request = Reveal::ActiveDescendant { viewport };
        if self.reveal_requests.contains(&request) {
            return false;
        }

        self.reveal_requests.push(request);
        true
    }

    pub(super) fn clear_reveal(&mut self, target: &Target) -> bool {
        let Some(index) = self
            .reveal_requests
            .iter()
            .position(|request| request.viewport() == target)
        else {
            return false;
        };

        self.reveal_requests.remove(index);
        true
    }

    pub(super) fn prune_removed(
        &mut self,
        removed_nodes: &[super::super::composition::NodeId],
        removed_elements: &[super::Id],
    ) -> bool {
        let before_offsets = self.offsets.len();
        let before_reveals = self.reveal_requests.len();
        self.offsets.retain(|entry| {
            !entry
                .target
                .matches_removed_identity(removed_nodes, removed_elements)
        });
        self.reveal_requests.retain(|request| {
            !request
                .viewport()
                .matches_removed_identity(removed_nodes, removed_elements)
        });
        before_offsets != self.offsets.len() || before_reveals != self.reveal_requests.len()
    }
}

impl Reveal {
    fn viewport(&self) -> &Target {
        match self {
            Self::Viewport(viewport) | Self::ActiveDescendant { viewport } => viewport,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_label_does_not_change_scroll_identity() {
        let mut scroll = Scroll::default();
        let first = Target::scroll("same.scroll", "First Label");
        let second = Target::scroll("same.scroll", "Second Label");

        assert!(scroll.scroll_by(first, ScrollDelta::vertical(42)));
        assert_eq!(scroll.offset(&second), ScrollOffset::new(0, 42));

        assert!(scroll.reveal(second));
        assert!(scroll.should_reveal(&Target::scroll("same.scroll", "Third Label")));
    }
}
