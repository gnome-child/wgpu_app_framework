use std::time::{Duration, Instant};

use winit::keyboard::PhysicalKey;

use crate::{animation, ui, window};

const DEFAULT_INITIAL_DELAY: Duration = Duration::from_millis(500);
const DEFAULT_INTERVAL: Duration = Duration::from_millis(30);
const MIN_INTERVAL: Duration = Duration::from_millis(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyRepeat {
    pub enabled: bool,
    pub initial_delay: Duration,
    pub interval: Duration,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum KeyRepeatPolicy {
    #[default]
    BackendGenerated,
    ClientTimer(KeyRepeat),
    Disabled,
}

impl KeyRepeat {
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            initial_delay: DEFAULT_INITIAL_DELAY,
            interval: DEFAULT_INTERVAL,
        }
    }

    pub(crate) fn normalized(self) -> Self {
        Self {
            interval: if self.interval < MIN_INTERVAL {
                MIN_INTERVAL
            } else {
                self.interval
            },
            ..self
        }
    }
}

impl Default for KeyRepeat {
    fn default() -> Self {
        Self {
            enabled: true,
            initial_delay: DEFAULT_INITIAL_DELAY,
            interval: DEFAULT_INTERVAL,
        }
    }
}

impl KeyRepeatPolicy {
    pub const fn client_timer(settings: KeyRepeat) -> Self {
        Self::ClientTimer(settings)
    }

    pub const fn disabled() -> Self {
        Self::Disabled
    }

    pub(crate) fn timer_settings(self) -> Option<KeyRepeat> {
        match self {
            Self::ClientTimer(settings) => Some(settings),
            Self::BackendGenerated | Self::Disabled => None,
        }
    }

    pub(crate) const fn accepts_backend_repeat(self) -> bool {
        matches!(self, Self::BackendGenerated)
    }

    pub(crate) const fn uses_client_timer(self) -> bool {
        matches!(self, Self::ClientTimer(_))
    }
}

#[derive(Debug)]
pub(crate) struct State {
    settings: KeyRepeat,
    held: Option<HeldKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Pulse {
    pub(crate) window: window::Id,
    pub(crate) key: ui::Key,
    pub(crate) text: Option<String>,
}

#[derive(Debug, Clone)]
struct HeldKey {
    window: window::Id,
    physical_key: PhysicalKey,
    key: ui::Key,
    text: Option<String>,
    next_at: Instant,
}

impl State {
    pub(crate) fn new(settings: KeyRepeat) -> Self {
        Self {
            settings: settings.normalized(),
            held: None,
        }
    }

    pub(crate) fn press(
        &mut self,
        window: window::Id,
        physical_key: PhysicalKey,
        key: ui::Key,
        text: Option<String>,
        repeatable: bool,
        now: Instant,
    ) {
        if !self.settings.enabled {
            self.held = None;
            return;
        }

        if !repeatable {
            if key != ui::Key::Other {
                self.held = None;
            }
            return;
        }

        self.held = Some(HeldKey {
            window,
            physical_key,
            key,
            text,
            next_at: now + self.settings.initial_delay,
        });
    }

    pub(crate) fn release(&mut self, window: window::Id, physical_key: PhysicalKey) {
        if self
            .held
            .as_ref()
            .is_some_and(|held| held.window == window && held.physical_key == physical_key)
        {
            self.held = None;
        }
    }

    pub(crate) fn clear_window(&mut self, window: window::Id) {
        if self.held.as_ref().is_some_and(|held| held.window == window) {
            self.held = None;
        }
    }

    pub(crate) fn schedule(&self) -> animation::Schedule {
        self.held
            .as_ref()
            .map_or(animation::Schedule::Idle, |held| {
                animation::Schedule::At(held.next_at)
            })
    }

