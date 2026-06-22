use thiserror::Error;

pub use window::{PointerCaptureKind, PointerCaptureStatus, Window};

pub mod window;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("window error")]
    Window(#[from] winit::error::OsError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_error_is_window_only() {
        fn assert_window_only(error: Error) {
            match error {
                Error::Window(_) => {}
            }
        }

        let _ = assert_window_only as fn(Error);
    }
}
