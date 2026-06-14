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
}
