use crate::{ui, window};

#[derive(Debug, Clone, PartialEq)]
pub enum Event<T> {
    Ui {
        window: window::Id,
        event: ui::Event,
    },
    App(T),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::point;

    #[test]
    fn ui_event_wraps_window_and_ui_payload() {
        let window = window::Id::new(1);
        let event = Event::<()>::Ui {
            window,
            event: ui::Event::PointerMoved {
                position: point::logical(1.0, 2.0),
                target: None,
            },
        };

        assert_eq!(
            event,
            Event::Ui {
                window,
                event: ui::Event::PointerMoved {
                    position: point::logical(1.0, 2.0),
                    target: None,
                },
            }
        );
    }

    #[test]
    fn keyboard_ui_event_wraps_key_payload() {
        let window = window::Id::new(1);
        let modifiers = ui::Modifiers::new(true, false, false, false);
        let target = Some(ui::Path::from(ui::Id::new("button")));
        let event = Event::<()>::Ui {
            window,
            event: ui::Event::KeyDown {
                key: ui::Key::Tab,
                modifiers,
                target: target.clone(),
                repeat: false,
            },
        };

        assert_eq!(
            event,
            Event::Ui {
                window,
                event: ui::Event::KeyDown {
                    key: ui::Key::Tab,
                    modifiers,
                    target,
                    repeat: false,
                },
            }
        );
        assert!(modifiers.shift());
        assert!(!modifiers.control());
    }
}
