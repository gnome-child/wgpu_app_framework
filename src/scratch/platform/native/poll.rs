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

    pub(in crate::scratch::platform::native) fn schedule_poll_request(&mut self) {
        self.poll_requested = true;
    }
}
