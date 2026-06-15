use crate::geometry::point;
use crate::{ui, widget};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pointer {
    position: Option<point::Logical>,
    previous_position: Option<point::Logical>,
    delta: point::Logical,
    primary_down: bool,
    secondary_down: bool,
    middle_down: bool,
    back_down: bool,
    forward_down: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Primary,
    Secondary,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Moved { position: point::Logical },
    Button { button: Button, pressed: bool },
    Left,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Capture {
    target: ui::Path,
    part: widget::Part,
    button: Button,
    origin: point::Logical,
    grab_offset: point::Logical,
}

impl Pointer {
    pub fn new() -> Self {
        Self {
            position: None,
            previous_position: None,
            delta: point::logical(0.0, 0.0),
            primary_down: false,
            secondary_down: false,
            middle_down: false,
            back_down: false,
            forward_down: false,
        }
    }

    pub fn position(&self) -> Option<point::Logical> {
        self.position
    }

    pub fn previous_position(&self) -> Option<point::Logical> {
        self.previous_position
    }

    pub fn delta(&self) -> point::Logical {
        self.delta
    }

    pub fn primary_down(&self) -> bool {
        self.primary_down
    }

    pub fn secondary_down(&self) -> bool {
        self.secondary_down
    }

    pub fn middle_down(&self) -> bool {
        self.middle_down
    }

    pub fn back_down(&self) -> bool {
        self.back_down
    }

    pub fn forward_down(&self) -> bool {
        self.forward_down
    }

    pub fn button_down(&self, button: Button) -> bool {
        match button {
            Button::Primary => self.primary_down,
            Button::Secondary => self.secondary_down,
            Button::Middle => self.middle_down,
            Button::Back => self.back_down,
            Button::Forward => self.forward_down,
            Button::Other(_) => false,
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Moved { position } => self.move_to(position),
            Event::Button { button, pressed } => self.set_button(button, pressed),
            Event::Left => self.clear(),
        }
    }

    fn move_to(&mut self, position: point::Logical) {
        self.previous_position = self.position;
        self.delta = self
            .previous_position
            .map(|previous| {
                point::logical(position.x() - previous.x(), position.y() - previous.y())
            })
            .unwrap_or_else(|| point::logical(0.0, 0.0));
        self.position = Some(position);
    }

    fn set_button(&mut self, button: Button, pressed: bool) {
        match button {
            Button::Primary => self.primary_down = pressed,
            Button::Secondary => self.secondary_down = pressed,
            Button::Middle => self.middle_down = pressed,
            Button::Back => self.back_down = pressed,
            Button::Forward => self.forward_down = pressed,
            Button::Other(_) => {}
        }
    }

    fn clear(&mut self) {
        self.position = None;
        self.previous_position = None;
        self.delta = point::logical(0.0, 0.0);
        self.primary_down = false;
        self.secondary_down = false;
        self.middle_down = false;
        self.back_down = false;
        self.forward_down = false;
    }
}

impl Default for Pointer {
    fn default() -> Self {
        Self::new()
    }
}

impl Capture {
    pub fn new(
        target: ui::Path,
        part: widget::Part,
        button: Button,
        origin: point::Logical,
        grab_offset: point::Logical,
    ) -> Self {
        Self {
            target,
            part,
            button,
            origin,
            grab_offset,
        }
    }

    pub fn target(&self) -> &ui::Path {
        &self.target
    }

    pub fn part(&self) -> widget::Part {
        self.part
    }

    pub fn button(&self) -> Button {
        self.button
    }

    pub fn origin(&self) -> point::Logical {
        self.origin
    }

    pub fn grab_offset(&self) -> point::Logical {
        self.grab_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movement_tracks_current_previous_and_delta() {
        let mut pointer = Pointer::new();

        pointer.handle_event(Event::Moved {
            position: point::logical(10.0, 20.0),
        });

        assert_eq!(pointer.position(), Some(point::logical(10.0, 20.0)));
        assert_eq!(pointer.previous_position(), None);
        assert_eq!(pointer.delta(), point::logical(0.0, 0.0));

        pointer.handle_event(Event::Moved {
            position: point::logical(13.0, 18.0),
        });

        assert_eq!(pointer.position(), Some(point::logical(13.0, 18.0)));
        assert_eq!(
            pointer.previous_position(),
            Some(point::logical(10.0, 20.0))
        );
        assert_eq!(pointer.delta(), point::logical(3.0, -2.0));
    }

    #[test]
    fn button_events_update_button_state() {
        let mut pointer = Pointer::new();

        for button in [
            Button::Primary,
            Button::Secondary,
            Button::Middle,
            Button::Back,
            Button::Forward,
        ] {
            pointer.handle_event(Event::Button {
                button,
                pressed: true,
            });
            assert!(pointer.button_down(button));

            pointer.handle_event(Event::Button {
                button,
                pressed: false,
            });
            assert!(!pointer.button_down(button));
        }
    }

    #[test]
    fn leave_clears_position_delta_and_buttons() {
        let mut pointer = Pointer::new();

        pointer.handle_event(Event::Moved {
            position: point::logical(1.0, 2.0),
        });
        pointer.handle_event(Event::Moved {
            position: point::logical(4.0, 6.0),
        });
        pointer.handle_event(Event::Button {
            button: Button::Primary,
            pressed: true,
        });
        pointer.handle_event(Event::Button {
            button: Button::Secondary,
            pressed: true,
        });

        pointer.handle_event(Event::Left);

        assert_eq!(pointer.position(), None);
        assert_eq!(pointer.previous_position(), None);
        assert_eq!(pointer.delta(), point::logical(0.0, 0.0));
        assert!(!pointer.primary_down());
        assert!(!pointer.secondary_down());
    }

    #[test]
    fn capture_stores_target_part_button_origin_and_grab_offset() {
        let target = ui::Path::from(ui::Id::new("scroll"));
        let capture = Capture::new(
            target.clone(),
            widget::Part::Scroll(widget::scroll::Part::VerticalThumb),
            Button::Primary,
            point::logical(10.0, 20.0),
            point::logical(0.0, 4.0),
        );

        assert_eq!(capture.target(), &target);
        assert_eq!(
            capture.part(),
            widget::Part::Scroll(widget::scroll::Part::VerticalThumb)
        );
        assert_eq!(capture.button(), Button::Primary);
        assert_eq!(capture.origin(), point::logical(10.0, 20.0));
        assert_eq!(capture.grab_offset(), point::logical(0.0, 4.0));
    }
}
