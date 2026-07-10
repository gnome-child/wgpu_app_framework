use crate::notification;

use super::Id;

/// The past-tense fact that a window has left the session.
pub struct Departed;

impl notification::Notification for Departed {
    type Payload = Id;

    const NAME: &'static str = "window.departed";
}
