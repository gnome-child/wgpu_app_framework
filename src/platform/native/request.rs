use super::Native;
use crate::session;

impl Native {
    pub fn requests(&self) -> &[session::Request] {
        &self.requests
    }

    pub fn take_requests(&mut self) -> Vec<session::Request> {
        std::mem::take(&mut self.requests)
    }

    pub fn clear_requests(&mut self) {
        self.requests.clear();
    }

    pub(in crate::platform::native) fn request_once(&mut self, request: session::Request) {
        if !self.requests.contains(&request) {
            log::debug!("queued native request: {request:?}");
            self.requests.push(request);
        } else {
            log::debug!("native request already queued: {request:?}");
        }
    }

    #[cfg(test)]
    pub fn track_request_for_test(&mut self, request: session::Request) {
        self.requests.push(request);
    }
}
