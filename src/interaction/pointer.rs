use super::Target;
use crate::geometry::Point;
use std::time::Instant;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Pointer {
    pub(super) position: Option<Point>,
    pub(super) surface: crate::popup::Surface,
    pub(super) hovered: Option<Target>,
    pub(super) pressed: Option<Target>,
    pub(super) capture: Option<Capture>,
    pub(super) press_intent: Option<PressIntent>,
    hover_tip: HoverTip,
    last_click: Option<Click>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct HoverTip {
    started_at: Option<Instant>,
    visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Capture {
    target: Target,
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
    ) -> ClickCount {
        let settings = crate::pointer::MultiClickSettings::system();
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
        self.hover_tip.visible
    }

    pub(crate) fn hover_tip_deadline(&self, delay: std::time::Duration) -> Option<Instant> {
        (!self.hover_tip.visible)
            .then_some(self.hover_tip.started_at)
            .flatten()
            .map(|started_at| started_at + delay)
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
        let tip_active = self.hover_tip.visible || self.hover_tip.started_at.is_some();
        let eligibility_changed = tip_active != tip_eligible;
        if eligibility_changed {
            self.hover_tip = if tip_eligible {
                HoverTip {
                    started_at: Some(at),
                    visible: false,
                }
            } else {
                HoverTip::default()
            };
        }
        target_changed || eligibility_changed
    }

    pub(super) fn promote_hover_tip(&mut self, now: Instant, delay: std::time::Duration) -> bool {
        if self.hover_tip.visible || self.hovered.is_none() {
            return false;
        }
        let Some(started_at) = self.hover_tip.started_at else {
            return false;
        };
        if now < started_at + delay {
            return false;
        }
        self.hover_tip.visible = true;
        true
    }

    pub(super) fn dismiss_hover_tip(&mut self) -> bool {
        let changed = self.hover_tip.visible || self.hover_tip.started_at.is_some();
        self.hover_tip = HoverTip::default();
        changed
    }

    pub(crate) fn position(&self) -> Option<Point> {
        self.position
    }

    pub(crate) fn surface(&self) -> crate::popup::Surface {
        self.surface
    }

    pub(crate) fn pressed(&self) -> Option<&Target> {
        self.pressed.as_ref()
    }

    pub(crate) fn capture(&self) -> Option<&Capture> {
        self.capture.as_ref()
    }

    pub(crate) fn activation_target(&self) -> Option<&Target> {
        (self.press_intent == Some(PressIntent::Activate))
            .then_some(self.pressed.as_ref())
            .flatten()
    }
}

impl Capture {
    pub(super) fn new(target: Target) -> Self {
        Self { target }
    }

    pub(crate) fn target(&self) -> &Target {
        &self.target
    }
}
