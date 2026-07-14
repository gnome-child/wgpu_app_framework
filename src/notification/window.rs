use crate::window as app_window;

impl super::Notification for app_window::Departed {
    type Payload = app_window::Id;

    const NAME: &'static str = "window.departed";
}
