use crate::notification::Notification;

pub struct OpenDialogCanceled;

impl Notification for OpenDialogCanceled {
    type Payload = ();

    const NAME: &'static str = "document.open_dialog_canceled";
}

pub struct SaveDialogCanceled;

impl Notification for SaveDialogCanceled {
    type Payload = ();

    const NAME: &'static str = "document.save_dialog_canceled";
}
