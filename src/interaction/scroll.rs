use std::collections::HashMap;

use super::Target;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Scroll {
    offsets: Vec<ScrollEntry>,
    reveal_requests: Vec<Reveal>,
    next_revision: u64,
    revisions: HashMap<Target, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScrollEntry {
    target: Target,
    position: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Position {
    Admitted(ScrollOffset),
    Pending {
        admitted: ScrollOffset,
        desired: ScrollOffset,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Reveal {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScrollUpdate {
    Relative(ScrollDelta),
    Absolute(ScrollOffset),
    Geometry(ScrollOffset),
}

impl Scroll {
    pub(crate) fn revision(&self, target: &Target) -> u64 {
        self.revisions.get(target).copied().unwrap_or_default()
    }

    pub(crate) fn offset(&self, target: &Target) -> ScrollOffset {
        self.offsets
            .iter()
            .find(|entry| &entry.target == target)
            .map(|entry| entry.position.admitted())
            .unwrap_or_default()
    }

    pub(crate) fn desired_offset(&self, target: &Target) -> ScrollOffset {
        self.offsets
            .iter()
            .find(|entry| &entry.target == target)
            .map(|entry| entry.position.desired())
            .unwrap_or_default()
    }

    pub(crate) fn should_reveal(&self, target: &Target) -> bool {
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

    pub(super) fn request(&mut self, target: Target, update: ScrollUpdate) -> Option<ScrollOffset> {
        let before = self.desired_offset(&target);
        let desired = match update {
            ScrollUpdate::Relative(delta) => before.scrolled_by(delta),
            ScrollUpdate::Absolute(offset) | ScrollUpdate::Geometry(offset) => offset,
        };
        if before == desired {
            return None;
        }

        let index = self.offsets.iter().position(|entry| entry.target == target);
        let admitted = index
            .map(|index| self.offsets[index].position.admitted())
            .unwrap_or_default();
        let position = Position::new(admitted, desired);
        if position.is_zero() {
            if let Some(index) = index {
                self.offsets.remove(index);
            }
        } else if let Some(index) = index {
            self.offsets[index].position = position;
        } else {
            self.offsets.push(ScrollEntry {
                target: target.clone(),
                position,
            });
        }
        self.mark_changed(target);
        Some(desired)
    }

    pub(super) fn admit(&mut self, target: Target, admitted: ScrollOffset) -> Option<ScrollOffset> {
        let before = self.offset(&target);
        if before == admitted {
            return None;
        }

        let index = self.offsets.iter().position(|entry| entry.target == target);
        let desired = index
            .map(|index| self.offsets[index].position.desired())
            .unwrap_or(admitted);
        let position = Position::new(admitted, desired);
        if position.is_zero() {
            if let Some(index) = index {
                self.offsets.remove(index);
            }
        } else if let Some(index) = index {
            self.offsets[index].position = position;
        } else {
            self.offsets.push(ScrollEntry {
                target: target.clone(),
                position,
            });
        }
        self.mark_changed(target);
        Some(admitted)
    }

    pub(super) fn project_desired(&mut self) {
        let mut changed = Vec::new();
        for entry in &mut self.offsets {
            let projected = Position::Admitted(entry.position.desired());
            if entry.position != projected {
                entry.position = projected;
                changed.push(entry.target.clone());
            }
        }
        for target in changed {
            self.mark_changed(target);
        }
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
        removed_nodes: &[super::super::composition::tree::NodeId],
        removed_elements: &[super::Id],
    ) -> bool {
        let before_offsets = self.offsets.len();
        let before_reveals = self.reveal_requests.len();
        let before_revisions = self.revisions.len();
        self.offsets.retain(|entry| {
            !entry
                .target
                .matches_removed_identity(removed_nodes, removed_elements, &[])
        });
        self.reveal_requests.retain(|request| {
            !request
                .viewport()
                .matches_removed_identity(removed_nodes, removed_elements, &[])
        });
        self.revisions.retain(|target, _| {
            !target.matches_removed_identity(removed_nodes, removed_elements, &[])
        });
        before_offsets != self.offsets.len()
            || before_reveals != self.reveal_requests.len()
            || before_revisions != self.revisions.len()
    }

    fn mark_changed(&mut self, target: Target) {
        self.next_revision = self.next_revision.saturating_add(1);
        self.revisions.insert(target, self.next_revision);
    }
}

impl Position {
    fn new(admitted: ScrollOffset, desired: ScrollOffset) -> Self {
        if admitted == desired {
            Self::Admitted(admitted)
        } else {
            Self::Pending { admitted, desired }
        }
    }

    fn admitted(self) -> ScrollOffset {
        match self {
            Self::Admitted(offset)
            | Self::Pending {
                admitted: offset, ..
            } => offset,
        }
    }

    fn desired(self) -> ScrollOffset {
        match self {
            Self::Admitted(offset)
            | Self::Pending {
                desired: offset, ..
            } => offset,
        }
    }

    fn is_zero(self) -> bool {
        self.admitted().is_zero() && self.desired().is_zero()
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_label_does_not_change_scroll_identity() {
        let mut scroll = Scroll::default();
        let first = Target::scroll("same.scroll", "First Label");
        let second = Target::scroll("same.scroll", "Second Label");

        assert_eq!(
            scroll.request(
                first.clone(),
                ScrollUpdate::Relative(ScrollDelta::vertical(42))
            ),
            Some(ScrollOffset::new(0, 42))
        );
        assert_eq!(scroll.offset(&first), ScrollOffset::default());
        assert_eq!(
            scroll.admit(first, ScrollOffset::new(0, 42)),
            Some(ScrollOffset::new(0, 42))
        );
        assert_eq!(scroll.offset(&second), ScrollOffset::new(0, 42));

        assert!(scroll.reveal(second));
        assert!(scroll.should_reveal(&Target::scroll("same.scroll", "Third Label")));
    }

    #[test]
    fn relative_absolute_and_geometry_share_one_request_admission_law() {
        let mut scroll = Scroll::default();
        let target = Target::scroll("shared.scroll", "Shared");

        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Relative(ScrollDelta::new(18, 24)),
            ),
            Some(ScrollOffset::new(18, 24))
        );
        assert_eq!(scroll.offset(&target), ScrollOffset::default());
        assert_eq!(scroll.desired_offset(&target), ScrollOffset::new(18, 24));
        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Absolute(ScrollOffset::new(40, 60)),
            ),
            Some(ScrollOffset::new(40, 60))
        );
        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Geometry(ScrollOffset::new(40, 60)),
            ),
            None
        );
        assert_eq!(scroll.offset(&target), ScrollOffset::default());
        assert_eq!(
            scroll.admit(target.clone(), ScrollOffset::new(40, 60)),
            Some(ScrollOffset::new(40, 60))
        );
        assert_eq!(scroll.offset(&target), ScrollOffset::new(40, 60));
    }

    #[test]
    fn relative_requests_accumulate_against_desired_while_admitted_stays_visible() {
        let mut scroll = Scroll::default();
        let target = Target::scroll("pending.scroll", "Pending");

        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Relative(ScrollDelta::vertical(30)),
            ),
            Some(ScrollOffset::new(0, 30))
        );
        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Relative(ScrollDelta::vertical(40)),
            ),
            Some(ScrollOffset::new(0, 70))
        );

        assert_eq!(scroll.offset(&target), ScrollOffset::default());
        assert_eq!(scroll.desired_offset(&target), ScrollOffset::new(0, 70));
    }

    #[test]
    fn intermediate_admission_preserves_a_farther_desired_offset() {
        let mut scroll = Scroll::default();
        let target = Target::scroll("progress.scroll", "Progress");

        scroll.request(
            target.clone(),
            ScrollUpdate::Absolute(ScrollOffset::new(0, 300)),
        );
        assert_eq!(
            scroll.admit(target.clone(), ScrollOffset::new(0, 120)),
            Some(ScrollOffset::new(0, 120))
        );
        assert_eq!(scroll.offset(&target), ScrollOffset::new(0, 120));
        assert_eq!(scroll.desired_offset(&target), ScrollOffset::new(0, 300));

        assert_eq!(
            scroll.admit(target.clone(), ScrollOffset::new(0, 300)),
            Some(ScrollOffset::new(0, 300))
        );
        assert_eq!(scroll.offset(&target), ScrollOffset::new(0, 300));
        assert_eq!(scroll.desired_offset(&target), ScrollOffset::new(0, 300));
    }

    #[test]
    fn desired_projection_changes_only_the_projection_clone() {
        let mut scroll = Scroll::default();
        let target = Target::scroll("projection.scroll", "Projection");
        scroll.request(
            target.clone(),
            ScrollUpdate::Absolute(ScrollOffset::new(0, 240)),
        );

        let mut projection = scroll.clone();
        projection.project_desired();

        assert_eq!(scroll.offset(&target), ScrollOffset::default());
        assert_eq!(projection.offset(&target), ScrollOffset::new(0, 240));
        assert_eq!(
            projection.desired_offset(&target),
            ScrollOffset::new(0, 240)
        );
    }

    #[test]
    fn source_revisions_advance_only_when_the_named_target_changes() {
        let mut scroll = Scroll::default();
        let first = Target::scroll("revision.first", "First");
        let second = Target::scroll("revision.second", "Second");

        assert_eq!(scroll.revision(&first), 0);
        assert_eq!(scroll.revision(&second), 0);
        scroll.request(
            first.clone(),
            ScrollUpdate::Absolute(ScrollOffset::new(12, 24)),
        );
        let requested = scroll.revision(&first);
        assert!(requested > 0);
        assert_eq!(scroll.revision(&second), 0);

        assert_eq!(
            scroll.request(
                first.clone(),
                ScrollUpdate::Geometry(ScrollOffset::new(12, 24)),
            ),
            None
        );
        assert_eq!(scroll.revision(&first), requested);

        scroll.admit(first.clone(), ScrollOffset::new(12, 24));
        let admitted = scroll.revision(&first);
        assert!(admitted > requested);

        scroll.request(
            first.clone(),
            ScrollUpdate::Relative(ScrollDelta::horizontal(5)),
        );
        let pending = scroll.revision(&first);
        assert!(pending > admitted);
        let mut projection = scroll.clone();
        projection.project_desired();
        assert!(projection.revision(&first) > pending);
        assert_eq!(scroll.revision(&first), pending);
    }
}
