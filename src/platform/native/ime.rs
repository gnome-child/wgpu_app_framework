use crate::{ime as app_ime, window as app_window};

use super::{ImeHost, Native, PopupKey};

impl Native {
    pub(in crate::platform::native) fn apply_ime_update(&mut self, update: app_ime::Update) {
        let parent = update.parent();
        let previous = self.ime_targets.get(&parent).copied();
        let next = update.target();
        if previous == next {
            return;
        }

        let previous_host = previous.map(|target| ime_host(parent, target));
        let next_host = next.map(|target| ime_host(parent, target));
        if previous_host != next_host
            && let Some(ImeHost::Popup(key)) = previous_host
            && let Some(popup) = self.popups.get(&key)
        {
            popup.window.set_ime_allowed(false);
        }

        match next {
            Some(target) if self.apply_ime_target(parent, target) => {
                self.ime_targets.insert(parent, target);
            }
            Some(target) => {
                self.ime_targets.remove(&parent);
                log::warn!(
                    target: "wgpu_l3::native_popup",
                    "cannot apply IME target {target:?} for missing native host of parent {parent:?}"
                );
            }
            None => {
                self.ime_targets.remove(&parent);
                if let Some(window) = self.windows.get(&parent) {
                    window.set_ime_allowed(false);
                }
            }
        }
    }

    fn apply_ime_target(&self, parent: app_window::Id, target: app_ime::Target) -> bool {
        // Popup HWNDs are nonactivating, so the parent keeps the keyboard/IME
        // context while the declared host owns coordinate conversion and area.
        let Some(parent_window) = self.windows.get(&parent) else {
            return false;
        };
        parent_window.set_ime_allowed(true);

        let host = ime_host(parent, target);
        let Some(window) = self.window_for_ime_host(parent, host) else {
            return false;
        };
        if matches!(host, ImeHost::Popup(_)) {
            window.set_ime_allowed(true);
        }
        window.set_ime_cursor_area(target.area());
        true
    }

    fn window_for_ime_host(
        &self,
        parent: app_window::Id,
        host: ImeHost,
    ) -> Option<&super::window::Window> {
        match host {
            ImeHost::Parent => self.windows.get(&parent),
            ImeHost::Popup(key) => self.popups.get(&key).map(|popup| &popup.window),
        }
    }
}

fn ime_host(parent: app_window::Id, target: app_ime::Target) -> ImeHost {
    match target {
        app_ime::Target::Parent { .. } => ImeHost::Parent,
        app_ime::Target::Popup { id, .. } => ImeHost::Popup(PopupKey::new(parent, id)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{geometry, interaction};

    #[test]
    fn logical_ime_target_names_its_physical_host() {
        let parent = app_window::Id::new(51);
        let id = interaction::Id::new("palette");
        assert_eq!(
            ime_host(
                parent,
                app_ime::Target::Parent {
                    area: geometry::Rect::new(1, 2, 1, 18),
                }
            ),
            ImeHost::Parent
        );
        assert_eq!(
            ime_host(
                parent,
                app_ime::Target::Popup {
                    id,
                    area: geometry::Rect::new(3, 4, 1, 18),
                }
            ),
            ImeHost::Popup(PopupKey::new(parent, id))
        );
    }
}
