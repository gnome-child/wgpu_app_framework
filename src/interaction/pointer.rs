use super::Target;
use crate::{geometry::Point, pointer};
use std::time::Instant;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Pointer {
    pub(super) location: Option<Location>,
    pub(super) modifiers: crate::keyboard::Modifiers,
    pub(super) hovered: Option<Target>,
    pub(super) press: Option<Press>,
    hover_tip: HoverTip,
    last_click: Option<Click>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Location {
    point: Point,
    surface: crate::popup::Surface,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum HoverTip {
    #[default]
    Idle,
    Waiting {
        started_at: Instant,
    },
    Visible {
        anchor: Point,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Capture {
    target: Target,
    cursor: pointer::Cursor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Press {
    Captured {
        capture: Capture,
        intent: PressIntent,
    },
    Uncaptured {
        target: Target,
        intent: PressIntent,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PressIntent {
    Activate,
    Manipulate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Click {
    target: Target,
    point: Point,
    at: Instant,
    count: ClickCount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClickCount {
    Single,
    Double,
    Triple,
}

impl Pointer {
    pub(crate) fn classify_click(
        &mut self,
        target: &Target,
        point: Point,
        at: Instant,
        settings: pointer::MultiClickSettings,
    ) -> ClickCount {
        let count = self
            .last_click
            .as_ref()
            .filter(|last| {
                last.target == *target
                    && settings.accepts(
                        at.saturating_duration_since(last.at),
                        point.x().abs_diff(last.point.x()) as i32,
                        point.y().abs_diff(last.point.y()) as i32,
                    )
            })
            .map_or(ClickCount::Single, |last| match last.count {
                ClickCount::Single => ClickCount::Double,
                ClickCount::Double => ClickCount::Triple,
                ClickCount::Triple => ClickCount::Single,
            });
        self.last_click = Some(Click {
            target: target.clone(),
            point,
            at,
            count,
        });
        count
    }

    pub(crate) fn cancel_click_sequence(&mut self) -> bool {
        self.last_click.take().is_some()
    }

    pub(crate) fn hovered(&self) -> Option<&Target> {
        self.hovered.as_ref()
    }

    pub(crate) fn hover_tip_visible(&self) -> bool {
        matches!(self.hover_tip, HoverTip::Visible { .. })
    }

    pub(crate) fn hover_tip_deadline(&self, delay: std::time::Duration) -> Option<Instant> {
        match self.hover_tip {
            HoverTip::Waiting { started_at } => Some(started_at + delay),
            HoverTip::Idle | HoverTip::Visible { .. } => None,
        }
    }

    pub(super) fn update_projected_hover(
        &mut self,
        target: Option<Target>,
        tip_eligible: bool,
        at: Instant,
    ) -> bool {
        let target_changed = self.hovered != target;
        if target_changed {
            self.hovered = target;
            self.hover_tip = HoverTip::default();
        }
        let tip_eligible = tip_eligible && self.hovered.is_some();
        let tip_active = self.hover_tip != HoverTip::Idle;
        let eligibility_changed = tip_active != tip_eligible;
        if eligibility_changed {
            self.hover_tip = if tip_eligible {
                HoverTip::Waiting { started_at: at }
            } else {
                HoverTip::default()
            };
        }
        target_changed || eligibility_changed
    }

    pub(super) fn promote_hover_tip(&mut self, now: Instant, delay: std::time::Duration) -> bool {
        if self.hovered.is_none() {
            return false;
        }
        let HoverTip::Waiting { started_at } = self.hover_tip else {
            return false;
        };
        if now < started_at + delay {
            return false;
        }
        let Some(anchor) = self.location.map(Location::point) else {
            return false;
        };
        self.hover_tip = HoverTip::Visible { anchor };
        true
    }

    pub(super) fn dismiss_hover_tip(&mut self) -> bool {
        let changed = self.hover_tip != HoverTip::Idle;
        self.hover_tip = HoverTip::default();
        changed
    }

    pub(crate) fn hover_tip_anchor(&self) -> Option<Point> {
        match self.hover_tip {
            HoverTip::Visible { anchor } => Some(anchor),
            HoverTip::Idle | HoverTip::Waiting { .. } => None,
        }
    }

    pub(crate) fn location(&self) -> Option<Location> {
        self.location
    }

    pub(crate) fn modifiers(&self) -> crate::keyboard::Modifiers {
        self.modifiers
    }

    pub(crate) fn pressed(&self) -> Option<&Target> {
        self.press.as_ref().map(Press::target)
    }

    pub(crate) fn capture(&self) -> Option<&Capture> {
        self.press.as_ref().and_then(Press::capture)
    }

    pub(crate) fn activation_target(&self) -> Option<&Target> {
        self.press
            .as_ref()
            .filter(|press| press.intent() == PressIntent::Activate)
            .map(Press::target)
    }
}

impl Location {
    pub(super) fn new(point: Point, surface: crate::popup::Surface) -> Self {
        Self { point, surface }
    }

    pub(crate) fn point(self) -> Point {
        self.point
    }

    pub(crate) fn surface(self) -> crate::popup::Surface {
        self.surface
    }
}

impl Press {
    pub(super) fn new(target: Target, intent: PressIntent, cursor: pointer::Cursor) -> Self {
        if target.captures() {
            Self::Captured {
                capture: Capture::new(target, cursor),
                intent,
            }
        } else {
            Self::Uncaptured { target, intent }
        }
    }

    pub(super) fn target(&self) -> &Target {
        match self {
            Self::Captured { capture, .. } => capture.target(),
            Self::Uncaptured { target, .. } => target,
        }
    }

    pub(super) fn capture(&self) -> Option<&Capture> {
        match self {
            Self::Captured { capture, .. } => Some(capture),
            Self::Uncaptured { .. } => None,
        }
    }

    pub(super) fn intent(&self) -> PressIntent {
        match self {
            Self::Captured { intent, .. } | Self::Uncaptured { intent, .. } => *intent,
        }
    }

    pub(super) fn set_intent(&mut self, next: PressIntent) {
        match self {
            Self::Captured { intent, .. } | Self::Uncaptured { intent, .. } => *intent = next,
        }
    }
}

impl Capture {
    pub(super) fn new(target: Target, cursor: pointer::Cursor) -> Self {
        Self { target, cursor }
    }

    pub(crate) fn target(&self) -> &Target {
        &self.target
    }

    pub(crate) fn cursor(&self) -> pointer::Cursor {
        self.cursor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn location_keeps_point_and_surface_under_one_presence_lifetime() {
        let point = Point::new(12, 34);
        let surface = crate::popup::Surface::Native(crate::interaction::Id::new("test.popup"));
        let location = Location::new(point, surface);

        assert_eq!(location.point(), point);
        assert_eq!(location.surface(), surface);
        assert_eq!(Pointer::default().location(), None);
    }

    #[test]
    fn visible_hover_tip_keeps_its_reveal_point_when_pointer_moves_within_target() {
        let mut pointer = Pointer::default();
        let target = Target::label("hover.target", "Hover target");
        let entered_at = Instant::now();
        let reveal_point = Point::new(40, 30);

        pointer.location = Some(Location::new(reveal_point, crate::popup::Surface::Parent));
        assert!(pointer.update_projected_hover(Some(target), true, entered_at));
        assert_eq!(pointer.hover_tip_deadline(Duration::ZERO), Some(entered_at));
        assert!(pointer.promote_hover_tip(entered_at, std::time::Duration::ZERO));
        assert_eq!(pointer.hover_tip_deadline(Duration::ZERO), None);
        assert_eq!(pointer.hover_tip_anchor(), Some(reveal_point));

        pointer.location = Some(Location::new(
            Point::new(80, 60),
            crate::popup::Surface::Parent,
        ));
        assert_eq!(
            pointer.hover_tip_anchor(),
            Some(reveal_point),
            "pointer attachment is a reveal snapshot, not a live-follow geometry clock"
        );
    }

    #[test]
    fn click_chain_consumes_injected_thresholds_and_target_identity() {
        let mut pointer = Pointer::default();
        let target = Target::label("click.target", "Click target");
        let other = Target::label("click.other", "Other target");
        let started = Instant::now();
        let settings = pointer::MultiClickSettings::new(Duration::from_millis(200), 4, 4);

        assert_eq!(
            pointer.classify_click(&target, Point::new(10, 10), started, settings),
            ClickCount::Single
        );
        assert_eq!(
            pointer.classify_click(
                &target,
                Point::new(14, 14),
                started + Duration::from_millis(200),
                settings,
            ),
            ClickCount::Double
        );
        assert_eq!(
            pointer.classify_click(
                &other,
                Point::new(14, 14),
                started + Duration::from_millis(201),
                settings,
            ),
            ClickCount::Single
        );
    }
}