    pub(crate) fn due(&mut self, now: Instant) -> Option<Pulse> {
        let held = self.held.as_mut()?;
        if held.next_at > now {
            return None;
        }

        let pulse = Pulse {
            window: held.window,
            key: held.key,
            text: held.text.clone(),
        };
        held.next_at = now + self.settings.interval;

        Some(pulse)
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new(KeyRepeat::default())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use winit::keyboard::KeyCode;

    use super::*;

    fn physical_key() -> PhysicalKey {
        PhysicalKey::Code(KeyCode::KeyA)
    }

    #[test]
    fn default_key_repeat_uses_gtk_like_timing() {
        let settings = KeyRepeat::default();

        assert!(settings.enabled);
        assert_eq!(settings.initial_delay, Duration::from_millis(500));
        assert_eq!(settings.interval, Duration::from_millis(30));
    }

    #[test]
    fn default_policy_uses_backend_generated_repeat() {
        let policy = KeyRepeatPolicy::default();

        assert_eq!(policy, KeyRepeatPolicy::BackendGenerated);
        assert!(policy.accepts_backend_repeat());
        assert!(!policy.uses_client_timer());
        assert_eq!(policy.timer_settings(), None);
    }

    #[test]
    fn client_timer_policy_suppresses_backend_repeat_and_exposes_settings() {
        let settings = KeyRepeat {
            initial_delay: Duration::from_millis(250),
            interval: Duration::from_millis(20),
            ..KeyRepeat::default()
        };
        let policy = KeyRepeatPolicy::client_timer(settings);

        assert!(!policy.accepts_backend_repeat());
        assert!(policy.uses_client_timer());
        assert_eq!(policy.timer_settings(), Some(settings));
    }

    #[test]
    fn disabled_policy_uses_no_repeat_source() {
        let policy = KeyRepeatPolicy::disabled();

        assert!(!policy.accepts_backend_repeat());
        assert!(!policy.uses_client_timer());
        assert_eq!(policy.timer_settings(), None);
    }

    #[test]
    fn normalized_key_repeat_clamps_zero_interval() {
        let settings = KeyRepeat {
            interval: Duration::ZERO,
            ..KeyRepeat::default()
        }
        .normalized();

        assert_eq!(settings.interval, Duration::from_millis(1));
    }

    #[test]
    fn disabled_repeat_never_tracks_held_key() {
        let now = Instant::now();
        let mut state = State::new(KeyRepeat::disabled());

        state.press(
            window::Id::new(1),
            physical_key(),
            ui::Key::Backspace,
            None,
            true,
            now,
        );

        assert_eq!(state.schedule(), animation::Schedule::Idle);
        assert_eq!(state.due(now + Duration::from_secs(1)), None);
    }

    #[test]
    fn repeatable_press_schedules_after_initial_delay() {
        let now = Instant::now();
        let mut state = State::new(KeyRepeat::default());

        state.press(
            window::Id::new(1),
            physical_key(),
            ui::Key::Backspace,
            None,
            true,
            now,
        );

        assert_eq!(
            state.schedule(),
            animation::Schedule::At(now + Duration::from_millis(500))
        );
        assert_eq!(state.due(now + Duration::from_millis(499)), None);
    }

    #[test]
    fn due_repeat_emits_one_pulse_and_advances_by_interval() {
        let now = Instant::now();
        let mut state = State::new(KeyRepeat::default());
        let window = window::Id::new(1);

        state.press(
            window,
            physical_key(),
            ui::Key::Character('a'),
            Some("a".to_owned()),
            true,
            now,
        );

        let due = now + Duration::from_millis(500);
        assert_eq!(
            state.due(due),
            Some(Pulse {
                window,
                key: ui::Key::Character('a'),
                text: Some("a".to_owned()),
            })
        );
        assert_eq!(
            state.schedule(),
            animation::Schedule::At(due + Duration::from_millis(30))
        );
    }

    #[test]
    fn stalled_wake_emits_only_one_pulse() {
        let now = Instant::now();
        let mut state = State::new(KeyRepeat::default());

        state.press(
            window::Id::new(1),
            physical_key(),
            ui::Key::Backspace,
            None,
            true,
            now,
        );

        let late = now + Duration::from_secs(10);
        assert!(state.due(late).is_some());
        assert_eq!(state.due(late), None);
    }

    #[test]
    fn matching_release_clears_held_key() {
        let now = Instant::now();
        let mut state = State::new(KeyRepeat::default());
        let window = window::Id::new(1);
        let key = physical_key();

        state.press(window, key, ui::Key::Backspace, None, true, now);
        state.release(window, key);

        assert_eq!(state.schedule(), animation::Schedule::Idle);
    }

    #[test]
    fn unmatched_release_preserves_held_key() {
        let now = Instant::now();
        let mut state = State::new(KeyRepeat::default());
        let window = window::Id::new(1);

        state.press(window, physical_key(), ui::Key::Backspace, None, true, now);
        state.release(window, PhysicalKey::Code(KeyCode::KeyB));

        assert_ne!(state.schedule(), animation::Schedule::Idle);
    }

    #[test]
    fn focus_loss_or_window_close_clears_held_key() {
        let now = Instant::now();
        let mut state = State::new(KeyRepeat::default());
        let window = window::Id::new(1);

        state.press(window, physical_key(), ui::Key::Backspace, None, true, now);
        state.clear_window(window);

        assert_eq!(state.schedule(), animation::Schedule::Idle);
    }
}
