use super::window;
use std::time::Duration;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cursor {
    #[default]
    Default,
    Text,
    ResizeHorizontal,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    #[default]
    Primary,
    Secondary,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Update {
    window: window::Id,
    cursor: Cursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MultiClickSettings {
    interval: Duration,
    distance_x: i32,
    distance_y: i32,
}

impl MultiClickSettings {
    pub(crate) fn system() -> Self {
        system_multi_click_settings()
    }

    pub(crate) fn accepts(self, elapsed: Duration, dx: i32, dy: i32) -> bool {
        elapsed <= self.interval && dx <= self.distance_x.max(1) && dy <= self.distance_y.max(1)
    }
}

#[cfg(target_os = "windows")]
fn system_multi_click_settings() -> MultiClickSettings {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetDoubleClickTime;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXDOUBLECLK, SM_CYDOUBLECLK,
    };

    MultiClickSettings {
        interval: Duration::from_millis(unsafe { GetDoubleClickTime() } as u64),
        distance_x: unsafe { GetSystemMetrics(SM_CXDOUBLECLK) }.max(1),
        distance_y: unsafe { GetSystemMetrics(SM_CYDOUBLECLK) }.max(1),
    }
}

#[cfg(not(target_os = "windows"))]
fn system_multi_click_settings() -> MultiClickSettings {
    MultiClickSettings {
        interval: Duration::from_millis(500),
        distance_x: 4,
        distance_y: 4,
    }
}

impl Update {
    pub(crate) fn new(window: window::Id, cursor: Cursor) -> Self {
        Self { window, cursor }
    }

    pub fn window(self) -> window::Id {
        self.window
    }

    pub fn cursor(self) -> Cursor {
        self.cursor
    }
}
