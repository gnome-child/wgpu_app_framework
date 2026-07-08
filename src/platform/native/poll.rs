use super::Native;

impl Native {
    pub fn poll_requested(&self) -> bool {
        self.poll_requested
    }

    pub fn take_poll_requested(&mut self) -> bool {
        let requested = self.poll_requested;
        self.poll_requested = false;
        requested
    }

    pub(in crate::platform::native) fn schedule_poll_request(&mut self) {
        log::debug!("queued native poll request");
        self.poll_requested = true;
    }
}
