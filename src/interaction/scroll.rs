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
    remainder: ScrollRemainder,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct ScrollRemainder {
    x: f64,
    y: f64,
    compensation_x: f64,
    compensation_y: f64,
}

impl Eq for ScrollRemainder {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Position {
    ResidentAccepted(ScrollOffset),
    Pending {
        resident_accepted: ScrollOffset,
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

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ScrollDelta {
    x: f64,
    y: f64,
}

impl Eq for ScrollDelta {}

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

    pub(crate) fn resident_offset(&self, target: &Target) -> ScrollOffset {
        self.offsets
            .iter()
            .find(|entry| &entry.target == target)
            .map(|entry| entry.position.resident_accepted())
            .unwrap_or_default()
    }

    #[cfg(test)]
    pub(crate) fn offset(&self, target: &Target) -> ScrollOffset {
        self.resident_offset(target)
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
        let index = self.offsets.iter().position(|entry| entry.target == target);
        let before = index
            .map(|index| self.offsets[index].position.desired())
            .unwrap_or_default();
        let resident_accepted = index
            .map(|index| self.offsets[index].position.resident_accepted())
            .unwrap_or_default();
        let previous_remainder = index
            .map(|index| self.offsets[index].remainder)
            .unwrap_or_default();
        let (desired, remainder) = match update {
            ScrollUpdate::Relative(delta) => {
                let (visual, remainder) = previous_remainder.accumulate(delta);
                (before.scrolled_by(visual), remainder)
            }
            ScrollUpdate::Absolute(offset) | ScrollUpdate::Geometry(offset) => {
                (offset, ScrollRemainder::default())
            }
        };

        let position = Position::new(resident_accepted, desired);
        if position.is_zero() && remainder.is_zero() {
            if let Some(index) = index {
                self.offsets.remove(index);
            }
        } else if let Some(index) = index {
            self.offsets[index].position = position;
            self.offsets[index].remainder = remainder;
        } else {
            self.offsets.push(ScrollEntry {
                target: target.clone(),
                position,
                remainder,
            });
        }
        if before == desired {
            return None;
        }
        self.mark_changed(target);
        Some(desired)
    }

    pub(super) fn accept_resident(
        &mut self,
        target: Target,
        resident_accepted: ScrollOffset,
    ) -> Option<ScrollOffset> {
        let before = self.resident_offset(&target);
        if before == resident_accepted {
            return None;
        }

        let index = self.offsets.iter().position(|entry| entry.target == target);
        let desired = index
            .map(|index| self.offsets[index].position.desired())
            .unwrap_or(resident_accepted);
        let remainder = index
            .map(|index| self.offsets[index].remainder)
            .unwrap_or_default();
        let position = Position::new(resident_accepted, desired);
        if position.is_zero() && remainder.is_zero() {
            if let Some(index) = index {
                self.offsets.remove(index);
            }
        } else if let Some(index) = index {
            self.offsets[index].position = position;
        } else {
            self.offsets.push(ScrollEntry {
                target: target.clone(),
                position,
                remainder,
            });
        }
        self.mark_changed(target);
        Some(resident_accepted)
    }

    pub(super) fn project_desired(&mut self) {
        let mut changed = Vec::new();
        for entry in &mut self.offsets {
            let projected = Position::ResidentAccepted(entry.position.desired());
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
    fn new(resident_accepted: ScrollOffset, desired: ScrollOffset) -> Self {
        if resident_accepted == desired {
            Self::ResidentAccepted(resident_accepted)
        } else {
            Self::Pending {
                resident_accepted,
                desired,
            }
        }
    }

    fn resident_accepted(self) -> ScrollOffset {
        match self {
            Self::ResidentAccepted(offset)
            | Self::Pending {
                resident_accepted: offset,
                ..
            } => offset,
        }
    }

    fn desired(self) -> ScrollOffset {
        match self {
            Self::ResidentAccepted(offset)
            | Self::Pending {
                desired: offset, ..
            } => offset,
        }
    }

    fn is_zero(self) -> bool {
        self.resident_accepted().is_zero() && self.desired().is_zero()
    }
}

impl ScrollRemainder {
    // Whole logical pixels remain exact. Only fractional components enter this
    // compensated accumulator; they cross a visual pixel by truncation toward
    // zero, with an 8-ULP snap solely at a computed integral boundary.
    fn accumulate(self, delta: ScrollDelta) -> (VisualScrollDelta, Self) {
        let (integral_x, fractional_x) = split_scroll_component(delta.x);
        let (integral_y, fractional_y) = split_scroll_component(delta.y);
        let (fractional_visual_x, remainder_x, compensation_x) =
            quantize_scroll_axis(self.x, self.compensation_x, fractional_x);
        let (fractional_visual_y, remainder_y, compensation_y) =
            quantize_scroll_axis(self.y, self.compensation_y, fractional_y);
        (
            VisualScrollDelta {
                x: integral_x.saturating_add(fractional_visual_x),
                y: integral_y.saturating_add(fractional_visual_y),
            },
            Self {
                x: remainder_x,
                y: remainder_y,
                compensation_x,
                compensation_y,
            },
        )
    }

    fn is_zero(self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.compensation_x == 0.0 && self.compensation_y == 0.0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct VisualScrollDelta {
    x: i32,
    y: i32,
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

    fn scrolled_by(self, delta: VisualScrollDelta) -> Self {
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
        Self::from_logical_pixels(f64::from(x), f64::from(y))
    }

    pub fn horizontal(x: i32) -> Self {
        Self::new(x, 0)
    }

    pub fn vertical(y: i32) -> Self {
        Self::new(0, y)
    }

    pub(crate) fn from_logical_pixels(x: f64, y: f64) -> Self {
        Self {
            x: normalized_scroll_component(x),
            y: normalized_scroll_component(y),
        }
    }

    pub(crate) fn from_physical_pixels(x: f64, y: f64, scale_factor: f64) -> Self {
        let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        Self::from_logical_pixels(x / scale_factor, y / scale_factor)
    }

    pub fn x(self) -> f64 {
        self.x
    }

    pub fn y(self) -> f64 {
        self.y
    }
}

fn normalized_scroll_component(value: f64) -> f64 {
    if !value.is_finite() || value == 0.0 {
        return 0.0;
    }
    value.clamp(f64::from(i32::MIN), f64::from(i32::MAX))
}

fn split_scroll_component(value: f64) -> (i32, f64) {
    let integral = value.trunc() as i32;
    (integral, normalized_zero(value - f64::from(integral)))
}

fn quantize_scroll_axis(remainder: f64, compensation: f64, delta: f64) -> (i32, f64, f64) {
    let adjusted = delta - compensation;
    let total = remainder + adjusted;
    let compensation = (total - remainder) - adjusted;
    let nearest = total.round();
    let boundary_tolerance = f64::EPSILON * 8.0 * total.abs().max(1.0);
    let at_visual_boundary = (total - nearest).abs() <= boundary_tolerance;
    let total = if at_visual_boundary { nearest } else { total };
    let compensation = if at_visual_boundary {
        0.0
    } else {
        compensation
    };
    let visual = total
        .trunc()
        .clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32;
    let remainder = total - f64::from(visual);
    (
        visual,
        normalized_zero(remainder),
        normalized_zero(compensation),
    )
}

fn normalized_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
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
            scroll.accept_resident(first, ScrollOffset::new(0, 42)),
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
            scroll.accept_resident(target.clone(), ScrollOffset::new(40, 60)),
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
            scroll.accept_resident(target.clone(), ScrollOffset::new(0, 120)),
            Some(ScrollOffset::new(0, 120))
        );
        assert_eq!(scroll.offset(&target), ScrollOffset::new(0, 120));
        assert_eq!(scroll.desired_offset(&target), ScrollOffset::new(0, 300));

        assert_eq!(
            scroll.accept_resident(target.clone(), ScrollOffset::new(0, 300)),
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

        scroll.accept_resident(first.clone(), ScrollOffset::new(12, 24));
        let accepted = scroll.revision(&first);
        assert!(accepted > requested);

        scroll.request(
            first.clone(),
            ScrollUpdate::Relative(ScrollDelta::horizontal(5)),
        );
        let pending = scroll.revision(&first);
        assert!(pending > accepted);
        let mut projection = scroll.clone();
        projection.project_desired();
        assert!(projection.revision(&first) > pending);
        assert_eq!(scroll.revision(&first), pending);
    }

    fn apply_physical_trace(scale: f64, physical_y: &[f64]) -> (Scroll, Target, usize) {
        let mut scroll = Scroll::default();
        let target = Target::scroll("precision.trace", "Precision Trace");
        let mut visual_updates = 0;
        for physical_y in physical_y {
            visual_updates += usize::from(
                scroll
                    .request(
                        target.clone(),
                        ScrollUpdate::Relative(ScrollDelta::from_physical_pixels(
                            0.0,
                            *physical_y,
                            scale,
                        )),
                    )
                    .is_some(),
            );
        }
        (scroll, target, visual_updates)
    }

    fn remainder(scroll: &Scroll, target: &Target) -> ScrollRemainder {
        scroll
            .offsets
            .iter()
            .find(|entry| &entry.target == target)
            .map(|entry| entry.remainder)
            .unwrap_or_default()
    }

    fn require_sum_preserved(scale: f64, physical_y: &[f64]) -> (Scroll, Target, usize) {
        let (scroll, target, visual_updates) = apply_physical_trace(scale, physical_y);
        let logical_total = physical_y.iter().sum::<f64>() / scale;
        let desired = scroll.desired_offset(&target);
        let remainder = remainder(&scroll, &target);
        assert_eq!(desired.y(), logical_total.trunc() as i32);
        assert!(
            (f64::from(desired.y()) + remainder.y - logical_total).abs() < 1.0e-12,
            "scale={scale} desired={desired:?} remainder={remainder:?} logical_total={logical_total}"
        );
        (scroll, target, visual_updates)
    }

    fn require_tiny_trace(scale: f64) {
        let (_, _, visual_updates) = require_sum_preserved(scale, &[0.4; 5]);
        assert!(visual_updates > 0 && visual_updates < 5);
    }

    fn require_reversal_trace(scale: f64) {
        let physical = [1.2, 1.2, -0.6, -0.6, -1.2];
        let mut scroll = Scroll::default();
        let target = Target::scroll("precision.reversal", "Precision Reversal");
        let mut maximum = 0;
        for physical_y in physical {
            scroll.request(
                target.clone(),
                ScrollUpdate::Relative(ScrollDelta::from_physical_pixels(0.0, physical_y, scale)),
            );
            maximum = maximum.max(scroll.desired_offset(&target).y());
        }
        let desired = scroll.desired_offset(&target);
        let remainder = remainder(&scroll, &target);
        assert!(maximum > 0, "scale={scale} reversal never moved visually");
        assert_eq!(desired, ScrollOffset::default());
        assert!(
            remainder.y.abs() < 1.0e-12,
            "scale={scale} reversal drifted by {:?}",
            remainder.y
        );
    }

    fn require_burst_trace(scale: f64) {
        let physical = [0.3, 0.3, 0.3, 4.1, 0.2, 0.2, 0.2, 0.4];
        let (scroll, target, visual_updates) = require_sum_preserved(scale, &physical);
        assert!(visual_updates > 0 && visual_updates < physical.len());
        assert_eq!(scroll.revision(&target), visual_updates as u64);
    }

    #[test]
    fn input_precision_case_tiny_scale_100() {
        require_tiny_trace(1.0);
    }

    #[test]
    fn input_precision_case_tiny_scale_125() {
        require_tiny_trace(1.25);
    }

    #[test]
    fn input_precision_case_tiny_scale_150() {
        require_tiny_trace(1.5);
    }

    #[test]
    fn input_precision_case_tiny_scale_175() {
        require_tiny_trace(1.75);
    }

    #[test]
    fn input_precision_case_tiny_scale_200() {
        require_tiny_trace(2.0);
    }

    #[test]
    fn input_precision_case_reversal_scale_100() {
        require_reversal_trace(1.0);
    }

    #[test]
    fn input_precision_case_reversal_scale_125() {
        require_reversal_trace(1.25);
    }

    #[test]
    fn input_precision_case_reversal_scale_150() {
        require_reversal_trace(1.5);
    }

    #[test]
    fn input_precision_case_reversal_scale_175() {
        require_reversal_trace(1.75);
    }

    #[test]
    fn input_precision_case_reversal_scale_200() {
        require_reversal_trace(2.0);
    }

    #[test]
    fn input_precision_case_burst_coalescing_scale_100() {
        require_burst_trace(1.0);
    }

    #[test]
    fn input_precision_case_burst_coalescing_scale_125() {
        require_burst_trace(1.25);
    }

    #[test]
    fn input_precision_case_burst_coalescing_scale_150() {
        require_burst_trace(1.5);
    }

    #[test]
    fn input_precision_case_burst_coalescing_scale_175() {
        require_burst_trace(1.75);
    }

    #[test]
    fn input_precision_case_burst_coalescing_scale_200() {
        require_burst_trace(2.0);
    }

    #[test]
    fn input_precision_case_thumb_absolute_resets_fractional_remainder() {
        let mut scroll = Scroll::default();
        let target = Target::scroll("precision.thumb", "Precision Thumb");
        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Relative(ScrollDelta::from_logical_pixels(0.0, 0.75)),
            ),
            None
        );
        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Absolute(ScrollOffset::new(0, 40)),
            ),
            Some(ScrollOffset::new(0, 40))
        );
        assert_eq!(remainder(&scroll, &target), ScrollRemainder::default());
        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Relative(ScrollDelta::from_logical_pixels(0.0, 0.5)),
            ),
            None
        );
        assert_eq!(scroll.desired_offset(&target), ScrollOffset::new(0, 40));
    }

    #[test]
    fn input_precision_case_keyboard_integral_delta_preserves_fractional_remainder() {
        let mut scroll = Scroll::default();
        let target = Target::scroll("precision.keyboard", "Precision Keyboard");
        scroll.request(
            target.clone(),
            ScrollUpdate::Relative(ScrollDelta::from_logical_pixels(0.0, -0.4)),
        );
        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Relative(ScrollDelta::vertical(28)),
            ),
            Some(ScrollOffset::new(0, 28))
        );
        assert!((remainder(&scroll, &target).y + 0.4).abs() < 1.0e-12);
        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Relative(ScrollDelta::from_logical_pixels(0.0, 1.4)),
            ),
            Some(ScrollOffset::new(0, 29))
        );
    }

    #[test]
    fn input_precision_case_reveal_geometry_resets_fractional_remainder() {
        let mut scroll = Scroll::default();
        let target = Target::scroll("precision.reveal", "Precision Reveal");
        scroll.request(
            target.clone(),
            ScrollUpdate::Relative(ScrollDelta::from_logical_pixels(0.0, 0.75)),
        );
        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Geometry(ScrollOffset::new(0, 72)),
            ),
            Some(ScrollOffset::new(0, 72))
        );
        assert_eq!(remainder(&scroll, &target), ScrollRemainder::default());
    }

    #[test]
    fn input_precision_case_programmatic_absolute_is_exact_after_reverse_fraction() {
        let mut scroll = Scroll::default();
        let target = Target::scroll("precision.programmatic", "Precision Programmatic");
        scroll.request(
            target.clone(),
            ScrollUpdate::Relative(ScrollDelta::from_logical_pixels(0.0, -0.75)),
        );
        assert_eq!(
            scroll.request(
                target.clone(),
                ScrollUpdate::Absolute(ScrollOffset::new(36, 84)),
            ),
            Some(ScrollOffset::new(36, 84))
        );
        assert_eq!(scroll.desired_offset(&target), ScrollOffset::new(36, 84));
        assert_eq!(remainder(&scroll, &target), ScrollRemainder::default());
    }
}
